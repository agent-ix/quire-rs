//! FR-096 generic flow-rule evaluation, loading and reporting.

use std::collections::BTreeMap;
use std::fs;
use std::net::TcpListener;
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
    parse_registry_with(rules, remediations, exclusions).expect("module parses")
}

fn parse_registry_with(
    rules: &str,
    remediations: &str,
    exclusions: &str,
) -> Result<Registry, String> {
    let mut schemas = BTreeMap::new();
    schemas.insert(
        "schemas/alpha.json".into(),
        schema(
            "alpha",
            r#", "state":{"enum":["open","closed"]}, "approval":{"enum":["covered","missing"]}, "nested":{"type":"object","properties":{"nested_only":{"type":"string"}}}"#,
        ),
    );
    schemas.insert(
        "schemas/beta.json".into(),
        schema(
            "beta",
            r#", "category":{}, "classification":{"type":"array","items":{"enum":["health","personal","financial"]}}, "single":{"enum":["x","y"]}"#,
        ),
    );
    schemas.insert(
        "schemas/gamma.json".into(),
        schema("gamma", r#", "trust":{"enum":["trusted","untrusted"]}, "category":{"enum":["health","personal","financial"]}"#),
    );
    schemas.insert(
        "schemas/control.json".into(),
        schema(
            "control",
            r#", "safeguards":{"enum":["encrypt","encrypt-alt","mask"]}"#,
        ),
    );
    schemas.insert(
        "schemas/sink.json".into(),
        schema("sink", r#", "kind":{"enum":["prompt","storage"]}, "agreement":{"enum":["covered","missing","unknown"]}"#),
    );
    parse_registry_with_schemas(rules, remediations, exclusions, schemas)
}

fn parse_registry_with_schemas(
    rules: &str,
    remediations: &str,
    exclusions: &str,
    schemas: BTreeMap<String, String>,
) -> Result<Registry, String> {
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
    delivers_to: [sink]
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
    carries: [gamma]
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
    direction: incoming
    check: relation-check
  flow_rules:
{rules}  remediations:
{remediations}"#
    );
    Registry::from_inline_parts(manifest.as_bytes(), &schemas).map_err(|error| error.to_string())
}

fn parse_local_ref_registry(
    rules: &str,
    sink_schema: &str,
    referenced_schemas: &[(&str, &str)],
) -> Result<Registry, String> {
    let manifest = format!(
        r#"name: local-ref-fixture
version: 0.1.0
description: Local schema reference fixture
archetypes:
- name: sink
  frontmatter_schema_ref: schemas/sink.json
traceability:
  flow_rules:
{rules}  remediations: []
"#
    );
    let mut schemas = BTreeMap::from([("schemas/sink.json".to_string(), sink_schema.to_string())]);
    for (path, content) in referenced_schemas {
        schemas.insert((*path).to_string(), (*content).to_string());
    }
    Registry::from_inline_parts(manifest.as_bytes(), &schemas).map_err(|error| error.to_string())
}

#[test]
#[trace("FR-096-AC-17", "FR-096-AC-26")]
fn flow_rule_any_of_resolves_a_local_schema_enum() {
    let base = "https://schemas.agent-ix.org/flow-rules-fixture/0.1.0/schemas";
    let root_schema = format!(
        r#"{{"$schema":"https://json-schema.org/draft/2020-12/schema","$id":"{base}/sink.json","type":"object","properties":{{"agreement":{{"$ref":"{base}/ProcessorAgreement.json"}}}}}}"#
    );
    let agreement_schema = format!(
        r#"{{"$schema":"https://json-schema.org/draft/2020-12/schema","$id":"{base}/ProcessorAgreement.json","enum":["covered","not_covered","unknown"]}}"#
    );
    let rules = "  - name: agreement-rule\n    subject: [sink]\n    match: [{path: [], field: agreement, any_of: [covered, not_covered, unknown]}]\n    check: agreement-check\n    rationale: Restrict agreement values.\n";
    let registry = parse_local_ref_registry(
        rules,
        &root_schema,
        &[("schemas/ProcessorAgreement.json", &agreement_schema)],
    )
    .expect("module parses");
    assert!(registry.failures().is_empty(), "{:?}", registry.failures());
    let sink = registry.archetype("sink").expect("sink archetype");
    let documents = sink
        .frontmatter_schema_documents
        .as_ref()
        .expect("resolved documents");
    assert!(documents.contains_key(&format!("{base}/ProcessorAgreement.json")));
    let (root, spec) = docs(&[(
        "sink.md",
        &document("S-1", "sink", "agreement: covered\n", &[]),
    )]);
    let flow = report(root.path(), &spec, &registry).flow.unwrap();
    assert_eq!(
        flow.vocabularies["sink"]["agreement"],
        ["covered", "not_covered", "unknown"]
    );

    let invalid_rules = "  - name: agreement-rule\n    subject: [sink]\n    match: [{path: [], field: agreement, any_of: [medical]}]\n    check: agreement-check\n    rationale: Restrict agreement values.\n";
    let invalid = parse_local_ref_registry(
        invalid_rules,
        &root_schema,
        &[("schemas/ProcessorAgreement.json", &agreement_schema)],
    )
    .expect("module loads so flow validation reports the invalid value");
    assert!(
        invalid.failures().iter().any(|failure| {
            failure.reason.contains("agreement-rule")
                && failure.reason.contains("agreement")
                && failure.reason.contains("medical")
        }),
        "{:?}",
        invalid.failures()
    );
}

#[test]
#[trace("FR-096-AC-26")]
fn flow_rule_any_of_resolves_a_local_array_item_enum() {
    let base = "https://schemas.agent-ix.org/flow-rules-fixture/0.1.0/schemas";
    let root_schema = format!(
        r#"{{"$schema":"https://json-schema.org/draft/2020-12/schema","$id":"{base}/sink.json","type":"object","properties":{{"agreements":{{"type":"array","items":{{"$ref":"{base}/ProcessorAgreement.json"}}}}}}}}"#
    );
    let agreement_schema = format!(
        r#"{{"$schema":"https://json-schema.org/draft/2020-12/schema","$id":"{base}/ProcessorAgreement.json","enum":["covered","not_covered","unknown"]}}"#
    );
    let rules = "  - name: agreement-rule\n    subject: [sink]\n    match: [{path: [], field: agreements, any_of: [covered]}]\n    check: agreement-check\n    rationale: Restrict agreement values.\n";
    let registry = parse_local_ref_registry(
        rules,
        &root_schema,
        &[("schemas/ProcessorAgreement.json", &agreement_schema)],
    )
    .expect("module parses");
    assert!(registry.failures().is_empty(), "{:?}", registry.failures());

    let invalid_rules = "  - name: agreement-rule\n    subject: [sink]\n    match: [{path: [], field: agreements, any_of: [medical]}]\n    check: agreement-check\n    rationale: Restrict agreement values.\n";
    let invalid = parse_local_ref_registry(
        invalid_rules,
        &root_schema,
        &[("schemas/ProcessorAgreement.json", &agreement_schema)],
    )
    .expect("module parses");
    assert!(
        invalid.failures().iter().any(|failure| {
            failure.reason.contains("agreement-rule")
                && failure.reason.contains("agreements")
                && failure.reason.contains("medical")
        }),
        "{:?}",
        invalid.failures()
    );
}

/// Whether `failure` drops the archetype `name` for an unresolvable schema reference.
fn archetype_drop(failure: &quire_rs::ArchetypeLoadFailure, name: &str) -> bool {
    failure
        .declaration
        .as_ref()
        .is_some_and(|d| d.kind.as_str() == "archetype" && d.item == name)
}

#[test]
#[trace("FR-096-AC-27")]
fn flow_rule_enum_refs_fail_closed_when_missing_or_cyclic() {
    let base = "https://schemas.agent-ix.org/flow-rules-fixture/0.1.0/schemas";
    let rules = "  - name: agreement-rule\n    subject: [sink]\n    match: [{path: [], field: agreement, any_of: [covered]}]\n    check: agreement-check\n    rationale: Restrict agreement values.\n";
    let missing_root = format!(
        r#"{{"$schema":"https://json-schema.org/draft/2020-12/schema","$id":"{base}/sink.json","type":"object","properties":{{"agreement":{{"$ref":"{base}/Missing.json"}}}}}}"#
    );
    let missing = parse_local_ref_registry(rules, &missing_root, &[]).expect("module parses");
    assert!(
        missing.failures().iter().any(|failure| {
            archetype_drop(failure, "sink")
                && failure.reason.contains("schemas/sink.json")
                && failure.reason.contains("Missing.json")
        }),
        "{:?}",
        missing.failures()
    );

    let cycle_root = format!(
        r#"{{"$schema":"https://json-schema.org/draft/2020-12/schema","$id":"{base}/sink.json","type":"object","properties":{{"agreement":{{"$ref":"{base}/A.json"}}}}}}"#
    );
    let a_schema = format!(r#"{{"$id":"{base}/A.json","$ref":"{base}/B.json"}}"#);
    let b_schema = format!(r#"{{"$id":"{base}/B.json","$ref":"{base}/A.json"}}"#);
    let cyclic = parse_local_ref_registry(
        rules,
        &cycle_root,
        &[("schemas/A.json", &a_schema), ("schemas/B.json", &b_schema)],
    )
    .expect("module parses");
    assert!(
        cyclic.failures().iter().any(|failure| {
            archetype_drop(failure, "sink")
                && failure.reason.contains("schemas/sink.json")
                && failure.reason.contains("$ref cycle")
        }),
        "{:?}",
        cyclic.failures()
    );

    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind local no-fetch probe");
    listener
        .set_nonblocking(true)
        .expect("nonblocking no-fetch probe");
    let address = listener.local_addr().expect("probe address");
    let remote_base = format!("http://{address}/schemas");
    let remote_root = format!(
        r#"{{"$schema":"https://json-schema.org/draft/2020-12/schema","$id":"{remote_base}/sink.json","type":"object","properties":{{"agreement":{{"$ref":"{remote_base}/Remote.json"}}}}}}"#
    );
    let remote = parse_local_ref_registry(rules, &remote_root, &[]).expect("module parses");
    assert!(
        remote.failures().iter().any(|failure| {
            archetype_drop(failure, "sink")
                && failure.reason.contains("schemas/sink.json")
                && failure.reason.contains("Remote.json")
                && failure.reason.contains("no shipped file")
        }),
        "{:?}",
        remote.failures()
    );
    assert!(
        matches!(
            listener.accept(),
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock
        ),
        "remote schema references must not issue an HTTP request"
    );
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
        "  - name: category-rule\n    subject: [alpha]\n    match:\n    - path: [{edge: links_to, direction: outgoing, to: [beta]}]\n      field: category\n      any_of: [health]\n    unless_any:\n    - path: [{edge: protects, direction: incoming, to: [control]}]\n      field: safeguards\n      any_of: [encrypt, encrypt-alt]\n    check: category-crossing\n    remediations: [encrypt, encrypt-alt]\n    rationale: Protect the declared category.\n",
        "  - name: encrypt\n    description: Add a control protecting the alpha.\n    adds: {edge: protects, direction: incoming, from: control, field: safeguards, value: encrypt}\n  - name: encrypt-alt\n    description: Add an alternate control protecting the alpha.\n    adds: {edge: protects, direction: incoming, from: control, field: safeguards, value: encrypt-alt}\n",
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
        ("b3.md", &document("B-3", "beta", "category: 7\n", &[])),
        (
            "b4.md",
            &document("B-4", "beta", "category: {nested: health}\n", &[]),
        ),
        ("b5.md", &document("B-5", "beta", "category: true\n", &[])),
        ("b6.md", &document("B-6", "beta", "category: null\n", &[])),
        ("b7.md", &document("B-7", "beta", "", &[])),
    ]);
    let result = report(root.path(), &spec, &registry);
    let flow = result.flow.expect("declared report");
    assert_eq!(flow.denials.len(), 1);
    assert_eq!(flow.denials[0].subject.id, "A-1");
    assert_eq!(flow.denials[0].subject.archetype, "alpha");
    assert_eq!(flow.denials[0].matched["beta"].len(), 1);
    assert_eq!(flow.denials[0].matched["beta"][0].id, "B-1");
    assert_eq!(flow.denials[0].severity, "warning");
    assert_eq!(flow.denials[0].remediations.len(), 2);
    let remediation = &flow.denials[0].remediations[0];
    assert_eq!(remediation.name, "encrypt");
    assert_eq!(
        remediation.description,
        "Add a control protecting the alpha."
    );
    assert_eq!(flow.denials[0].remediations[1].name, "encrypt-alt");
    assert_eq!(
        remediation.change.adds.as_ref().map(|change| (
            change.edge.as_str(),
            change.direction,
            change.from.as_str(),
            change.field.as_str(),
            change.value.as_str(),
        )),
        Some((
            "protects",
            quire_rs::traceability::EdgeSide::Incoming,
            "control",
            "safeguards",
            "encrypt"
        ))
    );
    assert!(flow.facts["alpha"][0].path.starts_with("spec/"));
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
    assert!(serialized.contains(
        "\"remediations\":[{\"name\":\"encrypt\",\"description\":\"Add a control protecting the alpha.\",\"change\":{"
    ));
    assert!(serialized.contains("\"name\":\"encrypt-alt\""));
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
    assert!(validation.warnings[0].message.contains("encrypt"));
    assert!(validation.warnings[0]
        .message
        .contains("\"description\":\"Add a control protecting the alpha.\""));
    assert!(validation.warnings[0]
        .message
        .contains("\"change\":{\"adds\":"));
    assert!(
        validation.warnings[0]
            .message
            .find("\"name\":\"encrypt\"")
            .unwrap()
            < validation.warnings[0]
                .message
                .find("\"name\":\"encrypt-alt\"")
                .unwrap()
    );
    assert!(validation.warnings[0]
        .message
        .contains("Protect the declared category"));
}

#[test]
#[trace("FR-096-AC-4", "FR-096-AC-9")]
fn any_path_and_absent_present_cover_empty_and_multi_hop_paths() {
    let registry = registry_with(
        "  - name: any-path-rule\n    subject: [sink]\n    match:\n    - any_path:\n      - [{edge: delivers_to, direction: incoming, to: [alpha]}, {edge: links_to, direction: outgoing, to: [beta]}]\n      - [{edge: carries, direction: outgoing, to: [gamma]}]\n      field: category\n      any_of: [health]\n    check: any-path-match\n    rationale: Find a category over either path.\n  - name: absent-rule\n    subject: [alpha]\n    match:\n    - {path: [], field: approval, absent: true}\n    check: absent-approval\n    rationale: Approval is unstated.\n  - name: absent-endpoint\n    subject: [alpha]\n    match: [{path: [{edge: links_to, direction: outgoing, to: [beta]}], field: id, absent: true}]\n    check: no-link\n    rationale: No linked beta exists.\n  - name: present-endpoint\n    subject: [alpha]\n    match: [{path: [{edge: links_to, direction: outgoing, to: [beta]}], field: id, present: true}]\n    check: has-link\n    rationale: A linked beta exists.\n",
        "[]",
        "[]",
    );
    assert!(registry.failures().is_empty(), "{:?}", registry.failures());
    let (root, spec) = docs(&[
        (
            "sink.md",
            &document(
                "S-1",
                "sink",
                "kind: prompt\nagreement: missing\n",
                &[("carries", "G-1")],
            ),
        ),
        (
            "alpha.md",
            &document(
                "A-1",
                "alpha",
                "",
                &[("delivers_to", "S-1"), ("links_to", "B-1")],
            ),
        ),
        (
            "beta.md",
            &document("B-1", "beta", "category: [health]\n", &[]),
        ),
        (
            "gamma.md",
            &document("G-1", "gamma", "category: health\n", &[]),
        ),
        ("alpha2.md", &document("A-2", "alpha", "", &[])),
    ]);
    let result = report(root.path(), &spec, &registry);
    let flow = result.flow.unwrap();
    assert_eq!(flow.denials.len(), 5);
    assert!(flow
        .denials
        .iter()
        .any(|denial| denial.rule == "any-path-rule"));
    assert!(flow
        .denials
        .iter()
        .any(|denial| denial.rule == "absent-rule"));
    assert!(flow
        .denials
        .iter()
        .any(|denial| denial.rule == "absent-endpoint" && denial.subject.id == "A-2"));
    assert!(flow
        .denials
        .iter()
        .any(|denial| denial.rule == "present-endpoint" && denial.subject.id == "A-1"));
    let multi = flow
        .denials
        .iter()
        .find(|denial| denial.rule == "any-path-rule")
        .unwrap();
    assert_eq!(multi.matched["beta"].len(), 1);
    assert_eq!(multi.matched["gamma"].len(), 1);
}

#[test]
#[trace("FR-096-AC-2")]
fn a_matching_control_clears_the_denial() {
    let registry = registry_with(
        "  - name: category-rule\n    subject: [alpha]\n    match: [{path: [{edge: links_to, direction: outgoing, to: [beta]}], field: category, any_of: [health]}]\n    unless_any: [{path: [{edge: protects, direction: incoming, to: [control]}], field: safeguards, any_of: [encrypt]}]\n    check: category-crossing\n    rationale: Protect the declared category.\n",
        "[]",
        "[]",
    );
    let (root, spec) = docs(&[
        (
            "alpha.md",
            &document("A-1", "alpha", "", &[("links_to", "B-1")]),
        ),
        (
            "beta.md",
            &document("B-1", "beta", "category: [health]\n", &[]),
        ),
        (
            "control.md",
            &document(
                "C-1",
                "control",
                "safeguards: [encrypt]\n",
                &[("protects", "A-1")],
            ),
        ),
    ]);
    assert!(report(root.path(), &spec, &registry)
        .flow
        .unwrap()
        .denials
        .is_empty());
}

#[test]
#[trace("FR-096-AC-3")]
fn sink_conditions_use_authored_kind_and_agreement_values() {
    let registry = registry_with(
        "  - name: sink-rule\n    subject: [sink]\n    match: [{path: [], field: kind, any_of: [prompt]}, {path: [], field: agreement, any_of: [missing, unknown]}]\n    check: agreement-missing\n    rationale: Agreement is not covered.\n",
        "[]",
        "[]",
    );
    let (root, spec) = docs(&[
        (
            "a.md",
            &document("S-1", "sink", "kind: prompt\nagreement: missing\n", &[]),
        ),
        (
            "b.md",
            &document("S-2", "sink", "kind: prompt\nagreement: covered\n", &[]),
        ),
        (
            "c.md",
            &document("S-3", "sink", "kind: prompt\nagreement: unknown\n", &[]),
        ),
    ]);
    let flow = report(root.path(), &spec, &registry).flow.unwrap();
    assert_eq!(flow.denials.len(), 2);
    assert_eq!(flow.denials[0].subject.id, "S-1");
    assert_eq!(flow.denials[1].subject.id, "S-3");
}

#[test]
#[trace("FR-096-AC-8", "FR-096-AC-21")]
fn missing_edge_or_authored_endpoint_value_produces_no_denial_or_inferred_fact() {
    let registry = registry_with(
        "  - name: edge-rule\n    subject: [alpha]\n    match: [{path: [{edge: links_to, direction: outgoing, to: [beta]}], field: category, any_of: [health]}]\n    check: no-edge\n    rationale: A category is required.\n",
        "[]",
        "[]",
    );
    let (root, spec) = docs(&[("a.md", &document("A-1", "alpha", "state: open\n", &[]))]);
    let flow = report(root.path(), &spec, &registry).flow.unwrap();
    assert!(flow.denials.is_empty());
    assert_eq!(flow.facts["alpha"][0].fields, BTreeMap::new());
}

#[test]
#[trace("FR-096-AC-18")]
fn a_corpus_without_flow_rules_keeps_the_relation_report_shape_unchanged() {
    let registry = registry_with("    []\n", "[]", "[]");
    let (root, spec) = docs(&[]);
    let result = report(root.path(), &spec, &registry);
    assert!(result.flow.is_none());
    assert!(!serde_json::to_string(&result).unwrap().contains("\"flow\""));
}

#[test]
#[trace(
    "FR-096-AC-5",
    "FR-096-AC-6",
    "FR-096-AC-24",
    "FR-096-AC-25",
    "FR-096-AC-28"
)]
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
            "  - name: nested-field\n    subject: [alpha]\n    match: [{path: [], field: nested_only, present: true}]\n    check: nested-field\n    rationale: Nested properties are not top-level frontmatter fields.\n",
            "nested_only",
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
        (
            "  - name: missing-remediation\n    subject: [alpha]\n    match: [{path: [], field: id, present: true}]\n    check: missing-remediation\n    remediations: [not-declared]\n    rationale: Missing remediation.\n",
            "not-declared",
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
    let duplicate = parse_registry_with(
        "  - name: duplicate-rule\n    subject: [alpha]\n    match: [{path: [], field: id, present: true}]\n    check: first-check\n    rationale: First.\n  - name: duplicate-rule\n    subject: [alpha]\n    match: [{path: [], field: id, present: true}]\n    check: second-check\n    rationale: Second.\n",
        "[]",
        "[]",
    )
    .expect("a duplicated name never fails the module");
    let (kind, item, reason) = crate::declaration_drop::sole_drop(&duplicate);
    assert_eq!((kind, item.as_str()), ("flow_rule", "duplicate-rule"));
    assert!(reason.contains("duplicate-rule"), "{reason}");
    assert!(duplicate
        .traceability()
        .expect("model")
        .flow_rules
        .is_empty());
    let valid = registry_with(
        "  - name: valid-enum\n    subject: [alpha]\n    match: [{path: [], field: state, any_of: [' OPEN ']}]\n    check: valid-enum\n    rationale: Valid folded enum.\n",
        "[]",
        "[]",
    );
    assert!(valid.failures().is_empty(), "{:?}", valid.failures());
}

