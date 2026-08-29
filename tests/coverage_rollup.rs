//! FR-050 — coverage reconciliation (TC-734..TC-740) and the FR-050-CON-2 /
//! FR-051-CON-1 static boundary audit (TC-756).

use std::fs;
use std::path::{Path, PathBuf};

use ix_trace_rs::trace;
use quire_rs::coverage::{compute, CoverageError, CoverageReport};
use quire_rs::metric::{Measurement, Metric};
use quire_rs::symbols::trace::BindingCensus;
use quire_rs::symbols::{extract_tree, trace};
use quire_rs::{validate_bundle_at, BundlePosture, Registry, Spec};

fn fixture_module(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("traceability")
        .join(name)
}

fn tmpdir(suffix: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("quire-rs-coverage-{}-{suffix}", std::process::id()));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).expect("mkdir");
    p
}

fn write(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("mkdir");
    }
    fs::write(path, body).expect("write");
}

/// An ISO-shaped bundle: one FR minting two AC ids, a matrix minting two TC
/// rows, and a Rust source tree whose tests carry `#[trace(...)]` markers.
struct Bundle {
    scope: PathBuf,
    source: PathBuf,
}

fn iso_bundle(suffix: &str, matrix_rows: &[(&str, &str, &str)], traced: &[&str]) -> Bundle {
    let root = tmpdir(suffix);
    let scope = root.join("spec");
    let source = root.join("src");
    fs::create_dir_all(&scope).expect("mkdir");
    fs::create_dir_all(&source).expect("mkdir");

    write(
        &scope,
        "FR-001.md",
        "---\nid: FR-001\ntype: FR\ntitle: A requirement\n---\n\n\
         ## Acceptance Criteria\n\n\
         | ID | Criteria | Verification |\n|----|----------|--------------|\n\
         | FR-001-AC-1 | The system shall do it. | Test (TC-001) |\n\
         | FR-001-AC-2 | The system shall also do it. | Test (TC-002) |\n",
    );

    // CR-062: the matrix is reached by archetype, so it must declare one. It
    // was frontmatter-less while `document:` binding read it off-corpus.
    let mut matrix = String::from(
        "---\nid: TM-001\ntype: TestMatrix\ntitle: Test Matrix\n---\n\n\
         # Test Matrix\n\n## Test Cases\n\n| ID | Traces To | Status |\n|----|-----------|--------|\n",
    );
    for (tc, traces_to, status) in matrix_rows {
        matrix.push_str(&format!("| {tc} | {traces_to} | {status} |\n"));
    }
    write(&scope, "tests.md", &matrix);

    let mut lib = String::from("//! Fixture source tree.\n\n#[cfg(test)]\nmod tests {\n");
    for (idx, id) in traced.iter().enumerate() {
        lib.push_str(&format!(
            "    #[trace(\"{id}\")]\n    #[test]\n    fn covers_{idx}() {{\n        let _ = 1;\n    }}\n"
        ));
    }
    lib.push_str("}\n");
    write(&source, "lib.rs", &lib);

    Bundle { scope, source }
}

/// Rewrite the bundle's FR with `cells` as its `Acceptance Criteria`, keeping
/// the ids and `Verification` references [`iso_bundle`] mints. Lets one test
/// choose whether the criteria classify as property-shaped or not.
fn rewrite_criteria(bundle: &Bundle, cells: &[&str]) {
    let mut md = String::from(
        "---\nid: FR-001\ntype: FR\ntitle: A requirement\n---\n\n\
         ## Acceptance Criteria\n\n\
         | ID | Criteria | Verification |\n|----|----------|--------------|\n",
    );
    for (i, cell) in cells.iter().enumerate() {
        md.push_str(&format!(
            "| FR-001-AC-{} | {cell} | Test (TC-{:03}) |\n",
            i + 1,
            i + 1
        ));
    }
    write(&bundle.scope, "FR-001.md", &md);
}

fn report_for(bundle: &Bundle, module: &str) -> Result<CoverageReport, CoverageError> {
    report_for_module(bundle, &fixture_module(module))
}

fn report_for_module(bundle: &Bundle, module: &Path) -> Result<CoverageReport, CoverageError> {
    let registry = Registry::load_module(module).expect("load module");
    let spec = Spec::from_path(&bundle.scope);
    let extraction = extract_tree(&bundle.source);
    let model = registry.traceability().cloned().unwrap_or_default();
    let graph = trace::bind(&extraction, &model);
    compute(&spec, &registry, &graph, &bundle.scope)
}

#[trace("TC-734", "FR-050-AC-3")]
// a reference row whose trace target has no backing
// `verifies` relation appears in unbacked rows with the row id and target id.
#[test]
fn tc734_unbacked_rows() {
    let bundle = iso_bundle(
        "734",
        &[
            ("TC-001", "FR-001-AC-1", "✅"),
            ("TC-002", "FR-001-AC-2", "🚧"),
        ],
        // Only TC-001 is bound by a test.
        &["TC-001"],
    );
    let report = report_for(&bundle, "iso").expect("model declared");

    let unbacked: Vec<&str> = report
        .unbacked_rows
        .iter()
        .filter_map(|r| r.row_id.as_deref())
        .collect();
    assert!(unbacked.contains(&"TC-002"), "unbacked: {unbacked:?}");
    assert!(!unbacked.contains(&"TC-001"));

    let row = report
        .unbacked_rows
        .iter()
        .find(|r| r.row_id.as_deref() == Some("TC-002"))
        .expect("TC-002 row");
    assert_eq!(row.reference, "traces-to");
    assert_eq!(row.document, "tests.md");
    assert!(row.target_ids.contains(&"TC-002".to_string()));
    assert!(row.target_ids.contains(&"FR-001-AC-2".to_string()));
}

#[trace("TC-735", "FR-050-AC-4")]
// a `complete`-classed row with no backing symbol is a
// status lie; the same row with a backing symbol is not.
#[test]
fn tc735_status_lies() {
    // TC-002 claims ✅ but nothing binds it.
    let lying = iso_bundle(
        "735-lying",
        &[
            ("TC-001", "FR-001-AC-1", "✅"),
            ("TC-002", "FR-001-AC-2", "✅"),
        ],
        &["TC-001"],
    );
    let report = report_for(&lying, "iso").expect("model declared");
    let lies: Vec<&str> = report
        .status_lies
        .iter()
        .filter_map(|l| l.row_id.as_deref())
        .collect();
    assert_eq!(lies, vec!["TC-002"]);
    assert_eq!(report.status_lies[0].status, "✅");

    // Bind it, and the lie disappears.
    let honest = iso_bundle(
        "735-honest",
        &[
            ("TC-001", "FR-001-AC-1", "✅"),
            ("TC-002", "FR-001-AC-2", "✅"),
        ],
        &["TC-001", "TC-002"],
    );
    assert!(report_for(&honest, "iso")
        .expect("model declared")
        .status_lies
        .is_empty());

    // A pending row that is unbacked is *not* a lie — only `complete` is.
    let pending = iso_bundle("735-pending", &[("TC-001", "FR-001-AC-1", "🚧")], &[]);
    assert!(report_for(&pending, "iso")
        .expect("model declared")
        .status_lies
        .is_empty());
}

#[trace("TC-941", "FR-050-AC-21")]
// a status value the model classes as nothing is reported
// as its own defect, carrying the authored string verbatim — and a corpus whose
// every status is declared omits the key entirely.
#[test]
fn tc941_undeclared_status_is_reported() {
    // `🟡` is in no `traceability.status` list, so `class_of` returns `Unknown`.
    // Before CR-083 that was computed and thrown away: the row was not backed,
    // not a lie, and named nowhere.
    //
    // The glyph matters. `⚠️` would *not* work here — this fixture classes it as
    // `pending` (`fixtures/traceability/iso/manifest.yaml:54`), which is the
    // choice `spec-artifacts-process` never made and the reason the real corpus
    // has the divergence this check exists to catch. `🟡` is undeclared in both,
    // and is authored in the wild (`ecaz/spec/tests.md`).
    let drifting = iso_bundle(
        "941-drift",
        &[
            ("TC-001", "FR-001-AC-1", "✅"),
            ("TC-002", "FR-001-AC-2", "🟡 scale evidence deferred"),
        ],
        &["TC-001"],
    );
    let report = report_for(&drifting, "iso").expect("model declared");

    let drifted: Vec<&str> = report
        .undeclared_statuses
        .iter()
        .filter_map(|s| s.row_id.as_deref())
        .collect();
    assert_eq!(drifted, vec!["TC-002"], "report: {report:?}");

    let entry = &report.undeclared_statuses[0];
    assert_eq!(entry.reference, "traces-to");
    assert_eq!(entry.document, "tests.md");
    // Verbatim, note and all: the reader needs to see which value drifted, and
    // the leading marker alone would not distinguish two undeclared glyphs.
    assert_eq!(entry.status, "🟡 scale evidence deferred");

    // FR-050-AC-7 byte-identity: a conformant corpus serializes exactly as it
    // did before the field existed.
    let clean = iso_bundle(
        "941-clean",
        &[
            ("TC-001", "FR-001-AC-1", "✅"),
            ("TC-002", "FR-001-AC-2", "🚧"),
        ],
        &["TC-001"],
    );
    let clean_report = report_for(&clean, "iso").expect("model declared");
    assert!(clean_report.undeclared_statuses.is_empty());
    assert!(
        !clean_report.to_json().contains("undeclared_statuses"),
        "an all-declared corpus must not carry the key at all",
    );
}

#[trace("TC-942", "FR-050-AC-21")]
// vocabulary drift is reported on a *backed* row too. The
// classification sits above the backed early-continue deliberately: drift is a
// property of the declaration, not of the row's evidence.
#[test]
fn tc942_undeclared_status_is_seen_on_a_backed_row() {
    // TC-001 is bound by a real symbol, so it never reaches `unbacked_rows` and
    // never reaches the status-lie block. Classing status down there would
    // report drift on unbacked rows only — a backstop that sees a subset.
    let bundle = iso_bundle(
        "942",
        &[
            ("TC-001", "FR-001-AC-1", "🟡 review-open"),
            ("TC-002", "FR-001-AC-2", "🚧"),
        ],
        &["TC-001", "TC-002"],
    );
    let report = report_for(&bundle, "iso").expect("model declared");

    assert!(
        report.unbacked_rows.is_empty(),
        "fixture premise: every row is backed — {:?}",
        report.unbacked_rows,
    );
    assert!(report.status_lies.is_empty());

    let drifted: Vec<&str> = report
        .undeclared_statuses
        .iter()
        .filter_map(|s| s.row_id.as_deref())
        .collect();
    assert_eq!(
        drifted,
        vec!["TC-001"],
        "a backed row's undeclared status must still be reported",
    );
    assert_eq!(report.undeclared_statuses[0].status, "🟡 review-open");
}

#[trace("TC-946", "FR-050-AC-21")]
// two identical matching rows are one defect, not two records — the list is
// deduplicated after its sort, mirroring `untracked_symbols` (#213). Before
// this, `undeclared_statuses` was the only reconciliation list sorted but
// never deduplicated, so a duplicated authored row yielded a duplicated
// record.
#[test]
fn tc946_duplicate_undeclared_status_rows_yield_one_record() {
    // The same drifting row authored twice, byte-identical. The matrix is
    // malformed — a duplicated row id is its own (separate) defect — but the
    // reconciliation must not amplify it: one undeclared value at one row id
    // is one drift finding.
    let bundle = iso_bundle(
        "946-dup",
        &[
            ("TC-001", "FR-001-AC-1", "✅"),
            ("TC-002", "FR-001-AC-2", "🟡 scale evidence deferred"),
            ("TC-002", "FR-001-AC-2", "🟡 scale evidence deferred"),
        ],
        &["TC-001"],
    );
    let report = report_for(&bundle, "iso").expect("model declared");

    let drifted: Vec<&str> = report
        .undeclared_statuses
        .iter()
        .filter_map(|s| s.row_id.as_deref())
        .collect();
    assert_eq!(
        drifted,
        vec!["TC-002"],
        "identical duplicate rows must collapse to one record: {:?}",
        report.undeclared_statuses,
    );

    // Two DIFFERENT undeclared values on duplicate row ids are two findings:
    // dedup collapses identical records only, never distinct drifted values.
    let distinct = iso_bundle(
        "946-distinct",
        &[
            ("TC-001", "FR-001-AC-1", "✅"),
            ("TC-002", "FR-001-AC-2", "🟡 scale evidence deferred"),
            ("TC-002", "FR-001-AC-2", "🔵 second drifted value"),
        ],
        &["TC-001"],
    );
    let report = report_for(&distinct, "iso").expect("model declared");
    let statuses: Vec<&str> = report
        .undeclared_statuses
        .iter()
        .map(|s| s.status.as_str())
        .collect();
    assert_eq!(
        statuses,
        vec!["🔵 second drifted value", "🟡 scale evidence deferred"],
        "distinct drifted values must both survive the dedup",
    );
}

#[trace("TC-950", "FR-050-AC-23")]
// one test-case id names one source symbol: an id bound by
// two distinct symbols is reported in `shared_trace_ids` with both binders,
// ordered, and the rollup's counts are untouched — the row is still backed,
// which is exactly why the state needs its own surface (CR-087; the shipped
// instances were TC-943 ×2 and TC-944 ×2 in v0.41.0, invisible to every
// report).
#[test]
fn tc950_an_id_bound_by_two_symbols_is_reported_with_both_binders() {
    // `iso_bundle` mints one distinct test fn per entry, so a repeated id is
    // two different symbols binding the same id.
    let bundle = iso_bundle(
        "950",
        &[
            ("TC-001", "FR-001-AC-1", "✅"),
            ("TC-002", "FR-001-AC-2", "✅"),
        ],
        &["TC-001", "TC-001", "TC-002"],
    );
    let report = report_for(&bundle, "iso").expect("model declared");

    assert_eq!(
        report.shared_trace_ids.len(),
        1,
        "exactly one shared id: {:?}",
        report.shared_trace_ids,
    );
    let shared = &report.shared_trace_ids[0];
    assert_eq!(shared.trace_id, "TC-001");
    let symbols: Vec<&str> = shared.symbols.iter().map(|s| s.symbol.as_str()).collect();
    assert_eq!(
        symbols,
        vec!["tests::covers_0", "tests::covers_1"],
        "both distinct binders, deterministically ordered",
    );

    // The report is advisory: the row stays backed and the totals stay what
    // they were — that a shared id keeps its row green is the defect the list
    // exists to make visible, not something the list changes. The totals are
    // the bundle's four minted targets (two AC ids, two TC ids) with the two
    // TC ids symbol-backed — identical to the uniquely-bound bundle TC-951
    // measures.
    assert!(report.unbacked_rows.is_empty());
    assert!(report.status_lies.is_empty());
    assert_eq!((report.totals.backed, report.totals.total), (2, 4));
}

