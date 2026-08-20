//! FR-049 — verification-reference integrity (TC-724..TC-731).
//!
//! Bundles are built on disk per test so the walk that acquires the matrix
//! exercises the real filesystem path. Before CR-062 the matrix was read
//! off-corpus by `document:` binding and needed no frontmatter; now it is an
//! ordinary corpus document of archetype `TestMatrix`.

use std::fs;
use std::path::{Path, PathBuf};

use quire_rs::{validate_bundle_at, BundleFinding, BundlePosture, BundleReport, Registry};

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
        "quire-rs-trace-refs-{}-{suffix}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).expect("mkdir");
    p
}

/// An FR document whose `Acceptance Criteria` table carries `verification`
/// cells — the referencing side of the ISO declaration.
fn fr_document(id: &str, rows: &[(&str, &str)]) -> String {
    let mut out = format!("---\nid: {id}\ntype: FR\ntitle: A requirement\n---\n\n## Acceptance Criteria\n\n| ID | Criteria | Verification |\n|----|----------|--------------|\n");
    for (ac, verification) in rows {
        out.push_str(&format!(
            "| {ac} | The system shall do it. | {verification} |\n"
        ));
    }
    out
}

/// A Test Matrix at the bundle root, reached by archetype like any other
/// corpus document (CR-062).
fn tests_matrix(rows: &[(&str, &str)]) -> String {
    let mut out = String::from("---\nid: TM-001\ntype: TestMatrix\ntitle: Test Matrix\n---\n\n# Test Matrix\n\n## Test Cases\n\n| ID | Traces To | Status |\n|----|-----------|--------|\n");
    for (tc, traces_to) in rows {
        out.push_str(&format!("| {tc} | {traces_to} | ✅ |\n"));
    }
    out
}

fn write(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("mkdir");
    }
    fs::write(path, body).expect("write");
}

fn dangling(report: &BundleReport) -> Vec<&BundleFinding> {
    report
        .errors
        .iter()
        .chain(report.warnings.iter())
        .filter(|f| f.reason == "dangling-trace-reference")
        .collect()
}

fn validate(root: &Path, module: &str, posture: BundlePosture) -> BundleReport {
    let registry = Registry::load_module(&fixture_module(module)).expect("load module");
    validate_bundle_at(root, &registry, posture)
}

// TC-724, FR-049-AC-1: references that resolve — one to a declared
// trace-source row, one to a TC document in the bundle — yield no finding.
#[test]
fn tc724_resolved_references_are_clean() {
    let root = tmpdir("724");
    write(
        &root,
        "FR-001.md",
        &fr_document(
            "FR-001",
            &[
                ("FR-001-AC-1", "Test (TC-001)"),
                ("FR-001-AC-2", "Test (TC-900)"),
            ],
        ),
    );
    // TC-001 is minted by the auxiliary matrix…
    write(
        &root,
        "tests.md",
        &tests_matrix(&[("TC-001", "FR-001-AC-1")]),
    );
    // …and TC-900 is a TC document in the bundle.
    write(
        &root,
        "TC-900.md",
        "---\nid: TC-900\ntype: TC\ntitle: A test case\n---\n\n## Description\n\nA case.\n",
    );

    let report = validate(&root, "iso", BundlePosture::Okf);
    assert!(
        dangling(&report).is_empty(),
        "unexpected findings: {:?}",
        dangling(&report)
    );
}

// TC-725, FR-049-AC-2: an unresolved id yields one finding carrying the
// document path and the id.
#[test]
fn tc725_unresolved_reference_is_reported() {
    let root = tmpdir("725");
    write(
        &root,
        "FR-001.md",
        &fr_document("FR-001", &[("FR-001-AC-1", "Test (TC-404)")]),
    );
    write(
        &root,
        "tests.md",
        &tests_matrix(&[("TC-001", "FR-001-AC-1")]),
    );

    let report = validate(&root, "iso", BundlePosture::Okf);
    let findings = dangling(&report);
    assert_eq!(findings.len(), 1, "findings: {findings:?}");
    assert!(findings[0].message.contains("TC-404"));
    assert!(findings[0].message.contains("verification"));
    assert!(findings[0].path.ends_with("FR-001.md"));
}

