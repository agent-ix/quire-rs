//! FR-072 surface (TC-1630, TC-1631, TC-1632, TC-1634, TC-1637, TC-1638,
//! TC-1644 Rust half, TC-1650). Plan-003 Task-020.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use ix_trace_rs::trace;
use jsonschema::Validator;
use quire_rs::semantic::python_entry::extract_semantic_json;
use quire_rs::semantic::{compile_module_schema, AvailabilityState, SEMANTIC_V1_SCHEMA};
use quire_rs::{extract_filament_core, FilamentExtractionInput, Registry};
use serde_json::{json, Value};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_json(rel: &str) -> Value {
    serde_json::from_slice(&fs::read(root().join(rel)).unwrap()).unwrap()
}

fn schema() -> Validator {
    compile_module_schema(
        &serde_json::from_str::<Value>(SEMANTIC_V1_SCHEMA).unwrap(),
        &|_| None,
        "https://agent-ix.github.io/quire-rs/schemas/output/",
    )
    .unwrap()
}

/// `cases.expected.json` records.
fn expected_records() -> Value {
    read_json("tests/fixtures/semantic/cases.expected.json")
}

fn cases() -> Vec<Value> {
    read_json("tests/fixtures/semantic/cases.json")["cases"]
        .as_array()
        .unwrap()
        .clone()
}

fn write_or_compare(rel: &str, actual: &str) {
    let path = root().join(rel);
    if std::env::var_os("UPDATE_SEMANTIC_BASELINES").is_some() {
        fs::write(&path, actual).unwrap();
        return;
    }
    let expected = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel}: {e}"));
    assert!(
        expected == actual,
        "{rel} differs from the checked-in expectation"
    );
}

