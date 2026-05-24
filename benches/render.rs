//! Render benches (Task 014). Stub: no real benchmarks yet.

use criterion::{criterion_group, criterion_main, Criterion};

fn bench_render_stub(c: &mut Criterion) {
    c.bench_function("render_stub", |b| b.iter(|| {}));
}

criterion_group!(benches, bench_render_stub);
criterion_main!(benches);
