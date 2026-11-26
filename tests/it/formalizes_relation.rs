//! FR-094 — the requirement→formal-rule relation (`formalizes`).
//!
//! The fixtures are `.native` sources written here. The point is the relation's
//! separation from `verifies` and `implements`, its binding rules, and that its
//! findings reach the coverage report.

use std::fs;
use std::path::Path;

use ix_trace_rs::trace;
use quire_rs::coverage::{compute, resolve_formalizes, CoverageReport};
use quire_rs::symbol_table;
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

#[trace("FR-062-AC-28")]
#[test]
fn a_qsl_form_is_refused_outside_formalizes_and_a_foreign_form_inside_it() {
    assert_eq!(model().validate(), vec![]);

    let mut wrong_language = model();
    wrong_language.trace_tags.formalizes[0].language = SourceLanguage::Rust;
    wrong_language.trace_tags.formalizes[0].syntax = None;
    let message = wrong_language
        .validate()
        .into_iter()
        .next()
        .map(|failure| failure.reason)
        .expect("rust formalizes form");
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
            .into_iter()
            .next()
            .map(|failure| failure.reason)
            .expect("qsl form outside formalizes");
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

/// One coverage run: the report, the corpus-resolved graph and its extraction.
struct Run {
    report: CoverageReport,
    graph: SymbolGraph,
    extraction: SymbolExtraction,
}

/// Coverage over `spec/FR-001.md` and the given `src/` files. With
/// `declare_form` the module declares the `formalizes` form; without it the
/// module is the unmodified fixture.
fn run(spec_body: &str, sources: &[(&str, &str)], declare_form: bool) -> Run {
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
    if declare_form {
        model.trace_tags.formalizes.push(formalizes_form());
    }
    let extraction = extract_tree(&dir.path().join("src"));
    let mut graph = bind(&extraction, &model);
    let spec = Spec::from_path(&scope);
    resolve_formalizes(&mut graph, &spec, &registry, &scope);
    let report = compute(&spec, &registry, &graph, &scope).expect("report");
    Run {
        report,
        graph,
        extraction,
    }
}

fn report(spec_body: &str, sources: &[(&str, &str)]) -> (CoverageReport, SymbolGraph) {
    let run = run(spec_body, sources, true);
    (run.report, run.graph)
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
fn an_id_absent_from_the_corpus_is_reported_and_never_minted() {
    let run = run(
        REQUIREMENT,
        &[(
            "rules.native",
            &format!("// formalizes: FR-001, FR-999\n{DECL}"),
        )],
        true,
    );
    assert_eq!(
        reasons(&run.report),
        [("formalizes-unknown-id", "FR-999", Some(1))]
    );
    let diagnostic = run
        .report
        .diagnostics
        .iter()
        .find(|d| d.reason == "formalizes-unknown-id")
        .expect("diagnostic");
    assert_eq!(diagnostic.path.as_deref(), Some("rules.native"));
    assert!(diagnostic.message.contains("FR-999"));

    // Absent from the graph, the symbol table and search.
    assert_eq!(relations(&run.graph), [("rules.native", "P1", "FR-001")]);
    let table = symbol_table::build(&run.extraction, Some(&run.graph));
    let record = table.symbols.iter().find(|s| s.symbol == "P1").expect("P1");
    assert_eq!(record.formalizes, ["FR-001"]);
    let found = search(&run.graph, &run.extraction, &Query::Id("FR-999".into())).expect("search");
    assert!(found.formalizes.is_empty() && !found.resolved);
}

#[trace("FR-094-AC-6")]
#[test]
fn renumbering_the_requirement_orphans_the_marker() {
    let renumbered = REQUIREMENT.replace("FR-001", "FR-050");
    let run = run(
        &renumbered,
        &[("rules.native", &format!("// formalizes: FR-001\n{DECL}"))],
        true,
    );
    assert_eq!(
        reasons(&run.report),
        [("formalizes-unknown-id", "FR-001", Some(1))]
    );
    assert!(run.graph.formalizes.is_empty());
    let table = symbol_table::build(&run.extraction, Some(&run.graph));
    assert!(table.symbols.iter().all(|s| s.formalizes.is_empty()));
    let found = search(&run.graph, &run.extraction, &Query::Id("FR-001".into())).expect("search");
    assert!(found.formalizes.is_empty());

    let (healthy, graph) = report_ok();
    assert!(reasons(&healthy).is_empty());
    assert_eq!(relations(&graph), [("rules.native", "Pre1", "FR-001")]);
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

#[trace("FR-094-AC-8")]
#[test]
fn native_files_under_a_module_without_the_form_change_no_other_output() {
    let rust = ("lib.rs", "#[test]\nfn t() {}\n");
    let native = ("rules.native", &*format!("// formalizes: FR-001\n{DECL}"));
    let with = run(REQUIREMENT, &[rust, (native.0, native.1)], false);
    let without = run(REQUIREMENT, &[rust], false);

    // No relation and no formalizes-* diagnostic, though the marker is there.
    assert!(with.graph.formalizes.is_empty() && with.graph.formalizes_findings.is_empty());
    assert!(reasons(&with.report).is_empty());

    // Everything the report says is identical to the tree without the file.
    assert_eq!(
        serde_json::to_value(&with.report).expect("report"),
        serde_json::to_value(&without.report).expect("report")
    );

    // The symbol table differs only by the `.native` records, which carry no
    // relation.
    let table = symbol_table::build(&with.extraction, Some(&with.graph));
    let other = symbol_table::build(&without.extraction, Some(&without.graph));
    let kept: Vec<_> = table
        .symbols
        .iter()
        .filter(|s| !s.path.ends_with(".native"))
        .collect();
    assert_eq!(kept, other.symbols.iter().collect::<Vec<_>>());
    assert!(table
        .symbols
        .iter()
        .filter(|s| s.path.ends_with(".native"))
        .all(|s| s.formalizes.is_empty()));
}

#[trace("FR-051-AC-52")]
#[test]
fn native_and_qsl_files_read_as_qsl_declarations_when_present() {
    let case: serde_json::Value = serde_json::from_str(include_str!(
        "../fixtures/corpus_cases/qsl-source-extension.json"
    ))
    .expect("QSL source-extension case");
    assert_eq!(case["issue_ref"], "PLAT-1914");
    assert_eq!(case["language_exclusions"].as_object().unwrap().len(), 3);

    assert_eq!(
        serde_json::to_string(&SourceLanguage::Qsl).expect("serialize"),
        "\"qsl\""
    );
    assert_eq!(SourceLanguage::Qsl.as_str(), "qsl");

    let source_pairs = |key: &str| {
        case["input"][key]
            .as_object()
            .expect("source files")
            .iter()
            .map(|(path, source)| {
                (
                    path.clone(),
                    source.as_str().expect("source text").to_string(),
                )
            })
            .collect::<Vec<_>>()
    };
    let with_qsl = source_pairs("with_qsl");
    let with_qsl_refs = with_qsl
        .iter()
        .map(|(path, source)| (path.as_str(), source.as_str()))
        .collect::<Vec<_>>();
    let (_dir, extraction) = tree(&with_qsl_refs);
    let qsl: Vec<_> = extraction
        .symbols
        .iter()
        .filter(|s| s.language == SourceLanguage::Qsl)
        .collect();
    let qsl_paths = qsl
        .iter()
        .map(|symbol| symbol.path.as_str())
        .collect::<Vec<_>>();
    let expected_paths = case["expect"]["qsl_paths"]
        .as_array()
        .expect("expected QSL paths")
        .iter()
        .map(|path| path.as_str().expect("path text"))
        .collect::<Vec<_>>();
    let expected_name = case["expect"]["qualified_name"]
        .as_str()
        .expect("qualified name");
    assert_eq!(qsl_paths, expected_paths);
    assert!(qsl.iter().all(|symbol| {
        symbol.kind.as_str() == "declaration" && symbol.qualified_name == expected_name
    }));

    // Under a model declaring no form the declaration carries no relation.
    let graph = bind(&extraction, &TraceabilityModel::default());
    let table = symbol_table::build(&extraction, Some(&graph));
    let row = table.symbols.iter().find(|s| s.symbol == "P1").expect("P1");
    assert!(row.formalizes.is_empty());

    // The fixture's no-QSL control must not report the language.
    let without_qsl = source_pairs("without_qsl");
    let without_qsl_refs = without_qsl
        .iter()
        .map(|(path, source)| (path.as_str(), source.as_str()))
        .collect::<Vec<_>>();
    let (_dir, plain) = tree(&without_qsl_refs);
    let qsl_symbols: Vec<_> = plain
        .symbols
        .iter()
        .filter(|symbol| symbol.language == SourceLanguage::Qsl)
        .collect();
    assert!(qsl_symbols.is_empty());
    assert!(case["expect"]["no_qsl_language"].as_bool().unwrap());
    assert!(plain
        .files
        .iter()
        .all(|f| f.language != SourceLanguage::Qsl));
    let table = symbol_table::build(&plain, None);
    assert!(table.by_language.iter().all(|l| l.language != "qsl"));
}

#[trace("FR-051-AC-53")]
#[test]
fn unbalanced_braces_warn_per_file_and_the_rest_of_the_tree_extracts() {
    let (_dir, extraction) = tree(&[
        ("bad.native", "pre A {\n  x {\n}\n"),
        ("good.native", "pre B\n"),
        ("lib.rs", "fn f() {}\n"),
    ]);
    let warnings: Vec<_> = extraction
        .diagnostics
        .iter()
        .filter(|d| d.reason.starts_with("qsl-unbalanced-braces"))
        .collect();
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].path, "bad.native");
    assert!(warnings[0].reason.contains("bad.native:1"));
    let names: Vec<_> = extraction
        .symbols
        .iter()
        .map(|s| (s.path.as_str(), s.qualified_name.as_str()))
        .collect();
    assert_eq!(names, [("good.native", "B"), ("lib.rs", "f")]);
}

#[trace("FR-094-AC-10")]
#[test]
fn two_forms_matching_one_line_read_it_once() {
    let mut second = formalizes_form();
    second.name = "also-matches".to_string();
    let mut both = model();
    both.trace_tags.formalizes.push(second);
    let (_dir, extraction) = tree(&[(
        "rules.native",
        &format!("// formalizes: FR-001, FR-002\n{DECL}"),
    )]);
    let graph = bind(&extraction, &both);
    assert_eq!(
        relations(&graph),
        [
            ("rules.native", "P1", "FR-001"),
            ("rules.native", "P1", "FR-002")
        ]
    );
    assert_eq!(graph.formalizes[0].form, "qsl-formalizes-comment");
    assert!(graph.formalizes_findings.is_empty());
}

const NFR_SPEC: &str = "---\nid: NFR-006\ntype: NFR\ntitle: Determinism\n---\n\n\
     ## Measurement and Evaluation\n\n\
     | Metric | Target | Threshold | Method |\n\
     |--------|--------|-----------|--------|\n\
     | Identical output | all equal | all equal | Proptest |\n";

/// One run of the obligations-nfr module over an NFR whose measurement row is an obligation no trace target mints,
/// with the marker naming `id`. With `retained` the run reconciles an evidence
/// inventory that selects nothing; without it the model derives its obligations.
fn obligation_run(id: &str, retained: bool, resolve_first: bool) -> (SymbolGraph, CoverageReport) {
    use quire_rs::coverage::{compute_with_evidence, resolve_formalizes_with_evidence};
    use quire_rs::evidence::BackingInput;
    use quire_rs::obligation::inventory::{inventory, InventoryOptions};

    let dir = tempfile::tempdir().expect("tempdir");
    let scope = dir.path().join("spec");
    write(&scope, "NFR-006-determinism.md", NFR_SPEC);
    write(
        &dir.path().join("src"),
        "rules.native",
        &format!("// formalizes: {id}\n{DECL}"),
    );
    let module =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/traceability/obligations-nfr");
    let registry = Registry::load_module(&module).expect("load module");
    let mut model = registry.traceability().cloned().expect("model");
    model.trace_tags.formalizes.push(formalizes_form());
    let mut graph = bind(&extract_tree(&dir.path().join("src")), &model);
    let spec = Spec::from_path(&scope);
    let report = if retained {
        let none = inventory(dir.path(), &model, &[], InventoryOptions::default());
        let backing = BackingInput {
            inventory: &none,
            claims: &[],
            results: &[],
        };
        if resolve_first {
            resolve_formalizes_with_evidence(&mut graph, &spec, &registry, dir.path(), &backing);
        }
        compute_with_evidence(
            &spec,
            &registry,
            &graph,
            dir.path(),
            &backing,
            quire_rs::coverage::EvidenceCoverageMode::Strict,
        )
        .expect("report")
    } else {
        if resolve_first {
            resolve_formalizes(&mut graph, &spec, &registry, dir.path());
        }
        compute(&spec, &registry, &graph, dir.path()).expect("report")
    };
    (graph, report)
}

#[trace("FR-094-AC-5")]
#[test]
fn relations_and_diagnostics_agree_with_and_without_evidence() {
    for retained in [false, true] {
        for id in ["NFR-006", "NFR-006-M-1", "FR-404"] {
            let (resolved, resolved_report) = obligation_run(id, retained, true);
            let (raw, raw_report) = obligation_run(id, retained, false);
            let diagnosed = |r: &CoverageReport| reasons(r).len();
            // Resolving first or not, the same ids are diagnosed.
            assert_eq!(
                diagnosed(&resolved_report),
                diagnosed(&raw_report),
                "{id} retained={retained}"
            );
            // And a relation survives resolution exactly when it is not diagnosed.
            assert_eq!(
                resolved.formalizes.len() + resolved.formalizes_unknown.len(),
                raw.formalizes.len(),
                "{id} retained={retained}"
            );
            assert_eq!(
                resolved.formalizes.is_empty(),
                diagnosed(&resolved_report) == 1,
                "{id} retained={retained}: {:?}",
                reasons(&resolved_report)
            );
        }
    }
}

#[trace("FR-094-AC-6")]
#[test]
fn resolving_twice_changes_nothing_and_a_renumbered_id_stays_dropped() {
    let (mut graph, _) = obligation_run("FR-404", false, true);
    let before = (graph.formalizes.clone(), graph.formalizes_unknown.clone());
    let again = Registry::load_module(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/traceability/iso-obligations"),
    )
    .expect("module");
    let dir = tempfile::tempdir().expect("tempdir");
    let spec = Spec::from_path(dir.path());
    resolve_formalizes(&mut graph, &spec, &again, dir.path());
    assert_eq!((graph.formalizes, graph.formalizes_unknown), before);
    assert_eq!(before.1.len(), 1);
}

#[trace("FR-094-AC-5")]
#[test]
fn an_obligation_only_id_is_known_only_where_the_run_carries_the_obligation() {
    let (derived, _) = obligation_run("NFR-006-M-1", false, true);
    let (retained, report) = obligation_run("NFR-006-M-1", true, true);
    assert_eq!(
        derived.formalizes.len(),
        1,
        "derived run knows the criterion"
    );
    assert!(
        retained.formalizes.is_empty(),
        "an empty inventory carries no obligation"
    );
    assert_eq!(
        reasons(&report),
        [("formalizes-unknown-id", "NFR-006-M-1", Some(1))]
    );
}
