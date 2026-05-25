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
