//! FR-094 — the requirement→formal-rule relation (`formalizes`).
//!
//! The fixtures are `.native` sources written here. The point is the relation's
//! separation from `verifies` and `implements`, its binding rules, and that its
//! findings reach the coverage report.

use std::fs;
use std::path::Path;

use ix_trace_rs::trace;
use quire_rs::coverage::{compute, CoverageReport};
use quire_rs::symbols::trace::{bind, FormalizesFindingKind, SymbolGraph};
use quire_rs::symbols::trace_search::{language_confidence, search, Confidence, Query};
use quire_rs::symbols::{extract_tree, SymbolExtraction};
use quire_rs::traceability::{
    SourceLanguage, TraceMarkerForm, TraceMarkerSyntax, TraceabilityModel,
};
use quire_rs::{Registry, Spec};

const DECL: &str = "pre P1 using v on M::Server::request { }\n";

fn formalizes_form() -> TraceMarkerForm {
    TraceMarkerForm {
        syntax: Some(TraceMarkerSyntax::Comment),
        name: "qsl-formalizes-comment".to_string(),
        language: SourceLanguage::Qsl,
        pattern: r"^//\s*formalizes:\s*(.*)$".to_string(),
        template: Some("// formalizes: {ids}".to_string()),
        entry_paths: None,
    }
}

fn model() -> TraceabilityModel {
    let mut model = TraceabilityModel::default();
    model.trace_tags.formalizes.push(formalizes_form());
    model
}

fn write(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    fs::write(path, body).expect("write");
}

fn tree(files: &[(&str, &str)]) -> (tempfile::TempDir, SymbolExtraction) {
    let dir = tempfile::tempdir().expect("tempdir");
    for (rel, body) in files {
        write(dir.path(), rel, body);
    }
    let extraction = extract_tree(dir.path());
    (dir, extraction)
}

fn bound(files: &[(&str, &str)]) -> SymbolGraph {
    let (_dir, extraction) = tree(files);
    bind(&extraction, &model())
}

fn relations(graph: &SymbolGraph) -> Vec<(&str, &str, &str)> {
    graph
        .formalizes
        .iter()
        .map(|r| (r.path.as_str(), r.symbol.as_str(), r.trace_id.as_str()))
        .collect()
}

fn findings(graph: &SymbolGraph, kind: FormalizesFindingKind) -> Vec<(usize, &str)> {
    graph
        .formalizes_findings
        .iter()
        .filter(|f| f.kind == kind)
        .map(|f| (f.line, f.id.as_str()))
        .collect()
}

#[trace("FR-094-AC-1")]
#[test]
fn a_marker_above_a_declaration_formalizes_it() {
    let graph = bound(&[("rules.native", &format!("// formalizes: FR-001\n{DECL}"))]);
    assert_eq!(relations(&graph), [("rules.native", "P1", "FR-001")]);
    assert!(graph.formalizes_findings.is_empty());
    assert!(graph.verifies.is_empty() && graph.implements.is_empty());
}

#[trace("FR-094-AC-2")]
#[test]
fn one_marker_names_several_ids() {
    let graph = bound(&[(
        "rules.native",
        &format!("// formalizes: FR-001, FR-001-AC-2\n{DECL}"),
    )]);
    assert_eq!(
        relations(&graph),
        [
            ("rules.native", "P1", "FR-001"),
            ("rules.native", "P1", "FR-001-AC-2")
        ]
    );
}

#[trace("FR-094-AC-3")]
#[test]
fn a_requirement_claimed_by_two_files_lists_both_in_path_order() {
    let marked = format!("// formalizes: FR-001\n{DECL}");
    let graph = bound(&[("b/two.native", &marked), ("a/one.native", &marked)]);
    assert_eq!(
        relations(&graph),
        [
            ("a/one.native", "P1", "FR-001"),
            ("b/two.native", "P1", "FR-001")
        ]
    );
    assert!(graph.formalizes_findings.is_empty());
}

