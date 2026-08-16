//! NFR-017 — concurrency permutation for the parallel-walk collect.
//!
//! `load_repo`'s parse fan-out (FR-024) is *data-parallel*: each task
//! produces an **owned** result, and the results are collected then
//! sorted — there is no shared mutable state, no `Mutex`/`RwLock`/
//! `Atomic` in first-party code (FR-024-AC-9). This test proves that
//! invariant under loom's exhaustive interleaving.
//!
//! loom cannot model rayon's thread pool, so we model the *pattern*
//! abstractly: N threads each compute an owned `(key, value)`, joined
//! into a `Vec`, then `sort_by_key`. The claim under test: for every
//! interleaving, (a) there is no data race, and (b) the sorted output
//! is identical regardless of completion order.
//!
//! Run on the scheduled lane: `make loom` (sets `RUSTFLAGS=--cfg loom`).
//! Without the flag this file compiles to an empty test binary.

#![cfg(loom)]

use loom::sync::atomic::{AtomicUsize, Ordering};
use loom::sync::{Arc, Mutex};
use loom::thread;

#[test]
fn parallel_collect_is_race_free_and_order_independent() {
    loom::model(|| {
        // Two "parse tasks", each returning an owned (path-key, id).
        // Distinct owned values, no shared mutable state — exactly the
        // shape of `files.par_iter().map(parse_one).collect()`.
        let h_b = thread::spawn(|| (2usize, "FR-002"));
        let h_a = thread::spawn(|| (1usize, "FR-001"));

        let mut collected = vec![h_b.join().unwrap(), h_a.join().unwrap()];

        // The post-parallel deterministic sort (NFR-006).
        collected.sort_by_key(|(key, _)| *key);

        // Independent of which thread finished first, the result is the
        // same path-sorted vector.
        assert_eq!(collected, vec![(1, "FR-001"), (2, "FR-002")]);
    });
}

// TC-815, NFR-017-AC-4
//
// The FR-025 lazy body cache (CR-047): two threads first-touch the SAME
// document's body cell, and the contract is exactly-once init with every
// racer observing the one stored value. The production primitive is std
// `OnceLock<QuireDocument>` (src/corpus/body_cache.rs), which loom cannot
// instrument — loom ships no OnceLock model and cannot see into std's
// internals — so this permutation models the once-cell **contract** with
// loom primitives: a `Mutex<Option<_>>` cell plus an atomic init counter,
// both threads racing `get_or_insert_with`. The real primitive is raced
// for real under TSAN in tests/corpus_concurrency.rs (TC-816, NFR-018).
#[test]
fn lazy_body_first_touch_parses_once_and_agrees() {
    loom::model(|| {
        // The cell starts empty (the walk parses no body, FR-025-AC-7).
        let cell: Arc<Mutex<Option<&'static str>>> = Arc::new(Mutex::new(None));
        let inits = Arc::new(AtomicUsize::new(0));

        // Both first-touchers run the same deterministic "parse" — the
        // body parse is a pure function of the stored text (NFR-006).
        let parse = |inits: &AtomicUsize| -> &'static str {
            inits.fetch_add(1, Ordering::SeqCst);
            "parsed-body"
        };

        let spawned = {
            let cell = Arc::clone(&cell);
            let inits = Arc::clone(&inits);
            thread::spawn(move || *cell.lock().unwrap().get_or_insert_with(|| parse(&inits)))
        };
        let got_main = *cell.lock().unwrap().get_or_insert_with(|| parse(&inits));
        let got_spawned = spawned.join().unwrap();

        // Every interleaving: racers agree on the one stored value, and
        // the init ran exactly once.
        assert_eq!(got_main, got_spawned);
        assert_eq!(got_main, "parsed-body");
        assert_eq!(inits.load(Ordering::SeqCst), 1);
    });
}
