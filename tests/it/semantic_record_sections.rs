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

/// Where a case's schemas come from: this crate's synthetic roots, or the
/// published architecture module, fetched and never copied.
struct Schemas {
    dir: std::path::PathBuf,
}

impl Schemas {
    fn of(case: &Value) -> Self {
        let dir = if case["module"] == "architecture" {
            assert!(
                spec_objects_architecture_fixture::is_available(),
                "the architecture module is not fetched: {:?}",
                spec_objects_architecture_fixture::unavailable_reason()
            );
            spec_objects_architecture_fixture::module_dir().join("schemas")
        } else {
            quire_fixtures::record_sections_dir().join("schemas")
        };
        Self { dir }
    }

    fn root(&self, name: &str) -> Value {
        self.sibling(&format!("{name}.json"))
            .unwrap_or_else(|| panic!("no schema {name}"))
            .unwrap()
    }

    fn sibling(&self, name: &str) -> Option<Result<Value, String>> {
        let text = std::fs::read_to_string(self.dir.join(name)).ok()?;
        Some(serde_json::from_str(&text).map_err(|e| e.to_string()))
    }

    fn documents(&self, root: &Value) -> BTreeMap<String, Value> {
        let id = root["$id"].as_str().unwrap();
        let base = &id[..=id.rfind('/').unwrap()];
        module_schema_documents(root, &|name| self.sibling(name), base).unwrap()
    }
}

/// A validator for one semantic-core document.
fn core_gate(name: &str) -> Validator {
    let core = quire_rs::semantic::embedded::embedded_semantic_core_version();
    let schema = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://schemas.agent-ix.org/agent-ix/quire-rs/0.1.0/Gate.json",
        "$ref": format!("https://schemas.agent-ix.org/semantic-core/{core}/{name}.json")
    });
    compile_module_schema(
        &schema,
        &|_| None,
        "https://schemas.agent-ix.org/agent-ix/quire-rs/",
    )
    .unwrap()
}

fn semantic_v1() -> Validator {
    compile_module_schema(
        &serde_json::from_str::<Value>(SEMANTIC_V1_SCHEMA).unwrap(),
        &|_| None,
        "https://agent-ix.github.io/quire-rs/schemas/output/",
    )
    .unwrap()
}

#[trace("FR-069-AC-5")]
#[test]
fn authentic_safety_schema_bundle_resolves_and_validates_records() {
    if !quire_fixtures::safety_module_available() {
        eprintln!(
            "SKIPPED: published safety module unavailable: {}",
            quire_fixtures::safety_module_unavailable_reason().unwrap_or("unknown fetch failure")
        );
        return;
    }

    let schemas = Schemas {
        dir: quire_fixtures::safety_schemas_dir(),
    };
    let mut names: Vec<String> = std::fs::read_dir(&schemas.dir)
        .unwrap_or_else(|e| panic!("read safety schema directory: {e}"))
        .map(|entry| entry.unwrap_or_else(|e| panic!("read safety schema entry: {e}")))
        .filter_map(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|extension| extension == "json")
                .then(|| entry.file_name().to_string_lossy().into_owned())
        })
        .collect();
    names.sort();
    assert!(
        names.len() > 3,
        "safety fixture must include referenced schemas"
    );

    let mut validators = BTreeMap::new();
    for name in &names {
        let root = schemas.root(name.trim_end_matches(".json"));
        let docs = schemas.documents(&root);
        let validator = compile_documents(&root, &docs)
            .unwrap_or_else(|e| panic!("authentic safety schema {name} must compile: {e:?}"));
        validators.insert(name.as_str(), validator);
    }

    let assessment = json!({
        "severity": "catastrophic",
        "likelihood": "probable",
        "rationale": "The hazard is severe and likely enough to require control."
    });
    let failure_analysis = json!({
        "effect": "The system enters an unsafe state.",
        "cause": "The interlock fails to engage.",
        "detection": "direct"
    });
    let hazard = json!({
        "fields": [{
            "name": "hazard_id",
            "type": {"target": {"kind": "KernelScalar", "value": "String"}},
            "identity": true
        }],
        "assessment": assessment,
        "status": "accepted",
        "provenance": {
            "assertedBy": "Safety owner",
            "assertedAt": "2026-11-26T00:00:00Z"
        }
    });

    assert!(validators["HazardAssessment.json"].is_valid(&hazard["assessment"]));
    assert!(validators["FailureAnalysis.json"].is_valid(&failure_analysis));
    assert!(validators["Hazard.json"].is_valid(&hazard));
    let mut unproven_hazard = hazard;
    unproven_hazard
        .as_object_mut()
        .expect("hazard is an object")
        .remove("provenance");
    assert!(
        !validators["Hazard.json"].is_valid(&unproven_hazard),
        "accepted hazards require provenance"
    );
}