#[trace("TC-951", "FR-050-AC-23")]
// the check's negative space, in both halves. A corpus
// whose every status-row id is uniquely bound reports an empty list and the
// key is ABSENT from the JSON — byte-identity (FR-050-AC-7) for every
// repository already conformant, exactly as `undeclared_statuses` keeps it.
// And the scoping is load-bearing: an id whose rows carry NO status (an
// acceptance criterion verified by several tests) is legitimately N:1 and is
// never reported, even when several distinct symbols bind it (CR-087).
#[test]
fn tc951_uniquely_bound_ids_report_nothing_and_omit_the_key() {
    // Two extra symbols both bind the AC id directly — the deliberate
    // many-tests-per-criterion shape (TC-941/TC-942 both bind FR-050-AC-21 in
    // this very repository). The AC table carries no status column, so this
    // must mint no record.
    let bundle = iso_bundle(
        "951",
        &[
            ("TC-001", "FR-001-AC-1", "✅"),
            ("TC-002", "FR-001-AC-2", "✅"),
        ],
        &["TC-001", "TC-002", "FR-001-AC-1", "FR-001-AC-1"],
    );
    let report = report_for(&bundle, "iso").expect("model declared");

    assert!(
        report.shared_trace_ids.is_empty(),
        "unique status-row bindings and shared NON-status ids alike must mint \
         no record: {:?}",
        report.shared_trace_ids,
    );
    assert!(
        !report.to_json().contains("shared_trace_ids"),
        "a conformant corpus must not carry the key at all",
    );
}

#[trace("TC-953", "FR-050-AC-24")]
// what `source_exclude` subtracts is carried all the way
// into the report (#215): the count travels extraction → symbol graph →
// `CoverageReport.excluded_source_files` and the JSON key, and is ABSENT —
// never 0 — when nothing was excluded, keeping FR-050-AC-7 byte-identity for
// every repository already conformant. Before this an over-broad glob's
// subtraction was invisible in both human and JSON output, indistinguishable
// from tests that were never written.
#[test]
fn tc953_excluded_source_file_count_reaches_the_coverage_json() {
    use quire_rs::symbols::extract_tree_scoped;

    let bundle = iso_bundle("953", &[("TC-001", "FR-001-AC-1", "✅")], &["TC-001"]);
    let registry = Registry::load_module(&fixture_module("iso")).expect("load module");
    let spec = Spec::from_path(&bundle.scope);
    let model = registry.traceability().cloned().expect("declared model");

    // The glob subtracts the bundle's only source file: one exclusion.
    let excluded = extract_tree_scoped(&bundle.source, &[], &["lib.rs".to_string()]);
    let graph = trace::bind(&excluded, &model);
    let report = compute(&spec, &registry, &graph, &bundle.scope).expect("model declared");
    assert_eq!(
        report.excluded_source_files, 1,
        "the walk's count is the report's"
    );
    assert!(
        report.to_json().contains("\"excluded_source_files\": 1"),
        "the count must reach the JSON payload",
    );

    // No glob: nothing excluded, and the key is absent — not 0.
    let full = extract_tree_scoped(&bundle.source, &[], &[]);
    let graph = trace::bind(&full, &model);
    let report = compute(&spec, &registry, &graph, &bundle.scope).expect("model declared");
    assert_eq!(report.excluded_source_files, 0);
    assert!(
        !report.to_json().contains("excluded_source_files"),
        "a repository excluding nothing must not carry the key at all",
    );
}

#[trace("TC-955", "FR-050-AC-26")]
// every row-shaped record carries the 1-based document
// line of the matrix row it came from (#210): line information was discarded
// at `parse_table` since v0.1, so no consumer could render `path:line:` or
// jump an editor to the offending row. The fixture's row positions are
// hand-counted ground truth, frontmatter included — the same numbering
// `validate` findings use — and two unbacked rows in one document must report
// different lines.
#[test]
fn tc955_row_shaped_records_carry_the_matrix_row_line() {
    // tests.md: 5 frontmatter lines, blank, heading, blank, section heading,
    // blank, header (11), separator (12), rows at 13 and 14. FR-001.md: the
    // AC rows sit at 11 and 12.
    let bundle = iso_bundle(
        "955",
        &[
            ("TC-001", "FR-001-AC-1", "✅"),
            ("TC-002", "FR-001-AC-2", "🟡"),
        ],
        &[],
    );
    let report = report_for(&bundle, "iso").expect("model declared");

    let lines: Vec<(Option<&str>, &str, Option<usize>)> = report
        .unbacked_rows
        .iter()
        .map(|r| (r.row_id.as_deref(), r.document.as_str(), r.line))
        .collect();
    assert!(
        lines.contains(&(Some("TC-001"), "tests.md", Some(13)))
            && lines.contains(&(Some("TC-002"), "tests.md", Some(14))),
        "two unbacked rows in one document report their own distinct lines: {lines:?}",
    );
    assert!(
        lines.contains(&(Some("FR-001-AC-1"), "FR-001.md", Some(11)))
            && lines.contains(&(Some("FR-001-AC-2"), "FR-001.md", Some(12))),
        "the line accounts for the frontmatter block: {lines:?}",
    );

    // The lie and the undeclared status point at their rows too.
    assert_eq!(report.status_lies.len(), 1);
    assert_eq!(report.status_lies[0].line, Some(13));
    assert_eq!(report.undeclared_statuses.len(), 1);
    assert_eq!(report.undeclared_statuses[0].line, Some(14));

    // And so does a no-symbol row (Type column shifts nothing: the line is
    // the row's, not a cell's). typed_matrix_bundle rows sit at 13..16.
    let typed = typed_matrix_bundle("955-typed");
    let report = report_for(&typed, "no-symbol").expect("model declared");
    let no_symbol: Vec<(Option<&str>, Option<usize>)> = report
        .no_symbol_rows
        .iter()
        .map(|r| (r.row_id.as_deref(), r.line))
        .collect();
    assert_eq!(
        no_symbol,
        vec![(Some("TC-003"), Some(15)), (Some("TC-004"), Some(16))],
        "no-symbol rows carry their matrix-row lines",
    );
}

#[trace("TC-956", "FR-050-AC-26")]
// `untracked_symbols` carries the 1-based declaration
// line of the tagged test (#210) — `SymbolRecord` always had it, the
// `VerifiesRelation` in between discarded it.
#[test]
fn tc956_untracked_symbols_carry_the_tagged_tests_line() {
    // iso_bundle's lib.rs: 4 preamble lines, then 5 lines per traced entry —
    // covers_1's `fn` declaration sits on line 12.
    let bundle = iso_bundle(
        "956",
        &[("TC-001", "FR-001-AC-1", "✅")],
        &["TC-001", "TC-999"],
    );
    let report = report_for(&bundle, "iso").expect("model declared");

    assert_eq!(report.untracked_symbols.len(), 1);
    let entry = &report.untracked_symbols[0];
    assert_eq!(entry.trace_id, "TC-999");
    assert_eq!(
        entry.line,
        Some(12),
        "the untracked symbol's line is the tagged test's declaration line",
    );
}

#[trace("TC-736", "FR-050-AC-5")]
// a symbol whose trace tag resolves to no declared
// target or row appears in untracked symbols with its file and symbol name.
#[test]
fn tc736_untracked_symbols() {
    let bundle = iso_bundle(
        "736",
        &[("TC-001", "FR-001-AC-1", "✅")],
        &["TC-001", "TC-999"],
    );
    let report = report_for(&bundle, "iso").expect("model declared");

    let untracked: Vec<&str> = report
        .untracked_symbols
        .iter()
        .map(|s| s.trace_id.as_str())
        .collect();
    assert_eq!(untracked, vec!["TC-999"]);
    let entry = &report.untracked_symbols[0];
    assert_eq!(entry.path, "lib.rs");
    assert!(entry.symbol.contains("covers_1"));
}

#[trace("TC-737", "FR-050-AC-6")]
// per-minting-document counts, summing to the totals.
#[test]
fn tc737_per_group_counts_sum_to_totals() {
    let bundle = iso_bundle(
        "737",
        &[
            ("TC-001", "FR-001-AC-1", "✅"),
            ("TC-002", "FR-001-AC-2", "🚧"),
        ],
        &["TC-001", "FR-001-AC-1"],
    );
    let report = report_for(&bundle, "iso").expect("model declared");

    let fr = report
        .groups
        .iter()
        .find(|g| g.document == "FR-001.md")
        .expect("FR group");
    assert_eq!(fr.target, "acceptance-criterion");
    assert_eq!(fr.total, 2, "two AC ids minted");
    assert_eq!(fr.backed, 1, "only FR-001-AC-1 is bound");

    let matrix = report
        .groups
        .iter()
        .find(|g| g.document == "tests.md")
        .expect("matrix group");
    assert_eq!(matrix.total, 2);
    assert_eq!(matrix.backed, 1);

    assert_eq!(
        report.totals.total,
        report.groups.iter().map(|g| g.total).sum::<usize>()
    );
    assert_eq!(
        report.totals.backed,
        report.groups.iter().map(|g| g.backed).sum::<usize>()
    );
    assert_eq!(report.totals.total, 4);
    assert_eq!(report.totals.backed, 2);
}

#[trace("TC-1073", "FR-050-AC-38", "FR-066-AC-2")]
#[test]
fn tc1073_minted_targets_are_the_row_level_totals() {
    let bundle = iso_bundle(
        "1073",
        &[
            ("TC-002", "FR-001-AC-2", "🚧"),
            ("TC-001", "FR-001-AC-1", "✅"),
        ],
        &["TC-001", "FR-001-AC-1"],
    );
    let report = report_for(&bundle, "iso").expect("model declared");

    assert_eq!(report.minted_targets.len(), report.totals.total);
    assert_eq!(
        report
            .minted_targets
            .iter()
            .filter(|row| row.backed)
            .count(),
        report.totals.backed
    );
    assert!(report.minted_targets.iter().all(|row| {
        !row.id.is_empty() && !row.target.is_empty() && !row.document.is_empty() && row.line > 0
    }));
    assert!(report.minted_targets.windows(2).all(|rows| {
        (
            &rows[0].target,
            &rows[0].document,
            &rows[0].id,
            rows[0].line,
        ) <= (
            &rows[1].target,
            &rows[1].document,
            &rows[1].id,
            rows[1].line,
        )
    }));

    let empty_bundle = iso_bundle("1073-empty", &[], &[]);
    let empty = report_for(&empty_bundle, "fails-open").expect("model declared");
    assert!(empty.minted_targets.is_empty());
    let value: serde_json::Value = serde_json::from_str(&empty.to_json()).expect("JSON");
    assert!(value.get("minted_targets").is_none());
}

#[trace("TC-1075", "FR-050-AC-40", "FR-066-AC-2")]
#[test]
fn tc1075_reference_only_targets_resolve_without_entering_coverage() {
    let bundle = iso_bundle(
        "1075",
        &[
            ("TC-001", "FR-001-AC-1", "✅"),
            ("TC-002", "FR-001-AC-2", "✅"),
        ],
        &["TC-001", "FR-001-AC-1"],
    );
    let report = report_for(&bundle, "reference-only").expect("model declared");

    assert_eq!(report.totals.total, 2, "only the two source AC rows count");
    assert_eq!(report.totals.backed, 1, "the source AC tag still binds");
    assert!(report
        .groups
        .iter()
        .all(|group| group.target != "test-case"));
    assert!(report
        .minted_targets
        .iter()
        .all(|row| row.id != "TC-001" && row.target != "test-case"));

    let absent = iso_bundle("1075-absent", &[], &[]);
    fs::remove_file(absent.scope.join("tests.md")).expect("remove optional registry");
    let absent_report = report_for(&absent, "reference-only").expect("model declared");
    assert!(
        absent_report
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.declaration != "test-case"),
        "an optional reference registry must be silent when absent: {:?}",
        absent_report.diagnostics
    );

    let registry = Registry::load_module(&fixture_module("reference-only")).expect("load module");
    let resolved = validate_bundle_at(&bundle.scope, &registry, BundlePosture::Strict);
    assert!(
        resolved
            .errors
            .iter()
            .all(|finding| finding.reason != "dangling-trace-reference"),
        "the reference-only id must remain resolvable: {:?}",
        resolved.errors
    );

    write(
        &bundle.scope,
        "FR-001.md",
        "---\nid: FR-001\ntype: FR\ntitle: A requirement\n---\n\n\
         ## Acceptance Criteria\n\n\
         | ID | Criteria | Verification |\n|----|----------|--------------|\n\
         | FR-001-AC-1 | The system shall do it. | Test (TC-404) |\n",
    );
    let dangling = validate_bundle_at(&bundle.scope, &registry, BundlePosture::Strict);
    assert!(dangling
        .errors
        .iter()
        .any(|finding| finding.reason == "dangling-trace-reference"));
}

#[trace("TC-738", "FR-050-AC-7")]
// repeated runs over identical inputs emit (Property)
// byte-identical JSON.
#[test]
fn tc738_report_json_is_byte_identical() {
    let bundle = iso_bundle(
        "738",
        &[
            ("TC-001", "FR-001-AC-1", "✅"),
            ("TC-002", "FR-001-AC-2", "✅"),
            ("TC-003", "FR-001-AC-2", "❌"),
        ],
        &["TC-001", "TC-999"],
    );
    let first = report_for(&bundle, "iso")
        .expect("model declared")
        .to_json();
    assert!(first.contains("unbacked_rows"));
    for _ in 0..8 {
        assert_eq!(
            first,
            report_for(&bundle, "iso")
                .expect("model declared")
                .to_json(),
            "coverage JSON must be byte-identical across runs"
        );
    }
}

