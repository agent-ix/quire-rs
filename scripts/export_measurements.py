#!/usr/bin/env python3
"""Run the Quire benchmark producer and export one governed collection."""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import re
import subprocess
import sys
from datetime import datetime, timezone
from typing import Any

from bench import MANIFEST, ROOT, BenchError, build_engine, collect

COUNT_METRICS = {
    "coverage.dead_tags",
    "coverage.minting_repos",
    "sentinel.silent_zero",
}
FULL_SHA = re.compile(r"^[0-9a-f]{40}$")
SHA256 = re.compile(r"^sha256:[0-9a-f]{64}$")


class ExportError(RuntimeError):
    """A collection that cannot be derived without guessing."""


def digest(parts: list[bytes]) -> str:
    value = hashlib.sha256()
    for part in parts:
        value.update(len(part).to_bytes(8, "big"))
        value.update(part)
    return f"sha256:{value.hexdigest()}"


def git_revision(root: pathlib.Path) -> str:
    done = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=root,
        capture_output=True,
        text=True,
        check=False,
    )
    revision = done.stdout.strip()
    if done.returncode != 0 or not FULL_SHA.fullmatch(revision):
        raise ExportError("the Quire source revision is unavailable")
    status = subprocess.run(
        ["git", "status", "--porcelain=v1", "--untracked-files=all"],
        cwd=root,
        capture_output=True,
        text=True,
        check=False,
    )
    if status.returncode != 0 or status.stdout.strip():
        raise ExportError("the Quire source tree is dirty")
    return revision


def normalized_remote(value: str) -> str:
    value = value.strip()
    ssh = re.fullmatch(r"git@github\.com:(.+)", value)
    if ssh:
        value = f"https://github.com/{ssh.group(1)}"
    return value.removesuffix(".git").rstrip("/")


def git_remote(root: pathlib.Path) -> str:
    done = subprocess.run(
        ["git", "remote", "get-url", "origin"],
        cwd=root,
        capture_output=True,
        text=True,
        check=False,
    )
    if done.returncode != 0 or not done.stdout.strip():
        raise ExportError("the Quire origin remote is unavailable")
    return normalized_remote(done.stdout)


def load_verification_stack(
    path: pathlib.Path,
    *,
    source_name: str,
    source_revision: str,
    source_remote: str,
) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text())
    except (OSError, json.JSONDecodeError) as error:
        raise ExportError(
            f"verification-stack attestation is unreadable: {error}"
        ) from error
    if not isinstance(value, dict):
        raise ExportError("verification-stack attestation must be an object")
    if value.get("schemaVersion") != "verification-stack-attestation-v1":
        raise ExportError(
            "verification-stack attestation has unsupported schemaVersion"
        )
    for key in ("lockDigest", "executableDigest"):
        if not SHA256.fullmatch(str(value.get(key, ""))):
            raise ExportError(f"verification-stack {key} is not a full sha256 digest")
    sources = value.get("sources")
    if not isinstance(sources, dict) or not sources:
        raise ExportError("verification-stack sources must be a non-empty object")
    for name, source in sources.items():
        if (
            not isinstance(source, dict)
            or not FULL_SHA.fullmatch(str(source.get("revision", "")))
            or source.get("sourceState") != "clean"
            or not isinstance(source.get("remote"), str)
            or not source["remote"]
        ):
            raise ExportError(
                f"verification-stack source {name} is not clean and immutable"
            )
    own_source = sources.get(source_name)
    if not isinstance(own_source, dict):
        raise ExportError(f"verification-stack has no {source_name} source")
    if own_source["revision"] != source_revision:
        raise ExportError(
            f"verification-stack {source_name} revision does not match exporter source"
        )
    if normalized_remote(own_source["remote"]) != normalized_remote(source_remote):
        raise ExportError(
            f"verification-stack {source_name} remote does not match exporter origin"
        )
    capabilities = value.get("capabilities")
    if (
        not isinstance(capabilities, list)
        or not capabilities
        or any(not isinstance(item, str) or not item for item in capabilities)
        or capabilities != sorted(set(capabilities))
    ):
        raise ExportError(
            "verification-stack capabilities must be non-empty, unique, and sorted"
        )
    artifacts = value.get("artifacts")
    if not isinstance(artifacts, dict) or not artifacts:
        raise ExportError("verification-stack artifacts must be a non-empty object")
    for name, artifact_digest in artifacts.items():
        if not SHA256.fullmatch(str(artifact_digest)):
            raise ExportError(
                f"verification-stack artifact {name} is not a full sha256 digest"
            )
    return value


def plan_identity(path: pathlib.Path) -> tuple[str, str]:
    text = path.read_text()
    frontmatter = re.match(r"^---\n(.*?)\n---", text, re.DOTALL)
    if not frontmatter:
        raise ExportError(f"{path}: measurement plan has no frontmatter")

    def field(name: str) -> str:
        match = re.search(
            rf"^{re.escape(name)}:\s*([^\n]+)$", frontmatter.group(1), re.MULTILINE
        )
        if not match:
            raise ExportError(f"{path}: measurement plan lacks {name}")
        return match.group(1).strip().strip('"')

    return field("id"), field("definition_version")


