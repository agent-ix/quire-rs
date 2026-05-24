#!/usr/bin/env bash
# check_dep_pins.sh — NFR-009-AC-1, NFR-009-AC-3
#
# Enforce: load-bearing dependencies in [dependencies] and [dev-dependencies]
# use tilde ("~") or exact ("=") version pins (NOT bare wildcards or `*`).
# Caret ("^") is permitted today per the handoff defaults but should be
# tightened as Task 026 lands.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
MANIFEST="$ROOT/Cargo.toml"

if [[ ! -f "$MANIFEST" ]]; then
  echo "check_dep_pins: Cargo.toml not present — skipping (OK)."
  exit 0
fi

FAILED=0

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

if [[ "$FAILED" -ne 0 ]]; then
  exit 1
fi

echo "check_dep_pins: OK"
