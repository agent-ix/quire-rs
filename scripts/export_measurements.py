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
    if done.returncode != 0 or not done.stdout.strip():
        raise ExportError("the Quire source revision is unavailable")
    return done.stdout.strip()


def plan_identity(path: pathlib.Path) -> tuple[str, str]:
    text = path.read_text()
    frontmatter = re.match(r"^---\n(.*?)\n---", text, re.DOTALL)
    if not frontmatter:
        raise ExportError(f"{path}: measurement plan has no frontmatter")

    def field(name: str) -> str:
        match = re.search(rf"^{re.escape(name)}:\s*([^\n]+)$", frontmatter.group(1), re.MULTILINE)
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
        "schemaVersion": 1,
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
    args = parser.parse_args()
    consumer = pathlib.Path(args.consumer).expanduser()
    if not consumer.is_absolute():
        consumer = (ROOT / consumer).resolve()
    manifest = json.loads(MANIFEST.read_text())
    raw_evidence: dict[str, Any] = {}
    try:
        quire = build_engine(consumer, release=True)
        observed = collect(manifest, quire, args.module, raw_evidence)
        timestamp = datetime.now(timezone.utc).isoformat(
            timespec="milliseconds"
        ).replace("+00:00", "Z")
        collection = build_collection(
            manifest,
            observed,
            raw_evidence,
            timestamp=timestamp,
            source_revision=git_revision(ROOT),
            tool_version=cli_version(quire),
            consumer=consumer,
            module=args.module,
        )
    except (BenchError, ExportError, OSError, json.JSONDecodeError) as error:
        print(f"measurement export: {error}", file=sys.stderr)
        return 1
    args.output.write_text(json.dumps(collection, indent=2, sort_keys=True) + "\n")
    print(args.output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
