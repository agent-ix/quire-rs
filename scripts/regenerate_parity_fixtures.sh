#!/usr/bin/env bash
# Regenerate render-parity expecteds via the pinned Python + Jinja2 venv.
#
# Walks tests/render_parity/corpus.yaml; for every case whose input
# lives under cases/<archetype>/<name>.input.json, runs the
# corresponding module's templates/<archetype>.md.j2 through Jinja2
# using the input as context, and writes cases/<archetype>/<name>.expected.md.
#
# The venv pin lives at scripts/parity-venv.txt. CI hosts that don't
# already have the venv create it on first run; subsequent runs reuse.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VENV="${QUIRE_PARITY_VENV:-$ROOT/.parity-venv}"

if [[ ! -d "$VENV" ]]; then
  echo "==> creating venv at $VENV (matches scripts/parity-venv.txt)"
  python3 -m venv "$VENV"
fi

# Use the venv's pip; pin via parity-venv.txt.
"$VENV/bin/pip" install --quiet -r "$ROOT/scripts/parity-venv.txt"

cd "$ROOT"
"$VENV/bin/python3" "$ROOT/scripts/_render_parity.py"
