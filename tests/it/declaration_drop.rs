//! A declaration that cannot load drops itself, never the model (FR-058-AC-38 onward).

use std::fs;
use std::path::{Path, PathBuf};

use ix_trace_rs::trace;
use quire_rs::corpus::{validate_bundle, validate_corpus, CorpusMember, Spec};
use quire_rs::coverage::{compute, CoverageError};
use quire_rs::grammar::{merge_severity_overrides, GrammarSeverityLevel, SeverityOverrideError};
use quire_rs::symbols::{extract_tree, trace};
use quire_rs::traceability::TraceabilityModel;
use quire_rs::{BundlePosture, Registry};
use tempfile::TempDir;

const FIRST: &str = "name: decl-one
version: 0.1.0
description: Declares a relation and a carried field
archetypes:
- name: alpha
  frontmatter_schema_ref: schemas/alpha.json
edge_types:
  satisfies: {description: satisfies, category: traceability}
traceability:
  required_relations:
  - name: R
    from: alpha
    direction: outgoing
    edges: [satisfies]
    check: r-check
  carried_fields:
  - name: tier
    from: [alpha]
    key: tier
";

const SECOND: &str = "name: decl-two
version: 0.1.0
description: Declares one valid and one unloadable flow rule
archetypes:
- name: beta
  frontmatter_schema_ref: schemas/beta.json
traceability:
  flow_rules:
  - name: ok
    subject: [beta]
    match: [{path: [], field: id, present: true}]
    check: ok-check
    rationale: Valid.
  - name: bad
    subject: [nowhere]
    match: [{path: [], field: id, present: true}]
    check: bad-check
    rationale: Names an archetype nothing declares.
";

fn write_modules(manifests: &[(&str, &str)]) -> (TempDir, Vec<PathBuf>) {
    let parent = tempfile::tempdir().expect("modules");
    let mut roots = Vec::new();
    for (name, manifest) in manifests {
        let root = parent.path().join(name);
        fs::create_dir_all(&root).expect("module dir");
        fs::write(root.join("manifest.yaml"), manifest).expect("manifest");
        fs::create_dir_all(root.join("schemas")).expect("schemas dir");
        for archetype in ["alpha", "beta"] {
            fs::write(
                root.join("schemas").join(format!("{archetype}.json")),
                r#"{"type":"object","properties":{"id":{"type":"string"}}}"#,
            )
            .expect("schema");
        }
        roots.push(root);
    }
    (parent, roots)
}

/// The one declaration a registry dropped: `(kind, item, reason)`. Fails when the
/// registry holds any other load failure.
pub(crate) fn sole_drop(registry: &Registry) -> (&'static str, String, String) {
    let failures = registry.failures();
    assert_eq!(failures.len(), 1, "{failures:?}");
    let declaration = failures[0]
        .declaration
        .as_ref()
        .unwrap_or_else(|| panic!("not a declaration drop: {failures:?}"));
    (
        declaration.kind.as_str(),
        declaration.item.clone(),
        failures[0].reason.clone(),
    )
}

fn load(manifests: &[(&str, &str)]) -> (TempDir, Registry) {
    let (parent, roots) = write_modules(manifests);
    let paths: Vec<&std::path::Path> = roots.iter().map(PathBuf::as_path).collect();
    let registry = Registry::load_module_set(&paths).expect("modules load");
    (parent, registry)
}

#[test]
#[trace("FR-058-AC-38", "FR-058-AC-42", "FR-096-AC-29")]
fn an_unloadable_flow_rule_drops_only_itself() {
    let (_dir, registry) = load(&[("decl-one", FIRST), ("decl-two", SECOND)]);

    let failures = registry.failures();
    assert_eq!(failures.len(), 1, "{failures:?}");
    assert_eq!(failures[0].module, "decl-two");
    assert!(failures[0].reason.contains("bad"), "{}", failures[0].reason);

    let model = registry
        .traceability()
        .expect("the merged model survives one dropped declaration");
    assert_eq!(model.required_relations.len(), 1);
    assert_eq!(model.required_relations[0].name, "R");
    assert_eq!(model.carried_fields.len(), 1);
    assert_eq!(model.flow_rules.len(), 1);
    assert_eq!(model.flow_rules[0].name, "ok");
}

