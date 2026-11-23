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
    // The same declarations in a module: it loads, and the drop is on record.
    let indented: String = yaml.lines().map(|line| format!("  {line}\n")).collect();
    let manifest = format!("name: unit\nversion: 0.1.0\ndescription: x\ntraceability:\n{indented}");
    let (_dir, root) = write_one("unit", &manifest);
    let registry = Registry::load_module(&root)
        .unwrap_or_else(|e| panic!("{yaml}: a bad declaration never fails the module: {e}"));
    assert!(
        registry
            .dropped_declarations()
            .any(|(_, d)| (d.kind.as_str(), d.item.as_str()) == (kind, item)),
        "{yaml}: {:?}",
        registry.failures()
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

/// Three modules, each declaring `entry` (a traceability line); the failures of
/// `(kind, item)` are one per declaration, each naming the two other modules.
fn three_way(entries: [&str; 3], kind: &str, items: [&str; 3]) {
    let manifests: Vec<(String, String)> = ["m-a", "m-b", "m-c"]
        .iter()
        .zip(entries)
        .map(|(name, entry)| {
            (
                (*name).to_string(),
                format!("name: {name}\nversion: 0.1.0\ndescription: x\ntraceability:\n{entry}"),
            )
        })
        .collect();
    let borrowed: Vec<(&str, &str)> = manifests
        .iter()
        .map(|(n, m)| (n.as_str(), m.as_str()))
        .collect();
    let (_dir, registry) = load(&borrowed);
    for ((module, item), name) in ["m-a", "m-b", "m-c"].iter().zip(items).zip(0..) {
        let found: Vec<_> = registry
            .failures()
            .iter()
            .filter(|f| {
                f.module == *module
                    && f.declaration
                        .as_ref()
                        .is_some_and(|d| d.kind.as_str() == kind && d.item == item)
            })
            .collect();
        assert_eq!(
            found.len(),
            1,
            "{kind} {item} in {module} ({name}): {:?}",
            registry.failures()
        );
        for other in ["m-a", "m-b", "m-c"].iter().filter(|o| *o != module) {
            assert!(found[0].reason.contains(other), "{}", found[0].reason);
        }
    }
}

#[test]
#[trace("FR-058-AC-40")]
fn three_declaring_modules_drop_every_conflicting_declaration_naming_the_others() {
    three_way(
        [
            "  carried_fields:\n  - {name: x, from: [A], key: k1}\n",
            "  carried_fields:\n  - {name: x, from: [A], key: k2}\n",
            "  carried_fields:\n  - {name: x, from: [A], key: k3}\n",
        ],
        "carried_field",
        ["x", "x", "x"],
    );
    let rule = |subject: &str| {
        format!(
            "  flow_rules:\n  - {{name: dup, subject: [{subject}], match: [{{path: [], field: id, \
             present: true}}], check: c-{subject}, rationale: r}}\n"
        )
    };
    three_way(
        [&rule("a"), &rule("b"), &rule("c")],
        "flow_rule",
        ["dup"; 3],
    );
    let remediation = |edge: &str| {
        format!(
            "  remediations:\n  - {{name: dup, description: d, removes: {{edge: {edge}, \
             direction: outgoing}}}}\n"
        )
    };
    three_way(
        [&remediation("a"), &remediation("b"), &remediation("c")],
        "remediation",
        ["dup"; 3],
    );
    // Each prefix overlaps both of the others; each of the three names the other two.
    let qualifier = |prefix: &str, repo: &str| {
        format!("  trace_id_qualifiers:\n  - {{prefix: '{prefix}', repository: {repo}}}\n")
    };
    three_way(
        [
            &qualifier("F-", "o/a"),
            &qualifier("F-X-", "o/b"),
            &qualifier("F-X-Y-", "o/c"),
        ],
        "trace_id_qualifier",
        ["F-", "F-X-", "F-X-Y-"],
    );
    // Two modules retire `x` and one declares it as minting no source symbol:
    // one failure per declaration, each naming both other modules.
    let retiring = "  vocabularies:\n    retired: [x]\n";
    let minting =
        "  vocabularies:\n    test_type_column: T\n    test_type: [X]\n    no_source_symbol: [X]\n";
    let manifests = [("m-a", retiring), ("m-b", retiring), ("m-c", minting)].map(|(name, body)| {
        (
            name,
            format!("name: {name}\nversion: 0.1.0\ndescription: x\ntraceability:\n{body}"),
        )
    });
    let borrowed: Vec<(&str, &str)> = manifests.iter().map(|(n, m)| (*n, m.as_str())).collect();
    let (_dir, registry) = load(&borrowed);
    let failures = registry.failures();
    assert_eq!(failures.len(), 3, "{failures:?}");
    for (module, kind, item) in [
        ("m-a", "retirement_vocabulary", "x"),
        ("m-b", "retirement_vocabulary", "x"),
        ("m-c", "no_source_symbol", "X"),
    ] {
        let found: Vec<_> = failures
            .iter()
            .filter(|f| {
                f.module == module
                    && f.declaration
                        .as_ref()
                        .is_some_and(|d| d.kind.as_str() == kind && d.item == item)
            })
            .collect();
        assert_eq!(found.len(), 1, "{module}: {failures:?}");
        for other in ["m-a", "m-b", "m-c"].iter().filter(|o| **o != module) {
            assert!(found[0].reason.contains(other), "{}", found[0].reason);
        }
    }
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
        let refused = Registry::load_module(&root)
            .err()
            .unwrap_or_else(|| panic!("{level} must be refused"))
            .to_string();
        assert!(
            refused.contains("trace:declared-model-dropped")
                && refused.contains("never configurable"),
            "{level}: {refused}"
        );
    }
    // Control: any other key in the same position loads.
    let control = "name: sev\nversion: 0.1.0\ndescription: x\ngrammar_severity:\n  \
                   trace:r-check: off\n";
    let (_dir, root) = write_one("sev", control);
    Registry::load_module(&root).expect("another key is accepted");

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
    // An archetype whose schema fails to load produces a failure that carries no
    // declaration kind, and whose reason (it names the schema path) reads exactly
    // like a declaration drop. It must raise no finding.
    let imitation = "merged selected traceability model: flow_rules entry 'bad' dropped";
    let manifest = format!(
        "name: plain\nversion: 0.1.0\ndescription: x\narchetypes:\n\
         - name: alpha\n  frontmatter_schema_ref: \"schemas/{imitation}.json\"\n"
    );
    let (dir, root) = write_one("plain", &manifest);
    fs::create_dir_all(root.join("schemas")).expect("schemas");
    fs::write(root.join(format!("schemas/{imitation}.json")), "not json").expect("schema");
    let registry = Registry::load_module(&root).expect("loads tolerantly");
    let failures = registry.failures();
    assert!(!failures.is_empty());
    assert!(
        failures.iter().all(|f| f.declaration.is_none()),
        "{failures:?}"
    );
    assert!(
        failures
            .iter()
            .any(|f| f.reason.contains("merged selected traceability model")),
        "the reason must imitate a drop: {failures:?}"
    );
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
#[trace("FR-058-AC-45")]
fn an_unreadable_traceability_section_drops_only_the_section() {
    let manifest = "name: broken\nversion: 0.1.0\ndescription: x\narchetypes:\n\
                    - name: alpha\n  frontmatter_schema_ref: schemas/alpha.json\n\
                    traceability:\n  required_relations: 5\n";
    let (_dir, registry) = load(&[("broken", manifest)]);
    let (kind, item, reason) = sole_drop(&registry);
    assert_eq!(
        (kind, item.as_str()),
        ("traceability_section", "traceability")
    );
    assert!(reason.contains("expected a sequence"), "{reason}");
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
    let design = registry
        .failures()
        .iter()
        .find(|f| {
            f.declaration
                .as_ref()
                .is_some_and(|d| d.kind.as_str() == "design_relation" && d.item == "gone")
        })
        .expect("the design relation drops");
    assert!(
        design
            .reason
            .contains("depends on dropped required_relations entry 'gone'"),
        "{}",
        design.reason
    );
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
fn every_consumer_of_the_model_refuses_when_declarations_were_dropped() {
    use quire_rs::corpus::{design_join, design_matrix, relation_report, RelationReportError};
    use quire_rs::obligation::inventory::{inventory, InventoryOptions, InventoryTerminal};
    use quire_rs::symbols::trace_search::{search, Query, SearchError};
    use quire_rs::traceability::{DeclarationKind, DroppedDeclarationRef, DroppedDeclarations};

    let (_dir, registry) = load(&[("decl-one", FIRST), ("decl-two", SECOND)]);
    let expected = DroppedDeclarations(vec![DroppedDeclarationRef {
        module: "decl-two".into(),
        kind: DeclarationKind::FlowRule,
        item: "bad".into(),
    }]);
    assert_eq!(registry.require_complete_model(), Err(expected.clone()));
    let text = expected.to_string();
    for named in ["decl-two", "flow_rule", "bad"] {
        assert!(text.contains(named), "{text}");
    }

    let scope = tempfile::tempdir().expect("scope");
    let source = scope.path().join("src");
    fs::create_dir_all(&source).expect("src");
    fs::create_dir_all(scope.path().join("spec")).expect("spec");
    let spec = Spec::from_path(&scope.path().join("spec"));
    let model = registry.traceability().expect("model");
    let extraction = extract_tree(&source);
    let graph = trace::bind(&extraction, model);

    // coverage
    assert_eq!(
        compute(&spec, &registry, &graph, scope.path()).expect_err("no report"),
        CoverageError::DeclarationsDropped(expected.clone())
    );
    // relation report, design join and design matrix
    let members = [quire_rs::corpus::RelationReportMember {
        root: scope.path(),
        matrix: &[],
    }];
    assert_eq!(
        relation_report(&spec, &registry, &members, &[]).expect_err("no report"),
        RelationReportError::DeclarationsDropped(expected.clone())
    );
    assert_eq!(
        design_join(&spec, &registry, &members).expect_err("no report"),
        RelationReportError::DeclarationsDropped(expected.clone())
    );
    assert_eq!(
        design_matrix(&spec, &registry, &members).expect_err("no matrix"),
        RelationReportError::DeclarationsDropped(expected.clone())
    );
    // trace search
    assert_eq!(
        search(&graph, &extraction, &Query::Id("FR-1".into())).expect_err("no result"),
        SearchError::DeclarationsDropped(expected.clone())
    );
    // obligation inventory
    let inventoried = inventory(scope.path(), model, &[], InventoryOptions::default());
    assert_eq!(inventoried.terminal(), InventoryTerminal::Incomplete);
    assert!(inventoried.causes().iter().any(|cause| {
        matches!(
            cause.detail(),
            quire_rs::obligation::inventory::InventoryCauseDetail::Model(reason) if reason == &text
        )
    }));

    // A registry with nothing dropped is not refused.
    let (_clean, registry) = load(&[("decl-one", FIRST)]);
    assert_eq!(registry.require_complete_model(), Ok(()));
    let model = registry.traceability().expect("model");
    let graph = trace::bind(&extraction, model);
    assert!(compute(&spec, &registry, &graph, scope.path()).is_ok());
}

#[test]
#[trace("FR-058-AC-39")]
fn an_archetype_with_an_unresolvable_schema_reference_drops_next_to_a_valid_one() {
    let base = "https://schemas.agent-ix.org/arch-drop/0.1.0/schemas";
    let manifest = "name: arch-drop\nversion: 0.1.0\ndescription: x\narchetypes:\n\
        - name: good\n  frontmatter_schema_ref: schemas/good.json\n\
        - name: broken\n  frontmatter_schema_ref: schemas/broken.json\n";
    let (_dir, root) = write_one("arch-drop", manifest);
    fs::create_dir_all(root.join("schemas")).expect("schemas");
    fs::write(
        root.join("schemas/good.json"),
        r#"{"type":"object","properties":{"id":{"type":"string"}}}"#,
    )
    .expect("schema");
    fs::write(
        root.join("schemas/broken.json"),
        format!(
            r#"{{"$schema":"https://json-schema.org/draft/2020-12/schema","$id":"{base}/broken.json","type":"object","properties":{{"x":{{"$ref":"{base}/Missing.json"}}}}}}"#
        ),
    )
    .expect("schema");
    let registry = Registry::load_module(&root).expect("the module loads");
    let (kind, item, reason) = sole_drop(&registry);
    assert_eq!((kind, item.as_str()), ("archetype", "broken"));
    assert!(reason.contains("Missing.json"), "{reason}");
    assert!(registry.archetype("good").is_some());
    assert!(registry.archetype("broken").is_none());
}

#[test]
#[trace("FR-058-AC-39")]
fn a_model_holding_a_retired_value_that_mints_no_source_symbol_reports_both() {
    let mut contradicting = model(
        "vocabularies:\n  test_type_column: T\n  test_type: [Inspection]\n  \
         no_source_symbol: [Inspection]\n  retired: [inspection, Withdrawn]\n",
    );
    let failures = contradicting.validate();
    let kinds: Vec<&str> = failures.iter().map(|f| f.kind.as_str()).collect();
    assert_eq!(
        kinds,
        ["retirement_vocabulary", "no_source_symbol"],
        "{failures:?}"
    );
    contradicting.prune();
    assert_eq!(contradicting.vocabularies.retired, ["Withdrawn"]);
    assert!(contradicting.vocabularies.no_source_symbol.is_empty());
}

#[test]
#[trace("FR-013-AC-19")]
fn an_unknown_key_fails_the_load_even_after_a_shape_error_in_the_section() {
    let manifest = "name: skew\nversion: 4.5.6\ndescription: x\ntraceability:\n  \
                    required_relations: 5\n  a_future_key: 1\n";
    let (_dir, root) = write_one("skew", manifest);
    let error = Registry::load_module(&root)
        .err()
        .expect("an unknown key is a manifest from a newer engine")
        .to_string();
    for named in ["a_future_key", "skew", "4.5.6"] {
        assert!(error.contains(named), "{error}");
    }
}

#[test]
#[trace("FR-058-AC-39")]
fn every_spelling_of_a_contradicting_retired_value_drops_and_is_reported() {
    let mut contradicting = model(
        "vocabularies:\n  test_type_column: T\n  test_type: [Inspection]\n  \
         no_source_symbol: [Inspection]\n  retired: [inspection, Inspection, Withdrawn]\n",
    );
    let failures = contradicting.prune();
    let items: Vec<(&str, &str)> = failures
        .iter()
        .map(|f| (f.kind.as_str(), f.item.as_str()))
        .collect();
    assert_eq!(
        items,
        [
            ("retirement_vocabulary", "inspection"),
            ("retirement_vocabulary", "Inspection"),
            ("no_source_symbol", "Inspection"),
        ]
    );
    assert_eq!(contradicting.vocabularies.retired, ["Withdrawn"]);
    assert!(contradicting.vocabularies.no_source_symbol.is_empty());
}

#[test]
#[trace("FR-013-AC-19")]
fn an_unknown_key_in_a_nested_mapping_fails_the_load_naming_it() {
    for (nested, key) in [
        ("vocabularies: {a_future_key: 1}", "a_future_key"),
        ("trace_tags: {a_future_list: []}", "a_future_list"),
        ("status: {column: S, a_future_class: [x]}", "a_future_class"),
    ] {
        let manifest =
            format!("name: skew\nversion: 4.5.6\ndescription: x\ntraceability:\n  {nested}\n");
        let (_dir, root) = write_one("skew", &manifest);
        let error = Registry::load_module(&root)
            .err()
            .unwrap_or_else(|| panic!("{nested} must fail the load"))
            .to_string();
        for named in [key, "skew", "4.5.6"] {
            assert!(error.contains(named), "{nested}: {error}");
        }
    }
}
