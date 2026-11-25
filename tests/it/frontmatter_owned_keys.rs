//! Frontmatter-owned record keys (FR-069-AC-39..45, FR-081-AC-18..21).
//!
//! The key `cats`, its frontmatter source `raw` and its values are authored in
//! the fixtures below; the engine knows none of them.

use std::collections::BTreeMap;

use ix_trace_rs::trace;
use quire_rs::semantic::python_entry::extract_semantic_json;
use quire_rs::{parse_document, validate_document_in_registry, Registry, ValidationResult};
use serde_json::{json, Value};

const DRAFT: &str = "https://json-schema.org/draft/2020-12/schema";

/// A closed `data_schema` whose `cats` is a list of distinct members of an
/// enum, with at least one.
fn list_schema() -> Value {
    json!({
        "$schema": DRAFT,
        "type": "object",
        "properties": {
            "cats": {
                "type": "array",
                "items": {"enum": ["a", "b"]},
                "minItems": 1,
                "uniqueItems": true
            }
        },
        "additionalProperties": false
    })
}

fn string_schema() -> Value {
    json!({
        "$schema": DRAFT,
        "type": "object",
        "properties": {"cats": {"type": "string"}},
        "additionalProperties": false
    })
}

/// The `cats` locator, as the `yield_pattern.match` YAML of the object type.
fn locator(path: &str, extra: &str) -> String {
    format!(
        "        cats:\n          from: frontmatter_field\n          path: {path}\n          required: false\n{extra}"
    )
}

fn registry(locator: &str, schema: &Value, mappings: &str) -> Registry {
    let manifest = format!(
        "name: owned\nversion: 0.1.0\nartifact_types:\n- name: Doc\nobject_types:\n- name: Thing\n  data_schema:\n    schema: schemas/Thing.json\n  body_extraction:\n    yield_pattern:\n      match:\n{locator}semantic:\n  contract_version: 1.0.0\n  package: agent-ix/owned\n  exports: [Thing]\n  targets: [json-schema, markdown]\n  mappings: [{mappings}]\n"
    );
    let schemas = BTreeMap::from([("schemas/Thing.json".to_string(), schema.to_string())]);
    let registry = Registry::from_inline_parts(manifest.as_bytes(), &schemas).expect("registry");
    assert!(registry.failures().is_empty(), "{:?}", registry.failures());
    registry
}

fn doc(frontmatter: &str, body: &str) -> String {
    format!("---\nid: DOC-001\ntitle: t\ntype: Doc\nobject: Thing\n{frontmatter}---\n\n{body}")
}

fn validate(registry: &Registry, text: &str) -> ValidationResult {
    validate_document_in_registry(registry, registry.archetype("Doc").unwrap(), text)
}

fn line_of(text: &str, needle: &str) -> usize {
    text.lines()
        .position(|l| l.contains(needle))
        .unwrap_or_else(|| panic!("{needle:?} not in document"))
        + 1
}

/// `(line, message)` of each `semantic.record-invalid` error.
fn invalid(result: &ValidationResult) -> Vec<(Option<usize>, String)> {
    result
        .errors
        .iter()
        .filter(|e| e.message.starts_with("semantic.record-invalid"))
        .map(|e| (e.line, e.message.clone()))
        .collect()
}

fn extraction(text: &str, schema: &Value, mappings: &[&str]) -> Value {
    let request = json!({
        "markdown": text,
        "module": {
            "contractVersion": "1.0.0",
            "package": "agent-ix/owned",
            "exports": ["Thing"],
            "targets": ["json-schema", "markdown"],
            "mappings": mappings
        },
        "objectType": "Thing",
        "dataSchema": schema,
        "bodyExtraction": {"yield_pattern": {"match": {"cats": {
            "from": "frontmatter_field", "path": ["raw"], "required": false
        }}}}
    });
    let out = extract_semantic_json(&request).expect("extraction");
    serde_json::to_value(out).expect("serializes")
}