fn model(yaml: &str) -> TraceabilityModel {
    serde_yaml::from_str(yaml).expect("model parses")
}

/// Prune `yaml`: exactly one drop, of `(kind, item)`, and the model after it.
fn one_drop(yaml: &str, kind: &str, item: &str) -> TraceabilityModel {
    let mut model = model(yaml);
    let failures = model.prune();
    assert_eq!(failures.len(), 1, "{yaml}: {failures:?}");
    assert_eq!(
        (failures[0].kind.as_str(), failures[0].item.as_str()),
        (kind, item),
        "{yaml}"
    );
    model
}

const RELATION: &str = "  - name: good\n    from: FR\n    direction: outgoing\n    \
                        edges: [satisfies]\n    check: good-check\n";

#[test]
#[trace("FR-058-AC-39", "FR-058-AC-10")]
fn each_unexecutable_declaration_drops_alone_next_to_a_valid_one_of_its_kind() {
    let relations = |bad: &str| format!("required_relations:\n{RELATION}  - name: bad\n{bad}");
    let kept = one_drop(
        &relations("    from: FR\n    direction: outgoing\n    edges: []\n    check: c\n"),
        "required_relation",
        "bad",
    );
    assert_eq!(kept.required_relations.len(), 1);
    one_drop(
        &relations("    from: FR\n    direction: outgoing\n    edges: [x]\n    check: 'a:b'\n"),
        "required_relation",
        "bad",
    );
    // A relation's own `exclude` that does not compile drops that relation.
    one_drop(
        &relations(
            "    from: FR\n    direction: outgoing\n    edges: [x]\n    check: c\n    \
             exclude: ['[']\n",
        ),
        "required_relation",
        "bad",
    );
    let kept = one_drop("acyclic_edges: ['', refines]", "acyclic_edge", "#1");
    assert_eq!(kept.acyclic_edges, ["refines"]);
    let kept = one_drop("design_relations: ['', good]", "design_relation", "#1");
    assert_eq!(kept.design_relations, ["good"]);
    let kept = one_drop(
        "vocabulary_coverage:\n  - {name: good, from: A, field: f, check: g}\n  \
         - {name: bad, from: A, field: f, check: 'a:b'}\n",
        "vocabulary_coverage",
        "bad",
    );
    assert_eq!(kept.vocabulary_coverage.len(), 1);
    // A model-level glob list never partially filters, so the whole list drops.
    let kept = one_drop("exclude: ['a/**', '[']", "exclude", "exclude");
    assert!(kept.exclude.is_empty());
    let kept = one_drop(
        "source_exclude: ['a/**', '[']",
        "source_exclude",
        "source_exclude",
    );
    assert!(kept.source_exclude.is_empty());
    one_drop(
        "status: {column: Status, complete: [a], pending: [a]}",
        "status_vocabulary",
        "Status",
    );
    let kept = one_drop(
        "trace_id_qualifiers:\n  - {prefix: 'A-', repository: o/r}\n  \
         - {prefix: '', repository: o/r}\n",
        "trace_id_qualifier",
        "#2",
    );
    assert_eq!(kept.trace_id_qualifiers.len(), 1);
    let kept = one_drop(
        "vocabularies: {retired: ['', gone]}",
        "retirement_vocabulary",
        "#1",
    );
    assert_eq!(kept.vocabularies.retired, ["gone"]);
    let kept = one_drop(
        "carried_fields:\n  - {name: good, from: [A], key: k}\n  \
         - {name: Bad, from: [A], key: k}\n",
        "carried_field",
        "Bad",
    );
    assert_eq!(kept.carried_fields.len(), 1);
    let kept = one_drop(
        "trace_targets:\n  - {name: good, archetype: A, section: S, id_column: ID}\n  \
         - {name: bad, archetype: A, section: S, id_column: []}\n",
        "trace_target",
        "bad",
    );
    assert_eq!(kept.trace_targets.len(), 1);
    let kept = one_drop(
        "trace_targets:\n  - {name: t, archetype: A, section: S, id_column: ID}\n\
         document_references:\n  - {name: good, archetype: A, section: S, column: C, \
         pattern: '(x)', targets: [t]}\n  - {name: bad, archetype: A, section: S, \
         column: C, pattern: '(x)', targets: []}\n",
        "document_reference",
        "bad",
    );
    assert_eq!(kept.document_references.len(), 1);
    let kept = one_drop(
        "obligations:\n  - {name: good, archetype: A, section: S, id_format: '{row}', \
         statement_column: C}\n  - {name: bad, statement_column: C}\n",
        "obligation",
        "bad",
    );
    assert_eq!(kept.obligations.len(), 1);
    let kept = one_drop(
        "flow_rules:\n  - {name: good, subject: [A], match: [{path: [], field: id, present: true}], \
         check: g, rationale: r}\n  - {name: bad, subject: [A], match: [{path: [], field: id, \
         present: true}], check: b, rationale: ''}\n",
        "flow_rule",
        "bad",
    );
    assert_eq!(kept.flow_rules.len(), 1);
    let kept = one_drop(
        "remediations:\n  - {name: good, description: d, removes: {edge: e, direction: outgoing}}\n  \
         - {name: bad, description: '', removes: {edge: e, direction: outgoing}}\n",
        "remediation",
        "bad",
    );
    assert_eq!(kept.remediations.len(), 1);
    let kept = one_drop(
        "required_fields:\n  - {name: good, from: [A], field: f, check: g}\n  \
         - {name: bad, from: [], field: f, check: b}\n",
        "required_field",
        "bad",
    );
    assert_eq!(kept.required_fields.len(), 1);
    let kept = one_drop(
        "vocabulary:\n  - {name: good, from: [A], field: f, carried: c, check: g}\n  \
         - {name: bad, from: [A], field: '', carried: c, check: b}\n",
        "vocabulary",
        "bad",
    );
    assert_eq!(kept.vocabulary.len(), 1);
    let consistency = |name: &str, edges: &str| {
        format!(
            "  - {{name: {name}, from: [A], field: f, edges: {edges}, direction: outgoing, \
             to: [A], to_field: f, order_from: {{archetype: A, field: f}}, rule: not_below, \
             check: c-{name}}}\n"
        )
    };
    let kept = one_drop(
        &format!(
            "field_consistency:\n{}{}",
            consistency("good", "[e]"),
            consistency("bad", "[]")
        ),
        "field_consistency",
        "bad",
    );
    assert_eq!(kept.field_consistency.len(), 1);
    let kept = one_drop(
        "trace_tags:\n  markers:\n    - {name: good, language: rust, pattern: '(x)'}\n    \
         - {name: bad, language: rust, pattern: '('}\n",
        "trace_tag",
        "bad",
    );
    assert_eq!(kept.trace_tags.markers.len(), 1);
}

