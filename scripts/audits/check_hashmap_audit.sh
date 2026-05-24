#!/usr/bin/env bash
# check_hashmap_audit.sh — NFR-006-AC-3, TC-058
#
# Enforce: render/parse code paths MUST NOT use `std::collections::HashMap`
# (iteration order is observable and breaks determinism). Use `IndexMap` or
# `BTreeMap` instead.
#
# Scoped to:
#   src/parser/
#   src/render/
#   src/merge.rs
#   src/extract/
#
# Other modules may use HashMap freely.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
SRC="$ROOT/src"

if [[ ! -d "$SRC" ]]; then
  echo "check_hashmap_audit: src/ not present yet — skipping (OK)."
  exit 0
fi

SCOPES=(
  "$SRC/parser"
  "$SRC/render"
  "$SRC/merge.rs"
  "$SRC/extract"
)

# Build a list of paths that actually exist (skip silently otherwise).
EXISTING=()
for path in "${SCOPES[@]}"; do
  if [[ -e "$path" ]]; then
    EXISTING+=("$path")
  fi
done

if [[ "${#EXISTING[@]}" -eq 0 ]]; then
  echo "check_hashmap_audit: no in-scope modules present yet — skipping (OK)."
  exit 0
fi

# Match `std::collections::HashMap` and bare `use ... HashMap` imports.
PATTERN='(std::collections::HashMap|^use[[:space:]]+std::collections::HashMap|::HashMap[^A-Za-z0-9_])'

if grep -REn --include='*.rs' -E "$PATTERN" "${EXISTING[@]}" >/dev/null 2>&1; then
  echo "check_hashmap_audit: FAIL — std HashMap used in deterministic code path (NFR-006-AC-3):" >&2
  grep -REn --include='*.rs' -E "$PATTERN" "${EXISTING[@]}" >&2 || true
  exit 1
fi

echo "check_hashmap_audit: OK"
