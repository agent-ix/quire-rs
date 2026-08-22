//! The declarative regression corpus (FR-050-AC-29, CR-098) —
//! `agent-ix/quire-rs#232` and `agent-ix/quire-rs#233`.
//!
//! One parameterized test over `tests/fixtures/corpus_cases/coverage.json`.
//! Every battletest failure family from `agent-ix/quoin#197` is a JSON object
//! in that file, and adding the next one costs no `.rs`.
//!
//! Each case carries an `issue_ref`. That is the bug-to-fixture link
//! (`agent-ix/quire-rs#234`) and it is required, not decorative: a fixture
//! whose origin is unrecorded becomes a fixture nobody dares change, which is
//! how a corpus rots into a set of assertions everybody works around.

mod corpus_case;

use ix_trace_rs::trace;
use tempfile::TempDir;

use corpus_case::{assert_expected, run, CorpusCase};

const CASES: &str = include_str!("fixtures/corpus_cases/coverage.json");

fn cases() -> Vec<CorpusCase> {
    serde_json::from_str(CASES).expect("corpus cases parse")
}

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
#[test]
fn corpus_cases_hold() {
    let all = cases();
    assert!(!all.is_empty(), "the corpus is not empty");
    for case in &all {
        let dir = TempDir::new().expect("tempdir");
        let report = run(case, dir.path());
        assert_expected(case, &report);
    }
}

/// Every case names the filing it is the regression for, and every case is
/// uniquely named — the two properties that keep the corpus navigable as it
/// grows past the point where anyone remembers all of it.
#[test]
fn every_case_is_attributed_and_uniquely_named() {
    let all = cases();
    let mut names: Vec<&str> = all.iter().map(|c| c.name.as_str()).collect();
    names.sort_unstable();
    let before = names.len();
    names.dedup();
    assert_eq!(before, names.len(), "duplicate case name: {names:?}");

    for case in &all {
        assert!(
            case.issue_ref.contains('#'),
            "{}: issue_ref must name a filing, got {:?}",
            case.name,
            case.issue_ref
        );
        assert!(
            case.tags.iter().any(|t| t.starts_with("TC-")),
            "{}: at least one tracking id, got {:?}",
            case.name,
            case.tags
        );
    }
}

/// The corpus is deterministic: the same case run twice produces the same
/// report. Mirrors the `filament_core` corpus's own determinism guard, which is
/// the pattern this generalizes.
#[test]
fn corpus_cases_are_deterministic() {
    for case in &cases() {
        let first = {
            let dir = TempDir::new().expect("tempdir");
            run(case, dir.path()).to_json()
        };
        let second = {
            let dir = TempDir::new().expect("tempdir");
            run(case, dir.path()).to_json()
        };
        assert_eq!(first, second, "{} is not deterministic", case.name);
    }
}