#[trace("FR-069-AC-39")]
#[test]
fn tc_mapped_frontmatter_key_projects_in_authored_order() {
    let registry = registry(&locator("[raw]", ""), &list_schema(), "");
    let text = doc("raw: [b, a]\n", "");
    let result = validate(&registry, &text);
    assert!(result.is_valid, "{:?}", result.errors);

    let out = extraction(&text, &list_schema(), &[]);
    assert_eq!(out["record"], json!({"cats": ["b", "a"]}));
    assert!(out["record"].get("raw").is_none());
}

#[trace("FR-069-AC-40")]
#[test]
fn tc_schema_failures_name_the_key_and_sit_at_the_authored_line() {
    let registry = registry(&locator("[raw]", ""), &list_schema(), "");
    for (value, needle) in [
        ("raw: [x]\n", "x"),
        ("raw: []\n", "less than 1 item"),
        ("raw: [a, a]\n", "non-unique"),
        ("raw: a\n", "is not of type"),
    ] {
        let text = doc(value, "");
        let result = validate(&registry, &text);
        assert!(!result.is_valid, "{value}");
        let findings = invalid(&result);
        assert!(!findings.is_empty(), "{value}: {:?}", result.errors);
        for (line, message) in &findings {
            assert_eq!(*line, Some(line_of(&text, "raw:")), "{value}: {message}");
            assert!(message.contains("cats"), "{value}: {message}");
        }
        assert!(
            findings.iter().any(|(_, m)| m.contains(needle)),
            "{value}: {findings:?}"
        );
    }
}

#[trace("FR-069-AC-41")]
#[test]
fn tc_absent_source_leaves_the_key_absent_and_unfilled() {
    let registry = registry(&locator("[raw]", ""), &list_schema(), "");
    let text = doc("", "");
    assert!(validate(&registry, &text).is_valid);
    let out = extraction(&text, &list_schema(), &[]);
    assert!(out.get("record").is_none_or(Value::is_null), "{out}");
}

#[trace("FR-069-AC-42")]
#[test]
fn tc_multi_segment_path_resolves_the_nested_leaf() {
    let registry = registry(&locator("[outer, inner]", ""), &list_schema(), "");
    let text = doc("outer:\n  inner: [a]\n", "");
    assert!(validate(&registry, &text).is_valid);

    // The leaf is projected as the key and `outer` is not also a property.
    let nested_locator = json!({"yield_pattern": {"match": {"cats": {
        "from": "frontmatter_field", "path": ["outer", "inner"], "required": false
    }}}});
    let mut request = json!({
        "markdown": text,
        "module": {"contractVersion": "1.0.0", "package": "agent-ix/owned",
                   "exports": ["Thing"], "targets": ["json-schema", "markdown"]},
        "objectType": "Thing",
        "dataSchema": list_schema(),
        "bodyExtraction": nested_locator
    });
    let out = serde_json::to_value(extract_semantic_json(&request).unwrap()).unwrap();
    assert_eq!(out["record"], json!({"cats": ["a"]}));

    // An invalid leaf is located at the line of `inner`.
    let bad = doc("outer:\n  inner: [x]\n", "");
    let findings = invalid(&validate(&registry, &bad));
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].0, Some(line_of(&bad, "inner:")));

    // A missing segment records no key.
    for missing in ["", "outer:\n  other: [a]\n"] {
        request["markdown"] = json!(doc(missing, ""));
        let out = serde_json::to_value(extract_semantic_json(&request).unwrap()).unwrap();
        assert!(out.get("record").is_none_or(Value::is_null), "{out}");
    }
}

