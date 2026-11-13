//! FR-086 — the design-element join, with the FR-058 counterpart and criterion
//! readings (CR-416, CR-417) and FR-038 CR-418 it depends on.
//!
//! Every kind, verb, relation and id below is authored in the fixture module
//! (`tests/fixtures/traceability/design-join`) and is unrelated to
//! requirements or design: a `Glyph` is the obliged document, a `Lantern`
//! the thing that lights it, and `kindles` / `kindled_by` the verb pair. The
//! engine knows none of them.

use std::fs;
use std::path::{Path, PathBuf};

use ix_trace_rs::trace;
use quire_rs::corpus::{
    design_join, relation_report, validate_bundle, validate_corpus, CorpusMember, LinkedEntry,
    RelationReport, RelationReportError, RelationReportMember, RelationRow, RowState, Spec,
};
use quire_rs::grammar::{GrammarSeverityLevel, GrammarSeverityMap};
use quire_rs::traceability::TraceabilityModel;
use quire_rs::{BundlePosture, Registry};
use tempfile::TempDir;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/traceability")
        .join(name)
}

fn module(name: &str) -> Registry {
    Registry::load_module(&fixture(name)).expect("fixture module loads")
}

fn write(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    fs::write(path, body).expect("write");
}

/// A document of kind `ty` with `relationships` and one `section` table
/// minting `ids`.
fn doc(id: &str, ty: &str, rels: &[(&str, &str)], section: &str, ids: &[&str]) -> String {
    let mut text = format!("---\nid: {id}\ntype: {ty}\ntitle: {id}\n");
    if !rels.is_empty() {
        text.push_str("relationships:\n");
        for (verb, target) in rels {
            text.push_str(&format!("  - target: \"{target}\"\n    type: \"{verb}\"\n"));
        }
    }
    text.push_str(&format!("---\n\n# {id}\n"));
    if !ids.is_empty() {
        text.push_str(&format!(
            "\n## {section}\n\n| ID | Criteria |\n|----|----------|\n"
        ));
        for minted in ids {
            text.push_str(&format!("| {minted} | The thing holds. |\n"));
        }
    }
    text
}

fn glyph(id: &str, rels: &[(&str, &str)], criteria: &[&str]) -> String {
    doc(id, "Glyph", rels, "Acceptance Criteria", criteria)
}

fn lantern(id: &str, rels: &[(&str, &str)]) -> String {
    doc(id, "Lantern", rels, "Test Cases", &[])
}

fn repo(docs: &[String]) -> TempDir {
    let dir = TempDir::new().expect("tempdir");
    for text in docs {
        let id = text
            .lines()
            .find_map(|l| l.strip_prefix("id: "))
            .expect("id line");
        write(dir.path(), &format!("spec/{id}.md"), text);
    }
    dir
}

fn spec_of(dir: &TempDir) -> Spec {
    Spec::from_path(&dir.path().join("spec"))
}

fn single<'a>(dir: &'a TempDir) -> [RelationReportMember<'a>; 1] {
    [RelationReportMember {
        root: dir.path(),
        matrix: &[],
    }]
}

fn join_of(dir: &TempDir, registry: &Registry) -> RelationReport {
    design_join(&spec_of(dir), registry, &single(dir)).expect("join")
}

fn report_named(dir: &TempDir, registry: &Registry, names: &[&str]) -> RelationReport {
    let names: Vec<String> = names.iter().map(|n| (*n).to_string()).collect();
    relation_report(&spec_of(dir), registry, &single(dir), &names).expect("report")
}

fn section<'r>(report: &'r RelationReport, name: &str) -> &'r quire_rs::corpus::RelationSection {
    report
        .relations
        .iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("no section {name}"))
}

fn row<'r>(report: &'r RelationReport, name: &str, id: &str) -> &'r RelationRow {
    section(report, name)
        .rows
        .iter()
        .find(|r| r.document.id == id)
        .unwrap_or_else(|| panic!("no row {id} in {name}"))
}

