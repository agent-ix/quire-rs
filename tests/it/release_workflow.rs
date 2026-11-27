// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Release workflow call and publish gates for tag attestation.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test failures intentionally panic with assertion context"
)]

use ix_trace_rs::trace;
use serde_yaml::{Mapping, Value};
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn workflow(name: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".github/workflows")
        .join(name);
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read workflow {}: {error}", path.display()));
    serde_yaml::from_str(&source)
        .unwrap_or_else(|error| panic!("parse workflow {}: {error}", path.display()))
}

fn events(document: &Value) -> &Mapping {
    document["on"]
        .as_mapping()
        .expect("workflow has event mapping")
}

fn event<'a>(events: &'a Mapping, name: &str) -> &'a Value {
    events
        .get(Value::String(name.to_owned()))
        .expect("workflow has expected event")
}

#[trace("FR-093-AC-1")]
#[test]
fn tc_2258_manual_publish_dispatch_defaults_false() {
    let wheels = workflow("wheels.yml");
    let wheels_events = events(&wheels);
    let dispatch = event(wheels_events, "workflow_dispatch");
    assert_eq!(
        dispatch["inputs"]["publish"]["default"].as_bool(),
        Some(false)
    );
}

#[trace("FR-093-AC-2")]
#[test]
fn tc_2259_wheel_workflow_call_publish_input_is_false_by_default() {
    let wheels = workflow("wheels.yml");
    let wheels_events = events(&wheels);
    let call = event(wheels_events, "workflow_call");
    assert_eq!(call["inputs"]["publish"]["type"].as_str(), Some("boolean"));
    assert_eq!(call["inputs"]["publish"]["default"].as_bool(), Some(false));
}

#[trace("FR-093-AC-3")]
#[test]
fn tc_2260_publish_job_is_manual_only() {
    let wheels = workflow("wheels.yml");
    assert_eq!(
        wheels["jobs"]["publish"]["if"].as_str(),
        Some("${{ github.event_name == 'workflow_dispatch' && inputs.publish }}")
    );
}

#[trace("FR-093-AC-4")]
#[test]
fn tc_2261_tag_attestation_triggers_only_on_version_tags() {
    let tag_attestation = workflow("tag-attestation.yml");
    let tag_events = events(&tag_attestation);
    assert_eq!(
        tag_events.len(),
        2,
        "direct tag push remains available while exposing reusable workflow_call"
    );
    assert_eq!(event(tag_events, "push")["tags"][0].as_str(), Some("v*"));
    assert!(tag_events.contains_key(Value::String("workflow_call".to_owned())));
}

#[trace("FR-093-AC-5")]
#[test]
fn tc_2262_tag_build_calls_wheels_without_publishing() {
    let tag_attestation = workflow("tag-attestation.yml");
    let build = &tag_attestation["jobs"]["build"];
    assert_eq!(
        build["uses"].as_str(),
        Some("./.github/workflows/wheels.yml")
    );
    assert_eq!(build["with"]["publish"].as_bool(), Some(false));
}

#[trace("FR-093-AC-6")]
#[test]
fn tc_2263_tag_attestation_calls_shared_release_workflow() {
    let tag_attestation = workflow("tag-attestation.yml");
    let attest = &tag_attestation["jobs"]["attest"];
    assert!(attest["uses"]
        .as_str()
        .expect("tag workflow calls the shared attestation workflow")
        .starts_with("agent-ix/engineering-assurance/.github/workflows/release-attestation.yml@"));
}

#[trace("FR-095-AC-1")]
#[test]
fn tc_2229_tag_workflow_runs_and_uploads_the_declared_release_suite() {
    let tag_attestation = workflow("tag-attestation.yml");
    let suite_registry = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spec/evidence/suites.md"),
    )
    .expect("read suite registry");
    assert!(suite_registry.contains("| SUITE-001 |"));
    assert!(suite_registry.contains("`make evidence`"));

    let evidence_steps = tag_attestation["jobs"]["release-evidence"]["steps"]
        .as_sequence()
        .expect("release evidence job has steps");
    let checkout = evidence_steps
        .iter()
        .find(|step| step["uses"].as_str() == Some("actions/checkout@v7"))
        .expect("tagged source is checked out for evidence generation");
    assert_eq!(checkout["with"]["ref"].as_str(), Some("${{ github.sha }}"));
    assert_eq!(checkout["with"]["fetch-depth"].as_u64(), Some(0));
    assert_eq!(checkout["with"]["submodules"].as_str(), Some("recursive"));
    let evidence_index = evidence_steps
        .iter()
        .position(|step| step["run"].as_str() == Some("make -C source evidence"))
        .expect("tag workflow runs the release evidence suite");
    assert_eq!(
        evidence_steps[evidence_index]["name"].as_str(),
        Some("Generate and verify release evidence")
    );
    let upload_index = evidence_steps
        .iter()
        .position(|step| {
            step["uses"].as_str() == Some("actions/upload-artifact@v7")
                && step["with"]["name"].as_str()
                    == Some("${{ inputs.evidence_artifact_name || 'release-evidence' }}")
        })
        .expect("release evidence job uploads the named artifact");
    assert!(
        evidence_index < upload_index,
        "evidence generation and run-record verification precede artifact upload"
    );
    assert_eq!(
        evidence_steps[upload_index]["with"]["path"].as_str(),
        Some("source/target/release-evidence/verified/")
    );
}

#[trace("FR-095-AC-2")]
#[test]
fn tc_2232_attestation_waits_for_the_retained_evidence_job() {
    let tag_attestation = workflow("tag-attestation.yml");
    let needs = tag_attestation["jobs"]["attest"]["needs"]
        .as_sequence()
        .expect("attestation job declares prerequisites");
    assert!(needs
        .iter()
        .any(|job| job.as_str() == Some("release-evidence")));
    assert!(needs
        .iter()
        .any(|job| job.as_str() == Some("release-artifacts")));
    let steps = tag_attestation["jobs"]["release-evidence"]["steps"]
        .as_sequence()
        .expect("release evidence job has steps");
    assert!(steps
        .iter()
        .any(|step| step["uses"].as_str() == Some("actions/upload-artifact@v7")));
    assert!(steps.iter().any(|step| {
        step["run"]
            .as_str()
            .is_some_and(|run| run == "make -C source evidence")
    }));
    assert!(steps.iter().any(|step| {
        step["uses"].as_str() == Some("actions/upload-artifact@v7")
            && step["with"]["path"].as_str() == Some("source/target/release-evidence/verified/")
    }));
}