#[trace("TC-739", "FR-050-AC-8")]
// a non-ISO vocabulary gets a correct rollup from its
// own declaration, with no engine change.
#[test]
fn tc739_non_iso_model_rolls_up() {
    let root = tmpdir("739");
    let scope = root.join("rules");
    let source = root.join("src");
    fs::create_dir_all(&scope).expect("mkdir");
    fs::create_dir_all(&source).expect("mkdir");

    write(
        &scope,
        "R-001.md",
        "---\nid: R-001\ntype: Rule\ntitle: A rule\n---\n\n## Clauses\n\n\
         | Clause | Evidence |\n|--------|----------|\n\
         | R-001-C-1 | checked by C-001 |\n\
         | R-001-C-2 | checked by C-002 |\n",
    );
    write(
        &scope,
        "checks.md",
        // CR-062: reached by archetype, so it declares one — `CheckRegister`,
        // the alt module's own evidence archetype, not the ISO `TestMatrix`.
        "---\nid: CR-001\ntype: CheckRegister\ntitle: Check Register\n---\n\n\
         # Check Register\n\n## Checks\n\n| Check | Covers | State |\n|-------|--------|-------|\n\
         | C-001 | R-001-C-1 | done |\n\
         | C-002 | R-001-C-2 | done |\n",
    );
    // The alt module declares `#[covers(...)]`, not `#[trace(...)]`.
    write(
        &source,
        "lib.rs",
        "#[cfg(test)]\nmod tests {\n    #[covers(\"C-001\")]\n    #[test]\n    fn one() {\n        let _ = 1;\n    }\n}\n",
    );

    let bundle = Bundle { scope, source };
    let report = report_for(&bundle, "alt").expect("model declared");

    // Clause + check groups both counted from the alt declaration.
    assert_eq!(report.totals.total, 4);
    assert_eq!(report.totals.backed, 1, "only C-001 is covered");
    let clauses = report
        .groups
        .iter()
        .find(|g| g.target == "clause")
        .expect("clause group");
    assert_eq!(clauses.total, 2);
    // C-002's row claims `done` while nothing covers it — a status lie in the
    // alt vocabulary, with no ISO status value anywhere in play.
    let lies: Vec<&str> = report
        .status_lies
        .iter()
        .filter_map(|l| l.row_id.as_deref())
        .collect();
    assert_eq!(lies, vec!["C-002"]);
    assert_eq!(report.status_lies[0].status, "done");
}

#[trace("TC-740", "FR-050-AC-9")]
// with no declared model, coverage fails with a distinct
// diagnostic naming the missing declaration — never an empty report.
#[test]
fn tc740_no_model_is_a_distinct_diagnostic() {
    let bundle = iso_bundle("740", &[("TC-001", "FR-001-AC-1", "✅")], &["TC-001"]);
    let module = tmpdir("740-module");
    fs::write(
        module.join("manifest.yaml"),
        "name: no-model\nartifact_types:\n- name: FR\n",
    )
    .expect("write manifest");

    let registry = Registry::load_module(&module).expect("load module");
    let spec = Spec::from_path(&bundle.scope);
    let graph = trace::bind(&extract_tree(&bundle.source), &Default::default());
    let err = compute(&spec, &registry, &graph, &bundle.scope).expect_err("must not report");

    assert_eq!(err, CoverageError::ModelUndeclared);
    assert!(err.to_string().contains("traceability"));
}

#[trace("TC-788", "FR-052-AC-10", "FR-050-AC-13")]
// the CR-028 criteria rollup — one entry
// per document binding criteria plus the two new totals, and byte-identical
// serialization across runs.
#[test]
fn tc788_criteria_counts_and_totals() {
    let bundle = iso_bundle(
        "788",
        &[
            ("TC-001", "FR-001-AC-1", "✅"),
            ("TC-002", "FR-001-AC-2", "🚧"),
        ],
        &["TC-001"],
    );
    // A universally quantified criterion and a specific-scenario one, so the
    // entry carries a property-shaped count that is neither 0 nor the total.
    rewrite_criteria(
        &bundle,
        &[
            "A finding whose key is absent from the merged map defaults to warning.",
            "The loader emits a `Duplicate` diagnostic for the second declaration.",
        ],
    );
    let report = report_for(&bundle, "iso").expect("model declared");

    // One entry per minting document. These fixture matrices carry no
    // frontmatter, so under CR-044 membership they are not documents and bind
    // no criteria — `FR-001.md` is the only contributor. (When #74 removes the
    // `document:` origin they gain `type: TestMatrix` and this inverts.)
    assert_eq!(
        report
            .criteria
            .iter()
            .map(|c| c.document.as_str())
            .collect::<Vec<_>>(),
        vec!["FR-001.md"],
    );
    let entry = &report.criteria[0];
    assert_eq!(entry.archetype, "FR");
    assert_eq!(entry.criteria, 2, "both AC rows are binding criteria");
    assert_eq!(
        entry.property_shaped, 1,
        "only the quantified criterion is extractable: {:?}",
        entry.by_property
    );
    // The histogram accounts for every criterion exactly once.
    assert_eq!(
        entry.by_property.values().sum::<usize>(),
        entry.criteria,
        "by_property: {:?}",
        entry.by_property
    );

    // The totals are the sum over the entries — the same relation FR-050-AC-6
    // states for backed/total — and they are present as a pair.
    assert_eq!(
        report.totals.criteria,
        Some(report.criteria.iter().map(|c| c.criteria).sum::<usize>())
    );
    assert_eq!(
        report.totals.property_shaped,
        Some(
            report
                .criteria
                .iter()
                .map(|c| c.property_shaped)
                .sum::<usize>()
        )
    );
    assert_eq!(report.totals.criteria, Some(2));
    assert_eq!(report.totals.property_shaped, Some(1));

    // …and the classification is data, not a verdict: it moves nothing in the
    // reconciliation the report already carried.
    assert_eq!(report.totals.total, 4);
    assert_eq!(report.totals.backed, 1);

    // FR-050-AC-7 still holds over the enlarged payload.
    let first = report.to_json();
    assert!(first.contains("\"criteria\""));
    assert!(first.contains("property_shaped"));
    for _ in 0..8 {
        assert_eq!(
            first,
            report_for(&bundle, "iso")
                .expect("model declared")
                .to_json(),
            "coverage JSON must stay byte-identical across runs"
        );
    }
}

#[trace("TC-788", "FR-050-AC-13")]
// a corpus binding criteria of which *none* (continued)
// are property-shaped emits `property_shaped: 0` — present and zero, never
// absent. The two totals move as a pair, so a JSON consumer computing the
// extraction ratio divides by a number rather than by `undefined`, in exactly
// the corpus most worth reporting on (CR-020: criteria validated by
// demonstration legitimately score zero).
#[test]
fn tc788_zero_property_shaped_is_emitted_not_omitted() {
    let bundle = iso_bundle("788-zero", &[("TC-001", "FR-001-AC-1", "✅")], &["TC-001"]);
    // Two specific scenarios: binding criteria, neither quantified.
    rewrite_criteria(
        &bundle,
        &[
            "The loader emits a `Duplicate` diagnostic for the second declaration.",
            "The report lists one row per declared module.",
        ],
    );
    let report = report_for(&bundle, "iso").expect("model declared");

    assert_eq!(report.criteria.len(), 1);
    assert_eq!(report.criteria[0].criteria, 2);
    assert_eq!(report.criteria[0].property_shaped, 0);
    assert_eq!(report.totals.criteria, Some(2));
    assert_eq!(
        report.totals.property_shaped,
        Some(0),
        "zero is a value, not an absence"
    );

    let json = report.to_json();
    let value: serde_json::Value = serde_json::from_str(&json).expect("parses");
    let totals: Vec<&str> = value["totals"]
        .as_object()
        .expect("object")
        .keys()
        .map(String::as_str)
        .collect();
    // CR-095 adds `specific_shaped` to the all-or-nothing set: it is the
    // honest half of the same headline, so it is present exactly when the
    // other two are and absent exactly when they are.
    assert_eq!(
        totals,
        vec![
            "backed",
            "criteria",
            "property_shaped",
            "specific_shaped",
            "total"
        ]
    );
    assert_eq!(value["totals"]["property_shaped"], serde_json::json!(0));
    assert_eq!(
        value["totals"]["specific_shaped"],
        serde_json::json!(0),
        "zero is a value here too"
    );
    assert!(
        json.contains("\"property_shaped\": 0"),
        "the key must be written, not skipped: {json}"
    );

    let restored: CoverageReport = serde_json::from_str(&json).expect("round-trips");
    assert_eq!(restored, report);
}

#[trace("TC-788", "FR-050-AC-13")]
// a corpus binding no criteria carries an (continued)
// empty list, and its JSON is byte-for-byte what an engine predating the
// field would have written — the CR-028 keys are absent, not zero-valued.
#[test]
fn tc788_no_criteria_corpus_is_unchanged() {
    let root = tmpdir("788-none");
    let scope = root.join("rules");
    let source = root.join("src");
    fs::create_dir_all(&scope).expect("mkdir");
    fs::create_dir_all(&source).expect("mkdir");

    // The alt archetype declares no `grammar_ref`, so nothing binds criteria.
    write(
        &scope,
        "R-001.md",
        "---\nid: R-001\ntype: Rule\ntitle: A rule\n---\n\n## Clauses\n\n\
         | Clause | Evidence |\n|--------|----------|\n\
         | R-001-C-1 | checked by C-001 |\n",
    );
    write(
        &scope,
        "checks.md",
        "# Check Register\n\n## Checks\n\n| Check | Covers | State |\n|-------|--------|-------|\n\
         | C-001 | R-001-C-1 | done |\n",
    );
    write(&source, "lib.rs", "//! No traced symbols.\n");

    let bundle = Bundle { scope, source };
    let report = report_for(&bundle, "alt").expect("model declared");

    assert!(report.criteria.is_empty(), "{:?}", report.criteria);
    assert_eq!(report.totals.criteria, None);
    assert_eq!(report.totals.property_shaped, None);

    let json = report.to_json();
    // Asserted as an absent KEY rather than an absent substring (CR-094): the
    // FR-063 metric block names `coverage.property_shaped` even when it was
    // not computed — that naming is the point of the envelope — so a substring
    // test now answers a different question from the one this AC asks.
    let value: serde_json::Value = serde_json::from_str(&json).expect("parses");
    let object = value.as_object().expect("object");
    assert!(
        !object.contains_key("criteria"),
        "an absent field, not an empty one: {json}"
    );
    let keys: Vec<&str> = object.keys().map(String::as_str).collect();
    // `serde_json::Value` holds its object as a map, so the comparison is over
    // the key *set*, not the (separately asserted) emitted order.
    // `diagnostics` is in the set since CR-135 (#304): this fixture's model
    // declares a trace target over an archetype the corpus has no document of,
    // and the model-wide gate that used to hide that as soon as anything else
    // minted is gone. The key this AC is about is `criteria`, asserted absent
    // above; the rest of the set is listed so an unnoticed new key fails here
    // rather than in a consumer.
    assert_eq!(
        keys,
        vec![
            "diagnostic_reason_registry",
            "diagnostics",
            "groups",
            "metrics",
            "minted_targets",
            "status_lies",
            "totals",
            "unbacked_rows",
            "untracked_symbols"
        ]
    );
    // The uncomputed metric says so in its own state, and carries no numbers
    // that could be read as a zero.
    let shaped = value["metrics"]
        .as_array()
        .expect("metrics")
        .iter()
        .find(|m| m["name"] == "coverage.property_shaped")
        .expect("the metric is named whether or not it ran");
    assert_eq!(shaped["state"], "not_computed");
    assert!(shaped.get("value").is_none());
    assert!(!value["totals"]
        .as_object()
        .expect("object")
        .contains_key("property_shaped"));
    let totals: Vec<&str> = value["totals"]
        .as_object()
        .expect("object")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(totals, vec!["backed", "total"]);

    // A report written by an older engine still round-trips through the
    // derived `Deserialize` — what `#[serde(default)]` buys.
    let restored: CoverageReport = serde_json::from_str(&json).expect("round-trips");
    assert_eq!(restored, report);
}

#[trace("TC-756", "FR-050-CON-2", "FR-051-CON-1")]
// static boundary audit over the coverage
// and symbol modules — no network or service I/O, and no execution of the code
// the symbols were extracted from. Mirrors the TC-690 pattern.
#[test]
fn tc756_coverage_and_symbol_modules_have_no_forbidden_surface() {
    const SOURCES: &[(&str, &str)] = &[
        ("src/coverage.rs", include_str!("../src/coverage.rs")),
        ("src/symbols/mod.rs", include_str!("../src/symbols/mod.rs")),
        (
            "src/symbols/rust.rs",
            include_str!("../src/symbols/rust.rs"),
        ),
        (
            "src/symbols/python.rs",
            include_str!("../src/symbols/python.rs"),
        ),
        (
            "src/symbols/typescript.rs",
            include_str!("../src/symbols/typescript.rs"),
        ),
        (
            "src/symbols/trace.rs",
            include_str!("../src/symbols/trace.rs"),
        ),
        (
            "src/corpus/declared_tables.rs",
            include_str!("../src/corpus/declared_tables.rs"),
        ),
        (
            "src/corpus/trace_refs.rs",
            include_str!("../src/corpus/trace_refs.rs"),
        ),
    ];
    // Reading local files is explicitly in scope ("inputs are the corpus, the
    // registry, and local source trees"), so `std::fs` is allowed here — what
    // must never appear is a network, service, or *execution* surface.
    const FORBIDDEN: &[&str] = &[
        "std::net",
        "TcpStream",
        "UdpSocket",
        "reqwest",
        "hyper",
        // `tokio` alone would trip on the Rust adapter's documentation of the
        // `#[tokio::test]` attribute it classifies — the surface that matters
        // is an actual runtime dependency.
        "use tokio",
        "tokio::runtime",
        "tokio::spawn",
        "Command",
        "std::process",
        "libloading",
        "dlopen",
        "PGlite",
        "pglite",
        "CloudManager",
        "cloud_manager",
        "embedding",
    ];
    for (name, source) in SOURCES {
        for needle in FORBIDDEN {
            assert!(
                !source.contains(needle),
                "{name} must not reference {needle:?}: the coverage/symbol boundary performs no \
                 network or service I/O and never executes extracted code \
                 (FR-050-CON-2, FR-051-CON-1)"
            );
        }
    }
}