/// `(id, repo, verb, realized_criteria)` of every resolved `linked` entry.
fn resolved(row: &RelationRow) -> Vec<(String, Option<String>, String, Vec<String>)> {
    row.linked
        .iter()
        .filter_map(|entry| match entry {
            LinkedEntry::Resolved {
                document,
                verb,
                realized_criteria,
            } => Some((
                document.id.clone(),
                document.repo.clone(),
                verb.clone(),
                realized_criteria.clone().expect("the join carries the key"),
            )),
            LinkedEntry::Dangling { .. } => None,
        })
        .collect()
}

fn names(report: &RelationReport) -> Vec<&str> {
    report.relations.iter().map(|s| s.name.as_str()).collect()
}

fn reasons(
    dir: &TempDir,
    registry: &Registry,
    posture: BundlePosture,
    reason: &str,
) -> Vec<String> {
    let spec = spec_of(dir);
    let report = validate_bundle(
        &spec,
        registry,
        posture,
        &dir.path().join("spec"),
        dir.path(),
    );
    report
        .errors
        .iter()
        .chain(&report.warnings)
        .filter(|f| f.reason == reason)
        .map(|f| f.message.clone())
        .collect()
}

/// A module on disk with the given manifest text.
fn temp_module(manifest: &str) -> (TempDir, Registry) {
    let dir = TempDir::new().expect("tempdir");
    write(dir.path(), "manifest.yaml", manifest);
    let registry = Registry::load_module(dir.path()).expect("module loads");
    (dir, registry)
}

/// A manifest with the pair, one relation and optionally an obligations source.
fn manifest(extra_traceability: &str, with_obligations: bool) -> String {
    let obligations = if with_obligations {
        "  trace_targets:
  - name: glyph-check
    archetype: Glyph
    section: Acceptance Criteria
    id_column: ID
  obligations:
  - name: glyph-check
    target: glyph-check
    statement_column: Criteria
"
    } else {
        ""
    };
    format!(
        "name: ad-hoc
version: 0.1.0
description: ad hoc
artifact_types:
- name: Glyph
  defaults:
    id_pattern: GLY-{{next:03d}}
- name: Lantern
  defaults:
    id_pattern: LAN-{{next:03d}}
edge_types:
  kindles: {{ description: x, category: traceability, inverse: kindled_by }}
traceability:
{obligations}  required_relations:
  - name: glyph-has-lantern
    from: Glyph
    edges: [kindles]
    to: [Lantern]
    direction: incoming
    check: glyph-dark
{extra_traceability}"
    )
}

#[trace("FR-086-AC-1")]
#[trace("FR-086-AC-12")]
#[test]
fn a_requirement_without_an_element_is_an_uncovered_row() {
    let dir = repo(&[
        glyph("GLY-001", &[], &[]),
        glyph("GLY-002", &[], &[]),
        lantern("LAN-001", &[("kindles", "GLY-002")]),
    ]);
    let report = join_of(&dir, &module("design-join"));
    let covered = row(&report, "glyph-has-lantern", "GLY-002");
    assert_eq!(covered.state, RowState::Covered);
    assert_eq!(
        resolved(covered),
        vec![("LAN-001".into(), None, "kindles".into(), vec![])]
    );
    let LinkedEntry::Resolved { document, .. } = &covered.linked[0] else {
        panic!("resolved entry")
    };
    assert_eq!(document.archetype.as_deref(), Some("Lantern"));
    let gap = row(&report, "glyph-has-lantern", "GLY-001");
    assert_eq!(gap.state, RowState::Uncovered);
    assert!(gap.linked.is_empty());
    assert_eq!(gap.check, "glyph-dark");
    assert_eq!(section(&report, "glyph-has-lantern").summary.rows, 2);
}