#[trace("FR-095-AC-3")]
#[test]
fn tc_2233_oidc_attestation_consumes_artifact_without_running_checkout_code() {
    let tag_attestation = workflow("tag-attestation.yml");
    let attest = &tag_attestation["jobs"]["attest"];
    assert_eq!(
        attest["with"]["evidence_artifact"].as_str(),
        Some("${{ inputs.evidence_artifact_name || 'release-evidence' }}")
    );
    assert_eq!(
        attest["with"]["quoin_tooling_tag"].as_str(),
        Some("${{ inputs.quoin_tooling_tag || '0.36.10' }}")
    );
    assert_eq!(
        tag_attestation["on"]["workflow_call"]["inputs"]["quoin_tooling_tag"]["default"].as_str(),
        Some("0.36.10")
    );
    assert_eq!(attest["permissions"]["id-token"].as_str(), Some("write"));
    assert!(
        attest["steps"].is_null(),
        "reusable job has no local run steps"
    );
    assert_eq!(
        tag_attestation["jobs"]["build"]["with"]["publish"].as_bool(),
        Some(false)
    );
}

#[trace("FR-095-AC-4")]
#[test]
fn tc_2234_make_evidence_records_fresh_suite_run_for_retention() {
    let makefile = fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Makefile"))
        .expect("read Makefile");
    let evidence_target_start = makefile
        .lines()
        .position(|line| line == "evidence:")
        .expect("Makefile defines evidence target");
    let target = makefile
        .lines()
        .skip(evidence_target_start)
        .take_while(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(target.contains("ix-trace-run"));
    assert!(target.contains("--rev 8a0ec153a097036e257cdc8e0ba391bf4dea93d1"));
    assert!(target.contains("scripts/check_release_evidence_checkout.py --repo-root ."));
    assert!(target.contains("git describe --tags --exact-match HEAD"));
    assert!(target.contains("git rev-parse \"refs/tags/$$tag^{commit}\""));
    assert!(target.contains("git for-each-ref --format='%(taggerdate:unix)' \"refs/tags/$$tag\""));
    assert!(target.contains("--tag-created-at \"$$tag_created_at\""));
    assert!(target.contains("CARGO_BUILD_JOBS=4"));
    assert!(target.contains("--workspace --jobs=4"));
    assert!(target.contains("evidence record --repo . --suite SUITE-001"));
    assert!(!target.contains("--commit \"$$commit\""));
    assert!(target.contains("record-result.json"));
    assert!(target.contains("--junit-results"));
    assert!(!target.contains("--package ix-trace-run"));
    assert!(target.contains("release-evidence.tar.gz"));
    assert!(target.contains("scripts/package_release_evidence.py"));
    assert!(target.contains("@agent-ix/quoin@0.30.1"));
    assert!(target.contains("$(RELEASE_QUOIN)"));
    assert!(target.contains("--tool 'ix-trace-run 0.1.0+8a0ec153'"));

    let workflow = workflow("tag-attestation.yml");
    let evidence_steps = workflow["jobs"]["release-evidence"]["steps"]
        .as_sequence()
        .expect("release evidence job has steps");
    let generate = evidence_steps
        .iter()
        .find(|step| step["run"].as_str() == Some("make -C source evidence"))
        .expect("release evidence job runs make evidence");
    assert!(evidence_steps.iter().any(|step| {
        step["uses"].as_str() == Some("actions/checkout@v7")
            && step["with"]["ref"].as_str() == Some("${{ github.sha }}")
            && step["with"]["path"].as_str() == Some("source")
    }));
    let setup_node = evidence_steps
        .iter()
        .find(|step| step["uses"].as_str() == Some("actions/setup-node@v7"))
        .expect("release evidence build has Node available");
    assert_eq!(setup_node["with"]["node-version"].as_str(), Some("22"));
    assert!(evidence_steps.iter().any(|step| {
        step["run"].as_str().is_some_and(|run| {
            run.contains("rustup show active-toolchain || rustup toolchain install")
        })
    }));
    assert!(evidence_steps.iter().any(|step| {
        step["uses"].as_str() == Some("agent-ix/nodejs-actions/setup-npmrc@main")
            && step["with"]["npm_registry_token"].as_str() == Some("${{ secrets.REGISTRY_TOKEN }}")
    }));
    for (repository, revision, path, variable) in [
        (
            "agent-ix/spec-artifacts-process",
            "c29697dd2c083e08defdec73d31efa3f41141a80",
            ".ci/spec-artifacts-process",
            "VALIDATION_PROCESS_ROOT",
        ),
        (
            "agent-ix/spec-artifacts-iso",
            "f3e556c423eee451ac833eb1c0a950006c137bc6",
            ".ci/spec-artifacts-iso",
            "VALIDATION_ISO_ROOT",
        ),
    ] {
        assert!(evidence_steps.iter().any(|step| {
            step["uses"].as_str() == Some("actions/checkout@v7")
                && step["with"]["repository"].as_str() == Some(repository)
                && step["with"]["ref"].as_str() == Some(revision)
                && step["with"]["path"].as_str() == Some(path)
        }));
        assert_eq!(
            generate["env"][variable].as_str(),
            Some(match variable {
                "VALIDATION_PROCESS_ROOT" => {
                    "${{ github.workspace }}/.ci/spec-artifacts-process"
                }
                _ => "${{ github.workspace }}/.ci/spec-artifacts-iso",
            })
        );
    }
    assert_eq!(
        generate["env"]["CARGO_PROFILE_DEV_DEBUG"].as_str(),
        Some("0")
    );
    assert_eq!(
        generate["env"]["CARGO_PROFILE_TEST_DEBUG"].as_str(),
        Some("0")
    );
    assert_eq!(generate["env"]["CARGO_INCREMENTAL"].as_str(), Some("0"));
    assert_eq!(
        events(&workflow)["workflow_call"]["secrets"]["REGISTRY_TOKEN"]["required"].as_bool(),
        Some(true)
    );
    assert_eq!(
        generate["env"]["RELEASE_TAG"].as_str(),
        Some("${{ github.ref_name }}")
    );
    assert_eq!(
        generate["env"]["NPM_CONFIG_USERCONFIG"].as_str(),
        Some("${{ github.workspace }}/.npmrc")
    );
    assert!(target.contains("tag=$${RELEASE_TAG:-$$(git describe --tags --exact-match HEAD)}"));
}

const RELEASE_RUN_PATH: &str = "runs/SUITE-001/2026-11-26T00~3A00~3A00~2E123456789Z.json";

fn release_bundle_fixture(
    root: &std::path::Path,
    commit: &str,
    outcome: &str,
) -> (PathBuf, PathBuf, PathBuf) {
    let source_root = root.join("source");
    let source_evidence = source_root.join("spec/evidence");
    let bundle_root = root.join("bundle");
    let run_path = RELEASE_RUN_PATH.to_owned();
    fs::create_dir_all(source_evidence.join("runs/SUITE-001")).expect("create source evidence");
    fs::create_dir_all(bundle_root.join("runs/SUITE-001")).expect("create bundle evidence");

    let suite_registry = b"id: SUR-001\ntitle: release suites\n\nFor the release Test Result, SUITE-001 SHALL bind each acceptance criterion in Quire-rs FR-095 (AC-1 through AC-6).\n";
    let required_bindings = (1..=6)
        .map(|criterion| {
            serde_json::json!({
                "obligation": format!("FR-095-AC-{criterion}"),
                "statementHashAtBinding": "sha256:statement",
                "suite": "SUITE-001",
                "symbols": [format!("tests::tc_{}", 2228 + criterion)]
            })
        })
        .collect::<Vec<_>>();
    let all_bindings = serde_json::json!({
        "schemaVersion": 1,
        "bindings": required_bindings
    });
    let committed_bindings =
        serde_json::to_vec(&all_bindings).expect("serialize committed bindings");
    let generated_bindings =
        serde_json::to_vec_pretty(&all_bindings).expect("serialize generated bindings");
    let test_entries = (1..=6)
        .map(|criterion| {
            serde_json::json!({
                "symbol": format!("tests::tc_{}", 2228 + criterion),
                "outcome": outcome
            })
        })
        .collect::<Vec<_>>();
    let junit_cases = (1..=6)
        .map(|criterion| {
            format!(
                "<testcase classname=\"tests\" name=\"tc_{}\"/>",
                2228 + criterion
            )
        })
        .collect::<String>();
    fs::write(source_evidence.join("suites.md"), suite_registry).expect("write suite registry");
    fs::write(source_evidence.join("bindings.json"), &committed_bindings)
        .expect("write committed bindings");
    let run = serde_json::json!({
        "schemaVersion": 1,
        "suite": "SUITE-001",
        "tool": "ix-trace-run 0.1.0+8a0ec153",
        "evidenceKind": "Unit",
        "timestamp": "2026-11-26T00:00:00.123456789Z",
        "entries": test_entries
    });
    let manifest = serde_json::json!({
        "schemaVersion": 1,
        "tag": "v0.63.3",
        "commit": commit,
        "tagCreatedAt": 1795564800,
        "suite": "SUITE-001",
        "runPath": run_path,
        "tool": "ix-trace-run 0.1.0+8a0ec153"
    });
    fs::write(bundle_root.join("suites.md"), suite_registry).expect("write bundle suite registry");
    fs::write(
        bundle_root.join("bindings.committed.json"),
        &committed_bindings,
    )
    .expect("write bundle committed bindings");
    fs::write(source_evidence.join("bindings.json"), &generated_bindings)
        .expect("write generated source bindings");
    fs::write(bundle_root.join("bindings.json"), &generated_bindings)
        .expect("write generated bindings");
    fs::write(
        bundle_root.join("release-evidence.json"),
        serde_json::to_vec_pretty(&manifest).expect("serialize manifest"),
    )
    .expect("write manifest");
    fs::write(
        bundle_root.join(&run_path),
        serde_json::to_vec_pretty(&run).expect("serialize run"),
    )
    .expect("write run record");
    fs::write(
        bundle_root.join("junit.xml"),
        format!("<testsuite>{junit_cases}</testsuite>"),
    )
    .expect("write raw JUnit result");
    let hashes = Command::new("python3")
        .arg("-c")
        .arg("import hashlib,json,sys; from pathlib import Path; root=Path(sys.argv[1]); p=root/'release-evidence.json'; m=json.loads(p.read_text()); m['sha256']={f.relative_to(root).as_posix():hashlib.sha256(f.read_bytes()).hexdigest() for f in root.rglob('*') if f.is_file() and f!=p}; p.write_text(json.dumps(m))")
        .arg(&bundle_root)
        .status()
        .expect("hash exact raw fixture payloads");
    assert!(hashes.success());
    let source_run = source_evidence.join(&run_path);
    fs::create_dir_all(source_run.parent().expect("run path has a parent"))
        .expect("create source run directory");
    fs::copy(bundle_root.join(&run_path), source_run).expect("copy run into source evidence");

    let archive_path = root.join("release-evidence.tar.gz");
    let archive = Command::new("tar")
        .arg("-czf")
        .arg(&archive_path)
        .arg("-C")
        .arg(&bundle_root)
        .args([
            "release-evidence.json",
            "suites.md",
            "bindings.committed.json",
            "bindings.json",
            "junit.xml",
        ])
        .arg(run_path)
        .status()
        .expect("run tar to create fixture archive");
    assert!(archive.success(), "fixture tar command succeeds");
    (source_root, archive_path, bundle_root)
}

fn rearchive_release_bundle(bundle_root: &std::path::Path, archive_path: &std::path::Path) {
    let hashes = Command::new("python3")
        .arg("-c")
        .arg("import hashlib,json,sys; from pathlib import Path; root=Path(sys.argv[1]); p=root/'release-evidence.json'; m=json.loads(p.read_text()); m['sha256']={f.relative_to(root).as_posix():hashlib.sha256(f.read_bytes()).hexdigest() for f in root.rglob('*') if f.is_file() and f!=p}; p.write_text(json.dumps(m))")
        .arg(bundle_root)
        .status()
        .expect("rehash exact release fixture payloads");
    assert!(hashes.success(), "fixture payload hashes are refreshed");
    let archived = Command::new("tar")
        .arg("-czf")
        .arg(archive_path)
        .arg("-C")
        .arg(bundle_root)
        .args([
            "release-evidence.json",
            "suites.md",
            "bindings.committed.json",
            "bindings.json",
            "junit.xml",
            RELEASE_RUN_PATH,
        ])
        .status()
        .expect("rebuild release fixture archive");
    assert!(archived.success(), "tampered fixture is rearchived");
}

#[trace("FR-095-AC-4")]
#[test]
fn tc_2235_valid_release_bundle_is_verified_and_extracted_for_the_tag_sha() {
    let root = tempfile::tempdir().expect("create fixture root");
    let commit = "a".repeat(40);
    let (source_root, archive, bundle_root) = release_bundle_fixture(root.path(), &commit, "pass");
    let evidence_root = source_root.join("spec/evidence");
    let package =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/package_release_evidence.py");
    let packaged = Command::new("python3")
        .arg(&package)
        .args(["--source-root"])
        .arg(&source_root)
        .args(["--committed-bindings"])
        .arg(bundle_root.join("bindings.committed.json"))
        .args(["--run-record"])
        .arg(evidence_root.join(RELEASE_RUN_PATH))
        .arg("--junit-results")
        .arg(bundle_root.join("junit.xml"))
        .args(["--expected-tag", "v0.63.3", "--expected-commit", &commit])
        .args(["--tag-created-at", "1795564800"])
        .arg("--archive")
        .arg(&archive)
        .status()
        .expect("run release evidence packager");
    assert!(packaged.success(), "matching Terra run is packaged");
    fs::write(
        evidence_root.join("bindings.json"),
        fs::read(bundle_root.join("bindings.committed.json")).unwrap(),
    )
    .expect("restore tagged source bindings before verification");
    let output = root.path().join("verified");
    let verifier =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/verify_release_evidence.py");
    let result = Command::new("python3")
        .arg(verifier)
        .args(["--archive"])
        .arg(&archive)
        .args(["--source-root"])
        .arg(&source_root)
        .args(["--expected-tag", "v0.63.3", "--expected-commit", &commit])
        .args(["--tag-created-at", "1795564800"])
        .arg("--output-dir")
        .arg(&output)
        .status()
        .expect("run release evidence verifier");
    assert!(result.success(), "matching release evidence verifies");
    assert!(output.join(RELEASE_RUN_PATH).is_file());
    assert!(output.join("bindings.committed.json").is_file());
    assert_eq!(
        fs::read(output.join(RELEASE_RUN_PATH)).unwrap(),
        fs::read(evidence_root.join(RELEASE_RUN_PATH)).unwrap(),
        "raw Quoin record bytes are preserved"
    );
    let run: serde_json::Value =
        serde_json::from_slice(&fs::read(output.join(RELEASE_RUN_PATH)).unwrap()).unwrap();
    assert!(
        run.get("commit").is_none(),
        "published Quoin records have no invented commit field"
    );
}

#[trace("FR-095-AC-6")]
#[test]
fn tc_2236_wrong_tag_sha_refuses_terra_bundle_before_extraction() {
    let root = tempfile::tempdir().expect("create fixture root");
    let commit = "a".repeat(40);
    let (source_root, archive, _) = release_bundle_fixture(root.path(), &commit, "pass");
    let output = root.path().join("verified");
    let verifier =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/verify_release_evidence.py");
    let wrong_commit = "b".repeat(40);
    let result = Command::new("python3")
        .arg(verifier)
        .args(["--archive"])
        .arg(archive)
        .args(["--source-root"])
        .arg(source_root)
        .args([
            "--expected-tag",
            "v0.63.3",
            "--expected-commit",
            &wrong_commit,
        ])
        .args(["--tag-created-at", "1795564800"])
        .arg("--output-dir")
        .arg(&output)
        .status()
        .expect("run release evidence verifier");
    assert!(!result.success(), "wrong tag SHA refuses the bundle");
    assert!(!output.exists(), "refused evidence is not extracted");
}

#[trace("FR-095-AC-6")]
#[test]
fn tc_2237_changed_committed_registry_refuses_terra_bundle() {
    let root = tempfile::tempdir().expect("create fixture root");
    let commit = "c".repeat(40);
    let (source_root, archive, _) = release_bundle_fixture(root.path(), &commit, "pass");
    fs::write(
        source_root.join("spec/evidence/suites.md"),
        "id: SUR-001\ntitle: changed suite registry\n",
    )
    .expect("change current source registry");
    let output = root.path().join("verified");
    let verifier =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/verify_release_evidence.py");
    let result = Command::new("python3")
        .arg(verifier)
        .args(["--archive"])
        .arg(archive)
        .args(["--source-root"])
        .arg(source_root)
        .args(["--expected-tag", "v0.63.3", "--expected-commit", &commit])
        .args(["--tag-created-at", "1795564800"])
        .arg("--output-dir")
        .arg(&output)
        .status()
        .expect("run release evidence verifier");
    assert!(!result.success(), "changed registry refuses the bundle");
    assert!(!output.exists(), "refused evidence is not extracted");
}

#[trace("FR-095-AC-6")]
#[test]
fn tc_2238_failed_suite_record_refuses_terra_bundle() {
    let root = tempfile::tempdir().expect("create fixture root");
    let commit = "d".repeat(40);
    let (source_root, archive, _) = release_bundle_fixture(root.path(), &commit, "fail");
    let output = root.path().join("verified");
    let verifier =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/verify_release_evidence.py");
    let result = Command::new("python3")
        .arg(verifier)
        .args(["--archive"])
        .arg(archive)
        .args(["--source-root"])
        .arg(source_root)
        .args(["--expected-tag", "v0.63.3", "--expected-commit", &commit])
        .args(["--tag-created-at", "1795564800"])
        .arg("--output-dir")
        .arg(&output)
        .status()
        .expect("run release evidence verifier");
    assert!(!result.success(), "failed suite refuses the bundle");
    assert!(!output.exists(), "refused evidence is not extracted");
}

#[trace("FR-095-AC-6")]
#[test]
fn tc_2239_stale_run_record_refuses_terra_bundle() {
    let root = tempfile::tempdir().expect("create fixture root");
    let commit = "e".repeat(40);
    let (source_root, archive, _) = release_bundle_fixture(root.path(), &commit, "pass");
    let output = root.path().join("verified");
    let verifier =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/verify_release_evidence.py");
    let result = Command::new("python3")
        .arg(verifier)
        .args(["--archive"])
        .arg(archive)
        .args(["--source-root"])
        .arg(source_root)
        .args(["--expected-tag", "v0.63.3", "--expected-commit", &commit])
        .args(["--tag-created-at", "1795651200"])
        .arg("--output-dir")
        .arg(&output)
        .status()
        .expect("run release evidence verifier");
    assert!(!result.success(), "stale run refuses the bundle");
    assert!(!output.exists(), "refused evidence is not extracted");
}

#[trace("FR-095-AC-1")]
#[test]
fn tc_2241_make_evidence_refuses_a_dirty_checkout_before_running_tests() {
    let root = tempfile::tempdir().expect("create checkout fixture");
    let checkout = root.path();
    let git_init = Command::new("git")
        .args(["init", "--quiet"])
        .current_dir(checkout)
        .status()
        .expect("initialize synthetic checkout");
    assert!(git_init.success(), "synthetic checkout initialized");
    fs::create_dir(checkout.join("scripts")).expect("create scripts directory");
    let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    fs::copy(repository.join("Makefile"), checkout.join("Makefile"))
        .expect("copy actual evidence target");
    fs::copy(
        repository.join("scripts/check_release_evidence_checkout.py"),
        checkout.join("scripts/check_release_evidence_checkout.py"),
    )
    .expect("copy actual checkout preflight");
    fs::write(
        checkout.join(".git/info/exclude"),
        "\n/Makefile\n/scripts/check_release_evidence_checkout.py\n",
    )
    .expect("exclude only the copied implementation files");
    fs::write(checkout.join("src-dirty.rs"), "uncommitted source\n")
        .expect("create dirty untracked source");

    let result = Command::new("make")
        .args(["--no-print-directory", "evidence"])
        .current_dir(checkout)
        .output()
        .expect("run actual make evidence target");
    assert!(!result.status.success(), "dirty checkout is refused");
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        stderr.contains("release_evidence_dirty_checkout"),
        "dirty checkout diagnostic is stable: {stderr}"
    );
    assert!(
        !checkout.join("target/release-evidence").exists(),
        "refusal happens before evidence or test output is created"
    );

    let clone = root.path().join("uninitialized-submodule-checkout");
    let cloned = Command::new("git")
        .args(["clone", "--quiet", "--shared", "--no-recurse-submodules"])
        .arg(&repository)
        .arg(&clone)
        .status()
        .expect("clone checkout with submodule left uninitialized");
    assert!(
        cloned.success(),
        "synthetic checkout cloned without submodules"
    );
    let preflight = Command::new("python3")
        .arg(repository.join("scripts/check_release_evidence_checkout.py"))
        .args(["--repo-root"])
        .arg(&clone)
        .output()
        .expect("check uninitialized submodule checkout");
    assert!(
        !preflight.status.success(),
        "uninitialized recursive submodule is refused"
    );
    assert!(
        String::from_utf8_lossy(&preflight.stderr).contains("release_evidence_dirty_checkout"),
        "submodule refusal diagnostic is stable"
    );
}