#[trace("FR-094-AC-4")]
#[test]
fn a_marker_binds_across_blank_and_comment_lines_only() {
    let graph = bound(&[
        (
            "bound.native",
            &format!("// formalizes: FR-001\n\n// note\n{DECL}"),
        ),
        (
            "other_line.native",
            &format!("// formalizes: FR-001\n42 not a head\n{DECL}"),
        ),
        ("eof.native", "pre A\n// formalizes: FR-001\n"),
    ]);
    assert_eq!(relations(&graph), [("bound.native", "P1", "FR-001")]);
    let unbound: Vec<_> = graph
        .formalizes_findings
        .iter()
        .filter(|f| f.kind == FormalizesFindingKind::Unbound)
        .map(|f| (f.path.as_str(), f.line))
        .collect();
    assert_eq!(unbound, [("eof.native", 2), ("other_line.native", 1)]);
}

#[trace("FR-094-AC-6")]
#[test]
fn renaming_the_label_keeps_the_link() {
    let graph = bound(&[(
        "rules.native",
        "// formalizes: FR-001\npre Pre1 using v on M::S::op { }\n",
    )]);
    assert_eq!(relations(&graph), [("rules.native", "Pre1", "FR-001")]);
    assert!(graph.formalizes_findings.is_empty());
}

#[trace("FR-094-AC-9")]
#[test]
fn only_column_zero_depth_zero_heads_are_symbols() {
    let source = "language qsl\nprofile core\nmodel M\n\
                  pre P1 {\n  pre Inner\n}\n  indented Ignored\npost P2\n";
    let (_dir, extraction) = tree(&[("rules.native", source)]);
    let names: Vec<_> = extraction
        .symbols
        .iter()
        .map(|s| (s.qualified_name.as_str(), s.kind.as_str()))
        .collect();
    assert_eq!(names, [("P1", "declaration"), ("P2", "declaration")]);
}

#[trace("FR-094-AC-10")]
#[test]
fn a_repeated_id_mints_one_relation_and_one_duplicate_finding() {
    let graph = bound(&[
        (
            "same_marker.native",
            &format!("// formalizes: FR-001, FR-001\n{DECL}"),
        ),
        (
            "two_markers.native",
            &format!("// formalizes: FR-001\n// formalizes: FR-001\n{DECL}"),
        ),
    ]);
    assert_eq!(
        relations(&graph),
        [
            ("same_marker.native", "P1", "FR-001"),
            ("two_markers.native", "P1", "FR-001")
        ]
    );
    let duplicates: Vec<_> = graph
        .formalizes_findings
        .iter()
        .filter(|f| f.kind == FormalizesFindingKind::Duplicate)
        .map(|f| (f.path.as_str(), f.line))
        .collect();
    assert_eq!(
        duplicates,
        [("same_marker.native", 1), ("two_markers.native", 2)]
    );
}

#[trace("FR-094-AC-11")]
#[test]
fn colliding_labels_warn_and_a_declaration_less_file_leaves_markers_unbound() {
    let (_dir, extraction) = tree(&[
        ("dup.native", "pre P1\npre P1\n"),
        (
            "none.native",
            "// formalizes: FR-001\n\n// formalizes: FR-002\n",
        ),
    ]);
    assert!(
        extraction
            .diagnostics
            .iter()
            .any(|d| d.path == "dup.native" && d.reason.starts_with("symbol-id-collision")),
        "{:?}",
        extraction.diagnostics
    );
    let graph = bind(&extraction, &model());
    assert!(extraction.symbols.iter().all(|s| s.path != "none.native"));
    assert_eq!(
        findings(&graph, FormalizesFindingKind::Unbound),
        [(1, ""), (3, "")]
    );
    assert!(graph.formalizes.is_empty());
}

#[trace("FR-094-AC-13")]
#[test]
fn two_runs_agree() {
    let files = [
        (
            "a.native",
            &format!("// formalizes: FR-002, FR-001\n{DECL}"),
        ),
        (
            "b.native",
            &format!("// formalizes: nope\n// formalizes: FR-001\n{DECL}"),
        ),
    ];
    let refs: Vec<(&str, &str)> = files.iter().map(|(p, b)| (*p, b.as_str())).collect();
    let (first, second) = (bound(&refs), bound(&refs));
    assert_eq!(first.formalizes, second.formalizes);
    assert_eq!(first.formalizes_findings, second.formalizes_findings);
}

