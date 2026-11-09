//! FR-081 record-sections extraction (quoin FR-121, FR-122, FR-123). Oracle:
//! `cases.json` and `schemas/` of the `quire-fixtures` crate
//! (`fixtures/record-sections`).

use std::collections::BTreeMap;

use ix_trace_rs::trace;
use jsonschema::Validator;
use quire_rs::semantic::python_entry::extract_semantic_json;
use quire_rs::semantic::{
    compile_documents, compile_module_schema, module_schema_documents, SemanticContext,
    SEMANTIC_V1_SCHEMA,
};
use serde_json::{json, Value};

/// `$id` base of the fixture schemas.
const BASE: &str = "https://schemas.agent-ix.org/agent-ix/quire-fixtures/0.1.0/";

fn schema_root(name: &str) -> Value {
    quire_fixtures::record_sections_json(&format!("schemas/{name}.json"))
}

fn siblings(name: &str) -> Option<Result<Value, String>> {
    let path = quire_fixtures::record_sections_dir()
        .join("schemas")
        .join(name);
    let text = std::fs::read_to_string(path).ok()?;
    Some(serde_json::from_str(&text).map_err(|e| e.to_string()))
}

fn documents(root: &Value) -> BTreeMap<String, Value> {
    module_schema_documents(root, &siblings, BASE).unwrap()
}

fn semantic_v1() -> Validator {
    compile_module_schema(
        &serde_json::from_str::<Value>(SEMANTIC_V1_SCHEMA).unwrap(),
        &|_| None,
        "https://agent-ix.github.io/quire-rs/schemas/output/",
    )
    .unwrap()
}

/// The 1-based line of the first line containing `at`; `<text>#2` selects the
/// second such line.
fn line_of(markdown: &str, at: &str) -> usize {
    let (text, nth) = at
        .rsplit_once('@')
        .map_or((at, 1), |(t, n)| (t, n.parse().unwrap()));
    markdown
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains(text))
        .nth(nth - 1)
        .unwrap_or_else(|| panic!("no line {nth} contains {text:?}"))
        .0
        + 1
}

fn request(case: &Value, root: &Value, docs: &BTreeMap<String, Value>) -> Value {
    let mut request = json!({
        "markdown": case["artifact"],
        "module": {
            "contractVersion": "1.0.0",
            "package": case.get("package").unwrap_or(&json!("acme/safety")),
            "exports": [],
            "mappings": case.get("mappings").unwrap_or(&json!(["record-sections"])),
        },
        "path": "artifact.md",
        "sourceIdentity": "ix://acme/safety/spec",
        "bundle": case.get("bundle").unwrap_or(&json!({ "package": "acme/safety" })),
        "dataSchema": root,
        "schemaDocuments": docs,
    });
    let map = request.as_object_mut().unwrap();
    for key in ["relationVocabulary", "bodyExtraction"] {
        if let Some(value) = case.get(key) {
            map.insert(key.to_string(), value.clone());
        }
    }
    if case["withoutSourceIdentity"] == json!(true) {
        map.remove("sourceIdentity");
    }
    request
}

fn expected_diagnostics(markdown: &str, case: &Value) -> Vec<(String, String, String, usize)> {
    let mut out: Vec<_> = case["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| {
            (
                d["code"].as_str().unwrap().to_string(),
                d["severity"].as_str().unwrap().to_string(),
                d["reason"].as_str().unwrap().to_string(),
                line_of(markdown, d["at"].as_str().unwrap()),
            )
        })
        .collect();
    out.sort();
    out
}

fn actual_diagnostics(record: &Value) -> Vec<(String, String, String, usize)> {
    let mut out: Vec<_> = record["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| {
            (
                d["code"].as_str().unwrap().to_string(),
                d["severity"].as_str().unwrap().to_string(),
                d["reason"].as_str().unwrap_or_default().to_string(),
                usize::try_from(d["line"].as_u64().unwrap()).unwrap(),
            )
        })
        .collect();
    out.sort();
    out
}

