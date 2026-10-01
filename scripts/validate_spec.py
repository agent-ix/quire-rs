#!/usr/bin/env python3
"""Validate Quire's spec against the schema-provider modules."""

from __future__ import annotations

import argparse
import contextlib
import os
import pathlib
import subprocess
import sys
import tempfile
from collections.abc import Iterator, Sequence

ROOT = pathlib.Path(__file__).resolve().parents[1]
MODULE_PATHS = {
    "spec-artifacts-process": "spec_artifacts_process",
    "spec-artifacts-iso": "spec_artifacts_iso",
}


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
    parser.add_argument("--process-root", type=pathlib.Path, required=True)
    parser.add_argument("--iso-root", type=pathlib.Path, required=True)
    return parser.parse_args(argv)


def main(argv: Sequence[str] | None = None) -> int:
    args = parse_args(argv)
    roots = {
        "spec-artifacts-process": args.process_root,
        "spec-artifacts-iso": args.iso_root,
    }
    modules = {
        name: (roots[name] / module_path).resolve()
        for name, module_path in MODULE_PATHS.items()
    }
    for name, module in modules.items():
        if not module.is_dir():
            print(f"validate_spec: {name}: module directory missing: {module}", file=sys.stderr)
            return 1
    with isolated_module_root(modules) as module_root:
        done = subprocess.run(
            ["cargo", "run", "--locked", "--quiet", "--example", "spec_validate"],
            cwd=ROOT,
            env=validation_environment(module_root),
            check=False,
        )
    return done.returncode


if __name__ == "__main__":
    raise SystemExit(main())