def build_collection(
    manifest: dict[str, Any],
    observed: dict[str, dict[str, float]],
    raw_evidence: dict[str, Any],
    *,
    timestamp: str,
    source_revision: str,
    tool_version: str,
    consumer: pathlib.Path,
    module: str | None,
    verification_stack: dict[str, Any],
) -> dict[str, Any]:
    observations: list[dict[str, Any]] = []
    plan_bytes: list[bytes] = []
    for corpus in sorted(observed):
        for metric in sorted(observed[corpus]):
            definition = manifest["metrics"].get(metric)
            if not definition:
                raise ExportError(f"{metric}: metric has no manifest definition")
            plan_path = ROOT / definition["measurement_plan"]
            plan_id, definition_version = plan_identity(plan_path)
            plan_bytes.append(plan_path.read_bytes())
            observations.append(
                {
                    "metric": metric,
                    "planId": plan_id,
                    "definitionVersion": definition_version,
                    "state": "measured",
                    "value": observed[corpus][metric],
                    "unit": definition["unit"],
                    "shape": "count" if metric in COUNT_METRICS else "ratio",
                    "population": {
                        "complete": True,
                        "identity": {
                            "corpus": corpus,
                            "source": raw_evidence[corpus]["identity"],
                        },
                    },
                    "dimensions": {"corpus": corpus},
                }
            )
    if not observations:
        raise ExportError("the benchmark produced no observations")
    represented = {row["metric"] for row in observations}
    missing = sorted(set(manifest["metrics"]) - represented)
    if missing:
        raise ExportError(
            "active benchmark metrics were not produced: " + ", ".join(missing)
        )
    evidence_bytes = json.dumps(raw_evidence, sort_keys=True).encode("utf-8")
    evidence_digest = digest([evidence_bytes])
    compact_time = re.sub(r"[^0-9]", "", timestamp)
    identities = [
        f"{name}:{raw_evidence[name]['identity']}".encode("utf-8")
        for name in sorted(raw_evidence)
    ]
    return {
        "schemaVersion": 2,
        "collectionId": f"quire-bench-{compact_time}-{evidence_digest[7:19]}",
        "subject": "Quire engine benchmark",
        "scope": {
            "corpora": sorted(observed),
            "metrics": sorted({row["metric"] for row in observations}),
        },
        "toolIdentity": "quire-rs scripts/bench.py",
        "toolVersion": tool_version,
        "configDigest": digest([MANIFEST.read_bytes(), *sorted(set(plan_bytes))]),
        "timestamp": timestamp,
        "sourceRevision": source_revision,
        "corpusRevision": digest(identities)[7:],
        "environment": {
            "consumer": str(consumer),
            "module": f"per-manifest with default {module or 'default'}",
        },
        "verificationStack": verification_stack,
        "observations": observations,
        "rawEvidence": raw_evidence,
    }


def cli_version(quire: str) -> str:
    done = subprocess.run(
        [quire, "--version"], capture_output=True, text=True, check=False
    )
    if done.returncode != 0 or not done.stdout.strip():
        raise ExportError("the built Quire CLI did not report a version")
    return done.stdout.strip()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--consumer", default="../quire-cli")
    parser.add_argument("--module")
    parser.add_argument("--output", type=pathlib.Path, required=True)
    parser.add_argument(
        "--verification-stack",
        type=pathlib.Path,
        required=True,
        help="canonical verification-stack-attestation-v1 JSON",
    )
    args = parser.parse_args()
    consumer = pathlib.Path(args.consumer).expanduser()
    if not consumer.is_absolute():
        consumer = (ROOT / consumer).resolve()
    manifest = json.loads(MANIFEST.read_text())
    raw_evidence: dict[str, Any] = {}
    try:
        source_revision = git_revision(ROOT)
        verification_stack = load_verification_stack(
            args.verification_stack,
            source_name="quire",
            source_revision=source_revision,
            source_remote=git_remote(ROOT),
        )
        quire = build_engine(consumer, release=True)
        observed = collect(manifest, quire, args.module, raw_evidence)
        timestamp = (
            datetime.now(timezone.utc)
            .isoformat(timespec="milliseconds")
            .replace("+00:00", "Z")
        )
        collection = build_collection(
            manifest,
            observed,
            raw_evidence,
            timestamp=timestamp,
            source_revision=source_revision,
            tool_version=cli_version(quire),
            consumer=consumer,
            module=args.module,
            verification_stack=verification_stack,
        )
    except (BenchError, ExportError, OSError, json.JSONDecodeError) as error:
        print(f"measurement export: {error}", file=sys.stderr)
        return 1
    args.output.write_text(json.dumps(collection, indent=2, sort_keys=True) + "\n")
    print(args.output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
