// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-002-AC-5 — per-call patch cost for a typical four-kilobyte artifact.
//!
//! The demo archetype is compiled before timing. The measured call only deep-merges and
//! validates the resulting JSON value.

use std::{
    path::Path,
    time::{Duration, Instant},
};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ix_trace_rs::trace;
use quire_rs::{apply_patch, Registry};
use serde_json::{json, Value};

const MEDIAN_BATCHES: usize = 21;
const CALLS_PER_BATCH: u32 = 1_000;

fn assert_median_below_100_micros(
    archetype: &quire_rs::CompiledArchetype,
    current: &Value,
    patch: &Value,
) -> Duration {
    let mut samples = Vec::with_capacity(MEDIAN_BATCHES);
    for _ in 0..MEDIAN_BATCHES {
        let start = Instant::now();
        for _ in 0..CALLS_PER_BATCH {
            let merged = apply_patch(black_box(archetype), black_box(current), black_box(patch))
                .expect("the benchmark patch preserves the demo schema");
            black_box(merged);
        }
        samples.push(start.elapsed() / CALLS_PER_BATCH);
    }
    samples.sort_unstable();
    let median = samples[MEDIAN_BATCHES / 2];
    assert!(
        median < Duration::from_micros(100),
        "apply_patch median {median:?} is not below the 100 microsecond acceptance limit"
    );
    eprintln!("apply_patch 4 KB acceptance median: {median:?} (< 100 µs)");
    median
}

#[trace("FR-002-AC-5")]
fn bench_apply_patch_4kb_artifact(c: &mut Criterion) {
    let modules = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("modules");
    let registry = Registry::load_from(&[modules.as_path()]).expect("load demo archetype");
    let archetype = registry
        .archetype("demo-item")
        .expect("demo-item archetype is registered");
    let current = json!({
        "id": "DEMO-1",
        "title": "Original title",
        "body": "x".repeat(4_096),
    });
    let patch = json!({"title": "Updated title"});
    assert_eq!(current["body"].as_str().expect("body").len(), 4_096);

    assert_median_below_100_micros(archetype, &current, &patch);

    c.bench_function("apply_patch_4kb_artifact", |b| {
        b.iter(|| {
            black_box(
                apply_patch(black_box(archetype), black_box(&current), black_box(&patch))
                    .expect("the benchmark patch preserves the demo schema"),
            );
        });
    });
}

criterion_group!(benches, bench_apply_patch_4kb_artifact);
criterion_main!(benches);
