#!/usr/bin/env bash
# verify_cookiecutter_inheritance.sh — StR-004-AC-1, TC-203
#
# Verify the safety scaffolding inherited from `agent-ix/rust-lib-cookiecutter`
# is still present. We assert the *files* exist (drift detection on contents
# lives in a follow-up CR — today we only check presence).

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"

REQUIRED=(
  "clippy.toml"
  "deny.toml"
  "rustfmt.toml"
  "rust-toolchain.toml"
  "scripts/check_unsafe_comments.sh"
  "scripts/unsafe_comment_baseline.txt"
  ".github/workflows/ci.yml"
  "Makefile"
)

FAILED=0
for rel in "${REQUIRED[@]}"; do
  if [[ ! -e "$ROOT/$rel" ]]; then
    echo "verify_cookiecutter_inheritance: FAIL — missing $rel (StR-004-AC-1)." >&2
    FAILED=1
  fi
done

if [[ "$FAILED" -ne 0 ]]; then
  exit 1
fi

echo "verify_cookiecutter_inheritance: OK"
