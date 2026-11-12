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
         #[trace(\"TC-001\", \"FR-001-AC-1\")]\n    #[test]\n    fn covers_0() {\n        let _ = 1;\n    }\n}\n",
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
        "it('a', () => { trace('TC-001', 'FR-001-AC-1'); }); it('b', () => { trace('TC-001', 'FR-001-AC-1'); });\n\
         describe('SuiteA', () => {\n  it('works', () => { trace('TC-002', 'FR-001-AC-2'); });\n});\n\
         describe('SuiteB', () => {\n  it('works', () => { trace('TC-002', 'FR-001-AC-2'); });\n});\n",
    );
    write(
        &root,
        "spec/tests.md",
        "---\nid: TM-001\ntype: TestMatrix\n---\n\n## Test Cases\n\n| ID | Traces To | Status |\n|---|---|---|\n| TC-001 | FR-001-AC-1 | 🚧 |\n| TC-002 | FR-001-AC-2 | 🚧 |\n",
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
         it('keeps the module binding', () => { trace('TC-001', 'FR-001-AC-1'); });\n",
    );

    write(
        &root,
        "spec/tests.md",
        "---\nid: TM-001\ntype: TestMatrix\n---\n\n## Test Cases\n\n| ID | Traces To | Status |\n|---|---|---|\n| TC-001 | FR-001-AC-1 | 🚧 |\n",
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
    assert_eq!(
        report
            .binding_census
            .iter()
            .find(|census| census.language == "typescript")
            .map(|census| census.bound),
        Some(1)
    );

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
         \x20   #[trace(\"TC-001\", \"FR-001-AC-1\")]\n    #[test]\n    fn covers_1() {\n        let _ = 1;\n    }\n\n\
         \x20   #[trace(\"TC-003\", \"FR-001-AC-3\")]\n    #[ignore]\n    #[test]\n    fn covers_3_ignored() {\n        let _ = 1;\n    }\n}\n",
    );
    write(
        &root,
        "spec/tests.md",
        "---\nid: TM-001\ntype: TestMatrix\n---\n\n## Test Cases\n\n| ID | Traces To | Status |\n|---|---|---|\n| TC-001 | FR-001-AC-1 | 🚧 |\n| TC-003 | FR-001-AC-3 | 🚧 |\n",
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
         \x20   #[trace(\"TC-001\", \"FR-001-AC-1\")]\n    #[test]\n    fn covers_1() {\n        let _ = 1;\n    }\n\n\
         \x20   #[trace(\"TC-003\", \"FR-001-AC-3\")]\n    #[ignore]\n    #[test]\n    fn covers_3_ignored() {\n        let _ = 1;\n    }\n\n\
         \x20   #[trace(\"TC-003\", \"FR-001-AC-3\")]\n    #[test]\n    fn covers_3_running() {\n        let _ = 1;\n    }\n}\n",
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
// a range in a marker (same-prefix, differing-prefix, and short-suffix
// forms) binds no id at all — not even the left endpoint.
#[test]
fn tc1937_a_marker_range_binds_neither_endpoint() {
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
         \x20   #[trace(\"FR-034-AC-1..FR-034-AC-5\")]\n    #[test]\n    fn same_prefix() {\n        let _ = 1;\n    }\n\n\
         \x20   #[trace(\"FR-034-AC-1..FR-035-AC-2\")]\n    #[test]\n    fn differing_prefix() {\n        let _ = 1;\n    }\n\n\
         \x20   #[trace(\"FR-034-AC-1..5\")]\n    #[test]\n    fn short_suffix() {\n        let _ = 1;\n    }\n}\n",
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

    // FR-034-AC-1 must not be bound by any of the three.
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

/// Core projection fixtures deliberately do not assert producer qualification.
/// The Quoin bridge must separately exercise the actual auditor and FR-116 acts.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct BackingComponentCase {
    name: String,
    issue_ref: String,
    id: String,
    method: String,
    statement: String,
    component_only: bool,
}

#[trace("FR-078-AC-12", "FR-078-AC-13", "FR-078-AC-14")]
#[test]
fn retained_backing_components_reconcile_named_source_free_boundaries() {
    use quire_rs::evidence::{
        strict_decision, trace, ActResult, BackingInput, CauseCode, RecordClaim, RecordLocator,
        TraceQuery,
    };
    use quire_rs::obligation::inventory::{inventory, InventoryOptions};

    let cases: Vec<BackingComponentCase> = serde_json::from_str(include_str!(
        "../fixtures/corpus_cases/qualified-evidence-cases.json"
    ))
    .expect("attributed component fixture data");
    assert_eq!(cases.len(), 4);
    for case in cases {
        assert!(case.component_only && !case.issue_ref.is_empty());
        let root = tmpdir(&case.name);
        write(&root, "spec/FR-001.md", &format!(
            "---\nid: FR-001\ntype: FR\n---\n\n## Acceptance Criteria\n\n| ID | Criteria | Verification |\n|---|---|---|\n| {} | {} | {} |\n",
            case.id, case.statement, case.method));
        let registry = Registry::load_module(&fixture_module("iso-obligations")).unwrap();
        let model = registry.traceability().unwrap();
        let inventory = inventory(
            &root,
            model,
            &["acceptance-criterion"],
            InventoryOptions::default(),
        );
        let entry = inventory.entries().next().expect("native selected target");
        let spec = Spec::from_path(&root.join("spec"));
        let graph = sym_trace::SymbolGraph::default();
        let baseline = compute(&spec, &registry, &graph, &root).unwrap();
        assert_eq!(baseline.unbacked_rows.len(), 1, "{} before", case.name);
        assert!(baseline.coverage_matrix[0].criteria[0].binders.is_empty());
        assert_eq!(
            baseline.coverage_matrix[0].criteria[0].method.as_deref(),
            Some(case.method.as_str())
        );
        assert_eq!(
            baseline.coverage_matrix[0].criteria[0].status,
            if case.method == "Inspection" {
                CoverageMatrixStatus::MethodWithoutSymbol
            } else {
                CoverageMatrixStatus::Untagged
            },
            "the named owner method determines static no-symbol classification"
        );
        let legacy_json = baseline.to_json();
        assert!(!legacy_json.contains("record_binders"));
        assert!(!legacy_json.contains("evidence_results"));

        let absent = [ActResult {
            target: Some(entry.identity().into()),
            requested_target: &case.id,
            locator: None,
            cause: CauseCode::Absent,
            detail: "required retained act absent",
        }];
        let empty = BackingInput {
            inventory: &inventory,
            claims: &[],
            results: &absent,
        };
        let report =
            quire_rs::coverage::compute_with_evidence(&spec, &registry, &graph, &root, &empty)
                .unwrap();
        assert_eq!(report.unbacked_rows, baseline.unbacked_rows);
        assert_eq!(
            report.coverage_matrix[0].criteria[0].record_binders,
            Some(vec![])
        );
        let rejection = strict_decision(&report, &[]).unwrap_err();
        assert_eq!(rejection.evidence[0].cause, CauseCode::Absent);
        assert!(rejection.evidence[0].locator.is_none());

        // This is a low-level input test, not a forged owner qualification token.
        let claims = [RecordClaim {
            target: entry.identity().into(),
            locator: RecordLocator {
                record: "owner/record",
                run: "owner/run",
                entry: "owner/entry",
            },
            method: &case.method,
            actor: "component-input",
            provenance: &["component-projection-only"],
        }];
        let backing = BackingInput {
            inventory: &inventory,
            claims: &claims,
            results: &[],
        };
        let report =
            quire_rs::coverage::compute_with_evidence(&spec, &registry, &graph, &root, &backing)
                .unwrap();
        let schema_value: serde_json::Value =
            serde_json::from_str(include_str!("../../schemas/output/coverage-v1.schema.json"))
                .unwrap();
        let schema = jsonschema::options()
            .with_draft(jsonschema::Draft::Draft202012)
            .build(&schema_value)
            .unwrap();
        let payload = serde_json::to_value(&report).unwrap();
        let errors: Vec<_> = schema
            .iter_errors(&payload)
            .map(|error| error.to_string())
            .collect();
        assert!(
            errors.is_empty(),
            "{} runtime schema: {:?}",
            case.name,
            errors
        );
        let criterion = &report.coverage_matrix[0].criteria[0];
        assert_eq!(criterion.status, CoverageMatrixStatus::QualifiedRecord);
        assert!(
            criterion.binders.is_empty(),
            "records never manufacture symbols"
        );
        assert_eq!(criterion.record_binders.as_ref().unwrap().len(), 1);
        assert_eq!(criterion.method.as_deref(), Some(case.method.as_str()));
        assert_eq!(
            criterion.record_binders.as_ref().unwrap()[0].method,
            case.method,
            "retained binding preserves Inspection and compile-time-check without relabeling"
        );
        assert!(report.unbacked_rows.is_empty());
        assert!(report.no_symbol_rows.is_empty());
        assert_eq!(report.totals.backed, baseline.totals.backed + 1);
        assert!(strict_decision(&report, &[]).is_ok());
        assert_eq!(
            trace(&backing, TraceQuery::Target(entry.identity().into())),
            *criterion.record_binders.as_ref().unwrap()
        );
        assert_eq!(
            trace(&backing, TraceQuery::Run("owner/run")),
            *criterion.record_binders.as_ref().unwrap()
        );

        let failed = [ActResult {
            target: Some(entry.identity().into()),
            requested_target: &case.id,
            locator: Some(claims[0].locator),
            cause: CauseCode::Failed,
            detail: "original owner failure",
        }];
        let rejected = BackingInput {
            inventory: &inventory,
            claims: &claims,
            results: &failed,
        };
        let report =
            quire_rs::coverage::compute_with_evidence(&spec, &registry, &graph, &root, &rejected)
                .unwrap();
        assert_eq!(report.unbacked_rows, baseline.unbacked_rows);
        assert!(report.coverage_matrix[0].criteria[0]
            .record_binders
            .as_ref()
            .unwrap()
            .is_empty());
        assert!(strict_decision(&report, &[])
            .unwrap_err()
            .evidence
            .iter()
            .any(|r| r.cause == CauseCode::Failed));
        assert_eq!(
            compute(&spec, &registry, &graph, &root).unwrap().to_json(),
            legacy_json
        );
        fs::remove_dir_all(root).unwrap();
    }
}

#[trace("FR-078-AC-9", "FR-078-AC-13", "FR-078-AC-14", "FR-078-AC-15")]
#[test]
fn retained_gate_targets_do_not_discharge_inspection_siblings() {
    use quire_rs::evidence::{
        declared_target, BackingInput, RecordClaim, RecordLocator, TraceQuery,
    };
    use quire_rs::obligation::inventory::{inventory, InventoryLookupError, InventoryOptions};
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../fixtures/corpus_cases/qualified-evidence-rt-gates.json"
    ))
    .unwrap();
    assert_eq!(fixture["component_only"], true);
    assert!(fixture["issue_ref"].as_str().unwrap().contains("#138"));
    let root = tmpdir("retained-rt-gates");
    for (path, body) in fixture["documents"].as_object().unwrap() {
        write(&root, path, body.as_str().unwrap());
    }
    let registry = Registry::load_module(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/qualified_evidence/rt-module"),
    )
    .unwrap();
    let model = registry.traceability().unwrap();
    let inventory = inventory(
        &root,
        model,
        &["acceptance-criterion"],
        InventoryOptions::default(),
    );
    assert_eq!(
        inventory.len(),
        2,
        "TC rows are not invented criterion entries"
    );
    let spec = Spec::from_path(&root.join("spec"));
    let graph = sym_trace::SymbolGraph::default();
    let baseline = compute(&spec, &registry, &graph, &root).unwrap();
    let targets: Vec<_> = ["TC-198", "TC-199"]
        .iter()
        .map(|id| {
            declared_target(
                &inventory,
                &spec,
                "test-case",
                "spec/exact/matrix/tests.md",
                id,
            )
            .unwrap()
        })
        .collect();
    assert_eq!(
        declared_target(&inventory, &spec, "test-case", "spec/foreign.md", "TC-198"),
        Err(InventoryLookupError::UnknownIdentity)
    );
    let document_target = declared_target(
        &inventory,
        &spec,
        "test-case-document",
        "spec/exact/tests/TC-198.md",
        "TC-198",
    )
    .unwrap();
    assert!(
        document_target.line().is_none(),
        "frontmatter target is not a fabricated table row"
    );
    assert_eq!(document_target.id(), "TC-198");
    let targets: Vec<_> = targets.into_iter().chain([document_target]).collect();
    let claims: Vec<_> = targets
        .iter()
        .map(|target| RecordClaim {
            target: (*target).into(),
            locator: RecordLocator {
                record: "component-record",
                run: "component-run",
                entry: "per-target-observed-entry",
            },
            method: "Test",
            actor: "component-projection",
            provenance: &["not-producer-qualification"],
        })
        .collect();
    let backing = BackingInput {
        inventory: &inventory,
        claims: &claims,
        results: &[],
    };
    let report =
        quire_rs::coverage::compute_with_evidence(&spec, &registry, &graph, &root, &backing)
            .unwrap();
    assert_eq!(report.totals.backed, baseline.totals.backed + 2);
    assert_eq!(report.coverage_matrix.len(), 1);
    assert_eq!(report.coverage_matrix[0].criteria.len(), 2);
    for criterion in &report.coverage_matrix[0].criteria {
        assert_eq!(criterion.status, CoverageMatrixStatus::MethodWithoutSymbol);
        assert_eq!(
            criterion.record_binders,
            Some(vec![]),
            "TC receipt cannot discharge sibling Inspection"
        );
    }
    assert_eq!(report.no_symbol_rows.len(), 2);
    assert_eq!(report.record_binders.as_ref().unwrap().len(), 3);
    assert_eq!(
        quire_rs::evidence::trace(&backing, TraceQuery::Run("component-run")).len(),
        3
    );
    assert!(quire_rs::evidence::strict_decision(&report, &[]).is_err());
    fs::remove_dir_all(root).unwrap();
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct BackingContextCase {
    name: String,
    issue_ref: String,
    reject: bool,
}

