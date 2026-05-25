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

use quire_rs::load_repo;

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
                "---\nid: FR-{i:04}\nartifact_type: FR\n---\n# Behavior\n\nbody {i}\n\n## Acceptance\n\n- ok\n"
            ),
        )
        .unwrap();
    }
    root
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
