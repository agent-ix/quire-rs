//! Validator-choice bench (Task 026 ADR placeholder).
//!
//! Currently only measures the chosen `jsonschema` crate path. Task 026
//! will extend this to compare against `boon` or another candidate
//! and produce the ADR — for now, this provides the baseline number
//! we'll compare against.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jsonschema::JSONSchema;
use serde_json::json;

fn bench_jsonschema_validate(c: &mut Criterion) {
    let schema = json!({
        "type": "object",
        "required": ["id", "title"],
        "additionalProperties": false,
        "properties": {
            "id": {"type": "string", "pattern": "^FR-[0-9]+$"},
            "title": {"type": "string", "minLength": 1, "maxLength": 200},
            "tags": {"type": "array", "items": {"type": "string"}}
        }
    });
    let validator = JSONSchema::options().compile(&schema).expect("compile");
    let good = json!({"id": "FR-001", "title": "Hello", "tags": ["a", "b"]});
    let mut group = c.benchmark_group("validator");
    group.bench_function("jsonschema_validate_good", |b| {
        b.iter(|| {
            let r = validator.is_valid(black_box(&good));
            black_box(r);
        });
    });
    group.finish();
}

criterion_group!(benches, bench_jsonschema_validate);
criterion_main!(benches);
