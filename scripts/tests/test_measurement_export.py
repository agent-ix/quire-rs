import json
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent.parent))

import pytest

from export_measurements import ExportError, build_collection, load_verification_stack


def attestation(revision: str = "a" * 40) -> dict:
    return {
        "schemaVersion": "verification-stack-attestation-v1",
        "lockDigest": "sha256:" + "1" * 64,
        "executableDigest": "sha256:" + "2" * 64,
        "sources": {
            "quire": {
                "revision": revision,
                "sourceState": "clean",
                "remote": "https://github.com/agent-ix/quire-rs",
            }
        },
        "capabilities": ["fixture.capability"],
        "artifacts": {"fixture": "sha256:" + "3" * 64},
    }


def test_collection_is_plan_derived_and_keeps_corpus_boundaries():
    source_revision = "a" * 40
    manifest = json.loads(
        (
            pathlib.Path(__file__).resolve().parents[2] / "bench" / "manifest.json"
        ).read_text()
    )
    manifest = {
        "metrics": {"coverage.dead_tags": manifest["metrics"]["coverage.dead_tags"]}
    }
    observed = {
        "first": {"coverage.dead_tags": 2},
        "second": {"coverage.dead_tags": 0},
    }
    raw = {
        "first": {"identity": "first-sha", "payload": {"untracked_symbols": [1, 2]}},
        "second": {"identity": "second-sha", "payload": {"untracked_symbols": []}},
    }
    collection = build_collection(
        manifest,
        observed,
        raw,
        timestamp="2026-08-27T00:00:00.000Z",
        source_revision=source_revision,
        tool_version="quire test",
        consumer=pathlib.Path("/consumer"),
        module=None,
        verification_stack=attestation(source_revision),
    )

    assert collection["collectionId"].startswith("quire-bench-20260827000000000-")
    assert [row["dimensions"]["corpus"] for row in collection["observations"]] == [
        "first",
        "second",
    ]
    assert all(row["planId"] == "MP-203" for row in collection["observations"])
    assert all(
        row["definitionVersion"] == "coverage.dead-tags-v1"
        for row in collection["observations"]
    )
    assert collection["rawEvidence"] == raw
    assert collection["schemaVersion"] == 2
    assert collection["verificationStack"] == attestation(source_revision)


def test_attestation_must_match_clean_exact_exporter_source(tmp_path: pathlib.Path):
    path = tmp_path / "attestation.json"
    path.write_text(json.dumps(attestation()))
    assert (
        load_verification_stack(
            path,
            source_name="quire",
            source_revision="a" * 40,
            source_remote="git@github.com:agent-ix/quire-rs.git",
        )
        == attestation()
    )

    changed = attestation("b" * 40)
    path.write_text(json.dumps(changed))
    with pytest.raises(ExportError, match="revision does not match"):
        load_verification_stack(
            path,
            source_name="quire",
            source_revision="a" * 40,
            source_remote="https://github.com/agent-ix/quire-rs",
        )


@pytest.mark.parametrize(
    ("mutate", "reason"),
    [
        (lambda value: value.update(schemaVersion="moving-v2"), "schemaVersion"),
        (lambda value: value.update(lockDigest="sha256:short"), "lockDigest"),
        (
            lambda value: value.update(executableDigest="sha256:short"),
            "executableDigest",
        ),
        (
            lambda value: value["sources"]["quire"].update(sourceState="dirty"),
            "not clean and immutable",
        ),
        (lambda value: value.update(capabilities=["z", "a"]), "capabilities"),
        (
            lambda value: value.update(capabilities=["fixture", "fixture"]),
            "capabilities",
        ),
        (lambda value: value.update(artifacts={"fixture": "moving"}), "artifact"),
    ],
)
def test_attestation_drift_fails_closed(tmp_path: pathlib.Path, mutate, reason: str):
    value = attestation()
    mutate(value)
    path = tmp_path / "attestation.json"
    path.write_text(json.dumps(value))
    with pytest.raises(ExportError, match=reason):
        load_verification_stack(
            path,
            source_name="quire",
            source_revision="a" * 40,
            source_remote="https://github.com/agent-ix/quire-rs",
        )


def test_attestation_remote_must_match_exporter_origin(tmp_path: pathlib.Path):
    path = tmp_path / "attestation.json"
    path.write_text(json.dumps(attestation()))
    with pytest.raises(ExportError, match="remote does not match"):
        load_verification_stack(
            path,
            source_name="quire",
            source_revision="a" * 40,
            source_remote="https://github.com/other/quire-rs",
        )
