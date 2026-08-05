//! FR-049 — verification-reference integrity (TC-724..TC-731).
//!
//! Bundles are built on disk per test so the auxiliary-source harvest (a
//! `tests.md` the corpus walk excludes) exercises the real filesystem path.

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

/// A Test Matrix at the bundle root — a declared auxiliary trace source that
/// the corpus walk skips as a non-artifact.
fn tests_matrix(rows: &[(&str, &str)]) -> String {
    let mut out = String::from("# Test Matrix\n\n## Test Cases\n\n| ID | Traces To | Status |\n|----|-----------|--------|\n");
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

// TC-724 (FR-049-AC-1): references that resolve — one to a declared
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

// TC-725 (FR-049-AC-2): an unresolved id yields one finding carrying the
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

// TC-726 (FR-049-AC-3): the finding is posture-degradable — error under
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

// TC-727 (FR-049-AC-4): the pattern and column come from the declaration — a
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
        "# Check Register\n\n## Checks\n\n| Check | Covers | State |\n|-------|--------|-------|\n\
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

// TC-728 (FR-049-AC-5): a declared auxiliary source outside the corpus walk
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

// TC-729 (FR-049-AC-6): with no declared model, the check emits nothing.
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

// TC-730 (FR-049-AC-7): a cell with several annotations resolves each id
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

// TC-731 (FR-049-AC-8, Property): repeated validation yields the same findings
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
