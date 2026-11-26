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
use std::process::Command;

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
        1,
        "tag workflow is independent of manual release"
    );
    assert_eq!(event(tag_events, "push")["tags"][0].as_str(), Some("v*"));
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
    assert!(!evidence_steps
        .iter()
        .any(|step| step["run"].as_str() == Some("make evidence")));
    let wait_index = evidence_steps
        .iter()
        .position(|step| {
            step["name"].as_str()
                == Some("Wait for the Terra evidence asset on the matching release")
        })
        .expect("tag workflow waits for the Terra release asset");
    let tag_identity_index = evidence_steps
        .iter()
        .position(|step| step["id"].as_str() == Some("tag"))
        .expect("tag SHA and annotated creation time are checked");
    let verify_index = evidence_steps
        .iter()
        .position(|step| {
            step["run"]
                .as_str()
                .is_some_and(|run| run.contains("scripts/verify_release_evidence.py"))
        })
        .expect("tag workflow validates the downloaded release asset");
    let upload_index = evidence_steps
        .iter()
        .position(|step| step["with"]["name"].as_str() == Some("release-evidence"))
        .expect("release evidence job uploads the named artifact");
    assert!(
        tag_identity_index < wait_index && wait_index < verify_index && verify_index < upload_index,
        "asset download and validation precede artifact upload"
    );
    assert_eq!(
        evidence_steps[upload_index]["with"]["path"].as_str(),
        Some("release-evidence/")
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
    assert!(steps.iter().any(|step| step["run"]
        .as_str()
        .is_some_and(|run| { run.contains("release_evidence_asset_missing_or_unavailable") })));
}

#[trace("FR-095-AC-3")]
#[test]
fn tc_2233_oidc_attestation_consumes_artifact_without_running_checkout_code() {
    let tag_attestation = workflow("tag-attestation.yml");
    let attest = &tag_attestation["jobs"]["attest"];
    assert_eq!(
        attest["with"]["evidence_artifact"].as_str(),
        Some("release-evidence")
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
    let target = makefile
        .split("evidence:")
        .nth(1)
        .and_then(|body| body.split("\n\n").next())
        .expect("Makefile defines evidence target");
    assert!(target.contains("ix-trace-run"));
    assert!(target.contains("git describe --tags --exact-match HEAD"));
    assert!(target.contains("git rev-parse \"$$tag^{commit}\""));
    assert!(target.contains("git for-each-ref --format='%(taggerdate:unix)'"));
    assert!(target.contains("--tag-created-at \"$$tag_created_at\""));
    assert!(target.contains("CARGO_BUILD_JOBS=4"));
    assert!(target.contains("--workspace --jobs=4"));
    assert!(target.contains("evidence record --repo . --suite SUITE-001"));
    assert!(target.contains("--commit \"$$commit\""));
    assert!(target.contains("release-evidence.tar.gz"));
    assert!(target.contains("scripts/package_release_evidence.py"));
    assert!(target.contains("@agent-ix/quoin@0.30.1"));
    assert!(target.contains("$(RELEASE_QUOIN)"));
    assert!(target.contains("--tool 'ix-trace-run 0.1.0+3cd3285'"));

    let workflow = workflow("tag-attestation.yml");
    assert!(!workflow["jobs"]["release-evidence"]["steps"]
        .as_sequence()
        .expect("release evidence job has steps")
        .iter()
        .any(|step| step["run"].as_str() == Some("make evidence")));
}

fn terra_bundle_fixture(
    root: &std::path::Path,
    commit: &str,
    outcome: &str,
) -> (PathBuf, PathBuf, PathBuf) {
    let source_root = root.join("source");
    let source_evidence = source_root.join("spec/evidence");
    let bundle_root = root.join("bundle");
    let run_path = format!("runs/SUITE-001/{}.json", &commit[..12]);
    fs::create_dir_all(source_evidence.join("runs/SUITE-001")).expect("create source evidence");
    fs::create_dir_all(bundle_root.join("runs/SUITE-001")).expect("create bundle evidence");

    let suite_registry = b"id: SUR-001\ntitle: release suites\n";
    let committed_bindings = b"{\"bindings\":[],\"schemaVersion\":1}\n";
    fs::write(source_evidence.join("suites.md"), suite_registry).expect("write suite registry");
    fs::write(source_evidence.join("bindings.json"), committed_bindings)
        .expect("write committed bindings");

    let generated_bindings = serde_json::json!({
        "schemaVersion": 1,
        "bindings": [{
            "obligation": "FR-095-AC-1",
            "statementHashAtBinding": "sha256:statement",
            "suite": "SUITE-001",
            "commit": commit,
            "symbols": ["tests::tc_2229"]
        }]
    });
    let run = serde_json::json!({
        "schemaVersion": 1,
        "suite": "SUITE-001",
        "commit": commit,
        "tool": "ix-trace-run 0.1.0+3cd3285",
        "evidenceKind": "Unit",
        "timestamp": "2026-11-26T00:00:00Z",
        "entries": [{"symbol":"tests::tc_2229", "outcome":outcome}]
    });
    let manifest = serde_json::json!({
        "schemaVersion": 1,
        "tag": "v0.63.3",
        "commit": commit,
        "tagCreatedAt": 1795564800,
        "suite": "SUITE-001",
        "runPath": run_path,
        "tool": "ix-trace-run 0.1.0+3cd3285"
    });
    fs::write(bundle_root.join("suites.md"), suite_registry).expect("write bundle suite registry");
    fs::write(
        bundle_root.join("bindings.committed.json"),
        committed_bindings,
    )
    .expect("write bundle committed bindings");
    fs::write(
        bundle_root.join("bindings.json"),
        serde_json::to_vec_pretty(&generated_bindings).expect("serialize generated bindings"),
    )
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
        ])
        .arg(run_path)
        .status()
        .expect("run tar to create fixture archive");
    assert!(archive.success(), "fixture tar command succeeds");
    (source_root, archive_path, bundle_root)
}

