#!/usr/bin/env bash
# FR-052-CON-1 — the property classifier SHALL NOT construct a `GrammarFinding`.
#
# The constraint is structural rather than conventional: `src/grammar/property.rs`
# does not import the type, so its return types cannot express a finding, there
# is no FR-048 `grammar_severity` key for it and `--strict` immunity holds by
# construction. This script is what keeps that true — adding a finding to the
# classifier becomes a one-line change this gate rejects, instead of a silent
# drift a reviewer has to notice.
#
# Runs in CI and locally via `make ci`.
set -euo pipefail

target="src/grammar/property.rs"

if [[ ! -f "$target" ]]; then
  echo "property purity: ${target} is missing" >&2
  exit 1
fi

# Doc comments are allowed to *name* the type — the module documents why it
# must not construct one, and that prose is the reviewer's signpost. Only code
# is audited, so whole-line `//`, `///` and `//!` comments are stripped first;
# anything else mentioning the type (a `use`, a struct literal, a return type,
# even behind a trailing comment) still fails.
if grep -vE '^[[:space:]]*//' "$target" | grep -n 'GrammarFinding'; then
  echo "" >&2
  echo "property purity: ${target} references GrammarFinding (FR-052-CON-1)." >&2
  echo "Classification is metadata, never a finding: it carries no severity," >&2
  echo "so it has no promotion path and no --strict exposure. Emit the signal" >&2
  echo "on the classification record instead, or take the constraint to the" >&2
  echo "spec first." >&2
  exit 1
fi

exit 0