#[trace("FR-078-AC-12", "FR-078-AC-16")]
#[test]
fn retained_backing_uses_native_snapshot_and_preserves_partial_population() {
    use quire_rs::evidence::{BackingInput, CauseCode};
    use quire_rs::obligation::inventory::{inventory, InventoryOptions};
    let cases: Vec<BackingContextCase> = serde_json::from_str(include_str!(
        "../fixtures/corpus_cases/qualified-evidence-context-cases.json"
    ))
    .unwrap();
    for case in cases {
        assert!(!case.issue_ref.is_empty());
        let root = tmpdir(&case.name);
        let body = "---\nid: FR-001\ntype: FR\n---\n\n## Acceptance Criteria\n\n| ID | Criteria | Verification |\n|---|---|---|\n| FR-001-AC-1 | The result shall be retained. | Test |\n";
        write(&root, "spec/FR-001.md", body);
        let registry = Registry::load_module(&fixture_module("iso-obligations")).unwrap();
        let mut selected_model = registry.traceability().unwrap().clone();
        selected_model.obligations[0]
            .exclude
            .push("spec/excluded/**".into());
        // Give both producer and coverage the same declared source exclusion.
        let module_root = root.join("module");
        fs::create_dir_all(&module_root).unwrap();
        let mut module: serde_yaml::Value = serde_yaml::from_str(
            &fs::read_to_string(fixture_module("iso-obligations").join("manifest.yaml")).unwrap(),
        )
        .unwrap();
        module["traceability"] = serde_yaml::to_value(&selected_model).unwrap();
        fs::write(
            module_root.join("manifest.yaml"),
            serde_yaml::to_string(&module).unwrap(),
        )
        .unwrap();
        let registry = Registry::load_module(&module_root).unwrap();
        let model = registry.traceability().expect("valid model");
        let mut inventory_model = model.clone();
        if case.name == "model-changed" {
            inventory_model.exclude.push("unrelated/**".into());
        }
        if case.name == "source-exclusion-changed" {
            inventory_model.obligations[0]
                .exclude
                .push("additional/**".into());
        }
        let selected = inventory(
            &root,
            &inventory_model,
            &["acceptance-criterion"],
            InventoryOptions::default(),
        );
        assert_eq!(selected.len(), 1);
        match case.name.as_str() {
            "selected-added" => write(&root, "spec/FR-002.md", &body.replace("FR-001", "FR-002")),
            "selected-changed" => write(
                &root,
                "spec/FR-001.md",
                &body.replace("retained", "reported"),
            ),
            "selected-removed" => fs::remove_file(root.join("spec/FR-001.md")).unwrap(),
            "unselected-added" => write(
                &root,
                "spec/extra.md",
                "---\nid: TM-1\ntype: TestMatrix\n---\n",
            ),
            "excluded-added" => write(
                &root,
                "spec/excluded/FR-002.md",
                &body.replace("FR-001", "FR-002"),
            ),
            "model-changed" | "source-exclusion-changed" | "root-changed" => {}
            other => panic!("unknown fixture mutation: {other}"),
        }
        let spec = Spec::from_path(&root.join("spec"));
        let graph = sym_trace::SymbolGraph::default();
        let backing = BackingInput {
            inventory: &selected,
            claims: &[],
            results: &[],
        };
        let other_root = root.join("different-scope");
        let coverage_root = if case.name == "root-changed" {
            &other_root
        } else {
            &root
        };
        let report = quire_rs::coverage::compute_with_evidence(
            &spec,
            &registry,
            &graph,
            coverage_root,
            &backing,
        )
        .unwrap();
        assert_eq!(
            report
                .evidence_results
                .as_ref()
                .unwrap()
                .iter()
                .any(|r| r.cause == CauseCode::SelectionMismatch),
            case.reject,
            "{}",
            case.name
        );
        assert_eq!(
            report
                .coverage_matrix
                .iter()
                .map(|r| r.criteria.len())
                .sum::<usize>(),
            1,
            "owner population retained, not a second census"
        );
        if !case.reject {
            let partial = inventory(
                &root,
                model,
                &["acceptance-criterion"],
                InventoryOptions {
                    stop_limit: Some(1),
                    ..InventoryOptions::default()
                },
            );
            let backing = BackingInput {
                inventory: &partial,
                claims: &[],
                results: &[],
            };
            let report = quire_rs::coverage::compute_with_evidence(
                &spec, &registry, &graph, &root, &backing,
            )
            .unwrap();
            assert_eq!(report.coverage_matrix[0].criteria.len(), 1);
            assert!(report
                .evidence_results
                .as_ref()
                .unwrap()
                .iter()
                .any(|r| r.cause == CauseCode::InventoryIncomplete));
            assert!(quire_rs::evidence::strict_decision(&report, &[]).is_err());
        }
        fs::remove_dir_all(root).unwrap();
    }
}