#[trace("FR-086-AC-2")]
#[trace("FR-058-AC-26")]
#[test]
fn a_dangling_inverse_leaves_a_gap_that_names_the_reference_as_authored() {
    let dir = repo(&[glyph("GLY-001", &[("kindled_by", "ghost")], &[])]);
    let registry = module("design-join");
    let report = join_of(&dir, &registry);
    let gap = row(&report, "glyph-has-lantern", "GLY-001");
    assert_eq!(gap.state, RowState::Uncovered);
    assert_eq!(gap.dangling.len(), 1);
    assert_eq!(gap.dangling[0].reference, "ghost");
    assert_eq!(gap.dangling[0].verb, "kindles");
    assert!(gap.linked.is_empty());

    let messages = reasons(&dir, &registry, BundlePosture::Okf, "glyph-dark");
    assert_eq!(messages.len(), 1);
    assert!(messages[0].contains("'ghost'"), "{messages:?}");

    // `to: []`: the same edge to an absent document satisfies the relation,
    // is named in `dangling` and appears in `linked` as dangling.
    let open = report_named(&dir, &registry, &["glyph-open"]);
    let covered = row(&open, "glyph-open", "GLY-001");
    assert_eq!(covered.state, RowState::Covered);
    assert_eq!(covered.dangling.len(), 1);
    assert!(matches!(covered.linked[0], LinkedEntry::Dangling { .. }));
    assert!(reasons(&dir, &registry, BundlePosture::Okf, "glyph-open-dark").is_empty());
}

#[trace("FR-086-AC-3")]
#[trace("FR-086-AC-4")]
#[test]
fn the_join_reports_only_the_named_relations_in_the_named_order() {
    let dir = repo(&[
        glyph("GLY-001", &[], &[]),
        doc("PRI-001", "Prism", &[], "Acceptance Criteria", &[]),
    ]);
    let registry = module("design-join");
    let report = join_of(&dir, &registry);
    assert_eq!(
        names(&report),
        vec!["prism-has-lantern", "glyph-has-lantern", "glyph-lit-by"]
    );
    for section in &report.relations {
        assert_eq!(
            section.summary.state.covered + section.summary.state.uncovered,
            section.summary.rows
        );
    }
    assert_eq!(section(&report, "prism-has-lantern").summary.rows, 1);
    assert_eq!(section(&report, "glyph-has-lantern").summary.rows, 1);

    // FR-084 over the same corpus reports a relation the join does not name.
    let every = report_named(&dir, &registry, &[]);
    assert!(names(&every).contains(&"glyph-open"));
    assert!(!names(&report).contains(&"glyph-open"));
}

#[trace("FR-086-AC-5")]
#[trace("FR-058-AC-23")]
#[trace("FR-058-AC-25")]
#[test]
fn either_end_may_author_the_edge_and_a_pair_counts_once() {
    let dir = repo(&[
        glyph("GLY-001", &[], &[]),
        glyph("GLY-002", &[("kindled_by", "LAN-002")], &[]),
        glyph("GLY-003", &[("kindled_by", "LAN-003")], &[]),
        glyph("GLY-004", &[], &[]),
        lantern("LAN-001", &[("kindles", "GLY-001")]),
        lantern("LAN-002", &[]),
        lantern("LAN-003", &[("kindles", "GLY-003")]),
    ]);
    let registry = module("design-join");
    let report = join_of(&dir, &registry);
    let incoming = "glyph-has-lantern";
    for (glyph, lantern) in [
        ("GLY-001", "LAN-001"),
        ("GLY-002", "LAN-002"),
        ("GLY-003", "LAN-003"),
    ] {
        let found = row(&report, incoming, glyph);
        assert_eq!(found.state, RowState::Covered, "{glyph}");
        assert_eq!(
            resolved(found),
            vec![(lantern.into(), None, "kindles".into(), vec![])],
            "{glyph}"
        );
    }
    assert_eq!(row(&report, incoming, "GLY-004").state, RowState::Uncovered);

    // The outgoing relation listing the inverse label is satisfied by either
    // spelling, and reports the same documents.
    let outgoing = "glyph-lit-by";
    let satisfied: Vec<&str> = section(&report, outgoing)
        .rows
        .iter()
        .filter(|r| r.state == RowState::Covered)
        .map(|r| r.document.id.as_str())
        .collect();
    assert_eq!(satisfied, vec!["GLY-001", "GLY-002", "GLY-003"]);
    assert_eq!(
        resolved(row(&report, outgoing, "GLY-001")),
        vec![("LAN-001".into(), None, "kindled_by".into(), vec![])]
    );

    // Listing both labels accepts the same documents with the same links.
    let both = report_named(&dir, &registry, &["glyph-both", "glyph-has-lantern"]);
    for id in ["GLY-001", "GLY-002", "GLY-003", "GLY-004"] {
        let (only, pair) = (
            row(&both, "glyph-has-lantern", id),
            row(&both, "glyph-both", id),
        );
        assert_eq!(only.state, pair.state, "{id}");
        let ids = |r: &RelationRow| -> Vec<String> {
            r.linked
                .iter()
                .map(|l| match l {
                    LinkedEntry::Resolved { document, verb, .. } => {
                        format!("{}:{verb}", document.id)
                    }
                    LinkedEntry::Dangling { reference, .. } => reference.clone(),
                })
                .collect()
        };
        assert_eq!(ids(only), ids(pair), "{id}");
    }

    // The findings agree: only the document with neither spelling is reported.
    let dark = reasons(&dir, &registry, BundlePosture::Okf, "glyph-dark");
    assert_eq!(dark.len(), 1);
    assert!(dark[0].contains("GLY-004"), "{dark:?}");
    let unlit = reasons(&dir, &registry, BundlePosture::Okf, "glyph-unlit");
    assert_eq!(unlit.len(), 1);
    assert!(unlit[0].contains("GLY-004"), "{unlit:?}");
}