#[trace("FR-094-AC-15")]
#[test]
fn malformed_entries_are_reported_and_mint_nothing() {
    let graph = bound(&[
        (
            "trailing.native",
            &format!("// formalizes: FR-001,\n{DECL}"),
        ),
        ("word.native", &format!("// formalizes: not-an-id\n{DECL}")),
        ("empty.native", &format!("// formalizes:\n{DECL}")),
        ("spaces.native", &format!("// formalizes:   \n{DECL}")),
        ("comma.native", &format!("// formalizes: ,\n{DECL}")),
    ]);
    assert_eq!(relations(&graph), [("trailing.native", "P1", "FR-001")]);
    let reported: Vec<_> = graph
        .formalizes_findings
        .iter()
        .map(|f| (f.path.as_str(), f.line, f.id.as_str(), f.kind))
        .collect();
    let not_an_id = FormalizesFindingKind::NotAnId;
    assert_eq!(
        reported,
        [
            ("comma.native", 1, "", not_an_id),
            ("empty.native", 1, "", not_an_id),
            ("spaces.native", 1, "", not_an_id),
            ("word.native", 1, "not-an-id", not_an_id),
        ]
    );
}

#[trace("FR-094-AC-16")]
#[test]
fn unbalanced_braces_yield_one_warning_and_no_symbols() {
    let (_dir, extraction) = tree(&[
        ("open.native", "// formalizes: FR-001\npre A {\n  x {\n"),
        ("stray.native", "pre A\n}\n"),
        ("ok.native", "pre Fine\n"),
    ]);
    let warnings: Vec<_> = extraction
        .diagnostics
        .iter()
        .filter(|d| d.reason.starts_with("qsl-unbalanced-braces"))
        .map(|d| d.reason.as_str())
        .collect();
    assert_eq!(warnings.len(), 2, "{warnings:?}");
    assert!(warnings.iter().any(|w| w.contains("open.native:2")));
    assert!(warnings.iter().any(|w| w.contains("stray.native:2")));
    let names: Vec<_> = extraction.symbols.iter().map(|s| s.path.as_str()).collect();
    assert_eq!(names, ["ok.native"]);
    let graph = bind(&extraction, &model());
    assert!(graph.formalizes.is_empty() && graph.formalizes_findings.is_empty());
}

#[trace("FR-094-AC-17")]
#[test]
fn an_escaped_quote_does_not_hide_the_next_declaration() {
    let (_dir, extraction) = tree(&[("strings.native", "pre A \"a\\\"{\"\npre B\n")]);
    let names: Vec<_> = extraction
        .symbols
        .iter()
        .map(|s| s.qualified_name.as_str())
        .collect();
    assert_eq!(names, ["A", "B"]);
}

#[trace("FR-094-AC-18")]
#[test]
fn an_id_in_a_plain_comment_is_a_mention_and_qsl_is_line_heuristic() {
    let graph = bound(&[(
        "rules.native",
        &format!("// see FR-007 for background\n// formalizes: FR-001\n{DECL}"),
    )]);
    let mentions: Vec<_> = graph
        .mentions
        .iter()
        .map(|m| (m.trace_id.as_str(), m.line))
        .collect();
    assert_eq!(mentions, [("FR-007", 1)]);
    assert!(graph.outside_marker.is_empty() && graph.unmatched_tags.is_empty());
    assert_eq!(language_confidence("qsl"), Confidence::LineHeuristic);
}

