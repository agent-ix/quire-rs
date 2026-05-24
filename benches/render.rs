//! Render-path benches (Task 014, NFR-001).
//!
//! Measures the per-render cost of `render_by_name` against the
//! self-contained `demo` archetype from the parity suite. Loads the
//! Registry once (outside the timed loop) and re-renders the same
//! input N times inside. NFR-001 target: median <1ms per archetype on
//! baseline hardware.

use std::path::Path;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use quire_rs::{render_by_name, Registry};
use serde_json::json;

fn bench_render_demo_item(c: &mut Criterion) {
    let parent = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("render_parity")
        .join("modules");
    let r = Registry::load_from(&[parent.as_path()]).expect("load registry for bench");

    let inputs = [
        ("small", json!({"id": "DEMO-001", "title": "x"})),
        (
            "with_tags",
            json!({"id": "DEMO-002", "title": "y", "tags": ["a", "b", "c"]}),
        ),
        (
            "with_body",
            json!({"id": "DEMO-003", "title": "z", "body": "long body ".repeat(100)}),
        ),
    ];

    let mut group = c.benchmark_group("render");
    for (label, data) in &inputs {
        group.bench_with_input(BenchmarkId::new("demo-item", label), data, |b, d| {
            b.iter(|| {
                let out = render_by_name(&r, "demo-item", black_box(d)).expect("render");
                black_box(out);
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_render_demo_item);
criterion_main!(benches);
