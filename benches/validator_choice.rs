//! Validator-choice benches (Task 026 ADR). Stub: no real benchmarks yet.

use criterion::{criterion_group, criterion_main, Criterion};

fn bench_validator_stub(c: &mut Criterion) {
    c.bench_function("validator_stub", |b| b.iter(|| {}));
}

criterion_group!(benches, bench_validator_stub);
criterion_main!(benches);
