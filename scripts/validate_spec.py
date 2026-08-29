#!/usr/bin/env python3
"""Validate Quire's spec against an exact, reviewable module stack."""

from __future__ import annotations

import argparse
import contextlib
import json
import os
import pathlib
import re
import subprocess
import sys
import tempfile
from collections.abc import Iterator, Sequence
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parents[1]
DEFAULT_LOCK = ROOT / "quality/validation-stack-lock.json"
FULL_SHA = re.compile(r"^[0-9a-f]{40}$")
REMOTE_REF = re.compile(r"^refs/remotes/origin/[A-Za-z0-9._/-]+$")


class ValidationStackError(RuntimeError):
    """The schema-provider stack is not exactly the checked-in stack."""


def git(repo: pathlib.Path, *args: str) -> str:
    done = subprocess.run(
        ["git", "-C", str(repo), *args],
        capture_output=True,
        text=True,
        check=False,
    )
    if done.returncode != 0:
        detail = done.stderr.strip() or done.stdout.strip() or "git command failed"
        raise ValidationStackError(f"{repo}: {detail}")
    return done.stdout.strip()


def normalized_remote(value: str) -> str:
    value = value.strip()
    ssh = re.fullmatch(r"git@github\.com:(.+)", value)
    if ssh:
        value = f"https://github.com/{ssh.group(1)}"
    return value.removesuffix(".git").rstrip("/")


def load_lock(path: pathlib.Path) -> dict[str, dict[str, str]]:
    try:
        value: Any = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise ValidationStackError(f"validation stack lock is unreadable: {error}") from error
    if not isinstance(value, dict) or set(value) != {"schemaVersion", "repositories"}:
        raise ValidationStackError("validation stack lock has unexpected top-level fields")
    if value["schemaVersion"] != "quire-validation-stack-v1":
        raise ValidationStackError("validation stack lock has an unsupported schemaVersion")
    repositories = value["repositories"]
    expected_names = {"spec-artifacts-process", "spec-artifacts-iso"}
    if not isinstance(repositories, dict) or set(repositories) != expected_names:
        raise ValidationStackError("validation stack lock must name exactly both schema providers")
    for name, entry in repositories.items():
        if not isinstance(entry, dict) or set(entry) != {
            "modulePath",
            "remote",
            "remoteRef",
            "revision",
        }:
            raise ValidationStackError(f"{name}: validation lock entry has unexpected fields")
        if not FULL_SHA.fullmatch(str(entry["revision"])):
            raise ValidationStackError(f"{name}: revision must be a full lowercase Git SHA")
        if not str(entry["remote"]).startswith("https://github.com/agent-ix/"):
            raise ValidationStackError(f"{name}: remote must be a canonical agent-ix HTTPS URL")
        if normalized_remote(str(entry["remote"])) != str(entry["remote"]):
            raise ValidationStackError(f"{name}: remote must already be normalized")
        if not REMOTE_REF.fullmatch(str(entry["remoteRef"])):
            raise ValidationStackError(f"{name}: remoteRef must be an origin remote-tracking ref")
        module_path = pathlib.PurePosixPath(str(entry["modulePath"]))
        if module_path.is_absolute() or ".." in module_path.parts or not module_path.parts:
            raise ValidationStackError(f"{name}: modulePath must stay within its repository")
    return repositories


def verify_repository(
    name: str, root: pathlib.Path, locked: dict[str, str]
) -> pathlib.Path:
    root = root.resolve()
    if not root.is_dir():
        raise ValidationStackError(f"{name}: repository root does not exist: {root}")
    top = pathlib.Path(git(root, "rev-parse", "--show-toplevel")).resolve()
    if top != root:
        raise ValidationStackError(f"{name}: root must be the Git worktree root, got {top}")
    head = git(root, "rev-parse", "HEAD")
    if head != locked["revision"]:
        raise ValidationStackError(
            f"{name}: HEAD {head} does not equal locked revision {locked['revision']}"
        )
    dirty = git(root, "status", "--porcelain=v1", "--untracked-files=all")
    if dirty:
        raise ValidationStackError(f"{name}: repository is dirty")
    remote = normalized_remote(git(root, "remote", "get-url", "origin"))
    if remote != locked["remote"]:
        raise ValidationStackError(
            f"{name}: origin {remote!r} does not equal locked remote {locked['remote']!r}"
        )
    remote_ref = locked["remoteRef"]
    try:
        git(root, "show-ref", "--verify", remote_ref)
    except ValidationStackError as error:
        raise ValidationStackError(
            f"{name}: locked provenance ref is unavailable: {remote_ref}"
        ) from error
    ancestor = subprocess.run(
        ["git", "-C", str(root), "merge-base", "--is-ancestor", head, remote_ref],
        capture_output=True,
        text=True,
        check=False,
    )
    if ancestor.returncode != 0:
        raise ValidationStackError(
            f"{name}: locked revision is not reachable from {remote_ref}"
        )
    module = (root / locked["modulePath"]).resolve()
    try:
        module.relative_to(root)
    except ValueError as error:
        raise ValidationStackError(f"{name}: modulePath escapes its repository") from error
    if not (module / "manifest.yaml").is_file():
        raise ValidationStackError(f"{name}: locked module manifest is missing: {module}")
    return module


@contextlib.contextmanager
def isolated_module_root(modules: dict[str, pathlib.Path]) -> Iterator[pathlib.Path]:
    with tempfile.TemporaryDirectory(prefix="quire-validation-modules-") as temporary:
        root = pathlib.Path(temporary)
        for name, module in sorted(modules.items()):
            (root / name).symlink_to(module, target_is_directory=True)
        yield root


def validation_environment(module_root: pathlib.Path) -> dict[str, str]:
    env = os.environ.copy()
    env["IX_FILAMENT_MODULES_PATH"] = str(module_root)
    # Set, rather than inherit or merely delete, the legacy alias. The preferred
    # variable above wins, and an empty alias cannot become a fallback if loader
    # precedence changes accidentally.
    env["IX_SCHEMA_PATH"] = ""
    return env


def parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--lock", type=pathlib.Path, default=DEFAULT_LOCK)
    parser.add_argument("--process-root", type=pathlib.Path, required=True)
    parser.add_argument("--iso-root", type=pathlib.Path, required=True)
    parser.add_argument(
        "--check-only",
        action="store_true",
        help="verify identities and manifests without running Cargo",
    )
    return parser.parse_args(argv)


def main(argv: Sequence[str] | None = None) -> int:
    args = parse_args(argv)
    try:
        locked = load_lock(args.lock.resolve())
        roots = {
            "spec-artifacts-process": args.process_root,
            "spec-artifacts-iso": args.iso_root,
        }
        modules = {
            name: verify_repository(name, roots[name], locked[name])
            for name in sorted(locked)
        }
        if args.check_only:
            for name in sorted(modules):
                print(f"{name}: {locked[name]['revision']} ({modules[name]})")
            return 0
        with isolated_module_root(modules) as module_root:
            done = subprocess.run(
                ["cargo", "run", "--locked", "--quiet", "--example", "spec_validate"],
                cwd=ROOT,
                env=validation_environment(module_root),
                check=False,
            )
        return done.returncode
    except ValidationStackError as error:
        print(f"validate_spec: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