#[trace("FR-095-AC-2")]
#[test]
fn tc_2242_streaming_archive_reader_enforces_member_and_byte_bounds() {
    let root = tempfile::tempdir().expect("create archive fixture root");
    let commit = "9".repeat(40);
    let (source_root, _, bundle_root) = release_bundle_fixture(root.path(), &commit, "pass");
    let verifier =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/verify_release_evidence.py");
    let many_archive = root.path().join("many-members.tar.gz");
    let oversized_archive = root.path().join("oversized-member.tar.gz");
    let compressed_archive = root.path().join("oversized-compressed.tar.gz");
    let fixtures = Command::new("python3")
        .arg("-c")
        .arg(
            "import io,os,sys,tarfile; from pathlib import Path; root=Path(sys.argv[1]); many=Path(sys.argv[2]); oversized=Path(sys.argv[3]); compressed=Path(sys.argv[4]); files=[p for p in root.rglob('*') if p.is_file()]; t=tarfile.open(many,'w:gz'); [t.add(p,arcname=p.relative_to(root).as_posix()) for p in files]; [(lambda info,data:t.addfile(info,io.BytesIO(data)))(tarfile.TarInfo('extra-'+str(i)),b'x') for i in range(10)]; t.close(); t=tarfile.open(oversized,'w:gz'); data=b'x'*30_000_000; [(lambda info:(setattr(info,'size',len(data)),t.addfile(info,io.BytesIO(data))))(tarfile.TarInfo('part-'+str(i))) for i in range(2)]; t.close(); t=tarfile.open(compressed,'w:gz'); data=os.urandom(20_000_001); info=tarfile.TarInfo('random'); info.size=len(data); t.addfile(info,io.BytesIO(data)); t.close()",
        )
        .arg(&bundle_root)
        .arg(&many_archive)
        .arg(&oversized_archive)
        .arg(&compressed_archive)
        .status()
        .expect("create adversarial archives");
    assert!(fixtures.success(), "adversarial archives created");

    for (label, archive) in [
        ("many", many_archive),
        ("decompressed", oversized_archive),
        ("compressed", compressed_archive),
    ] {
        let output = root.path().join(format!("{label}-output"));
        let result = Command::new("python3")
            .arg(&verifier)
            .args(["--archive"])
            .arg(archive)
            .args(["--source-root"])
            .arg(&source_root)
            .args(["--expected-tag", "v0.63.3", "--expected-commit", &commit])
            .args(["--tag-created-at", "1795564800"])
            .arg("--output-dir")
            .arg(&output)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("run streaming archive verifier");
        assert!(!result.success(), "{label} archive is refused");
        assert!(!output.exists(), "{label} archive is not extracted");
    }
}

