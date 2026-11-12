//! FR-050-AC-63 — a `table_row` trace target's `id_column` is one column name
//! or a list, with `*` the only metacharacter (TC-2095..TC-2100).

use std::fs;
use std::path::Path;

use ix_trace_rs::trace;
use quire_rs::coverage::{compute, CoverageReport};
use quire_rs::symbols::{extract_tree, trace};
use quire_rs::{Registry, Spec};
use serde_json::Value;

fn write(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    fs::write(path, body).expect("write");
}

/// Materialise `module`, `documents` and `sources`, and compute coverage.
fn run(module: &str, documents: &[(&str, &str)], sources: &[(&str, &str)]) -> CoverageReport {
    let root = tempfile::tempdir().expect("scope");
    write(&root.path().join("module"), "manifest.yaml", module);
    let registry = Registry::load_module(&root.path().join("module")).expect("module loads");
    let scope = root.path().join("spec");
    fs::create_dir_all(&scope).expect("scope dir");
    for (path, body) in documents {
        write(&scope, path, body);
    }
    let source = root.path().join("src");
    fs::create_dir_all(&source).expect("src dir");
    for (path, body) in sources {
        write(&source, path, body);
    }
    let model = registry.traceability().expect("model declared");
    let graph = trace::bind(&extract_tree(&source), model);
    compute(&Spec::from_path(&scope), &registry, &graph, &scope).expect("coverage")
}

fn load_error(module: &str) -> String {
    let root = tempfile::tempdir().expect("module");
    write(root.path(), "manifest.yaml", module);
    Registry::load_module(root.path())
        .expect_err("the module fails to load")
        .to_string()
}

fn manifest(id_column: &str) -> String {
    format!(
        "name: id-column-list\nversion: 0.1.0\nartifact_types:\n- name: TestMatrix\n\
         traceability:\n  trace_targets:\n  - name: test-case\n    archetype: TestMatrix\n    \
         section: Test Cases\n    id_column: {id_column}\n"
    )
}

fn matrix(table: &str) -> String {
    format!("---\nid: TM-001\ntype: TestMatrix\n---\n\n## Test Cases\n\n{table}")
}

fn minted_ids(report: &CoverageReport) -> Vec<&str> {
    report
        .minted_targets
        .iter()
        .map(|record| record.id.as_str())
        .collect()
}

fn mint(id_column: &str, table: &str) -> Vec<String> {
    let report = run(&manifest(id_column), &[("tests.md", &matrix(table))], &[]);
    minted_ids(&report).into_iter().map(String::from).collect()
}

fn language_cases(tc: &str) -> Vec<Value> {
    let cases: Value = serde_json::from_str(include_str!(
        "../fixtures/corpus_cases/id-column-lists.json"
    ))
    .expect("id-column-lists fixture");
    cases
        .as_array()
        .expect("cases")
        .iter()
        .filter(|case| case["tags"][0] == tc)
        .cloned()
        .collect()
}

fn check_language_cases(tc: &str, language: &str) {
    let cases = language_cases(tc);
    assert_eq!(cases.len(), 3, "list, scalar control, one-column control");
    for case in cases {
        let name = case["name"].as_str().expect("name");
        assert!(name.ends_with(language), "{name}");
        let documents: Vec<(&str, &str)> = case["input"]["documents"]
            .as_object()
            .expect("documents")
            .iter()
            .map(|(path, body)| (path.as_str(), body.as_str().expect("text")))
            .collect();
        let sources: Vec<(&str, &str)> = case["input"]["sources"]
            .as_object()
            .expect("sources")
            .iter()
            .map(|(path, body)| (path.as_str(), body.as_str().expect("text")))
            .collect();
        let report = run(
            case["input"]["module"].as_str().expect("module"),
            &documents,
            &sources,
        );
        let minted: Vec<Value> = report
            .minted_targets
            .iter()
            .map(|r| serde_json::json!([r.target, r.id, r.document, r.line, r.backed]))
            .collect();
        assert_eq!(Value::Array(minted), case["expect"]["minted"], "{name}");
    }
}

#[trace("FR-050-AC-63")]
#[test]
fn python_sequence_wildcard_and_exact_columns_mint_with_scalar_control() {
    check_language_cases("TC-2095", "python");
}

#[trace("FR-050-AC-63")]
#[test]
fn typescript_sequence_wildcard_and_exact_columns_mint_with_scalar_control() {
    check_language_cases("TC-2096", "typescript");
}

#[trace("FR-050-AC-63")]
#[test]
fn rust_sequence_wildcard_and_exact_columns_mint_with_scalar_control() {
    check_language_cases("TC-2097", "rust");
}

const WIDE: &str = "| ID | Test ID | Alt ID |\n|----|---------|--------|\n\
                    | X-1 | T-1 | T-1 |\n| X-2 | T-2 | T-9 |\n";

#[trace("FR-050-AC-63")]
#[test]
fn one_declared_column_reads_no_other_column() {
    assert_eq!(mint("ID", WIDE), ["X-1", "X-2"]);
    assert_eq!(mint("Test ID", WIDE), ["T-1", "T-2"]);
}

#[trace("FR-050-AC-63")]
#[test]
fn a_header_matching_two_entries_is_read_once() {
    assert_eq!(mint("[Test ID, 'test*']", WIDE), ["T-1", "T-2"]);
}

#[trace("FR-050-AC-63")]
#[test]
fn a_row_holding_one_id_in_two_matched_columns_mints_it_once() {
    // Row X-1 repeats T-1 across both columns; row X-2 holds two distinct ids.
    assert_eq!(
        mint("[Test ID, Alt ID]", WIDE),
        ["T-1", "T-2", "T-9"],
        "the repeated id mints once, distinct ids each mint, in column order"
    );
}