#[trace("FR-094-AC-8")]
#[test]
fn a_module_without_formalizes_forms_binds_nothing() {
    let (_dir, extraction) = tree(&[
        ("rules.native", &format!("// formalizes: FR-001\n{DECL}")),
        ("lib.rs", "#[test]\nfn t() {}\n"),
    ]);
    let with_files = bind(&extraction, &TraceabilityModel::default());
    assert!(with_files.formalizes.is_empty() && with_files.formalizes_findings.is_empty());

    let (_dir, rust_only) = tree(&[("lib.rs", "#[test]\nfn t() {}\n")]);
    let without = bind(&rust_only, &TraceabilityModel::default());
    assert_eq!(with_files.verifies, without.verifies);
    assert_eq!(with_files.implements, without.implements);
    assert_eq!(with_files.binding_census, without.binding_census);
    assert_eq!(with_files.unmatched_tags, without.unmatched_tags);
}

#[trace("FR-062-AC-28")]
#[test]
fn a_qsl_form_is_refused_outside_formalizes_and_a_foreign_form_inside_it() {
    assert_eq!(model().validate(), Ok(()));

    let mut wrong_language = model();
    wrong_language.trace_tags.formalizes[0].language = SourceLanguage::Rust;
    wrong_language.trace_tags.formalizes[0].syntax = None;
    let message = wrong_language.validate().expect_err("rust formalizes form");
    assert!(message.contains("qsl-formalizes-comment"), "{message}");

    for section in ["markers", "implements"] {
        let mut misplaced = TraceabilityModel::default();
        let form = formalizes_form();
        match section {
            "markers" => misplaced.trace_tags.markers.push(form),
            _ => misplaced.trace_tags.implements.push(form),
        }
        let message = misplaced
            .validate()
            .expect_err("qsl form outside formalizes");
        assert!(message.contains("qsl-formalizes-comment"), "{message}");
    }
}

// ── Search ──────────────────────────────────────────────────────────────────

#[trace("FR-077-AC-8")]
#[test]
fn search_carries_formalizes_in_its_own_field_in_both_directions() {
    let (_dir, extraction) = tree(&[("rules.native", &format!("// formalizes: FR-001\n{DECL}"))]);
    let graph = bind(&extraction, &model());

    let forward = search(&graph, &extraction, &Query::Id("FR-001".into())).expect("search");
    assert_eq!(forward.formalizes.len(), 1);
    assert_eq!(forward.formalizes[0].symbol, "P1");
    assert!(forward.verifies.is_empty() && forward.implements.is_empty());
    assert!(forward.resolved);

    let inverse = search(&graph, &extraction, &Query::File("rules.native".into())).expect("search");
    assert_eq!(inverse.formalizes.len(), 1);
    assert_eq!(inverse.formalizes[0].trace_id, "FR-001");
    assert!(inverse.verifies.is_empty() && inverse.implements.is_empty());

    let by_symbol = search(
        &graph,
        &extraction,
        &Query::Symbol {
            path: "rules.native".into(),
            qualified_name: "P1".into(),
        },
    )
    .expect("search");
    assert_eq!(by_symbol.formalizes.len(), 1);
}

// ── Coverage ────────────────────────────────────────────────────────────────

const REQUIREMENT: &str = "---\nid: FR-001\ntype: FR\ntitle: A requirement\n---\n\n\
     ## Acceptance Criteria\n\n\
     | ID | Criteria | Verification |\n|----|----------|--------------|\n\
     | FR-001-AC-1 | The system shall do it. | Test |\n";

/// Coverage over `spec/FR-001.md` and the given `src/` files.
fn report(spec_body: &str, sources: &[(&str, &str)]) -> (CoverageReport, SymbolGraph) {
    let dir = tempfile::tempdir().expect("tempdir");
    let scope = dir.path().join("spec");
    write(&scope, "FR-001.md", spec_body);
    for (rel, body) in sources {
        write(&dir.path().join("src"), rel, body);
    }
    let module = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/traceability/obligations-catalog");
    let registry = Registry::load_module(&module).expect("load module");
    let mut model = registry.traceability().cloned().expect("model");
    model.trace_tags.formalizes.push(formalizes_form());
    let graph = bind(&extract_tree(&dir.path().join("src")), &model);
    let spec = Spec::from_path(&scope);
    let report = compute(&spec, &registry, &graph, &scope).expect("report");
    (report, graph)
}