#[trace("FR-095-AC-6")]
#[test]
fn tc_2240_unsafe_archive_path_and_missing_run_are_refused() {
    let root = tempfile::tempdir().expect("create fixture root");
    let commit = "f".repeat(40);
    let (source_root, valid_archive, bundle_root) =
        release_bundle_fixture(root.path(), &commit, "pass");
    let verifier =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/verify_release_evidence.py");
    let unsafe_archive = root.path().join("unsafe.tar.gz");
    let create_unsafe_archive = Command::new("python3")
        .arg("-c")
        .arg(
            "import io,sys,tarfile; from pathlib import Path; root=Path(sys.argv[2]); run_path='runs/SUITE-001/'+'f'*12+'.json'; archive=tarfile.open(sys.argv[1], 'w:gz'); [archive.add(path, arcname=path.relative_to(root).as_posix()) for path in root.rglob('*') if path.is_file() and path.relative_to(root).as_posix()!=run_path]; info=tarfile.TarInfo('../escape'); info.size=1; archive.addfile(info, io.BytesIO(b'x')); archive.close()",
        )
        .arg(&unsafe_archive)
        .arg(&bundle_root)
        .status()
        .expect("create unsafe archive fixture");
    assert!(
        create_unsafe_archive.success(),
        "unsafe fixture archive created"
    );
    let unsafe_output = root.path().join("unsafe-output");
    let unsafe_result = Command::new("python3")
        .arg(&verifier)
        .args(["--archive"])
        .arg(&unsafe_archive)
        .args(["--source-root"])
        .arg(&source_root)
        .args(["--expected-tag", "v0.63.3", "--expected-commit", &commit])
        .args(["--tag-created-at", "1795564800"])
        .arg("--output-dir")
        .arg(&unsafe_output)
        .status()
        .expect("run release evidence verifier");
    assert!(!unsafe_result.success(), "unsafe archive member is refused");
    assert!(!unsafe_output.exists(), "unsafe archive is not extracted");

    let missing_run_archive = root.path().join("missing-run.tar.gz");
    let missing_run_created = Command::new("tar")
        .arg("-czf")
        .arg(&missing_run_archive)
        .arg("-C")
        .arg(&bundle_root)
        .args([
            "release-evidence.json",
            "suites.md",
            "bindings.committed.json",
            "bindings.json",
        ])
        .status()
        .expect("create archive without the run record");
    assert!(
        missing_run_created.success(),
        "incomplete fixture archive created"
    );
    let missing_run_output = root.path().join("missing-run-output");
    let missing_run_result = Command::new("python3")
        .arg(&verifier)
        .args(["--archive"])
        .arg(&missing_run_archive)
        .args(["--source-root"])
        .arg(&source_root)
        .args(["--expected-tag", "v0.63.3", "--expected-commit", &commit])
        .args(["--tag-created-at", "1795564800"])
        .arg("--output-dir")
        .arg(&missing_run_output)
        .status()
        .expect("run release evidence verifier");
    assert!(
        !missing_run_result.success(),
        "missing run record is refused"
    );
    assert!(
        !missing_run_output.exists(),
        "incomplete archive is not extracted"
    );

    let occupied_output = root.path().join("occupied-output");
    fs::create_dir(&occupied_output).expect("create occupied destination");
    fs::write(occupied_output.join("keep.txt"), "preserve").expect("write sentinel");
    let occupied_result = Command::new("python3")
        .arg(&verifier)
        .args(["--archive"])
        .arg(&valid_archive)
        .args(["--source-root"])
        .arg(&source_root)
        .args(["--expected-tag", "v0.63.3", "--expected-commit", &commit])
        .args(["--tag-created-at", "1795564800"])
        .arg("--output-dir")
        .arg(&occupied_output)
        .status()
        .expect("run release evidence verifier");
    assert!(
        !occupied_result.success(),
        "existing extraction destination is refused"
    );
    assert_eq!(
        fs::read_to_string(occupied_output.join("keep.txt")).expect("read preserved sentinel"),
        "preserve",
        "refusal preserves the caller's existing destination"
    );
}

