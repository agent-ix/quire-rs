//! Canonical schema-error snapshots (NFR-005-AC-3).

use std::collections::BTreeMap;
use std::path::Path;

use ix_trace_rs::trace;
use quire_rs::{validate, Registry};
use serde_json::json;

#[trace("NFR-005-AC-3")]
#[test]
fn schema_violation_snapshot_covers_every_fixture_archetype() {
    let modules = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/modules");
    let mut actual = BTreeMap::new();
    for module in ["bundle", "demo", "req-fixture"] {
        let registry = Registry::load_module(&modules.join(module)).expect("load fixture module");
        for archetype in registry.active_archetypes().filter(|archetype| {
            archetype.frontmatter_schema.is_some() || archetype.data_schema.is_some()
        }) {
            let error = validate(archetype, &json!({})).expect_err("empty object violates schema");
            actual.insert(
                format!("{module}/{}", archetype.name),
                quire_rs::error::format_violation(&error),
            );
        }
    }

    let expected = [
        (
            "bundle/index",
            "SchemaViolation [index]: field <root> — expected required property \"type\"; observed {}",
        ),
        (
            "bundle/note",
            "SchemaViolation [note]: field <root> — expected required property \"id\"; observed {}",
        ),
        (
            "demo/demo-item",
            "SchemaViolation [demo-item]: field <root> — expected required property \"id\"; observed {}",
        ),
        (
            "req-fixture/FR",
            "SchemaViolation [FR]: field <root> — expected required property \"id\"; observed {}",
        ),
        (
            "req-fixture/NFR",
            "SchemaViolation [NFR]: field <root> — expected required property \"id\"; observed {}",
        ),
        (
            "req-fixture/master-requirements",
            "SchemaViolation [master-requirements]: field <root> — expected required property \"type\"; observed {}",
        ),
    ]
    .into_iter()
    .map(|(name, error)| (name.to_string(), error.to_string()))
    .collect();
    assert_eq!(actual, expected, "canonical per-archetype errors changed");
}
