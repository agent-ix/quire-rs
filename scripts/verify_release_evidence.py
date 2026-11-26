#!/usr/bin/env python3
"""Validate and safely unpack a Terra-produced release evidence asset."""

import argparse
import json
import re
import tarfile
from datetime import datetime, timezone
from pathlib import Path, PurePosixPath

parser = argparse.ArgumentParser()
parser.add_argument("--archive", type=Path, required=True)
parser.add_argument("--source-root", type=Path, required=True)
parser.add_argument("--expected-tag", required=True)
parser.add_argument("--expected-commit", required=True)
parser.add_argument("--tag-created-at", type=int, required=True)
parser.add_argument("--output-dir", type=Path, required=True)
args = parser.parse_args()

if not re.fullmatch(r"v[0-9]+\.[0-9]+\.[0-9]+", args.expected_tag):
    parser.error("expected tag must be vMAJOR.MINOR.PATCH")
if not re.fullmatch(r"[0-9a-f]{40}", args.expected_commit):
    parser.error("expected commit must be a full lowercase git SHA")

run_path = f"runs/SUITE-001/{args.expected_commit[:12]}.json"
expected_paths = {
    "release-evidence.json",
    "suites.md",
    "bindings.committed.json",
    "bindings.json",
    run_path,
}
members = {}
with tarfile.open(args.archive, "r:gz") as archive:
    for member in archive.getmembers():
        path = PurePosixPath(member.name)
        if (
            path.is_absolute()
            or ".." in path.parts
            or not member.isfile()
            or member.name in members
            or member.size > 50_000_000
        ):
            parser.error("release evidence archive contains an unsafe or duplicate entry")
        extracted = archive.extractfile(member)
        if extracted is None:
            parser.error("release evidence archive contains an unreadable entry")
        members[member.name] = extracted.read()

if set(members) != expected_paths:
    parser.error("release evidence archive is missing or contains unexpected files")

manifest = json.loads(members["release-evidence.json"])
if manifest != {
    "schemaVersion": 1,
    "tag": args.expected_tag,
    "commit": args.expected_commit,
    "tagCreatedAt": args.tag_created_at,
    "suite": "SUITE-001",
    "runPath": run_path,
    "tool": "ix-trace-run 0.1.0+3cd3285",
}:
    parser.error("release evidence manifest does not match the triggering tag SHA")

source_evidence = args.source_root / "spec" / "evidence"
if members["suites.md"] != (source_evidence / "suites.md").read_bytes():
    parser.error("release evidence suite registry differs from the tagged source")
if members["bindings.committed.json"] != (
    source_evidence / "bindings.json"
).read_bytes():
    parser.error("committed binding registry differs from the tagged source")

base_store = json.loads(members["bindings.committed.json"])
run_store = json.loads(members["bindings.json"])
for name, store in (("committed", base_store), ("generated", run_store)):
    if store.get("schemaVersion") != 1 or not isinstance(store.get("bindings"), list):
        parser.error(f"{name} binding registry is not a schema-version-1 Quoin store")

run = json.loads(members[run_path])
if (
    run.get("schemaVersion") != 1
    or run.get("suite") != "SUITE-001"
    or run.get("commit") != args.expected_commit
    or run.get("tool") != manifest["tool"]
    or run.get("evidenceKind") != "Unit"
    or not run.get("timestamp")
    or not run.get("entries")
    or any(entry.get("outcome") != "pass" for entry in run["entries"])
):
    parser.error("release run record is stale, empty, or contains a non-passing result")
try:
    run_time = datetime.fromisoformat(run["timestamp"].replace("Z", "+00:00"))
except (TypeError, ValueError):
    parser.error("release run record timestamp is not RFC3339")
tag_created_at = datetime.fromtimestamp(args.tag_created_at, timezone.utc)
if run_time.tzinfo is None or run_time <= tag_created_at:
    parser.error("release run record predates the exact annotated release tag")

output = args.output_dir.resolve()
if output.exists():
    parser.error("release evidence output directory already exists")
output.mkdir(parents=True)
for relative, content in members.items():
    destination = output / relative
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_bytes(content)

print(f"verified release evidence for {args.expected_tag} at {args.expected_commit}")
