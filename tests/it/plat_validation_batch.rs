//! Validation and diagnostics fixes batched from the old quire-rs issues
//! #451/#453 (regex mismatch), #452 (`max_rows`), #441 (duplicate assert),
//! #440 (record-invalid line), #399 (heading in a comment), #398 (one id,
//! two forms), #397 (`pattern` spelling), #391 (unavailable kind), #222
//! (archetype declared in two blocks).

use std::collections::BTreeMap;
use std::fs;

use quire_rs::semantic::{extract_fields, BundleIndex, SemanticContext};
use quire_rs::{validate_document_in_registry, Registry, ValidationReason, ValidationResult};
use serde_json::{json, Map, Value};

const SCHEMA: &str = r#"{"type":"object"}"#;

fn registry(manifest: &str) -> Registry {
    let mut schemas = BTreeMap::new();
    schemas.insert("schemas/a.schema.json".to_string(), SCHEMA.to_string());
    let registry = Registry::from_inline_parts(manifest.as_bytes(), &schemas).expect("registry");
    assert!(registry.failures().is_empty(), "{:?}", registry.failures());
    registry
}

fn validate(registry: &Registry, archetype: &str, doc: &str) -> ValidationResult {
    let archetype = registry.archetype(archetype).expect("archetype");
    validate_document_in_registry(registry, archetype, doc)
}

fn reasons(result: &ValidationResult) -> Vec<(&'static str, String)> {
    result
        .errors
        .iter()
        .map(|e| (e.reason.as_str(), e.message.clone()))
        .collect()
}

const TABLE_MANIFEST: &str = r#"
name: batch
artifact_types:
- name: doc
  frontmatter_schema_ref: schemas/a.schema.json
  body_extraction:
    yield_pattern:
      match:
        binding:
          from: table_row
          under_section: Binding
          required: true
          assert:
            columns: [Native Type, Constraint]
            min_rows: 1
            max_rows: 1
"#;

#[test]
fn max_rows_refuses_a_table_with_more_rows_than_allowed() {
    let r = registry(TABLE_MANIFEST);
    let one = "## Binding\n| Native Type | Constraint |\n|---|---|\n| Int | [0,9] |\n";
    assert!(validate(&r, "doc", one).is_valid);

    let two =
        "## Binding\n| Native Type | Constraint |\n|---|---|\n| Int | [0,9] |\n| Int | [0,5] |\n";
    let result = validate(&r, "doc", two);
    assert_eq!(
        reasons(&result),
        vec![(
            "assert",
            "[doc] 'binding': table has 2 data row(s) but assert allows at most 1".to_string()
        )]
    );
}

#[test]
fn max_rows_is_refused_on_a_locator_that_is_not_a_table() {
    let manifest = r#"
name: batch
artifact_types:
- name: doc
  body_extraction:
    yield_pattern:
      match:
        body:
          from: section_body
          after_heading: Body
          assert:
            max_rows: 1
"#;
    let outcome = Registry::from_inline_parts(manifest.as_bytes(), &BTreeMap::new()).unwrap();
    assert_eq!(outcome.failures().len(), 1, "{:?}", outcome.failures());
    assert!(outcome.failures()[0].reason.contains("max_rows"));
}

const REGEX_MANIFEST: &str = r#"
name: batch
artifact_types:
- name: doc
  frontmatter_schema_ref: schemas/a.schema.json
  body_extraction:
    yield_pattern:
      match:
        id:
          from: frontmatter_field
          path: [id]
          regex: '^[A-Za-z][A-Za-z0-9_]*$'
          required: true
        binding:
          from: table_row
          under_section: Binding
          regex: '^Int'
          required: true
"#;

#[test]
fn a_present_value_the_regex_rejects_is_not_reported_missing_or_empty() {
    let r = registry(REGEX_MANIFEST);
    let doc = "---\nid: entity-001\n---\n## Binding\n| Type | Constraint |\n|---|---|\n| Float | none |\n";
    let result = validate(&r, "doc", doc);
    let got = reasons(&result);
    assert_eq!(got.len(), 2, "{got:?}");
    assert!(got.iter().all(|(reason, _)| *reason == "regex"), "{got:?}");
    assert!(got[0].1.contains("entity-001") && got[0].1.contains("^[A-Za-z][A-Za-z0-9_]*$"));
    assert!(got[1].1.contains("Float") && got[1].1.contains("^Int"));
    assert!(result
        .errors
        .iter()
        .all(|e| e.reason == ValidationReason::Regex));
}