// ── CR-015: status classification, declared vocabularies, cell normalization ──

/// A module manifest declaring only what a test needs from the model.
fn model_from(yaml: &str) -> quire_rs::traceability::TraceabilityModel {
    let module = tmpdir(&format!("cr015-{}", yaml.len()));
    fs::write(module.join("manifest.yaml"), yaml).expect("write manifest");
    Registry::load_module(&module)
        .expect("load module")
        .traceability()
        .cloned()
        .expect("declared model")
}

#[trace("TC-758", "FR-050-AC-10")]
// a status cell with a trailing note classes by its
// leading marker, and a declared `retired` value classes retired.
#[test]
fn tc758_status_classes_by_leading_marker() {
    use quire_rs::traceability::StatusClass;
    let model = model_from(
        "name: m\ntraceability:\n  status:\n    column: Status\n    complete: [\"✅\"]\n    \
         pending: [\"🚧\"]\n    failed: [\"❌\"]\n    retired: [\"⛔\"]\n",
    );
    let status = model.status.as_ref().expect("status vocabulary");

    // Bare markers still class exactly.
    assert_eq!(status.class_of("✅"), StatusClass::Complete);
    // …and a marker carrying the reason classes the same, keeping the note.
    assert_eq!(status.class_of("✅ Complete"), StatusClass::Complete);
    assert_eq!(
        status.class_of("🚧 implementation in progress"),
        StatusClass::Pending
    );
    assert_eq!(
        status.class_of("⛔ RETIRED — render removed"),
        StatusClass::Retired
    );
    // An undeclared value is still unknown, not silently absorbed.
    assert_eq!(status.class_of("Done"), StatusClass::Unknown);
    // A word vocabulary keeps working, and a longer word is not a prefix match.
    let words = model_from(
        "name: w\ntraceability:\n  status:\n    column: State\n    complete: [\"done\"]\n",
    );
    let words = words.status.as_ref().unwrap();
    assert_eq!(words.class_of("done"), StatusClass::Complete);
    assert_eq!(words.class_of("done, verified"), StatusClass::Complete);
    assert_eq!(words.class_of("doneish"), StatusClass::Unknown);
}

#[trace("TC-759", "FR-050-AC-11")]
// a declared column vocabulary is exposed on the
// Registry, and is the same list a matrix contract would validate against.
#[test]
fn tc759_declared_column_vocabulary() {
    let module = tmpdir("cr015-vocab");
    fs::write(
        module.join("manifest.yaml"),
        "name: m\ntraceability:\n  vocabularies:\n    test_type: [Unit, Integration, E2E, \
         Property, pg_test]\n",
    )
    .expect("write manifest");
    let registry = Registry::load_module(&module).expect("load module");

    assert_eq!(
        registry.column_vocabulary("test_type"),
        ["Unit", "Integration", "E2E", "Property", "pg_test"]
    );
    // An undeclared column reports empty rather than a guessed default.
    assert!(registry.column_vocabulary("priority").is_empty());
    let bare = Registry::load_module(&fixture_module("alt")).expect("load module");
    assert!(bare.column_vocabulary("test_type").is_empty());
}

/// A bundle in the shape the CR-038 scoping exists for: a canonical
/// `spec/tests.md` reached by `document:` binding rather than by the walk —
/// it carries no frontmatter, so CR-044 membership excludes it — a second
/// in-corpus matrix, and a
/// **fixture** matrix under `fixtures/` whose rows are test data — including one
/// reusing a real id, which is what turns a fixture into a phantom backed row.
fn scoped_bundle(suffix: &str) -> Bundle {
    let bundle = iso_bundle(
        suffix,
        &[
            ("TC-001", "FR-001-AC-1", "✅"),
            ("TC-002", "FR-001-AC-2", "🚧"),
        ],
        &["TC-001"],
    );
    // In-corpus matrix: a real claim, and the reason `archetype:` binding is
    // wanted at all.
    write(
        &bundle.scope,
        "matrix.md",
        "---\nid: TM-002\ntype: TestMatrix\ntitle: Extra matrix\n---\n\n\
         ## Test Cases\n\n| ID | Traces To | Status |\n|----|-----------|--------|\n\
         | TC-003 | FR-001-AC-1 | 🚧 |\n",
    );
    // Fixture matrix: deliberately reuses TC-001, so admitting it mints a
    // second TC-001 that the real test appears to back.
    write(
        &bundle.scope,
        "fixtures/bad-matrix.md",
        "---\nid: TM-900\ntype: TestMatrix\ntitle: Deliberately malformed fixture\n---\n\n\
         ## Test Cases\n\n| ID | Traces To | Status |\n|----|-----------|--------|\n\
         | TC-001 | FR-001-AC-1 | ✅ |\n\
         | TC-900 | FR-001-AC-2 | ✅ |\n",
    );
    bundle
}

#[trace("TC-801", "FR-050-AC-15")]
// a declaration excluding `fixtures/**` contributes no
// rows from a matching document; the same corpus without the exclusion reports
// the fixture's rows as real.
#[test]
fn tc801_excluded_documents_contribute_no_rows() {
    let bundle = scoped_bundle("801");

    let scoped = report_for(&bundle, "scoped").expect("model declared");
    let scoped_docs: Vec<&str> = scoped.groups.iter().map(|g| g.document.as_str()).collect();
    assert!(
        !scoped_docs.contains(&"fixtures/bad-matrix.md"),
        "the fixture matrix must mint nothing: {scoped_docs:?}"
    );
    assert!(
        !scoped
            .unbacked_rows
            .iter()
            .any(|r| r.document == "fixtures/bad-matrix.md"),
        "an excluded document cannot produce reference rows"
    );
    assert!(
        !scoped
            .status_lies
            .iter()
            .any(|l| l.row_id.as_deref() == Some("TC-900")),
        "TC-900 exists only in the fixture: {:?}",
        scoped.status_lies
    );

    // The control: identical corpus, exclusion removed.
    let unscoped = report_for(&bundle, "unscoped").expect("model declared");
    let unscoped_docs: Vec<&str> = unscoped
        .groups
        .iter()
        .map(|g| g.document.as_str())
        .collect();
    assert!(
        unscoped_docs.contains(&"fixtures/bad-matrix.md"),
        "without the exclusion the fixture is admitted: {unscoped_docs:?}"
    );
    // The phantom the exclusion exists to prevent: a fixture row reusing TC-001
    // reads as backed, because the real test bound that id.
    let phantom = unscoped
        .groups
        .iter()
        .find(|g| g.document == "fixtures/bad-matrix.md")
        .expect("fixture group");
    assert_eq!(
        phantom.backed, 1,
        "the fixture's reused id reads as backed: {phantom:?}"
    );
    assert!(unscoped.totals.total > scoped.totals.total);
}

#[trace("TC-826")]
// a model-level `exclude:` scopes the (FR-050-AC-13 / AC-15, CR-060)
// **criteria walk**, which has no declaration of its own to hang an exclusion
// on and so had none at all before this.
//
// Deliberately malformed fixture data was minting nothing and referencing
// nothing — correct — while still contributing to `criteria` and to both
// totals, inflating the denominator, and being body-parsed despite the
// declaration saying it is not corpus data (agent-ix/quire-rs#124).
#[test]
fn tc826_model_level_exclusion_scopes_the_criteria_walk() {
    let bundle = iso_bundle(
        "826-model-scoped",
        &[("TC-001", "FR-001-AC-1", "✅")],
        &["TC-001"],
    );
    // A fixture FR: same archetype, same grammar binding, deliberately reusing
    // a real AC id and referencing a TC nothing mints.
    write(
        &bundle.scope,
        "fixtures/FR-900.md",
        "---\nid: FR-900\ntype: FR\ntitle: Deliberately malformed fixture\n---\n\n\
         ## Acceptance Criteria\n\n\
         | ID | Criteria | Verification |\n|----|----------|--------------|\n\
         | FR-001-AC-1 | The system shall always hold it. | Test (TC-666) |\n",
    );

    let scoped = report_for(&bundle, "model-scoped").expect("model declared");
    assert!(
        !scoped
            .criteria
            .iter()
            .any(|c| c.document.contains("fixtures/")),
        "an excluded document contributed criteria: {:?}",
        scoped.criteria
    );
    assert!(
        !scoped
            .groups
            .iter()
            .any(|g| g.document.contains("fixtures/")),
        "and mints no ids: {:?}",
        scoped.groups
    );
    assert!(
        !scoped.to_json().contains("TC-666"),
        "and its references are never read"
    );

    // The exclusion scopes **traceability**, not membership: the fixture is
    // still a document in the corpus, and `validate_bundle` still schema- and
    // grammar-checks it. Being outside the rollup is not a licence to be
    // malformed in ways nobody reports.
    let spec = Spec::from_path(&bundle.scope);
    assert!(
        spec.by_id("FR-900").is_some(),
        "an excluded document is still loaded and still validated"
    );

    // The control: the same corpus under a model declaring no exclusion at all.
    // Every count the fixture inflates is visible here.
    let unscoped = report_for(&bundle, "iso").expect("model declared");
    assert!(
        unscoped
            .criteria
            .iter()
            .any(|c| c.document.contains("fixtures/")),
        "without the exclusion the fixture is counted: {:?}",
        unscoped.criteria
    );
    assert!(
        unscoped.totals.criteria > scoped.totals.criteria,
        "the fixture inflates the criteria denominator: {:?} vs {:?}",
        unscoped.totals,
        scoped.totals
    );
    assert!(
        unscoped.totals.property_shaped > scoped.totals.property_shaped,
        "and the property-shaped numerator with it"
    );
}

#[trace("TC-830", "FR-050-AC-15")]
// ONE archetype-bound entry mints from every (CR-062)
// matrix in the corpus, whatever each one is called. This is what replaced
// enumeration: the retired `document:` form needed an entry per filename and
// still reached nothing nested.
#[test]
fn tc830_one_archetype_entry_mints_from_every_matrix_filename() {
    let bundle = scoped_bundle("830");
    let report = report_for(&bundle, "scoped").expect("model declared");

    let documents: Vec<&str> = report
        .groups
        .iter()
        .filter(|g| g.target == "test-case")
        .map(|g| g.document.as_str())
        .collect();
    // Two different filenames, one declaration — and `scoped` declares exactly
    // one `test-case` entry, so neither is reached by a path.
    assert!(
        documents.contains(&"tests.md"),
        "the canonical filename must mint: {documents:?}"
    );
    assert!(
        documents.contains(&"matrix.md"),
        "a differently-named matrix must mint from the same entry: {documents:?}"
    );

    // Rows from both documents reconcile against the same target kind: TC-003
    // lives in `matrix.md` and nothing binds it.
    assert!(
        report
            .unbacked_rows
            .iter()
            .any(|r| r.document == "matrix.md" && r.row_id.as_deref() == Some("TC-003")),
        "unbacked: {:?}",
        report.unbacked_rows
    );
}

/// A matrix whose rows carry a `Type` column, so a declaration can exempt the
/// methods that mint no source symbol.
fn typed_matrix_bundle(suffix: &str) -> Bundle {
    let bundle = iso_bundle(suffix, &[], &["TC-001"]);
    let mut matrix = String::from(
        "---\nid: TM-001\ntype: TestMatrix\ntitle: Test Matrix\n---\n\n\
         # Test Matrix\n\n## Test Cases\n\n| ID | Type | Traces To | Status |\n\
         |----|------|-----------|--------|\n",
    );
    for (tc, ty, traces, status) in [
        // Backed by a real test: never a lie, exempt or not.
        ("TC-001", "Unit", "FR-001-AC-1", "✅"),
        // Unbacked, and its method could have produced a symbol → a lie.
        ("TC-002", "Unit", "FR-001-AC-2", "✅"),
        // Unbacked, but an agent-behaviour eval has no symbol to tag.
        ("TC-003", "Eval", "FR-001-AC-2", "✅"),
        // Same, by inspection.
        ("TC-004", "Inspection", "FR-001-AC-2", "✅"),
    ] {
        matrix.push_str(&format!("| {tc} | {ty} | {traces} | {status} |\n"));
    }
    write(&bundle.scope, "tests.md", &matrix);
    bundle
}

#[trace("TC-805", "FR-050-AC-16")]
// a row whose declared `Type` names a method that mints
// no source symbol is reported as a no-symbol row rather than a status lie —
// and only when the module declares the vocabulary.
#[test]
fn tc805_no_source_symbol_rows_are_not_status_lies() {
    let bundle = typed_matrix_bundle("805");

    let report = report_for(&bundle, "no-symbol").expect("model declared");
    let lies: Vec<&str> = report
        .status_lies
        .iter()
        .filter_map(|l| l.row_id.as_deref())
        .collect();
    let exempt: Vec<&str> = report
        .no_symbol_rows
        .iter()
        .filter_map(|r| r.row_id.as_deref())
        .collect();

    // The unbacked row whose method could have produced a symbol is still a lie.
    assert_eq!(lies, ["TC-002"], "lies: {lies:?}");
    // The two whose declared method cannot are explained instead.
    assert_eq!(exempt, ["TC-003", "TC-004"], "exempt: {exempt:?}");
    assert_eq!(report.no_symbol_rows[0].test_type, "Eval");
    assert_eq!(report.no_symbol_rows[1].test_type, "Inspection");

    // Exemption changes the verdict, never the facts: every unbacked row is
    // still listed as unbacked, and the counts are untouched.
    let unbacked: Vec<&str> = report
        .unbacked_rows
        .iter()
        .filter_map(|r| r.row_id.as_deref())
        .collect();
    for id in ["TC-002", "TC-003", "TC-004"] {
        assert!(unbacked.contains(&id), "unbacked: {unbacked:?}");
    }
    assert!(!unbacked.contains(&"TC-001"));
}