#[trace("FR-095-AC-6")]
#[test]
fn tc_2243_changed_raw_junit_payload_is_refused() {
    let root = tempfile::tempdir().expect("create fixture root");
    let commit = "a".repeat(40);
    let (source, archive, bundle) = release_bundle_fixture(root.path(), &commit, "pass");
    fs::write(bundle.join("junit.xml"), "changed raw producer result").expect("tamper JUnit");
    let tar = Command::new("tar")
        .arg("-czf")
        .arg(&archive)
        .arg("-C")
        .arg(&bundle)
        .args([
            "release-evidence.json",
            "suites.md",
            "bindings.committed.json",
            "bindings.json",
            "junit.xml",
            RELEASE_RUN_PATH,
        ])
        .status()
        .expect("repack tampered payload");
    assert!(tar.success());
    let output = root.path().join("verified");
    let result = Command::new("python3")
        .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/verify_release_evidence.py"))
        .arg("--archive")
        .arg(archive)
        .arg("--source-root")
        .arg(source)
        .args([
            "--expected-tag",
            "v0.63.3",
            "--expected-commit",
            &commit,
            "--tag-created-at",
            "1795564800",
        ])
        .arg("--output-dir")
        .arg(&output)
        .output()
        .expect("verify tampered payload");
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("payload hashes"));
    assert!(!output.exists());
}