// TC-726, FR-049-AC-3: the finding is posture-degradable — error under
// Strict, warning under Okf.
#[test]
fn tc726_posture_degradable() {
    let root = tmpdir("726");
    write(
        &root,
        "FR-001.md",
        &fr_document("FR-001", &[("FR-001-AC-1", "Test (TC-404)")]),
    );
    write(
        &root,
        "tests.md",
        &tests_matrix(&[("TC-001", "FR-001-AC-1")]),
    );

    let strict = validate(&root, "iso", BundlePosture::Strict);
    assert_eq!(
        strict
            .errors
            .iter()
            .filter(|f| f.reason == "dangling-trace-reference")
            .count(),
        1
    );
    assert!(strict
        .warnings
        .iter()
        .all(|f| f.reason != "dangling-trace-reference"));

    let okf = validate(&root, "iso", BundlePosture::Okf);
    assert_eq!(
        okf.warnings
            .iter()
            .filter(|f| f.reason == "dangling-trace-reference")
            .count(),
        1
    );
    assert!(okf
        .errors
        .iter()
        .all(|f| f.reason != "dangling-trace-reference"));
}

// TC-727, FR-049-AC-4: the pattern and column come from the declaration — a
// module with a different vocabulary resolves by its own model.
#[test]
fn tc727_pattern_and_column_are_module_data() {
    let root = tmpdir("727");
    // The `alt` module declares archetype `Rule`, section `Clauses`, column
    // `Evidence`, and the pattern `checked by (C-\d+)`.
    write(
        &root,
        "R-001.md",
        "---\nid: R-001\ntype: Rule\ntitle: A rule\n---\n\n## Clauses\n\n\
         | Clause | Evidence |\n|--------|----------|\n\
         | R-001-C-1 | checked by C-001 |\n\
         | R-001-C-2 | checked by C-404 |\n",
    );
    write(
        &root,
        "checks.md",
        // The title heading must not repeat the declared section name —
        // `section()` returns the first heading that matches, and a level-1
        // `# Checks` would shadow the level-2 table section.
        "---\nid: CR-001\ntype: CheckRegister\ntitle: Check Register\n---\n\n\
         # Check Register\n\n## Checks\n\n| Check | Covers | State |\n|-------|--------|-------|\n\
         | C-001 | R-001-C-1 | done |\n",
    );

    let report = validate(&root, "alt", BundlePosture::Okf);
    let findings = dangling(&report);
    assert_eq!(findings.len(), 1, "findings: {findings:?}");
    assert!(findings[0].message.contains("C-404"));
    assert!(findings[0].message.contains("evidence"));

    // The ISO declaration finds nothing here: neither its column nor its
    // pattern appears in this bundle — no ISO-specific behaviour leaked into
    // the engine.
    assert!(dangling(&validate(&root, "iso", BundlePosture::Okf)).is_empty());
}

// TC-728, FR-049-AC-5: a declared auxiliary source outside the corpus walk
// contributes its minted ids to the resolution set.
#[test]
fn tc728_auxiliary_source_contributes_ids() {
    let root = tmpdir("728");
    write(
        &root,
        "FR-001.md",
        &fr_document("FR-001", &[("FR-001-AC-1", "Test (TC-777)")]),
    );

    // Without the matrix, TC-777 resolves to nothing.
    let before = validate(&root, "iso", BundlePosture::Okf);
    assert_eq!(dangling(&before).len(), 1);

    // The matrix is skipped by the corpus walk, yet the declared auxiliary
    // source harvest picks its rows up.
    write(
        &root,
        "tests.md",
        &tests_matrix(&[("TC-777", "FR-001-AC-1")]),
    );
    let after = validate(&root, "iso", BundlePosture::Okf);
    assert!(
        dangling(&after).is_empty(),
        "findings: {:?}",
        dangling(&after)
    );
}

// TC-729, FR-049-AC-6: with no declared model, the check emits nothing.
#[test]
fn tc729_no_model_no_findings() {
    let root = tmpdir("729");
    write(
        &root,
        "FR-001.md",
        &fr_document("FR-001", &[("FR-001-AC-1", "Test (TC-404)")]),
    );

    // A module directory declaring no `traceability:` section at all.
    let module = tmpdir("729-module");
    fs::write(
        module.join("manifest.yaml"),
        "name: no-model\nartifact_types:\n- name: FR\n",
    )
    .expect("write manifest");
    let registry = Registry::load_module(&module).expect("load module");
    assert!(registry.traceability().is_none());

    for posture in [BundlePosture::Strict, BundlePosture::Okf] {
        let report = validate_bundle_at(&root, &registry, posture);
        assert!(dangling(&report).is_empty());
    }
}

