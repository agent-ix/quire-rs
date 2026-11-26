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
    let run_index = evidence_steps
        .iter()
        .position(|step| step["run"].as_str() == Some("make evidence"))
        .expect("release evidence job runs make evidence");
    let upload_index = evidence_steps
        .iter()
        .position(|step| step["with"]["name"].as_str() == Some("release-evidence"))
        .expect("release evidence job uploads the named artifact");
    assert!(
        run_index < upload_index,
        "upload follows successful suite run"
    );
    assert_eq!(
        evidence_steps[upload_index]["with"]["path"].as_str(),
        Some("spec/evidence/")
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
    assert!(target.contains("evidence record --repo . --suite SUITE-001"));
    assert!(target.contains("--commit \"$$(git rev-parse HEAD)\""));
    assert!(target.contains("--results \"$(EVIDENCE_TARGET)/junit.xml\""));
    assert!(target.contains("--tool 'ix-trace-run 0.1.0+3cd3285'"));

    let workflow = workflow("tag-attestation.yml");
    let evidence_steps = workflow["jobs"]["release-evidence"]["steps"]
        .as_sequence()
        .expect("release evidence job has steps");
    assert!(evidence_steps
        .iter()
        .any(|step| step["with"]["path"].as_str() == Some("spec/evidence/")));
}