#[trace("FR-050-AC-63")]
#[test]
fn entries_match_ignoring_ascii_case_and_headers_are_not_normalized() {
    let table = "| 2. Test ID | TEST NOTE |\n|----|----|\n| T-1 | N-1 |\n";
    assert_eq!(mint("'*test id'", table), ["T-1"], "wildcard, any case");
    assert_eq!(mint("'test note'", table), ["N-1"], "exact, any case");
    let report = run(&manifest("Test ID"), &[("tests.md", &matrix(table))], &[]);
    assert!(
        minted_ids(&report).is_empty(),
        "a numbered prefix is part of the header"
    );
}

#[trace("FR-050-AC-63")]
#[test]
fn id_column_matches_nothing_fires_only_when_no_entry_matches_and_names_every_entry() {
    let table = "| ID | Name |\n|----|------|\n| X-1 | a |\n";
    let reasons = |id_column: &str| -> Vec<String> {
        run(&manifest(id_column), &[("tests.md", &matrix(table))], &[])
            .diagnostics
            .into_iter()
            .filter(|d| d.reason == "id-column-matches-nothing")
            .map(|d| d.message)
            .collect()
    };
    assert!(reasons("[Missing, ID]").is_empty(), "one entry matches");
    assert!(
        reasons("[Missing, 'nam*']").is_empty(),
        "a wildcard matches"
    );
    let none = reasons("[Missing, 'Gone*']");
    assert_eq!(none.len(), 1, "{none:?}");
    assert!(
        none[0].contains("'Missing'") && none[0].contains("'Gone*'"),
        "{}",
        none[0]
    );
}

#[trace("FR-050-AC-63")]
#[test]
fn an_empty_list_or_blank_entry_fails_load_naming_the_declaration() {
    for id_column in ["[]", "[ID, '']", "'  '"] {
        let error = load_error(&manifest(id_column));
        assert!(error.contains("test-case"), "{id_column}: {error}");
        assert!(error.contains("id_column"), "{id_column}: {error}");
    }
}

#[trace("FR-050-AC-63")]
#[test]
fn a_one_name_declaration_round_trips_as_the_scalar() {
    let root = tempfile::tempdir().expect("module");
    write(root.path(), "manifest.yaml", &manifest("Test ID"));
    let registry = Registry::load_module(root.path()).expect("loads");
    let target = registry
        .traceability()
        .and_then(|model| model.target("test-case"))
        .expect("target");
    let wire = serde_json::to_value(target).expect("serialize");
    assert_eq!(wire["id_column"], "Test ID");

    let list = tempfile::tempdir().expect("module");
    write(list.path(), "manifest.yaml", &manifest("[Test ID, 'TC*']"));
    let registry = Registry::load_module(list.path()).expect("loads");
    let target = registry
        .traceability()
        .and_then(|model| model.target("test-case"))
        .expect("target");
    let wire = serde_json::to_value(target).expect("serialize");
    assert_eq!(wire["id_column"], serde_json::json!(["Test ID", "TC*"]));
}

fn row_binding_module() -> String {
    let cases: Value = serde_json::from_str(include_str!(
        "../fixtures/corpus_cases/row-aware-trace-binding.json"
    ))
    .expect("row-aware fixture");
    cases[0]["input"]["module"]
        .as_str()
        .expect("module")
        .to_string()
}

#[trace("FR-050-AC-63")]
#[test]
fn a_list_or_wildcard_target_bound_by_an_obligations_source_fails_load() {
    let module = row_binding_module();
    let anchor = "    section: Acceptance Criteria\n    id_column: ID\n";
    assert!(module.contains(anchor));
    for declared in ["[ID, Other]", "'I*'"] {
        let widened = module.replacen(
            anchor,
            &format!("    section: Acceptance Criteria\n    id_column: {declared}\n"),
            1,
        );
        let error = load_error(&widened);
        assert!(
            error.contains("acceptance-criteria") && error.contains("acceptance-criterion"),
            "{declared}: {error}"
        );
    }
}

#[trace("FR-050-AC-63")]
#[test]
fn row_binding_never_joins_a_list_target_while_a_single_name_target_joins() {
    let cases: Value = serde_json::from_str(include_str!(
        "../fixtures/corpus_cases/row-aware-trace-binding.json"
    ))
    .expect("row-aware fixture");
    let case = &cases[0];
    let module = case["input"]["module"].as_str().expect("module");
    let anchor = "    section: Test Cases\n    id_column: ID\n";
    let documents: Vec<(&str, &str)> = case["input"]["documents"]
        .as_object()
        .expect("documents")
        .iter()
        .map(|(p, b)| (p.as_str(), b.as_str().expect("text")))
        .collect();
    let sources: Vec<(&str, &str)> = case["input"]["sources"]
        .as_object()
        .expect("sources")
        .iter()
        .map(|(p, b)| (p.as_str(), b.as_str().expect("text")))
        .collect();
    let has_row_diagnostic = |module: &str| {
        run(module, &documents, &sources)
            .diagnostics
            .iter()
            .any(|d| d.reason == "criterion-tagged-outside-row")
    };
    assert!(has_row_diagnostic(module), "single-name target joins");

    // `test-case` is bound by no `obligations:` source, so it may widen.
    assert!(module.contains(anchor));
    for declared in ["[ID, Other]", "'I*'"] {
        let widened = module.replacen(
            anchor,
            &format!("    section: Test Cases\n    id_column: {declared}\n"),
            1,
        );
        assert!(
            !has_row_diagnostic(&widened),
            "{declared}: loads but is never the row-binding join partner"
        );
    }
}
