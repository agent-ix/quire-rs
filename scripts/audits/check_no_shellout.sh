#!/usr/bin/env bash
# check_no_shellout.sh — StR-001-AC-2, TC-201
#
# Enforce: src/ MUST NOT shell out to python/node/npm/pip via
# std::process::Command (or duct/tokio::process). The engine is a pure Rust
# library; child-process orchestration belongs to consumers.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
SRC="$ROOT/src"

if [[ ! -d "$SRC" ]]; then
  echo "check_no_shellout: src/ not present yet — skipping (OK)."
  exit 0
fi

# Look for `Command::new("<runtime>")` patterns. We allow `Command` usages in
# tests/ and benches/ but not in src/.
PATTERN='Command::new\((b)?"(python|python3|node|npm|pnpm|yarn|pip|pip3|bash|sh|zsh)"\)'

if grep -REn --include='*.rs' "$PATTERN" "$SRC" >/dev/null 2>&1; then
  echo "check_no_shellout: FAIL — shellout to runtime detected in src/ (StR-001-AC-2):" >&2
  grep -REn --include='*.rs' "$PATTERN" "$SRC" >&2 || true
  exit 1
fi

echo "check_no_shellout: OK"