#[trace("FR-058-AC-24")]
#[test]
fn a_verb_the_index_relates_to_nothing_matches_exactly_as_authored() {
    let dir = repo(&[
        glyph("GLY-001", &[("wards", "LAN-001")], &[]),
        glyph("GLY-002", &[], &[]),
        lantern("LAN-001", &[]),
        lantern("LAN-002", &[("wards", "GLY-002")]),
    ]);
    let report = report_named(&dir, &module("design-join"), &["glyph-warded"]);
    // Authored by the obliged end, `wards` is not an incoming edge: no
    // counterpart is read for a verb with none.
    assert_eq!(
        row(&report, "glyph-warded", "GLY-001").state,
        RowState::Uncovered
    );
    assert_eq!(
        row(&report, "glyph-warded", "GLY-002").state,
        RowState::Covered
    );

    // Two forward verbs declaring one inverse: the first-wins entry is the only
    // counterpart.
    let (_dir, registry) = temp_module(
        "name: pair
version: 0.1.0
description: x
artifact_types:
- name: Glyph
  defaults:
    id_pattern: GLY-{next:03d}
- name: Lantern
  defaults:
    id_pattern: LAN-{next:03d}
edge_types:
  alpha: { description: x, category: traceability, inverse: back }
  beta: { description: x, category: traceability, inverse: back }
traceability:
  required_relations:
  - name: wins
    from: Glyph
    edges: [alpha]
    to: []
    direction: incoming
    check: wins-dark
  - name: loses
    from: Glyph
    edges: [beta]
    to: []
    direction: incoming
    check: loses-dark
",
    );
    let dir = repo(&[
        glyph("GLY-001", &[("back", "LAN-001")], &[]),
        lantern("LAN-001", &[]),
    ]);
    let both = report_named(&dir, &registry, &[]);
    assert_eq!(row(&both, "wins", "GLY-001").state, RowState::Covered);
    assert_eq!(row(&both, "loses", "GLY-001").state, RowState::Uncovered);
}

fn identified(dirs: &[(&TempDir, &str)]) -> Spec {
    Spec::from_members(
        dirs.iter()
            .map(|(dir, identity)| CorpusMember {
                identity: Some((*identity).to_string()),
                load: quire_rs::corpus::walk::load_repo(&dir.path().join("spec")),
            })
            .collect(),
    )
    .expect("corpus")
}

fn members<'a>(dirs: &[&'a TempDir]) -> Vec<RelationReportMember<'a>> {
    dirs.iter()
        .map(|dir| RelationReportMember {
            root: dir.path(),
            matrix: &[],
        })
        .collect()
}

