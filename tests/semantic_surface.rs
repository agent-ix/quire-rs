//! FR-072 surface (TC-1630, TC-1631, TC-1632, TC-1634, TC-1637, TC-1638,
//! TC-1644 Rust half, TC-1650). Plan-003 Task-020.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use ix_trace_rs::trace;
use jsonschema::JSONSchema;
use quire_rs::semantic::python_entry::extract_semantic_json;
use quire_rs::semantic::{AvailabilityState, SEMANTIC_V1_SCHEMA};
use quire_rs::{extract_filament_core, FilamentExtractionInput, Registry};
use serde_json::{json, Value};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_json(rel: &str) -> Value {
    serde_json::from_slice(&fs::read(root().join(rel)).unwrap()).unwrap()
}

fn schema() -> JSONSchema {
    JSONSchema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .compile(&serde_json::from_str::<Value>(SEMANTIC_V1_SCHEMA).unwrap())
        .unwrap()
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
        let errors: Vec<String> = match schema.validate(&value) {
            Ok(()) => Vec::new(),
            Err(errs) => errs
                .map(|e| format!("{} at {}", e, e.instance_path))
                .collect(),
        };
        assert!(errors.is_empty(), "{name}: {errors:?}");
        let expect = &case["expect"];
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
        records.insert(name.to_string(), value);
    }
    // The Rust outputs are the parity oracle for the Python and WASM legs.
    write_or_compare(
        "tests/fixtures/semantic/cases.expected.json",
        &format!("{}\n", serde_json::to_string_pretty(&records).unwrap()),
    );
}

