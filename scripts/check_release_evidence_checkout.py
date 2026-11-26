#!/usr/bin/env python3
"""Refuse to run release evidence against a dirty checkout or submodule."""
# implements: FR-095-AC-1

import argparse
import subprocess
import sys
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument("--repo-root", type=Path, required=True)
args = parser.parse_args()

try:
    result = subprocess.run(
        [
            "git",
            "-C",
            str(args.repo_root),
            "status",
            "--porcelain=v1",
            "--untracked-files=all",
            "--ignore-submodules=none",
        ],
        check=True,
        capture_output=True,
        text=True,
    )
except (OSError, subprocess.CalledProcessError) as error:
    print(f"release_evidence_checkout_status_failed: {error}", file=sys.stderr)
    raise SystemExit(1)

try:
    submodule_state = subprocess.run(
        ["git", "-C", str(args.repo_root), "submodule", "status", "--recursive"],
        check=True,
        capture_output=True,
        text=True,
    )
    nested_state = subprocess.run(
        [
            "git",
            "-C",
            str(args.repo_root),
            "submodule",
            "foreach",
            "--quiet",
            "--recursive",
            "git status --porcelain=v1 --untracked-files=all --ignore-submodules=none",
        ],
        check=True,
        capture_output=True,
        text=True,
    )
except (OSError, subprocess.CalledProcessError) as error:
    print(f"release_evidence_submodule_status_failed: {error}", file=sys.stderr)
    raise SystemExit(1)

uninitialized_or_moved = any(
    line and line[0] != " " for line in submodule_state.stdout.splitlines()
)
if result.stdout or uninitialized_or_moved or nested_state.stdout:
    print(
        "release_evidence_dirty_checkout: tracked, untracked, or submodule changes are present",
        file=sys.stderr,
    )
    raise SystemExit(1)