#[trace("FR-078-AC-6", "FR-078-AC-10", "FR-078-AC-12")]
#[test]
fn retained_results_preserve_source_claims_all_causes_and_record_order() {
    use quire_rs::evidence::{ActResult, BackingInput, CauseCode, RecordClaim, RecordLocator};
    use quire_rs::obligation::inventory::{inventory, InventoryOptions};
    let root = tmpdir("retained-source-causes");
    write(&root, "spec/FR-001.md", "---\nid: FR-001\ntype: FR\n---\n\n## Acceptance Criteria\n\n| ID | Criteria | Verification |\n|---|---|---|\n| FR-001-AC-1 | Every failure shall retain its cause. | Test (TC-001) |\n");
    write(
        &root,
        "src/lib.rs",
        "#[cfg(test)]\nmod tests {\n#[trace(\"TC-001\", \"FR-001-AC-1\")]\n#[test]\nfn existing() {}\n}\n",
    );
    write(
        &root,
        "spec/tests.md",
        "---\nid: TM-001\ntype: TestMatrix\n---\n\n## Test Cases\n\n| ID | Traces To | Status |\n|---|---|---|\n| TC-001 | FR-001-AC-1 | 🚧 |\n",
    );
    let registry = Registry::load_module(&fixture_module("iso-obligations")).unwrap();
    let model = registry.traceability().unwrap();
    let inventory = inventory(
        &root,
        model,
        &["acceptance-criterion"],
        InventoryOptions::default(),
    );
    let entry = inventory.entries().next().unwrap();
    let spec = Spec::from_path(&root.join("spec"));
    let graph = sym_trace::bind(&extract_tree(&root.join("src")), model);
    let baseline = compute(&spec, &registry, &graph, &root).unwrap();
    assert_eq!(baseline.coverage_matrix[0].criteria[0].binders.len(), 1);
    let cause_data: serde_json::Value = serde_json::from_str(include_str!(
        "../fixtures/corpus_cases/qualified-evidence-cause-cases.json"
    ))
    .unwrap();
    assert!(!cause_data["issue_ref"].as_str().unwrap().is_empty());
    let causes: Vec<CauseCode> = serde_json::from_value(cause_data["cases"].clone()).unwrap();
    assert_eq!(causes.len(), 12);
    let results: Vec<_> = causes
        .iter()
        .map(|cause| ActResult {
            target: Some(entry.identity().into()),
            requested_target: "FR-001-AC-1",
            locator: None,
            cause: *cause,
            detail: "original owner cause and premise",
        })
        .collect();
    let backing = BackingInput {
        inventory: &inventory,
        claims: &[],
        results: &results,
    };
    let report =
        quire_rs::coverage::compute_with_evidence(&spec, &registry, &graph, &root, &backing)
            .unwrap();
    assert_eq!(
        report.coverage_matrix[0].criteria[0].binders,
        baseline.coverage_matrix[0].criteria[0].binders
    );
    assert_eq!(
        report.coverage_matrix[0].criteria[0].status,
        CoverageMatrixStatus::Tagged
    );
    assert!(report.unbacked_rows.is_empty());
    assert_eq!(report.evidence_results.as_ref().unwrap().len(), 12);
    let rejection = quire_rs::evidence::strict_decision(&report, &[]).unwrap_err();
    assert_eq!(
        rejection.evidence.len(),
        12,
        "source backing never erases a requested failed act"
    );
    assert!(rejection.inherited.is_empty());
    let mut reversed = results.clone();
    reversed.reverse();
    let reordered = BackingInput {
        inventory: &inventory,
        claims: &[],
        results: &reversed,
    };
    assert_eq!(
        report.to_json(),
        quire_rs::coverage::compute_with_evidence(&spec, &registry, &graph, &root, &reordered)
            .unwrap()
            .to_json()
    );

    let claim = RecordClaim {
        target: entry.identity().into(),
        locator: RecordLocator {
            record: "record",
            run: "run",
            entry: "entry",
        },
        method: "Test",
        actor: "actor",
        provenance: &["component-only"],
    };
    let duplicate = [claim, claim];
    let backing = BackingInput {
        inventory: &inventory,
        claims: &duplicate,
        results: &[],
    };
    let report =
        quire_rs::coverage::compute_with_evidence(&spec, &registry, &graph, &root, &backing)
            .unwrap();
    assert_eq!(report.record_binders.as_ref().unwrap().len(), 1);
    let conflict = [
        claim,
        RecordClaim {
            actor: "other-actor",
            ..claim
        },
    ];
    let backing = BackingInput {
        inventory: &inventory,
        claims: &conflict,
        results: &[],
    };
    let report =
        quire_rs::coverage::compute_with_evidence(&spec, &registry, &graph, &root, &backing)
            .unwrap();
    assert!(report.record_binders.as_ref().unwrap().is_empty());
    assert!(report
        .evidence_results
        .as_ref()
        .unwrap()
        .iter()
        .all(|result| result.cause == CauseCode::RecordConflict));
    let reversed = [conflict[1], conflict[0]];
    let backing = BackingInput {
        inventory: &inventory,
        claims: &reversed,
        results: &[],
    };
    assert_eq!(
        report.to_json(),
        quire_rs::coverage::compute_with_evidence(&spec, &registry, &graph, &root, &backing)
            .unwrap()
            .to_json()
    );
    fs::remove_dir_all(root).unwrap();
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeBackingCases {
    issue_ref: String,
    authored_rows: String,
    duplicate_rows: String,
    unselected_rows: String,
    rendered_rows: String,
    reordered_rendered_rows: String,
    configuration_rows: String,
}

fn native_backing_cases() -> NativeBackingCases {
    let cases: NativeBackingCases = serde_json::from_str(include_str!(
        "../fixtures/corpus_cases/qualified-evidence-native-boundaries.json"
    ))
    .unwrap();
    assert_eq!(
        cases.issue_ref,
        "https://linear.app/agent-ix/issue/PLAT-1160"
    );
    cases
}

fn native_backing_registry(
    root: &Path,
    model: &quire_rs::traceability::TraceabilityModel,
) -> Registry {
    let mut module: serde_yaml::Value = serde_yaml::from_str(
        &fs::read_to_string(fixture_module("iso-obligations").join("manifest.yaml")).unwrap(),
    )
    .unwrap();
    module["traceability"] = serde_yaml::to_value(model).unwrap();
    write(
        root,
        "module/manifest.yaml",
        &serde_yaml::to_string(&module).unwrap(),
    );
    Registry::load_module(&root.join("module")).unwrap()
}

fn native_backing_document(root: &Path, path: &str, rows: &str) {
    write(root, path, &format!("---\nid: FR-900\ntype: FR\n---\n\n## Acceptance Criteria\n\n| ID | Criteria | Verification |\n|---|---|---|\n{rows}"));
}

#[trace("FR-078-AC-15", "FR-078-AC-16")]
#[test]
fn evidence_population_uses_only_native_selected_observations() {
    use quire_rs::evidence::BackingInput;
    use quire_rs::obligation::inventory::{inventory, InventoryCauseCode, InventoryOptions};
    let cases = native_backing_cases();
    let root = tmpdir("native-selected-only");
    native_backing_document(&root, "spec/selected.md", &cases.authored_rows);
    write(&root, "spec/unselected.md", &format!("---\nid: NFR-900\ntype: NFR\n---\n\n## Measurements\n\n| Metric | Statement | Method |\n|---|---|---|\n{}", cases.unselected_rows));
    let base = Registry::load_module(&fixture_module("iso-obligations")).unwrap();
    let mut model = base.traceability().unwrap().clone();
    let mut unselected = model.obligations[0].clone();
    unselected.name = "unselected-measurement".into();
    unselected.target = None;
    unselected.archetype = Some("NFR".into());
    unselected.section = Some("Measurements".into());
    unselected.id_format = Some("{document}-M-{row}".into());
    unselected.statement_column = "Statement".into();
    unselected.method_column = Some("Method".into());
    model.obligations.push(unselected);
    let registry = native_backing_registry(&root, &model);
    let model = registry.traceability().unwrap();
    let selected = inventory(
        &root,
        model,
        &["acceptance-criterion"],
        InventoryOptions::default(),
    );
    assert!(selected.causes().is_empty());
    let spec = Spec::from_path(&root.join("spec"));
    let graph = sym_trace::SymbolGraph::default();
    let legacy = compute(&spec, &registry, &graph, &root).unwrap();
    assert!(
        legacy
            .diagnostics
            .iter()
            .any(|d| d.declaration == "unselected-measurement"
                && d.reason == "obligation-row-states-nothing"),
        "adverse unselected legacy source really reproduces the finding"
    );
    let backing = BackingInput {
        inventory: &selected,
        claims: &[],
        results: &[],
    };
    let report =
        quire_rs::coverage::compute_with_evidence(&spec, &registry, &graph, &root, &backing)
            .unwrap();
    assert_eq!(report.obligations.len(), 2);
    assert!(
        !report
            .diagnostics
            .iter()
            .any(|d| d.declaration == "unselected-measurement"
                && d.reason == "obligation-row-states-nothing"),
        "evidence mode must not derive skipped-row observations for unselected sources"
    );
    native_backing_document(&root, "spec/selected.md", "| FR-900-AC-1 | | Test |\n");
    let selected = inventory(
        &root,
        model,
        &["acceptance-criterion"],
        InventoryOptions::default(),
    );
    let spec = Spec::from_path(&root.join("spec"));
    let backing = BackingInput {
        inventory: &selected,
        claims: &[],
        results: &[],
    };
    let report =
        quire_rs::coverage::compute_with_evidence(&spec, &registry, &graph, &root, &backing)
            .unwrap();
    assert!(report.obligations.is_empty());
    assert!(
        report
            .inventory_causes
            .as_ref()
            .unwrap()
            .iter()
            .any(|cause| cause.code == InventoryCauseCode::UnusableRow
                && cause.source.as_deref() == Some("acceptance-criterion")),
        "native selected defect causes are retained without the legacy census"
    );
    fs::remove_dir_all(root).unwrap();
}

#[trace("FR-078-AC-9", "FR-078-AC-14", "FR-078-AC-15")]
#[test]
fn criterion_claims_execute_native_lookup_refusals_and_exact_record_queries() {
    use quire_rs::evidence::{
        trace, BackingInput, CauseCode, RecordClaim, RecordLocator, TraceQuery,
    };
    use quire_rs::obligation::inventory::{inventory, InventoryOptions};
    let cases = native_backing_cases();
    let root = tmpdir("native-lookup-refusals");
    native_backing_document(&root, "spec/first.md", &cases.authored_rows);
    // The same display IDs in a second document remain distinct native targets.
    native_backing_document(&root, "spec/second.md", &cases.authored_rows);
    let base = Registry::load_module(&fixture_module("iso-obligations")).unwrap();
    let mut model = base.traceability().unwrap().clone();
    // A second owner source over the same rows also remains a separate identity.
    let mut second_source = model.obligations[0].clone();
    second_source.name = "other-source".into();
    model.obligations.push(second_source);
    let registry = native_backing_registry(&root, &model);
    let model = registry.traceability().unwrap();
    let selected = inventory(
        &root,
        model,
        &["acceptance-criterion", "other-source"],
        InventoryOptions::default(),
    );
    let entries: Vec<_> = selected.entries().collect();
    assert_eq!(entries.len(), 8);
    let claims = [
        RecordClaim {
            target: entries[0].identity().into(),
            locator: RecordLocator {
                record: "record",
                run: "shared-run",
                entry: "entry-one",
            },
            method: "Test",
            actor: "component",
            provenance: &["component-only"],
        },
        RecordClaim {
            target: entries[1].identity().into(),
            locator: RecordLocator {
                record: "record",
                run: "shared-run",
                entry: "entry-two",
            },
            method: "Test",
            actor: "component",
            provenance: &["component-only"],
        },
    ];
    let spec = Spec::from_path(&root.join("spec"));
    let graph = sym_trace::SymbolGraph::default();
    let backing = BackingInput {
        inventory: &selected,
        claims: &claims,
        results: &[],
    };
    let report =
        quire_rs::coverage::compute_with_evidence(&spec, &registry, &graph, &root, &backing)
            .unwrap();
    assert_eq!(report.record_binders.as_ref().unwrap().len(), 2);
    for requirement in &report.coverage_matrix {
        for criterion in &requirement.criteria {
            let identity = criterion.identity.as_ref().unwrap();
            let accepted =
                identity.document == "spec/first.md" && identity.source == "acceptance-criterion";
            assert_eq!(
                criterion.record_binders.as_ref().unwrap().len(),
                usize::from(accepted),
                "same display ID cannot discharge another source or document"
            );
        }
    }
    assert!(trace(&backing, TraceQuery::Target(entries[2].identity().into())).is_empty());
    assert!(trace(&backing, TraceQuery::Target(entries[4].identity().into())).is_empty());
    assert_eq!(trace(&backing, TraceQuery::Run("shared-run")).len(), 2);
    for claim in claims {
        let inverse = trace(&backing, TraceQuery::Record(claim.locator));
        assert_eq!(inverse.len(), 1);
        assert_eq!(inverse, trace(&backing, TraceQuery::Target(claim.target)));
        assert_eq!(inverse[0].locator.entry, claim.locator.entry);
    }
    for locator in [
        RecordLocator {
            record: "other-record",
            ..claims[0].locator
        },
        RecordLocator {
            run: "other-run",
            ..claims[0].locator
        },
        RecordLocator {
            entry: "other-entry",
            ..claims[0].locator
        },
    ] {
        assert!(trace(&backing, TraceQuery::Record(locator)).is_empty());
    }

    // An equal captured selection with a stopped prefix has no second entry:
    // exercise UnknownIdentity through the actual native lookup, not a cause list.
    let partial = inventory(
        &root,
        model,
        &["acceptance-criterion", "other-source"],
        InventoryOptions {
            stop_limit: Some(1),
            ..InventoryOptions::default()
        },
    );
    assert_eq!(partial.selection(), selected.selection());
    let backing = BackingInput {
        inventory: &partial,
        claims: &claims[1..],
        results: &[],
    };
    let report =
        quire_rs::coverage::compute_with_evidence(&spec, &registry, &graph, &root, &backing)
            .unwrap();
    assert!(report.record_binders.as_ref().unwrap().is_empty());
    assert!(report
        .evidence_results
        .as_ref()
        .unwrap()
        .iter()
        .any(|result| result.cause == CauseCode::InvalidTarget));

    let foreign_root = tmpdir("native-foreign-selection");
    native_backing_document(&foreign_root, "spec/first.md", &cases.authored_rows);
    native_backing_document(&foreign_root, "spec/second.md", &cases.authored_rows);
    let foreign = inventory(
        &foreign_root,
        model,
        &["acceptance-criterion", "other-source"],
        InventoryOptions::default(),
    );
    let foreign_claims = [RecordClaim {
        target: foreign.entries().next().unwrap().identity().into(),
        ..claims[0]
    }];
    let backing = BackingInput {
        inventory: &selected,
        claims: &foreign_claims,
        results: &[],
    };
    let report =
        quire_rs::coverage::compute_with_evidence(&spec, &registry, &graph, &root, &backing)
            .unwrap();
    assert!(report.record_binders.as_ref().unwrap().is_empty());
    assert!(report
        .evidence_results
        .as_ref()
        .unwrap()
        .iter()
        .any(|result| result.cause == CauseCode::SelectionMismatch));

    native_backing_document(&root, "spec/first.md", &cases.duplicate_rows);
    let ambiguous = inventory(
        &root,
        model,
        &["acceptance-criterion", "other-source"],
        InventoryOptions::default(),
    );
    let duplicate_claims = [RecordClaim {
        target: ambiguous.entries().next().unwrap().identity().into(),
        ..claims[0]
    }];
    let spec = Spec::from_path(&root.join("spec"));
    let backing = BackingInput {
        inventory: &ambiguous,
        claims: &duplicate_claims,
        results: &[],
    };
    let report =
        quire_rs::coverage::compute_with_evidence(&spec, &registry, &graph, &root, &backing)
            .unwrap();
    assert!(report.record_binders.as_ref().unwrap().is_empty());
    assert!(report
        .evidence_results
        .as_ref()
        .unwrap()
        .iter()
        .any(|result| result.cause == CauseCode::AmbiguousTarget));
    assert!(trace(&backing, TraceQuery::Record(duplicate_claims[0].locator)).is_empty());
    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(foreign_root).unwrap();
}

#[trace("FR-078-AC-9", "FR-078-AC-14", "FR-078-AC-15")]
#[test]
fn retained_native_origins_preserve_unmapped_rows_and_refuse_rendered_reordering() {
    use quire_rs::evidence::{
        trace, BackingInput, CauseCode, RecordClaim, RecordLocator, TraceQuery,
    };
    use quire_rs::obligation::inventory::{inventory, CriterionOrigin, InventoryOptions};
    use quire_rs::traceability::CombinatorialColumns;
    let cases = native_backing_cases();
    let root = tmpdir("native-rendered-origins");
    let body = |rows: &str| {
        format!("---\nid: FR-900\ntype: FR\n---\n\n## Measurements\n\n| Metric | Method |\n|---|---|\n{rows}\n## Configuration Dimensions\n\n| Dimension | Values | Excludes |\n|---|---|---|\n{}", cases.configuration_rows)
    };
    write(&root, "spec/FR-900.md", &body(&cases.rendered_rows));
    let base = Registry::load_module(&fixture_module("iso-obligations")).unwrap();
    let mut model = base.traceability().unwrap().clone();
    let mut rendered = model.obligations[0].clone();
    rendered.name = "rendered".into();
    rendered.target = None;
    rendered.archetype = Some("FR".into());
    rendered.section = Some("Measurements".into());
    rendered.id_format = Some("{document}-M-{row}".into());
    rendered.statement_column = "Metric".into();
    rendered.method_column = Some("Method".into());
    let mut combinatorial = rendered.clone();
    combinatorial.name = "configuration-space".into();
    combinatorial.section = Some("Configuration Dimensions".into());
    combinatorial.id_format = Some("{document}-COMB".into());
    combinatorial.statement_column = "Dimension".into();
    combinatorial.method_column = None;
    combinatorial.combinatorial = Some(CombinatorialColumns {
        dimension_column: "Dimension".into(),
        values_column: "Values".into(),
        excludes_column: Some("Excludes".into()),
        strength: 2,
    });
    model.obligations = vec![rendered, combinatorial];
    let registry = native_backing_registry(&root, &model);
    let model = registry.traceability().unwrap();
    let selected = inventory(
        &root,
        model,
        &["rendered", "configuration-space"],
        InventoryOptions::default(),
    );
    let entries: Vec<_> = selected.entries().collect();
    assert_eq!(entries.len(), 3);
    assert!(matches!(
        entries[0].identity().origin(),
        CriterionOrigin::RenderedRow { row: 1, .. }
    ));
    assert!(matches!(
        entries[2].identity().origin(),
        CriterionOrigin::CombinatorialTable { .. }
    ));
    assert!(
        entries
            .iter()
            .all(|entry| entry.obligation().target_ids.is_empty()),
        "unmapped owner criteria stay in the population"
    );
    let claims = [RecordClaim {
        target: entries[0].identity().into(),
        locator: RecordLocator {
            record: "record",
            run: "run",
            entry: "rendered-one",
        },
        method: "Analysis",
        actor: "component",
        provenance: &["component-only"],
    }];
    let graph = sym_trace::SymbolGraph::default();
    let spec = Spec::from_path(&root.join("spec"));
    let backing = BackingInput {
        inventory: &selected,
        claims: &claims,
        results: &[],
    };
    let report =
        quire_rs::coverage::compute_with_evidence(&spec, &registry, &graph, &root, &backing)
            .unwrap();
    let criteria = &report.coverage_matrix[0].criteria;
    assert_eq!(criteria.len(), 3);
    assert_eq!(
        criteria[0].identity.as_ref().unwrap().origin,
        "rendered-row"
    );
    assert_eq!(criteria[0].identity.as_ref().unwrap().row, Some(1));
    assert_eq!(criteria[0].record_binders.as_ref().unwrap().len(), 1);
    assert_eq!(criteria[1].record_binders, Some(vec![]));
    assert_eq!(
        criteria[2].identity.as_ref().unwrap().origin,
        "combinatorial-table"
    );
    assert_eq!(criteria[2].identity.as_ref().unwrap().row, None);
    assert_eq!(
        trace(&backing, TraceQuery::Record(claims[0].locator)),
        trace(&backing, TraceQuery::Target(claims[0].target))
    );
    // The native combinatorial source declares no method. Supplying its exact
    // identity must retain that origin and refuse an invented method, not infer Test.
    assert!(entries[2].obligation().method.is_none());
    let combinatorial_claims = [RecordClaim {
        target: entries[2].identity().into(),
        method: "Analysis",
        ..claims[0]
    }];
    let backing = BackingInput {
        inventory: &selected,
        claims: &combinatorial_claims,
        results: &[],
    };
    let report =
        quire_rs::coverage::compute_with_evidence(&spec, &registry, &graph, &root, &backing)
            .unwrap();
    let result = report
        .evidence_results
        .as_ref()
        .unwrap()
        .iter()
        .find(|r| r.cause == CauseCode::QualificationRejected)
        .unwrap();
    assert_eq!(
        result.target.as_ref().unwrap().origin,
        "combinatorial-table"
    );
    assert!(report.record_binders.as_ref().unwrap().is_empty());

    write(
        &root,
        "spec/FR-900.md",
        &body(&cases.reordered_rendered_rows),
    );
    let reordered = inventory(
        &root,
        model,
        &["rendered", "configuration-space"],
        InventoryOptions::default(),
    );
    let fresh = reordered.entries().next().unwrap();
    assert_eq!(fresh.obligation().id, entries[0].obligation().id);
    assert_eq!(fresh.row(), entries[0].row());
    assert_ne!(
        fresh.obligation().statement,
        entries[0].obligation().statement
    );
    let spec = Spec::from_path(&root.join("spec"));
    let backing = BackingInput {
        inventory: &reordered,
        claims: &claims,
        results: &[],
    };
    let report =
        quire_rs::coverage::compute_with_evidence(&spec, &registry, &graph, &root, &backing)
            .unwrap();
    assert!(report.record_binders.as_ref().unwrap().is_empty());
    assert!(report
        .evidence_results
        .as_ref()
        .unwrap()
        .iter()
        .any(|r| r.cause == CauseCode::SelectionMismatch));
    assert!(trace(&backing, TraceQuery::Record(claims[0].locator)).is_empty());
    let fresh_claims = [RecordClaim {
        target: fresh.identity().into(),
        ..claims[0]
    }];
    let backing = BackingInput {
        inventory: &reordered,
        claims: &fresh_claims,
        results: &[],
    };
    let report =
        quire_rs::coverage::compute_with_evidence(&spec, &registry, &graph, &root, &backing)
            .unwrap();
    assert_eq!(
        report.coverage_matrix[0].criteria[0]
            .record_binders
            .as_ref()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        trace(&backing, TraceQuery::Record(fresh_claims[0].locator)).len(),
        1
    );
    fs::remove_dir_all(root).unwrap();
}

// FR-082: requirement-level criticality on `coverage_matrix` entries.

fn criticality_doc(id: &str, frontmatter_extra: &str, priority: &str) -> String {
    format!(
        "---\nid: {id}\ntype: FR\ntitle: A requirement\n{frontmatter_extra}---\n\n\
         ## Acceptance Criteria\n\n\
         | ID | Criteria | Verification | Priority |\n|----|----------|--------------|----------|\n\
         | {id}-AC-1 | The system shall do it. | Test (TC-001) | {priority} |\n"
    )
}

fn entry(report: &CoverageReport, document: &str) -> serde_json::Value {
    let value = serde_json::to_value(report).expect("serialize report");
    value["coverage_matrix"]
        .as_array()
        .expect("coverage_matrix")
        .iter()
        .find(|e| e["document"] == document)
        .unwrap_or_else(|| panic!("no entry for {document}"))
        .clone()
}

#[trace(
    "FR-082-AC-1",
    "FR-082-AC-2",
    "FR-082-AC-3",
    "FR-082-AC-4",
    "FR-082-AC-6"
)]
#[test]
fn fr082_entries_carry_their_own_value_as_written_and_absent_when_missing() {
    let root = tmpdir("fr082-values");
    write(
        &root,
        "spec/FR-001.md",
        &criticality_doc("FR-001", "criticality: low\n", ""),
    );
    write(
        &root,
        "spec/FR-002.md",
        &criticality_doc("FR-002", "criticality: medium\n", ""),
    );
    write(
        &root,
        "spec/FR-003.md",
        &criticality_doc("FR-003", "criticality: high\n", ""),
    );
    write(&root, "spec/FR-004.md", &criticality_doc("FR-004", "", ""));
    write(
        &root,
        "spec/FR-005.md",
        &criticality_doc("FR-005", "criticality: unassigned\n", ""),
    );
    write(
        &root,
        "spec/FR-006.md",
        &criticality_doc("FR-006", "criticality: High\n", ""),
    );
    write(&root, "src/lib.rs", "//! fixture\n");
    let report = report_over(&root, "obligations");

    assert_eq!(entry(&report, "spec/FR-001.md")["criticality"], "low");
    assert_eq!(entry(&report, "spec/FR-002.md")["criticality"], "medium");
    assert_eq!(entry(&report, "spec/FR-003.md")["criticality"], "high");
    // AC-2: no key at all, not null, empty or `unassigned`.
    assert!(entry(&report, "spec/FR-004.md")
        .as_object()
        .unwrap()
        .get("criticality")
        .is_none());
    // AC-3: a literal `unassigned` is a present value.
    assert_eq!(
        entry(&report, "spec/FR-005.md")["criticality"],
        "unassigned"
    );
    // AC-6: out-of-enumeration values are carried verbatim.
    assert_eq!(entry(&report, "spec/FR-006.md")["criticality"], "High");

    let _ = fs::remove_dir_all(&root);
}