fn write_one(name: &str, manifest: &str) -> (TempDir, PathBuf) {
    let (dir, mut roots) = write_modules(&[(name, manifest)]);
    (dir, roots.remove(0))
}

#[test]
#[trace("FR-058-AC-40")]
fn conflicting_modules_all_drop_each_naming_every_other_and_identical_ones_merge() {
    let relation = |module: &str, check: &str| {
        format!(
            "name: {module}\nversion: 0.1.0\ndescription: x\ntraceability:\n  \
             required_relations:\n  - {{name: one, from: A, direction: outgoing, edges: [e], \
             check: {check}}}\n  - {{name: two, from: A, direction: outgoing, edges: [e], \
             check: {check}}}\n  - {{name: same, from: A, direction: outgoing, edges: [e], \
             check: same-check}}\n  carried_fields:\n  - {{name: kept, from: [A], key: k}}\n"
        )
    };
    let manifests = [
        ("m-a", relation("m-a", "check-a")),
        ("m-b", relation("m-b", "check-b")),
        ("m-c", relation("m-c", "check-c")),
    ];
    let borrowed: Vec<(&str, &str)> = manifests.iter().map(|(n, m)| (*n, m.as_str())).collect();
    let (_dir, registry) = load(&borrowed);

    let failures = registry.failures();
    assert_eq!(failures.len(), 6, "{failures:?}");
    for (module, others) in [
        ("m-a", ["m-b", "m-c"]),
        ("m-b", ["m-a", "m-c"]),
        ("m-c", ["m-a", "m-b"]),
    ] {
        for item in ["one", "two"] {
            let failure = failures
                .iter()
                .find(|f| {
                    f.module == module && f.declaration.as_ref().is_some_and(|d| d.item == item)
                })
                .unwrap_or_else(|| panic!("{module} {item}: {failures:?}"));
            for other in others {
                assert!(failure.reason.contains(other), "{}", failure.reason);
            }
        }
    }
    // The identical relation merges to one, and the other declarations load.
    let model = registry
        .traceability()
        .expect("the rest of the model loads");
    assert_eq!(
        model
            .required_relations
            .iter()
            .map(|r| r.name.as_str())
            .collect::<Vec<_>>(),
        ["same"]
    );
    assert_eq!(model.carried_fields.len(), 1);
}

