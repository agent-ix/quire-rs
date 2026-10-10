//! CR-187 `coverage_matrix` (FR-050-AC-47..51) and the range-in-trace-tag
//! finding (FR-050-AC-50, FR-051-AC-28).

use std::fs;
use std::path::{Path, PathBuf};

use ix_trace_rs::trace;
use quire_rs::coverage::{compute, CoverageMatrixStatus, CoverageReport};
use quire_rs::symbols::{extract_tree, trace as sym_trace};
use quire_rs::{Registry, Spec};

fn fixture_module(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("traceability")
        .join(name)
}

fn tmpdir(suffix: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!(
        "quire-rs-coverage-matrix-{}-{suffix}",
        std::process::id()
    ));
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

fn report_over(scope: &Path, module: &str) -> CoverageReport {
    let registry = Registry::load_module(&fixture_module(module)).expect("load module");
    let spec = Spec::from_path(&scope.join("spec"));
    let extraction = extract_tree(&scope.join("src"));
    let model = registry.traceability().cloned().unwrap_or_default();
    let graph = sym_trace::bind(&extraction, &model);
    compute(&spec, &registry, &graph, scope).expect("model declared")
}

#[trace("TC-1930", "FR-050-AC-47")]
// a fixture module declares an `obligations:` source over its FR
// documents, tagged tests and no `spec/tests.md` (or any other matrix file) at
// all: `coverage_matrix` lists one criterion per derived obligation, grouped
// by document; a minted `test-case` trace target has no corresponding
// obligation source declared here and so mints no criterion.
#[trace("TC-1931", "FR-050-AC-47")]
// the identical fixture with a `spec/tests.md` added — and the module
// declaring no `obligations:` source that reads it — serializes a
// byte-identical `coverage_matrix`.
#[test]
fn tc1930_tc1931_coverage_matrix_population_is_the_obligation_set() {
    let root = tmpdir("1930");
    write(
        &root,
        "spec/FR-001.md",
        "---\nid: FR-001\ntype: FR\ntitle: A requirement\n---\n\n\
         ## Acceptance Criteria\n\n\
         | ID | Criteria | Verification |\n|----|----------|--------------|\n\
         | FR-001-AC-1 | The system shall do it. | Test (TC-001) |\n",
    );
    write(
        &root,
        "src/lib.rs",
        "//! fixture\n\n#[cfg(test)]\nmod tests {\n    \
         #[trace(\"FR-001-AC-1\")]\n    #[test]\n    fn covers_0() {\n        let _ = 1;\n    }\n}\n",
    );
    let without_matrix = report_over(&root, "iso-obligations");
    assert_eq!(
        without_matrix.coverage_matrix.len(),
        1,
        "one requirement group for FR-001.md: {:?}",
        without_matrix.coverage_matrix
    );
    let requirement = &without_matrix.coverage_matrix[0];
    assert_eq!(requirement.document, "spec/FR-001.md");
    assert_eq!(requirement.criteria.len(), 1);
    assert_eq!(requirement.criteria[0].id, "FR-001-AC-1");
    assert_eq!(requirement.criteria[0].status, CoverageMatrixStatus::Tagged);

    // TC-1931: add a Test Matrix that mints a `test-case` trace target but no
    // obligation (this module declares no `obligations:` source reading it).
    // The obligations source only reads FR documents, so the matrix's
    // presence must not change `coverage_matrix` at all.
    write(
        &root,
        "spec/tests.md",
        "---\nid: TM-001\ntype: TestMatrix\ntitle: Test Matrix\n---\n\n\
         # Test Matrix\n\n## Test Cases\n\n\
         | ID | Traces To | Status |\n|----|-----------|--------|\n\
         | TC-001 | FR-001-AC-1 | \u{2705} |\n",
    );
    let with_matrix = report_over(&root, "iso-obligations");
    assert_eq!(
        serde_json::to_string(&with_matrix.coverage_matrix).unwrap(),
        serde_json::to_string(&without_matrix.coverage_matrix).unwrap(),
        "the matrix file mints a test-case trace target but no obligation, so \
         coverage_matrix must be byte-identical"
    );
    assert!(
        with_matrix.groups.iter().any(|g| g.target == "test-case"),
        "sanity: the matrix DID mint a test-case trace target"
    );

    let _ = fs::remove_dir_all(&root);
}

