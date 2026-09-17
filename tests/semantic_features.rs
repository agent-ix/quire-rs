//! FR-075 `Features` table: an interface's fields and operations as one
//! ordered sequence (TC-1874, TC-1875).

use std::path::PathBuf;

use ix_trace_rs::trace;
use jsonschema::JSONSchema;
use quire_rs::semantic::python_entry::extract_semantic_json;
use quire_rs::semantic::{SemanticExtraction, SEMANTIC_V1_SCHEMA};
use quire_rs::Registry;
use serde_json::{json, Value};

const PATH: &str = "spec/architecture/quant_codec.md";

/// The interface locators: an optional Properties section, the Operations
/// section, and the `Features` table.
fn body_extraction() -> Value {
    json!({ "yield_pattern": { "match": {
        "features": {
            "from": "table_row",
            "under_section": "Features",
            "required": true,
            "assert": { "columns": ["Feature", "Kind"], "min_rows": 1 },
        },
    } } })
}

/// Extract `md`; the record must validate against semantic-v1.
fn extract(md: &str) -> (SemanticExtraction, Value) {
    let request = json!({
        "markdown": md,
        "module": {
            "contractVersion": "1.0.0",
            "semanticCore": "0.1.0",
            "package": "agent-ix/spec-objects-architecture",
            "exports": ["interface"],
        },
        "path": PATH,
        "sourceIdentity": "ix://agent-ix/shop/spec",
        "bundle": { "package": "agent-ix/shop" },
        "bodyExtraction": body_extraction(),
    });
    let record = extract_semantic_json(&request).unwrap();
    let value = serde_json::to_value(&record).unwrap();
    let schema = JSONSchema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .compile(&serde_json::from_str::<Value>(SEMANTIC_V1_SCHEMA).unwrap())
        .unwrap();
    let errors: Vec<String> = match schema.validate(&value) {
        Ok(()) => Vec::new(),
        Err(errs) => errs
            .map(|e| format!("{e} at {}", e.instance_path))
            .collect(),
    };
    assert!(errors.is_empty(), "{errors:?}\n{value:#}");
    (record, value)
}

fn line_of(md: &str, needle: &str) -> u64 {
    md.split('\n')
        .position(|l| l.contains(needle))
        .unwrap_or_else(|| panic!("{needle:?} not in document")) as u64
        + 1
}

/// Whether the record carries the error `code` at `line`, and no `model`.
fn refused(record: &Value, code: &str, line: u64) -> bool {
    let at =
        record["diagnostics"].as_array().unwrap().iter().any(|d| {
            d["code"] == code && d["severity"] == "error" && d["line"].as_u64() == Some(line)
        });
    at && record.get("model").is_none() && record["availability"]["model"]["state"] == "unavailable"
}

/// An interface with one field and two operations, and `features` as the
/// body of its `## Features` table.
fn interface(features: &str) -> String {
    format!(
        "---\nid: quant_codec\ntitle: QuantCodec\ntype: interface\nobject: interface\n---\n# [quant_codec] QuantCodec\n\n## Properties\n\n| Field | Type | Multiplicity | Constraints |\n|---|---|---|---|\n| codec_kind | String | 1..1 | |\n\n## Operations\n\n### prepare_ip_query\n\nReturns: Bytes[1..1]\n\n### score_ip_batch\n\nReturns: Bytes[1..1]\n\n## Features\n\n| Feature | Kind |\n|---|---|\n{features}"
    )
}

const ORDER: &str =
    "| prepare_ip_query | operation |\n| codec_kind | field |\n| score_ip_batch | operation |\n";

#[trace("TC-1874", "FR-075-AC-14")]
#[test]
fn features_table_yields_feature_order_in_row_order() {
    let md = interface(ORDER);
    let (record, value) = extract(&md);
    let order = value["model"]["featureOrder"].as_array().unwrap();
    let rows: Vec<(&str, &str)> = order
        .iter()
        .map(|e| (e["name"].as_str().unwrap(), e["kind"].as_str().unwrap()))
        .collect();
    assert_eq!(
        rows,
        [
            ("prepare_ip_query", "operation"),
            ("codec_kind", "field"),
            ("score_ip_batch", "operation"),
        ],
        "{value:#}"
    );
    assert_eq!(
        order[1]["sourceSpan"]["startLine"],
        line_of(&md, "| codec_kind | field")
    );
    assert_eq!(value["availability"]["model"]["state"], "available");

    // The declaration record carries `featureOrder` as the ordered names,
    // and satisfies spec-objects-architecture's `Interface.json`.
    let declaration = record.declaration_record().unwrap();
    assert_eq!(
        declaration["featureOrder"],
        json!(["prepare_ip_query", "codec_kind", "score_ip_batch"])
    );
    let module = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/semantic/spec-objects-architecture");
    let registry = Registry::load_module(&module).unwrap();
    let validator = registry
        .archetype("interface")
        .and_then(|a| a.data_validator())
        .unwrap();
    let violations: Vec<String> = match validator.validate(&declaration) {
        Ok(()) => Vec::new(),
        Err(errs) => errs
            .map(|e| format!("{e} at {}", e.instance_path))
            .collect(),
    };
    assert!(violations.is_empty(), "{violations:?}\n{declaration:#}");
}

#[trace("TC-1875", "FR-075-AC-15")]
#[test]
fn features_table_refuses_rows_that_disagree_with_the_declarations() {
    // A row naming no declared feature.
    let md = interface(&format!("{ORDER}| rescore | operation |\n"));
    let (_, value) = extract(&md);
    assert!(
        refused(
            &value,
            "semantic.unknown-feature",
            line_of(&md, "| rescore")
        ),
        "{value:#}"
    );

    // A row whose kind is not the declared kind.
    let md = interface(&ORDER.replace("codec_kind | field", "codec_kind | operation"));
    let (_, value) = extract(&md);
    assert!(
        refused(
            &value,
            "semantic.feature-kind-mismatch",
            line_of(&md, "| codec_kind | operation")
        ),
        "{value:#}"
    );

    // An unknown kind.
    let md = interface(&ORDER.replace("codec_kind | field", "codec_kind | method"));
    let (_, value) = extract(&md);
    assert!(
        refused(
            &value,
            "semantic.invalid-model-cell",
            line_of(&md, "| codec_kind | method")
        ),
        "{value:#}"
    );

    // A duplicate row.
    let md = interface(&format!("{ORDER}| codec_kind | field |\n"));
    let (_, value) = extract(&md);
    let second = md
        .split('\n')
        .enumerate()
        .filter(|(_, l)| l.contains("| codec_kind | field"))
        .map(|(i, _)| i as u64 + 1)
        .nth(1)
        .unwrap();
    assert!(
        refused(&value, "semantic.duplicate-model-entry", second),
        "{value:#}"
    );

    // A declared feature with no row, located at the table header.
    let md = interface("| prepare_ip_query | operation |\n| codec_kind | field |\n");
    let (_, value) = extract(&md);
    assert!(
        refused(
            &value,
            "semantic.missing-feature",
            line_of(&md, "| Feature | Kind")
        ),
        "{value:#}"
    );
}