#[test]
#[trace("FR-058-AC-40")]
fn conflicting_qualifiers_remediations_and_retirement_values_drop_every_declaration() {
    let a = "name: q-a\nversion: 0.1.0\ndescription: x\ntraceability:\n  \
             trace_id_qualifiers:\n  - {prefix: 'F-', repository: o/a}\n  \
             - {prefix: 'ok-', repository: o/ok}\n  vocabularies:\n    test_type_column: T\n    \
             test_type: [Inspection]\n    no_source_symbol: [Inspection]\n  \
             remediations:\n  - {name: fix, description: d, removes: {edge: e, direction: outgoing}}\n";
    let b = "name: q-b\nversion: 0.1.0\ndescription: x\ntraceability:\n  \
             trace_id_qualifiers:\n  - {prefix: 'F-X-', repository: o/b}\n  \
             - {prefix: 'ok-', repository: o/ok}\n  vocabularies:\n    retired: [inspection]\n  \
             remediations:\n  - {name: fix, description: d, removes: {edge: e, direction: outgoing}}\n";
    let (_dir, registry) = load(&[("q-a", a), ("q-b", b)]);
    let dropped: Vec<(String, &str, String)> = registry
        .failures()
        .iter()
        .filter_map(|f| {
            f.declaration
                .as_ref()
                .map(|d| (f.module.clone(), d.kind.as_str(), d.item.clone()))
        })
        .collect();
    for expected in [
        ("q-a", "trace_id_qualifier", "F-"),
        ("q-b", "trace_id_qualifier", "F-X-"),
        ("q-a", "remediation", "fix"),
        ("q-b", "remediation", "fix"),
        ("q-a", "no_source_symbol", "Inspection"),
        ("q-b", "retirement_vocabulary", "inspection"),
    ] {
        assert!(
            dropped
                .iter()
                .any(|d| (d.0.as_str(), d.1, d.2.as_str()) == expected),
            "{expected:?} in {dropped:?}"
        );
    }
    // The identical qualifier merges and loads.
    let model = registry
        .traceability()
        .expect("the rest of the model loads");
    assert_eq!(model.trace_id_qualifiers.len(), 1);
    assert_eq!(model.trace_id_qualifiers[0].prefix, "ok-");
}

fn document(id: &str, kind: &str) -> String {
    format!("---\nid: {id}\ntype: {kind}\ntitle: {id}\n---\n\n# {id}\n")
}

fn repo_with_alpha() -> TempDir {
    let dir = tempfile::tempdir().expect("repo");
    let path = dir.path().join("spec/A-1.md");
    fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    fs::write(path, document("A-1", "alpha")).expect("document");
    dir
}

fn dropped_findings(report: &quire_rs::BundleReport) -> Vec<&quire_rs::BundleFinding> {
    report
        .errors
        .iter()
        .chain(&report.warnings)
        .filter(|f| f.reason == "declared-model-dropped")
        .collect()
}

