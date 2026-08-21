//! FR-055 — the published JSON output contract (TC-854..TC-860).
//!
//! The schemas are hand-authored artifacts under `schemas/output/`. These tests
//! are what make them a contract rather than documentation: a payload change
//! that the schema does not describe fails here, in the same run that the
//! CR-057 byte-golden fails, so the two are reviewed together.

use std::fs;
use std::path::{Path, PathBuf};

use ix_trace_rs::trace;
use jsonschema::JSONSchema;
use serde_json::{json, Value};

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn schema_value(name: &str) -> Value {
    let path = repo().join("schemas").join("output").join(name);
    serde_json::from_str(&fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!("reading published schema {}: {e}", path.display());
    }))
    .expect("schema is valid JSON")
}

fn compile(name: &str) -> JSONSchema {
    let value = schema_value(name);
    JSONSchema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .compile(&value)
        .unwrap_or_else(|e| panic!("compiling {name}: {e}"))
}

fn errors(schema: &JSONSchema, instance: &Value) -> Vec<String> {
    match schema.validate(instance) {
        Ok(()) => Vec::new(),
        Err(errs) => errs.map(|e| format!("{}: {e}", e.instance_path)).collect(),
    }
}

fn baseline() -> Value {
    let path = repo()
        .join("tests")
        .join("fixtures")
        .join("coverage_baseline")
        .join("expected.json");
    serde_json::from_str(&fs::read_to_string(&path).expect("baseline present"))
        .expect("baseline is valid JSON")
}

#[trace("TC-854", "FR-055-AC-1")]
// the artifacts are themselves valid, and self-identify.
#[test]
fn tc854_published_schemas_are_valid_and_versioned() {
    for name in ["coverage-v1.schema.json", "properties-v1.schema.json"] {
        let value = schema_value(name);
        // Compiling under the 2020-12 draft is the meta-validation: a schema
        // with a malformed keyword fails here rather than silently accepting
        // every payload.
        let _ = compile(name);
        let id = value["$id"].as_str().unwrap_or_default();
        assert!(
            id.ends_with(name),
            "{name} must self-identify by its versioned filename, got `{id}`",
        );
        assert_eq!(
            value["$schema"].as_str(),
            Some("https://json-schema.org/draft/2020-12/schema"),
            "{name} must declare the 2020-12 draft",
        );
    }
}

#[trace("TC-855", "FR-055-AC-2")]
// the CR-057 byte-golden corpus is also the contract
// corpus — one input, two gates, reviewed in one diff.
#[test]
fn tc855_coverage_baseline_conforms() {
    let schema = compile("coverage-v1.schema.json");
    let errs = errors(&schema, &baseline());
    assert!(
        errs.is_empty(),
        "the checked-in baseline violates the published contract:\n{errs:#?}"
    );
}

#[trace("TC-856", "FR-055-AC-3")]
// the optional keys are covered by a payload that
// actually carries them, not only by one that omits them — the way an optional
// field quietly stops being described.
#[test]
fn tc856_payload_carrying_every_optional_key_conforms() {
    let schema = compile("coverage-v1.schema.json");
    let full = json!({
        "unbacked_rows": [{
            "reference": "verification",
            "document": "FR-001.md",
            "row_id": "FR-001-AC-1",
            "target_ids": ["TC-001"]
        }],
        "status_lies": [{
            "reference": "traces-to",
            "document": "tests.md",
            "row_id": "TC-001",
            "status": "✅",
            "target_ids": ["TC-001"]
        }],
        "no_symbol_rows": [{
            "reference": "traces-to",
            "document": "tests.md",
            "row_id": "TC-002",
            "test_type": "Eval",
            "target_ids": ["TC-002"]
        }],
        "undeclared_statuses": [{
            "reference": "traces-to",
            "document": "tests.md",
            "row_id": "TC-003",
            "status": "⚠️ scale evidence deferred"
        }],
        "untracked_symbols": [{
            "path": "src/lib.rs",
            "symbol": "tests::covers_nothing",
            "trace_id": "TC-999"
        }],
        "groups": [{"document": "tests.md", "target": "test-case", "backed": 1, "total": 2}],
        "criteria": [{
            "document": "FR-001.md",
            "archetype": "FR",
            "criteria": 2,
            "property_shaped": 1,
            "by_property": {"universal": 1, "example": 1}
        }],
        "diagnostics": [{
            "declaration": "test-case",
            "reason": "archetype-matches-nothing",
            "message": "no document of archetype TestMatrix",
            "path": null
        }],
        "obligations": [{
            "source": "acceptance-criterion",
            "id": "FR-001-AC-1",
            "document": "FR-001.md",
            "statement": "The system shall do it.",
            "statement_hash": "5b3f1a2c4d6e8f0a1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e5f607182",
            "method": "Test",
            "parameters": {"threshold": "< 8ms"},
            "criticality": "P1"
        }],
        "totals": {"backed": 1, "total": 2, "criteria": 2, "property_shaped": 1}
    });
    let errs = errors(&schema, &full);
    assert!(errs.is_empty(), "{errs:#?}");
}

