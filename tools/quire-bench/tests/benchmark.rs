use std::collections::BTreeMap;
use std::fs;
use std::process::Command;

use ix_trace_rs::trace;
use quire_bench::collection::require_clean;
use quire_bench::metrics::{metrics_from, silent_zeros, CoveragePayload};
use quire_bench::{compare, score, Direction, Manifest, Observed, Verdict};
use serde_json::{json, Number};

fn manifest() -> Manifest {
    serde_json::from_str(include_str!("../../../bench/manifest.json")).unwrap()
}

#[trace("FR-050-AC-30")]
#[test]
fn metric_dictionary_declares_provenance_for_every_score() {
    for (name, spec) in manifest().metrics {
        assert!(!spec.unit.is_empty(), "{name}: missing unit");
        assert!(!spec.population.is_empty(), "{name}: missing population");
        assert!(!spec.method.is_empty(), "{name}: missing method");
    }
}

fn number(value: serde_json::Value) -> Number {
    value.as_number().unwrap().clone()
}

fn payload(value: serde_json::Value) -> CoveragePayload {
    serde_json::from_value(value).unwrap()
}

#[trace("FR-050-AC-30")]
#[test]
fn ratchet_improves_holds_regresses_and_gates_zero() {
    let low = number(json!(3));
    let high = number(json!(10));
    assert_eq!(
        compare(Direction::LowerIsBetter, &low, Some(&high)).0,
        Verdict::Improved
    );
    assert_eq!(
        compare(Direction::LowerIsBetter, &high, Some(&low)).0,
        Verdict::Regressed
    );
    assert_eq!(
        compare(Direction::HigherIsBetter, &high, Some(&high)).0,
        Verdict::Held
    );
    assert_eq!(
        compare(Direction::HigherIsBetter, &high, None).0,
        Verdict::New
    );
    let (verdict, next) = compare(Direction::GateZero, &high, Some(&high));
    assert_eq!(verdict, Verdict::Regressed);
    assert_eq!(next, Number::from(0));
}

#[trace("FR-050-AC-30")]
#[test]
fn integer_ratchet_distinguishes_adjacent_counts_above_f64_precision() {
    let baseline = Number::from(9_007_199_254_740_992_u64);
    let worse = Number::from(9_007_199_254_740_993_u64);
    assert_eq!(
        compare(Direction::LowerIsBetter, &worse, Some(&baseline)).0,
        Verdict::Regressed
    );
    assert_eq!(
        compare(Direction::HigherIsBetter, &worse, Some(&baseline)).0,
        Verdict::Improved
    );
}

#[trace("FR-050-AC-30")]
#[test]
fn score_refuses_undeclared_metric_and_carries_provenance() {
    let mut observed = Observed::new();
    observed.insert(
        "self".to_owned(),
        BTreeMap::from([("coverage.invented".to_owned(), Number::from(1))]),
    );
    assert!(score(&manifest(), &observed, &Observed::new())
        .unwrap_err()
        .to_string()
        .contains("not in the dictionary"));

    observed.insert(
        "self".to_owned(),
        BTreeMap::from([("coverage.backed_pct".to_owned(), number(json!(51.0)))]),
    );
    let report = score(&manifest(), &observed, &Observed::new()).unwrap();
    let row = &report.rows[0];
    assert_eq!(row.verdict, Verdict::New);
    assert!(!row.unit.is_empty() && !row.population.is_empty() && !row.method.is_empty());
    let value = serde_json::to_value(&report).unwrap();
    assert!(value.get("timestamp").is_none());
    assert!(value["rows"][0].get("timestamp").is_none());
}

#[trace("FR-050-AC-30")]
#[test]
fn score_orders_corpora_and_metrics() {
    let observed = Observed::from([
        (
            "z-corpus".to_owned(),
            BTreeMap::from([("coverage.dead_tags".to_owned(), Number::from(2))]),
        ),
        (
            "a-corpus".to_owned(),
            BTreeMap::from([
                ("coverage.dead_tags".to_owned(), Number::from(1)),
                ("coverage.backed_pct".to_owned(), number(json!(50.0))),
            ]),
        ),
    ]);
    let report = score(&manifest(), &observed, &Observed::new()).unwrap();
    let order: Vec<_> = report
        .rows
        .iter()
        .map(|row| (row.corpus.as_str(), row.metric.as_str()))
        .collect();
    assert_eq!(
        order,
        [
            ("a-corpus", "coverage.backed_pct"),
            ("a-corpus", "coverage.dead_tags"),
            ("z-corpus", "coverage.dead_tags"),
        ]
    );
}

#[trace("FR-050-AC-30")]
#[test]
fn unreadable_metric_is_omitted_with_a_reason() {
    let payload = payload(json!({
        "totals": {"backed": 5, "total": 10},
        "untracked_symbols": []
    }));
    let (measured, omissions) = metrics_from(&payload).unwrap();
    assert_eq!(measured["coverage.backed_pct"], number(json!(50.0)));
    assert!(!measured.contains_key("coverage.binding_read_pct"));
    assert!(!measured.contains_key("authoring.tag_rate"));
    assert!(!measured.contains_key("properties.specific_shaped_pct"));
    assert!(omissions
        .iter()
        .any(|message| message.contains("binding_census")));
    assert!(omissions
        .iter()
        .any(|message| message.contains("specific_shaped")));
}

#[trace("FR-050-AC-30")]
#[test]
fn silent_zero_detects_only_unreported_ratio_holes() {
    let hollow = json!({
        "name": "coverage.backed", "state": "measured", "shape": "ratio",
        "value": 0, "population": 100, "examined": 50, "matched": 0
    });
    assert_eq!(
        silent_zeros(&payload(json!({"metrics": [hollow.clone()]}))),
        1
    );
    assert_eq!(
        silent_zeros(&payload(json!({
            "metrics": [hollow.clone()], "diagnostics": [{"reason": "hollow-denominator"}]
        }))),
        0
    );
    let mut count = hollow;
    count["shape"] = json!("count");
    assert_eq!(silent_zeros(&payload(json!({"metrics": [count]}))), 0);
}

#[trace("FR-050-AC-32")]
#[test]
fn suspicion_rate_uses_evidence_symbol_population() {
    let payload = payload(json!({
        "binding_census": [{"candidates": 551, "bound": 374}],
        "suspicions": vec![json!({"kind": "vacuous-under-guard"}); 549]
    }));
    let (measured, _) = metrics_from(&payload).unwrap();
    assert_eq!(measured["skeptic.suspicion_rate"], number(json!(99.64)));
}

#[trace("FR-050-AC-30")]
#[test]
fn declared_clean_corpus_refuses_uncommitted_input() {
    let corpus = tempfile::tempdir().unwrap();
    assert!(Command::new("git")
        .arg("init")
        .arg("-q")
        .arg(corpus.path())
        .status()
        .unwrap()
        .success());
    require_clean(corpus.path(), "fixture", ".", true).unwrap();
    fs::write(corpus.path().join("new.md"), "uncommitted").unwrap();
    assert!(require_clean(corpus.path(), "fixture", ".", true)
        .unwrap_err()
        .to_string()
        .contains("dirty"));
}