#[trace("TC-1932", "FR-050-AC-48")]
// a criterion bound by two TypeScript registrations on the same
// line carries two distinct binders, disambiguated by column; each binder
// carries path, line, column, qualified name, kind and ignored, ordered by
// (path, line, column) then qualified_name then kind; a criterion with no
// binder carries an empty binders list, never an absent key.
#[trace("TC-1944", "FR-050-AC-48")]
// a criterion bound by two identically-titled `it(...)` registrations at
// different declaration sites in the same file — one inside `describe('A',
// ...)`, one inside `describe('B', ...)` — carries two distinct binders keyed
// on (path, line, column) rather than the identically-spelled qualified name.
#[test]
fn tc1932_tc1944_binders_are_keyed_on_path_line_column_not_qualified_name() {
    let root = tmpdir("1932");
    write(
        &root,
        "spec/FR-001.md",
        "---\nid: FR-001\ntype: FR\ntitle: A requirement\n---\n\n\
         ## Acceptance Criteria\n\n\
         | ID | Criteria | Verification |\n|----|----------|--------------|\n\
         | FR-001-AC-1 | Every finding shall default to warning. | Test (TC-001) |\n\
         | FR-001-AC-2 | Every criterion shall bind. | Test (TC-002) |\n\
         | FR-001-AC-3 | An unbound criterion. | Test (TC-003) |\n",
    );
    write(
        &root,
        "src/service.test.ts",
        "it('a', () => { trace('FR-001-AC-1'); }); it('b', () => { trace('FR-001-AC-1'); });\n\
         describe('SuiteA', () => {\n  it('works', () => { trace('FR-001-AC-2'); });\n});\n\
         describe('SuiteB', () => {\n  it('works', () => { trace('FR-001-AC-2'); });\n});\n",
    );
    let report = report_over(&root, "iso-obligations");

    let criterion = |id: &str| {
        report
            .coverage_matrix
            .iter()
            .flat_map(|r| &r.criteria)
            .find(|c| c.id == id)
            .unwrap_or_else(|| panic!("no criterion {id}: {:?}", report.coverage_matrix))
    };

    // TC-1932: `a` and `b` share one line, disambiguated by column. Ordering
    // is asserted directly — (path, line, column, qualified_name, kind) —
    // not merely "both are present in some order".
    let ac1 = criterion("FR-001-AC-1");
    assert_eq!(
        ac1.binders.len(),
        2,
        "two distinct binders: {:?}",
        ac1.binders
    );
    assert_eq!(
        ac1.statement, "Every finding shall default to warning.",
        "CR-188: the criterion carries the obligation's own statement verbatim"
    );
    assert_eq!(ac1.binders[0].line, ac1.binders[1].line, "same line");
    assert!(
        ac1.binders[0].column < ac1.binders[1].column,
        "binders must be ordered by column ascending: {:?}",
        ac1.binders
    );
    assert_eq!(ac1.binders[0].qualified_name, "a");
    assert_eq!(ac1.binders[1].qualified_name, "b");

    // TC-1944: two `it('works', ...)` in different `describe` blocks.
    let ac2 = criterion("FR-001-AC-2");
    assert_eq!(
        ac2.binders.len(),
        2,
        "keying on qualified_name alone would have collapsed these into one: {:?}",
        ac2.binders
    );
    assert_eq!(
        ac2.statement, "Every criterion shall bind.",
        "CR-188: statement carried for this criterion too"
    );
    assert!(ac2.binders.iter().all(|b| b.qualified_name == "works"));
    assert!(
        ac2.binders[0].line < ac2.binders[1].line,
        "binders must be ordered by line ascending: {:?}",
        ac2.binders
    );

    // A criterion with no binder carries an empty list, never an absent key.
    let ac3 = criterion("FR-001-AC-3");
    assert!(ac3.binders.is_empty());
    let json = serde_json::to_value(ac3).expect("criterion serializes");
    assert!(
        json.get("binders").is_some(),
        "binders key must be present even when empty: {json}"
    );

    let _ = fs::remove_dir_all(&root);
}