fn expected_availability(markdown: &str, expected: &Value) -> Value {
    let Some(keys) = expected.as_object().filter(|keys| !keys.is_empty()) else {
        return Value::Null;
    };
    Value::Object(
        keys.iter()
            .map(|(key, state)| {
                let mut state = state.as_object().unwrap().clone();
                if let Some(at) = state.remove("reasonAt") {
                    let mut lines: Vec<usize> = at
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|a| line_of(markdown, a.as_str().unwrap()))
                        .collect();
                    lines.sort_unstable();
                    lines.dedup();
                    let lines: Vec<String> = lines.iter().map(usize::to_string).collect();
                    state.insert(
                        "reason".to_string(),
                        json!(format!("entry-errors: lines {}", lines.join(", "))),
                    );
                }
                (key.clone(), Value::Object(state))
            })
            .collect(),
    )
}

/// Run every case that backs `ac`.
fn run_cases(ac: &str) {
    let file = quire_fixtures::record_sections_json("cases.json");
    let v1 = semantic_v1();
    let mut ran = 0;
    for case in file["cases"].as_array().unwrap() {
        if case["ac"] != ac {
            continue;
        }
        ran += 1;
        let id = case["id"].as_str().unwrap();
        let markdown = case["artifact"].as_str().unwrap();
        let root = schema_root(case["schema"].as_str().unwrap());
        let docs = documents(&root);
        let extraction = extract_semantic_json(&request(case, &root, &docs)).unwrap();
        let record = serde_json::to_value(&extraction).unwrap();
        let errors: Vec<String> = v1
            .iter_errors(&record)
            .map(|e| format!("{e} at {}", e.instance_path()))
            .collect();
        assert!(errors.is_empty(), "{id}: {errors:?}\n{record:#}");

        assert_eq!(
            actual_diagnostics(&record),
            expected_diagnostics(markdown, case),
            "{id}: diagnostics\n{record:#}"
        );
        assert_eq!(
            record.get("record").cloned().unwrap_or(Value::Null),
            case["record"],
            "{id}: record"
        );
        let sources: Vec<&str> = record["recordSources"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|s| s["key"].as_str().unwrap())
            .collect();
        let expected_sources: Vec<&str> = case["recordSources"]
            .as_array()
            .unwrap()
            .iter()
            .map(|k| k.as_str().unwrap())
            .collect();
        assert_eq!(sources, expected_sources, "{id}: recordSources");
        assert_eq!(
            record["availability"]
                .get("recordKeys")
                .cloned()
                .unwrap_or(Value::Null),
            expected_availability(markdown, &case["availability"]),
            "{id}: availability.recordKeys"
        );
        for (key, identities) in case["typedRelations"].as_object().into_iter().flatten() {
            assert_eq!(&record[key], identities, "{id}: typed relation {key}");
        }
        if case["modelHasValues"] == json!(true) {
            assert!(record["model"]["values"].is_array(), "{id}: model.values");
        }

        // The declaration record validates against the case schema exactly
        // when the case says so.
        let declaration = extraction.declaration_record().unwrap();
        let validator = compile_documents(&root, &docs).unwrap();
        assert_eq!(
            validator.is_valid(&declaration),
            case["recordValid"] == json!(true),
            "{id}: declaration record {declaration:#}"
        );
        // Every extracted key rides in the declaration record.
        for (key, value) in record["record"].as_object().into_iter().flatten() {
            assert_eq!(&declaration[key], value, "{id}: declaration key {key}");
        }
    }
    assert!(ran > 0, "no case backs {ac}");
}

#[trace("TC-2167", "FR-081-AC-1", "FR-081-AC-2", "FR-081-CON-1")]
#[test]
fn assessment_and_lifecycle_keys() {
    run_cases("FR-081-AC-1");
    run_cases("FR-081-AC-2");
}

#[trace("TC-2168", "FR-081-AC-3")]
#[test]
fn evidence_targets_resolve_by_the_target_rules() {
    run_cases("FR-081-AC-3");
}

#[trace("TC-2169", "FR-081-AC-4")]
#[test]
fn table_and_cell_refusals() {
    run_cases("FR-081-AC-4");
}

#[trace("TC-2170", "FR-081-AC-5")]
#[test]
fn blocks_not_of_the_keys_form() {
    run_cases("FR-081-AC-5");
}

