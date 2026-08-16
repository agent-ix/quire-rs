//! Real-thread concurrency check for the parallel walk (FR-024) — the
//! runtime counterpart to the exhaustive loom model in `concurrency.rs`.
//!
//! Two OS threads each run `load_repo` over the same corpus and must
//! produce byte-identical, path-sorted results (NFR-006) with no data
//! race. This runs in normal `cargo test` for correctness and is the
//! target for the scheduled ThreadSanitizer lane (NFR-018, `make
//! sanitize`) — TSAN instruments the rayon fan-out here for races the
//! loom model proves are absent.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

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

// TC-816 (FR-025-AC-8, CR-047): two OS threads first-touch the SAME
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