#[trace("FR-086-AC-6")]
#[trace("FR-058-AC-26")]
#[test]
fn an_element_in_another_member_joins_and_an_unloaded_one_dangles() {
    let a = repo(&[glyph("GLY-001", &[], &[])]);
    let b = repo(&[lantern("LAN-001", &[("kindles", "ix://o/a/spec/GLY-001")])]);
    let registry = module("design-join");
    let corpus = identified(&[(&a, "o/a"), (&b, "o/b")]);
    let report = design_join(&corpus, &registry, &members(&[&a, &b])).expect("join");
    let found = row(&report, "glyph-has-lantern", "GLY-001");
    assert_eq!(found.state, RowState::Covered);
    assert_eq!(
        resolved(found),
        vec![(
            "LAN-001".into(),
            Some("o/b".into()),
            "kindles".into(),
            vec![]
        )]
    );

    // The requirement authors the inverse toward a member that is not loaded.
    let lone = repo(&[glyph(
        "GLY-001",
        &[("kindled_by", "ix://o/b/spec/LAN-001")],
        &[],
    )]);
    let corpus = identified(&[(&lone, "o/a")]);
    let report = design_join(&corpus, &registry, &members(&[&lone])).expect("join");
    let gap = row(&report, "glyph-has-lantern", "GLY-001");
    assert_eq!(gap.state, RowState::Uncovered);
    assert_eq!(gap.dangling[0].reference, "ix://o/b/spec/LAN-001");

    // An id absent from a loaded member is the same gap.
    let empty = repo(&[glyph("GLY-009", &[], &[])]);
    let a = repo(&[glyph(
        "GLY-001",
        &[("kindled_by", "ix://o/b/spec/LAN-404")],
        &[],
    )]);
    let corpus = identified(&[(&a, "o/a"), (&empty, "o/b")]);
    let report = design_join(&corpus, &registry, &members(&[&a, &empty])).expect("join");
    let gap = row(&report, "glyph-has-lantern", "GLY-001");
    assert_eq!(gap.state, RowState::Uncovered);
    assert_eq!(gap.dangling[0].reference, "ix://o/b/spec/LAN-404");
}

#[trace("FR-086-AC-7")]
#[trace("FR-058-AC-26")]
#[test]
fn a_wrong_kind_source_never_joins_and_only_outgoing_lists_the_mismatch() {
    let dir = repo(&[
        glyph("GLY-001", &[], &[]),
        doc("BUL-001", "Bulb", &[("kindles", "GLY-001")], "Notes", &[]),
    ]);
    let registry = module("design-join");
    let report = join_of(&dir, &registry);
    let incoming = row(&report, "glyph-has-lantern", "GLY-001");
    assert_eq!(incoming.state, RowState::Uncovered);
    assert!(incoming.linked.is_empty());
    assert!(incoming.kind_mismatch.is_empty());
    assert!(
        reasons(&dir, &registry, BundlePosture::Okf, "target-kind-mismatch")
            .iter()
            .all(|m| !m.contains("glyph-has-lantern"))
    );

    // Under the outgoing relation, the same edge read from the obliged end is
    // a wrong-kind target.
    let outgoing = row(&report, "glyph-lit-by", "GLY-001");
    assert_eq!(outgoing.state, RowState::Uncovered);
    assert!(outgoing.linked.is_empty());
    assert_eq!(outgoing.kind_mismatch.len(), 1);
    let mismatch = &outgoing.kind_mismatch[0];
    assert_eq!(mismatch.id, "BUL-001");
    assert_eq!(mismatch.archetype.as_deref(), Some("Bulb"));
    assert_eq!(mismatch.verb, "kindled_by");
    assert_eq!(mismatch.accepted, vec!["Lantern".to_string()]);

    let findings = reasons(&dir, &registry, BundlePosture::Okf, "target-kind-mismatch");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(
        findings[0].contains("'GLY-001'")
            && findings[0].contains("kindled_by")
            && findings[0].contains("BUL-001"),
        "{findings:?}"
    );
}