#[trace("FR-050-AC-48")]
// A reserved TypeScript word is valid as an optional property name in a type
// literal. It must not make the whole source module disappear from the
// computed matrix (AGE-2236).
#[test]
fn optional_abstract_property_preserves_typescript_matrix_binding() {
    let root = tmpdir("age-2236-abstract");
    write(
        &root,
        "spec/FR-001.md",
        "---\nid: FR-001\ntype: FR\ntitle: A requirement\n---\n\n\
         ## Acceptance Criteria\n\n\
         | ID | Criteria | Verification |\n|----|----------|--------------|\n\
         | FR-001-AC-1 | The TypeScript test shall remain visible. | Test (TC-001) |\n",
    );
    write(
        &root,
        "src/record.test.ts",
        "type TestRecord = { abstract?: boolean };\n\
         it('keeps the module binding', () => { trace('FR-001-AC-1'); });\n",
    );

    let report = report_over(&root, "iso-obligations");
    let criterion = report
        .coverage_matrix
        .iter()
        .flat_map(|requirement| &requirement.criteria)
        .find(|criterion| criterion.id == "FR-001-AC-1")
        .expect("the criterion remains in the computed matrix");

    assert_eq!(criterion.status, CoverageMatrixStatus::Tagged);
    assert_eq!(
        criterion.binders.len(),
        1,
        "the TypeScript test remains bound"
    );
    assert_eq!(criterion.binders[0].path, "record.test.ts");
    assert_eq!(
        criterion.binders[0].qualified_name,
        "keeps the module binding"
    );
    assert_eq!(report.binding_census.bound, 1);

    let _ = fs::remove_dir_all(&root);
}

#[trace("FR-050-AC-48")]
// A malformed optional property must keep the parser diagnostic even when
// the first diagnostic points at the otherwise recoverable `abstract?` token.
#[test]
fn malformed_optional_abstract_property_remains_unbound() {
    let root = tmpdir("age-2236-malformed-abstract");
    write(
        &root,
        "spec/FR-001.md",
        "---\nid: FR-001\ntype: FR\ntitle: A requirement\n---\n\n\
         ## Acceptance Criteria\n\n\
         | ID | Criteria | Verification |\n|----|----------|--------------|\n\
         | FR-001-AC-1 | The malformed TypeScript test shall be rejected. | Test (TC-002) |\n",
    );
    write(
        &root,
        "src/malformed.test.ts",
        "type TestRecord = { abstract?: : boolean };\n\
         type BrokenRecord = { ??? };\n\
         it('must remain unbound', () => { trace('FR-001-AC-1'); });\n",
    );

    let extraction = extract_tree(&root.join("src"));
    assert_eq!(
        extraction.symbols.len(),
        0,
        "malformed file must mint no symbols"
    );
    assert_eq!(extraction.diagnostics.len(), 1);
    assert_eq!(extraction.diagnostics[0].path, "malformed.test.ts");
    assert!(extraction.diagnostics[0]
        .reason
        .contains("unresolvable declaration structure"));

    let report = report_over(&root, "iso-obligations");
    let criterion = report
        .coverage_matrix
        .iter()
        .flat_map(|requirement| &requirement.criteria)
        .find(|criterion| criterion.id == "FR-001-AC-1")
        .expect("the criterion remains in the computed matrix");
    assert_eq!(criterion.status, CoverageMatrixStatus::Untagged);
    assert!(criterion.binders.is_empty());

    let _ = fs::remove_dir_all(&root);
}

