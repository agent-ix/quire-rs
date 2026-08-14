//! Parse-path benches (Task 014, NFR-002).
//!
//! Measures the cost of `parse_document` on three input sizes:
//! a small fixture, a medium fixture, and a synthetic ~1 MB doc
//! (extrapolating to the NFR-002 5 MB target).

use std::path::Path;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use quire_rs::{parse_document, validate_document, Registry};

const SMALL: &str = "## A\nfoo\n## B\nbar\n";

/// A typical authored FR artifact (< 32 KB): frontmatter + the four
/// required sections + an AC table — the NFR-002-AC-4 envelope.
const FR_ARTIFACT: &str = "---\n\
id: FR-901\n\
title: \"A conformant requirement\"\n\
type: FR\n\
---\n\
# [FR-901] A conformant requirement\n\
\n\
## Description\n\
The system SHALL preserve byte-exact content across a parse round-trip.\n\
\n\
## Specification\n\
On parse, the engine retains every byte of the section body verbatim.\n\
\n\
## Acceptance Criteria\n\
\n\
| ID | Criteria | Verification |\n\
|----|----------|--------------|\n\
| FR-901-AC-1 | Round-trip is byte-identical | Integration Test |\n\
\n\
## Dependencies\n\
\n\
- **Upstream**: none\n\
- **Downstream**: none\n";

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

/// NFR-002-AC-4 / TC-577: `validate_document` on a typical FR-sized
/// artifact against a **warm** Registry (load cost excluded). Target:
/// median below 1 ms on the canonical baseline runner.
// TC-577 (NFR-007-AC-1): the validate-document benchmark the CI baseline compares against.
fn bench_validate_document(c: &mut Criterion) {
    let module = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/modules/iso");
    let registry = Registry::load_module(&module).expect("load iso module");
    let arch = registry.archetype("FR").expect("FR archetype loaded");

    let mut group = c.benchmark_group("validate_document");
    group.bench_function("fr_artifact", |b| {
        b.iter(|| {
            let r = validate_document(black_box(arch), black_box(FR_ARTIFACT));
            black_box(r);
        });
    });
    group.finish();
}

criterion_group!(benches, bench_parse, bench_validate_document);
criterion_main!(benches);
