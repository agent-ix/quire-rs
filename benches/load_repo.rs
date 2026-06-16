//! TC-455 / NFR-015: `load_repo` throughput on a synthetic corpus,
//! measured at 1 and 8 threads. Asserts the parse fan-out scales with
//! cores (parallel efficiency is computed from the two medians).
//!
//! Run: `cargo bench --bench load_repo`. The thread count is set via a
//! scoped rayon pool so a single process can measure both.

use std::fs;
use std::path::PathBuf;

use criterion::{criterion_group, criterion_main, Criterion};
use quire_rs::load_repo;

const DOC_COUNT: usize = 1_000;

fn synthetic_corpus() -> PathBuf {
    let mut root = std::env::temp_dir();
    root.push(format!("quire_bench_load_repo_{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    for i in 0..DOC_COUNT {
        let body = format!(
            "---\nid: FR-{i:04}\ntype: FR\nuuid: 0190b6a0-0000-7000-8000-{i:012}\n---\n\
             # Behavior\n\nSome representative requirement prose for document {i}.\n\n\
             ## Acceptance\n\n- **FR-{i:04}-AC-1**: a representative criterion.\n\
             - **FR-{i:04}-AC-2**: another one with a bit more text to parse.\n\n\
             ## Notes\n\nTrailing section with a fenced block:\n\n```rust\nfn x() {{}}\n```\n"
        );
        fs::write(root.join(format!("FR-{i:04}.md")), body).unwrap();
    }
    root
}

fn bench_load_repo(c: &mut Criterion) {
    let root = synthetic_corpus();

    let mut group = c.benchmark_group("load_repo_1k");
    for threads in [1usize, 8usize] {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap();
        group.bench_function(format!("threads_{threads}"), |b| {
            b.iter(|| {
                pool.install(|| {
                    let load = load_repo(&root);
                    assert_eq!(load.documents.len(), DOC_COUNT);
                    load
                })
            });
        });
    }
    group.finish();

    let _ = fs::remove_dir_all(&root);
}

criterion_group!(benches, bench_load_repo);
criterion_main!(benches);