#[trace("FR-095-AC-4", "FR-095-AC-6")]
#[test]
fn release_evidence_refuses_retired_or_unknown_producers() {
    let case: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../fixtures/corpus_cases/plat-1934-producer-identity.json"
    ))
    .expect("attributed producer identity fixture");
    for tool in case["input"]["refused_tools"].as_array().unwrap() {
        let tool = tool.as_str().unwrap();
        let root = tempfile::tempdir().unwrap();
        let commit = "a".repeat(40);
        let (source, archive, bundle) = release_bundle_fixture(root.path(), &commit, "pass");
        let run_path = source.join("spec/evidence").join(RELEASE_RUN_PATH);
        let mut run: serde_json::Value =
            serde_json::from_slice(&fs::read(&run_path).unwrap()).unwrap();
        run["tool"] = serde_json::json!(tool);
        fs::write(&run_path, serde_json::to_vec(&run).unwrap()).unwrap();
        fs::write(
            bundle.join(RELEASE_RUN_PATH),
            serde_json::to_vec(&run).unwrap(),
        )
        .unwrap();
        let rejected_archive = root.path().join("rejected.tar.gz");
        let packaged = Command::new("python3")
            .arg(
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("scripts/package_release_evidence.py"),
            )
            .arg("--source-root")
            .arg(&source)
            .arg("--committed-bindings")
            .arg(bundle.join("bindings.committed.json"))
            .arg("--run-record")
            .arg(&run_path)
            .arg("--junit-results")
            .arg(bundle.join("junit.xml"))
            .args([
                "--expected-tag",
                "v0.63.3",
                "--expected-commit",
                &commit,
                "--tag-created-at",
                "1795564800",
            ])
            .arg("--archive")
            .arg(&rejected_archive)
            .output()
            .unwrap();
        assert!(!packaged.status.success(), "{tool}");
        assert!(
            String::from_utf8_lossy(&packaged.stderr)
                .contains(case["expect"]["producer_refuses"].as_str().unwrap()),
            "{tool}: {:?}",
            packaged
        );
        assert!(!rejected_archive.exists());

        // Keep every payload hash correct, so only the retired/unknown tool
        // identity makes the otherwise-valid archive fail verification.
        let hashed = Command::new("python3").arg("-c")
            .arg("import hashlib,json,sys; from pathlib import Path; root=Path(sys.argv[1]); p=root/'release-evidence.json'; m=json.loads(p.read_text()); m['tool']=sys.argv[2]; m['sha256']={f.relative_to(root).as_posix():hashlib.sha256(f.read_bytes()).hexdigest() for f in root.rglob('*') if f.is_file() and f!=p}; p.write_text(json.dumps(m))")
            .arg(&bundle).arg(tool).status().unwrap();
        assert!(hashed.success());
        let archived = Command::new("tar")
            .arg("-czf")
            .arg(&archive)
            .arg("-C")
            .arg(&bundle)
            .args([
                "release-evidence.json",
                "suites.md",
                "bindings.committed.json",
                "bindings.json",
                "junit.xml",
                RELEASE_RUN_PATH,
            ])
            .status()
            .unwrap();
        assert!(archived.success());
        let output = root.path().join("verified");
        let verified = Command::new("python3")
            .arg(
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("scripts/verify_release_evidence.py"),
            )
            .arg("--archive")
            .arg(&archive)
            .arg("--source-root")
            .arg(&source)
            .args([
                "--expected-tag",
                "v0.63.3",
                "--expected-commit",
                &commit,
                "--tag-created-at",
                "1795564800",
            ])
            .arg("--output-dir")
            .arg(&output)
            .output()
            .unwrap();
        assert!(!verified.status.success(), "{tool}");
        assert!(
            String::from_utf8_lossy(&verified.stderr)
                .contains(case["expect"]["verifier_refuses"].as_str().unwrap()),
            "{tool}: {:?}",
            verified
        );
        assert!(!output.exists());
    }
}

