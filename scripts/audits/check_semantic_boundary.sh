#!/usr/bin/env bash
# check_semantic_boundary.sh — NFR-021-AC-1/AC-2, FR-070-CON-1/CON-2,
# FR-071-CON-1, FR-072-CON-2 (TC-1619, TC-1620, TC-1627, TC-1640, TC-1641,
# TC-1642). Plan-003 Task-015 scaffold, tightened in Task-021.
#
# The semantic extraction path must not parse clause text, render, generate
# code, reach the network, shell out, or write files. Two checks:
#   1. Cargo.lock carries none of the denylisted crates.
#   2. src/semantic/ (when present) contains none of the denylisted symbols.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
LOCK="$ROOT/Cargo.lock"
SEMANTIC="$ROOT/src/semantic"

DENY_CRATES=(ocl sysml fret fretish tera handlebars minijinja askama liquid reqwest ureq hyper curl git2 gix rusqlite sled redb)
DENY_SYMBOLS='std::net|std::process|Command::new|reqwest|ureq|git2|fs::write|File::create|OpenOptions|write_all\(|eval\(|parse_expr|typecheck|Tera|Handlebars|minijinja|askama'

FAILED=0
if [[ -f "$LOCK" ]]; then
  for dep in "${DENY_CRATES[@]}"; do
    if grep -E "^name = \"${dep}\"\$" "$LOCK" >/dev/null 2>&1; then
      echo "check_semantic_boundary: FAIL — denylisted crate '${dep}' in Cargo.lock (NFR-021-AC-1)." >&2
      FAILED=1
    fi
  done
fi

if [[ -d "$SEMANTIC" ]]; then
  if grep -rnE "$DENY_SYMBOLS" "$SEMANTIC" --include='*.rs' | grep -v '^\s*//' ; then
    echo "check_semantic_boundary: FAIL — denylisted symbol in src/semantic/ (NFR-021-AC-1/AC-2)." >&2
    FAILED=1
  fi
else
  echo "check_semantic_boundary: src/semantic/ not present yet — crate check only."
fi

if [[ $FAILED -ne 0 ]]; then exit 1; fi
echo "check_semantic_boundary: OK"