#[trace("TC-1631", "FR-072-AC-2")]
// every state token and both lossy values appear, each distinct;
// unavailable/missing carry a reason.
#[test]
fn availability_states_are_exercised_and_distinct() {
    let expected = read_json("tests/fixtures/semantic/cases.expected.json");
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

/// The digest the registry owner recorded over Entity.json (FR-069: Quire
/// passes it through, never minting a second one).
const SNAPSHOT_DIGEST: &str =
    "sha256:8692992e186f40a73b78fd1b0915f0fe78e05a2b00782fb1df58c951a37c91d5";

fn fixture_snapshot(with_context: bool) -> FilamentExtractionInput {
    let entity: Value = read_json("tests/fixtures/semantic/quoin/module-ok/schemas/Entity.json");
    let markdown = fs::read_to_string(
        root().join("tests/fixtures/semantic/quoin/mapping/config-version.table.md"),
    )
    .unwrap();
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
        object_type["semantic"] = json!({ "contractVersion": "1.0.0", "semanticCore": "0.1.0", "package": "agent-ix/spec-objects-fixture", "exports": ["entity"], "imports": {}, "schemaDigest": SNAPSHOT_DIGEST });
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
// with a context: dataJson.semantic + diagnostics with locus and mapped
// severity; without: byte-identical to the pre-change graph baseline.
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
    assert_eq!(
        semantic["schemaDigest"], SNAPSHOT_DIGEST,
        "passed through, not re-minted"
    );
    // The Filament record equals the library record for the same inputs
    // (golden-table-available carries the same digest, identity, and path).
    let expected = read_json("tests/fixtures/semantic/cases.expected.json");
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
    input.markdown =
        fs::read_to_string(root().join("tests/fixtures/semantic/quoin/mapping/legacy-bullets.md"))
            .unwrap();
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

    // Without a context: no `semantic` key anywhere, no locus, and the
    // baseline graph cases reproduce byte for byte.
    let result = extract_filament_core(fixture_snapshot(false));
    let value = serde_json::to_value(&result).unwrap();
    assert!(!serde_json::to_string(&value)
        .unwrap()
        .contains("\"semantic\""));
    assert!(!serde_json::to_string(&value).unwrap().contains("\"locus\""));
    let baseline = read_json("tests/fixtures/semantic/baseline/filament-graph-cases.json");
    let cases: Vec<Value> = read_json("tests/fixtures/filament_core/graph_cases.json")
        .as_array()
        .unwrap()
        .clone();
    for case in cases {
        let input: FilamentExtractionInput = serde_json::from_value(case["input"].clone()).unwrap();
        let out = serde_json::to_string_pretty(
            &serde_json::to_value(extract_filament_core(input)).unwrap(),
        )
        .unwrap();
        let want = serde_json::to_string_pretty(&baseline[case["name"].as_str().unwrap()]).unwrap();
        assert_eq!(out, want, "{}", case["name"]);
    }
}

#[trace("TC-1634", "FR-072-AC-5")]
// validate_document over the corpus copy warns at its line; over both-forms
// it errors at the fence line and fails validation.
#[test]
fn validate_document_surface() {
    let registry =
        Registry::load_module(&root().join("tests/fixtures/semantic/quoin/module-ok")).unwrap();
    let entity = registry.archetype("entity").unwrap();
    let corpus = fs::read_to_string(root().join(
        "tests/fixtures/semantic/quoin/corpus/config-service/FR-006-config-version-entity.md",
    ))
    .unwrap();
    let result = quire_rs::validate_document_in_registry(&registry, entity, &corpus);
    let legacy = result
        .warnings
        .iter()
        .find(|w| w.message.starts_with("semantic.legacy-properties-form"))
        .unwrap_or_else(|| panic!("{:?}", result.warnings));
    assert_eq!(legacy.line, Some(17));
    assert_eq!(legacy.reason.as_str(), "semantic");
    let both =
        fs::read_to_string(root().join("tests/fixtures/semantic/quoin/mapping/both-forms.md"))
            .unwrap();
    let result = quire_rs::validate_document_in_registry(&registry, entity, &both);
    assert!(!result.is_valid);
    let e = result
        .errors
        .iter()
        .find(|e| e.message.starts_with("semantic.properties-both-forms"))
        .unwrap();
    assert_eq!(e.line, Some(16));
    // The golden table validates cleanly against the resolved Entity.json.
    let table = fs::read_to_string(
        root().join("tests/fixtures/semantic/quoin/mapping/config-version.table.md"),
    )
    .unwrap();
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
// compatibility fixture pins every field and state token; a removed field
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
    fn every_object_closed(v: &Value, path: &str) {
        if let Some(map) = v.as_object() {
            if map.get("type") == Some(&json!("object"))
                && !map.contains_key("additionalProperties")
            {
                panic!("{path}: object without additionalProperties");
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
    if std::env::var_os("UPDATE_SEMANTIC_BASELINES").is_some() {
        // Mint: the engine's records for five named cases, `$case` first.
        let mut records = Vec::new();
        for name in [
            "golden-table-available",
            "operations-available",
            "required-section-missing",
            "legacy-bullets-unavailable",
            "unresolved-type-lossy",
        ] {
            let case = cases().into_iter().find(|c| c["name"] == name).unwrap();
            let v = serde_json::to_value(extract_semantic_json(&case["input"]).unwrap()).unwrap();
            let mut ordered = serde_json::Map::new();
            ordered.insert("$case".into(), json!(name));
            for (k, val) in v.as_object().unwrap() {
                ordered.insert(k.clone(), val.clone());
            }
            records.push(Value::Object(ordered));
        }
        let out = json!({ "$comment": "FR-072-AC-8 compatibility fixture: semantic-v1 records the engine produced for the named cases.json inputs at minting; every field and state token is pinned. A drift here is a contract change.", "records": records });
        fs::write(
            root().join("tests/fixtures/semantic/semantic-v1.json"),
            format!("{}\n", serde_json::to_string_pretty(&out).unwrap()),
        )
        .unwrap();
    }
    let fixture = read_json("tests/fixtures/semantic/semantic-v1.json");
    let records = fixture["records"].as_array().unwrap();
    assert!(records.len() >= 3);
    let mut tokens = std::collections::BTreeSet::new();
    let mut keys = std::collections::BTreeSet::new();
    for record in records {
        let mut bare = record.clone();
        bare.as_object_mut().unwrap().remove("$case");
        assert!(schema.is_valid(&bare), "{record}");
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
        "semanticCore",
        "package",
        "schemaDigest",
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
    broken.as_object_mut().unwrap().remove("$case");
    broken.as_object_mut().unwrap().remove("availability");
    assert!(!schema.is_valid(&broken));
    let mut renamed = records[0].clone();
    renamed.as_object_mut().unwrap().remove("$case");
    let v = renamed
        .as_object_mut()
        .unwrap()
        .remove("diagnostics")
        .unwrap();
    renamed["findings"] = v;
    assert!(!schema.is_valid(&renamed));
    // The fixture is the engine's own output for its inputs.
    for record in records {
        let name = record["$case"].as_str().unwrap();
        let case = cases().into_iter().find(|c| c["name"] == name).unwrap();
        let mut produced =
            serde_json::to_value(extract_semantic_json(&case["input"]).unwrap()).unwrap();
        produced["$case"] = json!(name);
        let mut pinned = record.clone();
        pinned.as_object_mut().unwrap().remove("$case");
        let mut produced_cmp = produced.clone();
        produced_cmp.as_object_mut().unwrap().remove("$case");
        assert_eq!(
            produced_cmp, pinned,
            "{name}: compatibility fixture drifted from the engine"
        );
    }
}

#[trace("TC-1650", "FR-072-CON-3")]
// the schemars audit covers the semantic schema.
#[test]
fn generator_audit_covers_semantic_schema() {
    let script = fs::read_to_string(root().join("scripts/audits/check_no_schemars.sh")).unwrap();
    assert!(script.contains("semantic-v1.schema.json"));
    assert!(!SEMANTIC_V1_SCHEMA.to_lowercase().contains("schemars"));
}
