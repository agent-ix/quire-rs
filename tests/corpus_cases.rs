//! The controlled corpus (FR-050-AC-29, FR-065, CR-098 / CR-106) —
//! `agent-ix/quire-rs#232`, `#233`, `#267`.
//!
//! One parameterized test over **`agent-ix/qa-corpus`**, pinned as a submodule
//! at `corpus/`. It was `include_str!` of one hardcoded JSON file; it is now a
//! walk of `corpus/cases/`, so adding a case is adding a directory and costs no
//! `.rs` edit.
//!
//! The inputs are read **in place**. They used to be strings materialised into
//! a tempdir under a hardcoded `module/`/`spec/`/`src/` layout, which meant no
//! case could express a `tests/` topology or exercise `source_exclude` — and
//! meant no case could be read at all without running this file.
//!
//! Each case carries an `issue_ref`. That is the bug-to-fixture link
//! (`agent-ix/quire-rs#234`) and it is required, not decorative: a fixture
//! whose origin is unrecorded becomes a fixture nobody dares change, which is
//! how a corpus rots into a set of assertions everybody works around.

mod corpus_case;

use std::collections::BTreeSet;

use ix_trace_rs::trace;

use corpus_case::{grade, load_cases, run, Level};

#[trace("TC-992", "FR-050-AC-29")]
// marker-form mismatch, and its control. (CR-098)
#[trace("TC-993", "FR-050-AC-29")]
// a stale test name over a correct marker.
#[trace("TC-994", "FR-050-AC-29")]
// a greenfield corpus reports 0% honestly.
#[trace("TC-995", "FR-050-AC-29")]
// `implements` was never asked, not answered none.
#[trace("TC-996", "FR-050-AC-29")]
// the catch-all-only properties headline.
#[trace("TC-1011", "FR-065-AC-1")]
// read in place: no case is materialised.
#[test]
fn corpus_cases_hold() {
    let cases = load_cases();
    assert!(!cases.is_empty(), "the corpus is not empty");

    let mut failures = String::new();
    for case in &cases {
        let outcome = grade(case, &run(case));
        if !outcome.passed() {
            failures.push_str(&outcome.report());
        }
    }
    assert!(
        failures.is_empty(),
        "corpus cases lost a detection level:\n{failures}"
    );
}

#[trace("TC-1016", "FR-065-AC-11")]
// a failing case names the level it lost, and the
// deepest level it reached. Driven by MUTATING a real case's expectation
// rather than by a synthetic fixture: the claim is about the grader's reading
// of real corpus data, and a hand-built `Outcome` would assert the enum
// ordering and nothing else.
#[test]
fn tc1016_a_lost_level_is_named_and_graded() {
    let mut cases = load_cases();
    let case = cases
        .iter_mut()
        .find(|c| !c.expect.diagnostic_message_contains.is_empty())
        .expect("a case asserting an L3 message");
    let report = run(case);

    // Baseline: it passes as authored.
    assert!(
        grade(case, &report).passed(),
        "{}",
        grade(case, &report).report()
    );

    // Break ONLY the L3 assertion. L1 and L2 must still hold, so the grader
    // has to distinguish them — a grader that failed everything at once would
    // pass a test that only checked "it failed".
    let reason = case
        .expect
        .diagnostic_message_contains
        .keys()
        .next()
        .expect("a reason")
        .clone();
    case.expect
        .diagnostic_message_contains
        .insert(reason, "a phrase no diagnostic will ever carry".to_string());

    let outcome = grade(case, &report);
    assert!(!outcome.passed());
    assert_eq!(outcome.level_lost(), Some(Level::L3Actionable));
    assert_eq!(outcome.level_reached(), Some(Level::L2Localised));

    let text = outcome.report();
    assert!(text.contains("LOST L3 actionable"), "{text}");
    assert!(text.contains(&outcome.issue_ref), "{text}");
    assert!(text.contains("reached L2 localised"), "{text}");
}

#[trace("TC-1016", "FR-065-AC-12")]
// losing L1 reports no level reached — the case that
// distinguishes "the detector stopped firing" from "the message got worse".
#[test]
fn tc1016_losing_l1_reports_no_level_reached() {
    let mut cases = load_cases();
    let case = cases
        .iter_mut()
        .find(|c| !c.expect.diagnostic_reasons.is_empty())
        .expect("a case asserting an L1 reason");
    let report = run(case);

    case.expect
        .diagnostic_reasons
        .push("a-reason-no-engine-emits".to_string());
    let outcome = grade(case, &report);

    assert_eq!(outcome.level_lost(), Some(Level::L1Detected));
    assert_eq!(outcome.level_reached(), None, "{}", outcome.report());
    assert!(outcome.report().contains("reached no level"));
}

/// Every case names the filing it is the regression for, and every case is
/// uniquely named — the two properties that keep the corpus navigable as it
/// grows past the point where anyone remembers all of it.
#[trace("TC-1012", "FR-065-AC-3")]
// attribution is required, not decorative.
#[test]
fn every_case_is_attributed_and_uniquely_named() {
    let cases = load_cases();
    let mut ids: Vec<&str> = cases.iter().map(|c| c.meta.id.as_str()).collect();
    ids.sort_unstable();
    let before = ids.len();
    ids.dedup();
    assert_eq!(before, ids.len(), "duplicate case id: {ids:?}");

    for case in &cases {
        assert!(
            case.meta.issue_ref.contains('#'),
            "{}: issue_ref must name a filing, got {:?}",
            case.meta.id,
            case.meta.issue_ref
        );
    }
}