#[trace("FR-095-AC-6")]
#[test]
fn tc_2245_missing_required_release_binding_is_refused_by_package_and_verifier() {
    let root = tempfile::tempdir().expect("create fixture root");
    let commit = "e".repeat(40);
    let (source, archive, bundle) = release_bundle_fixture(root.path(), &commit, "pass");

    let generated_path = bundle.join("bindings.json");
    let mut generated: serde_json::Value =
        serde_json::from_slice(&fs::read(&generated_path).expect("read generated bindings"))
            .expect("parse generated bindings");
    generated["bindings"]
        .as_array_mut()
        .expect("binding list")
        .retain(|binding| binding["obligation"] != "FR-095-AC-5");
    let generated_bytes =
        serde_json::to_vec_pretty(&generated).expect("serialize missing-binding fixture");
    fs::write(&generated_path, &generated_bytes).expect("write missing-binding bundle");
    fs::write(source.join("spec/evidence/bindings.json"), &generated_bytes)
        .expect("write missing generated registry for packaging");
    rearchive_release_bundle(&bundle, &archive);

    let evidence = source.join("spec/evidence");
    let package =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/package_release_evidence.py");
    let rejected_archive = root.path().join("missing-ac5.tar.gz");
    let packaged = Command::new("python3")
        .arg(&package)
        .arg("--source-root")
        .arg(&source)
        .arg("--committed-bindings")
        .arg(bundle.join("bindings.committed.json"))
        .arg("--run-record")
        .arg(evidence.join(RELEASE_RUN_PATH))
        .arg("--junit-results")
        .arg(bundle.join("junit.xml"))
        .args([
            "--expected-tag",
            "v0.63.3",
            "--expected-commit",
            &commit,
            "--tag-created-at",
            "1795564800",
        ])
        .arg("--archive")
        .arg(&rejected_archive)
        .output()
        .expect("run package validator against missing binding");
    assert!(
        !packaged.status.success(),
        "packager refuses a missing AC-5 binding"
    );
    assert!(
        String::from_utf8_lossy(&packaged.stderr).contains("FR-095-AC-5"),
        "{:?}",
        packaged
    );
    fs::write(
        source.join("spec/evidence/bindings.json"),
        fs::read(bundle.join("bindings.committed.json")).expect("read committed bindings"),
    )
    .expect("restore tagged source bindings before verification");
    assert!(
        !rejected_archive.exists(),
        "invalid binding is not packaged"
    );

    let output = root.path().join("verified-missing-ac5");
    let verifier =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/verify_release_evidence.py");
    let verified = Command::new("python3")
        .arg(&verifier)
        .arg("--archive")
        .arg(&archive)
        .arg("--source-root")
        .arg(&source)
        .args([
            "--expected-tag",
            "v0.63.3",
            "--expected-commit",
            &commit,
            "--tag-created-at",
            "1795564800",
        ])
        .arg("--output-dir")
        .arg(&output)
        .output()
        .expect("run release verifier against missing binding");
    assert!(
        !verified.status.success(),
        "verifier refuses a missing AC-5 binding"
    );
    assert!(
        String::from_utf8_lossy(&verified.stderr).contains("FR-095-AC-5"),
        "{:?}",
        verified
    );
    assert!(!output.exists(), "invalid evidence is not extracted");

    let mut unsupported: serde_json::Value = serde_json::from_slice(
        &fs::read(bundle.join("bindings.committed.json")).expect("read complete bindings"),
    )
    .expect("parse complete bindings");
    unsupported["bindings"][0]["symbols"] = serde_json::json!(["tests::never_executed"]);
    let unsupported_bytes =
        serde_json::to_vec_pretty(&unsupported).expect("serialize unsupported binding fixture");
    fs::write(&generated_path, &unsupported_bytes).expect("write unsupported bundle bindings");
    fs::write(
        source.join("spec/evidence/bindings.json"),
        &unsupported_bytes,
    )
    .expect("write unsupported source bindings");
    rearchive_release_bundle(&bundle, &archive);

    let package_without_support = Command::new("python3")
        .arg(&package)
        .arg("--source-root")
        .arg(&source)
        .arg("--committed-bindings")
        .arg(bundle.join("bindings.committed.json"))
        .arg("--run-record")
        .arg(evidence.join(RELEASE_RUN_PATH))
        .arg("--junit-results")
        .arg(bundle.join("junit.xml"))
        .args([
            "--expected-tag",
            "v0.63.3",
            "--expected-commit",
            &commit,
            "--tag-created-at",
            "1795564800",
        ])
        .arg("--archive")
        .arg(root.path().join("unsupported-binding.tar.gz"))
        .output()
        .expect("run package validator against unsupported test symbol");
    assert!(
        !package_without_support.status.success(),
        "packager rejects a required binding with no passing run symbol"
    );
    assert!(
        String::from_utf8_lossy(&package_without_support.stderr).contains("tests::never_executed")
    );
    fs::write(
        source.join("spec/evidence/bindings.json"),
        fs::read(bundle.join("bindings.committed.json")).expect("read committed bindings"),
    )
    .expect("restore tagged source bindings before verification");

    let verify_without_support = Command::new("python3")
        .arg(&verifier)
        .arg("--archive")
        .arg(&archive)
        .arg("--source-root")
        .arg(&source)
        .args([
            "--expected-tag",
            "v0.63.3",
            "--expected-commit",
            &commit,
            "--tag-created-at",
            "1795564800",
        ])
        .arg("--output-dir")
        .arg(root.path().join("verified-unsupported-binding"))
        .output()
        .expect("run release verifier against unsupported test symbol");
    assert!(
        !verify_without_support.status.success(),
        "verifier rejects a required binding with no passing run symbol"
    );
    assert!(
        String::from_utf8_lossy(&verify_without_support.stderr).contains("tests::never_executed")
    );
}