#[trace("TC-805", "FR-050-AC-16", "FR-050-AC-7")]
// a module declaring no `no_source_symbol`
// vocabulary reports exactly as before — the same rows are lies, and the report
// serializes without the new key.
#[test]
fn tc805_undeclared_vocabulary_changes_nothing() {
    let bundle = typed_matrix_bundle("805-undeclared");
    let report = report_for(&bundle, "iso").expect("model declared");

    let lies: Vec<&str> = report
        .status_lies
        .iter()
        .filter_map(|l| l.row_id.as_deref())
        .collect();
    assert!(
        lies.contains(&"TC-003") && lies.contains(&"TC-004"),
        "without the declaration the eval rows are ordinary lies: {lies:?}"
    );
    assert!(report.no_symbol_rows.is_empty());
    // Absent KEY, not absent substring (CR-094): the FR-063 envelope names
    // `coverage.no_symbol_rows` precisely so an undeclared vocabulary reads as
    // "not computed" rather than as "none found" — the #226 ambiguity.
    let value: serde_json::Value = serde_json::from_str(&report.to_json()).expect("parses");
    assert!(
        !value
            .as_object()
            .expect("object")
            .contains_key("no_symbol_rows"),
        "an undeclared vocabulary must leave the row list absent"
    );
    let metric = value["metrics"]
        .as_array()
        .expect("metrics")
        .iter()
        .find(|m| m["name"] == "coverage.no_symbol_rows")
        .expect("named whether or not it ran");
    assert_eq!(metric["state"], "not_computed");
    assert!(
        metric["because"]
            .as_str()
            .expect("a reason")
            .contains("no_source_symbol"),
        "the condition is named in the engine's own vocabulary: {metric}"
    );
}

#[trace("TC-818", "FR-050-AC-18")]
// coverage parses bodies only for the (CR-049)
// archetypes its declared model names. Selection is decided on the header
// tier (frontmatter `type`), never by filename, before any body touch —
// the declaration the engine used to discard now bounds what is parsed.
#[test]
fn tc818_coverage_parses_only_declared_archetype_bodies() {
    let bundle = iso_bundle("818", &[("TC-001", "FR-001-AC-1", "✅")], &["TC-001"]);
    // Documents whose archetypes no trace target, document reference, or
    // grammar binding names: real corpus members, never body-parsed here.
    write(
        &bundle.scope,
        "ADR-001.md",
        "---\nid: ADR-001\ntype: ADR\n---\n# adr\n\n## Context\n\nwords.\n",
    );
    write(
        &bundle.scope,
        "NOTE-001.md",
        "---\nid: NOTE-001\ntype: Note\n---\n# note\n\n## Body\n\nwords.\n",
    );
    // The falsifier (CR-054): an undeclared type in a file NAMED like a
    // declared one. With the undeclared documents also named unlike the
    // declared ones, a filename-driven engine passed this test — the
    // fixture could not tell "decided on the header tier" from "decided by
    // the FR-* filename", which is the claim FR-050-AC-18 actually makes.
    write(
        &bundle.scope,
        "FR-002.md",
        "---\nid: FR-002\ntype: Note\n---\n# note\n\n## Acceptance Criteria\n\n| ID | Criteria |\n|----|----------|\n| FR-002-AC-1 | never minted |\n",
    );

    let registry = Registry::load_module(&fixture_module("iso")).expect("load module");
    let spec = Spec::from_path(&bundle.scope);
    let extraction = extract_tree(&bundle.source);
    let model = registry.traceability().cloned().unwrap_or_default();
    let graph = trace::bind(&extraction, &model);
    let report = compute(&spec, &registry, &graph, &bundle.scope).expect("report");
    assert!(report.totals.total > 0, "the declared model must mint");

    // Declared archetype: the FR's body was read — its AC table minted ids.
    assert!(
        spec.by_id("FR-001").expect("FR-001").body_is_parsed(),
        "the declared archetype's body must have been parsed"
    );
    // Undeclared archetypes: bodies stay unparsed through the whole rollup —
    // including the one whose FILENAME matches the declared archetype's.
    for id in ["ADR-001", "NOTE-001", "FR-002"] {
        assert!(
            !spec.by_id(id).expect(id).body_is_parsed(),
            "{id} is not declared by the model; its body was parsed during coverage"
        );
    }
    // And nothing it holds reached the report: a filename-driven engine would
    // have minted its AC row.
    assert!(
        !report.to_json().contains("FR-002"),
        "an undeclared type in an FR-named file must mint nothing: {}",
        report.to_json()
    );
}

#[trace("TC-822", "FR-050-AC-19")]
// a model that loads and (CR-054, amended CR-059)
// selects nothing is reported, not silently accepted. Both shapes reach the
// report: an archetype no document has (a typo in the declaration), and a
// declared auxiliary document that did not resolve to rows — the CR-045
// silent-un-minting class, where the ids simply vanish and the only symptom is
// a distant count. The fixture's document is absent, and this model mints
// nothing, which is when an absence is the thing to check first.
#[test]
fn tc822_declarations_that_select_nothing_are_reported() {
    let bundle = iso_bundle(
        "822-selects-nothing",
        &[("TC-001", "FR-001-AC-1", "✅")],
        &[],
    );
    let report = report_for(&bundle, "fails-open").expect("model declared");

    assert_eq!(
        report.totals.total, 0,
        "the fixture's whole point is that it mints nothing"
    );

    let reasons: Vec<&str> = report
        .diagnostics
        .iter()
        .map(|d| d.reason.as_str())
        .collect();
    assert!(
        reasons.contains(&"archetype-matches-nothing"),
        "an archetype no document has must be reported: {:?}",
        report.diagnostics
    );
    // CR-062: `archetype-matches-nothing` is the only scan diagnostic. The
    // `absent-declared-document` and `unreadable-declared-document` reasons went
    // with the `document:` form — there is no declared path left to be absent,
    // and a minting document that cannot be *read* is now the walk's
    // `DocumentUnreadable`, which is strictly better than the silent `None` the
    // off-corpus reader returned.
    // TWO, not three, since CR-135. Every declaration in this fixture selects
    // nothing, but only a TRACE TARGET is reported: a reference declaration
    // reads an existing column rather than minting ids, and having no document
    // of its archetype is ordinary rather than a finding — the same `mints`
    // distinction that keeps `section-matches-nothing` off healthy
    // repositories (#304, FR-050-AC-36).
    assert_eq!(
        reasons,
        vec!["archetype-matches-nothing", "archetype-matches-nothing"],
        "every declared TARGET selects nothing, each for the same reason: {:?}",
        report.diagnostics
    );

    let archetype = report
        .diagnostics
        .iter()
        .find(|d| d.declaration == "acceptance-criterion")
        .expect("present");
    assert!(
        archetype.message.contains("FRR"),
        "names the archetype nothing has: {}",
        archetype.message
    );

    let target = report
        .diagnostics
        .iter()
        .find(|d| d.declaration == "test-case")
        .expect("present");
    assert!(
        target.message.contains("TestMatrixx"),
        "a misspelled matrix archetype is reported the same way: {}",
        target.message
    );
    assert!(
        target
            .path
            .as_deref()
            .is_some_and(|path| path.ends_with("fails-open/manifest.yaml")),
        "a declaration-level fault points to its manifest declaration: {target:?}"
    );
    assert_eq!(target.line, Some(36));

    // Order is a property of the model, not of the walk (NFR-006).
    let again = report_for(&bundle, "fails-open").expect("model declared");
    assert_eq!(report.diagnostics, again.diagnostics);
}

#[test]
fn tc822_model_that_mints_nothing_points_to_traceability_declaration() {
    let module = tmpdir("822-model-origin-module");
    fs::write(
        module.join("manifest.yaml"),
        "name: no-targets\narchetypes:\n- name: FR\ntraceability:\n  status:\n    column: Status\n    complete: [done]\n    pending: [pending]\n    failed: [failed]\n",
    )
    .expect("write manifest");
    let bundle = iso_bundle("822-model-origin", &[], &[]);
    let registry = Registry::load_module(&module).expect("load module");
    let spec = Spec::from_path(&bundle.scope);
    let extraction = extract_tree(&bundle.source);
    let model = registry.traceability().cloned().expect("declared model");
    let graph = trace::bind(&extraction, &model);
    let report = compute(&spec, &registry, &graph, &bundle.scope).expect("report");

    let finding = report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.reason == "model-mints-nothing")
        .expect("model diagnostic");
    assert!(finding
        .path
        .as_deref()
        .is_some_and(|path| path.ends_with("manifest.yaml")));
    assert_eq!(finding.line, Some(4));

    fs::remove_dir_all(module).ok();
    fs::remove_dir_all(bundle.scope.parent().expect("bundle root")).ok();
}

#[trace("TC-822", "FR-050-AC-19")]
// and a model whose declarations all select (CR-054)
// something reports no diagnostics at all — the key is absent from the JSON,
// so FR-050-AC-7 byte-identity holds for every repo without the defect.
//
// AMENDED BY CR-135 (#304). "All select something" is a stronger condition
// than it used to be: the model-wide gate that hid `archetype-matches-nothing`
// as soon as ANY declaration minted is gone, so a declared trace target with no
// document of its archetype is now reported even in an otherwise healthy
// bundle. This fixture declares `test-case-document` over archetype `TC` and
// has no TC document, which is precisely that case — and precisely the fact
// the gate used to hide. The byte-identity claim is unchanged and is asserted
// on a bundle where every declared target really does select.
#[test]
fn tc822_a_healthy_model_reports_no_diagnostics_and_no_key() {
    let bundle = iso_bundle(
        "822-healthy",
        &[("TC-001", "FR-001-AC-1", "✅")],
        &["TC-001"],
    );
    let report = report_for(&bundle, "iso").expect("model declared");

    let unexpected: Vec<&quire_rs::coverage::CoverageDiagnostic> = report
        .diagnostics
        .iter()
        .filter(|d| d.declaration != "test-case-document")
        .collect();
    assert!(
        unexpected.is_empty(),
        "a model selecting normally reports nothing beyond the declared target \
         this bundle has no document for: {unexpected:?}"
    );
    assert!(
        report
            .diagnostics
            .iter()
            .all(|d| d.reason == "archetype-matches-nothing"),
        "and the one it does report is that, not something else: {:?}",
        report.diagnostics
    );
    // THE OMIT-WHEN-EMPTY CONTRACT, asserted as a contract rather than through
    // whichever fixture happens to produce an empty list. Since CR-135 the
    // `iso` model reports on `test-case-document` — archetype `TC`, which this
    // bundle has no document of — so this report is legitimately non-empty, and
    // reaching for a different model to get an empty one only trades this
    // diagnostic for `no-symbol-bound` and `hollow-denominator` from a model
    // that declares no trace tags. The serialization rule is what FR-050-AC-7
    // byte-identity rests on, so it is asserted on the serializer directly.
    let mut empty = report.clone();
    empty.diagnostics.clear();
    assert!(
        !empty.to_json().contains("diagnostics"),
        "an empty diagnostics list must leave the JSON byte-identical"
    );
}

#[trace("TC-822", "FR-050-AC-19")]
// excluding every document of a declared (CR-054)
// archetype is a deliberate act, not a missing archetype — the count that
// decides is taken before `exclude` applies.
#[test]
fn tc822_excluding_every_match_is_not_a_missing_archetype() {
    let bundle = iso_bundle(
        "822-excluded",
        &[("TC-001", "FR-001-AC-1", "✅")],
        &["TC-001"],
    );
    let report = report_for(&bundle, "scoped").expect("model declared");

    assert!(
        !report
            .diagnostics
            .iter()
            .any(|d| d.reason == "archetype-matches-nothing"),
        "an exclusion is not a typo: {:?}",
        report.diagnostics
    );
}

/// Rewrite the bundle's source tree with `total` test functions, the first
/// `readable` of them carrying the declared `#[trace(...)]` marker and the rest
/// carrying an attribute no declared form matches.
///
/// This is the `agent-ix/filament-ide-rs` shape in miniature: real tests, real
/// tags, a marker spelling the module never declared.
fn rewrite_source(bundle: &Bundle, total: usize, readable: usize) {
    let mut lib = String::from("//! Fixture source tree.\n\n#[cfg(test)]\nmod tests {\n");
    for idx in 0..total {
        let attribute = if idx < readable { "trace" } else { "tracks" };
        lib.push_str(&format!(
            "    #[{attribute}(\"TC-{:03}\")]\n    #[test]\n    fn covers_{idx}() {{\n        \
             let _ = 1;\n    }}\n",
            idx + 1
        ));
    }
    lib.push_str("}\n");
    let path = bundle.source.join("lib.rs");
    fs::write(path, lib).expect("write");
}

fn rewrite_mixed_self_named_source(bundle: &Bundle) {
    let path = bundle.source.join("lib.rs");
    fs::write(
        path,
        concat!(
            "//! Fixture source tree.\n\n#[cfg(test)]\nmod tests {\n",
            "    // TC-001\n    #[test]\n    fn tc_001_first() {}\n\n",
            "    // TC-002\n    #[test]\n    fn tc_002_second() {}\n\n",
            "    // TC-003\n    #[test]\n    fn tc_003_third() {}\n",
            "}\n",
        ),
    )
    .expect("write mixed self-name source");
}

fn census_for<'r>(report: &'r CoverageReport, language: &str) -> &'r BindingCensus {
    report
        .binding_census
        .iter()
        .find(|c| c.language == language)
        .unwrap_or_else(|| panic!("no {language} census in {:?}", report.binding_census))
}