#[test]
fn an_absent_value_is_still_reported_missing() {
    let r = registry(REGEX_MANIFEST);
    let result = validate(&r, "doc", "---\ntitle: x\n---\nprose\n");
    let got = reasons(&result);
    assert!(
        got.iter()
            .any(|(reason, m)| *reason == "missing" && m.contains("required 'id'")),
        "{got:?}"
    );
    assert!(got.iter().all(|(reason, _)| *reason != "regex"), "{got:?}");
}

#[test]
fn a_matching_value_still_validates() {
    let r = registry(REGEX_MANIFEST);
    let doc =
        "---\nid: entity_one\n---\n## Binding\n| Type | Constraint |\n|---|---|\n| Int | none |\n";
    assert!(validate(&r, "doc", doc).is_valid);
}

#[test]
fn an_object_naming_its_own_type_runs_each_assert_once() {
    let manifest = r#"
name: batch
artifact_types:
- name: entity
  frontmatter_schema_ref: schemas/a.schema.json
  body_extraction:
    yield_pattern:
      match:
        relationships:
          from: table_row
          under_section: Relationships
          assert:
            columns: [Name, Verb, Target, Multiplicity]
"#;
    let r = registry(manifest);
    let doc = "---\nid: e-1\nobject: entity\n---\n## Relationships\n| Name | Verb | Target |\n|---|---|---|\n| a | emits | b |\n";
    let result = validate(&r, "entity", doc);
    let asserts: Vec<_> = result
        .errors
        .iter()
        .filter(|e| e.reason == ValidationReason::Assert)
        .collect();
    assert_eq!(asserts.len(), 1, "{:?}", result.errors);
}

#[test]
fn a_heading_quoted_inside_an_html_comment_does_not_satisfy_a_required_section() {
    let manifest = r#"
name: batch
artifact_types:
- name: doc
  body_extraction:
    yield_pattern:
      match:
        sub:
          from: section_body
          after_heading: Sub-capabilities
          required: true
"#;
    let r = registry(manifest);
    let commented = "## Overview\ntext\n<!--\nAuthoring notes:\n- \"## Sub-capabilities\" (H2, required)\n-->\n## Something Else\nbody\n";
    let result = validate(&r, "doc", commented);
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.reason == ValidationReason::Missing),
        "{:?}",
        result.errors
    );
    let real = "<!--\n## Sub-capabilities\n-->\n## Sub-capabilities\nreal content\n";
    assert!(validate(&r, "doc", real).is_valid);
}