#[test]
#[trace("FR-058-AC-41")]
fn bundle_validation_reports_each_dropped_declaration_once_as_an_unconfigurable_error() {
    let (_dir, registry) = load(&[("decl-one", FIRST), ("decl-two", SECOND)]);
    let repo = repo_with_alpha();
    let spec = Spec::from_path(&repo.path().join("spec"));
    for posture in [BundlePosture::Okf, BundlePosture::Strict] {
        let report = validate_bundle(
            &spec,
            &registry,
            posture,
            &repo.path().join("spec"),
            repo.path(),
        );
        let found = dropped_findings(&report);
        assert_eq!(found.len(), 1, "{posture:?}: {:?}", report.errors);
        assert!(report.errors.iter().any(|f| std::ptr::eq(f, found[0])));
        assert!(found[0].pack.is_none());
        assert!(found[0].path.ends_with("decl-two/manifest.yaml"));
        for named in ["decl-two", "flow_rule", "bad"] {
            assert!(found[0].message.contains(named), "{}", found[0].message);
        }
        assert!(!report.is_valid());
        // Relation `R` still reports its own findings.
        assert!(
            report
                .errors
                .iter()
                .chain(&report.warnings)
                .any(|f| f.reason == "r-check"),
            "{report:?}"
        );
    }

    // No dropped declaration, no such finding.
    let (_clean, registry) = load(&[("decl-one", FIRST)]);
    let report = validate_bundle(
        &spec,
        &registry,
        BundlePosture::Strict,
        &repo.path().join("spec"),
        repo.path(),
    );
    assert!(dropped_findings(&report).is_empty());

    // A corpus of two members raises it once, not once per member.
    let (_dir, registry) = load(&[("decl-one", FIRST), ("decl-two", SECOND)]);
    let second = repo_with_alpha();
    let member = |dir: &TempDir, identity: &str| CorpusMember {
        identity: Some(identity.to_string()),
        load: quire_rs::corpus::walk::load_repo(&dir.path().join("spec")),
    };
    let corpus = Spec::from_members(vec![member(&repo, "o/a"), member(&second, "o/b")])
        .expect("corpus builds");
    let roots = [
        (repo.path().join("spec"), repo.path().to_path_buf()),
        (second.path().join("spec"), second.path().to_path_buf()),
    ];
    let pairs: Vec<(&Path, &Path)> = roots
        .iter()
        .map(|(d, r)| (d.as_path(), r.as_path()))
        .collect();
    let report =
        validate_corpus(&corpus, &registry, BundlePosture::Okf, &pairs).expect("validates");
    assert_eq!(dropped_findings(&report).len(), 1);
}

#[test]
#[trace("FR-058-AC-43")]
fn no_severity_entry_can_name_the_dropped_declaration_finding() {
    for level in ["off", "warning", "error"] {
        let entry = format!("trace:declared-model-dropped={level}");
        let refused = merge_severity_overrides(&Default::default(), [entry.as_str()]);
        assert!(
            matches!(refused, Err(SeverityOverrideError::NonRegistrable { .. })),
            "{entry}: {refused:?}"
        );
        let manifest = format!(
            "name: sev\nversion: 0.1.0\ndescription: x\ngrammar_severity:\n  \
             trace:declared-model-dropped: {level}\n"
        );
        let (_dir, root) = write_one("sev", &manifest);
        assert!(Registry::load_module(&root).is_err(), "{level}");
    }

    // Any other entry leaves the finding an error.
    let (_dir, registry) = load(&[("decl-one", FIRST), ("decl-two", SECOND)]);
    let repo = repo_with_alpha();
    let spec = Spec::from_path(&repo.path().join("spec"));
    let other = registry
        .with_grammar_severity([("trace:r-check".to_string(), GrammarSeverityLevel::Off)].into());
    for posture in [BundlePosture::Okf, BundlePosture::Strict] {
        let report = validate_bundle(
            &spec,
            &other,
            posture,
            &repo.path().join("spec"),
            repo.path(),
        );
        assert!(
            report
                .errors
                .iter()
                .any(|f| f.reason == "declared-model-dropped"),
            "{posture:?}"
        );
    }
}