#[trace("FR-069-AC-43")]
#[test]
fn tc_multiple_and_regex_follow_the_locator_semantics() {
    // `multiple: true` wraps the resolved value: the list `[a]` yields `[["a"]]`,
    // which a schema requiring strings refuses.
    let multiple = registry(
        &locator("[raw]", "          multiple: true\n"),
        &json!({
            "$schema": DRAFT,
            "type": "object",
            "properties": {"cats": {"type": "array", "items": {"type": "string"}}},
            "additionalProperties": false
        }),
        "",
    );
    let text = doc("raw: [a]\n", "");
    let findings = invalid(&validate(&multiple, &text));
    assert!(!findings.is_empty(), "the nested array is refused");
    assert_eq!(findings[0].0, Some(line_of(&text, "raw:")));

    // A `regex` projects its capture, and a non-match leaves the key absent.
    let regex = registry(
        &locator("[raw]", "          regex: '^v-(.+)$'\n"),
        &string_schema(),
        "",
    );
    let matched = doc("raw: v-x\n", "");
    assert!(validate(&regex, &matched).is_valid);
    let unmatched = doc("raw: y\n", "");
    assert!(validate(&regex, &unmatched).is_valid);

    let request = |text: &str| {
        json!({
            "markdown": text,
            "module": {"contractVersion": "1.0.0", "package": "agent-ix/owned",
                       "exports": ["Thing"], "targets": ["json-schema", "markdown"]},
            "objectType": "Thing",
            "dataSchema": string_schema(),
            "bodyExtraction": {"yield_pattern": {"match": {"cats": {
                "from": "frontmatter_field", "path": ["raw"], "required": false,
                "regex": "^v-(.+)$"
            }}}}
        })
    };
    let out = serde_json::to_value(extract_semantic_json(&request(&matched)).unwrap()).unwrap();
    assert_eq!(out["record"], json!({"cats": "x"}));
    let out = serde_json::to_value(extract_semantic_json(&request(&unmatched)).unwrap()).unwrap();
    assert!(out.get("record").is_none_or(Value::is_null), "{out}");

    let wrapped = json!({"yield_pattern": {"match": {"cats": {
        "from": "frontmatter_field", "path": ["raw"], "required": false, "multiple": true
    }}}});
    let mut multi = request(&text);
    multi["bodyExtraction"] = wrapped;
    multi["dataSchema"] = json!({"$schema": DRAFT, "type": "object",
        "properties": {"cats": {"type": "array"}}, "additionalProperties": false});
    let out = serde_json::to_value(extract_semantic_json(&multi).unwrap()).unwrap();
    assert_eq!(out["record"], json!({"cats": [["a"]]}));
}

#[trace("FR-069-AC-44")]
#[test]
fn tc_explicit_null_is_present_not_absent() {
    let registry = registry(&locator("[raw]", ""), &list_schema(), "");
    let text = doc("raw: null\n", "");
    let result = validate(&registry, &text);
    assert!(!result.is_valid);
    let findings = invalid(&result);
    assert!(!findings.is_empty(), "{:?}", result.errors);
    assert!(findings
        .iter()
        .all(|(line, _)| *line == Some(line_of(&text, "raw:"))));

    let out = extraction(&text, &list_schema(), &[]);
    assert_eq!(out["record"], json!({"cats": null}));
}

#[trace("FR-069-AC-45")]
#[test]
fn tc_two_frontmatter_sources_of_one_key_are_a_duplicate() {
    let registry = registry(&locator("[raw]", ""), &list_schema(), "");
    let text = doc("raw: [a]\ncats: [a]\n", "");
    let result = validate(&registry, &text);
    assert!(!result.is_valid);
    let duplicate = result
        .errors
        .iter()
        .find(|e| e.message.starts_with("semantic.duplicate-record-key"))
        .expect("duplicate finding");
    assert_eq!(duplicate.line, Some(line_of(&text, "cats:")));
    assert!(duplicate.message.contains("cats"), "{}", duplicate.message);
    assert!(
        duplicate.message.contains("frontmatter"),
        "{}",
        duplicate.message
    );

    let out = extraction(&text, &list_schema(), &[]);
    assert!(
        out.get("record").is_none_or(|r| r.get("cats").is_none()),
        "{out}"
    );

    // The control: where the authored key is the key, one source is no duplicate.
    let same = registry_same_name();
    let one = doc("cats: [a]\n", "");
    let result = validate(&same, &one);
    assert!(result.is_valid, "{:?}", result.errors);
}

fn registry_same_name() -> Registry {
    registry(&locator("[cats]", ""), &list_schema(), "")
}

fn owner_case() -> Value {
    serde_json::from_str(include_str!("../fixtures/corpus_cases/plat-1874.json"))
        .expect("PLAT-1874 fixture")
}

