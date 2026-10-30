use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Command;

use ix_trace_rs::trace;
use quire_bench::collection::{collect, Collection, RawEvidence};
use quire_bench::export::{
    build_collection, load_verification_stack, validate_repository_against_stack, VerificationStack,
};
use quire_bench::{Manifest, Observed};
use serde_json::{json, Number};

fn attestation(revision: &str) -> VerificationStack {
    serde_json::from_value(json!({
        "schemaVersion": "verification-stack-attestation-v1",
        "buildProfile": "release",
        "sources": {
            "quire": {
                "revision": revision,
                "sourceState": "clean",
                "remote": "https://github.com/agent-ix/quire-rs"
            }
        },
        "capabilities": ["binding_census", "metrics_envelope"],
        "toolchains": {"node": "fixture", "rust": "fixture", "python": "fixture"}
    }))
    .unwrap()
}

fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

#[trace("FR-050-AC-30")]
#[test]
fn collection_is_plan_derived_and_complete() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let manifest: Manifest =
        serde_json::from_str(include_str!("../../../bench/manifest.json")).unwrap();
    let observations = manifest
        .metrics
        .keys()
        .map(|name| (name.clone(), Number::from(1)))
        .collect();
    let measurements = Collection {
        observed: Observed::from([("fixture".to_owned(), observations)]),
        raw_evidence: BTreeMap::from([(
            "fixture".to_owned(),
            RawEvidence {
                module: None,
                payload: json!({"totals": {"backed": 1}}),
            },
        )]),
    };
    let collection = build_collection(
        &root,
        &manifest,
        measurements,
        "2026-08-27T00:00:00.000Z",
        "a".repeat(40),
        Path::new("/consumer"),
        None,
        attestation(&"a".repeat(40)),
    )
    .unwrap();
    let value = serde_json::to_value(collection).unwrap();
    assert_eq!(value["collectionId"], "quire-bench-20260827000000000");
    assert_eq!(value["schemaVersion"], 2);
    assert_eq!(
        value["rawEvidence"]["fixture"]["payload"]["totals"]["backed"],
        1
    );
    assert!(value["observations"].as_array().unwrap().iter().any(|row| {
        row["metric"] == "coverage.dead_tags"
            && row["planId"] == "MP-203"
            && row["definitionVersion"] == "coverage.dead-tags-v1"
            && row["shape"] == "count"
            && row["population"]["complete"] == true
    }));
}

#[trace("FR-050-AC-30")]
#[test]
fn attestation_refuses_drift_and_accepts_equivalent_remote() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("stack.json");
    let revision = "a".repeat(40);
    let mut value = serde_json::to_value(attestation(&revision)).unwrap();
    fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    load_verification_stack(&path, "quire", "git@github.com:agent-ix/quire-rs.git").unwrap();

    for (field, invalid) in [
        ("buildProfile", json!("debug")),
        ("capabilities", json!(["z", "a"])),
    ] {
        let old = value[field].clone();
        value[field] = invalid;
        fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
        assert!(
            load_verification_stack(&path, "quire", "https://github.com/agent-ix/quire-rs")
                .is_err(),
            "{field}"
        );
        value[field] = old;
    }
}

#[trace("FR-050-AC-30")]
#[test]
fn repository_overlay_accepts_only_linear_evidence() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    git(root, &["init", "-q"]);
    git(root, &["config", "user.email", "test@example.invalid"]);
    git(root, &["config", "user.name", "Test"]);
    git(
        root,
        &[
            "remote",
            "add",
            "origin",
            "https://github.com/agent-ix/quire-rs",
        ],
    );
    fs::write(root.join("source.rs"), "locked\n").unwrap();
    git(root, &["add", "source.rs"]);
    git(root, &["commit", "-qm", "locked source"]);
    let locked = git(root, &["rev-parse", "HEAD"]);
    let stack = attestation(&locked);
    assert_eq!(
        validate_repository_against_stack(root, &stack, "quire", &["spec/evidence/measurements"])
            .unwrap(),
        locked
    );

    let evidence = root.join("spec/evidence/measurements/run.json");
    fs::create_dir_all(evidence.parent().unwrap()).unwrap();
    fs::write(&evidence, "{}\n").unwrap();
    git(root, &["add", "spec/evidence/measurements/run.json"]);
    git(root, &["commit", "-qm", "record evidence"]);
    validate_repository_against_stack(root, &stack, "quire", &["spec/evidence/measurements"])
        .unwrap();
    fs::write(root.join("source.rs"), "drifted\n").unwrap();
    git(root, &["add", "source.rs"]);
    git(root, &["commit", "-qm", "code drift"]);
    assert!(validate_repository_against_stack(
        root,
        &stack,
        "quire",
        &["spec/evidence/measurements"]
    )
    .unwrap_err()
    .to_string()
    .contains("non-evidence paths"));
    fs::write(root.join("uncommitted.txt"), "dirty\n").unwrap();
    assert!(validate_repository_against_stack(
        root,
        &stack,
        "quire",
        &["spec/evidence/measurements"]
    )
    .unwrap_err()
    .to_string()
    .contains("dirty"));
}

#[trace("FR-050-AC-30")]
#[test]
fn governed_collection_refuses_unreadable_corpus() {
    let temp = tempfile::tempdir().unwrap();
    let mut manifest: Manifest =
        serde_json::from_str(include_str!("../../../bench/manifest.json")).unwrap();
    manifest.corpora.truncate(1);
    manifest.corpora[0].path = "missing".to_owned();
    manifest.corpora[0].module = Some("missing-module".to_owned());
    manifest.corpora[0].require_clean = true;
    let error = collect(temp.path(), &manifest, Path::new("/no-engine"), None, true)
        .err()
        .unwrap()
        .to_string();
    assert!(error.contains("specific-shape-control"));
    assert!(error.contains("absent"));
}

#[trace("FR-050-AC-52")]
#[test]
fn corpus_checkout_must_equal_its_attested_revision() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    git(root, &["init", "-q"]);
    git(root, &["config", "user.email", "test@example.invalid"]);
    git(root, &["config", "user.name", "Test"]);
    git(
        root,
        &[
            "remote",
            "add",
            "origin",
            "https://github.com/agent-ix/quire-rs",
        ],
    );
    fs::write(root.join("spec.md"), "attested\n").unwrap();
    git(root, &["add", "spec.md"]);
    git(root, &["commit", "-qm", "attested corpus"]);
    let attested = git(root, &["rev-parse", "HEAD"]);
    let stack = attestation(&attested);
    let refusal = |name: &str| {
        validate_repository_against_stack(root, &stack, name, &[])
            .unwrap_err()
            .to_string()
    };
    assert_eq!(
        validate_repository_against_stack(root, &stack, "quire", &[]).unwrap(),
        attested
    );
    assert_eq!(refusal("corpus"), "verification-stack has no corpus source");
    fs::write(root.join("spec.md"), "edited\n").unwrap();
    assert_eq!(refusal("quire"), "quire checkout is dirty");
    git(root, &["commit", "-qam", "moved corpus"]);
    assert_eq!(
        refusal("quire"),
        "quire checkout does not equal the attested revision"
    );
    git(
        root,
        &[
            "remote",
            "set-url",
            "origin",
            "https://github.com/agent-ix/other",
        ],
    );
    assert_eq!(
        refusal("quire"),
        "quire checkout remote does not match the attestation"
    );
}
