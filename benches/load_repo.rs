//! TC-455 / NFR-015: `load_repo` throughput on a synthetic corpus,
//! measured at 1 and 8 threads. Asserts the parse fan-out scales with
//! cores (parallel efficiency is computed from the two medians).
//!
//! Since CR-047 the walk parses headers only, so `load_repo_1k` measures
//! the header-tier walk; `load_repo_1k_bodies` additionally forces every
//! lazy body so NFR-015's full-parse scaling claim stays evidenced.
//!
//! Run: `cargo bench --bench load_repo`. The thread count is set via a
//! scoped rayon pool so a single process can measure both.

use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use criterion::{Criterion, criterion_group, criterion_main};
use ix_trace_rs::trace;
use quire_rs::load_repo;
use rayon::prelude::*;

const DOC_COUNT: usize = 1_000;
const SPEEDUP_SAMPLES: usize = 11;

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

#[trace("FR-024-AC-8")]
fn bench_load_repo(c: &mut Criterion) {
    let root = synthetic_corpus();

    let pools = [1usize, 8usize].map(|threads| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap()
    });
    let mut medians = Vec::with_capacity(pools.len());
    for pool in &pools {
        let mut samples = Vec::with_capacity(SPEEDUP_SAMPLES);
        for _ in 0..SPEEDUP_SAMPLES {
            let start = Instant::now();
            let load = pool.install(|| load_repo(&root));
            samples.push(start.elapsed());
            assert_eq!(load.documents.len(), DOC_COUNT);
        }
        samples.sort_unstable();
        medians.push(samples[SPEEDUP_SAMPLES / 2]);
    }
    let speedup = medians[0].as_secs_f64() / medians[1].as_secs_f64();
    assert!(medians.iter().all(|median| !median.is_zero()));
    assert!(speedup.is_finite() && speedup > 0.0);
    eprintln!(
        "load_repo {DOC_COUNT}-document median speedup: 1-thread={:?}, 8-thread={:?}, ratio={speedup:.3}x",
        medians[0], medians[1]
    );

    // Header-tier walk only (the CR-047 lazy-body default).
    let mut group = c.benchmark_group("load_repo_1k");
    for (threads, pool) in [1usize, 8usize].into_iter().zip(&pools) {
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

    // Walk + every body forced (the pre-CR-047 full-parse quantity).
    let mut group = c.benchmark_group("load_repo_1k_bodies");
    for (threads, pool) in [1usize, 8usize].into_iter().zip(&pools) {
        group.bench_function(format!("threads_{threads}"), |b| {
            b.iter(|| {
                pool.install(|| {
                    let load = load_repo(&root);
                    assert_eq!(load.documents.len(), DOC_COUNT);
                    load.documents.par_iter().for_each(|d| {
                        d.body();
                    });
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
