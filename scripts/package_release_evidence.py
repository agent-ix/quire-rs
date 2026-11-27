#!/usr/bin/env python3
"""Package the single SUITE-001 run for its signed release tag."""
# implements: FR-095-AC-4

import argparse
import hashlib
import json
import re
import tarfile
from datetime import datetime, timezone
from pathlib import Path

from release_evidence_validation import missing_required_release_bindings

parser = argparse.ArgumentParser()
parser.add_argument("--source-root", type=Path, required=True)
parser.add_argument("--committed-bindings", type=Path, required=True)
parser.add_argument("--run-record", type=Path, required=True)
parser.add_argument("--junit-results", type=Path, required=True)
parser.add_argument("--expected-tag", required=True)
parser.add_argument("--expected-commit", required=True)
parser.add_argument("--tag-created-at", type=int, required=True)
parser.add_argument("--archive", type=Path, required=True)
args = parser.parse_args()

if not re.fullmatch(r"v[0-9]+\.[0-9]+\.[0-9]+", args.expected_tag):
    parser.error("expected tag must be vMAJOR.MINOR.PATCH")
if not re.fullmatch(r"[0-9a-f]{40}", args.expected_commit):
    parser.error("expected commit must be a full lowercase git SHA")

source = args.source_root.resolve()
evidence = source / "spec" / "evidence"
committed_bindings = args.committed_bindings.read_bytes()
run_record = json.loads(args.run_record.read_text(encoding="utf-8"))
generated_bindings = (evidence / "bindings.json").read_bytes()
suite_registry = (evidence / "suites.md").read_bytes()

try:
    run_path = args.run_record.resolve().relative_to(evidence).as_posix()
except ValueError:
    parser.error("run record is outside the source evidence store")
if not re.fullmatch(r"runs/SUITE-001/[A-Za-z0-9~_.-]+\.json", run_path):
    parser.error("run record path does not identify the release suite")
if (
    run_record.get("schemaVersion") != 1
    or run_record.get("suite") != "SUITE-001"
    or run_record.get("tool") != "ix-trace-run 0.1.0+8a0ec153"
    or run_record.get("evidenceKind") != "Unit"
    or not run_record.get("timestamp")
):
    parser.error("run record does not match the declared release suite and tag")
try:
    run_time = datetime.fromisoformat(re.sub(r"(\.\d{6})\d+", r"\1", run_record["timestamp"]).replace("Z", "+00:00"))
except (TypeError, ValueError):
    parser.error("run record timestamp is not RFC3339")
tag_created_at = datetime.fromtimestamp(args.tag_created_at, timezone.utc)
if run_time.tzinfo is None or run_time <= tag_created_at:
    parser.error("run record is not newer than the exact annotated release tag")
if not run_record.get("entries") or any(
    entry.get("outcome") != "pass" for entry in run_record["entries"]
):
    parser.error("release suite run is empty or contains a non-passing result")

base_store = json.loads(committed_bindings)
run_store = json.loads(generated_bindings)
for name, store in (("committed", base_store), ("generated", run_store)):
    if store.get("schemaVersion") != 1 or not isinstance(store.get("bindings"), list):
        parser.error(f"{name} bindings registry is not a schema-version-1 Quoin store")

try:
    missing_obligations = missing_required_release_bindings(
        suite_registry.decode("utf-8"), run_store, run_record
    )
except ValueError as error:
    parser.error(str(error))
if missing_obligations:
    parser.error(
        "generated bindings lack required passing release evidence: "
        + ", ".join(missing_obligations)
    )

bundle = args.archive.parent / "release-evidence-bundle"
bundle.mkdir(parents=True, exist_ok=True)
files = {
    "suites.md": suite_registry,
    "bindings.committed.json": committed_bindings,
    "bindings.json": generated_bindings,
    run_path: args.run_record.read_bytes(),
    "junit.xml": args.junit_results.read_bytes(),
}
for relative, content in files.items():
    destination = bundle / relative
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_bytes(content)

manifest = {
    "schemaVersion": 1,
    "tag": args.expected_tag,
    "commit": args.expected_commit,
    "tagCreatedAt": args.tag_created_at,
    "suite": "SUITE-001",
    "runPath": run_path,
    "sha256": {name: hashlib.sha256(content).hexdigest() for name, content in files.items()},
    "tool": "ix-trace-run 0.1.0+8a0ec153",
}
(bundle / "release-evidence.json").write_text(
    json.dumps(manifest, sort_keys=True, indent=2) + "\n", encoding="utf-8"
)

with tarfile.open(args.archive, "w:gz") as archive:
    for relative in ("release-evidence.json", *files):
        archive.add(bundle / relative, arcname=relative, recursive=False)

print(f"release evidence bundle: {args.archive}")
