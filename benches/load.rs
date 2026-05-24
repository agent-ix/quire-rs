//! Loader benches (Task 006). Stub: no real benchmarks yet.

use criterion::{criterion_group, criterion_main, Criterion};

fn bench_load_stub(c: &mut Criterion) {
    c.bench_function("load_stub", |b| b.iter(|| {}));
}

criterion_group!(benches, bench_load_stub);
criterion_main!(benches);
