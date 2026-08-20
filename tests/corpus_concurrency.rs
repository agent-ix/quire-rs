//! Real-thread concurrency check for the parallel walk (FR-024) — the
//! runtime counterpart to the exhaustive loom model in `concurrency.rs`.
//!
//! Threads run `load_repo` and first-touch lazy bodies over shared corpora
//! and must produce byte-identical, path-sorted results (NFR-006) with no
//! data race. This runs in normal `cargo test` for correctness and is the
//! target for the ThreadSanitizer lane (NFR-018, `make sanitize`, in `make
//! hardening` since CR-053) — TSAN instruments the rayon fan-out here for
//! races the loom model proves are absent.
//!
//! This file is the load-bearing control for the FR-025 once-cell, because
//! it is the only place the **real** `OnceLock` runs: loom ships no model
//! of it, so TC-815 mirrors the contract with loom primitives instead.
//! CR-053 widened the coverage accordingly — past 2 threads × 1 document,
//! and over the rayon-forcing shape `python::load_repo` runs.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

use ix_trace_rs::trace;
use quire_rs::{load_repo, Spec};

fn corpus(tag: &str, n: usize) -> PathBuf {
    let mut root = std::env::temp_dir();
    root.push(format!(
        "quire_conc_{tag}_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&root).unwrap();
    for i in 0..n {
        fs::write(
            root.join(format!("FR-{i:04}.md")),
            format!(
                "---\nid: FR-{i:04}\ntype: FR\n---\n# Behavior\n\nbody {i}\n\n## Acceptance\n\n- ok\n"
            ),
        )
        .unwrap();
    }
    root
}

#[trace("TC-816", "FR-025-AC-8")]
// two OS threads first-touch the SAME (CR-047)
// document's lazy body — the real std once-lock, raced for real (this file
// is the NFR-018 TSAN lane target; the loom model of the same contract is
// TC-815 in tests/concurrency.rs). Both racers must receive the identical
// parsed body, zero bodies are parsed before the touch, only the touched
// document is parsed after, and repeated access within a thread returns
// the very same cached value (pointer-identical).
#[test]
fn concurrent_first_touch_parses_once_and_agrees() {
    let root = corpus("first_touch", 8);
    let spec = Spec::from_path(&root);

    // Construction + header-tier queries parsed zero bodies (FR-025-AC-7).
    assert_eq!(spec.len(), 8);
    for d in spec.by_type("FR") {
        assert!(
            !d.body_is_parsed(),
            "{}: body parsed before first touch",
            d.id
        );
    }

    let handles: Vec<_> = (0..2)
        .map(|_| {
            let spec = spec.clone(); // shares the same Arc'd inner
            thread::spawn(move || {
                let doc = spec.by_id("FR-0003").unwrap();
                let first = doc.body();
                // Repeated access returns the same cached value, not a
                // re-parse (external immutability: never a different
                // answer twice).
                assert!(std::ptr::eq(first, doc.body()));
                first.clone()
            })
        })
        .collect();
    let bodies: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // Both racers observed the identical parsed body.
    assert_eq!(bodies[0], bodies[1]);

    // Exactly the touched document is parsed; its 7 siblings stay lazy.
    for d in spec.by_type("FR") {
        assert_eq!(d.body_is_parsed(), d.id == "FR-0003");
    }

    fs::remove_dir_all(&root).ok();
}

#[trace("TC-816", "FR-025-AC-8")]
// the same contract past (CR-047, widened CR-053)
// 2 threads × 1 document. Eight OS threads first-touch SIXTEEN documents,
// each thread starting at a different offset so the racers collide on
// different cells at different moments rather than lining up on one. Every
// thread must observe the identical body for every document, and each
// document must end up parsed exactly once — `body_is_parsed()` is the
// observable that a second init would have to move.
#[test]
fn concurrent_first_touch_over_many_documents_agrees() {
    const DOCS: usize = 16;
    const THREADS: usize = 8;

    let root = corpus("many_first_touch", DOCS);
    let spec = Spec::from_path(&root);
    assert_eq!(spec.len(), DOCS);

    let handles: Vec<_> = (0..THREADS)
        .map(|t| {
            let spec = spec.clone(); // shares the same Arc'd inner
            thread::spawn(move || {
                // Each thread walks the corpus from its own offset.
                (0..DOCS)
                    .map(|i| {
                        let id = format!("FR-{:04}", (i + t * 3) % DOCS);
                        let doc = spec.by_id(&id).unwrap();
                        let first = doc.body();
                        assert!(
                            std::ptr::eq(first, doc.body()),
                            "{id}: repeated access re-parsed"
                        );
                        (id, first.clone())
                    })
                    .collect::<Vec<_>>()
            })
        })
        .collect();

    let per_thread: Vec<Vec<(String, _)>> =
        handles.into_iter().map(|h| h.join().unwrap()).collect();

    // Every thread observed the identical body for every document.
    for t in 1..THREADS {
        for (id, body) in &per_thread[t] {
            let reference = per_thread[0]
                .iter()
                .find(|(other, _)| other == id)
                .map(|(_, b)| b)
                .expect("same corpus in every thread");
            assert_eq!(body, reference, "{id}: racers disagreed on the body");
        }
    }

    // All of them are materialised, and the count is exactly the corpus.
    assert_eq!(
        spec.by_type("FR")
            .iter()
            .filter(|d| d.body_is_parsed())
            .count(),
        DOCS
    );

    fs::remove_dir_all(&root).ok();
}

#[trace("TC-816", "FR-025-AC-8")]
// the shape `python::load_repo` runs — a rayon (CR-053)
// region that forces every lazy body after the walk, with the GIL released.
// The PyO3 binding does exactly this (`load.documents.par_iter().for_each(|d|
// d.body())`), and it is the only place first-touch happens *inside* a
// parallel region; the walk itself never touches a body cell. Covering it in
// Rust puts it on the TSAN lane, which the wheel-only binding suite is not.
#[test]
fn rayon_forced_bodies_match_a_sequential_force() {
    use rayon::prelude::*;

    let root = corpus("rayon_force", 64);
    let spec = Spec::from_path(&root);

    let expected: Vec<_> = {
        let sequential = Spec::from_path(&root);
        sequential
            .by_type("FR")
            .iter()
            .map(|d| d.body().clone())
            .collect()
    };

    let docs = spec.by_type("FR");
    assert!(docs.iter().all(|d| !d.body_is_parsed()));
    docs.par_iter().for_each(|d| {
        d.body();
    });

    let forced: Vec<_> = spec
        .by_type("FR")
        .iter()
        .map(|d| d.body().clone())
        .collect();
    assert_eq!(
        forced, expected,
        "parallel force must land on the same bodies as a sequential one"
    );
    assert!(spec.by_type("FR").iter().all(|d| d.body_is_parsed()));

    fs::remove_dir_all(&root).ok();
}

#[test]
fn concurrent_load_repo_is_consistent() {
    let root = Arc::new(corpus("consistent", 200));

    let handles: Vec<_> = (0..2)
        .map(|_| {
            let root = Arc::clone(&root);
            thread::spawn(move || {
                let load = load_repo(&root);
                let ids: Vec<String> = load.documents.iter().map(|d| d.id.clone()).collect();
                ids
            })
        })
        .collect();

    let results: Vec<Vec<String>> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // Both threads see the full, path-sorted corpus, identically.
    assert_eq!(results[0].len(), 200);
    assert_eq!(results[0], results[1]);

    fs::remove_dir_all(root.as_ref()).ok();
}
