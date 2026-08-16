//! The FR-050-AC-7 byte-identity **gate** (CR-057).
//!
//! CR-045, CR-047 and CR-049 each rest their correctness argument on one
//! property — "the coverage JSON is byte-identical to the pre-program
//! baseline" — and nothing backed it. There was no checked-in baseline, no
//! script, no make target, no CI step. The claim was verified once, by hand,
//! against a corpus (`spec-artifacts-process`) that no gate in this repo has
//! ever referenced.
//!
//! `tc738_report_json_is_byte_identical` is not that gate: it runs the same
//! engine twice over a synthetic fixture, which is determinism. It passes
//! unchanged when the engine starts reconciling something different.
//!
//! This is the gate. `tests/fixtures/coverage_baseline/` is a corpus chosen to
//! exercise the whole reconciliation surface at once — archetype-minted ids,
//! an auxiliary matrix, an `exclude:` glob, all three status classes, the
//! `no_source_symbol` exemption, an untracked symbol, a dangling reference, an
//! undeclared archetype, and criteria classification — and `expected.json` is
//! its report, checked in.
//!
//! **Regenerating is legitimate and must be visible.** Unlike the parser
//! golden (TC-821), a change to what coverage reconciles is a real change
//! someone may intend; what must never happen is that it lands unnoticed. So
//! the baseline is regenerated deliberately, `make coverage-baseline-update`,
//! and the diff goes in the pull request — the `check_unsafe_comments.sh
//! --update-baseline` pattern.

use std::path::{Path, PathBuf};

use quire_rs::coverage::compute;
use quire_rs::symbols::{extract_tree_excluding, trace};
use quire_rs::{Registry, Spec};

fn fixture(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/coverage_baseline")
        .join(rel)
}

/// The report, exactly as `quire coverage` would render it: the two roots
/// derived from one scope, the code walk excluding the document root.
fn render_report() -> String {
    let scope = fixture("scope");
    let document_root = scope.join("spec");

    let registry = Registry::load_module(&fixture("module")).expect("load baseline module");
    let spec = Spec::from_path(&document_root);
    let extraction = extract_tree_excluding(&scope, &[Path::new("spec")]);
    let model = registry.traceability().cloned().unwrap_or_default();
    let graph = trace::bind(&extraction, &model);

    let report = compute(&spec, &registry, &graph, &scope).expect("model declared");
    format!("{}\n", report.to_json())
}

/// TC-824 (FR-050-AC-7, CR-057): the coverage report over the baseline corpus
/// is byte-identical to the checked-in snapshot.
#[test]
fn tc824_coverage_report_matches_the_checked_in_baseline() {
    let actual = render_report();
    let path = fixture("expected.json");

    if std::env::var_os("QUIRE_UPDATE_COVERAGE_BASELINE").is_some() {
        std::fs::write(&path, &actual).expect("write baseline");
        eprintln!(
            "coverage baseline REWRITTEN at {} — the diff belongs in the pull request",
            path.display()
        );
        return;
    }

    let expected = std::fs::read_to_string(&path).expect("read baseline");
    assert_eq!(
        actual, expected,
        "the coverage report changed against tests/fixtures/coverage_baseline/expected.json.\n\
         This is the property CR-045, CR-047 and CR-049 rest on (FR-050-AC-7).\n\
         If the change is intended, regenerate with `make coverage-baseline-update` \
         and put the diff in the pull request."
    );
}

/// TC-824 (FR-050-AC-7, CR-057): and the baseline corpus still exercises what
/// it was chosen to exercise. A gate over a corpus that has quietly stopped
/// producing findings passes forever while measuring nothing — the failure
/// mode of every baseline nobody re-reads.
#[test]
fn tc824_the_baseline_corpus_still_exercises_the_surface() {
    let scope = fixture("scope");
    let document_root = scope.join("spec");
    let registry = Registry::load_module(&fixture("module")).expect("load baseline module");
    let spec = Spec::from_path(&document_root);
    let extraction = extract_tree_excluding(&scope, &[Path::new("spec")]);
    let model = registry.traceability().cloned().unwrap_or_default();
    let graph = trace::bind(&extraction, &model);
    let report = compute(&spec, &registry, &graph, &scope).expect("model declared");

    assert!(report.totals.total > 0, "the model must mint");
    assert!(report.totals.backed > 0, "something must be backed");
    assert!(
        report.totals.backed < report.totals.total,
        "and something must not be — a fully-backed corpus exercises no gap"
    );
    assert!(!report.unbacked_rows.is_empty(), "unbacked rows");
    assert!(!report.status_lies.is_empty(), "status lies");
    assert!(
        !report.no_symbol_rows.is_empty(),
        "the no_source_symbol exemption"
    );
    assert!(
        !report.untracked_symbols.is_empty(),
        "a symbol tagged with an id nothing declares"
    );
    assert!(!report.groups.is_empty(), "per-document group counts");
    assert!(!report.criteria.is_empty(), "criteria classification");
    assert!(
        report.diagnostics.is_empty(),
        "the baseline model is healthy: {:?}",
        report.diagnostics
    );

    // The `exclude:` glob holds where it is declared: the fixture's colliding
    // id mints nothing and its dangling `TC-666` is never read.
    let json = report.to_json();
    assert!(
        !json.contains("TC-666"),
        "an excluded document's references reached the report"
    );
    assert!(
        !report
            .groups
            .iter()
            .any(|g| g.document.contains("fixtures/")),
        "an excluded document minted ids: {:?}",
        report.groups
    );

    // KNOWN GAP, pinned deliberately (agent-ix/quire-rs#124): `exclude:` is
    // declared on trace targets and document references, and the CR-028
    // criteria counts walk the corpus on their own axis with no exclusion to
    // apply — so deliberately malformed fixture data still inflates the
    // criteria denominator. The baseline's job is to pin what the engine does
    // today, defects included; asserting the leak here means closing #124
    // fails this test and the baseline diff is reviewed rather than absorbed.
    assert!(
        report
            .criteria
            .iter()
            .any(|c| c.document.contains("fixtures/")),
        "if this now passes, #124 was fixed: regenerate the baseline and \
         drop this assertion"
    );
    // And the undeclared archetype was never body-parsed (CR-049).
    assert!(
        !spec
            .by_id("ADR-001")
            .expect("ADR-001 is in the corpus")
            .body_is_parsed(),
        "an undeclared archetype must not be body-parsed during coverage"
    );
}
