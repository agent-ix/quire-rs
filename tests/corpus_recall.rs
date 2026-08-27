//! The controlled-corpus recall ratchet (#269).

#![allow(dead_code, unused_imports)]

mod corpus_case;

use std::collections::{BTreeMap, BTreeSet};
use std::process::Command;

use corpus_case::{grade, load_cases, run, Case, Level};
use serde::{Deserialize, Serialize};

const DEFINITION: &str = "detection-recall-v1";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
struct RecallRow {
    mode: String,
    language: String,
    level: String,
    reached: usize,
    population: usize,
    gap_count: usize,
    misses: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Baseline {
    definition_version: String,
    runner: String,
    corpus_revision: String,
    gap_count: usize,
    rows: Vec<RecallRow>,
}

#[derive(Default)]
struct Group {
    population: usize,
    reached: [usize; 3],
    misses: [Vec<String>; 3],
}

#[test]
fn detection_recall_is_ratcheted_per_level_mode_and_language() {
    let gap_count = current_gap_count();
    let rows = score(gap_count);
    assert!(
        rows.iter().all(|row| row.gap_count == gap_count),
        "bounds.gap_count must be adjacent to every recall row"
    );

    let path = corpus_case::corpus_root().join("baselines/quire-rs.json");
    if std::env::var_os("UPDATE_CORPUS_RECALL").is_some() {
        let baseline = Baseline {
            definition_version: DEFINITION.to_string(),
            runner: "quire-rs".to_string(),
            corpus_revision: corpus_revision(),
            gap_count,
            rows,
        };
        std::fs::write(
            &path,
            format!(
                "{}\n",
                serde_json::to_string_pretty(&baseline).expect("serialize recall baseline")
            ),
        )
        .expect("write recall baseline");
        return;
    }

    let baseline: Baseline =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap_or_else(|_| {
            panic!(
                "{} is missing; run `make corpus-recall-update` to create a measured baseline",
                path.display()
            )
        }))
        .expect("valid recall baseline");
    assert_eq!(baseline.definition_version, DEFINITION);
    assert_eq!(baseline.runner, "quire-rs");
    assert_eq!(
        baseline.gap_count, gap_count,
        "GAP count moved; re-baseline deliberately"
    );

    let before: BTreeMap<_, _> = baseline.rows.iter().map(|row| (key(row), row)).collect();
    let now: BTreeMap<_, _> = rows.iter().map(|row| (key(row), row)).collect();
    assert_eq!(
        before.keys().collect::<Vec<_>>(),
        now.keys().collect::<Vec<_>>()
    );
    for (dimension, observed) in now {
        let old = before[&dimension];
        assert_eq!(
            old.population, observed.population,
            "{dimension}: population moved; no delta is comparable until reviewed"
        );
        assert!(
            observed.reached >= old.reached,
            "{dimension}: recall regressed {}/{} -> {}/{}; missed {:?}",
            old.reached,
            old.population,
            observed.reached,
            observed.population,
            observed.misses
        );
        assert_eq!(
            observed.reached, old.reached,
            "{dimension}: recall improved; run `make corpus-recall-update` to retain the tighter ratchet"
        );
    }
}