#[trace("TC-1060", "FR-051-AC-19", "FR-063-AC-8")]
#[test]
fn tc1060_tagged_separates_authoring_absence_from_an_unread_tag() {
    let bundle = iso_bundle("1060", &[("TC-001", "FR-001-AC-1", "✅")], &[]);

    fs::write(
        bundle.source.join("lib.rs"),
        "#[cfg(test)]\nmod tests {\n    #[test]\n    fn untagged_one() {\n        let _fixture_data = \"TC-999\";\n    }\n\n    #[test]\n    fn untagged_two() {}\n}\n",
    )
    .expect("write untagged source");
    let absent = report_for(&bundle, "iso").expect("model declared");
    let census = census_for(&absent, "rust");
    assert_eq!((census.bound, census.tagged, census.candidates), (0, 0, 2));
    assert!(census.unmatched_example.is_none());
    let metric = metric_for(&absent, "authoring.tag_rate");
    assert_eq!(
        metric.measurement,
        Measurement::Measured {
            value: 0,
            population: 2,
            examined: 2,
            matched: 2,
        },
        "zero tags is an observed authoring state, not an unread denominator"
    );
    assert!(!metric.is_hollow());

    rewrite_source(&bundle, 3, 0);
    let unread = report_for(&bundle, "iso").expect("model declared");
    let census = census_for(&unread, "rust");
    assert_eq!((census.bound, census.tagged, census.candidates), (0, 3, 3));
    let example = census
        .unmatched_example
        .as_ref()
        .expect("an unread authored tag has a locus");
    assert_eq!((example.path.as_str(), example.line), ("lib.rs", 5));

    rewrite_source(&bundle, 3, 1);
    let mixed = report_for(&bundle, "iso").expect("model declared");
    let census = census_for(&mixed, "rust");
    assert_eq!((census.bound, census.tagged, census.candidates), (1, 3, 3));
    assert!(census.bound <= census.tagged && census.tagged <= census.candidates);
}

#[trace("TC-983", "FR-050-AC-27")]
// a language whose evidence symbols all fail to (CR-093)
// bind is reported as a diagnostic naming the counts and the declared forms,
// never as a low percentage indistinguishable from missing tests.
#[test]
fn tc983_a_language_that_binds_nothing_is_a_diagnostic() {
    let bundle = iso_bundle(
        "983",
        &[
            ("TC-001", "FR-001-AC-1", "✅"),
            ("TC-002", "FR-001-AC-2", "✅"),
        ],
        &[],
    );
    rewrite_source(&bundle, 3, 0);
    let report = report_for(&bundle, "iso").expect("model declared");

    // The premise, stated: three symbols were examined and none bound.
    let census = census_for(&report, "rust");
    assert_eq!(census.candidates, 3);
    assert_eq!(census.bound, 0);
    assert_eq!(
        census.forms,
        vec![
            "rust-trace-attribute",
            "trace-line",
            "comment-id",
            "test-name-id"
        ],
        "markers first, then legacy — the order the binder tries them in"
    );

    let finding = report
        .diagnostics
        .iter()
        .find(|d| d.reason == "no-symbol-bound")
        .unwrap_or_else(|| panic!("no binding diagnostic in {:?}", report.diagnostics));
    assert_eq!(finding.declaration, "traceability.trace_tags");
    assert_eq!(finding.value.as_deref(), Some("rust"));
    assert_eq!(finding.path.as_deref(), Some("lib.rs"));
    assert_eq!(finding.line, Some(5));
    assert!(
        finding.message.contains('3') && finding.message.contains("rust-trace-attribute"),
        "the message carries the count and the forms to check: {}",
        finding.message
    );

    // Same tree, same tests, the declared spelling: the diagnostic goes away
    // and the census becomes the reassurance the payload could not give before.
    rewrite_source(&bundle, 3, 3);
    let healthy = report_for(&bundle, "iso").expect("model declared");
    let census = census_for(&healthy, "rust");
    assert_eq!((census.candidates, census.bound), (3, 3));
    assert!(
        !healthy
            .diagnostics
            .iter()
            .any(|d| d.reason == "no-symbol-bound" || d.reason == "low-symbol-binding"),
        "a language that reads cleanly reports nothing: {:?}",
        healthy.diagnostics
    );
    // Reported whether or not it holds — that is what makes it a premise
    // rather than a defect list.
    assert!(healthy.to_json().contains("binding_census"));
}

#[trace("TC-1083", "FR-050-AC-44")]
#[test]
fn tc1083_mixed_comment_bindings_do_not_mask_a_broken_name_form() {
    let bundle = iso_bundle(
        "1080",
        &[
            ("TC-001", "FR-001-AC-1", "✅"),
            ("TC-002", "FR-001-AC-2", "✅"),
            ("TC-003", "FR-001-AC-3", "✅"),
        ],
        &[],
    );
    rewrite_mixed_self_named_source(&bundle);
    let report = report_for(&bundle, "iso").expect("model declared");
    let census = census_for(&report, "rust");
    assert_eq!((census.candidates, census.bound), (3, 3));
    assert_eq!((census.self_named, census.self_named_bound), (3, 0));
    assert!(census.self_named_unbound_example.is_some());
    assert!(report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.reason == "marker-form-mismatch"));
    let hollow = report
        .diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic.reason == "hollow-denominator"
                && diagnostic.value.as_deref() == Some("coverage.self_named_binding.rust")
        })
        .expect("self-name subpopulation is a named hollow ratio");
    assert!(
        hollow.path.is_none(),
        "aggregate metric has no invented locus"
    );
}

#[trace("TC-984", "FR-050-AC-27")]
// Below the observation boundary (MP-201), the engine reports both counts and
// the ambiguity rather than diagnosing sparse tagging or a pattern mismatch.
#[test]
fn tc984_binding_below_the_floor_is_reported_with_both_counts() {
    let bundle = iso_bundle("984", &[("TC-001", "FR-001-AC-1", "✅")], &[]);

    // 1 of 21 = 4.8%, under the 5% floor.
    rewrite_source(&bundle, 21, 1);
    let report = report_for(&bundle, "iso").expect("model declared");
    let census = census_for(&report, "rust");
    assert_eq!((census.candidates, census.bound), (21, 1));

    let finding = report
        .diagnostics
        .iter()
        .find(|d| d.reason == "low-symbol-binding")
        .unwrap_or_else(|| panic!("no floor diagnostic in {:?}", report.diagnostics));
    assert_eq!(finding.value.as_deref(), Some("rust"));
    assert_eq!(finding.path.as_deref(), Some("lib.rs"));
    assert_eq!(finding.line, Some(10));
    assert!(
        finding.message.contains("1 of 21"),
        "both counts, so the reader judges rather than the engine: {}",
        finding.message
    );
    assert!(
        finding
            .message
            .contains("cannot distinguish sparse tagging"),
        "the observation must retain uncertainty: {}",
        finding.message
    );
    assert!(
        !report
            .diagnostics
            .iter()
            .any(|d| d.reason == "no-symbol-bound"),
        "something bound, so this is not the zero case"
    );

    // 2 of 21 = 9.5%, over the floor: an untagged tail is not a finding.
    rewrite_source(&bundle, 21, 2);
    let over = report_for(&bundle, "iso").expect("model declared");
    assert_eq!(census_for(&over, "rust").bound, 2);
    assert!(
        !over
            .diagnostics
            .iter()
            .any(|d| d.reason == "low-symbol-binding"),
        "over the floor is silence: {:?}",
        over.diagnostics
    );
}

fn metric_for<'r>(report: &'r CoverageReport, name: &str) -> &'r Metric {
    report
        .metrics
        .iter()
        .find(|m| m.name == name)
        .unwrap_or_else(|| panic!("no `{name}` metric in {:?}", report.metrics))
}

#[trace("TC-988", "FR-063-AC-3", "FR-063-AC-4", "FR-063-AC-5")]
// every headline number the coverage payload (CR-094)
// emits carries its unit, population, examined and matched counts, and a ratio
// computed over input the measurement could not read is a diagnostic.
#[test]
fn tc988_coverage_metrics_carry_provenance_and_flag_a_hollow_denominator() {
    let bundle = iso_bundle(
        "988",
        &[
            ("TC-001", "FR-001-AC-1", "✅"),
            ("TC-002", "FR-001-AC-2", "✅"),
        ],
        &[],
    );

    // ── Hollow: real tests, real tags, a marker spelling nothing declares ──
    rewrite_source(&bundle, 3, 0);
    let report = report_for(&bundle, "iso").expect("model declared");

    let backed = metric_for(&report, "coverage.backed");
    assert_eq!(backed.unit, "matrix row");
    assert!(
        !backed.method.is_empty(),
        "a metric states how it was taken"
    );
    match backed.measurement {
        Measurement::Measured {
            population,
            examined,
            matched,
            ..
        } => {
            assert!(population > 0, "there are rows to be a ratio over");
            assert_eq!(examined, 3, "three evidence symbols were walked");
            assert_eq!(matched, 0, "and none of them was read");
        }
        Measurement::NotComputed { .. } => panic!("backed/total is always computed"),
    }
    assert!(backed.is_hollow());

    let finding = report
        .diagnostics
        .iter()
        .find(|d| d.reason == "hollow-denominator")
        .unwrap_or_else(|| panic!("no hollow finding in {:?}", report.diagnostics));
    assert_eq!(finding.value.as_deref(), Some("coverage.backed"));
    assert_eq!(finding.declaration, "metrics");

    // FR-063-AC-4: `implements` draws its denominator from the production
    // symbols examined, so the relation count is never a bare number. The iso
    // module declares no `implements` forms, so this is the #226 fold: the
    // metric is named, states it did not run, and carries no zero.
    let implements = metric_for(&report, "coverage.implements");
    match &implements.measurement {
        Measurement::NotComputed { because } => {
            assert!(
                because.contains("implements"),
                "the condition is named, not 'no data': {because}"
            );
        }
        Measurement::Measured { .. } => panic!("the iso module declares no implements forms"),
    }
    assert_eq!(implements.value(), None, "not computed is not a zero");

    // ── Read cleanly: the same tree, the declared spelling ──
    rewrite_source(&bundle, 3, 3);
    let healthy = report_for(&bundle, "iso").expect("model declared");
    let backed = metric_for(&healthy, "coverage.backed");
    assert!(!backed.is_hollow());
    assert!(
        !healthy
            .diagnostics
            .iter()
            .any(|d| d.reason == "hollow-denominator"),
        "a measurement that read its input reports nothing: {:?}",
        healthy.diagnostics
    );

    // ── Nothing to read: a repository with no tests is 0%, honestly ──
    // The distinction `examined` exists for. Without it this fires on every
    // greenfield corpus, which would make the check worthless.
    fs::write(bundle.source.join("lib.rs"), "//! No symbols at all.\n").expect("write");
    let greenfield = report_for(&bundle, "iso").expect("model declared");
    let backed = metric_for(&greenfield, "coverage.backed");
    match backed.measurement {
        Measurement::Measured {
            examined, matched, ..
        } => assert_eq!((examined, matched), (0, 0)),
        Measurement::NotComputed { .. } => panic!("backed/total is always computed"),
    }
    assert!(!backed.is_hollow(), "nothing offered is not a hollow read");
    assert!(
        !greenfield
            .diagnostics
            .iter()
            .any(|d| d.reason == "hollow-denominator"),
        "an honest zero is not a finding: {:?}",
        greenfield.diagnostics
    );
}

#[trace("TC-989", "FR-050-AC-28", "FR-052-AC-18")]
// the catch-all is split out of the headline, and (CR-095)
// the span-grounding rate is reported per shape — the two facts a bare
// `extractable (54%)` hid.
#[test]
fn tc989_the_properties_headline_separates_the_catch_all() {
    let bundle = iso_bundle("989", &[("TC-001", "FR-001-AC-1", "✅")], &["TC-001"]);

    // Two criteria a generator can quantify over. Neither names a property to
    // write: `universal` is the catch-all, and `example` is one scenario.
    rewrite_criteria(
        &bundle,
        &[
            "For every configuration, the parser shall accept the document.",
            "Given a config of 3 lines, the parser shall accept it.",
        ],
    );
    let report = report_for(&bundle, "iso").expect("model declared");
    let criteria = report.totals.criteria.expect("criteria bound");
    assert_eq!(
        report.totals.specific_shaped,
        Some(0),
        "a corpus of catch-alls says so: {:?}",
        report.criteria
    );
    assert!(
        report.totals.property_shaped.expect("shaped") > 0,
        "and `extractable` is still the larger, still-true number"
    );

    // Both figures reach the envelope under their own names, so a reader who
    // repeats the first can find the second. This is the whole failure:
    // 54% travels and 8% does not.
    let shaped = metric_for(&report, "coverage.property_shaped");
    let specific = metric_for(&report, "coverage.specific_shaped");
    assert_eq!(specific.value(), Some(0));
    assert_eq!(specific.unit, shaped.unit);
    match specific.measurement {
        Measurement::Measured { population, .. } => assert_eq!(
            population, criteria as u64,
            "both are ratios over the same denominator, so they are comparable"
        ),
        Measurement::NotComputed { .. } => panic!("criteria were classified"),
    }
    assert!(
        specific.method.contains("universal"),
        "the method names what it excludes: {}",
        specific.method
    );

    // Grounding is reported per shape, so "which shapes arrive usable" is
    // readable without a bespoke sweep.
    let doc = report
        .criteria
        .iter()
        .find(|c| !c.grounding.is_empty())
        .expect("a document with classified criteria");
    let records: usize = doc.grounding.values().map(|g| g.records).sum();
    assert_eq!(
        records, doc.criteria,
        "every classified criterion is counted exactly once"
    );
    for (shape, counts) in &doc.grounding {
        assert!(counts.domain <= counts.records, "{shape}");
        assert!(counts.precondition <= counts.records, "{shape}");
        assert!(counts.oracle <= counts.records, "{shape}");
        assert!(
            counts.all_three <= counts.domain.min(counts.precondition).min(counts.oracle),
            "{shape}: all-three cannot exceed any of its parts"
        );
    }
    // `example` is not-extractable by construction, so its records carry no
    // spans — the one shape whose zero is correct rather than a finding.
    if let Some(example) = doc.grounding.get("example") {
        assert_eq!(example.all_three, 0);
    }
}

