#!/usr/bin/env python3
"""Fail when the engine a consumer links is not the engine in this tree.

`agent-ix/quire-rs#265` (EPIC #264, Wave 0). Direct analogue of
`quoin/scripts/check-version-agreement.mjs`, applied to the seam that one never
covered: the engine *inside* the CLI.

## What went wrong without it

`quire-cli/Cargo.toml` pins this crate independently of this crate's own
identity, and nothing compared the two. Measured at the time of writing: the
installed CLI **0.29.0** pinned engine **v0.42.0**, while `binding_census` — the
only signal saying whether the trace binder read a single test — landed in
**v0.43.0**. Four battle-testing passes reported ecosystem figures from a binary
that could not emit it, and no surface said so.

## The three facts, and why all three

A pin can agree with a tree and still not be what ran, so this compares:

1. **Declared** — the `rev`/`tag` in the consumer's `Cargo.toml`.
2. **Resolved** — what that pin names *in this repository*, and whether it is
   reachable from `HEAD`. A pin to a commit this tree has never contained is
   drift even when the strings look plausible.
3. **Reported** — `engine.engine` from a real payload emitted by a binary built
   from that pin (quire-cli#68). This is the only one of the three that is
   evidence rather than intent; the other two are what somebody wrote down.

Comparing 1 against 2 catches a stale pin. Comparing 2 against 3 catches a
binary built from something other than what the manifest claims — the failure a
manifest check alone cannot see, and the one that actually produced the wrong
numbers.
"""

from __future__ import annotations

import argparse
import json
import pathlib
import re
import subprocess
import sys

# `quire-rs`'s own manifest version is NOT one of the three facts, deliberately.
# It reads `0.33.0` at every tag from v0.42.0 through v0.45.0 — this crate
# derives its released version from the git tag and does not keep the manifest
# in step. Comparing against it would fail on every correct configuration, which
# is how a gate gets disabled. It is reported as an advisory instead; see
# `manifest_staleness`.
# git's own floor for an unambiguous abbreviation. Below it, a "prefix match"
# stops being evidence: a single hex digit prefixes one sha in sixteen.
MIN_SHA_ABBREV = 7

PIN = re.compile(
    r"""quire-rs\s*=\s*\{[^}]*?\b(?P<kind>rev|tag)\s*=\s*"(?P<value>[^"]+)\"""",
    re.VERBOSE,
)


class Drift(Exception):
    """A disagreement worth failing the build over."""


def read_pin(manifest: pathlib.Path) -> tuple[str, str]:
    """The `rev` or `tag` the consumer's manifest pins this crate to."""
    if not manifest.is_file():
        raise Drift(f"no consumer manifest at {manifest}")
    match = PIN.search(manifest.read_text(encoding="utf-8"))
    if not match:
        raise Drift(
            f"{manifest} declares no `rev`/`tag` for quire-rs. An unpinned git "
            f"dependency resolves to whatever the default branch happens to be "
            f"at build time, which is the opposite of provenance."
        )
    return match.group("kind"), match.group("value")


def git(repo: pathlib.Path, *args: str) -> str:
    done = subprocess.run(
        ["git", "-C", str(repo), *args],
        capture_output=True,
        text=True,
        check=False,
    )
    if done.returncode != 0:
        raise Drift(f"git {' '.join(args)}: {done.stderr.strip() or 'failed'}")
    return done.stdout.strip()


def resolve(repo: pathlib.Path, pin: str) -> str:
    """The full sha `pin` names in this repository."""
    try:
        return git(repo, "rev-parse", f"{pin}^{{commit}}")
    except Drift as error:
        raise Drift(
            f"the consumer pins `{pin}`, which does not exist in this "
            f"repository ({error}). The pin names a commit or tag this tree has "
            f"never contained."
        ) from error


def assert_reachable(repo: pathlib.Path, sha: str, pin: str) -> None:
    """The pinned commit must be an ancestor of `HEAD` (or HEAD itself).

    A pin to a commit on an abandoned branch resolves fine and is still drift:
    the consumer is linking work this tree does not carry.
    """
    done = subprocess.run(
        ["git", "-C", str(repo), "merge-base", "--is-ancestor", sha, "HEAD"],
        capture_output=True,
        text=True,
        check=False,
    )
    if done.returncode != 0:
        raise Drift(
            f"the consumer pins `{pin}` ({sha[:8]}), which is not reachable from "
            f"this tree's HEAD ({git(repo, 'rev-parse', '--short', 'HEAD')}). "
            f"Either the consumer is behind, or it is linking a branch this "
            f"repository has not merged."
        )


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


