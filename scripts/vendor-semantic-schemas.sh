#!/usr/bin/env bash
# vendor-semantic-schemas.sh — FR-069 Inputs / FR-069-CON-2 (Plan-003 Task-015)
#
# Re-vendor the semantic module contract inputs into schemas/vendored/ from
# pinned upstream revisions and rewrite schemas/vendored/PROVENANCE.json.
# The only sanctioned way to change anything under schemas/vendored/.
#
# Upstream checkouts are read with `git show <rev>:<path>`; a missing object is
# fetched over HTTPS by revision. Nothing else touches the network.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/schemas/vendored"
DEV="${IX_DEV:-$HOME/dev}"

FCS_REV="a77f31efc757f3578ad80d8c7e619897aa3b2513"   # agent-ix/filament-core-service
FCD_REV="d48b8da7ae5e40b8b3d465d45b2bd3e24b994dbb"   # agent-ix/filament-core-data
CORE_VERSION="0.1.0"

show() { # repo-dir github-slug rev path
  local dir="$1" slug="$2" rev="$3" path="$4"
  if ! git -C "$dir" cat-file -e "$rev^{commit}" 2>/dev/null; then
    git -C "$dir" fetch -q "https://github.com/$slug.git" "$rev"
  fi
  git -C "$dir" show "$rev:$path"
}

rm -rf "$OUT"
mkdir -p "$OUT/semantic-core/$CORE_VERSION"

show "$DEV/filament-core-service" agent-ix/filament-core-service "$FCS_REV" \
  filament_core_service/schemas/module-manifest.schema.json > "$OUT/module-manifest.schema.json"
show "$DEV/filament-core-data" agent-ix/filament-core-data "$FCD_REV" \
  schema/semantic/v1/common.schema.json > "$OUT/common.schema.json"

CORE_SRC="packages/semantic-core/generated/json-schema"
for name in $(git -C "$DEV/filament-core-data" ls-tree --name-only "$FCD_REV" "$CORE_SRC/" | xargs -n1 basename | sort); do
  show "$DEV/filament-core-data" agent-ix/filament-core-data "$FCD_REV" "$CORE_SRC/$name" \
    > "$OUT/semantic-core/$CORE_VERSION/$name"
done
show "$DEV/filament-core-data" agent-ix/filament-core-data "$FCD_REV" \
  packages/semantic-core/generated/toolchain.json > "$OUT/semantic-core/$CORE_VERSION/toolchain.json"

sha() { sha256sum "$1" | cut -d' ' -f1; }

{
  echo '{'
  echo '  "$comment": "Written by scripts/vendor-semantic-schemas.sh; never edit by hand (FR-069-CON-2).",'
  echo '  "files": {'
  first=1
  while IFS= read -r f; do
    rel="${f#$OUT/}"
    case "$rel" in
      module-manifest.schema.json) repo=agent-ix/filament-core-service; rev=$FCS_REV; src=filament_core_service/schemas/module-manifest.schema.json ;;
      common.schema.json)          repo=agent-ix/filament-core-data;    rev=$FCD_REV; src=schema/semantic/v1/common.schema.json ;;
      semantic-core/*/toolchain.json) repo=agent-ix/filament-core-data; rev=$FCD_REV; src=packages/semantic-core/generated/toolchain.json ;;
      semantic-core/*)             repo=agent-ix/filament-core-data;    rev=$FCD_REV; src="$CORE_SRC/$(basename "$f")" ;;
    esac
    [ $first -eq 1 ] || echo ','
    first=0
    printf '    "%s": { "repository": "%s", "revision": "%s", "path": "%s", "sha256": "sha256:%s" }' \
      "$rel" "$repo" "$rev" "$src" "$(sha "$f")"
  done < <(find "$OUT" -type f -name '*.json' ! -name PROVENANCE.json | sort)
  echo
  echo '  },'
  # Bundle digest: sha256 over "<name>\n<bytes>" for every schema file in sorted
  # order, excluding toolchain.json — the same rule filament-core-data uses.
  digest=$(cd "$OUT/semantic-core/$CORE_VERSION" && for n in $(ls *.json | grep -v '^toolchain.json$' | sort); do printf '%s\n' "$n"; cat "$n"; done | sha256sum | cut -d' ' -f1)
  echo "  \"semanticCore\": { \"$CORE_VERSION\": { \"repository\": \"agent-ix/filament-core-data\", \"revision\": \"$FCD_REV\", \"path\": \"$CORE_SRC\", \"bundleDigest\": \"sha256:$digest\" } }"
  echo '}'
} > "$OUT/PROVENANCE.json"

echo "vendored $(find "$OUT" -type f -name '*.json' ! -name PROVENANCE.json | wc -l) files into $OUT"
grep -o '"bundleDigest": "[^"]*"' "$OUT/PROVENANCE.json"
grep -o '"digest": *"[^"]*"' "$OUT/semantic-core/$CORE_VERSION/toolchain.json"