#[trace("TC-1001", "FR-064-AC-4", "FR-064-CON-1")]
// suspicions reach the report, ordered and (CR-100)
// evidenced, and change no number and no exit.
#[test]
fn tc1001_suspicions_reach_the_report_and_move_nothing() {
    let bundle = iso_bundle("1001", &[("TC-001", "FR-001-AC-1", "✅")], &[]);
    // Two tests whose only assertion sits behind a narrowing guard — the shape
    // measured green while checking 2.3% of its samples.
    fs::write(
        bundle.source.join("lib.rs"),
        concat!(
            "//! fixture\n\n#[cfg(test)]\nmod tests {\n",
            "    #[trace(\"TC-001\")]\n    #[test]\n",
            "    fn covers_0() {\n",
            "        if let Some(v) = parse() {\n",
            "            assert_eq!(v, 1);\n",
            "        }\n    }\n",
            "    #[trace(\"TC-002\")]\n    #[test]\n",
            "    fn covers_1() {\n",
            "        if let Some(v) = parse() {\n",
            "            assert_eq!(v, 2);\n",
            "        }\n    }\n}\n",
        ),
    )
    .expect("write");

    let report = report_for(&bundle, "iso").expect("model declared");
    assert_eq!(report.suspicions.len(), 2, "{:#?}", report.suspicions);
    for s in &report.suspicions {
        assert_eq!(s.kind, "vacuous-under-guard");
        assert!(
            !s.evidence.is_empty(),
            "a suspicion carries its measurement"
        );
        assert!(s.line > 0);
    }
    // Deterministically ordered by (path, line, symbol).
    let lines: Vec<usize> = report.suspicions.iter().map(|s| s.line).collect();
    let mut sorted = lines.clone();
    sorted.sort_unstable();
    assert_eq!(lines, sorted);

    // CON-1: advisory. It moves no total and adds no diagnostic.
    let before = (
        report.totals.backed,
        report.totals.total,
        report.diagnostics.len(),
    );
    fs::write(
        bundle.source.join("lib.rs"),
        concat!(
            "//! fixture\n\n#[cfg(test)]\nmod tests {\n",
            "    #[trace(\"TC-001\")]\n    #[test]\n",
            "    fn covers_0() {\n        assert_eq!(parse(), 1);\n    }\n",
            "    #[trace(\"TC-002\")]\n    #[test]\n",
            "    fn covers_1() {\n        assert_eq!(parse(), 2);\n    }\n}\n",
        ),
    )
    .expect("write");
    let clean = report_for(&bundle, "iso").expect("model declared");
    assert!(clean.suspicions.is_empty(), "{:#?}", clean.suspicions);
    assert_eq!(
        (
            clean.totals.backed,
            clean.totals.total,
            clean.diagnostics.len()
        ),
        before,
        "a suspicion is advisory: removing them changes no other number"
    );
    // Absent from the JSON when there are none — asserted as a missing KEY,
    // not as an absent substring (CR-102). The substring form answers a
    // different question now that the metric envelope NAMES its keys: any
    // future field or method sentence containing the word would fail it, and a
    // `suspicions` key nested anywhere would pass it.
    let payload: serde_json::Value = serde_json::from_str(&clean.to_json()).expect("valid json");
    assert!(
        payload
            .as_object()
            .expect("payload is an object")
            .get("suspicions")
            .is_none(),
        "no suspicions means no key, not an empty list: {payload:#}"
    );
}

// ─── CR-117 / #270: the two minting faults the payload could not name ───────

/// Rewrite the bundle's matrix with `heading` as its section and `id_column` as
/// its first column, keeping the rows [`iso_bundle`] writes.
///
/// The two single-cell edits behind the ecosystem's unreached-declaration census
/// (a candidate 3,514 TC ids; CR-118 measured the section fix at +83 rows, the
/// population being confounded with id-column mismatch — #318). The
/// `iso` fixture module declares `section: Test Cases` and `id_column: ID`, so
/// passing anything else here is the defect and passing those two is the
/// control.
fn rewrite_matrix(bundle: &Bundle, heading: &str, id_column: &str, rows: &[(&str, &str, &str)]) {
    rewrite_matrix_with_status(bundle, heading, id_column, "Status", rows);
}

fn rewrite_matrix_with_status(
    bundle: &Bundle,
    heading: &str,
    id_column: &str,
    status_column: &str,
    rows: &[(&str, &str, &str)],
) {
    let mut md = format!(
        "---\nid: TM-001\ntype: TestMatrix\ntitle: Test Matrix\n---\n\n\
         # Test Matrix\n\n## Overview\n\nProse.\n\n## {heading}\n\n\
         | {id_column} | Traces To | {status_column} |\n|----|-----------|--------|\n"
    );
    for (tc, traces_to, status) in rows {
        md.push_str(&format!("| {tc} | {traces_to} | {status} |\n"));
    }
    write(&bundle.scope, "tests.md", &md);
}

#[trace("TC-1079", "FR-050-AC-43")]
// A status-shaped near miss used to make status classification silently skip
// every row. The diagnostic names the configured and observed columns at the
// table header; restoring the configured header restores the status lie and
// leaves a valid matrix quiet (#341).
#[test]
fn tc1079_a_status_column_near_miss_is_actionable_and_the_control_is_quiet() {
    let bundle = iso_bundle(
        "1079",
        &[("TC-002", "FR-001-AC-2", "✅")],
        // Deliberately unbound: with a readable status this is a status lie.
        &[],
    );
    rewrite_matrix_with_status(
        &bundle,
        "Test Cases",
        "ID",
        "Coverage Status",
        &[("TC-002", "FR-001-AC-2", "✅")],
    );

    let skipped = report_for(&bundle, "iso").expect("model declared");
    assert!(
        skipped.status_lies.is_empty(),
        "the fixture pins the former silent skip: {:?}",
        skipped.status_lies
    );
    let finding = diagnostic_for(&skipped, "status-column-matches-nothing");
    assert_eq!(finding.declaration, "traces-to");
    assert_eq!(finding.path.as_deref(), Some("tests.md"));
    assert_eq!(finding.line, Some(15), "the repair locus is the header");
    for detail in [
        "'Status'",
        "'Coverage Status'",
        "'Test Cases'",
        "tests.md",
        "traceability.status.column",
        "rename the document column",
        "will not guess",
    ] {
        assert!(
            finding.message.contains(detail),
            "message names `{detail}`: {}",
            finding.message
        );
    }

    rewrite_matrix(
        &bundle,
        "Test Cases",
        "ID",
        &[("TC-002", "FR-001-AC-2", "✅")],
    );
    let healthy = report_for(&bundle, "iso").expect("model declared");
    assert!(
        !healthy
            .diagnostics
            .iter()
            .any(|d| d.reason == "status-column-matches-nothing"),
        "the configured column remains quiet: {:?}",
        healthy.diagnostics
    );
    assert_eq!(
        healthy
            .status_lies
            .iter()
            .filter_map(|lie| lie.row_id.as_deref())
            .collect::<Vec<_>>(),
        vec!["TC-002"],
        "the control proves status classification resumed"
    );

    // A module may keep a different model-wide status header for sibling
    // tables while naming the schema-required header on this reference. This
    // is an explicit declaration, not the engine guessing from a near match.
    rewrite_matrix_with_status(
        &bundle,
        "Test Cases",
        "ID",
        "Coverage Status",
        &[("TC-002", "FR-001-AC-2", "✅")],
    );
    let module = tmpdir("1079-status-override");
    let manifest = fs::read_to_string(fixture_module("iso").join("manifest.yaml"))
        .expect("read fixture module")
        .replace(
            "    row_id_column: ID\n    pattern: '((?:StR|US|FR|NFR)-",
            "    row_id_column: ID\n    status_column: Coverage Status\n    pattern: '((?:StR|US|FR|NFR)-",
        );
    assert!(
        manifest.contains("status_column: Coverage Status"),
        "the fixture override was inserted"
    );
    fs::write(module.join("manifest.yaml"), &manifest).expect("write override module");
    let overridden = report_for_module(&bundle, &module).expect("model declared");
    assert!(
        !overridden
            .diagnostics
            .iter()
            .any(|d| d.reason == "status-column-matches-nothing"),
        "an explicitly configured reference header is quiet: {:?}",
        overridden.diagnostics
    );
    assert_eq!(
        overridden
            .status_lies
            .iter()
            .filter_map(|lie| lie.row_id.as_deref())
            .collect::<Vec<_>>(),
        vec!["TC-002"],
        "the per-reference header restores classification without changing the global vocabulary"
    );

    // An override selects a column; the model-wide vocabulary still supplies
    // the values and classes. Without that vocabulary the declaration would
    // be accepted but have no runtime effect, so module loading rejects it.
    let invalid_module = tmpdir("1079-status-override-without-vocabulary");
    let without_vocabulary = manifest.replace(
        "  status:\n    column: Status\n    complete: [\"✅\"]\n    pending: [\"🚧\", \"⚠️\"]\n    failed: [\"❌\"]\n",
        "",
    );
    assert!(
        !without_vocabulary.contains("\n  status:\n"),
        "the invalid control removed the model-wide vocabulary"
    );
    fs::write(invalid_module.join("manifest.yaml"), without_vocabulary)
        .expect("write invalid override module");
    let invalid = Registry::load_module(&invalid_module).expect("load registry");
    assert_eq!(
        invalid.failures().len(),
        1,
        "the invalid declaration must be recorded as a module-load failure"
    );
    let error = &invalid.failures()[0].reason;
    assert!(error.contains("status_column"), "{error}");
    assert!(error.contains("no `status` vocabulary"), "{error}");
}

fn diagnostic_for<'r>(
    report: &'r CoverageReport,
    reason: &str,
) -> &'r quire_rs::coverage::CoverageDiagnostic {
    report
        .diagnostics
        .iter()
        .find(|d| d.reason == reason)
        .unwrap_or_else(|| panic!("no `{reason}` diagnostic in {:?}", report.diagnostics))
}

#[trace("TC-1033", "FR-050-AC-33")]
// a declared section the archetype-matching document (CR-117)
// does not have is reported per document, naming the file, the heading it
// FOUND and the heading it DECLARED — and a document carrying the declared
// heading reports nothing.
#[test]
fn tc1033_a_declared_section_the_document_lacks_is_reported_with_both_names() {
    let bundle = iso_bundle("1033", &[("TC-001", "FR-001-AC-1", "✅")], &["TC-001"]);
    // The archetype still matches; only the heading is wrong. This is what 88
    // of 239 ecosystem repositories look like.
    rewrite_matrix(
        &bundle,
        "Test Case Summary",
        "ID",
        &[("TC-001", "FR-001-AC-1", "✅")],
    );
    let report = report_for(&bundle, "iso").expect("model declared");

    // The silence this closes: the matrix mints nothing at all, and before
    // CR-117 that produced a smaller denominator and no finding.
    assert!(
        !report
            .groups
            .iter()
            .any(|g| g.target == "test-case" && g.total > 0),
        "the whole point of the fixture is that the matrix mints nothing: {:?}",
        report.groups
    );

    let finding = diagnostic_for(&report, "section-matches-nothing");
    assert_eq!(finding.declaration, "test-case");
    // L2: the one file whose one word is wrong.
    assert_eq!(
        finding.path.as_deref(),
        Some("tests.md"),
        "the finding names the document to open: {finding:?}"
    );
    // L3: both values, so the message IS the diff. "the declared section was
    // not found" satisfies neither half.
    assert!(
        finding.message.contains("'Test Case Summary'"),
        "names what was DECLARED: {}",
        finding.message
    );
    assert!(
        finding.message.contains("'Test Cases'"),
        "names what was FOUND: {}",
        finding.message
    );
    assert!(
        finding.message.contains("'Overview'"),
        "every heading the document has, so a near miss is visible: {}",
        finding.message
    );

    // The control: the same tree with the declared heading. Neither token
    // fires — a check that cannot stay silent on healthy input is a constant.
    rewrite_matrix(
        &bundle,
        "Test Cases",
        "ID",
        &[("TC-001", "FR-001-AC-1", "✅")],
    );
    let healthy = report_for(&bundle, "iso").expect("model declared");
    assert!(
        !healthy
            .diagnostics
            .iter()
            .any(|d| d.reason == "section-matches-nothing"
                || d.reason == "id-column-matches-nothing"),
        "a healthy matrix fires neither: {:?}",
        healthy.diagnostics
    );
    assert_eq!(
        healthy.totals.backed, 1,
        "and the row it strands is backed once the heading is right"
    );
}

#[trace("TC-1034", "FR-050-AC-33")]
// the section found and the declared id column (CR-117)
// absent is its OWN token: the two faults produce payloads agreeing in every
// key a reader looks at, so one shared "matched nothing" sends a reader of
// `agent-ix/identity` to the heading that is already correct.
#[test]
fn tc1034_a_declared_id_column_the_table_lacks_is_its_own_token() {
    let bundle = iso_bundle("1034", &[("TC-001", "FR-001-AC-1", "✅")], &["TC-001"]);
    // The heading is RIGHT. Only the id column is wrong, so the table IS read
    // and mints a row whose identity is null.
    rewrite_matrix(
        &bundle,
        "Test Cases",
        "Test Case ID",
        &[("TC-001", "FR-001-AC-1", "✅")],
    );
    let report = report_for(&bundle, "iso").expect("model declared");

    let finding = diagnostic_for(&report, "id-column-matches-nothing");
    assert_eq!(finding.declaration, "test-case");
    assert_eq!(finding.path.as_deref(), Some("tests.md"));
    assert!(
        finding.message.contains("'ID'"),
        "names what was DECLARED: {}",
        finding.message
    );
    assert!(
        finding.message.contains("'Test Case ID'"),
        "names what was FOUND: {}",
        finding.message
    );
    // The section is fine, so the section token must not fire alongside it —
    // otherwise the reader is back to two indistinguishable findings.
    assert!(
        !report
            .diagnostics
            .iter()
            .any(|d| d.reason == "section-matches-nothing"),
        "a heading that matched is not a heading that did not: {:?}",
        report.diagnostics
    );

    // The discrimination claim, asserted rather than argued: the SAME tree
    // with the heading wrong instead reports the other token. Both mint zero.
    rewrite_matrix(
        &bundle,
        "Test Case Summary",
        "ID",
        &[("TC-001", "FR-001-AC-1", "✅")],
    );
    let section = report_for(&bundle, "iso").expect("model declared");
    assert_eq!(
        (report.totals.backed, section.totals.backed),
        (0, 0),
        "the two defects agree on the number everybody reads"
    );
    assert!(
        !section
            .diagnostics
            .iter()
            .any(|d| d.reason == "id-column-matches-nothing"),
        "and disagree on the token, which is the whole ticket: {:?}",
        section.diagnostics
    );
}