fn owner_registry(case: &Value) -> Registry {
    let module = &case["module"];
    let locator = &case["locator"];
    let path = serde_json::to_string(&locator["path"]).expect("locator path");
    let targets = module["targets"]
        .as_array()
        .expect("targets")
        .iter()
        .map(|target| target.as_str().expect("target"))
        .collect::<Vec<_>>()
        .join(", ");
    let manifest = format!(
        "name: owner-fixture\nversion: 0.1.0\nartifact_types:\n- name: {}\nobject_types:\n- name: {}\n  data_schema:\n    schema: {}\n  body_extraction:\n    yield_pattern:\n      match:\n        {}: {{ from: {}, path: {}, required: {} }}\nsemantic:\n  contract_version: {}\n  package: {}\n  exports: [{}]\n  targets: [{}]\n",
        module["artifact_type"].as_str().expect("artifact type"),
        module["object_type"].as_str().expect("object type"),
        module["schema_path"].as_str().expect("schema path"),
        locator["key"].as_str().expect("locator key"),
        locator["from"].as_str().expect("locator source"),
        path,
        locator["required"],
        module["contract_version"].as_str().expect("contract version"),
        module["package"].as_str().expect("package"),
        module["object_type"].as_str().expect("export"),
        targets
    );
    let schemas = BTreeMap::from([(
        module["schema_path"]
            .as_str()
            .expect("schema path")
            .to_string(),
        case["schema"].to_string(),
    )]);
    let registry = Registry::from_inline_parts(manifest.as_bytes(), &schemas).expect("registry");
    assert!(registry.failures().is_empty(), "{:?}", registry.failures());
    registry
}

fn owner_extraction(case: &Value, text: &str) -> Value {
    let module = &case["module"];
    let locator = &case["locator"];
    let mut match_fields = serde_json::Map::new();
    match_fields.insert(
        locator["key"].as_str().expect("locator key").to_string(),
        json!({
            "from": locator["from"],
            "path": locator["path"],
            "required": locator["required"]
        }),
    );
    let request = json!({
        "markdown": text,
        "module": {
            "contractVersion": module["contract_version"],
            "package": module["package"],
            "exports": [module["object_type"]],
            "targets": module["targets"]
        },
        "objectType": module["object_type"],
        "dataSchema": case["schema"],
        "bodyExtraction": {"yield_pattern": {"match": match_fields}}
    });
    let out = extract_semantic_json(&request).expect("extraction");
    serde_json::to_value(out).expect("serializes")
}

#[trace("FR-069-AC-34", "FR-069-AC-45", "FR-069-AC-47")]
#[test]
fn tc_scalar_owner_metadata_coexists_with_typed_nested_owner() {
    let case = owner_case();
    assert_eq!(case["issue_ref"], "PLAT-1874");
    let registry = owner_registry(&case);

    let control = case["control"].as_str().expect("healthy control");
    let control_result = validate(&registry, control);
    assert!(control_result.is_valid, "{:?}", control_result.errors);
    assert_eq!(
        owner_extraction(&case, control)["record"],
        case["expect"]["control_record"]
    );

    let collision = case["input"]["collision"]
        .as_str()
        .expect("collision input");
    let collision_result = validate(&registry, collision);
    assert!(collision_result.is_valid, "{:?}", collision_result.errors);
    assert!(!collision_result
        .errors
        .iter()
        .any(|e| { e.message.starts_with("semantic.duplicate-record-key") }));
    let collision_output = owner_extraction(&case, collision);
    assert_eq!(
        collision_output["record"],
        case["expect"]["collision_record"]
    );
    assert_eq!(
        collision_output["diagnostics"],
        case["expect"]["collision_diagnostics"]
    );
    let parsed = parse_document(collision);
    assert_eq!(
        parsed.frontmatter.expect("frontmatter")["owner"],
        case["expect"]["note_metadata_owner"]
    );

    let invalid_doc = case["input"]["invalid_leaf"]
        .as_str()
        .expect("invalid typed owner input");
    let invalid_result = validate(&registry, invalid_doc);
    assert!(!invalid_result.is_valid);
    let findings = invalid(&invalid_result);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(
        findings[0]
            .1
            .contains(case["expect"]["invalid_record_error"].as_str().unwrap()),
        "{findings:?}"
    );
    let leaf_key = case["expect"]["invalid_record_error_source"]
        .as_str()
        .unwrap()
        .rsplit('.')
        .next()
        .unwrap();
    assert_eq!(
        findings[0].0,
        Some(line_of(invalid_doc, &format!("  {leaf_key}: 42"))),
        "{findings:?}"
    );
    assert!(!invalid_result
        .errors
        .iter()
        .any(|e| { e.message.starts_with("semantic.duplicate-record-key") }));
}