// TC-730, FR-049-AC-7: a cell with several annotations resolves each id
// independently and reports only the unresolved ones.
#[test]
fn tc730_multiple_annotations_resolve_independently() {
    let root = tmpdir("730");
    write(
        &root,
        "FR-001.md",
        &fr_document(
            "FR-001",
            &[("FR-001-AC-1", "Test (TC-001), Test (TC-404), Test (TC-002)")],
        ),
    );
    write(
        &root,
        "tests.md",
        &tests_matrix(&[("TC-001", "FR-001-AC-1"), ("TC-002", "FR-001-AC-1")]),
    );

    let report = validate(&root, "iso", BundlePosture::Okf);
    let findings = dangling(&report);
    assert_eq!(findings.len(), 1, "findings: {findings:?}");
    assert!(findings[0].message.contains("TC-404"));
    assert!(!findings[0].message.contains("TC-001"));
}

// TC-731, FR-049-AC-8 (Property): repeated validation yields the same findings
// in the same order.
#[test]
fn tc731_findings_are_deterministic() {
    let root = tmpdir("731");
    for n in 1..=6 {
        write(
            &root,
            &format!("FR-{n:03}.md"),
            &fr_document(
                &format!("FR-{n:03}"),
                &[
                    (&format!("FR-{n:03}-AC-1"), "Test (TC-404)"),
                    (&format!("FR-{n:03}-AC-2"), "Test (TC-405), Test (TC-001)"),
                ],
            ),
        );
    }
    write(
        &root,
        "tests.md",
        &tests_matrix(&[("TC-001", "FR-001-AC-1")]),
    );

    let render = || {
        let report = validate(&root, "iso", BundlePosture::Okf);
        dangling(&report)
            .iter()
            .map(|f| format!("{}|{}", f.path.display(), f.message))
            .collect::<Vec<String>>()
    };
    let first = render();
    assert_eq!(first.len(), 12, "two unresolved ids per FR document");
    for _ in 0..8 {
        assert_eq!(first, render(), "findings must be order-stable");
    }
}

// TC-760, FR-050-AC-12 (CR-015): declared normalizations apply before ids are
// read, and only when declared. The `normalizing` and `plain` fixtures carry
// the same declaration; only the two flags differ, so each half of this test is
// a controlled comparison.
#[test]
fn tc760_declared_cell_normalization() {
    let root = tmpdir("760");
    write(
        &root,
        "FR-001.md",
        &fr_document(
            "FR-001",
            &[
                // A range whose *middle* member is absent from the matrix:
                // expansion introduces the reference, so only the normalizing
                // module can see it dangle.
                ("FR-001-AC-1", "TC-001..TC-003"),
                // A qualifier naming an absent id: stripping the parenthetical
                // stops it becoming a reference at all.
                ("FR-001-AC-2", "TC-001 (superseded by TC-404)"),
            ],
        ),
    );
    write(
        &root,
        "tests.md",
        &tests_matrix(&[("TC-001", "FR-001-AC-1"), ("TC-003", "FR-001-AC-1")]),
    );

    let ids = |module: &str| -> Vec<String> {
        let report = validate(&root, module, BundlePosture::Okf);
        let mut out: Vec<String> = dangling(&report)
            .iter()
            .filter_map(|f| f.message.split('\'').nth(1).map(|id| id.to_string()))
            .collect();
        out.sort();
        out.dedup();
        out
    };

    // Both flags on: the range expands (TC-002 is missing → dangles) and the
    // parenthetical is stripped (TC-404 never becomes a reference).
    assert_eq!(ids("normalizing"), vec!["TC-002".to_string()]);

    // Both flags off: the range is read as its literal endpoints (both exist,
    // nothing dangles) and the qualifier's id IS read (TC-404 dangles).
    assert_eq!(ids("plain"), vec!["TC-404".to_string()]);
}

