from __future__ import annotations

import importlib.util
import json
import pathlib
import subprocess

import pytest

ROOT = pathlib.Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts/validate_spec.py"
SPEC = importlib.util.spec_from_file_location("validate_spec", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
validate_spec = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(validate_spec)


def git(repo: pathlib.Path, *args: str) -> str:
    done = subprocess.run(
        ["git", "-C", str(repo), *args],
        capture_output=True,
        text=True,
        check=True,
    )
    return done.stdout.strip()


def repository(
    tmp_path: pathlib.Path, name: str, module_path: str, remote: str, remote_ref: str
) -> tuple[pathlib.Path, str]:
    root = tmp_path / name
    (root / module_path).mkdir(parents=True)
    (root / module_path / "manifest.yaml").write_text(f"name: {name}\n", encoding="utf-8")
    git(root.parent, "init", "-q", str(root))
    git(root, "config", "user.email", "test@example.invalid")
    git(root, "config", "user.name", "Test")
    git(root, "add", ".")
    git(root, "commit", "-qm", "fixture")
    revision = git(root, "rev-parse", "HEAD")
    git(root, "remote", "add", "origin", remote)
    git(root, "update-ref", remote_ref, revision)
    return root, revision


@pytest.fixture
def stack(tmp_path: pathlib.Path) -> tuple[pathlib.Path, dict[str, pathlib.Path]]:
    definitions = {
        "spec-artifacts-process": (
            "spec_artifacts_process",
            "https://github.com/agent-ix/spec-artifacts-process",
            "refs/remotes/origin/integration",
        ),
        "spec-artifacts-iso": (
            "spec_artifacts_iso",
            "https://github.com/agent-ix/spec-artifacts-iso",
            "refs/remotes/origin/main",
        ),
    }
    roots: dict[str, pathlib.Path] = {}
    entries: dict[str, dict[str, str]] = {}
    for name, (module_path, remote, remote_ref) in definitions.items():
        root, revision = repository(tmp_path, name, module_path, remote, remote_ref)
        roots[name] = root
        entries[name] = {
            "modulePath": module_path,
            "remote": remote,
            "remoteRef": remote_ref,
            "revision": revision,
        }
    lock = tmp_path / "validation-lock.json"
    lock.write_text(
        json.dumps(
            {"schemaVersion": "quire-validation-stack-v1", "repositories": entries}
        ),
        encoding="utf-8",
    )
    return lock, roots


def run_check(
    lock: pathlib.Path, roots: dict[str, pathlib.Path]
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [
            "python3",
            str(SCRIPT),
            "--lock",
            str(lock),
            "--process-root",
            str(roots["spec-artifacts-process"]),
            "--iso-root",
            str(roots["spec-artifacts-iso"]),
            "--check-only",
        ],
        capture_output=True,
        text=True,
        check=False,
    )


def test_exact_clean_remote_reachable_stack_passes(stack) -> None:
    lock, roots = stack
    result = run_check(lock, roots)
    assert result.returncode == 0, result.stderr
    assert "spec-artifacts-process:" in result.stdout
    assert "spec-artifacts-iso:" in result.stdout


def test_dirty_provider_fails_closed(stack) -> None:
    lock, roots = stack
    (roots["spec-artifacts-iso"] / "untracked.txt").write_text("drift")
    result = run_check(lock, roots)
    assert result.returncode != 0
    assert "repository is dirty" in result.stderr


def test_wrong_head_fails_closed(stack) -> None:
    lock, roots = stack
    root = roots["spec-artifacts-process"]
    (root / "new.txt").write_text("new")
    git(root, "add", ".")
    git(root, "commit", "-qm", "advance")
    result = run_check(lock, roots)
    assert result.returncode != 0
    assert "does not equal locked revision" in result.stderr


def test_wrong_origin_fails_closed(stack) -> None:
    lock, roots = stack
    git(
        roots["spec-artifacts-iso"],
        "remote",
        "set-url",
        "origin",
        "https://github.com/agent-ix/not-the-provider",
    )
    result = run_check(lock, roots)
    assert result.returncode != 0
    assert "does not equal locked remote" in result.stderr


def test_revision_unreachable_from_locked_remote_ref_fails_closed(stack) -> None:
    lock, roots = stack
    root = roots["spec-artifacts-process"]
    git(root, "update-ref", "-d", "refs/remotes/origin/integration")
    result = run_check(lock, roots)
    assert result.returncode != 0
    assert "locked provenance ref is unavailable" in result.stderr


def test_validation_environment_replaces_both_ambient_paths(
    tmp_path: pathlib.Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setenv("IX_FILAMENT_MODULES_PATH", "/ambient/preferred")
    monkeypatch.setenv("IX_SCHEMA_PATH", "/ambient/legacy")
    isolated = tmp_path / "isolated"
    env = validate_spec.validation_environment(isolated)
    assert env["IX_FILAMENT_MODULES_PATH"] == str(isolated)
    assert env["IX_SCHEMA_PATH"] == ""