fn section_text(body: &str) -> String {
    doc("", body)
}

/// The section `## cats` under an object type owning `cats` in frontmatter.
fn section_extraction(frontmatter: &str, mappings: &[&str]) -> Value {
    let text = doc(frontmatter, "## cats\n\n- a\n- b\n");
    extraction(&text, &list_schema(), mappings)
}

fn reasons(out: &Value) -> Vec<(String, String)> {
    out["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["code"] == "semantic.feature-not-extractable")
        .map(|d| {
            (
                d["reason"].as_str().unwrap_or_default().to_string(),
                d["severity"].as_str().unwrap_or_default().to_string(),
            )
        })
        .collect()
}

#[trace("FR-081-AC-18")]
#[test]
fn tc_section_without_a_frontmatter_value_is_an_error_and_yields_nothing() {
    let out = section_extraction("", &["record-sections"]);
    assert_eq!(
        reasons(&out),
        [("frontmatter-owned".into(), "error".into())]
    );
    assert!(out.get("record").is_none_or(Value::is_null), "{out}");
    assert!(out.get("recordSources").is_none_or(Value::is_null), "{out}");
    assert!(
        out["availability"]
            .get("recordKeys")
            .is_none_or(|k| k.get("cats").is_none()),
        "{out}"
    );

    let registry = registry(&locator("[raw]", ""), &list_schema(), "record-sections");
    let result = validate(&registry, &section_text("## cats\n\n- a\n"));
    assert!(!result.is_valid);
}

#[trace("FR-081-AC-19")]
#[test]
fn tc_section_and_frontmatter_value_keep_the_value_and_warn() {
    let out = section_extraction("raw: [b]\n", &["record-sections"]);
    assert_eq!(
        reasons(&out),
        [("frontmatter-owned".into(), "warning".into())]
    );
    assert_eq!(out["record"], json!({"cats": ["b"]}));
    assert!(out.get("recordSources").is_none_or(Value::is_null), "{out}");
    assert!(
        out["availability"]
            .get("recordKeys")
            .is_none_or(|k| k.get("cats").is_none()),
        "{out}"
    );
    assert!(
        !out["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["code"] == "semantic.duplicate-record-key"),
        "{out}"
    );

    let registry = registry(&locator("[raw]", ""), &list_schema(), "record-sections");
    let result = validate(&registry, &doc("raw: [b]\n", "## cats\n\n- a\n"));
    assert!(result.is_valid, "{:?}", result.errors);
    assert!(
        result
            .warnings
            .iter()
            .any(|w| w.message.contains("frontmatter-owned")
                || w.message.contains("feature-not-extractable")),
        "{:?}",
        result.warnings
    );
}

#[trace("FR-081-AC-20")]
#[test]
fn tc_frontmatter_value_without_a_section_has_no_diagnostic_or_availability() {
    let text = doc("raw: [b]\n", "");
    let out = extraction(&text, &list_schema(), &["record-sections"]);
    assert!(reasons(&out).is_empty(), "{out}");
    assert_eq!(out["record"], json!({"cats": ["b"]}));
    assert!(out.get("recordSources").is_none_or(Value::is_null), "{out}");
    assert!(
        out["availability"]
            .get("recordKeys")
            .is_none_or(|k| k.get("cats").is_none()),
        "{out}"
    );
}

#[trace("FR-081-AC-21")]
#[test]
fn tc_without_the_token_the_section_is_prose() {
    let out = section_extraction("", &[]);
    assert!(reasons(&out).is_empty(), "{out}");
    assert!(out.get("record").is_none_or(Value::is_null), "{out}");
}
