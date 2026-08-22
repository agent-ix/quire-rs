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
            "target_ids": ["TC-001"],
            "line": 9
        }],
        "status_lies": [{
            "reference": "traces-to",
            "document": "tests.md",
            "row_id": "TC-001",
            "status": "✅",
            "target_ids": ["TC-001"],
            "line": 11
        }],
        "no_symbol_rows": [{
            "reference": "traces-to",
            "document": "tests.md",
            "row_id": "TC-002",
            "test_type": "Eval",
            "target_ids": ["TC-002"],
            "line": 12
        }],
        "undeclared_statuses": [{
            "reference": "traces-to",
            "document": "tests.md",
            "row_id": "TC-003",
            "status": "⚠️ scale evidence deferred",
            "line": 13
        }],
        "untracked_symbols": [{
            "path": "src/lib.rs",
            "symbol": "tests::covers_nothing",
            "trace_id": "TC-999",
            "line": 7
        }],
        "shared_trace_ids": [{
            "trace_id": "TC-004",
            "symbols": [
                {"path": "src/lib.rs", "symbol": "tests::covers_it"},
                {"path": "src/lib.rs", "symbol": "tests::also_covers_it"}
            ]
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
            "path": null,
            "value": "CI Gate"
        }],
        "vocabulary_coverage": [{
            "vocabulary": "quality-characteristics",
            "archetype": "NFR",
            "field": "quality_attribute",
            "check": "unowned-quality-characteristic",
            "value": "safety",
            "state": "excused",
            "documents": ["spec.md"]
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
        "excluded_source_files": 3,
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
        "shared_trace_ids",
        "criteria",
        "diagnostics",
        "obligations",
        "implements",
        "excluded_source_files",
        "vocabulary_coverage",
        // CR-093: present whenever the walk found an evidence symbol, so a
        // corpus with no source tree at all omits it.
        "binding_census",
        // CR-094: the engine always emits it, but the contract stays additive
        // so a payload from an engine predating the field still conforms
        // (FR-055-CON-3).
        "metrics",
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

#[trace("TC-947", "FR-055-AC-6")]
// the `implements` optional-key acceptance, pinned in both
// directions against the ENGINE, not only the schema. #203 (CR-083) silently
// widened tc859's optional list with `implements` — a real correction
// (`skip_serializing_if` since CR-080) with no matrix presence (#213,
// SR-010 FND-003). This is the discrete record: an empty `implements`
// serializes with no key at all (FR-050-AC-7 byte-identity for modules that
// have not adopted the marker forms), a populated one carries the key, and
// the published contract accepts both payloads.
#[test]
fn tc947_implements_key_is_omitted_when_empty_and_carried_when_populated() {
    use quire_rs::coverage::{CoverageReport, ImplementsRecord};

    let schema = compile("coverage-v1.schema.json");

    // Engine direction, empty: no `implements` key anywhere in the payload.
    let empty = CoverageReport::default();
    let empty_json: Value = serde_json::from_str(&empty.to_json()).expect("valid JSON");
    assert!(
        !empty_json.as_object().unwrap().contains_key("implements"),
        "an empty `implements` must be omitted, not serialized as `[]`",
    );

    // Engine direction, populated: the key is present and shaped as declared.
    let mut populated = CoverageReport::default();
    populated.implements.push(ImplementsRecord {
        path: "src/lib.rs".to_string(),
        symbol: "compute".to_string(),
        trace_id: "FR-001".to_string(),
        form: "implements-comment".to_string(),
    });
    let populated_json: Value = serde_json::from_str(&populated.to_json()).expect("valid JSON");
    assert_eq!(
        populated_json["implements"][0]["trace_id"],
        json!("FR-001"),
        "a populated `implements` must carry its records",
    );

    // Contract direction: the published schema accepts both engine outputs.
    assert!(
        errors(&schema, &empty_json).is_empty(),
        "the contract must accept a payload omitting `implements`",
    );
    assert!(
        errors(&schema, &populated_json).is_empty(),
        "the contract must accept a payload carrying `implements`",
    );
}

#[trace("TC-957", "FR-050-AC-26")]
// the `line` keys are optional in BOTH directions
// (#210, the CR-083 additive pattern): a record whose line was not recovered
// serializes with no key at all — never `null` — a recovered one carries it,
// and the published contract accepts both payloads, so a consumer written
// against the pre-line schema reads an unchanged shape and a payload from a
// pre-line engine still conforms.
#[test]
fn tc957_line_keys_are_optional_in_both_directions() {
    use quire_rs::coverage::{CoverageReport, UnbackedRow, UntrackedSymbol};

    let schema = compile("coverage-v1.schema.json");

    let mut report = CoverageReport::default();
    report.unbacked_rows.push(UnbackedRow {
        reference: "verification".to_string(),
        document: "FR-001.md".to_string(),
        row_id: Some("FR-001-AC-1".to_string()),
        target_ids: vec!["TC-001".to_string()],
        line: None,
    });
    report.untracked_symbols.push(UntrackedSymbol {
        path: "src/lib.rs".to_string(),
        symbol: "tests::covers_nothing".to_string(),
        trace_id: "TC-999".to_string(),
        line: None,
    });
    let without: Value = serde_json::from_str(&report.to_json()).expect("valid JSON");
    assert!(
        without["unbacked_rows"][0].get("line").is_none()
            && without["untracked_symbols"][0].get("line").is_none(),
        "an unrecovered line must be omitted, not serialized as null",
    );

    report.unbacked_rows[0].line = Some(13);
    report.untracked_symbols[0].line = Some(7);
    let with: Value = serde_json::from_str(&report.to_json()).expect("valid JSON");
    assert_eq!(with["unbacked_rows"][0]["line"], json!(13));
    assert_eq!(with["untracked_symbols"][0]["line"], json!(7));

    assert!(
        errors(&schema, &without).is_empty(),
        "the contract must accept a payload omitting the line keys",
    );
    assert!(
        errors(&schema, &with).is_empty(),
        "the contract must accept a payload carrying them",
    );
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