fn reasons(report: &CoverageReport) -> Vec<(&str, &str, Option<usize>)> {
    report
        .diagnostics
        .iter()
        .filter(|d| {
            matches!(
                d.reason.as_str(),
                "formalizes-unbound" | "formalizes-unknown-id" | "duplicate-formalizes-claim"
            )
        })
        .map(|d| (d.reason.as_str(), d.value.as_deref().unwrap_or(""), d.line))
        .collect()
}

#[trace("FR-094-AC-5")]
#[test]
fn an_id_absent_from_the_corpus_is_reported_with_file_and_line() {
    let (report, graph) = report(
        REQUIREMENT,
        &[(
            "rules.native",
            &format!("// formalizes: FR-001, FR-999\n{DECL}"),
        )],
    );
    assert_eq!(
        reasons(&report),
        [("formalizes-unknown-id", "FR-999", Some(1))]
    );
    let diagnostic = report
        .diagnostics
        .iter()
        .find(|d| d.reason == "formalizes-unknown-id")
        .expect("diagnostic");
    assert_eq!(diagnostic.path.as_deref(), Some("rules.native"));
    assert!(diagnostic.message.contains("FR-999"));
    let unknown: Vec<_> = graph
        .unknown_formalizes(|id| id == "FR-001")
        .map(|r| r.trace_id.as_str())
        .collect();
    assert_eq!(unknown, ["FR-999"]);
}

#[trace("FR-094-AC-6")]
#[test]
fn renumbering_the_requirement_orphans_the_marker() {
    let renumbered = REQUIREMENT.replace("FR-001", "FR-050");
    let (report, _) = report(
        &renumbered,
        &[("rules.native", &format!("// formalizes: FR-001\n{DECL}"))],
    );
    assert_eq!(
        reasons(&report),
        [("formalizes-unknown-id", "FR-001", Some(1))]
    );

    let (healthy, _) = report_ok();
    assert!(reasons(&healthy).is_empty());
}

fn report_ok() -> (CoverageReport, SymbolGraph) {
    report(
        REQUIREMENT,
        &[(
            "rules.native",
            "// formalizes: FR-001\npre Pre1 using v on M::S::op { }\n",
        )],
    )
}

#[trace("FR-094-AC-12")]
#[test]
fn a_qualified_id_resolves_through_the_corpus_and_an_unresolvable_one_is_reported() {
    let (report, _) = report(
        REQUIREMENT,
        &[(
            "rules.native",
            &format!("// formalizes: ix://acme/widgets/FR-001\n{DECL}"),
        )],
    );
    // The scope declares no identity, so `ix://` ids resolve by trailing id.
    assert!(reasons(&report).is_empty(), "{:?}", reasons(&report));

    let (report, _) = report_for_missing_qualified();
    assert_eq!(
        reasons(&report),
        [("formalizes-unknown-id", "ix://acme/widgets/FR-404", Some(1))]
    );
}

fn report_for_missing_qualified() -> (CoverageReport, SymbolGraph) {
    report(
        REQUIREMENT,
        &[(
            "rules.native",
            &format!("// formalizes: ix://acme/widgets/FR-404\n{DECL}"),
        )],
    )
}

#[trace("FR-094-AC-7")]
#[test]
fn formalizes_never_backs_a_criterion_or_moves_a_total() {
    let (with, with_graph) = report(
        REQUIREMENT,
        &[(
            "rules.native",
            &format!("// formalizes: FR-001-AC-1, FR-001\n{DECL}"),
        )],
    );
    let (without, _) = report(REQUIREMENT, &[]);

    assert_eq!(with.totals.backed, 0);
    assert_eq!(with.totals.backed, without.totals.backed);
    assert_eq!(with.totals.total, without.totals.total);
    assert_eq!(with.unbacked_rows, without.unbacked_rows);
    assert_eq!(with.untracked_symbols, without.untracked_symbols);
    assert_eq!(with.binding_census, without.binding_census);
    assert!(with.implements.is_empty());
    assert!(with_graph.backed_trace_ids().is_empty());
    assert!(with_graph.verifies.is_empty());
}