#[trace("TC-1933", "FR-050-AC-49")]
// a criterion bound by one non-ignored test symbol computes
// tagged; a criterion bound by none computes untagged.
#[trace("TC-1934", "FR-050-AC-49")]
// a criterion whose sole binder is ignored computes
// tagged-by-ignored-test; adding a second, non-ignored binder changes it to
// tagged.
#[test]
fn tc1933_tc1934_computed_status_reads_the_binder_set() {
    let root = tmpdir("1933");
    write(
        &root,
        "spec/FR-001.md",
        "---\nid: FR-001\ntype: FR\ntitle: A requirement\n---\n\n\
         ## Acceptance Criteria\n\n\
         | ID | Criteria | Verification |\n|----|----------|--------------|\n\
         | FR-001-AC-1 | Bound by one non-ignored test. | Test (TC-001) |\n\
         | FR-001-AC-2 | Bound by nothing. | Test (TC-002) |\n\
         | FR-001-AC-3 | Bound by an ignored test alone. | Test (TC-003) |\n",
    );
    write(
        &root,
        "src/lib.rs",
        "//! fixture\n\n#[cfg(test)]\nmod tests {\n\
         \x20   #[trace(\"FR-001-AC-1\")]\n    #[test]\n    fn covers_1() {\n        let _ = 1;\n    }\n\n\
         \x20   #[trace(\"FR-001-AC-3\")]\n    #[ignore]\n    #[test]\n    fn covers_3_ignored() {\n        let _ = 1;\n    }\n}\n",
    );
    let report = report_over(&root, "iso-obligations");
    let status = |id: &str| {
        report
            .coverage_matrix
            .iter()
            .flat_map(|r| &r.criteria)
            .find(|c| c.id == id)
            .unwrap_or_else(|| panic!("no criterion {id}"))
            .status
    };
    assert_eq!(status("FR-001-AC-1"), CoverageMatrixStatus::Tagged);
    assert_eq!(status("FR-001-AC-2"), CoverageMatrixStatus::Untagged);
    assert_eq!(
        status("FR-001-AC-3"),
        CoverageMatrixStatus::TaggedByIgnoredTest
    );

    // Add a second, non-ignored binder for FR-001-AC-3: it must flip to Tagged.
    write(
        &root,
        "src/lib.rs",
        "//! fixture\n\n#[cfg(test)]\nmod tests {\n\
         \x20   #[trace(\"FR-001-AC-1\")]\n    #[test]\n    fn covers_1() {\n        let _ = 1;\n    }\n\n\
         \x20   #[trace(\"FR-001-AC-3\")]\n    #[ignore]\n    #[test]\n    fn covers_3_ignored() {\n        let _ = 1;\n    }\n\n\
         \x20   #[trace(\"FR-001-AC-3\")]\n    #[test]\n    fn covers_3_running() {\n        let _ = 1;\n    }\n}\n",
    );
    let report2 = report_over(&root, "iso-obligations");
    let status2 = |id: &str| {
        report2
            .coverage_matrix
            .iter()
            .flat_map(|r| &r.criteria)
            .find(|c| c.id == id)
            .unwrap_or_else(|| panic!("no criterion {id}"))
            .status
    };
    assert_eq!(
        status2("FR-001-AC-3"),
        CoverageMatrixStatus::Tagged,
        "adding a non-ignored binder flips tagged-by-ignored-test to tagged"
    );

    let _ = fs::remove_dir_all(&root);
}

