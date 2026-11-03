use std::fs;
use std::path::Path;

use ix_trace_rs::trace;
use quire_bench::perf::check;

fn estimate(root: &Path, name: &str, body: &str) {
    let directory = root.join(name).join("change");
    fs::create_dir_all(&directory).unwrap();
    fs::write(directory.join("estimates.json"), body).unwrap();
}

#[trace("NFR-001", "NFR-002", "NFR-007")]
#[test]
fn regression_gate_checks_means_and_names_the_failed_bench() {
    let temp = tempfile::tempdir().unwrap();
    estimate(
        temp.path(),
        "parse/slow",
        r#"{"mean":{"point_estimate":0.11}}"#,
    );
    estimate(
        temp.path(),
        "render/fast",
        r#"{"mean":{"point_estimate":-0.20}}"#,
    );
    estimate(
        temp.path(),
        "load/edge",
        r#"{"mean":{"point_estimate":0.10}}"#,
    );
    let report = check(temp.path(), 0.10).unwrap();
    assert!(report.failed);
    assert_eq!(report.seen, 3);
    assert!(report
        .lines
        .iter()
        .any(|line| line.contains("regression in parse/slow")));
    assert!(report
        .lines
        .iter()
        .any(|line| line.contains("speedup render/fast")));
    assert!(report
        .lines
        .iter()
        .any(|line| line.contains("ok      load/edge")));
}

#[trace("NFR-001", "NFR-002", "NFR-007")]
#[test]
fn unreadable_and_absent_estimates_do_not_invent_a_regression() {
    let temp = tempfile::tempdir().unwrap();
    let missing = check(&temp.path().join("absent"), 0.10).unwrap();
    assert!(!missing.failed);
    assert!(missing.lines[0].contains("no criterion output"));
    estimate(temp.path(), "broken", "not JSON");
    let report = check(temp.path(), 0.10).unwrap();
    assert!(!report.failed);
    assert_eq!(report.seen, 0);
    assert!(report.lines.iter().any(|line| line.contains("skip broken")));
    assert!(report
        .lines
        .iter()
        .any(|line| line.contains("no change/ files found")));
}

#[trace("NFR-001", "NFR-002", "NFR-007")]
#[test]
fn invalid_band_fails_closed() {
    let temp = tempfile::tempdir().unwrap();
    assert!(check(temp.path(), f64::NAN).is_err());
    assert!(check(temp.path(), -0.1).is_err());
}