#[test]
fn an_archetype_declared_in_two_blocks_registers_once() {
    let dir = std::env::temp_dir().join(format!("quire-plat-batch-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let module = dir.join("m1");
    fs::create_dir_all(module.join("schemas")).unwrap();
    fs::write(
        module.join("manifest.yaml"),
        "name: m1\nartifact_types:\n- name: a\n  frontmatter_schema_ref: schemas/a.schema.json\narchetypes:\n- name: a\n  frontmatter_schema_ref: schemas/a.schema.json\n",
    )
    .unwrap();
    fs::write(module.join("schemas/a.schema.json"), SCHEMA).unwrap();
    let registry = Registry::load_from(&[dir.as_path()]).unwrap();
    assert!(registry.archetype("a").is_some());
    assert!(
        registry.diagnostics().is_empty(),
        "{:?}",
        registry.diagnostics()
    );
    let _ = fs::remove_dir_all(&dir);
}

fn doc_fm(id: &str, title: &str, object: &str) -> Map<String, Value> {
    let Value::Object(map) =
        json!({"id": id, "title": title, "object": object, "type": "data_schema"})
    else {
        unreachable!()
    };
    map
}

fn type_diagnostics(index: BundleIndex) -> Vec<String> {
    let registry = Registry::load_module(&quire_fixtures::module_ok_dir()).unwrap();
    let module = registry
        .semantic_module("spec-objects-fixture")
        .unwrap()
        .clone();
    let ctx = SemanticContext::new(module, "e.md", index);
    let raw = "---\nid: FR-900\n---\n# T\n\n## Properties\n\n| Field | Type | Multiplicity | Constraints |\n|---|---|---|---|\n| f | ArtifactRecord | 1 | |\n";
    extract_fields(raw, &ctx)
        .diagnostics
        .iter()
        .map(|d| d.code.clone())
        .collect()
}

#[test]
fn one_declaration_in_two_forms_is_not_an_ambiguous_type() {
    let a = doc_fm("data-schema-001", "ArtifactRecord", "data_schema");
    let b = doc_fm("data-schema-001", "ArtifactRecord", "data_schema");
    let index = BundleIndex::from_documents(
        "agent-ix/x",
        [(&a, ""), (&b, "")].into_iter(),
        std::iter::empty(),
    );
    assert_eq!(index.objects.len(), 1, "{:?}", index.objects);
    let codes = type_diagnostics(index);
    assert!(
        !codes.contains(&"semantic.ambiguous-type".to_string()),
        "{codes:?}"
    );

    // Two distinct ids that claim one name are still ambiguous, even when the
    // index lists one of them twice.
    let c = doc_fm("data-schema-002", "ArtifactRecord", "data_schema");
    let mut index = BundleIndex::from_documents(
        "agent-ix/x",
        [(&a, ""), (&c, "")].into_iter(),
        std::iter::empty(),
    );
    index.objects.push(index.objects[0].clone());
    let codes = type_diagnostics(index);
    assert!(
        codes.contains(&"semantic.ambiguous-type".to_string()),
        "{codes:?}"
    );
}

#[test]
fn a_bare_pattern_constraint_names_the_slash_form() {
    let registry = Registry::load_module(&quire_fixtures::module_ok_dir()).unwrap();
    let module = registry
        .semantic_module("spec-objects-fixture")
        .unwrap()
        .clone();
    let ctx = SemanticContext::new(module, "e.md", BundleIndex::default());
    let table = |cell: &str| {
        format!("---\nid: FR-900\n---\n# T\n\n## Properties\n\n| Field | Type | Multiplicity | Constraints |\n|---|---|---|---|\n| digest | String | 1..1 | {cell} |\n")
    };
    let bare = extract_fields(&table("pattern: ^[a-f0-9]{64}$"), &ctx);
    let message = &bare
        .diagnostics
        .iter()
        .find(|d| d.code == "semantic.invalid-constraint-value")
        .unwrap_or_else(|| panic!("{:?}", bare.diagnostics))
        .message;
    assert!(message.contains("pattern: /<regex>/"), "{message}");
    assert!(!bare
        .diagnostics
        .iter()
        .any(|d| d.code == "semantic.unknown-constraint-keyword"));

    let slashed = extract_fields(&table("pattern: /^[a-f0-9]{64}$/, minLength: 64"), &ctx);
    assert!(slashed.diagnostics.is_empty(), "{:?}", slashed.diagnostics);
    assert!(slashed.fields.is_some());
}

#[test]
fn an_unreadable_kind_is_not_validated_as_an_empty_record() {
    let registry = Registry::load_module(&quire_fixtures::module_ok_dir()).unwrap();
    let entity = registry.archetype("entity").unwrap();
    let legacy = quire_fixtures::mapping_fixture("legacy-free-table.md");
    let result = validate_document_in_registry(&registry, entity, &legacy);
    assert!(
        !result
            .errors
            .iter()
            .any(|e| e.message.starts_with("semantic.record-invalid")),
        "{:?}",
        result.errors
    );
    assert!(result
        .warnings
        .iter()
        .any(|w| w.message.starts_with("semantic.legacy-properties-form")));
}

#[test]
fn record_invalid_names_a_line() {
    let registry = Registry::load_module(&quire_fixtures::module_ok_dir()).unwrap();
    let entity = registry.archetype("entity").unwrap();
    // No Properties section at all: the key has no source block, so the
    // finding falls back to the first line rather than carrying none.
    let bare = "---\nid: FR-900\ntitle: Bare\nobject: entity\n---\n# FR-900\n\nprose\n";
    let result = validate_document_in_registry(&registry, entity, bare);
    let finding = result
        .errors
        .iter()
        .find(|e| e.message.starts_with("semantic.record-invalid"))
        .unwrap_or_else(|| panic!("{:?}", result.errors));
    assert_eq!(finding.line, Some(1));
}