#[trace("TC-1935", "FR-050-AC-49")]
// a criterion whose obligation's declared method is in the
// module's declared `no_source_symbol` vocabulary computes
// method-without-symbol whether or not it also carries a non-ignored binder —
// the method exemption wins over every other case.
#[test]
fn tc1935_method_without_symbol_wins_over_every_other_case() {
    let root = tmpdir("1935");
    write(
        &root,
        "spec/FR-001.md",
        "---\nid: FR-001\ntype: FR\ntitle: A requirement\n---\n\n\
         ## Acceptance Criteria\n\n\
         | ID | Criteria | Verification |\n|----|----------|--------------|\n\
         | FR-001-AC-1 | Verified by inspection alone. | Inspection |\n\
         | FR-001-AC-2 | Verified by inspection, but also tagged. | Inspection |\n",
    );
    write(
        &root,
        "src/lib.rs",
        "//! fixture\n\n#[cfg(test)]\nmod tests {\n\
         \x20   #[trace(\"FR-001-AC-2\")]\n    #[test]\n    fn covers_2() {\n        let _ = 1;\n    }\n}\n",
    );
    let report = report_over(&root, "iso-obligations");
    let status = |id: &str| {
        report
            .coverage_matrix
            .iter()
            .flat_map(|r| &r.criteria)
            .find(|c| c.id == id)
            .unwrap_or_else(|| panic!("no criterion {id}"))
            .status
    };
    assert_eq!(
        status("FR-001-AC-1"),
        CoverageMatrixStatus::MethodWithoutSymbol
    );
    assert_eq!(
        status("FR-001-AC-2"),
        CoverageMatrixStatus::MethodWithoutSymbol,
        "the method exemption wins even though a non-ignored binder exists"
    );

    let _ = fs::remove_dir_all(&root);
}

#[trace("TC-1936", "FR-050-AC-50", "FR-051-AC-28")]
// `#[trace("FR-034-AC-1..FR-034-AC-5")]` mints no `verifies`
// relation for any id in the range, including its endpoints, and is reported
// once under `range-in-trace-tag`; the range does not appear in
// `untracked_symbols` or `unmatched_tags`.
#[test]
fn tc1936_a_marker_range_binds_nothing_and_is_reported() {
    let root = tmpdir("1936");
    write(
        &root,
        "spec/FR-034.md",
        "---\nid: FR-034\ntype: FR\ntitle: A requirement\n---\n\n\
         ## Acceptance Criteria\n\n\
         | ID | Criteria | Verification |\n|----|----------|--------------|\n\
         | FR-034-AC-1 | First. | Test (TC-001) |\n\
         | FR-034-AC-2 | Second. | Test (TC-002) |\n\
         | FR-034-AC-3 | Third. | Test (TC-003) |\n\
         | FR-034-AC-4 | Fourth. | Test (TC-004) |\n\
         | FR-034-AC-5 | Fifth. | Test (TC-005) |\n",
    );
    write(
        &root,
        "src/lib.rs",
        "//! fixture\n\n#[cfg(test)]\nmod tests {\n\
         \x20   #[trace(\"FR-034-AC-1..FR-034-AC-5\")]\n    #[test]\n    fn covers_range() {\n        let _ = 1;\n    }\n}\n",
    );
    let report = report_over(&root, "iso-obligations");

    // "Reported once", naming the line and the qualified symbol — not merely
    // that a reason/value pair exists somewhere.
    let range_findings: Vec<_> = report
        .diagnostics
        .iter()
        .filter(|d| d.reason == "range-in-trace-tag")
        .collect();
    assert_eq!(
        range_findings.len(),
        1,
        "reported exactly once: {:?}",
        report.diagnostics
    );
    let finding = range_findings[0];
    assert_eq!(finding.value.as_deref(), Some("FR-034-AC-1..FR-034-AC-5"));
    assert_eq!(finding.path.as_deref(), Some("lib.rs"));
    assert_eq!(finding.line, Some(5));
    assert_eq!(finding.declaration, "tests::covers_range");
    assert!(
        report.untracked_symbols.is_empty(),
        "the range must not land in untracked_symbols: {:?}",
        report.untracked_symbols
    );
    assert!(
        report.unmatched_tags.is_empty(),
        "the range must not land in unmatched_tags either: {:?}",
        report.unmatched_tags
    );
    // Every criterion in the range keeps its other status — untagged, since
    // this is its only tag and the tag bound nothing.
    for id in [
        "FR-034-AC-1",
        "FR-034-AC-2",
        "FR-034-AC-3",
        "FR-034-AC-4",
        "FR-034-AC-5",
    ] {
        let status = report
            .coverage_matrix
            .iter()
            .flat_map(|r| &r.criteria)
            .find(|c| c.id == id)
            .unwrap_or_else(|| panic!("no criterion {id}"))
            .status;
        assert_eq!(status, CoverageMatrixStatus::Untagged, "{id}: {status:?}");
    }

    let _ = fs::remove_dir_all(&root);
}

