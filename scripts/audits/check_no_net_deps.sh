#!/usr/bin/env bash
# check_no_net_deps.sh — FR-013-AC-6, TC-085
#
# Enforce StR-001 / FR-013: the engine is filesystem-only and MUST NOT pull
# in HTTP/RPC client crates. Blocklist below is the set of common offenders;
# extend as needed.
#
# Exits 0 when no blocked dep is present in Cargo.lock (or when Cargo.lock
# doesn't exist yet — pre-skeleton state).

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
LOCK="$ROOT/Cargo.lock"

# Network-client crates that violate the filesystem-only constraint.
BLOCKED=(
  "reqwest"
  "hyper"
  "tonic"
  "ureq"
  "curl"
  "isahc"
  "surf"
  "awc"
  "h2"
  "h3"
  "quinn"
)

if [[ ! -f "$LOCK" ]]; then
  echo "check_no_net_deps: Cargo.lock not present yet — skipping (OK)."
  exit 0
fi

FAILED=0
for dep in "${BLOCKED[@]}"; do
  # Look for `name = "<dep>"` lines in Cargo.lock.
  if grep -E "^name = \"${dep}\"\$" "$LOCK" >/dev/null 2>&1; then
    echo "check_no_net_deps: FAIL — blocked network dep '${dep}' present in Cargo.lock (FR-013-AC-6)." >&2
    FAILED=1
  fi
done

if [[ "$FAILED" -ne 0 ]]; then
  exit 1
fi

echo "check_no_net_deps: OK"
