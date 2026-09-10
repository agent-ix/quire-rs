#!/usr/bin/env bash
# check_dep_pins.sh — NFR-009-AC-1, NFR-009-AC-3, NFR-009-AC-5
#
# Enforce the concrete load-bearing policy from NFR-009. Non-load-bearing
# dependencies may retain caret ranges; wildcard ranges are never accepted.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
MANIFEST="$ROOT/Cargo.toml"
FUZZ_MANIFEST="$ROOT/fuzz/Cargo.toml"
LOCKFILE="$ROOT/Cargo.lock"

if [[ ! -f "$MANIFEST" ]]; then
  echo "check_dep_pins: Cargo.toml not present — skipping (OK)."
  exit 0
fi

FAILED=0

# ADR-0012 makes both the import alias and package identity load-bearing.
# Ask Cargo to parse the manifest rather than depending on field order or
# whitespace. Cargo.lock is machine-generated, so its package blocks have a
# stable representation that can be inspected without another TOML library.
METADATA_FILE="$(mktemp)"
FUZZ_METADATA_FILE="$(mktemp)"
trap 'rm -f -- "$METADATA_FILE" "$FUZZ_METADATA_FILE"' EXIT
if ! cargo metadata --locked --no-deps --format-version 1 \
  --manifest-path "$MANIFEST" >"$METADATA_FILE"; then
  echo "check_dep_pins: FAIL — Cargo could not read the locked manifest" >&2
  FAILED=1
elif [[ ! -f "$FUZZ_MANIFEST" ]]; then
  echo "check_dep_pins: FAIL — fuzz/Cargo.toml is required for YAML package verification" >&2
  FAILED=1
elif ! cargo metadata --offline --format-version 1 \
  --manifest-path "$FUZZ_MANIFEST" >"$FUZZ_METADATA_FILE"; then
  echo "check_dep_pins: FAIL — Cargo could not read the fuzz manifest" >&2
  FAILED=1
elif ! python3 - "$METADATA_FILE" "$FUZZ_METADATA_FILE" "$LOCKFILE" <<'PY'
import json
import pathlib
import re
import sys

errors = []
for label, metadata_path in (
    ("root", pathlib.Path(sys.argv[1])),
    ("fuzz", pathlib.Path(sys.argv[2])),
):
    metadata = json.loads(metadata_path.read_text())
    workspace_members = set(metadata.get("workspace_members", []))
    dependencies = [
        dependency
        for package in metadata.get("packages", [])
        if package.get("id") in workspace_members
        for dependency in package.get("dependencies", [])
        if dependency.get("rename") == "serde_yaml"
    ]
    if len(dependencies) != 1 or dependencies[0].get("name") != "yaml_serde":
        errors.append(f"{label} serde_yaml must alias the yaml_serde package")
    elif dependencies[0].get("req") != "=0.10.7":
        errors.append(f"{label} serde_yaml must use exact version =0.10.7")

    if label == "fuzz":
        packages = {
            (package.get("name"), package.get("version"))
            for package in metadata.get("packages", [])
        }
        for package in (("yaml_serde", "0.10.7"), ("libyaml-rs", "0.3.0")):
            if package not in packages:
                errors.append(f"fuzz resolved package {package[0]} {package[1]} is required")
        for name in ("serde_yaml", "unsafe-libyaml"):
            if any(package_name == name for package_name, _ in packages):
                errors.append(f"deprecated fuzz resolved package {name} is forbidden")

lock_path = pathlib.Path(sys.argv[3])
if not lock_path.is_file():
    errors.append("Cargo.lock is required for YAML package verification")
else:
    packages = set()
    for block in lock_path.read_text().split("[[package]]")[1:]:
        name = re.search(r'^name = "([^"]+)"$', block, re.MULTILINE)
        version = re.search(r'^version = "([^"]+)"$', block, re.MULTILINE)
        if name and version:
            packages.add((name.group(1), version.group(1)))
    for package in (("yaml_serde", "0.10.7"), ("libyaml-rs", "0.3.0")):
        if package not in packages:
            errors.append(f"locked package {package[0]} {package[1]} is required")
    for name in ("serde_yaml", "unsafe-libyaml"):
        if any(package_name == name for package_name, _ in packages):
            errors.append(f"deprecated locked package {name} is forbidden")

for error in errors:
    print(f"check_dep_pins: FAIL — {error} (NFR-009-AC-1/3/5)", file=sys.stderr)
raise SystemExit(bool(errors))
PY
then
  FAILED=1
fi

# Flag wildcard pins like `foo = "*"` or `foo = { version = "*" }` in both
# independently resolved manifests.
for candidate_manifest in "$MANIFEST" "$FUZZ_MANIFEST"; do
  if grep -REn '^[[:space:]]*[A-Za-z0-9_-]+[[:space:]]*=[[:space:]]*"\*"' "$candidate_manifest" >/dev/null 2>&1; then
    echo "check_dep_pins: FAIL — wildcard '*' version pin in ${candidate_manifest#"$ROOT/"} (NFR-009-AC-1):" >&2
    grep -REn '^[[:space:]]*[A-Za-z0-9_-]+[[:space:]]*=[[:space:]]*"\*"' "$candidate_manifest" >&2 || true
    FAILED=1
  fi

  if grep -REn 'version[[:space:]]*=[[:space:]]*"\*"' "$candidate_manifest" >/dev/null 2>&1; then
    echo "check_dep_pins: FAIL — wildcard version field in ${candidate_manifest#"$ROOT/"} (NFR-009-AC-1):" >&2
    grep -REn 'version[[:space:]]*=[[:space:]]*"\*"' "$candidate_manifest" >&2 || true
    FAILED=1
  fi
done

if [[ "$FAILED" -ne 0 ]]; then
  exit 1
fi

echo "check_dep_pins: OK"