/// The 1-based line of the first line containing `at`; `<text>@2` selects the
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
    for key in [
        "relationVocabulary",
        "bodyExtraction",
        "constructReferences",
        "objectType",
    ] {
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
        let schemas = Schemas::of(case);
        let root = schemas.root(case["schema"].as_str().unwrap());
        let docs = schemas.documents(&root);
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
        let actual_keys = record["availability"]
            .get("recordKeys")
            .cloned()
            .unwrap_or(Value::Null);
        let expected_keys = expected_availability(markdown, &case["availability"]);
        if case["availabilitySubset"] == json!(true) {
            for (key, state) in expected_keys.as_object().into_iter().flatten() {
                assert_eq!(
                    &actual_keys[key], state,
                    "{id}: availability.recordKeys.{key}"
                );
            }
        } else {
            assert_eq!(actual_keys, expected_keys, "{id}: availability.recordKeys");
        }
        for (key, identities) in case["typedRelations"].as_object().into_iter().flatten() {
            assert_eq!(&record[key], identities, "{id}: typed relation {key}");
        }
        if case["modelHasValues"] == json!(true) {
            assert!(record["model"]["values"].is_array(), "{id}: model.values");
        }

        let declaration = extraction.declaration_record().unwrap();
        // The declaration record validates against the case schema exactly
        // when the case says so.
        if let Some(valid) = case.get("recordValid").and_then(Value::as_bool) {
            let validator = compile_documents(&root, &docs).unwrap();
            assert_eq!(
                validator.is_valid(&declaration),
                valid,
                "{id}: declaration record {declaration:#}"
            );
        }
        // Every extracted key rides in the declaration record.
        for (key, value) in record["record"].as_object().into_iter().flatten() {
            assert_eq!(&declaration[key], value, "{id}: declaration key {key}");
        }
        // Each key validates against the module schema the case names.
        for (key, name) in case["validates"].as_object().into_iter().flatten() {
            let name = name.as_str().unwrap();
            let (name, many) = name.strip_suffix("[]").map_or((name, false), |n| (n, true));
            let validator = match name.strip_prefix("core:") {
                Some(core) => core_gate(core),
                None => {
                    let schema = schemas.root(name);
                    compile_documents(&schema, &schemas.documents(&schema)).unwrap()
                }
            };
            let value = &record["record"][key];
            let items: Vec<&Value> = if many {
                value.as_array().unwrap().iter().collect()
            } else {
                vec![value]
            };
            assert!(!items.is_empty(), "{id}: nothing to validate for {key}");
            for item in items {
                assert!(
                    validator.is_valid(item),
                    "{id}: {key} fails {name}: {item:#}"
                );
            }
        }
        if case["modelStatesAbsentFromRecord"] == json!(true) {
            assert!(record["model"]["states"].is_array(), "{id}: model.states");
            assert!(
                record["record"].get("states").is_none(),
                "{id}: record.states"
            );
        }
    }
    assert!(ran > 0, "no case backs {ac}");
}

#[trace("FR-081-AC-1", "FR-081-AC-2", "FR-081-CON-1")]
#[test]
fn assessment_and_lifecycle_keys() {
    run_cases("FR-081-AC-1");
    run_cases("FR-081-AC-2");
}

#[trace("FR-081-AC-3")]
#[test]
fn evidence_targets_resolve_by_the_target_rules() {
    run_cases("FR-081-AC-3");
}

#[trace("FR-081-AC-4")]
#[test]
fn table_and_cell_refusals() {
    run_cases("FR-081-AC-4");
}

#[trace("FR-081-AC-5")]
#[test]
fn blocks_not_of_the_keys_form() {
    run_cases("FR-081-AC-5");
}

#[trace("FR-081-AC-6", "FR-081-CON-2")]
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
    let schemas = Schemas::of(case);
    let root = schemas.root("Lifecycle");
    let docs = schemas.documents(&root);
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

#[trace("FR-081-AC-7")]
#[test]
fn camel_case_headings_and_ambiguous_keys() {
    run_cases("FR-081-AC-7");
}

