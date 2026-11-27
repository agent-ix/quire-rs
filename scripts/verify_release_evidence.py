#!/usr/bin/env python3
"""Validate and safely unpack an unsigned release evidence asset."""
# implements: FR-095-AC-2, FR-095-AC-6

import argparse
import gzip
import hashlib
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

MAX_ARCHIVE_BYTES = 20_000_000
MAX_DECOMPRESSED_BYTES = 50_000_000
MAX_ARCHIVE_MEMBERS = 6


class DecompressedArchiveLimitExceeded(Exception):
    """Raised when the complete gzip stream exceeds its byte limit."""


class BoundedGzipReader:
    """Count all decompressed bytes, including tar headers and padding."""

    def __init__(self, stream: gzip.GzipFile, limit: int) -> None:
        self.stream = stream
        self.limit = limit
        self.total = 0

    # implements: FR-095-AC-6
    def read(self, size: int = -1) -> bytes:
        remaining = self.limit - self.total
        requested = remaining + 1 if size < 0 else min(size, remaining + 1)
        content = self.stream.read(requested)
        self.total += len(content)
        if self.total > self.limit:
            raise DecompressedArchiveLimitExceeded
        return content

    def close(self) -> None:
        self.stream.close()


try:
    archive_size = args.archive.stat().st_size
except OSError as error:
    parser.error(f"release evidence archive is unavailable: {error}")
if archive_size > MAX_ARCHIVE_BYTES:
    parser.error("release evidence archive exceeds the compressed byte limit")

members = {}
decompressed_bytes = 0
try:
    with args.archive.open("rb") as compressed:
        with gzip.GzipFile(fileobj=compressed, mode="rb") as gzip_stream:
            bounded_stream = BoundedGzipReader(gzip_stream, MAX_DECOMPRESSED_BYTES)
            with tarfile.open(fileobj=bounded_stream, mode="r|") as archive:
                for member in archive:
                    if len(members) >= MAX_ARCHIVE_MEMBERS:
                        parser.error("release evidence archive exceeds the member count limit")
                    path = PurePosixPath(member.name)
                    if (
                        path.is_absolute()
                        or ".." in path.parts
                        or not member.isfile()
                        or member.name in members
                        or member.size < 0
                        or decompressed_bytes + member.size > MAX_DECOMPRESSED_BYTES
                    ):
                        parser.error("release evidence archive contains an unsafe, duplicate, or oversized entry")
                    extracted = archive.extractfile(member)
                    if extracted is None:
                        parser.error("release evidence archive contains an unreadable entry")
                    content = bytearray()
                    while True:
                        chunk = extracted.read(64 * 1024)
                        if not chunk:
                            break
                        decompressed_bytes += len(chunk)
                        if decompressed_bytes > MAX_DECOMPRESSED_BYTES:
                            parser.error("release evidence archive exceeds the decompressed byte limit")
                        content.extend(chunk)
                    if len(content) != member.size:
                        parser.error("release evidence archive entry size does not match its header")
                    members[member.name] = bytes(content)
except DecompressedArchiveLimitExceeded:
    parser.error("release evidence archive exceeds the decompressed byte limit")
except (OSError, EOFError, tarfile.TarError) as error:
    parser.error(f"release evidence archive is malformed: {error}")

if "release-evidence.json" not in members:
    parser.error("release evidence archive has no manifest")
manifest = json.loads(members["release-evidence.json"])
run_path = manifest.get("runPath", "")
if not isinstance(run_path, str) or not re.fullmatch(r"runs/SUITE-001/[A-Za-z0-9~_.-]+\.json", run_path):
    parser.error("release evidence manifest has an unsafe run path")
expected_paths = {
    "release-evidence.json", "suites.md", "bindings.committed.json",
    "bindings.json", "junit.xml", run_path,
}
if set(members) != expected_paths:
    parser.error("release evidence archive is missing or contains unexpected files")
expected_hashes = {
    name: hashlib.sha256(content).hexdigest()
    for name, content in members.items() if name != "release-evidence.json"
}
if manifest != {
    "schemaVersion": 1,
    "tag": args.expected_tag,
    "commit": args.expected_commit,
    "tagCreatedAt": args.tag_created_at,
    "suite": "SUITE-001",
    "runPath": run_path,
    "tool": "ix-trace-run 0.1.0+8a0ec153",
    "sha256": expected_hashes,
}:
    parser.error("release evidence manifest or payload hashes do not match the triggering tag SHA")

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
    or run.get("tool") != manifest["tool"]
    or run.get("evidenceKind") != "Unit"
    or not run.get("timestamp")
    or not run.get("entries")
    or any(entry.get("outcome") != "pass" for entry in run["entries"])
):
    parser.error("release run record is stale, empty, or contains a non-passing result")
try:
    run_time = datetime.fromisoformat(re.sub(r"(\.\d{6})\d+", r"\1", run["timestamp"]).replace("Z", "+00:00"))
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
