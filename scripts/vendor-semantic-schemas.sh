#!/usr/bin/env bash
# vendor-semantic-schemas.sh — FR-069 Inputs / FR-069-CON-2 (Plan-003 Task-015)
#
# Re-vendor the semantic module contract inputs that still have no dependency
# edge to resolve through, into schemas/vendored/, and rewrite
# schemas/vendored/PROVENANCE.json. The only sanctioned way to change anything
# under schemas/vendored/.
#
# `common.schema.json` and the semantic-core 0.2.0 bundle are NOT vendored
# here (PLAT-906): they resolve through the `filament-core-data` Cargo
# dependency at build time (`build.rs`, `src/semantic/vendored.rs`). The
# semantic-core 0.1.0 bundle is not vendored either (PLAT-906): fcd does not
# publish it in a resolvable form (no commit carries both a Cargo workspace
# and 0.1.0-era schema bytes), and quire-rs no longer carries a copy while
# that is true — see the PLAT-906 PR description. The only thing this script
# still vendors is `module-manifest.schema.json` (private-origin, PLAT-901,
# blocked on filament-core-service publishing a public schema home); once
# that lands, this script and schemas/vendored/ are both deletable outright.
#
# Upstream checkouts are read with `git show <rev>:<path>`; a missing object is
# fetched over HTTPS by revision. Nothing else touches the network.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/schemas/vendored"
DEV="${IX_DEV:-$HOME/dev}"

FCS_REV="5b2af8b18c9ed5ca919217dea17fdf440df95bae"   # agent-ix/filament-core-service

show() { # repo-dir github-slug rev path
  local dir="$1" slug="$2" rev="$3" path="$4"
  if ! git -C "$dir" cat-file -e "$rev^{commit}" 2>/dev/null; then
    git -C "$dir" fetch -q "https://github.com/$slug.git" "$rev"
  fi
  git -C "$dir" show "$rev:$path"
}

rm -rf "$OUT"
mkdir -p "$OUT"

show "$DEV/filament-core-service" agent-ix/filament-core-service "$FCS_REV" \
  filament_core_service/schemas/module-manifest.schema.json > "$OUT/module-manifest.schema.json"

sha() { sha256sum "$1" | cut -d' ' -f1; }

{
  echo '{'
  echo '  "$comment": "Written by scripts/vendor-semantic-schemas.sh; never edit by hand (FR-069-CON-2).",'
  echo '  "files": {'
  printf '    "module-manifest.schema.json": { "repository": "agent-ix/filament-core-service", "revision": "%s", "path": "filament_core_service/schemas/module-manifest.schema.json", "sha256": "sha256:%s" }' \
    "$FCS_REV" "$(sha "$OUT/module-manifest.schema.json")"
  echo
  echo '  }'
  echo '}'
} > "$OUT/PROVENANCE.json"

echo "vendored $(find "$OUT" -type f -name '*.json' ! -name PROVENANCE.json | wc -l) file(s) into $OUT"
