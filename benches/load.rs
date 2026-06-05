//! Loader benches (Task 014, NFR-007).
//!
//! Measures cold-start `Registry::load_from` against the bootstrap
//! `demo` module. NFR-007 target: <100 ms median for the 17-archetype
//! baseline; we don't ship that here, so this bench provides the
//! per-archetype baseline that Task 013 multiplies up.

use std::path::Path;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use quire_rs::Registry;

fn bench_load_demo_module(c: &mut Criterion) {
    let parent = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("modules");
    let mut group = c.benchmark_group("load");
    group.bench_function("registry_load_from_demo", |b| {
        b.iter(|| {
            let r = Registry::load_from(&[black_box(parent.as_path())]).expect("load");
            black_box(r);
        });
    });
    group.finish();
}

criterion_group!(benches, bench_load_demo_module);
criterion_main!(benches);
