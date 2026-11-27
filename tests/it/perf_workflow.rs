use std::fs;
use std::path::PathBuf;

use ix_trace_rs::trace;
use serde_yaml::Value;

fn ci_workflow() -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".github/workflows/ci.yml");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read workflow {}: {error}", path.display()));
    serde_yaml::from_str(&source)
        .unwrap_or_else(|error| panic!("parse workflow {}: {error}", path.display()))
}

#[trace("NFR-015-AC-3")]
#[test]
fn tc_2314_load_repo_bench_uses_the_stored_baseline_and_regression_gate() {
    let workflow = ci_workflow();
    let steps = workflow["jobs"]["perf"]["steps"]
        .as_sequence()
        .expect("perf job has steps");
    let bench_step = steps
        .iter()
        .find(|step| step["name"].as_str() == Some("Run benches against stored baseline"))
        .expect("perf job runs benches against the stored baseline");
    let commands = bench_step["run"]
        .as_str()
        .expect("benchmark step has shell commands");
    assert!(commands.contains("--bench load_repo"));
    assert!(commands.contains("--baseline main"));
    assert!(commands.contains("--save-baseline main"));
    assert!(commands.contains("target/criterion/load/registry_load_from_demo/main"));
    assert!(commands.contains("target/criterion/load_repo_1k/threads_1/main"));
    assert!(commands.contains("target/criterion/load_repo_1k/threads_8/main"));
    assert!(commands.contains("target/criterion/load_repo_1k_bodies/threads_1/main"));
    assert!(commands.contains("target/criterion/load_repo_1k_bodies/threads_8/main"));

    let gate_step = steps
        .iter()
        .find(|step| {
            step["name"]
                .as_str()
                .is_some_and(|name| name.starts_with("Enforce regression band"))
        })
        .expect("perf job enforces a regression band");
    assert!(
        gate_step["run"]
            .as_str()
            .expect("regression step has a command")
            .contains("quire-perf-gate")
    );
    assert_eq!(
        gate_step["env"]["QUIRE_PERF_BAND"].as_str(),
        Some("0.50"),
        "Ubuntu CI uses the documented cross-runner allowance"
    );
}
