//! Parse benches (Task 008). Stub: no real benchmarks yet.

use criterion::{criterion_group, criterion_main, Criterion};

fn bench_parse_stub(c: &mut Criterion) {
    c.bench_function("parse_stub", |b| b.iter(|| {}));
}

criterion_group!(benches, bench_parse_stub);
criterion_main!(benches);