fn score(gap_count: usize) -> Vec<RecallRow> {
    let mut groups: BTreeMap<(String, String), Group> = BTreeMap::new();
    for case in load_cases() {
        if case.meta.kind != "failure" || !case.meta.findable {
            continue;
        }
        let claimed = claimed_levels(&case);
        // A shared case may be witnessed exclusively by another validator.
        // Its producer's runner scores it; counting it as a Quire miss would
        // make this runner's denominator include work it did not evaluate.
        if !evaluated_by_quire(&case) {
            continue;
        }
        let report = run(&case);
        let outcome = grade(&case, &report);
        let lost = outcome.level_lost();
        let achieved = [
            claimed[0] && lost != Some(Level::L1Detected),
            claimed[1] && !matches!(lost, Some(Level::L1Detected | Level::L2Localised)),
            claimed[2] && lost.is_none(),
        ];
        let group = groups
            .entry((case.meta.mode.clone(), case.meta.language.clone()))
            .or_default();
        group.population += 1;
        for (index, reached) in achieved.iter().enumerate() {
            if *reached {
                group.reached[index] += 1;
            } else {
                group.misses[index].push(case.meta.id.clone());
            }
        }
    }

    let levels = ["L1", "L2", "L3"];
    groups
        .into_iter()
        .flat_map(|((mode, language), group)| {
            levels.into_iter().enumerate().map(move |(index, level)| {
                let mut misses = group.misses[index].clone();
                misses.sort();
                RecallRow {
                    mode: mode.clone(),
                    language: language.clone(),
                    level: level.to_string(),
                    reached: group.reached[index],
                    population: group.population,
                    gap_count,
                    misses,
                }
            })
        })
        .collect()
}

#[test]
fn externally_witnessed_cases_are_outside_the_quire_population() {
    let cases = load_cases();
    let external: BTreeSet<String> = cases
        .iter()
        .filter(|case| {
            case.meta.kind == "failure"
                && case.meta.findable
                && !case.expect.external_observations.is_empty()
                && !claimed_levels(case)[0]
        })
        .map(|case| case.meta.id.clone())
        .collect();
    assert!(
        !external.is_empty(),
        "the corpus has no external-only witness"
    );

    let rows = score(current_gap_count());
    let misses: BTreeSet<&str> = rows
        .iter()
        .flat_map(|row| row.misses.iter().map(String::as_str))
        .collect();
    assert!(
        external.iter().all(|id| !misses.contains(id.as_str())),
        "external-only cases were reported as Quire misses"
    );

    let expected_population = cases
        .iter()
        .filter(|case| {
            case.meta.kind == "failure" && case.meta.findable && evaluated_by_quire(case)
        })
        .count();
    let observed_population: usize = rows
        .iter()
        .filter(|row| row.level == "L1")
        .map(|row| row.population)
        .sum();
    assert_eq!(observed_population, expected_population);
}

fn claimed_levels(case: &Case) -> [bool; 3] {
    let expect = &case.expect;
    let l3 = !expect.diagnostic_message_contains.is_empty()
        || expect
            .suspicions
            .iter()
            .any(|item| !item.message_contains.is_empty());
    let l2 = !expect.diagnostic_paths.is_empty()
        || expect
            .untracked_symbols
            .as_ref()
            .is_some_and(|symbols| !symbols.is_empty())
        || expect
            .binding_census
            .iter()
            .any(|item| item.unbound_example.is_some() || item.unmatched_example.is_some())
        || expect
            .suspicions
            .iter()
            .any(|item| item.path.is_some() || item.line.is_some() || item.symbol.is_some());
    let l1 = !expect.diagnostic_reasons.is_empty()
        || !expect.validate_contains.is_empty()
        || !expect.suspicions.is_empty()
        || l2
        || l3;
    [l1, l1 && l2, l1 && l2 && l3]
}

fn evaluated_by_quire(case: &Case) -> bool {
    case.expect.external_observations.is_empty() || claimed_levels(case)[0]
}

fn current_gap_count() -> usize {
    let output = Command::new("python3")
        .arg(corpus_case::corpus_root().join("bounds.py"))
        .arg("--json")
        .output()
        .expect("run bounds.py");
    assert!(output.status.success(), "bounds.py failed");
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("bounds JSON");
    value["bounds"]["gap_count"]
        .as_u64()
        .expect("numeric gap_count") as usize
}

fn corpus_revision() -> String {
    let output = Command::new("git")
        .args(["-C"])
        .arg(corpus_case::corpus_root())
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("read corpus revision");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn key(row: &RecallRow) -> String {
    format!("{}/{}/{}", row.level, row.mode, row.language)
}