#[trace("FR-095-AC-5")]
#[test]
fn tc_2230_attestation_requires_successful_evidence_generation_and_upload() {
    let tag_attestation = workflow("tag-attestation.yml");
    let evidence = &tag_attestation["jobs"]["release-evidence"];
    let steps = evidence["steps"].as_sequence().expect("evidence steps");
    let upload = steps
        .iter()
        .find(|step| step["uses"].as_str() == Some("actions/upload-artifact@v7"))
        .expect("verified evidence is uploaded");
    assert_eq!(
        upload["with"]["name"].as_str(),
        Some("${{ inputs.evidence_artifact_name || 'release-evidence' }}")
    );
    assert_eq!(
        tag_attestation["jobs"]["attest"]["needs"],
        serde_yaml::from_str::<Value>("[release-artifacts, release-evidence]").unwrap()
    );
    assert!(tag_attestation["jobs"]["attest"]["if"]
        .as_str()
        .expect("attestation fails closed")
        .contains("needs.release-evidence.result == 'success'"));
}

#[trace("FR-095-AC-7")]
#[test]
fn tc_2246_reusable_release_workflow_has_tag_guards_and_typed_inputs() {
    let tag_attestation = workflow("tag-attestation.yml");
    let call = event(events(&tag_attestation), "workflow_call");
    let inputs = call["inputs"].as_mapping().expect("workflow_call inputs");
    for name in [
        "run_wheels",
        "release_artifact_name",
        "evidence_artifact_name",
        "ea_tooling_tag",
        "quoin_tooling_tag",
    ] {
        assert!(
            inputs.contains_key(Value::String(name.to_owned())),
            "missing reusable input {name}"
        );
    }
    assert_eq!(inputs["run_wheels"]["type"], "boolean");
    for name in [
        "release_artifact_name",
        "evidence_artifact_name",
        "ea_tooling_tag",
        "quoin_tooling_tag",
    ] {
        assert_eq!(inputs[name]["type"], "string");
    }
    for job in ["build", "release-artifacts"] {
        let guard = tag_attestation["jobs"][job]["if"]
            .as_str()
            .expect("wheel job tag and reuse guard");
        assert!(guard.contains("inputs.run_wheels"));
        assert!(guard.contains("github.repository == 'agent-ix/quire-rs'"));
        assert!(guard.contains("github.workflow_ref"));
        assert!(guard.contains("startsWith(github.ref, 'refs/tags/v')"));
    }
    for job in ["release-evidence", "attest"] {
        assert!(tag_attestation["jobs"][job]["if"]
            .as_str()
            .expect("release evidence jobs are tag-only")
            .contains("startsWith(github.ref, 'refs/tags/v')"));
    }
    assert_eq!(
        tag_attestation["jobs"]["attest"]["with"]["evidence_artifact"],
        "${{ inputs.evidence_artifact_name || 'release-evidence' }}"
    );
    let evidence_steps = tag_attestation["jobs"]["release-evidence"]["steps"]
        .as_sequence()
        .expect("evidence retention steps");
    let retained = evidence_steps
        .iter()
        .find(|step| step["uses"].as_str() == Some("actions/upload-artifact@v7"))
        .expect("verified evidence is uploaded");
    assert_eq!(
        retained["with"]["name"],
        "${{ inputs.evidence_artifact_name || 'release-evidence' }}"
    );
}

#[trace("FR-095-AC-8")]
#[test]
fn tc_2247_adoption_how_to_calls_pinned_workflow_for_generated_release_evidence() {
    let how_to = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("docs/how-to-adopt-quoin-release-evidence.md"),
    )
    .expect("read release evidence adoption how-to");
    assert!(how_to.contains("agent-ix/quire-rs/.github/workflows/tag-attestation.yml@"));
    assert!(how_to.contains("tags:"));
    assert!(how_to.contains("make evidence"));
    assert!(how_to.contains("secrets: inherit"));
    assert!(how_to.contains("run_wheels: false"));
    assert!(how_to.contains("release_artifact_name:"));
    assert!(how_to.contains("evidence_artifact_name:"));
    assert!(how_to.contains("quoin_tooling_tag: 0.36.10"));
    assert!(!how_to.contains("copy .github/workflows/tag-attestation.yml"));
}

#[trace("FR-066-AC-9")]
#[test]
fn tc_2360_gap_census_is_manual_or_scheduled_and_bounded_to_explicit_inputs() {
    let census = workflow("census.yml");
    let census_events = events(&census);
    assert!(
        census_events.contains_key(Value::String("workflow_dispatch".to_owned()))
            || census_events.contains_key(Value::String("schedule".to_owned())),
        "census must have a manual or scheduled trigger"
    );
    assert!(
        !census_events.contains_key(Value::String("push".to_owned()))
            && !census_events.contains_key(Value::String("pull_request".to_owned())),
        "census must not run as a push or pull-request prerequisite"
    );

    let census_steps = census["jobs"]["census"]["steps"]
        .as_sequence()
        .expect("census job steps");
    let run = census_steps
        .iter()
        .filter_map(|step| step["run"].as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(run.contains("make census"), "workflow must invoke make census");
    assert!(
        run.contains("CENSUS_ROOT") && run.contains("CENSUS_MODULE"),
        "workflow must provide explicit bounded root and module inputs"
    );
}