#[test]
#[trace("FR-096-AC-6")]
fn blank_check_and_rationale_drop_the_rule_naming_it() {
    for (field, value) in [("check", "''"), ("rationale", "''")] {
        let rules = format!(
            "  - name: blank-{field}\n    subject: [alpha]\n    match: [{{path: [], field: id, present: true}}]\n    check: {}\n    rationale: {}\n",
            if field == "check" { value } else { "valid-check" },
            if field == "rationale" {
                value
            } else {
                "Valid rationale."
            }
        );
        let registry = parse_registry_with(&rules, "[]", "[]")
            .expect("a blank rule field never fails the module");
        let (kind, item, error) = crate::declaration_drop::sole_drop(&registry);
        assert_eq!(
            (kind, item.as_str()),
            ("flow_rule", format!("blank-{field}").as_str())
        );
        assert!(
            error.contains(&format!("blank-{field}")) && error.contains(&format!("`{field}`")),
            "expected a diagnostic naming rule and blank {field}, got {error}"
        );
    }
}

#[test]
#[trace("FR-096-AC-6", "FR-096-CON-1")]
fn duplicate_rule_names_across_modules_fail_with_both_module_names() {
    let parent = tempfile::tempdir().expect("modules");
    let mut module_roots = Vec::new();
    for name in ["flow-module-one", "flow-module-two"] {
        let root = parent.path().join(name);
        fs::create_dir_all(&root).expect("module dir");
        fs::write(
            root.join("manifest.yaml"),
            format!(
                "name: {name}\nversion: 0.1.0\ndescription: Duplicate flow rule\narchetypes:\n- name: alpha\ntraceability:\n  flow_rules:\n  - name: duplicate-rule\n    subject: [alpha]\n    match: [{{path: [], field: id, present: true}}]\n    check: duplicate-check\n    rationale: Duplicate declaration.\n"
            ),
        )
        .expect("manifest");
        module_roots.push(root);
    }
    let roots = module_roots
        .iter()
        .map(|root| root.as_path())
        .collect::<Vec<_>>();
    let registry = Registry::load_module_set(&roots).expect("modules load for diagnostics");
    assert!(registry.failures().iter().any(|failure| {
        failure.module == "flow-module-one" && failure.reason.contains("flow-module-two")
    }));
    assert!(registry.failures().iter().any(|failure| {
        failure.module == "flow-module-two" && failure.reason.contains("flow-module-one")
    }));
    assert!(registry
        .traceability()
        .is_none_or(|model| model.flow_rules.is_empty()));
}