#[trace("FR-081-AC-8")]
#[test]
fn single_values_and_one_row_policy_tables() {
    run_cases("FR-081-AC-8");
}

#[trace("FR-081-AC-9")]
#[test]
fn routes_name_declared_operations() {
    run_cases("FR-081-AC-9");
}

#[trace("FR-081-AC-10")]
#[test]
fn associated_types_come_from_the_table_only() {
    run_cases("FR-081-AC-10");
}

#[trace("FR-081-AC-11")]
#[test]
fn layout_fence_lowers_to_records_and_endianness() {
    run_cases("FR-081-AC-11");
}

#[trace("FR-081-AC-12")]
#[test]
fn threshold_rows_build_on_exceeded() {
    run_cases("FR-081-AC-12");
}

#[trace("FR-081-AC-13")]
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

#[trace("FR-081-AC-16")]
#[test]
fn members_vocabulary_and_unmapped_keys() {
    run_cases("FR-081-AC-16");
}

#[trace("FR-081-AC-22")]
#[test]
fn object_array_semantic_id_columns_become_typed_record_relations() {
    let schema = json!({
        "$id": "https://example.test/Workflow.json",
        "type": "object",
        "properties": { "Steps/Items": { "type": "array", "items": { "type": "object", "properties": {
            "name": { "type": "string" },
            "receiver~primary": { "$ref": "https://schemas.agent-ix.org/semantic-core/0.1.0/SemanticId.json" }
        } } } }
    });
    let markdown = "---\nid: WF-1\nobject: Workflow\n---\n\n## Steps/Items\n| Name | Receiver~Primary |\n| --- | --- |\n| first | PR-1 |\n| second | PR-2 |\n";
    let req = json!({
        "markdown": markdown,
        "module": { "contractVersion": "1.0.0", "package": "acme/work", "exports": [], "mappings": ["record-sections"] },
        "path": "workflow.md",
        "sourceIdentity": "ix://acme/work/WF-1",
        "dataSchema": schema,
        "schemaDocuments": {},
        "bundle": { "package": "acme/work", "artifacts": [
            { "id": "WF-1", "object": "Workflow", "operations": [] },
            { "id": "PR-1", "object": "Principal", "operations": [] },
            { "id": "PR-2", "object": "Principal", "operations": [] }
        ] },
        "relationVocabulary": { "objectType": "Workflow", "edgeTypes": {}, "roles": {}, "allowedLinks": { "Workflow": { "receiver~primary": ["Principal"] } } },
        "objectType": "Workflow"
    });
    let extracted = serde_json::to_value(extract_semantic_json(&req).unwrap()).unwrap();
    assert_eq!(
        extracted["record"]["Steps/Items"][0]["receiver~primary"],
        "ix://acme/work/PR-1"
    );
    assert_eq!(
        extracted["record"]["Steps/Items"][1]["receiver~primary"],
        "ix://acme/work/PR-2"
    );
    assert_eq!(extracted["recordRelations"].as_array().unwrap().len(), 2);
    assert_eq!(
        extracted["recordRelations"][0]["validationPath"],
        "/record/Steps~1Items/0/receiver~0primary"
    );

    let mut empty = req.clone();
    empty["markdown"] = json!(markdown.replace("| first | PR-1 |", "| first |  |"));
    let empty = serde_json::to_value(extract_semantic_json(&empty).unwrap()).unwrap();
    assert_eq!(empty["recordRelations"].as_array().unwrap().len(), 1);
    assert!(empty["record"]["Steps/Items"][0]
        .get("receiver~primary")
        .is_none());

    for (target, expected) in [
        ("WF-1", "semantic.reference-kind-mismatch"),
        ("MISSING", "semantic.invalid-model-cell"),
    ] {
        let mut invalid = req.clone();
        invalid["markdown"] =
            json!(markdown.replace("| first | PR-1 |", &format!("| first | {target} |")));
        let invalid = serde_json::to_value(extract_semantic_json(&invalid).unwrap()).unwrap();
        assert!(invalid["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["code"] == expected));
        assert!(invalid.get("record").is_none());
        assert!(invalid.get("recordRelations").is_none());
    }
}

/// An indexed multi-repository validation keeps the source member's package
/// for typed record relations even though the shared semantic index has no
/// single package of its own.
#[trace("FR-081-AC-22", "FR-085-AC-16")]
#[test]
fn indexed_multirepo_record_relations_use_the_source_package() {
    let case: Value = serde_json::from_slice(include_bytes!(
        "../fixtures/corpus_cases/plat-1931-indexed-record-relations.json"
    ))
    .expect("PLAT-1931 regression fixture parses");
    assert_eq!(case["issue_ref"], "PLAT-1931");

    let input = &case["input"];
    let manifest = input["module_manifest"]
        .as_str()
        .expect("fixture module manifest");
    let schema_path = input["schema_path"].as_str().expect("fixture schema path");
    let schemas = BTreeMap::from([(schema_path.to_string(), input["schema"].to_string())]);
    let registry = quire_rs::Registry::from_inline_parts(manifest.as_bytes(), &schemas)
        .expect("fixture module loads");
    assert!(registry.failures().is_empty(), "{:?}", registry.failures());
    let workflow = registry
        .archetype("workflow")
        .expect("workflow archetype loads");

    let source = input["source"].as_str().expect("source document");
    let local_event = input["local_event"].as_str().expect("same-member event");
    let wrong_kind_local = input["wrong_kind_local"]
        .as_str()
        .expect("same-member wrong-kind target");
    let remote_event = input["remote_event"].as_str().expect("cross-member event");
    let texts = [source, local_event, wrong_kind_local, remote_event];
    let frontmatter: Vec<_> = texts
        .iter()
        .map(|text| {
            quire_rs::parse_document(text)
                .frontmatter
                .expect("frontmatter")
        })
        .collect();
    let documents: Vec<_> = frontmatter
        .iter()
        .zip(texts)
        .map(|(frontmatter, text)| (frontmatter, text))
        .collect();
    let bundle = quire_rs::semantic::BundleIndex::from_documents(
        input["empty_index_package"]
            .as_str()
            .expect("multi-repository index has no one package"),
        documents.into_iter(),
        registry.semantic_modules().map(|(_, module)| module),
    );
    let source_package = input["source_package"]
        .as_str()
        .expect("source member package");

    let valid = quire_rs::validate_document_in_indexed_bundle(
        &registry,
        workflow,
        source,
        source_package,
        None,
        &bundle,
    );
    assert!(valid.is_valid, "{:?}", valid.errors);
    for reason in case["expect"]["valid_diagnostics_absent"]
        .as_array()
        .expect("valid diagnostics absent")
    {
        let reason = reason.as_str().expect("diagnostic reason");
        assert!(
            !valid
                .errors
                .iter()
                .any(|finding| finding.message.contains(reason))
                && !valid
                    .warnings
                    .iter()
                .any(|finding| finding.message.contains(reason)),
            "valid cross-repository record relation unexpectedly emitted {reason}: errors={:?}, warnings={:?}",
            valid.errors,
            valid.warnings
        );
    }

    let wrong_source = source.replace("| Local | EVENT-001 |", "| Local | PRINCIPAL-001 |");
    let wrong_kind = quire_rs::validate_document_in_indexed_bundle(
        &registry,
        workflow,
        &wrong_source,
        source_package,
        None,
        &bundle,
    );
    let wrong_kind_diagnostic = case["expect"]["wrong_kind_diagnostic"]
        .as_str()
        .expect("wrong-kind diagnostic");
    assert!(
        wrong_kind
            .errors
            .iter()
            .any(|finding| finding.message.contains(wrong_kind_diagnostic))
            || wrong_kind
                .warnings
                .iter()
                .any(|finding| finding.message.contains(wrong_kind_diagnostic)),
        "restoring package context must retain target-kind validation: errors={:?}, warnings={:?}",
        wrong_kind.errors,
        wrong_kind.warnings
    );
}

#[trace("FR-081-AC-17")]
#[test]
fn unreadable_schema_shapes_are_refused_not_prose() {
    run_cases("FR-081-AC-17");
}

#[trace("FR-081-AC-15")]
#[test]
fn model_table_entries_copy_to_record_keys() {
    run_cases("FR-081-AC-15");
}

/// quoin FR-122-AC-7 (TC-2115): every top-level property of every
/// object-type record schema that `agent-ix/spec-objects-architecture` ships
/// has an owner, and none is declared not-authored.
#[trace("FR-081-AC-14")]
#[test]
fn architecture_record_keys_are_all_owned() {
    use quire_rs::semantic::{record_key_owners, RelationVocabulary};
    assert!(
        spec_objects_architecture_fixture::is_available(),
        "the architecture module is not fetched, so nothing was audited: {:?}",
        spec_objects_architecture_fixture::unavailable_reason()
    );
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