/// Assert every field a case's `expect` records against its record.
fn assert_case(name: &str, expect: &Value, value: &Value) {
    if let Some(avail) = expect["availability"].as_object() {
        for (kind, want) in avail {
            let got = &value["availability"][kind];
            for (k, v) in want.as_object().unwrap() {
                assert_eq!(&got[k], v, "{name}: availability.{kind}.{k}");
            }
        }
    }
    if let Some(form) = expect["fieldsForm"].as_str() {
        assert_eq!(value["fieldsForm"], form, "{name}");
    }
    if let Some(n) = expect["fieldCount"].as_u64() {
        assert_eq!(
            value["fields"].as_array().map(Vec::len),
            Some(n as usize),
            "{name}"
        );
    }
    if let Some(ids) = expect["clauseIds"].as_array() {
        let got: Vec<&str> = value["clauses"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["clauseId"].as_str().unwrap())
            .collect();
        assert_eq!(
            got,
            ids.iter().map(|v| v.as_str().unwrap()).collect::<Vec<_>>(),
            "{name}"
        );
    }
    if let Some(names) = expect["operationNames"].as_array() {
        let got: Vec<&str> = value["operations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["name"].as_str().unwrap())
            .collect();
        assert_eq!(
            got,
            names
                .iter()
                .map(|v| v.as_str().unwrap())
                .collect::<Vec<_>>(),
            "{name}"
        );
    }
    if let Some(codes) = expect["diagnosticCodes"].as_array() {
        let got: Vec<&str> = value["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .map(|d| d["code"].as_str().unwrap())
            .collect();
        assert_eq!(
            got,
            codes
                .iter()
                .map(|v| v.as_str().unwrap())
                .collect::<Vec<_>>(),
            "{name}"
        );
    }
    if let Some(diags) = expect["diagnostics"].as_array() {
        for want in diags {
            let hit = value["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .any(|d| want.as_object().unwrap().iter().all(|(k, v)| &d[k] == v));
            assert!(hit, "{name}: expected {want}, got {}", value["diagnostics"]);
        }
    }
    if let Some(identity) = expect["sourceIdentity"].as_str() {
        assert_eq!(
            value["clauses"][0]["sourceSpan"]["sourceIdentity"], identity,
            "{name}"
        );
    }
    // Diagnostics are ordered by (line, column, code).
    let keys: Vec<(u64, u64, String)> = value["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| {
            (
                d["line"].as_u64().unwrap_or(0),
                d["column"].as_u64().unwrap_or(0),
                d["code"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    let mut sorted = keys.clone();
    sorted.sort();
    assert_eq!(keys, sorted, "{name}: diagnostic order");
    if let Some(values) = expect["values"].as_object() {
        for (pointer, want) in values {
            assert_eq!(
                value.pointer(pointer),
                Some(want),
                "{name}: {pointer}\n{value:#}"
            );
        }
    }
    for pointer in expect["absentPaths"].as_array().into_iter().flatten() {
        let pointer = pointer.as_str().unwrap();
        assert!(
            value.pointer(pointer).is_none(),
            "{name}: {pointer} is present\n{value:#}"
        );
    }
    for reason in expect["absent_diagnostic_reasons"]
        .as_array()
        .into_iter()
        .flatten()
    {
        assert!(
            !value["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .any(|d| &d["reason"] == reason),
            "{name}: a diagnostic carries {reason}\n{value:#}"
        );
    }
}

#[trace("TC-1630", "FR-072-AC-1")]
// every case yields its expected declarations, availability, and ordered
// diagnostics; every record validates against semantic-v1.
#[test]
fn case_suite() {
    let schema = schema();
    let mut records: BTreeMap<String, Value> = BTreeMap::new();
    for case in cases() {
        let name = case["name"].as_str().unwrap();
        let record =
            extract_semantic_json(&case["input"]).unwrap_or_else(|e| panic!("{name}: {e}"));
        let value = serde_json::to_value(&record).unwrap();
        let errors: Vec<String> = schema
            .iter_errors(&value)
            .map(|e| format!("{} at {}", e, e.instance_path()))
            .collect();
        assert!(errors.is_empty(), "{name}: {errors:?}");
        assert_case(name, &case["expect"], &value);
        records.insert(name.to_string(), value);
    }
    // The Rust outputs are the parity oracle for the Python and WASM legs.
    write_or_compare(
        "tests/fixtures/semantic/cases.expected.json",
        &format!("{}\n", serde_json::to_string_pretty(&records).unwrap()),
    );
}

/// Run the cases filed under `issue_ref`: each `key` has a case that
/// resolves, a refusal, and a control asserting absent diagnostic reasons;
/// every record is schema-valid and equals its `cases.expected.json` record,
/// the oracle the Python binding is checked against.
fn assert_filed_cases(issue_ref: &str, keys: &[&str]) {
    let schema = schema();
    let expected = expected_records();
    let filed: Vec<Value> = cases()
        .into_iter()
        .filter(|c| c["issue_ref"] == issue_ref)
        .collect();
    let tagged = |key: &str, outcome: &str| {
        filed
            .iter()
            .filter(|c| {
                let tags = c["tags"].as_array().unwrap();
                tags.contains(&json!(key)) && tags.contains(&json!(outcome))
            })
            .count()
    };
    for key in keys {
        for outcome in ["resolves", "refusal", "control"] {
            assert_eq!(tagged(key, outcome), 1, "{issue_ref} {key} {outcome}");
        }
    }
    for case in &filed {
        let name = case["name"].as_str().unwrap();
        if case["tags"].as_array().unwrap().contains(&json!("control")) {
            assert!(
                case["expect"]["absent_diagnostic_reasons"]
                    .as_array()
                    .is_some_and(|r| !r.is_empty()),
                "{name}: a control asserts absent diagnostic reasons"
            );
        }
        let value = serde_json::to_value(extract_semantic_json(&case["input"]).unwrap()).unwrap();
        assert!(schema.is_valid(&value), "{name}: {value:#}");
        assert_case(name, &case["expect"], &value);
        assert_eq!(
            value, expected[name],
            "{name}: differs from cases.expected.json"
        );
    }
}

#[trace("TC-1998", "FR-075-AC-27")]
// PLAT-35 cases per reference column: the resolved SemanticId, a refusal and
// a control.
#[test]
fn declaration_reference_cases() {
    assert_filed_cases(
        "PLAT-35",
        &["transition-emits", "step-consumes", "step-emits"],
    );
}

#[trace("TC-2004", "FR-076-AC-21")]
// PLAT-55 cases per typed relation key: the lowered identity, a refusal and a
// control.
#[test]
fn typed_relation_key_cases() {
    assert_filed_cases(
        "PLAT-55",
        &[
            "aggregate-root-emits",
            "process-emits",
            "repository-persists",
        ],
    );
}

/// Run the PLAT-57 cases tagged `tc`: each is schema-valid, meets its
/// `expect`, and equals its `cases.expected.json` record.
fn assert_inline_expression_cases(tc: &str) -> usize {
    let schema = schema();
    let expected = expected_records();
    let mut ran = 0;
    for case in cases().into_iter().filter(|c| {
        c["issue_ref"] == "PLAT-57" && c["tags"].as_array().unwrap().contains(&json!(tc))
    }) {
        let name = case["name"].as_str().unwrap();
        let value = serde_json::to_value(extract_semantic_json(&case["input"]).unwrap()).unwrap();
        let errors: Vec<String> = schema
            .iter_errors(&value)
            .map(|e| format!("{} at {}", e, e.instance_path()))
            .collect();
        assert!(errors.is_empty(), "{name}: {errors:?}\n{value:#}");
        assert_case(name, &case["expect"], &value);
        assert_eq!(
            value, expected[name],
            "{name}: differs from cases.expected.json"
        );
        ran += 1;
    }
    ran
}

#[trace("TC-2040", "FR-071-AC-10", "FR-071-AC-11", "FR-075-AC-30")]
// `Pre: x > 1 and y(a, b)` is one unsplit expression with an empty frame
// `pre`; the `Pre: a, b` control stays a reference list.
#[test]
fn pre_expression_cases() {
    assert_eq!(assert_inline_expression_cases("TC-2040"), 2);
}

#[trace("TC-2041", "FR-071-AC-11", "FR-071-AC-12", "FR-071-AC-14")]
// classification by text shape, empty lines refused, and no expression
// without its operation.
#[test]
fn expression_classification_cases() {
    assert_eq!(assert_inline_expression_cases("TC-2041"), 9);
}

#[trace("TC-2042", "FR-071-AC-13")]
// an invalid-syntax expression is carried verbatim with only the advisory.
#[test]
fn invalid_syntax_expression_is_carried() {
    assert_eq!(assert_inline_expression_cases("TC-2042"), 1);
}

#[trace("TC-2043", "FR-075-AC-28", "FR-075-AC-29")]
// transition `Guard` cells: expression, reference, dangling, empty.
#[test]
fn guard_expression_cases() {
    assert_eq!(assert_inline_expression_cases("TC-2043"), 9);
    // A `Guard` expression neither adds nor removes the `Emits` outcome of
    // the same table without a bundle index (FR-075-AC-26).
    let expected = expected_records();
    let with = &expected["plat57-guard-expression-no-bundle-index"];
    let without = &expected["plat57-guard-empty-no-bundle-index"];
    assert_eq!(with["availability"], without["availability"]);
    let emits = |record: &Value| -> Vec<Value> {
        record["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|d| d["code"] == "semantic.unresolved-target")
            .map(|d| json!({ "reason": d["reason"], "line": d["line"], "section": d["section"] }))
            .collect()
    };
    assert_eq!(emits(with), emits(without));
    assert_eq!(
        emits(with).len(),
        1,
        "the Emits outcome is one diagnostic: {with:#}"
    );
}

#[trace("TC-1631", "FR-072-AC-2")]
// every state token and both lossy values appear, each distinct;
// unavailable/missing carry a reason.
#[test]
fn availability_states_are_exercised_and_distinct() {
    let expected = expected_records();
    let mut states: BTreeMap<String, Vec<&str>> = BTreeMap::new();
    let mut lossy_values = std::collections::BTreeSet::new();
    for (_, record) in expected.as_object().unwrap() {
        for kind in ["fields", "clauses", "operations"] {
            let a = &record["availability"][kind];
            let state = a["state"].as_str().unwrap();
            states.entry(kind.to_string()).or_default().push(state);
            lossy_values.insert(a["lossy"].as_bool().unwrap());
            if state == "unavailable" || state == "missing" {
                assert!(
                    a["reason"].as_str().is_some_and(|r| !r.is_empty()),
                    "{kind}: {a}"
                );
            }
        }
    }
    for kind in ["fields", "clauses", "operations"] {
        let seen = &states[kind];
        for want in ["available", "not_applicable", "missing", "unavailable"] {
            assert!(seen.contains(&want), "{kind} never {want}");
        }
    }
    let all: Vec<&str> = states.values().flatten().copied().collect();
    for want in ["available", "not_applicable", "missing", "unavailable"] {
        assert!(all.contains(&want), "state {want} never exercised");
    }
    assert_eq!(lossy_values.len(), 2, "both lossy values");
    assert_eq!(
        serde_json::to_value(AvailabilityState::NotApplicable).unwrap(),
        json!("not_applicable")
    );
}

fn fixture_snapshot(with_context: bool) -> FilamentExtractionInput {
    let entity: Value = serde_json::from_str(
        &fs::read_to_string(quire_fixtures::module_ok_dir().join("schemas/Entity.json")).unwrap(),
    )
    .unwrap();
    let markdown = quire_fixtures::mapping_fixture("config-version.table.md");
    let mut object_type = json!({
        "name": "entity",
        "dataSchema": if with_context { entity } else { json!({ "type": "object" }) },
        "allowedLinks": {},
        "bodyExtraction": { "yield_pattern": { "match": {
            "id": { "from": "frontmatter_field", "path": ["id"], "required": true },
            "properties": { "from": "section_body", "after_heading": "Properties", "required": true }
        } } },
        "hasPlugin": false,
        "moduleId": "spec-objects-fixture"
    });
    if with_context {
        object_type["semantic"] = json!({ "contractVersion": "1.0.0", "package": "agent-ix/spec-objects-fixture", "exports": ["entity"], "imports": {} });
    }
    let mut input = json!({
        "projectId": "p", "documentId": "d", "artifactId": "a", "relPath": "spec/functional/FR-006.md",
        "repoName": "config-service", "org": "agent-ix", "markdown": markdown, "objectTypes": [object_type]
    });
    if with_context {
        input["semanticBundle"] = read_json("tests/fixtures/semantic/config-version.bundle.json");
    }
    serde_json::from_value(input).unwrap()
}

#[trace("TC-1632", "FR-072-AC-3", "FR-072-CON-1")]
#[trace("TC-2055", "FR-072-AC-9")]
// with a context: dataJson.semantic + diagnostics with locus and mapped
// severity; without: no `semantic` key and no locus.
#[test]
fn filament_surface_with_and_without_context() {
    let result = extract_filament_core(fixture_snapshot(true));
    let node = result
        .nodes
        .iter()
        .find(|n| n.object_type == "entity")
        .unwrap_or_else(|| panic!("{:?}", result.diagnostics));
    let data: Value = serde_json::from_str(&node.data_json).unwrap();
    assert!(
        data["properties"].as_str().unwrap().contains("| Field |"),
        "DSL record unchanged"
    );
    let semantic = &data["semantic"];
    assert_eq!(semantic["formatVersion"], 1);
    assert_eq!(semantic["fields"].as_array().map(Vec::len), Some(7));
    assert_eq!(
        semantic["clauses"][0]["sourceSpan"]["sourceIdentity"],
        "ix://agent-ix/config-service/spec"
    );
    assert_eq!(
        semantic["clauses"][0]["sourceSpan"]["path"],
        "spec/functional/FR-006.md"
    );
    // The Filament record equals the library record for the same inputs
    // (golden-table-available carries the same identity and path).
    let expected = expected_records();
    assert_eq!(semantic, &expected["golden-table-available"]);
    assert!(schema().is_valid(semantic));
    assert!(
        !result.diagnostics.iter().any(|d| d.severity == "error"),
        "{:?}",
        result.diagnostics
    );
    for d in &result.diagnostics {
        assert!(matches!(d.severity.as_str(), "info" | "warning" | "error"));
    }
    // A legacy-form document under the context: warning with a locus.
    let mut input = fixture_snapshot(true);
    input.markdown = quire_fixtures::mapping_fixture("legacy-bullets.md");
    let result = extract_filament_core(input);
    let d = result
        .diagnostics
        .iter()
        .find(|d| d.code == "semantic.legacy-properties-form")
        .unwrap();
    assert_eq!(d.severity, "warning");
    assert_eq!(
        d.locus.as_ref().map(|l| (l.path.as_str(), l.line)),
        Some(("spec/functional/FR-006.md", 12))
    );

    // Without a context: no `semantic` key anywhere, no locus.
    let result = extract_filament_core(fixture_snapshot(false));
    let value = serde_json::to_value(&result).unwrap();
    assert!(!serde_json::to_string(&value)
        .unwrap()
        .contains("\"semantic\""));
    assert!(!serde_json::to_string(&value).unwrap().contains("\"locus\""));
}

#[trace("TC-1634", "FR-072-AC-5")]
// validate_document over the free-table fixture warns at its line; over both-forms
// it errors at the fence line and fails validation.
#[test]
fn validate_document_surface() {
    let registry = Registry::load_module(&quire_fixtures::module_ok_dir()).unwrap();
    let entity = registry.archetype("entity").unwrap();
    let corpus = quire_fixtures::mapping_fixture("legacy-free-table.md");
    let result = quire_rs::validate_document_in_registry(&registry, entity, &corpus);
    let legacy = result
        .warnings
        .iter()
        .find(|w| w.message.starts_with("semantic.legacy-properties-form"))
        .unwrap_or_else(|| panic!("{:?}", result.warnings));
    assert_eq!(legacy.line, Some(17));
    assert_eq!(legacy.reason.as_str(), "semantic");
    let both = quire_fixtures::mapping_fixture("both-forms.md");
    let result = quire_rs::validate_document_in_registry(&registry, entity, &both);
    assert!(!result.is_valid);
    let e = result
        .errors
        .iter()
        .find(|e| e.message.starts_with("semantic.properties-both-forms"))
        .unwrap();
    assert_eq!(e.line, Some(16));
    // The golden table validates cleanly against the resolved Entity.json.
    let table = quire_fixtures::mapping_fixture("config-version.table.md");
    let result = quire_rs::validate_document_in_registry(&registry, entity, &table);
    assert!(result.is_valid, "{:?}", result.errors);
    // A document with no Properties fails the resolved schema (`fields` required).
    let bare = "---\nid: FR-900\ntitle: Bare\nobject: entity\n---\n# FR-900\n\nprose\n";
    let result = quire_rs::validate_document_in_registry(&registry, entity, bare);
    assert!(!result.is_valid);
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.message.starts_with("semantic.record-invalid")),
        "{:?}",
        result.errors
    );
}

#[trace("TC-1637", "FR-072-AC-7")]
#[trace("TC-1644", "NFR-021-AC-4")]
// byte-identical repeated runs; an unrelated edit changes nothing.
#[test]
fn determinism_and_isolation() {
    for case in cases() {
        let a = serde_json::to_string(&extract_semantic_json(&case["input"]).unwrap()).unwrap();
        let b = serde_json::to_string(&extract_semantic_json(&case["input"]).unwrap()).unwrap();
        assert_eq!(a, b, "{}", case["name"]);
    }
    let case = cases()
        .into_iter()
        .find(|c| c["name"] == "golden-table-available")
        .unwrap();
    let before = serde_json::to_value(extract_semantic_json(&case["input"]).unwrap()).unwrap();
    let mut edited = case["input"].clone();
    edited["markdown"] = json!(format!(
        "{}\n## Notes\n\nA new unrelated section.\n",
        case["input"]["markdown"].as_str().unwrap()
    ));
    let after = serde_json::to_value(extract_semantic_json(&edited).unwrap()).unwrap();
    assert_eq!(before, after);
}

#[trace("TC-1638", "FR-072-AC-8")]
// the schema is 2020-12 with additionalProperties: false everywhere; the
// expected-records fixture pins every field and state token; a removed field
// fails the fixture.
#[test]
fn schema_and_compatibility_fixture() {
    let schema_value: Value = serde_json::from_str(SEMANTIC_V1_SCHEMA).unwrap();
    assert_eq!(
        schema_value["$schema"],
        "https://json-schema.org/draft/2020-12/schema"
    );
    assert!(schema_value["$id"]
        .as_str()
        .unwrap()
        .ends_with("semantic-v1.schema.json"));
    // Every object is closed except the two whose keys are data: the record's
    // top level (FR-076 typed relation keys) and `clauseText`.
    fn every_object_closed(v: &Value, path: &str) {
        if let Some(map) = v.as_object() {
            if map.get("type") == Some(&json!("object")) {
                let want = match path {
                    "" => json!({ "type": "array", "items": { "$ref": "#/$defs/semanticId" } }),
                    "/properties/clauseText" => json!({ "type": "string" }),
                    _ => json!(false),
                };
                assert_eq!(
                    map.get("additionalProperties"),
                    Some(&want),
                    "{path}: additionalProperties"
                );
            }
            for (k, child) in map {
                every_object_closed(child, &format!("{path}/{k}"));
            }
        } else if let Some(items) = v.as_array() {
            for (i, child) in items.iter().enumerate() {
                every_object_closed(child, &format!("{path}/{i}"));
            }
        }
    }
    every_object_closed(&schema_value, "");
    let schema = schema();
    let fixture = expected_records();
    let records: Vec<&Value> = fixture.as_object().unwrap().values().collect();
    assert!(records.len() >= 3);
    let mut tokens = std::collections::BTreeSet::new();
    let mut keys = std::collections::BTreeSet::new();
    for record in &records {
        assert!(schema.is_valid(record), "{record}");
        for kind in ["fields", "clauses", "operations"] {
            tokens.insert(
                record["availability"][kind]["state"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            );
        }
        keys.extend(record.as_object().unwrap().keys().cloned());
    }
    for want in ["available", "not_applicable", "missing", "unavailable"] {
        assert!(tokens.contains(want), "fixture never pins {want}");
    }
    for want in [
        "formatVersion",
        "contractVersion",
        "package",
        "fields",
        "fieldsForm",
        "clauses",
        "clauseText",
        "operations",
        "availability",
        "diagnostics",
    ] {
        assert!(keys.contains(want), "fixture never pins {want}");
    }
    // Removing or renaming a pinned field fails.
    let mut broken = records[0].clone();
    broken.as_object_mut().unwrap().remove("availability");
    assert!(!schema.is_valid(&broken));
    let mut renamed = records[0].clone();
    let v = renamed
        .as_object_mut()
        .unwrap()
        .remove("diagnostics")
        .unwrap();
    renamed["findings"] = v;
    assert!(!schema.is_valid(&renamed));
}

#[trace("TC-1650", "FR-072-CON-3")]
// The dependency ban covers schemars; this checks the authored schema itself.
#[test]
fn semantic_schema_has_no_generator_marker() {
    let schema = SEMANTIC_V1_SCHEMA.to_lowercase();
    for marker in ["schemars", "x-generated", "\"generated\"", "generated by"] {
        assert!(
            !schema.contains(marker),
            "semantic-v1 schema contains generator marker {marker}"
        );
    }
}