// TC-814, FR-049-AC-9 (CR-056, amended CR-062): the two-root split, which now
// rests entirely on `exclude:`. A glob is authored against the repository scope
// — `exclude: ["spec/fixtures/**"]` — so it must be matched against the
// **reference** root. Matched against the document root instead, the same glob
// addresses `spec/spec/fixtures/**`, excludes nothing, and a fixture matrix's
// ids mint as if they were real.
//
// This was always the more fragile half, and since CR-062 it is the only one:
// the path-bound half un-minted loudly (a reference dangled), while a lapsed
// exclusion *adds* ids and every reference still resolves. Deleting `document:`
// removed the loud half of the risk, not the quiet one.
#[test]
fn tc814_exclude_globs_resolve_against_the_reference_root() {
    let root = tmpdir("814-exclude");

    let module = root.join("m");
    fs::create_dir_all(&module).expect("mkdir module");
    fs::write(
        module.join("manifest.yaml"),
        "name: m\nmanifest_version: 1.0.0\nversion: 0.0.1\nartifact_types:\n\
         - name: FR\n\
         traceability:\n  trace_targets:\n  - name: test-case\n\
         \x20   archetype: FR\n    section: Acceptance Criteria\n\
         \x20   id_column: ID\n    exclude: ['spec/fixtures/**']\n\
         \x20 document_references:\n  - name: verification\n    archetype: FR\n\
         \x20   section: Acceptance Criteria\n    column: Verification\n\
         \x20   row_id_column: ID\n    pattern: '\\((TC-\\d+)\\)'\n\
         \x20   targets: [test-case]\n    exclude: ['spec/fixtures/**']\n",
    )
    .expect("write manifest");

    // A real requirement, and a deliberately broken fixture under the
    // excluded subtree whose reference cannot resolve.
    write(
        &root,
        "spec/FR-001.md",
        &fr_document("FR-001", &[("FR-001-AC-1", "ok")]),
    );
    write(
        &root,
        "spec/fixtures/FR-900.md",
        &fr_document("FR-900", &[("FR-900-AC-1", "Test (TC-404)")]),
    );

    let registry = Registry::load_module(&module).expect("load module");
    let spec_root = root.join("spec");
    let spec = quire_rs::Spec::from_path(&spec_root);

    // Two roots stated separately: the glob is authored against the scope, so
    // the fixture is excluded and its dangling reference is never read.
    let split = quire_rs::validate_bundle(&spec, &registry, BundlePosture::Okf, &spec_root, &root);
    assert!(
        dangling(&split).is_empty(),
        "the excluded fixture must contribute no reference: {:?}",
        dangling(&split)
    );

    // Conflated roots: the same glob now addresses `spec/spec/fixtures/**`,
    // matches nothing, and the fixture is read as a real document.
    let conflated =
        quire_rs::validate_bundle(&spec, &registry, BundlePosture::Okf, &spec_root, &spec_root);
    assert_eq!(
        dangling(&conflated).len(),
        1,
        "conflating the roots should lapse the exclusion: {:?}",
        dangling(&conflated)
    );
}

// TC-889, FR-057-AC-6: the severity registry is per check, so switching one
// pack off leaves its siblings reporting. The trace pack is the one P2's new
// checks will sit beside, so the independence is worth pinning here rather than
// only over the packs that happen to share a file.
#[test]
fn tc889_switching_one_pack_off_leaves_siblings_reporting() {
    use quire_rs::grammar::{GrammarSeverityLevel, GrammarSeverityMap};

    let root = tmpdir("889");
    // A trace reference to a TC nothing mints…
    write(
        &root,
        "FR-001.md",
        &fr_document("FR-001", &[("FR-001-AC-1", "Test (TC-404)")]),
    );
    write(
        &root,
        "tests.md",
        &tests_matrix(&[("TC-001", "FR-001-AC-1")]),
    );
    // …and, separately, a dangling `ix://` reference.
    write(
        &root,
        "FR-002.md",
        &format!(
            "{}\nSee [missing](ix://o/r/MISSING).\n",
            fr_document("FR-002", &[("FR-002-AC-1", "Test (TC-001)")])
        ),
    );

    let registry = Registry::load_module(&fixture_module("iso")).expect("load module");
    let with_refs_off = {
        let mut map = GrammarSeverityMap::new();
        map.insert(
            "refs:dangling-reference".to_string(),
            GrammarSeverityLevel::Off,
        );
        registry.with_grammar_severity(map)
    };

    let baseline = validate_bundle_at(&root, &registry, BundlePosture::Okf);
    let reasons = |r: &BundleReport| -> Vec<String> {
        r.errors
            .iter()
            .chain(r.warnings.iter())
            .map(|f| f.reason.to_string())
            .collect()
    };
    assert!(
        reasons(&baseline).iter().any(|r| r == "dangling-reference"),
        "fixture must produce both: {:?}",
        reasons(&baseline)
    );
    assert!(
        reasons(&baseline)
            .iter()
            .any(|r| r == "dangling-trace-reference"),
        "fixture must produce both: {:?}",
        reasons(&baseline)
    );

    let scoped = validate_bundle_at(&root, &with_refs_off, BundlePosture::Okf);
    assert!(
        !reasons(&scoped).iter().any(|r| r == "dangling-reference"),
        "refs pack was switched off: {:?}",
        reasons(&scoped)
    );
    assert_eq!(
        dangling(&scoped).len(),
        dangling(&baseline).len(),
        "the trace pack is untouched"
    );

    fs::remove_dir_all(&root).ok();
}