#[trace("TC-857", "FR-055-AC-4")]
// every criterion the engine actually emits conforms.
#[test]
fn tc857_emitted_criteria_conform() {
    let schema_doc = schema_value("properties-v1.schema.json");
    // Validate against the `Criterion` definition directly: this crate emits
    // records, while the envelope around them is assembled by `quire-cli`.
    let mut criterion_schema = schema_doc["$defs"]["Criterion"].clone();
    criterion_schema["$defs"] = schema_doc["$defs"].clone();
    let compiled = JSONSchema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .compile(&criterion_schema)
        .expect("criterion definition compiles");

    let module = repo()
        .join("tests")
        .join("fixtures")
        .join("traceability")
        .join("obligations");
    let registry = quire_rs::Registry::load_module(&module).expect("load module");
    let archetype = registry.archetype("FR").expect("FR archetype");
    let doc = "---\nid: FR-001\ntype: FR\ntitle: A requirement\n---\n\n\
               ## Acceptance Criteria\n\n\
               | ID | Criteria | Verification | Priority |\n\
               |----|----------|--------------|----------|\n\
               | FR-001-AC-1 | Every finding absent from the merged map defaults to warning. | Test (TC-001) | P1 |\n\
               | FR-001-AC-2 | Logging in with a magic link lands on the dashboard. | Demonstration | P3 |\n";
    let records = quire_rs::classify_document_criteria(&registry, archetype, doc, None);
    assert!(!records.is_empty(), "the fixture must produce records");

    for record in &records {
        let value = record_json(record);
        let errs = errors(&compiled, &value);
        assert!(
            errs.is_empty(),
            "an emitted criterion violates the published contract:\n{errs:#?}\n{value:#?}",
        );
    }
}

/// Mirror of `quire-cli`'s `record_json`. Duplicated deliberately and narrowly:
/// the envelope lives in the CLI, and threading a serializer into the engine to
/// avoid ten lines here would put the CLI's presentation choices in the engine.
/// The CLI's own conformance test is the authoritative one for the envelope.
fn record_json(r: &quire_rs::AcClassification) -> Value {
    let span = |s: Option<&quire_rs::PropertySpan>| match s {
        None => Value::Null,
        Some(s) => json!({"start": s.start, "end": s.end, "text": s.text}),
    };
    let obligation = match r.obligation.as_ref() {
        None => Value::Null,
        Some(o) => {
            let mut out = json!({
                "source": o.source,
                "statement_hash": o.statement_hash,
                "method": o.method,
                "criticality": o.criticality,
            });
            if !o.parameters.is_empty() {
                out["parameters"] = json!(o.parameters);
            }
            out
        }
    };
    json!({
        "row_id": r.row_id,
        "statement": r.statement,
        "line": r.line,
        "shape": r.shape.as_str(),
        "property": r.property.as_str(),
        "extractable": r.extractable,
        "extraction": r.extraction.as_str(),
        "domain": span(r.domain.as_ref()),
        "precondition": span(r.precondition.as_ref()),
        "oracle": span(r.oracle.as_ref()),
        "signals": r.signals,
        "obligation": obligation,
    })
}

#[trace("TC-858", "FR-055-AC-5")]
// `additionalProperties: false` holds at depth, not only
// at the root — the usual place a hand-authored schema is accidentally open.
#[test]
fn tc858_added_fields_are_rejected_at_every_level() {
    let schema = compile("coverage-v1.schema.json");

    let mut root = baseline();
    root["surprise"] = json!("added at the root");
    assert!(!errors(&schema, &root).is_empty(), "root is open");

    let mut nested = baseline();
    nested["totals"]["surprise"] = json!(1);
    assert!(
        !errors(&schema, &nested).is_empty(),
        "a nested object accepted an unknown key, so the contract is open where it matters",
    );

    let mut in_array = baseline();
    if let Some(first) = in_array["groups"]
        .as_array_mut()
        .and_then(|a| a.first_mut())
    {
        first["surprise"] = json!(true);
        assert!(
            !errors(&schema, &in_array).is_empty(),
            "an array item accepted an unknown key",
        );
    }
}

#[trace("TC-859", "FR-055-AC-6")]
// the optional/required split matches the engine's
// skip-when-empty behaviour, in both directions.
#[test]
fn tc859_optional_and_required_split_matches_the_engine() {
    let schema = compile("coverage-v1.schema.json");

    // Optional: the engine omits these entirely when empty.
    // `implements` was absent from this list until CR-083 — it has carried
    // `skip_serializing_if` since CR-080, so the list disagreed with the engine
    // in the one direction this test exists to catch.
    for optional in [
        "no_symbol_rows",
        "undeclared_statuses",
        "criteria",
        "diagnostics",
        "obligations",
        "implements",
    ] {
        let mut payload = baseline();
        payload.as_object_mut().unwrap().remove(optional);
        assert!(
            errors(&schema, &payload).is_empty(),
            "`{optional}` must be optional — the engine omits it when empty",
        );
    }

    // Required: the engine always emits these, so their absence is a defect.
    for required in [
        "unbacked_rows",
        "status_lies",
        "untracked_symbols",
        "groups",
        "totals",
    ] {
        let mut payload = baseline();
        payload.as_object_mut().unwrap().remove(required);
        assert!(
            !errors(&schema, &payload).is_empty(),
            "`{required}` must be required — the engine always emits it",
        );
    }
}

#[trace("TC-860", "FR-055-AC-7")]
// the contract is carried by the artifact alone.
#[test]
fn tc860_no_version_key_in_the_payload_and_no_schemars() {
    let payload = baseline();
    let object = payload.as_object().expect("object");
    for banned in ["version", "$schema", "schema_version"] {
        assert!(
            !object.contains_key(banned),
            "FR-008-AC-5: the payload must carry no `{banned}` key; versioning lives in the schema artifact",
        );
    }

    // CON-1: the schemas are authored, so the generator must stay out of the
    // dependency graph. Asserted here as well as in the audit script, because a
    // test that runs on every `cargo test` is harder to skip than a shell gate.
    let lock = fs::read_to_string(repo().join("Cargo.lock")).expect("Cargo.lock");
    assert!(
        !lock.lines().any(|l| l.trim() == "name = \"schemars\""),
        "`schemars` entered the dependency graph; the output contract is authored, not derived",
    );
    assert!(Path::new(&repo().join("schemas/output")).is_dir());
}
