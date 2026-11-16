//! FR-096 generic flow-rule evaluation, loading and reporting.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use ix_trace_rs::trace;
use quire_rs::corpus::{relation_report, validate_bundle, RelationReportMember, Spec};
use quire_rs::{BundlePosture, Registry};
use tempfile::TempDir;

fn schema(kind: &str, fields: &str) -> String {
    format!(
        r#"{{"type":"object","properties":{{"id":{{"type":"string"}},"type":{{"const":"{kind}"}}{fields}}}}}"#
    )
}

fn registry_with(rules: &str, remediations: &str, exclusions: &str) -> Registry {
    let remediations = if remediations.trim() == "[]" {
        "    []".to_string()
    } else {
        remediations.trim_end().to_string()
    };
    let manifest = format!(
        r#"name: flow-rules-fixture
version: 0.1.0
description: Generic flow rule fixture
archetypes:
- name: alpha
  frontmatter_schema_ref: schemas/alpha.json
  allowed_links:
    links_to: [beta]
    crosses: [gamma]
- name: beta
  frontmatter_schema_ref: schemas/beta.json
  allowed_links:
    carries: [gamma]
- name: gamma
  frontmatter_schema_ref: schemas/gamma.json
- name: control
  frontmatter_schema_ref: schemas/control.json
  allowed_links:
    protects: [alpha]
- name: sink
  frontmatter_schema_ref: schemas/sink.json
  allowed_links:
    delivers_to: [alpha]
edge_types:
  links_to: {{description: links alpha to beta, category: traceability, inverse: linked_from}}
  crosses: {{description: alpha crosses gamma, category: traceability}}
  carries: {{description: beta carries gamma, category: traceability}}
  protects: {{description: control protects alpha, category: traceability}}
  delivers_to: {{description: sink delivers to alpha, category: traceability}}
traceability:
  exclude: {exclusions}
  required_relations:
  - name: sink-delivers
    from: sink
    edges: [delivers_to]
    to: [alpha]
    direction: outgoing
    check: relation-check
  flow_rules:
{rules}  remediations:
{remediations}"#
    );
    let mut schemas = BTreeMap::new();
    schemas.insert(
        "schemas/alpha.json".into(),
        schema(
            "alpha",
            r#", "state":{"enum":["open","closed"]}, "approval":{"enum":["covered","missing"]}"#,
        ),
    );
    schemas.insert(
        "schemas/beta.json".into(),
        schema(
            "beta",
            r#", "category":{"type":"array"}, "single":{"enum":["x","y"]}"#,
        ),
    );
    schemas.insert(
        "schemas/gamma.json".into(),
        schema("gamma", r#", "trust":{"enum":["trusted","untrusted"]}"#),
    );
    schemas.insert(
        "schemas/control.json".into(),
        schema("control", r#", "safeguards":{"enum":["encrypt","mask"]}"#),
    );
    schemas.insert(
        "schemas/sink.json".into(),
        schema("sink", r#", "kind":{"enum":["prompt","storage"]}, "agreement":{"enum":["covered","missing","unknown"]}"#),
    );
    Registry::from_inline_parts(manifest.as_bytes(), &schemas).expect("module parses")
}

fn docs(files: &[(&str, &str)]) -> (TempDir, Spec) {
    let root = tempfile::tempdir().expect("tempdir");
    for (path, content) in files {
        let full = root.path().join("spec").join(path);
        fs::create_dir_all(full.parent().unwrap()).expect("parent");
        fs::write(full, content).expect("document");
    }
    let spec = Spec::from_path(&root.path().join("spec"));
    (root, spec)
}

fn document(id: &str, kind: &str, values: &str, links: &[(&str, &str)]) -> String {
    let mut body = format!("---\nid: {id}\ntype: {kind}\n{values}");
    if !links.is_empty() {
        body.push_str("relationships:\n");
        for (edge, target) in links {
            body.push_str(&format!("  - type: {edge}\n    target: {target}\n"));
        }
    }
    body.push_str("---\n# document\n");
    body
}

fn report(root: &Path, spec: &Spec, registry: &Registry) -> quire_rs::corpus::RelationReport {
    relation_report(
        spec,
        registry,
        &[RelationReportMember { root, matrix: &[] }],
        &[],
    )
    .expect("flow report")
}

#[test]
#[trace(
    "FR-096-AC-1",
    "FR-096-AC-3",
    "FR-096-AC-8",
    "FR-096-AC-10",
    "FR-096-AC-11",
    "FR-096-AC-14",
    "FR-096-AC-16",
    "FR-096-AC-21",
    "FR-096-CON-2",
    "FR-096-CON-3",
    "FR-096-CON-4"
)]
fn generic_path_matches_authored_facts_and_reports_exact_matched_endpoints() {
    let registry = registry_with(
        "  - name: category-rule\n    subject: [alpha]\n    match:\n    - path: [{edge: links_to, direction: outgoing, to: [beta]}]\n      field: category\n      any_of: [health]\n    unless_any:\n    - path: [{edge: protects, direction: incoming, to: [control]}]\n      field: safeguards\n      any_of: [encrypt]\n    check: category-crossing\n    remediations: [encrypt]\n    rationale: Protect the declared category.\n",
        "  - name: encrypt\n    description: Add a control protecting the alpha.\n    adds: {edge: protects, direction: incoming, from: control, field: safeguards, value: encrypt}\n",
        "[]",
    );
    assert!(registry.failures().is_empty(), "{:?}", registry.failures());
    let (root, spec) = docs(&[
        (
            "a.md",
            &document(
                "A-1",
                "alpha",
                "state: open\n",
                &[("links_to", "B-1"), ("links_to", "B-2")],
            ),
        ),
        (
            "b1.md",
            &document(
                "B-1",
                "beta",
                "category: [ ' Health ', 7 ]\nunread: secret\n",
                &[],
            ),
        ),
        (
            "b2.md",
            &document("B-2", "beta", "category: [personal]\n", &[]),
        ),
    ]);
    let result = report(root.path(), &spec, &registry);
    let flow = result.flow.expect("declared report");
    assert_eq!(flow.denials.len(), 1);
    assert_eq!(flow.denials[0].subject.id, "A-1");
    assert_eq!(flow.denials[0].subject.archetype, "alpha");
    assert_eq!(flow.denials[0].matched["beta"].len(), 1);
    assert_eq!(flow.denials[0].matched["beta"][0].id, "B-1");
    assert_eq!(flow.denials[0].severity, "warning");
    assert_eq!(
        flow.facts.keys().cloned().collect::<Vec<_>>(),
        ["alpha", "beta", "control"]
    );
    assert!(flow.facts["alpha"][0].fields.is_empty());
    assert_eq!(flow.facts["alpha"][0].edges.len(), 2);
    assert_eq!(
        flow.facts["beta"][0]
            .fields
            .keys()
            .cloned()
            .collect::<Vec<_>>(),
        ["category"]
    );
    let serialized = serde_json::to_string(&flow).expect("json");
    assert!(serialized.find("\"path\"").unwrap() < serialized.find("\"fields\"").unwrap());
    assert!(serialized.contains("\"edges\":[{\"edge\":\"links_to\",\"direction\":\"outgoing\",\"target\":{\"id\":\"B-1\"},\"archetype\":\"beta\"}"));

    let validation = validate_bundle(
        &spec,
        &registry,
        BundlePosture::Okf,
        root.path(),
        root.path(),
    );
    assert_eq!(validation.warnings.len(), 1);
    assert!(validation.warnings[0].message.contains("B-1"));
    assert!(!validation.warnings[0].message.contains("B-2"));
}

#[test]
#[trace("FR-096-AC-2", "FR-096-AC-4", "FR-096-AC-9")]
fn any_path_and_absent_present_cover_empty_and_multi_hop_paths() {
    let registry = registry_with(
        "  - name: any-path-rule\n    subject: [sink]\n    match:\n    - any_path:\n      - [{edge: delivers_to, direction: outgoing, to: [alpha]}, {edge: links_to, direction: outgoing, to: [beta]}]\n      - [{edge: delivers_to, direction: outgoing, to: [alpha]}, {edge: links_to, direction: outgoing, to: [beta]}]\n      field: category\n      any_of: [health]\n    check: any-path-match\n    rationale: Find a category over either path.\n  - name: absent-rule\n    subject: [alpha]\n    match:\n    - {path: [], field: approval, absent: true}\n    check: absent-approval\n    rationale: Approval is unstated.\n",
        "[]",
        "[]",
    );
    assert!(registry.failures().is_empty(), "{:?}", registry.failures());
    let (root, spec) = docs(&[
        (
            "sink.md",
            &document("S-1", "sink", "kind: prompt\n", &[("delivers_to", "A-1")]),
        ),
        (
            "alpha.md",
            &document("A-1", "alpha", "", &[("links_to", "B-1")]),
        ),
        (
            "beta.md",
            &document("B-1", "beta", "category: [health]\n", &[]),
        ),
    ]);
    let result = report(root.path(), &spec, &registry);
    let flow = result.flow.unwrap();
    assert_eq!(flow.denials.len(), 2);
    assert!(flow
        .denials
        .iter()
        .any(|denial| denial.rule == "any-path-rule"));
    assert!(flow
        .denials
        .iter()
        .any(|denial| denial.rule == "absent-rule"));
}

#[test]
#[trace("FR-096-AC-5", "FR-096-AC-6", "FR-096-AC-24", "FR-096-AC-25")]
fn loader_rejects_unknown_registry_references_values_and_check_collisions() {
    let cases = [
        (
            "  - name: bad-subject\n    subject: [missing]\n    match: [{path: [], field: id, present: true}]\n    check: bad-subject\n    rationale: Bad subject.\n",
            "bad-subject",
        ),
        (
            "  - name: bad-target\n    subject: [alpha]\n    match: [{path: [{edge: links_to, direction: outgoing, to: [missing]}], field: id, present: true}]\n    check: bad-target\n    rationale: Bad target.\n",
            "bad-target",
        ),
        (
            "  - name: bad-edge\n    subject: [alpha]\n    match: [{path: [{edge: missing, direction: outgoing, to: [beta]}], field: single, any_of: [x]}]\n    check: bad-edge\n    rationale: Bad edge.\n",
            "bad-edge",
        ),
        (
            "  - name: bad-field\n    subject: [alpha]\n    match: [{path: [], field: typo, present: true}]\n    check: bad-field\n    rationale: Bad field.\n",
            "typo",
        ),
        (
            "  - name: bad-enum\n    subject: [alpha]\n    match: [{path: [], field: state, any_of: [medical]}]\n    check: bad-enum\n    rationale: Bad enum.\n",
            "medical",
        ),
        (
            "  - name: built-in-check\n    subject: [alpha]\n    match: [{path: [], field: id, present: true}]\n    check: dangling-trace-reference\n    rationale: Collision.\n",
            "dangling-trace-reference",
        ),
        (
            "  - name: relation-check\n    subject: [alpha]\n    match: [{path: [], field: id, present: true}]\n    check: relation-check\n    rationale: Collision.\n",
            "relation-check",
        ),
    ];
    for (rules, expected) in cases {
        let registry = registry_with(rules, "[]", "[]");
        assert!(
            registry
                .failures()
                .iter()
                .any(|failure| failure.reason.contains(expected)),
            "expected {expected:?}, got {:?}",
            registry.failures()
        );
    }
    let valid = registry_with(
        "  - name: valid-enum\n    subject: [alpha]\n    match: [{path: [], field: state, any_of: [' OPEN ']}]\n    check: valid-enum\n    rationale: Valid folded enum.\n",
        "[]",
        "[]",
    );
    assert!(valid.failures().is_empty(), "{:?}", valid.failures());
}

#[test]
#[trace("FR-096-AC-7", "FR-096-CON-1")]
fn remediation_must_clear_named_rule_and_model_survives_registry_merge() {
    let rules = "  - name: removable\n    subject: [alpha]\n    match: [{path: [{edge: links_to, direction: outgoing, to: [beta]}], field: single, any_of: [x]}]\n    check: removable\n    remediations: [remove-link]\n    rationale: A link sustains this denial.\n";
    let valid = registry_with(
        rules,
        "  - name: remove-link\n    description: Remove the outgoing link.\n    removes: {edge: links_to, direction: outgoing}\n",
        "[]",
    );
    assert!(valid.failures().is_empty(), "{:?}", valid.failures());
    let model = valid.traceability().expect("traceability");
    assert_eq!(model.flow_rules.len(), 1);
    assert_eq!(model.remediations.len(), 1);
    assert!(!model.is_empty());

    let invalid = registry_with(
        rules,
        "  - name: remove-link\n    description: Remove another edge.\n    removes: {edge: crosses, direction: outgoing}\n",
        "[]",
    );
    assert!(invalid.failures().iter().any(|failure| {
        failure.reason.contains("removable") && failure.reason.contains("remove-link")
    }));
}

#[test]
#[trace("FR-096-AC-12", "FR-096-AC-19", "FR-096-AC-20", "FR-096-CON-5")]
fn severity_off_exclusion_and_repeat_runs_keep_validation_and_report_aligned() {
    let registry = registry_with(
        "  - name: approval-rule\n    subject: [alpha]\n    match: [{path: [], field: approval, absent: true}]\n    check: approval-check\n    rationale: Approval is required.\n",
        "[]",
        "[\"spec/excluded/**\"]",
    );
    let (root, spec) = docs(&[
        ("included.md", &document("A-1", "alpha", "", &[])),
        ("excluded/hidden.md", &document("A-2", "alpha", "", &[])),
    ]);
    let first = report(root.path(), &spec, &registry);
    let second = report(root.path(), &spec, &registry);
    let flow = first.flow.as_ref().unwrap();
    assert_eq!(
        serde_json::to_vec(&first).unwrap(),
        serde_json::to_vec(&second).unwrap()
    );
    assert_eq!(flow.denials.len(), 1);
    assert_eq!(flow.denials[0].subject.id, "A-1");
    let validation = validate_bundle(
        &spec,
        &registry,
        BundlePosture::Strict,
        root.path(),
        root.path(),
    );
    assert_eq!(validation.warnings.len(), 1);

    let off = registry.with_grammar_severity(BTreeMap::from([(
        "trace:approval-check".into(),
        quire_rs::grammar::GrammarSeverityLevel::Off,
    )]));
    assert!(report(root.path(), &spec, &off)
        .flow
        .unwrap()
        .denials
        .is_empty());
    assert!(
        validate_bundle(&spec, &off, BundlePosture::Strict, root.path(), root.path())
            .warnings
            .is_empty()
    );
}

#[test]
#[trace(
    "FR-096-AC-22",
    "FR-096-AC-23",
    "FR-096-CON-2",
    "FR-096-CON-3",
    "FR-096-CON-4"
)]
fn engine_source_remains_generic_and_does_not_apply_remediations() {
    let source = include_str!("../../src/corpus/flow_rules.rs")
        .split("#[cfg(test)]")
        .next()
        .unwrap();
    for domain_term in ["regulated_categories", "trust_level", "safeguards"] {
        assert!(!source.contains(domain_term));
    }
    assert!(!source.contains("write("));
}

#[test]
#[trace("FR-096-AC-13", "FR-096-AC-15", "FR-096-AC-17")]
fn report_keeps_rule_and_vocabulary_order_and_uses_full_rule_shape() {
    let registry = registry_with(
        "  - name: z-rule\n    subject: [alpha]\n    match: [{path: [], field: state, any_of: [open]}]\n    check: z-check\n    rationale: Z rule.\n  - name: a-rule\n    subject: [alpha]\n    match: [{path: [], field: state, any_of: [closed]}]\n    check: a-check\n    rationale: A rule.\n",
        "[]",
        "[]",
    );
    let (root, spec) = docs(&[("a.md", &document("A-1", "alpha", "state: open\n", &[]))]);
    let result = report(root.path(), &spec, &registry);
    let flow = result.flow.unwrap();
    assert_eq!(flow.rules[0].name, "z-rule");
    assert_eq!(flow.rules[1].name, "a-rule");
    let json = serde_json::to_string(&flow).unwrap();
    assert!(json.contains("\"unless_any\":[]"));
    assert!(json.find("\"schema\"").unwrap() < json.find("\"rules\"").unwrap());
    assert_eq!(flow.vocabularies["alpha"]["state"], ["open", "closed"]);
}
