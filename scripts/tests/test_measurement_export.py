import json
import pathlib
import subprocess
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent.parent))

import pytest

from export_measurements import (
    ExportError,
    build_collection,
    load_verification_stack,
    validate_executable_digest,
    validate_manifest_attestation,
    validate_repository_against_stack,
)


def attestation(revision: str = "a" * 40) -> dict:
    return {
        "schemaVersion": "verification-stack-attestation-v1",
        "lockDigest": "sha256:" + "1" * 64,
        "executableDigest": "sha256:" + "2" * 64,
        "buildProfile": "release",
        "sources": {
            "quire": {
                "revision": revision,
                "sourceState": "clean",
                "remote": "https://github.com/agent-ix/quire-rs",
            },
            "filament-ide-rs": {
                "revision": "b" * 40,
                "sourceState": "clean",
                "remote": "https://github.com/agent-ix/filament-ide-rs",
            },
            "quoin": {
                "revision": "c" * 40,
                "sourceState": "clean",
                "remote": "https://github.com/agent-ix/quoin",
            },
            "quoin-benchmark-corpus": {
                "revision": "f" * 40,
                "sourceState": "clean",
                "remote": "https://github.com/agent-ix/quoin",
            },
            "spec-artifacts-process": {
                "revision": "d" * 40,
                "sourceState": "clean",
                "remote": "https://github.com/agent-ix/spec-artifacts-process",
            },
            "quire-cli": {
                "revision": "e" * 40,
                "sourceState": "clean",
                "remote": "https://github.com/agent-ix/quire-cli",
            },
        },
        "capabilities": ["fixture.capability"],
        "artifacts": {"fixture": "sha256:" + "3" * 64},
        "toolchains": {"node": "22.15.0", "rust": "1.94.1", "python": "3.10.12"},
    }


def git(root: pathlib.Path, *args: str) -> str:
    return subprocess.run(
        ["git", *args], cwd=root, check=True, capture_output=True, text=True
    ).stdout.strip()


def exporter_repository(tmp_path: pathlib.Path) -> tuple[pathlib.Path, str]:
    root = tmp_path / "quire-rs"
    root.mkdir()
    git(root, "init")
    git(root, "config", "user.email", "test@example.invalid")
    git(root, "config", "user.name", "Test")
    git(root, "remote", "add", "origin", "https://github.com/agent-ix/quire-rs")
    (root / "source.py").write_text("locked\n")
    git(root, "add", "source.py")
    git(root, "commit", "-m", "locked source")
    return root, git(root, "rev-parse", "HEAD")


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
        (lambda value: value.update(buildProfile="debug"), "buildProfile"),
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
        (lambda value: value.update(toolchains={}), "toolchains"),
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


def test_exporter_accepts_only_linear_evidence_overlay(tmp_path: pathlib.Path):
    root, locked = exporter_repository(tmp_path)
    value = attestation(locked)
    evidence = root / "spec" / "evidence" / "measurements" / "run.json"
    evidence.parent.mkdir(parents=True)
    evidence.write_text("{}\n")
    git(root, "add", "spec/evidence/measurements/run.json")
    git(root, "commit", "-m", "record evidence")

    assert (
        validate_repository_against_stack(
            root,
            value,
            "quire",
            allowed_overlay_paths=("spec/evidence/measurements",),
        )
        == locked
    )


def test_exporter_rejects_code_or_dirty_evidence_overlay(tmp_path: pathlib.Path):
    root, locked = exporter_repository(tmp_path)
    value = attestation(locked)
    (root / "source.py").write_text("drifted\n")
    git(root, "add", "source.py")
    git(root, "commit", "-m", "code drift")
    with pytest.raises(ExportError, match="non-evidence paths"):
        validate_repository_against_stack(
            root,
            value,
            "quire",
            allowed_overlay_paths=("spec/evidence/measurements",),
        )

    (root / "dirty.txt").write_text("uncommitted\n")
    with pytest.raises(ExportError, match="checkout is dirty"):
        validate_repository_against_stack(
            root,
            value,
            "quire",
            allowed_overlay_paths=("spec/evidence/measurements",),
        )


def test_manifest_pins_must_match_attested_clean_sources():
    manifest = {
        "corpora": [
            {
                "name": "filament-ide-rs",
                "identity": "sha",
                "pinned_sha": "b" * 40,
            },
            {"name": "self", "identity": "working-tree"},
        ]
    }
    validate_manifest_attestation(manifest, attestation())

    manifest["corpora"][0]["pinned_sha"] = "c" * 40
    with pytest.raises(ExportError, match="benchmark pin does not match"):
        validate_manifest_attestation(manifest, attestation())


def test_manifest_rejects_abbreviated_pins_even_when_the_prefix_matches():
    manifest = {
        "corpora": [
            {
                "name": "filament-ide-rs",
                "identity": "sha",
                "pinned_sha": "b" * 7,
            }
        ]
    }
    with pytest.raises(ExportError, match="not a full Git SHA"):
        validate_manifest_attestation(manifest, attestation())


def test_manifest_rejects_external_working_tree_inputs():
    manifest = {
        "corpora": [{"name": "quoin", "path": "../quoin", "identity": "working-tree"}]
    }
    with pytest.raises(ExportError, match="external benchmark input must use sha"):
        validate_manifest_attestation(manifest, attestation())


def test_manifest_module_pin_must_match_attestation():
    manifest = {
        "corpora": [],
        "module_source": {
            "name": "spec-artifacts-process",
            "path": "../spec-artifacts-process",
            "identity": "sha",
            "pinned_sha": "d" * 40,
        },
    }
    validate_manifest_attestation(manifest, attestation())
    manifest["module_source"]["pinned_sha"] = "f" * 40
    with pytest.raises(ExportError, match="benchmark pin does not match"):
        validate_manifest_attestation(manifest, attestation())


def test_manifest_source_name_separates_benchmark_subject_from_producer():
    manifest = {
        "corpora": [
            {
                "name": "quoin",
                "source_name": "quoin-benchmark-corpus",
                "identity": "sha",
                "pinned_sha": "f" * 40,
            }
        ]
    }
    validate_manifest_attestation(manifest, attestation())
    manifest["corpora"][0]["source_name"] = "quoin"
    with pytest.raises(ExportError, match="benchmark pin does not match"):
        validate_manifest_attestation(manifest, attestation())


def test_built_executable_must_match_attested_digest(tmp_path: pathlib.Path):
    executable = tmp_path / "quire"
    executable.write_bytes(b"exact binary")
    value = attestation()
    import hashlib

    value["executableDigest"] = "sha256:" + hashlib.sha256(b"exact binary").hexdigest()
    validate_executable_digest(str(executable), value)
    executable.write_bytes(b"drifted binary")
    with pytest.raises(ExportError, match="digest does not match"):
        validate_executable_digest(str(executable), value)