def assert_agrees(declared: str, reported: str, sha: str) -> None:
    """The binary reports the pin it was built from.

    Accepts either form the pin can take: a tag pin reports the version string
    (`v0.45.0` → `0.45.0`), a rev pin reports the sha, possibly abbreviated
    differently from the manifest's abbreviation.

    The prefix relation is allowed **only against the resolved sha, and only at
    git's minimum abbreviation length**. Without the length floor a binary
    reporting `8` satisfies a pin resolving to `84740d4…`; without restricting
    the relation to the sha, `0.45.0-3-gabc1234` satisfies a pin on tag
    `v0.45.0`, which is three commits of drift reading as agreement.
    """
    normalized = declared[1:] if declared.startswith("v") else declared
    if reported == normalized:
        return
    if len(reported) >= MIN_SHA_ABBREV and sha.startswith(reported):
        return
    raise Drift(
        f"the consumer's manifest pins `{declared}` but the binary built from it "
        f"reports engine `{reported}`. A manifest and a binary disagreeing is "
        f"the defect this check exists for: the manifest is what somebody wrote, "
        f"the payload is what actually ran."
    )


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


def manifest_staleness(repo: pathlib.Path) -> str | None:
    """Advisory: this crate's manifest version against its latest tag.

    NOT a failure. The manifest has read `0.33.0` since well before the tags it
    ships under, so promoting this to an error would make `ci` red on every
    correct configuration and the gate would be switched off within a day. It is
    printed because an unmentioned known-wrong number is how the next person
    reads `0.33.0` off `Cargo.toml` and believes it.
    """
    manifest = repo / "Cargo.toml"
    if not manifest.is_file():
        return None
    match = re.search(r'^version\s*=\s*"([^"]+)"', manifest.read_text(), re.MULTILINE)
    if not match:
        return None
    declared = match.group(1)
    try:
        latest = git(repo, "describe", "--tags", "--abbrev=0")
    except Drift:
        return None
    if latest.lstrip("v") == declared:
        return None
    return (
        f"advisory: Cargo.toml says version = \"{declared}\" while the latest tag "
        f"is {latest}. This crate derives its released version from the tag, so "
        f"the manifest value is not the version anybody runs — never read it as "
        f"one (agent-ix/quire-rs#282)."
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--repo",
        default=str(pathlib.Path(__file__).resolve().parent.parent),
        help="this quire-rs checkout (default: the tree containing this script)",
    )
    parser.add_argument(
        "--consumer",
        default="../quire-cli",
        help="the consuming crate whose manifest pins this one (default: ../quire-cli)",
    )
    parser.add_argument(
        "--payload",
        help="a JSON payload emitted by a binary built from the pin. Omit to "
        "check the manifest against the tree only, which is the weaker check.",
    )
    parser.add_argument(
        "--require",
        action="append",
        default=[],
        metavar="TOKEN",
        help="capability token the binary must report; repeatable",
    )
    args = parser.parse_args()

    repo = pathlib.Path(args.repo).expanduser().resolve()
    consumer = pathlib.Path(args.consumer).expanduser()
    if not consumer.is_absolute():
        consumer = (repo.parent / consumer.name).resolve()

    try:
        kind, pin = read_pin(consumer / "Cargo.toml")
        sha = resolve(repo, pin)
        assert_reachable(repo, sha, pin)
        print(f"check_engine: {consumer.name} pins {kind}={pin} -> {sha[:8]}, reachable from HEAD")

        if args.payload:
            payload = json.loads(pathlib.Path(args.payload).read_text(encoding="utf-8"))
            reported, capabilities = reported_engine(payload)
            assert_agrees(pin, reported, sha)
            assert_capabilities(capabilities, args.require)
            print(
                f"check_engine: the built binary reports engine {reported} "
                f"with {len(capabilities)} capability token(s)"
            )
        else:
            print(
                "check_engine: no --payload given, so the binary was not asked "
                "what it links. This is the weaker check: a manifest can agree "
                "with a tree and still not be what ran."
            )
    except Drift as error:
        print(f"check_engine: FAIL\n  {error}", file=sys.stderr)
        return 1

    advisory = manifest_staleness(repo)
    if advisory:
        print(f"check_engine: {advisory}")
    print("check_engine: OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