#[test]
#[trace("FR-058-AC-44")]
fn a_load_failure_is_told_from_a_drop_by_its_kind_not_its_reason_text() {
    // A schema that is not JSON fails its archetype, with a reason that says
    // nothing a drop would; the failure carries no declaration kind.
    let manifest = "name: plain\nversion: 0.1.0\ndescription: x\narchetypes:\n\
                    - name: alpha\n  frontmatter_schema_ref: schemas/alpha.json\n";
    let (dir, root) = write_one("plain", manifest);
    fs::create_dir_all(root.join("schemas")).expect("schemas");
    fs::write(root.join("schemas/alpha.json"), "not json").expect("schema");
    let registry = Registry::load_module(&root).expect("loads tolerantly");
    assert!(!registry.failures().is_empty());
    assert!(registry.failures().iter().all(|f| f.declaration.is_none()));
    let repo = repo_with_alpha();
    let spec = Spec::from_path(&repo.path().join("spec"));
    let report = validate_bundle(
        &spec,
        &registry,
        BundlePosture::Strict,
        &repo.path().join("spec"),
        repo.path(),
    );
    assert!(dropped_findings(&report).is_empty());
    drop(dir);

    // A failure carrying a kind raises one whatever its reason says.
    let (_dir, registry) = load(&[("decl-one", FIRST), ("decl-two", SECOND)]);
    let kinded = registry
        .failures()
        .iter()
        .filter(|f| f.declaration.is_some())
        .count();
    let report = validate_bundle(
        &spec,
        &registry,
        BundlePosture::Okf,
        &repo.path().join("spec"),
        repo.path(),
    );
    assert_eq!(dropped_findings(&report).len(), kinded);
}

#[test]
#[trace("FR-058-AC-45")]
fn an_unreadable_traceability_section_drops_only_the_section() {
    let manifest = "name: broken\nversion: 0.1.0\ndescription: x\narchetypes:\n\
                    - name: alpha\n  frontmatter_schema_ref: schemas/alpha.json\n\
                    traceability:\n  not_a_key: 1\n";
    let (_dir, registry) = load(&[("broken", manifest)]);
    let (kind, item, reason) = sole_drop(&registry);
    assert_eq!(
        (kind, item.as_str()),
        ("traceability_section", "traceability")
    );
    assert!(reason.contains("not_a_key"), "{reason}");
    assert!(registry.traceability().is_none());
    assert!(registry.archetype("alpha").is_some(), "the archetype loads");
    let repo = repo_with_alpha();
    let spec = Spec::from_path(&repo.path().join("spec"));
    let report = validate_bundle(
        &spec,
        &registry,
        BundlePosture::Okf,
        &repo.path().join("spec"),
        repo.path(),
    );
    assert_eq!(dropped_findings(&report).len(), 1);
}

#[test]
#[trace("FR-058-AC-46")]
fn a_declaration_depending_on_a_dropped_one_drops_with_a_failure_naming_it() {
    // Within a module: an obligation naming a dropped trace target, and a
    // document reference whose `status_column` needs a dropped status.
    let mut dependent = model(
        "status: {column: Status, complete: [a], pending: [a]}\n\
         trace_targets:\n  - {name: t, archetype: A, section: S, id_column: []}\n\
         obligations:\n  - {name: o, target: t, statement_column: C}\n\
         document_references:\n  - {name: r, archetype: A, section: S, column: C, \
         pattern: '(x)', targets: [t], status_column: Status}\n",
    );
    let failures = dependent.prune();
    let by_item = |item: &str| {
        failures
            .iter()
            .find(|f| f.item == item)
            .unwrap_or_else(|| panic!("{item}: {failures:?}"))
    };
    assert!(by_item("o")
        .reason
        .contains("dropped trace_targets entry 't'"));
    assert!(
        by_item("r").reason.contains("dropped"),
        "{}",
        by_item("r").reason
    );
    assert!(dependent.obligations.is_empty() && dependent.document_references.is_empty());

    // Across modules: a rule naming a dropped remediation, and a design
    // relation naming a dropped relation.
    let manifest = "name: deps\nversion: 0.1.0\ndescription: x\narchetypes:\n\
        - name: alpha\n  frontmatter_schema_ref: schemas/alpha.json\n\
        traceability:\n  design_relations: [gone]\n  \
        required_relations:\n  - {name: gone, from: alpha, direction: outgoing, edges: [], check: g}\n  \
        flow_rules:\n  - {name: rule, subject: [alpha], match: [{path: [], field: id, present: true}], \
        check: rc, rationale: r, remediations: [fix]}\n  \
        remediations:\n  - {name: fix, description: '', removes: {edge: e, direction: outgoing}}\n";
    let (_dir, registry) = load(&[("deps", manifest)]);
    let reasons = |item: &str| {
        registry
            .failures()
            .iter()
            .find(|f| f.declaration.as_ref().is_some_and(|d| d.item == item))
            .unwrap_or_else(|| panic!("{item}: {:?}", registry.failures()))
            .reason
            .clone()
    };
    assert!(reasons("rule").contains("remediations entry 'fix'"));
    assert!(reasons("gone").contains("gone"));
    let dropped: Vec<&str> = registry
        .failures()
        .iter()
        .filter_map(|f| f.declaration.as_ref().map(|d| d.kind.as_str()))
        .collect();
    for kind in [
        "flow_rule",
        "remediation",
        "required_relation",
        "design_relation",
    ] {
        assert!(dropped.contains(&kind), "{kind}: {dropped:?}");
    }
}