#[trace("TC-1035", "FR-050-AC-33")]
// neither token is gated on whether the model (CR-117)
// minted anything ELSE, and the section message names the id column it could
// not check — so a document carrying both faults is fixed in one pass.
#[test]
fn tc1035_the_minting_diagnostics_are_not_gated_on_another_declaration_minting() {
    let bundle = iso_bundle("1035", &[("TC-001", "FR-001-AC-1", "✅")], &["TC-001"]);
    // BOTH faults on one document — the `agent-ix/identity` shape. The FR is
    // untouched and mints its two criteria normally.
    rewrite_matrix(
        &bundle,
        "Test Case Summary",
        "Test Case ID",
        &[("TC-001", "FR-001-AC-1", "✅")],
    );
    let report = report_for(&bundle, "iso").expect("model declared");

    // The model minted: `archetype-matches-nothing` is suppressed for exactly
    // this reason, and these two must not be. A gate shared across
    // declarations lets one healthy declaration hide another's failure
    // (`agent-ix/quire-rs#304`).
    assert!(
        report.totals.total > 0,
        "the FR criteria still mint, which is what makes this the gated case: {:?}",
        report.groups
    );
    let finding = diagnostic_for(&report, "section-matches-nothing");
    assert_eq!(finding.path.as_deref(), Some("tests.md"));

    // The row's thesis: the wrong heading strands the table before the column
    // is read, so the column fault is UNREACHABLE here. A message naming only
    // the heading sends its reader round the loop a second time.
    assert!(
        finding.message.contains("'ID'"),
        "the section finding names the id column it could NOT check: {}",
        finding.message
    );

    // Fixing only the heading exposes the second fault, which is the loop the
    // sentence above is there to shorten.
    rewrite_matrix(
        &bundle,
        "Test Cases",
        "Test Case ID",
        &[("TC-001", "FR-001-AC-1", "✅")],
    );
    let half_fixed = report_for(&bundle, "iso").expect("model declared");
    assert!(
        half_fixed
            .diagnostics
            .iter()
            .any(|d| d.reason == "id-column-matches-nothing"),
        "the second fault was there all along: {:?}",
        half_fixed.diagnostics
    );

    // Order is a property of the model, not of the walk (NFR-006).
    let again = report_for(&bundle, "iso").expect("model declared");
    assert_eq!(half_fixed.diagnostics, again.diagnostics);
}

#[trace("TC-1036", "FR-063-AC-7")]
// `minting.section_hit_rate` is the premise under (CR-117)
// every minting number: the documents whose declared section was found over
// the documents the archetype selected. A RATIO, so reading none of them is
// hollow; a model declaring no trace targets never computes it.
#[test]
fn tc1036_the_section_hit_rate_is_a_ratio_over_the_documents_the_archetype_selected() {
    let bundle = iso_bundle("1036", &[("TC-001", "FR-001-AC-1", "✅")], &["TC-001"]);

    // Healthy: two minting declarations select one document each — the FR and
    // the matrix — and both find their section. (`test-case-document` selects
    // nothing, so it contributes to neither count.)
    let healthy = report_for(&bundle, "iso").expect("model declared");
    let metric = metric_for(&healthy, "minting.section_hit_rate");
    assert_eq!(metric.unit, "declared minting document");
    assert_eq!(metric.shape, quire_rs::metric::MetricShape::Ratio);
    assert_eq!(
        metric.measurement,
        Measurement::Measured {
            value: 2,
            population: 2,
            examined: 2,
            matched: 2,
        },
        "both declared sections were found"
    );
    assert!(!metric.is_hollow(), "everything was read");

    // One of the two headings wrong: 1 of 2, and NOT hollow — something was
    // read, and a judgement about a tail is not this metric's to make.
    rewrite_matrix(
        &bundle,
        "Test Case Summary",
        "ID",
        &[("TC-001", "FR-001-AC-1", "✅")],
    );
    let partial = report_for(&bundle, "iso").expect("model declared");
    let metric = metric_for(&partial, "minting.section_hit_rate");
    assert_eq!(metric.value(), Some(1));
    assert!(!metric.is_hollow());
    assert!(
        !partial
            .diagnostics
            .iter()
            .any(|d| d.reason == "hollow-denominator"
                && d.value.as_deref() == Some("minting.section_hit_rate")),
        "a partial read is reported per document, not as a hollow ratio: {:?}",
        partial.diagnostics
    );

    // BOTH headings wrong: nothing minting was read at all, and the ratio is
    // arithmetic over nothing — which is precisely what FR-063-AC-5 exists to
    // surface, and what a bare `0/0 rows backed` never said.
    write(
        &bundle.scope,
        "FR-001.md",
        "---\nid: FR-001\ntype: FR\ntitle: A requirement\n---\n\n\
         ## Criteria\n\n| ID | Criteria | Verification |\n|----|----------|--------------|\n\
         | FR-001-AC-1 | The system shall do it. | Test (TC-001) |\n",
    );
    let blind = report_for(&bundle, "iso").expect("model declared");
    let metric = metric_for(&blind, "minting.section_hit_rate");
    assert_eq!(
        metric.measurement,
        Measurement::Measured {
            value: 0,
            population: 2,
            examined: 2,
            matched: 0,
        }
    );
    assert!(
        metric.is_hollow(),
        "input was offered, none of it was read, and a ratio was published"
    );
    assert!(
        blind
            .diagnostics
            .iter()
            .any(|d| d.reason == "hollow-denominator"
                && d.value.as_deref() == Some("minting.section_hit_rate")),
        "and the schema invariant reports it by name: {:?}",
        blind.diagnostics
    );

    // A model with no trace targets never looked, and "not computed" is a
    // value rather than a zero dressed as an answer (FR-063-AC-2).
    let unminting = report_for(&bundle, "required-relations").expect("model declared");
    let metric = metric_for(&unminting, "minting.section_hit_rate");
    assert_eq!(
        metric.value(),
        None,
        "nothing selected anything, so there was no section to look for"
    );
    assert!(
        !metric.is_hollow(),
        "a measurement that never ran has no denominator to be hollow"
    );
}

// ─── CR-118 / #272: one heading name, and the rows under the other headings ──

/// A matrix whose four rows sit under four different headings. Three of them
/// are what the `iso-sections` module declares; `Edge Cases` is not, and is the
/// control inside the fixture — a widened declaration reads the sections it
/// names and stops there.
///
/// `TC-002`, under the qualified heading, claims `✅` and no test carries it.
/// That makes it a **status lie**, which is computed off the *reference*
/// declaration rather than the target — so the row is the one assertion that
/// can tell whether the reference was widened alongside the target.
fn matrix_across_headings(bundle: &Bundle) {
    write(
        &bundle.scope,
        "tests.md",
        "---\nid: TM-001\ntype: TestMatrix\ntitle: Test Matrix\n---\n\n\
         # Test Matrix\n\n\
         ## Test Cases\n\n| ID | Traces To | Status |\n|----|-----------|--------|\n\
         | TC-001 | FR-001-AC-1 | ✅ |\n\n\
         ## Test Cases (plugin scope)\n\n| ID | Traces To | Status |\n|----|-----------|--------|\n\
         | TC-002 | FR-001-AC-2 | ✅ |\n\n\
         ## Integration Test Matrix\n\n| ID | Traces To | Status |\n|----|-----------|--------|\n\
         | TC-003 | FR-001-AC-1 | ✅ |\n\n\
         ## Edge Cases\n\n| ID | Traces To | Status |\n|----|-----------|--------|\n\
         | TC-004 | FR-001-AC-2 | ✅ |\n",
    );
}

#[trace("TC-1037", "FR-050-AC-34")]
// a declaration naming SEVERAL sections mints from (CR-118)
// every one of them, in document order, and the same declaration naming ONE
// section still mints from exactly that one — a widening a module did not ask
// for is the failure mode this fixture pair exists to catch.
#[test]
fn tc1037_a_declaration_naming_several_sections_mints_from_all_of_them() {
    let bundle = iso_bundle(
        "1037",
        &[("TC-001", "FR-001-AC-1", "✅")],
        // TC-002 is deliberately untagged — see `matrix_across_headings`.
        &["TC-001", "TC-003", "TC-004"],
    );
    matrix_across_headings(&bundle);

    // The widened declaration: `Test Cases*` reaches the bare heading and the
    // locally-qualified one, `Integration Test Matrix` is named literally, and
    // `Edge Cases` is named by neither.
    let wide = report_for(&bundle, "iso-sections").expect("model declared");
    let minted: Vec<(&str, usize, usize)> = wide
        .groups
        .iter()
        .filter(|g| g.target == "test-case")
        .map(|g| (g.document.as_str(), g.backed, g.total))
        .collect();
    assert_eq!(
        minted,
        vec![("tests.md", 2, 3)],
        "three declared sections, three rows, and the untagged one unbacked: {:?}",
        wide.groups
    );

    // The row under the undeclared heading is the in-fixture control. Its test
    // binds — `TC-004` is tagged — so it can only be missing because no
    // declaration reached its row, and it says so in `untracked_symbols`.
    assert!(
        wide.untracked_symbols
            .iter()
            .any(|u| u.trace_id == "TC-004"),
        "a heading the declaration does not name mints nothing: {:?}",
        wide.untracked_symbols
    );
    assert!(
        !wide
            .untracked_symbols
            .iter()
            .any(|u| u.trace_id == "TC-003"),
        "and the ones it does name leave nothing homeless: {:?}",
        wide.untracked_symbols
    );

    // The REFERENCE declaration was widened in step, so a row under a qualified
    // heading is read as a reference row too — and TC-002's `✅` over no test
    // is reported as the status lie it is. Widening only the target would mint
    // that row's id and leave its claim unread.
    assert_eq!(
        wide.status_lies
            .iter()
            .map(|l| l.row_id.as_deref().unwrap_or(""))
            .collect::<Vec<_>>(),
        vec!["TC-002"],
        "the reference reads every section the target mints from: {:?}",
        wide.status_lies
    );

    // ── The control: the SAME tree read by the SAME model with `section: Test
    // Cases` — one name, no wildcard. It must mint the one row under that exact
    // heading and no other, or the single-string form has silently changed
    // meaning for every module in the ecosystem.
    let narrow = report_for(&bundle, "iso").expect("model declared");
    let minted: Vec<(&str, usize, usize)> = narrow
        .groups
        .iter()
        .filter(|g| g.target == "test-case")
        .map(|g| (g.document.as_str(), g.backed, g.total))
        .collect();
    assert_eq!(
        minted,
        vec![("tests.md", 1, 1)],
        "one declared section, one row — a target declaring one section does \
         not start matching others: {:?}",
        narrow.groups
    );
    assert!(
        narrow.status_lies.is_empty(),
        "and its reference reads that one section too, so the lie under the \
         qualified heading is one this reading never saw: {:?}",
        narrow.status_lies
    );

    // Nothing about the widening is a new diagnostic: the sections are found,
    // so the CR-117 tokens stay silent on both readings.
    for (label, report) in [("wide", &wide), ("narrow", &narrow)] {
        assert!(
            !report
                .diagnostics
                .iter()
                .any(|d| d.reason == "section-matches-nothing"),
            "{label}: a declared section that was found is not one that was not: {:?}",
            report.diagnostics
        );
    }
}

#[trace("TC-1038", "FR-050-AC-34")]
// when NONE of several declared sections is in the (CR-118)
// document, the message names every one of them. A declaration naming three
// headings and a finding naming one leaves its reader to guess which of the
// three the document was supposed to spell.
#[test]
fn tc1038_the_section_finding_names_every_declared_section() {
    let bundle = iso_bundle("1038", &[("TC-001", "FR-001-AC-1", "✅")], &["TC-001"]);
    // Not one of `Test Cases*` / `Integration Test Matrix`.
    write(
        &bundle.scope,
        "tests.md",
        "---\nid: TM-001\ntype: TestMatrix\ntitle: Test Matrix\n---\n\n\
         # Test Matrix\n\n## Verification Cases\n\n\
         | ID | Traces To | Status |\n|----|-----------|--------|\n\
         | TC-001 | FR-001-AC-1 | ✅ |\n",
    );
    let report = report_for(&bundle, "iso-sections").expect("model declared");

    let finding = diagnostic_for(&report, "section-matches-nothing");
    assert_eq!(finding.declaration, "test-case");
    assert_eq!(finding.path.as_deref(), Some("tests.md"));
    for declared in ["'Test Cases*'", "'Integration Test Matrix'"] {
        assert!(
            finding.message.contains(declared),
            "names every section it tried ({declared}): {}",
            finding.message
        );
    }
    assert!(
        finding.message.contains("'Verification Cases'"),
        "and what the document has instead: {}",
        finding.message
    );

    // A wildcard is not a licence to match anything: `Test Cases*` anchors at
    // the start, so a heading that merely contains the words is not reached.
    write(
        &bundle.scope,
        "tests.md",
        "---\nid: TM-001\ntype: TestMatrix\ntitle: Test Matrix\n---\n\n\
         # Test Matrix\n\n## Deferred Test Cases\n\n\
         | ID | Traces To | Status |\n|----|-----------|--------|\n\
         | TC-001 | FR-001-AC-1 | ✅ |\n",
    );
    let anchored = report_for(&bundle, "iso-sections").expect("model declared");
    assert!(
        anchored
            .diagnostics
            .iter()
            .any(|d| d.reason == "section-matches-nothing"),
        "`Test Cases*` is a prefix, not a substring: {:?}",
        anchored.diagnostics
    );
}
