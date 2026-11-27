//! Loader benches (Task 014, NFR-007).
//!
//! Measures cold-start `Registry::load_from` against the pinned 28-artifact
//! baseline corpus shared with the FR-013-AC-4 count test.

use std::path::Path;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ix_trace_rs::implements;
use quire_rs::Registry;

#[implements("FR-013-AC-4", "NFR-007-AC-1")]
fn bench_registry_load(c: &mut Criterion) {
    let parent =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/modules/baseline-artifacts");
    let mut group = c.benchmark_group("load");
    group.bench_function("registry_load_from_baseline_artifacts", |b| {
        b.iter(|| {
            let r = Registry::load_from(&[black_box(parent.as_path())]).expect("load");
            black_box(r);
        });
    });
    group.finish();
}

criterion_group!(benches, bench_registry_load);
criterion_main!(benches);
