#!/usr/bin/env python3
"""Helpers shared by the sweep scripts: build the consumer's `quire` binary and
read the `engine` block of a payload it emits."""

from __future__ import annotations

import json
import pathlib
import subprocess
import sys


class Drift(Exception):
    """A disagreement worth failing the build over."""


def reported_engine(payload: dict) -> tuple[str, list[str]]:
    """`engine.engine` and `engine.capabilities` from an emitted payload."""
    block = payload.get("engine")
    if not isinstance(block, dict):
        raise Drift(
            "the payload carries no `engine` block, so the binary that produced "
            "it predates quire-cli#68 and cannot say which engine it links. "
            "That is precisely the state this check exists to refuse."
        )
    version = block.get("engine")
    if not isinstance(version, str) or not version:
        raise Drift(f"the payload's `engine.engine` is not a version: {block!r}")
    capabilities = block.get("capabilities")
    if not isinstance(capabilities, list):
        raise Drift(f"the payload's `engine.capabilities` is not a list: {block!r}")
    return version, [c for c in capabilities if isinstance(c, str)]


def assert_capabilities(reported: list[str], required: list[str]) -> None:
    """Every required token is present, and the missing ones are NAMED.

    Aborts rather than omitting a metric and continuing (#265 AC-4). A sweep
    that silently drops `binding_census` still prints a coverage percentage, and
    that percentage is exactly the confident-looking number four battle-testing
    passes chased.
    """
    missing = [token for token in required if token not in reported]
    if missing:
        raise Drift(
            f"the binary lacks required capability token(s): {', '.join(missing)}. "
            f"It reports {reported or '[]'}. Aborting rather than omitting the "
            f"metric and continuing — a measurement missing its premise still "
            f"prints a number."
        )


def build_engine(consumer: pathlib.Path, release: bool = False) -> str:
    """Build `quire` from the consuming workspace and return the binary path.

    Never a `PATH` lookup: an installed `quire` CLI lags the branch under test.
    """
    manifest = consumer / "Cargo.toml"
    if not manifest.is_file():
        raise Drift(f"no consumer workspace at {consumer}")
    print(f"building the engine from {consumer} …", file=sys.stderr)
    command = [
        "cargo",
        "build",
        "--locked",
        "--manifest-path",
        str(manifest),
        "--message-format",
        "json",
    ]
    if release:
        command.insert(2, "--release")
    try:
        # A ceiling: with a CARGO_TARGET_DIR shared across repositories this call
        # can sit blocked on the build-directory lock behind an unrelated build.
        done = subprocess.run(
            command, capture_output=True, text=True, timeout=1800, check=False
        )
    except subprocess.TimeoutExpired as error:
        raise Drift(
            f"building {consumer} exceeded 30 minutes. A shared CARGO_TARGET_DIR "
            f"blocks on the build-directory lock behind any other cargo process; "
            f"check for one before retrying."
        ) from error
    if done.returncode != 0:
        # Compiler diagnostics arrive on STDOUT under --message-format json, so
        # reporting only stderr showed a build failure with no reason in it.
        detail = (done.stderr.strip() or done.stdout.strip())[-600:]
        raise Drift(f"building {consumer} failed:\n{detail}")
    # The path comes from cargo's own message stream rather than a guessed
    # `target/debug/quire`: a workspace, a CARGO_TARGET_DIR, or a shared target
    # directory all move it.
    for line in done.stdout.splitlines():
        try:
            message = json.loads(line)
        except json.JSONDecodeError:
            continue
        if (
            message.get("reason") == "compiler-artifact"
            and message.get("executable")
            and message.get("target", {}).get("name") == "quire"
        ):
            return message["executable"]
    raise Drift(f"cargo built {consumer} but emitted no `quire` executable")