#[test]
#[trace("FR-058-AC-46")]
fn a_remediation_the_subject_schema_refuses_drops_the_rule_and_stays_for_the_others() {
    let manifest = "name: sets\nversion: 0.1.0\ndescription: x\narchetypes:\n\
        - name: wide\n  frontmatter_schema_ref: schemas/wide.json\n\
        - name: narrow\n  frontmatter_schema_ref: schemas/narrow.json\n\
        traceability:\n  flow_rules:\n  \
        - {name: both, subject: [wide, narrow], match: [{path: [], field: id, present: true}], \
        unless_any: [{path: [], field: state, any_of: [y]}], check: both-check, rationale: r, \
        remediations: [fix]}\n  \
        - {name: only-wide, subject: [wide], match: [{path: [], field: id, present: true}], \
        unless_any: [{path: [], field: state, any_of: [y]}], check: wide-check, rationale: r, \
        remediations: [fix]}\n  \
        remediations:\n  - {name: fix, description: d, sets: {field: state, value: y}}\n";
    let (_dir, root) = write_one("sets", manifest);
    fs::create_dir_all(root.join("schemas")).expect("schemas");
    fs::write(
        root.join("schemas/wide.json"),
        r#"{"type":"object","properties":{"id":{"type":"string"},"state":{"enum":["x","y"]}}}"#,
    )
    .expect("schema");
    fs::write(
        root.join("schemas/narrow.json"),
        r#"{"type":"object","properties":{"id":{"type":"string"},"state":{"enum":["x"]}}}"#,
    )
    .expect("schema");
    let registry = Registry::load_module(&root).expect("loads");
    let (kind, item, reason) = sole_drop(&registry);
    assert_eq!((kind, item.as_str()), ("flow_rule", "both"));
    assert!(reason.contains("remediation 'fix'"), "{reason}");
    let model = registry
        .traceability()
        .expect("the rest of the model loads");
    assert_eq!(
        model
            .flow_rules
            .iter()
            .map(|r| r.name.as_str())
            .collect::<Vec<_>>(),
        ["only-wide"]
    );
    assert_eq!(model.remediations.len(), 1, "the remediation stays");
}

#[test]
#[trace("FR-058-AC-47")]
fn coverage_refuses_a_model_with_dropped_declarations_and_names_each() {
    let (_dir, registry) = load(&[("decl-one", FIRST), ("decl-two", SECOND)]);
    let scope = tempfile::tempdir().expect("scope");
    let source = scope.path().join("src");
    fs::create_dir_all(&source).expect("src");
    fs::create_dir_all(scope.path().join("spec")).expect("spec");
    let model = registry.traceability().expect("model");
    let graph = trace::bind(&extract_tree(&source), model);
    let error = compute(
        &Spec::from_path(&scope.path().join("spec")),
        &registry,
        &graph,
        scope.path(),
    )
    .expect_err("no report over a model with dropped declarations");
    match error {
        CoverageError::DeclarationsDropped(names) => {
            assert_eq!(names.len(), 1, "{names:?}");
            for named in ["decl-two", "flow_rule", "bad"] {
                assert!(names[0].contains(named), "{}", names[0]);
            }
        }
        other => panic!("{other:?}"),
    }
}