#[test]
#[trace("FR-096-AC-25")]
fn flow_rules_can_share_a_new_severity_check() {
    let registry = registry_with(
        "  - name: first\n    subject: [alpha]\n    match: [{path: [], field: state, any_of: [open]}]\n    check: shared-check\n    rationale: First rule.\n  - name: second\n    subject: [alpha]\n    match: [{path: [], field: approval, absent: true}]\n    check: shared-check\n    rationale: Second rule.\n",
        "[]",
        "[]",
    );
    assert!(registry.failures().is_empty(), "{:?}", registry.failures());
    let (root, spec) = docs(&[("a.md", &document("A-1", "alpha", "state: open\n", &[]))]);
    let error = registry.with_grammar_severity(BTreeMap::from([(
        "trace:shared-check".into(),
        quire_rs::grammar::GrammarSeverityLevel::Error,
    )]));
    let flow = report(root.path(), &spec, &error).flow.unwrap();
    assert_eq!(flow.denials.len(), 2);
    assert!(flow.denials.iter().all(|denial| denial.severity == "error"));
    let validation = validate_bundle(
        &spec,
        &error,
        BundlePosture::Strict,
        root.path(),
        root.path(),
    );
    assert_eq!(validation.errors.len(), 2);
}

#[test]
#[trace("FR-096-AC-7", "FR-096-AC-24", "FR-096-CON-1")]
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

    for (change, expected) in [
        (
            "adds: {edge: crosses, direction: incoming, from: control, field: safeguards, value: encrypt}",
            "add-rule",
        ),
        (
            "adds: {edge: protects, direction: incoming, from: sink, field: safeguards, value: encrypt}",
            "add-rule",
        ),
        (
            "adds: {edge: protects, direction: incoming, from: control, field: safeguards, value: mask}",
            "add-rule",
        ),
        (
            "adds: {edge: protects, direction: incoming, from: control, field: safeguards, value: invalid}",
            "invalid",
        ),
    ] {
        let rejected = registry_with(
            "  - name: add-rule\n    subject: [alpha]\n    unless_any: [{path: [{edge: protects, direction: incoming, to: [control]}], field: safeguards, any_of: [encrypt]}]\n    match: [{path: [], field: state, any_of: [open]}]\n    check: add-rule\n    remediations: [add-fix]\n    rationale: Add a matching control.\n",
            &format!("  - name: add-fix\n    description: Add control.\n    {change}\n"),
            "[]",
        );
        assert!(
            rejected
                .failures()
                .iter()
                .any(|failure| failure.reason.contains(expected)),
            "expected rejection for {change:?}, got {:?}",
            rejected.failures()
        );
    }

    let unreferenced = registry_with(
        "  - name: no-remediation-rule\n    subject: [alpha]\n    match: [{path: [], field: state, any_of: [open]}]\n    check: no-remediation\n    rationale: Remediation is optional.\n",
        "  - name: unused-fix\n    description: Add a control that no rule offers.\n    adds: {edge: protects, direction: incoming, from: control, field: safeguards, value: encrypt}\n",
        "[]",
    );
    assert!(
        unreferenced.failures().is_empty(),
        "{:?}",
        unreferenced.failures()
    );

    let valid_set = registry_with(
        "  - name: set-rule\n    subject: [alpha]\n    match: [{path: [], field: state, any_of: [open]}]\n    unless_any: [{path: [], field: approval, any_of: [covered]}]\n    check: set-rule\n    remediations: [set-fix]\n    rationale: Set approval.\n",
        "  - name: set-fix\n    description: Set approval to covered.\n    sets: {field: approval, value: covered}\n",
        "[]",
    );
    assert!(
        valid_set.failures().is_empty(),
        "{:?}",
        valid_set.failures()
    );

    let invalid_set = registry_with(
        "  - name: set-rule\n    subject: [alpha]\n    match: [{path: [], field: state, any_of: [open]}]\n    unless_any: [{path: [], field: approval, any_of: [covered]}]\n    check: set-rule\n    remediations: [set-fix]\n    rationale: Set approval.\n",
        "  - name: set-fix\n    description: Set state to closed.\n    sets: {field: state, value: closed}\n",
        "[]",
    );
    assert!(invalid_set.failures().iter().any(|failure| {
        failure.reason.contains("set-rule") && failure.reason.contains("set-fix")
    }));

    let incoming_remove = registry_with(
        "  - name: incoming-remove\n    subject: [alpha]\n    match: [{path: [{edge: delivers_to, direction: incoming, to: [sink]}], field: id, present: true}]\n    check: incoming-remove\n    remediations: [remove-incoming]\n    rationale: Remove every incoming sink edge.\n",
        "  - name: remove-incoming\n    description: Edit each sink and delete its outgoing delivers_to edge to the alpha.\n    removes: {edge: delivers_to, direction: incoming}\n",
        "[]",
    );
    assert!(
        incoming_remove.failures().is_empty(),
        "{:?}",
        incoming_remove.failures()
    );

    let valid_any_path_remove = registry_with(
        "  - name: all-paths-remove\n    subject: [alpha]\n    match: [{any_path: [[{edge: links_to, direction: outgoing, to: [beta]}], [{edge: links_to, direction: outgoing, to: [beta]}, {edge: carries, direction: outgoing, to: [gamma]}]], field: id, present: true}]\n    check: all-paths-remove\n    remediations: [remove-all-links]\n    rationale: Remove all linked paths.\n",
        "  - name: remove-all-links\n    description: Remove the links_to edges.\n    removes: {edge: links_to, direction: outgoing}\n",
        "[]",
    );
    assert!(
        valid_any_path_remove.failures().is_empty(),
        "{:?}",
        valid_any_path_remove.failures()
    );

    for (paths, removed_edge) in [
        (
            "[[{edge: links_to, direction: outgoing, to: [beta]}, {edge: carries, direction: outgoing, to: [gamma]}]]",
            "carries",
        ),
        (
            "[[{edge: links_to, direction: outgoing, to: [beta]}], [{edge: crosses, direction: outgoing, to: [gamma]}]]",
            "links_to",
        ),
    ] {
        let rule = format!(
            "  - name: bad-remove\n    subject: [alpha]\n    match: [{{any_path: {paths}, field: id, present: true}}]\n    check: bad-remove\n    remediations: [remove-link]\n    rationale: Every path must start with the removed edge.\n"
        );
        let rejected = registry_with(
            &rule,
            &format!("  - name: remove-link\n    description: Remove {removed_edge}.\n    removes: {{edge: {removed_edge}, direction: outgoing}}\n"),
            "[]",
        );
        assert!(rejected.failures().iter().any(|failure| {
            failure.reason.contains("bad-remove") && failure.reason.contains("remove-link")
        }));
    }

    let invalid_remediation_field = registry_with(
        "  - name: field-rule\n    subject: [alpha]\n    match: [{path: [], field: state, any_of: [open]}]\n    check: field-rule\n    remediations: [field-fix]\n    rationale: Remediation fields are schema checked.\n",
        "  - name: field-fix\n    description: Set a misspelled field.\n    sets: {field: typo, value: x}\n",
        "[]",
    );
    assert!(invalid_remediation_field.failures().iter().any(|failure| {
        failure.reason.contains("field-fix") && failure.reason.contains("typo")
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
    assert_eq!(flow.denials[0].severity, "warning");
    let validation = validate_bundle(
        &spec,
        &registry,
        BundlePosture::Strict,
        root.path(),
        root.path(),
    );
    assert_eq!(validation.warnings.len(), 1);
    assert!(validation.is_valid());

    let error = registry.with_grammar_severity(BTreeMap::from([(
        "trace:approval-check".into(),
        quire_rs::grammar::GrammarSeverityLevel::Error,
    )]));
    assert_eq!(
        report(root.path(), &spec, &error).flow.unwrap().denials[0].severity,
        "error"
    );
    assert!(!validate_bundle(
        &spec,
        &error,
        BundlePosture::Strict,
        root.path(),
        root.path()
    )
    .is_valid());

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
        "  - name: z-rule\n    subject: [alpha]\n    match: [{path: [], field: state, any_of: [open]}]\n    check: z-check\n    rationale: Z rule.\n  - name: a-rule\n    subject: [alpha]\n    match: [{path: [], field: state, any_of: [closed]}]\n    check: a-check\n    rationale: A rule.\n  - name: beta-vocabulary\n    subject: [beta]\n    match: [{path: [], field: classification, any_of: [health]}]\n    check: beta-check\n    rationale: Read the beta vocabulary.\n",
        "[]",
        "[]",
    );
    let (root, spec) = docs(&[
        ("a.md", &document("A-1", "alpha", "state: closed\n", &[])),
        ("b.md", &document("A-2", "alpha", "state: open\n", &[])),
        (
            "c.md",
            &document("B-1", "beta", "classification: [health]\n", &[]),
        ),
    ]);
    let result = report(root.path(), &spec, &registry);
    let flow = result.flow.unwrap();
    assert_eq!(flow.rules[0].name, "z-rule");
    assert_eq!(flow.rules[1].name, "a-rule");
    assert_eq!(flow.denials[0].check, "a-check");
    assert_eq!(flow.denials[1].check, "beta-check");
    assert_eq!(flow.denials[2].check, "z-check");
    let json = serde_json::to_string(&flow).unwrap();
    assert!(json.contains("\"unless_any\":[]"));
    assert!(json.starts_with("{\"schema\":\"quire.flow-report/v1\",\"rules\":"));
    let ordered = [",\"vocabularies\":", ",\"facts\":", ",\"denials\":"]
        .map(|marker| json.find(marker).unwrap());
    let remediations = json.rfind(",\"remediations\":").unwrap();
    assert!(ordered.last().unwrap() < &remediations);
    assert!(ordered.windows(2).all(|pair| pair[0] < pair[1]));
    assert_eq!(flow.vocabularies["alpha"]["state"], ["open", "closed"]);
    assert_eq!(
        flow.vocabularies["beta"]["classification"],
        ["health", "personal", "financial"]
    );
}