#[trace("FR-086-AC-8")]
#[trace("FR-058-AC-27")]
#[trace("FR-038-AC-12")]
#[test]
fn an_edge_to_a_criterion_joins_its_owner_and_keeps_the_id() {
    let dir = repo(&[
        glyph("GLY-001", &[], &["chk-a", "chk-b"]),
        glyph("GLY-002", &[], &["chk-c"]),
        lantern("LAN-001", &[("kindles", "chk-b"), ("kindles", "chk-a")]),
        lantern("LAN-002", &[("kindles", "GLY-001")]),
        lantern("LAN-003", &[("kindles", "chk-a")]),
    ]);
    let registry = module("design-join");
    let report = join_of(&dir, &registry);
    let owner = row(&report, "glyph-has-lantern", "GLY-001");
    assert_eq!(owner.state, RowState::Covered);
    assert_eq!(
        resolved(owner),
        vec![
            (
                "LAN-001".into(),
                None,
                "kindles".into(),
                vec!["chk-a".into(), "chk-b".into()]
            ),
            ("LAN-002".into(), None, "kindles".into(), vec![]),
            (
                "LAN-003".into(),
                None,
                "kindles".into(),
                vec!["chk-a".into()]
            ),
        ]
    );
    assert!(owner.dangling.is_empty(), "a read edge is not dangling");
    assert_eq!(
        row(&report, "glyph-has-lantern", "GLY-002").state,
        RowState::Uncovered
    );

    // Serialized, the key sits on the resolved entry.
    let json = serde_json::to_value(&report).expect("json");
    let entry = &json["relations"][1]["rows"][0]["linked"][0];
    assert_eq!(
        entry["realized_criteria"],
        serde_json::json!(["chk-a", "chk-b"])
    );

    // Relation matching reads the edge as to the owner; the edge set does not.
    let spec = spec_of(&dir);
    assert!(spec
        .dangling()
        .iter()
        .any(|e| e.source == "LAN-001" && e.target == "chk-a"));
    // The authoring pattern draws no dangling-reference (FR-038 CR-418), and the
    // relation raises no finding for the covered requirement.
    for posture in [BundlePosture::Okf, BundlePosture::Strict] {
        assert!(reasons(&dir, &registry, posture, "dangling-reference").is_empty());
    }
    let dark = reasons(&dir, &registry, BundlePosture::Okf, "glyph-dark");
    assert_eq!(dark.len(), 1);
    assert!(dark[0].contains("GLY-002"), "{dark:?}");
}

#[trace("FR-038-AC-12")]
#[trace("FR-058-AC-28")]
#[trace("FR-086-AC-15")]
#[test]
fn only_ids_one_document_minted_by_an_obligation_target_are_read() {
    let registry = module("design-join");

    // Minted by two documents of one member: stays dangling, as authored.
    let dir = repo(&[
        glyph("GLY-001", &[], &["twice"]),
        glyph("GLY-002", &[], &["twice"]),
        lantern("LAN-001", &[("kindles", "twice")]),
    ]);
    let report = join_of(&dir, &registry);
    for id in ["GLY-001", "GLY-002"] {
        assert_eq!(
            row(&report, "glyph-has-lantern", id).state,
            RowState::Uncovered
        );
    }
    assert_eq!(
        reasons(&dir, &registry, BundlePosture::Okf, "dangling-reference").len(),
        1
    );

    // A document id takes precedence over a minted criterion id.
    let dir = repo(&[
        glyph("GLY-001", &[], &["shared"]),
        lantern("shared", &[]),
        lantern("LAN-001", &[("kindles", "shared")]),
    ]);
    let report = join_of(&dir, &registry);
    assert_eq!(
        row(&report, "glyph-has-lantern", "GLY-001").state,
        RowState::Uncovered
    );

    // An id minted by a target no `obligations:` source names is never read.
    let dir = repo(&[
        glyph("GLY-001", &[("kindled_by", "case-1")], &[]),
        doc("LAN-001", "Lantern", &[], "Test Cases", &["case-1"]),
    ]);
    let report = join_of(&dir, &registry);
    let gap = row(&report, "glyph-has-lantern", "GLY-001");
    assert_eq!(gap.state, RowState::Uncovered);
    assert_eq!(gap.dangling[0].reference, "case-1");
    assert_eq!(
        reasons(&dir, &registry, BundlePosture::Okf, "dangling-reference").len(),
        1
    );

    // With no `obligations:` source no id is minted for this purpose.
    let (_module_dir, bare) = temp_module(&manifest("", false));
    let dir = repo(&[
        glyph("GLY-001", &[], &["chk-a"]),
        lantern("LAN-001", &[("kindles", "chk-a")]),
    ]);
    let report = report_named(&dir, &bare, &[]);
    let gap = row(&report, "glyph-has-lantern", "GLY-001");
    assert_eq!(gap.state, RowState::Uncovered);
    // The design element's edge stays a dangling edge, as authored.
    assert!(spec_of(&dir)
        .dangling()
        .iter()
        .any(|e| e.source == "LAN-001" && e.target == "chk-a"));
    assert_eq!(
        reasons(&dir, &bare, BundlePosture::Okf, "dangling-reference").len(),
        1
    );
}

