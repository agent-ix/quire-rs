//! Parse-path benches (Task 014, NFR-002).
//!
//! Measures the cost of `parse_document` on three input sizes:
//! a small fixture, a medium fixture, and a synthetic ~1 MB doc
//! (extrapolating to the NFR-002 5 MB target).

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use quire_rs::parse_document;

const SMALL: &str = "## A\nfoo\n## B\nbar\n";

fn build_medium() -> String {
    let mut s = String::with_capacity(64 * 1024);
    s.push_str("---\nid: bench\n---\n");
    for i in 0..200 {
        s.push_str(&format!("## Section {i}\n"));
        for _ in 0..20 {
            s.push_str("some content line that has a little weight to it.\n");
        }
    }
    s
}

fn build_one_megabyte() -> String {
    let mut s = String::with_capacity(1 << 20);
    while s.len() < (1 << 20) {
        s.push_str("## H\nsome content here\n");
    }
    s
}

fn bench_parse(c: &mut Criterion) {
    let medium = build_medium();
    let big = build_one_megabyte();
    let mut group = c.benchmark_group("parse");
    group.bench_function("small", |b| {
        b.iter(|| {
            let d = parse_document(black_box(SMALL));
            black_box(d);
        });
    });
    group.bench_function("medium_64kb", |b| {
        b.iter(|| {
            let d = parse_document(black_box(&medium));
            black_box(d);
        });
    });
    group.bench_function("one_megabyte", |b| {
        b.iter(|| {
            let d = parse_document(black_box(&big));
            black_box(d);
        });
    });
    group.finish();
}

criterion_group!(benches, bench_parse);
criterion_main!(benches);