#[trace("TC-2171", "FR-081-AC-6", "FR-081-CON-2")]
#[test]
fn token_gating_and_owned_keys() {
    run_cases("FR-081-AC-6");
    // CON-2: without the token the record is the one a request with no
    // record schema produces.
    let file = quire_fixtures::record_sections_json("cases.json");
    let case = file["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == "no-token")
        .unwrap();
    let root = schema_root("Hazard");
    let docs = documents(&root);
    let with_schema = extract_semantic_json(&request(case, &root, &docs)).unwrap();
    let mut bare = request(case, &root, &docs);
    let map = bare.as_object_mut().unwrap();
    map.remove("dataSchema");
    map.remove("schemaDocuments");
    let without = extract_semantic_json(&bare).unwrap();
    assert_eq!(
        serde_json::to_value(with_schema).unwrap(),
        serde_json::to_value(without).unwrap()
    );
}

#[trace("TC-2172", "FR-081-AC-7")]
#[test]
fn camel_case_headings_and_ambiguous_keys() {
    run_cases("FR-081-AC-7");
}

#[trace("TC-2173", "FR-081-AC-8")]
#[test]
fn single_values_and_one_row_policy_tables() {
    run_cases("FR-081-AC-8");
}

#[trace("TC-2174", "FR-081-AC-9")]
#[test]
fn routes_name_declared_operations() {
    run_cases("FR-081-AC-9");
}

#[trace("TC-2175", "FR-081-AC-10")]
#[test]
fn associated_types_come_from_the_table_only() {
    run_cases("FR-081-AC-10");
}

#[trace("TC-2176", "FR-081-AC-11")]
#[test]
fn layout_fence_lowers_to_records_and_endianness() {
    run_cases("FR-081-AC-11");
}

#[trace("TC-2177", "FR-081-AC-12")]
#[test]
fn threshold_rows_build_on_exceeded() {
    run_cases("FR-081-AC-12");
}

#[trace("TC-2178", "FR-081-AC-13")]
#[test]
fn every_case_record_validates_against_semantic_v1() {
    let schema: Value = serde_json::from_str(SEMANTIC_V1_SCHEMA).unwrap();
    let properties = &schema["properties"];
    assert!(properties["record"].is_object());
    assert!(properties["recordSources"].is_object());
    assert!(properties["availability"]["properties"]["recordKeys"].is_object());
    // `run_cases` validates every record it extracts against the schema and
    // the declaration record against the case schema.
    let file = quire_fixtures::record_sections_json("cases.json");
    let mut acs: Vec<&str> = file["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["ac"].as_str().unwrap())
        .collect();
    acs.sort_unstable();
    acs.dedup();
    for ac in acs {
        run_cases(ac);
    }
}

#[trace("TC-2180", "FR-081-AC-15")]
#[test]
fn model_table_entries_copy_to_record_keys() {
    run_cases("FR-081-AC-15");
}

/// quoin FR-122-AC-7 (TC-2115): every top-level property of every
/// object-type record schema that `agent-ix/spec-objects-architecture` ships
/// has an owner, and none is declared not-authored.
#[trace("TC-2179", "FR-081-AC-14")]
#[test]
fn architecture_record_keys_are_all_owned() {
    use quire_rs::semantic::{record_key_owners, RelationVocabulary};
    if let Some(reason) = spec_objects_architecture_fixture::unavailable_reason() {
        eprintln!("skipped: the architecture module is not fetched: {reason}");
        return;
    }
    let registry =
        quire_rs::Registry::load_module(spec_objects_architecture_fixture::module_dir()).unwrap();
    let module = registry
        .semantic_module("spec-objects-architecture")
        .expect("the architecture module declares a semantic block")
        .clone();
    let mut unowned = Vec::new();
    let mut audited = 0;
    for arch in registry.active_archetypes() {
        let (Some(schema), Some(documents)) = (
            arch.data_schema.as_deref(),
            arch.data_schema_documents.as_deref(),
        ) else {
            continue;
        };
        let mut ctx = SemanticContext::new(module.clone(), "<audit>", Default::default())
            .with_relation_vocabulary(RelationVocabulary::from_registry(&registry, arch))
            .with_record_schema(schema, documents);
        if let Some(dsl) = arch.body_extraction() {
            ctx = ctx.with_body_extraction(dsl);
        }
        for (key, owner) in record_key_owners(&ctx) {
            audited += 1;
            if owner.is_none() {
                unowned.push(format!("{}.{key}", arch.name));
            }
        }
    }
    assert!(audited > 0, "no object type of the module was audited");
    assert!(unowned.is_empty(), "keys no mapping owns: {unowned:?}");
}