#[trace("FR-058-AC-29")]
#[test]
fn a_criterion_is_read_against_the_member_the_reference_names() {
    let registry = module("design-join");
    // `o/a` mints `chk-a`; the edge names `o/b`, which does not.
    let a = repo(&[
        glyph("GLY-001", &[], &["chk-a"]),
        lantern("LAN-001", &[("kindles", "ix://o/b/spec/chk-a")]),
    ]);
    let b = repo(&[glyph("GLY-002", &[], &["chk-b"])]);
    let corpus = identified(&[(&a, "o/a"), (&b, "o/b")]);
    let report = design_join(&corpus, &registry, &members(&[&a, &b])).expect("join");
    let gap = row(&report, "glyph-has-lantern", "GLY-001");
    assert_eq!(gap.state, RowState::Uncovered);

    // `o/b` mints `chk-b`; an edge from `o/a` naming `o/b` reads against it.
    let a = repo(&[lantern("LAN-001", &[("kindles", "ix://o/b/spec/chk-b")])]);
    let corpus = identified(&[(&a, "o/a"), (&b, "o/b")]);
    let report = design_join(&corpus, &registry, &members(&[&a, &b])).expect("join");
    let found = row(&report, "glyph-has-lantern", "GLY-002");
    assert_eq!(found.state, RowState::Covered);
    assert_eq!(
        resolved(found),
        vec![(
            "LAN-001".into(),
            Some("o/a".into()),
            "kindles".into(),
            vec!["chk-b".into()]
        )]
    );

    // The same cross-member edge draws no dangling-reference from the member
    // validation either.
    let report = validate_corpus(
        &corpus,
        &registry,
        BundlePosture::Okf,
        &[
            (&a.path().join("spec"), a.path()),
            (&b.path().join("spec"), b.path()),
        ],
    )
    .expect("validate");
    assert!(report
        .warnings
        .iter()
        .chain(&report.errors)
        .all(|f| f.reason != "dangling-reference"));
}

#[trace("FR-086-AC-9")]
#[test]
fn no_design_relations_yield_an_empty_join() {
    let dir = repo(&[glyph("GLY-001", &[], &[])]);
    let report = join_of(&dir, &module("relation-report"));
    assert_eq!(
        serde_json::to_string(&report).expect("json"),
        r#"{"relations":[]}"#
    );
    // A corpus with no traceability model at all is the same.
    let (_module_dir, bare) = temp_module(
        "name: bare
version: 0.1.0
description: x
artifact_types:
- name: Glyph
  defaults:
    id_pattern: GLY-{next:03d}
",
    );
    let report = join_of(&dir, &bare);
    assert!(report.relations.is_empty());
}

#[trace("FR-086-AC-10")]
#[test]
fn a_bad_design_relation_entry_fails_load_or_reports_and_fails_the_join() {
    // Blank and repeated entries fail the module load, naming the entry.
    let blank = TraceabilityModel {
        design_relations: vec!["  ".into()],
        ..TraceabilityModel::default()
    };
    assert!(blank
        .validate()
        .expect_err("blank")
        .contains("design_relations"));
    let repeated = TraceabilityModel {
        design_relations: vec!["same".into(), "same".into()],
        ..TraceabilityModel::default()
    };
    let err = repeated.validate().expect_err("repeated");
    assert!(
        err.contains("design_relations") && err.contains("'same'"),
        "{err}"
    );
    let dir = TempDir::new().expect("tempdir");
    write(
        dir.path(),
        "manifest.yaml",
        &manifest(
            "  design_relations: [glyph-has-lantern, glyph-has-lantern]\n",
            false,
        ),
    );
    assert!(Registry::load_module_strict(dir.path()).is_err());

    // An entry naming no declared relation reports and fails the join.
    let (_module_dir, registry) = temp_module(&manifest("  design_relations: [ghost]\n", false));
    let docs = repo(&[glyph("GLY-001", &[], &[])]);
    let findings = reasons(
        &docs,
        &registry,
        BundlePosture::Okf,
        "undeclared-relation-vocabulary",
    );
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].contains("'ghost'"), "{findings:?}");
    let err = design_join(&spec_of(&docs), &registry, &single(&docs)).expect_err("no report");
    assert_eq!(
        err,
        RelationReportError::UnknownDesignRelation {
            name: "ghost".into()
        }
    );
    assert!(err.to_string().contains("ghost"));
}