/// A control names a case that exists, and does not claim a finding is
/// expected on it.
#[trace("TC-1017", "FR-065-AC-13")]
// every control's partner resolves.
#[test]
fn tc1017_every_control_names_a_case_that_exists() {
    let cases = load_cases();
    let ids: BTreeSet<&str> = cases.iter().map(|c| c.meta.id.as_str()).collect();

    for case in cases.iter().filter(|c| c.meta.kind == "control") {
        let partner = case
            .meta
            .control_for
            .as_deref()
            .unwrap_or_else(|| panic!("{}: a control declares control_for", case.meta.id));
        assert!(
            ids.contains(partner),
            "{}: control_for names `{partner}`, which is no case",
            case.meta.id
        );
        // A control is input on which nothing may be found. `findable: true`
        // on one tells a recall-scoring consumer to expect a finding there.
        assert!(
            !case.meta.findable,
            "{}: a control cannot be findable",
            case.meta.id
        );
    }
}

/// The corpus is deterministic: the same case run twice produces the same
/// report. Mirrors the `filament_core` corpus's own determinism guard, which is
/// the pattern this generalizes.
#[trace("TC-1019", "FR-065-AC-17")]
// two runs over unchanged input agree byte for byte.
#[test]
fn corpus_cases_are_deterministic() {
    for case in &load_cases() {
        let first = run(case).to_json();
        let second = run(case).to_json();
        assert_eq!(first, second, "{} is not deterministic", case.meta.id);
    }
}

/// The vocabularies are read from `corpus.yaml`, not compiled in here.
///
/// This is what makes FR-065's single-definition claim checkable from ONE
/// repository. A runner carrying its own copy of the mode families agrees with
/// the corpus only by coincidence, and nothing detects the day it stops.
#[trace("TC-1021", "FR-065-AC-19")]
// the bounds enum comes from corpus.yaml.
#[trace("TC-1021", "FR-065-AC-21")]
// so do the mode families, and a case naming an
// undeclared one is rejected.
#[test]
fn tc1021_the_vocabularies_come_from_the_corpus_not_from_this_file() {
    let declared: serde_yaml::Value = serde_yaml::from_str(
        &std::fs::read_to_string(corpus_case::corpus_root().join("corpus.yaml"))
            .expect("read corpus.yaml"),
    )
    .expect("corpus.yaml parses");

    let list = |key: &str| -> BTreeSet<String> {
        declared[key]
            .as_sequence()
            .unwrap_or_else(|| panic!("corpus.yaml declares `{key}`"))
            .iter()
            .map(|v| v.as_str().expect("a string").to_string())
            .collect()
    };
    let families = list("mode_families");
    let kinds = list("case_kinds");
    let states = list("bounds_states");
    let levels = list("grading_levels");

    // Non-vacuous: an empty declaration would make every assertion below pass.
    assert!(!families.is_empty() && !kinds.is_empty());
    // The ladder this file implements is the one the corpus declares. If the
    // corpus renamed a level, this file's `Level` enum would be a second
    // spelling — the exact thing FR-065-AC-20 forbids.
    assert_eq!(
        levels,
        ["L1", "L2", "L3"].iter().map(|s| s.to_string()).collect(),
        "the harness ladder and the declared ladder have diverged",
    );
    assert!(states.contains("GAP") && states.contains("covered"));

    for case in &load_cases() {
        assert!(
            families.contains(&case.meta.mode),
            "{}: mode `{}` is not a declared family {families:?}",
            case.meta.id,
            case.meta.mode,
        );
        assert!(
            kinds.contains(&case.meta.kind),
            "{}: kind `{}` is not declared {kinds:?}",
            case.meta.id,
            case.meta.kind,
        );
        // The module a case names must exist under `modules/`. The port shipped
        // `module: variants/bench-legacy` on two cases whose in-directory
        // manifest was a DIFFERENT synthetic module, so the field was a claim
        // nothing checked.
        let module = corpus_case::corpus_root()
            .join("modules")
            .join(&case.meta.module);
        assert!(
            module.join("manifest.yaml").is_file(),
            "{}: module `{}` names no manifest at {}",
            case.meta.id,
            case.meta.module,
            module.display(),
        );
        assert!(
            case.meta.tags.iter().any(|t| t.starts_with("TC-")),
            "{}: at least one tracking id, got {:?}",
            case.meta.id,
            case.meta.tags,
        );
        // The declared language is one the walker knows, and it agrees with
        // what the case's own census expects. A case labelled `python` whose
        // expectation names the `rust` census is a bounds-matrix entry filed
        // under a column it does not measure.
        assert!(
            ["rust", "python", "typescript"].contains(&case.meta.language.as_str()),
            "{}: language `{}` is not one the symbol walker reads",
            case.meta.id,
            case.meta.language,
        );
        for census in &case.expect.binding_census {
            assert_eq!(
                census.language, case.meta.language,
                "{}: declared language and expected census disagree",
                case.meta.id,
            );
        }
    }
}