#[trace("TC-1937", "FR-050-AC-50", "FR-051-AC-28")]
// a legacy range (same-prefix, differing-prefix, and short-suffix
// forms) binds no id at all — not even the left endpoint, which the legacy
// form used to bind alone.
#[test]
fn tc1937_a_legacy_range_binds_neither_endpoint() {
    let root = tmpdir("1937");
    write(
        &root,
        "spec/FR-034.md",
        "---\nid: FR-034\ntype: FR\ntitle: A requirement\n---\n\n\
         ## Acceptance Criteria\n\n\
         | ID | Criteria | Verification |\n|----|----------|--------------|\n\
         | FR-034-AC-1 | First. | Test (TC-001) |\n\
         | FR-034-AC-5 | Fifth. | Test (TC-005) |\n",
    );
    write(
        &root,
        "spec/FR-035.md",
        "---\nid: FR-035\ntype: FR\ntitle: Another requirement\n---\n\n\
         ## Acceptance Criteria\n\n\
         | ID | Criteria | Verification |\n|----|----------|--------------|\n\
         | FR-035-AC-2 | Second. | Test (TC-006) |\n",
    );
    write(
        &root,
        "src/lib.rs",
        "//! fixture\n\n#[cfg(test)]\nmod tests {\n\
         \x20   // Trace: FR-034-AC-1..FR-034-AC-5\n    #[test]\n    fn same_prefix() {\n        let _ = 1;\n    }\n\n\
         \x20   // Trace: FR-034-AC-1..FR-035-AC-2\n    #[test]\n    fn differing_prefix() {\n        let _ = 1;\n    }\n\n\
         \x20   // Trace: FR-034-AC-1..5\n    #[test]\n    fn short_suffix() {\n        let _ = 1;\n    }\n}\n",
    );
    let report = report_over(&root, "iso-obligations");

    assert_eq!(
        report.untracked_symbols.len(),
        0,
        "no endpoint bound, so nothing to report as untracked either: {:?}",
        report.untracked_symbols
    );
    let ranges: Vec<&str> = report
        .diagnostics
        .iter()
        .filter(|d| d.reason == "range-in-trace-tag")
        .filter_map(|d| d.value.as_deref())
        .collect();
    assert!(ranges.contains(&"FR-034-AC-1..FR-034-AC-5"), "{ranges:?}");
    assert!(ranges.contains(&"FR-034-AC-1..FR-035-AC-2"), "{ranges:?}");
    assert!(ranges.contains(&"FR-034-AC-1..5"), "{ranges:?}");

    // FR-034-AC-1 must not be bound by any of the three — the legacy form
    // used to bind its left endpoint alone; that stops.
    let status = |id: &str| {
        report
            .coverage_matrix
            .iter()
            .flat_map(|r| &r.criteria)
            .find(|c| c.id == id)
            .unwrap_or_else(|| panic!("no criterion {id}"))
            .status
    };
    assert_eq!(status("FR-034-AC-1"), CoverageMatrixStatus::Untagged);
    assert_eq!(status("FR-034-AC-5"), CoverageMatrixStatus::Untagged);
    assert_eq!(status("FR-035-AC-2"), CoverageMatrixStatus::Untagged);

    let _ = fs::remove_dir_all(&root);
}