#[trace("FR-082-AC-5")]
#[test]
fn fr082_criticality_is_never_inferred_across_relationships() {
    let root = tmpdir("fr082-inference");
    write(
        &root,
        "spec/FR-001.md",
        &criticality_doc("FR-001", "criticality: high\n", ""),
    );
    for (n, verb) in ["traces_to", "satisfies", "implements", "refines"]
        .iter()
        .enumerate()
    {
        let id = format!("FR-1{n:02}");
        write(
            &root,
            &format!("spec/{id}.md"),
            &criticality_doc(
                &id,
                &format!("relationships:\n  - target: \"ix://x/y/FR-001\"\n    type: \"{verb}\"\n"),
                "",
            ),
        );
    }
    write(&root, "src/lib.rs", "//! fixture\n");
    let report = report_over(&root, "obligations");
    assert_eq!(entry(&report, "spec/FR-001.md")["criticality"], "high");
    for n in 0..4 {
        assert!(entry(&report, &format!("spec/FR-1{n:02}.md"))
            .as_object()
            .unwrap()
            .get("criticality")
            .is_none());
    }
    let _ = fs::remove_dir_all(&root);
}

#[trace("FR-082-AC-7")]
#[test]
fn fr082_requirement_and_obligation_criticality_never_fill_each_other() {
    let root = tmpdir("fr082-levels");
    // Frontmatter value, no obligation criticality.
    write(
        &root,
        "spec/FR-001.md",
        &criticality_doc("FR-001", "criticality: high\n", ""),
    );
    // No frontmatter value, obligation criticality P1.
    write(
        &root,
        "spec/FR-002.md",
        &criticality_doc("FR-002", "", "P1"),
    );
    write(&root, "src/lib.rs", "//! fixture\n");
    let report = report_over(&root, "obligations");
    let registry = Registry::load_module(&fixture_module("obligations")).expect("load");
    let model = registry.traceability().cloned().unwrap_or_default();
    let (obligations, _) = quire_rs::obligation::derive(
        &Spec::from_path(&root.join("spec")),
        &root.join("spec"),
        &model,
    );
    let by_doc = |d: &str| obligations.iter().find(|o| o.document == d).unwrap();
    assert_eq!(by_doc("FR-001.md").criticality, None);
    assert_eq!(by_doc("FR-002.md").criticality.as_deref(), Some("P1"));
    assert_eq!(entry(&report, "spec/FR-001.md")["criticality"], "high");
    assert!(entry(&report, "spec/FR-002.md")
        .as_object()
        .unwrap()
        .get("criticality")
        .is_none());
    let _ = fs::remove_dir_all(&root);
}

#[trace("FR-082-AC-8", "FR-082-AC-9")]
#[test]
fn fr082_matrix_is_deterministic_and_leaves_documents_untouched() {
    let root = tmpdir("fr082-determinism");
    write(
        &root,
        "spec/FR-001.md",
        &criticality_doc("FR-001", "criticality: low\n", ""),
    );
    write(&root, "spec/FR-002.md", &criticality_doc("FR-002", "", ""));
    write(&root, "src/lib.rs", "//! fixture\n");
    let before = (
        fs::read(root.join("spec/FR-001.md")).unwrap(),
        fs::read(root.join("spec/FR-002.md")).unwrap(),
    );
    let a = report_over(&root, "obligations");
    let b = report_over(&root, "obligations");
    assert_eq!(a.to_json(), b.to_json());
    let after = (
        fs::read(root.join("spec/FR-001.md")).unwrap(),
        fs::read(root.join("spec/FR-002.md")).unwrap(),
    );
    assert_eq!(before, after);
    let _ = fs::remove_dir_all(&root);
}
