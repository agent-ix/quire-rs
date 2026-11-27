// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-027-AC-8 — whole-spec query latency over a deterministic corpus.
//!
//! Measures point lookup, reverse-edge lookup, and orphan selection over 200
//! documents. Criterion reports per-query latency; the acceptance target is
//! below one millisecond for each operation. Criterion supplies the latency
//! measurement: a hard wall-clock assertion would make this benchmark depend
//! on host load and hardware rather than query behavior.

use std::fs;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use ix_trace_rs::trace;
use quire_rs::corpus::Spec;

const ARTIFACT_COUNT: usize = 200;

fn write_artifact(root: &std::path::Path, id: &str, ty: &str, relationships: &[(&str, &str)]) {
    let mut document = format!("---\nid: {id}\ntype: {ty}\ntitle: {id}\n");
    if !relationships.is_empty() {
        document.push_str("relationships:\n");
        for (edge_type, target) in relationships {
            document.push_str(&format!(
                "  - target: \"{target}\"\n    type: \"{edge_type}\"\n"
            ));
        }
    }
    document.push_str(&format!("---\n\n# {id}\n\nBenchmark fixture.\n"));
    fs::write(root.join(format!("{id}.md")), document).expect("write benchmark artifact");
}

fn benchmark_spec() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("create benchmark corpus");

    write_artifact(root.path(), "FR-000", "FR", &[]);
    for index in 1..40 {
        write_artifact(root.path(), &format!("FR-{index:03}"), "FR", &[]);
    }

    for index in 0..40 {
        let id = format!("NFR-{index:03}");
        let relationships = if index < 20 {
            vec![("references", "FR-000")]
        } else {
            Vec::new()
        };
        write_artifact(root.path(), &id, "NFR", &relationships);
    }

    for index in 0..40 {
        let id = format!("US-{index:03}");
        if index < 20 {
            let target = format!("TC-{index:03}");
            write_artifact(root.path(), &id, "US", &[("implements", target.as_str())]);
        } else {
            write_artifact(root.path(), &id, "US", &[]);
        }
    }

    for index in 0..80 {
        write_artifact(root.path(), &format!("TC-{index:03}"), "TC", &[]);
    }

    root
}

#[trace("FR-027-AC-8")]
fn bench_query_200_artifacts(c: &mut Criterion) {
    let root = benchmark_spec();
    let spec = Spec::from_path(root.path());

    assert_eq!(
        spec.len(),
        ARTIFACT_COUNT,
        "fixture must contain 200 artifacts"
    );
    assert!(
        spec.by_id("FR-000").is_some(),
        "point lookup fixture exists"
    );
    assert_eq!(spec.referencing("FR-000").len(), 20);
    assert_eq!(spec.orphans("US", "implements", Some("TC")).len(), 20);

    let mut group = c.benchmark_group("whole_spec_query_200");
    group.bench_with_input(BenchmarkId::new("by_id", "FR-000"), &spec, |b, spec| {
        b.iter(|| black_box(spec.by_id(black_box("FR-000"))));
    });
    group.bench_with_input(
        BenchmarkId::new("referencing", "FR-000"),
        &spec,
        |b, spec| b.iter(|| black_box(spec.referencing(black_box("FR-000")))),
    );
    group.bench_with_input(
        BenchmarkId::new("orphans", "US-implements-TC"),
        &spec,
        |b, spec| {
            b.iter(|| black_box(spec.orphans(black_box("US"), "implements", Some("TC"))));
        },
    );
    group.finish();
}

criterion_group!(benches, bench_query_200_artifacts);
criterion_main!(benches);