#[trace("FR-086-AC-11")]
#[trace("FR-058-AC-18")]
#[test]
fn a_design_relation_reports_its_advisory_finding_and_off_drops_only_that() {
    let dir = repo(&[glyph("GLY-001", &[], &[])]);
    let registry = module("design-join");
    for posture in [BundlePosture::Okf, BundlePosture::Strict] {
        let spec = spec_of(&dir);
        let report = validate_bundle(
            &spec,
            &registry,
            posture,
            &dir.path().join("spec"),
            dir.path(),
        );
        assert!(
            report.warnings.iter().any(|f| f.reason == "glyph-dark"),
            "{posture:?}"
        );
        assert!(report.errors.iter().all(|f| f.reason != "glyph-dark"));
    }
    let before = join_of(&dir, &registry);
    let mut map = GrammarSeverityMap::new();
    map.insert("trace:glyph-dark".into(), GrammarSeverityLevel::Off);
    let silenced = registry.clone().with_grammar_severity(map);
    assert!(reasons(&dir, &silenced, BundlePosture::Okf, "glyph-dark").is_empty());
    assert!(!reasons(&dir, &silenced, BundlePosture::Okf, "glyph-open-dark").is_empty());
    assert_eq!(join_of(&dir, &silenced), before);
}

#[trace("FR-086-AC-14")]
#[trace("FR-086-CON-2")]
#[test]
fn the_join_is_byte_identical_across_runs_and_changes_no_document() {
    let docs = [
        glyph("GLY-002", &[("kindled_by", "LAN-002")], &["chk-z", "chk-y"]),
        glyph("GLY-001", &[], &["chk-a"]),
        lantern("LAN-001", &[("kindles", "chk-a"), ("kindles", "GLY-002")]),
        lantern("LAN-002", &[]),
    ];
    let forward = repo(&docs);
    let reversed = repo(&docs.iter().rev().cloned().collect::<Vec<_>>());
    let registry = module("design-join");
    let render = |dir: &TempDir| serde_json::to_string(&join_of(dir, &registry)).expect("json");
    let snapshot = |dir: &TempDir| {
        let mut out = Vec::new();
        for entry in fs::read_dir(dir.path().join("spec")).expect("dir") {
            let path = entry.expect("entry").path();
            out.push((path.clone(), fs::read(path).expect("read")));
        }
        out.sort();
        out
    };
    let before = snapshot(&forward);
    let first = render(&forward);
    assert_eq!(first, render(&forward));
    assert_eq!(first, render(&reversed));
    assert_eq!(snapshot(&forward), before);

    // No row, field or state beyond FR-084's, except `realized_criteria`.
    let json: serde_json::Value = serde_json::from_str(&first).expect("json");
    let linked = &json["relations"][1]["rows"][0]["linked"][0];
    let mut keys: Vec<&str> = linked
        .as_object()
        .expect("object")
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        vec![
            "archetype",
            "criteria",
            "id",
            "path",
            "realized_criteria",
            "resolution",
            "verb",
            "verification"
        ]
    );
    // The FR-084 report never carries the key.
    let plain = serde_json::to_string(&report_named(&forward, &registry, &[])).expect("json");
    assert!(!plain.contains("realized_criteria"));
}

#[trace("FR-086-CON-1")]
#[test]
fn design_relations_survive_is_empty_and_merge_in_module_order_without_duplicates() {
    let only = TraceabilityModel {
        design_relations: vec!["r".into()],
        ..TraceabilityModel::default()
    };
    assert!(!only.is_empty(), "a model declaring only design relations");

    let first = fixture("design-join");
    let more = fixture("design-join-more");
    let registry = Registry::load_module_set(&[first.as_path(), more.as_path()]).expect("load");
    let model = registry.traceability().expect("model");
    assert_eq!(
        model.design_relations,
        vec![
            "prism-has-lantern",
            "glyph-has-lantern",
            "glyph-lit-by",
            "glyph-extra"
        ]
    );
}