#[trace("FR-095-AC-4")]
#[test]
fn tc_2235_valid_terra_bundle_is_verified_and_extracted_for_the_tag_sha() {
    let root = tempfile::tempdir().expect("create fixture root");
    let commit = "a".repeat(40);
    let (source_root, archive, _) = terra_bundle_fixture(root.path(), &commit, "pass");
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
    assert!(result.success(), "matching Terra evidence verifies");
    assert!(output.join("runs/SUITE-001/aaaaaaaaaaaa.json").is_file());
    assert!(output.join("bindings.committed.json").is_file());
}

#[trace("FR-095-AC-6")]
#[test]
fn tc_2236_wrong_tag_sha_refuses_terra_bundle_before_extraction() {
    let root = tempfile::tempdir().expect("create fixture root");
    let commit = "a".repeat(40);
    let (source_root, archive, _) = terra_bundle_fixture(root.path(), &commit, "pass");
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
    let (source_root, archive, _) = terra_bundle_fixture(root.path(), &commit, "pass");
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
    let (source_root, archive, _) = terra_bundle_fixture(root.path(), &commit, "fail");
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
    let (source_root, archive, _) = terra_bundle_fixture(root.path(), &commit, "pass");
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

#[trace("FR-095-AC-6")]
#[test]
fn tc_2240_unsafe_archive_path_and_missing_run_are_refused() {
    let root = tempfile::tempdir().expect("create fixture root");
    let commit = "f".repeat(40);
    let (source_root, _, bundle_root) = terra_bundle_fixture(root.path(), &commit, "pass");
    let verifier =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/verify_release_evidence.py");
    let unsafe_archive = root.path().join("unsafe.tar.gz");
    let create_unsafe_archive = Command::new("python3")
        .arg("-c")
        .arg(
            "import io,sys,tarfile; from pathlib import Path; root=Path(sys.argv[2]); archive=tarfile.open(sys.argv[1], 'w:gz'); [archive.add(path, arcname=path.relative_to(root).as_posix()) for path in root.rglob('*') if path.is_file()]; info=tarfile.TarInfo('../escape'); info.size=1; archive.addfile(info, io.BytesIO(b'x')); archive.close()",
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
        .arg(&archive)
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
        fs::read_to_string(occupied_output.join("keep.txt")).as_deref(),
        Ok("preserve"),
        "refusal preserves the caller's existing destination"
    );
}
