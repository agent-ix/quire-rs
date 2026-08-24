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
    let mut pending_now_passing = String::new();
    let mut pending = 0usize;

    for case in &cases {
        let outcome = grade(case, &run(case));
        match (&case.meta.pending, outcome.passed()) {
            // Expected to fail, and does. The corpus records a defect the
            // engine has not fixed — which is the state EPIC #264 rule 3 wants
            // a fixture to be in BEFORE its fix lands.
            (Some(_), false) => pending += 1,
            // Expected to fail and passes: the fix landed, and the marker is
            // now lying about the engine. Failing here is what stops a corpus
            // filling up with stale `pending:` markers nobody revisits.
            (Some(ticket), true) => pending_now_passing.push_str(&format!(
                "  {} now PASSES — {ticket} appears to have landed. Remove \
                 `pending:` from its case.yaml.\n",
                case.meta.id
            )),
            (None, false) => failures.push_str(&outcome.report()),
            (None, true) => {}
        }
    }

    if pending > 0 {
        // Printed, not hidden. A count of known-failing cases is a measurement
        // of what the engine cannot yet do, and it belongs beside every run.
        println!("{pending} case(s) pending a fix — expected to fail, and did.");
    }
    assert!(
        pending_now_passing.is_empty(),
        "a pending case started passing:\n{pending_now_passing}"
    );
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

/// A case's declared module and its documented invocation name the same thing.
///
/// The review of #290 found nothing cross-checked them: `verify.py` never read
/// `module:`, and this harness never read `reproduce:`. So
/// `module: ecosystem` beside `--module modules/variants/bench-legacy` was
/// accepted by both runners — each testing a *different* module, with
/// `bounds.py` crediting the cell as ecosystem-covered. That is #266's defect
/// ("the field named one thing and the file loaded another") relocated rather
/// than removed.
#[trace("TC-1018", "FR-065-AC-15")]
// a variant binding names its relaxation ticket.
#[trace("TC-1020", "FR-065-AC-18")]
// the documented invocation names the module that loads.
#[test]
fn tc1020_the_documented_invocation_names_the_module_that_loads() {
    for case in &load_cases() {
        let reproduce = case
            .meta
            .reproduce
            .as_deref()
            .unwrap_or_else(|| panic!("{}: no `reproduce` invocation", case.meta.id));

        // FR-065-AC-18: the invocation names a module. Without one no model
        // loads, the run reports 0/0, and the case cannot exhibit the
        // declaration defect it exists for.
        let declared = format!("--module modules/{}", case.meta.module);
        assert!(
            reproduce.contains(&declared),
            "{}: declares `module: {}` but its invocation says `{reproduce}`. \
             Two runners would test different modules and neither would notice.",
            case.meta.id,
            case.meta.module,
        );
        assert!(
            reproduce.contains(&format!(
                "--scope {}",
                case.dir
                    .strip_prefix(corpus_case::corpus_root())
                    .unwrap()
                    .display()
            )),
            "{}: its invocation does not scope to its own directory: {reproduce}",
            case.meta.id,
        );

        // FR-065-CON-3 / AC-15: a variant binding names the ticket sizing it.
        if case.meta.module != "ecosystem" {
            assert!(
                case.meta.relaxation_ticket.is_some(),
                "{}: binds variant `{}` and names no `relaxation_ticket`",
                case.meta.id,
                case.meta.module,
            );
        }

        // A `pending:` marker with no stated reason is one nobody can decide
        // whether to remove, which is how stale markers accumulate.
        if case.meta.pending.is_some() {
            let reason = case.meta.pending_reason.as_deref().unwrap_or("");
            assert!(
                !reason.trim().is_empty(),
                "{}: is pending with no `pending_reason`",
                case.meta.id,
            );
        }

        // The same argument as `issue_ref`: a fixture nobody explained is a
        // fixture nobody dares change.
        let comment = case.meta.comment.as_deref().unwrap_or("");
        assert!(
            !comment.trim().is_empty(),
            "{}: carries no `comment` saying what it is about",
            case.meta.id,
        );
    }
}
