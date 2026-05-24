#!/usr/bin/env bash
# check_no_schemars.sh — FR-003-AC-4, TC-062
#
# Enforce: `schemars` MUST NOT appear in the dependency graph. Schema
# generation is out-of-scope for v1; consumers supply pre-built schemas.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
LOCK="$ROOT/Cargo.lock"
MANIFEST="$ROOT/Cargo.toml"

FAILED=0

if [[ -f "$LOCK" ]]; then
  if grep -E "^name = \"schemars\"\$" "$LOCK" >/dev/null 2>&1; then
    echo "check_no_schemars: FAIL — 'schemars' present in Cargo.lock (FR-003-AC-4)." >&2
    FAILED=1
  fi
else
  echo "check_no_schemars: Cargo.lock not present yet — checking Cargo.toml only."
fi

if [[ -f "$MANIFEST" ]]; then
  if grep -E "^schemars[[:space:]]*=" "$MANIFEST" >/dev/null 2>&1; then
    echo "check_no_schemars: FAIL — 'schemars' declared in Cargo.toml (FR-003-AC-4)." >&2
    FAILED=1
  fi
fi

if [[ "$FAILED" -ne 0 ]]; then
  exit 1
fi

echo "check_no_schemars: OK"
