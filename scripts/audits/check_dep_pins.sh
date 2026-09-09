#!/usr/bin/env bash
# check_dep_pins.sh — NFR-009-AC-1, NFR-009-AC-3, NFR-009-AC-5
#
# Enforce the concrete load-bearing policy from NFR-009. Non-load-bearing
# dependencies may retain caret ranges; wildcard and unbounded lower-bound
# ranges are never accepted.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
MANIFEST="$ROOT/Cargo.toml"

if [[ ! -f "$MANIFEST" ]]; then
  echo "check_dep_pins: Cargo.toml not present — skipping (OK)."
  exit 0
fi

FAILED=0

require_manifest_line() {
  local pattern="$1"
  local reason="$2"
  if ! grep -Eq "$pattern" "$MANIFEST"; then
    echo "check_dep_pins: FAIL — $reason (NFR-009-AC-1/3)" >&2
    FAILED=1
  fi
}

# ADR-0012 makes both the import alias and package identity load-bearing.
# Match the complete declaration so a renamed/reordered substitute fails
# closed and receives review instead of escaping a partial-field check.
require_manifest_line \
  '^[[:space:]]*serde_yaml[[:space:]]*=[[:space:]]*\{[[:space:]]*package[[:space:]]*=[[:space:]]*"yaml_serde"[[:space:]]*,[[:space:]]*version[[:space:]]*=[[:space:]]*"=0\.10\.7"[[:space:]]*\}[[:space:]]*$' \
  'serde_yaml must alias the yaml_serde package at exact version =0.10.7'

# ADR-0001 selected the 0.18 minor line. A caret would allow the next minor
# and weaken the behavior-bearing validation boundary.
require_manifest_line \
  '^[[:space:]]*jsonschema[[:space:]]*=[[:space:]]*\{[^}]*version[[:space:]]*=[[:space:]]*"~0\.18"[^}]*\}[[:space:]]*$' \
  'jsonschema ~0.18 is required by the load-bearing pin policy'

# Flag wildcard pins like `foo = "*"` or `foo = { version = "*" }`.
if grep -REn '^[[:space:]]*[A-Za-z0-9_-]+[[:space:]]*=[[:space:]]*"\*"' "$MANIFEST" >/dev/null 2>&1; then
  echo "check_dep_pins: FAIL — wildcard '*' version pin in Cargo.toml (NFR-009-AC-1):" >&2
  grep -REn '^[[:space:]]*[A-Za-z0-9_-]+[[:space:]]*=[[:space:]]*"\*"' "$MANIFEST" >&2 || true
  FAILED=1
fi

if grep -REn 'version[[:space:]]*=[[:space:]]*"\*"' "$MANIFEST" >/dev/null 2>&1; then
  echo "check_dep_pins: FAIL — wildcard version field in Cargo.toml (NFR-009-AC-1):" >&2
  grep -REn 'version[[:space:]]*=[[:space:]]*"\*"' "$MANIFEST" >&2 || true
  FAILED=1
fi

if grep -REn '(^|version[[:space:]]*=[[:space:]]*")>=[[:space:]]*[0-9]' "$MANIFEST" >/dev/null 2>&1; then
  echo "check_dep_pins: FAIL — unbounded lower-bound version range in Cargo.toml (NFR-009-AC-3):" >&2
  grep -REn '(^|version[[:space:]]*=[[:space:]]*")>=[[:space:]]*[0-9]' "$MANIFEST" >&2 || true
  FAILED=1
fi

if [[ "$FAILED" -ne 0 ]]; then
  exit 1
fi

echo "check_dep_pins: OK"