#[trace("TC-1938", "FR-050-AC-51", "FR-050-AC-7")]
// two runs over an identical corpus serialize byte-identically; a
// module declaring no `obligations:` source omits `coverage_matrix` entirely
// rather than emitting an empty structure.
#[test]
fn tc1938_determinism_and_zero_population_omission() {
    let root = tmpdir("1938");
    write(
        &root,
        "spec/FR-001.md",
        "---\nid: FR-001\ntype: FR\ntitle: A requirement\n---\n\n\
         ## Acceptance Criteria\n\n\
         | ID | Criteria | Verification |\n|----|----------|--------------|\n\
         | FR-001-AC-1 | The system shall do it. | Test (TC-001) |\n",
    );
    write(
        &root,
        "src/lib.rs",
        "//! fixture\n\n#[cfg(test)]\nmod tests {\n    \
         #[trace(\"FR-001-AC-1\")]\n    #[test]\n    fn covers_0() {\n        let _ = 1;\n    }\n}\n",
    );

    let a = report_over(&root, "iso-obligations");
    let b = report_over(&root, "iso-obligations");
    assert_eq!(
        a.to_json(),
        b.to_json(),
        "repeated runs must be byte-identical"
    );
    assert!(!a.coverage_matrix.is_empty());
    assert!(a.to_json().contains("\"coverage_matrix\""));

    // The `iso` module declares no `obligations:` source at all.
    let none = report_over(&root, "iso");
    assert!(
        none.coverage_matrix.is_empty(),
        "no obligations source means no population"
    );
    assert!(
        !none.to_json().contains("\"coverage_matrix\""),
        "omitted entirely, never an empty array"
    );

    let _ = fs::remove_dir_all(&root);
}

#[trace("TC-1942", "FR-050-AC-47", "FR-050-AC-5")]
// `#[trace("NFR-012-M-1")]` on a test function, where
// `NFR-012-M-1` is minted only as a derived NFR-metric obligation with no
// corresponding trace target, resolves against the obligation population and
// is not reported in `untracked_symbols`.
#[test]
fn tc1942_an_obligation_only_id_is_not_untracked() {
    let root = tmpdir("1942");
    write(
        &root,
        "spec/NFR-012.md",
        "---\nid: NFR-012\ntype: NFR\ntitle: Some NFR\n---\n\n\
         ## Acceptance Criteria\n\n\
         | ID | Criteria | Verification |\n|----|----------|--------------|\n\
         | NFR-012-AC-1 | Some criterion. | Analysis |\n\n\
         ## Measurement and Evaluation\n\n\
         | Metric | Method |\n|--------|--------|\n\
         | Some metric shall hold. | Test |\n",
    );
    write(
        &root,
        "src/lib.rs",
        "//! fixture\n\n#[cfg(test)]\nmod tests {\n    \
         #[trace(\"NFR-012-M-1\")]\n    #[test]\n    fn covers_metric() {\n        let _ = 1;\n    }\n}\n",
    );
    let report = report_over(&root, "obligations-nfr-tagged");

    assert!(
        report.obligations.iter().any(|o| o.id == "NFR-012-M-1"),
        "fixture premise: the metric row mints this obligation id: {:?}",
        report.obligations
    );
    assert!(
        !report
            .untracked_symbols
            .iter()
            .any(|s| s.trace_id == "NFR-012-M-1"),
        "an obligation-only id must not be untracked: {:?}",
        report.untracked_symbols
    );
    let criterion = report
        .coverage_matrix
        .iter()
        .flat_map(|r| &r.criteria)
        .find(|c| c.id == "NFR-012-M-1")
        .expect("the obligation-only id is a coverage_matrix criterion");
    assert_eq!(criterion.status, CoverageMatrixStatus::Tagged);

    let _ = fs::remove_dir_all(&root);
}
