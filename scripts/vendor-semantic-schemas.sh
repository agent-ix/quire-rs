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

FCS_REV="5b2af8b18c9ed5ca919217dea17fdf440df95bae"   # agent-ix/filament-core-service
FCD_REV="d48b8da7ae5e40b8b3d465d45b2bd3e24b994dbb"   # agent-ix/filament-core-data (common.schema.json)
# Every embedded semantic-core version and the filament-core-data revision it
# is generated at, ascending. A module manifest pins one by `semantic_core`.
CORES=(
  "0.1.0 d48b8da7ae5e40b8b3d465d45b2bd3e24b994dbb"
  "0.2.0 739b160b89954c15652799a7620cc6d08f039b41"
)

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
show "$DEV/filament-core-data" agent-ix/filament-core-data "$FCD_REV" \
  schema/semantic/v1/common.schema.json > "$OUT/common.schema.json"

CORE_SRC="packages/semantic-core/generated/json-schema"
core_rev() { # version -> revision
  local entry
  for entry in "${CORES[@]}"; do
    [ "${entry%% *}" = "$1" ] && { echo "${entry#* }"; return; }
  done
  return 1
}
for entry in "${CORES[@]}"; do
  version="${entry%% *}"; rev="${entry#* }"
  mkdir -p "$OUT/semantic-core/$version"
  for name in $(git -C "$DEV/filament-core-data" ls-tree --name-only "$rev" "$CORE_SRC/" | xargs -n1 basename | sort); do
    show "$DEV/filament-core-data" agent-ix/filament-core-data "$rev" "$CORE_SRC/$name" \
      > "$OUT/semantic-core/$version/$name"
  done
  show "$DEV/filament-core-data" agent-ix/filament-core-data "$rev" \
    packages/semantic-core/generated/toolchain.json > "$OUT/semantic-core/$version/toolchain.json"
done

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
      semantic-core/*/toolchain.json) repo=agent-ix/filament-core-data; rev=$(core_rev "$(basename "$(dirname "$f")")"); src=packages/semantic-core/generated/toolchain.json ;;
      semantic-core/*)             repo=agent-ix/filament-core-data;    rev=$(core_rev "$(basename "$(dirname "$f")")"); src="$CORE_SRC/$(basename "$f")" ;;
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
  echo '  "semanticCore": {'
  first=1
  for entry in "${CORES[@]}"; do
    version="${entry%% *}"; rev="${entry#* }"
    digest=$(cd "$OUT/semantic-core/$version" && for n in $(ls *.json | grep -v '^toolchain.json$' | sort); do printf '%s\n' "$n"; cat "$n"; done | sha256sum | cut -d' ' -f1)
    [ $first -eq 1 ] || echo ','
    first=0
    printf '    "%s": { "repository": "agent-ix/filament-core-data", "revision": "%s", "path": "%s", "bundleDigest": "sha256:%s" }' \
      "$version" "$rev" "$CORE_SRC" "$digest"
  done
  echo
  echo '  }'
  echo '}'
} > "$OUT/PROVENANCE.json"

echo "vendored $(find "$OUT" -type f -name '*.json' ! -name PROVENANCE.json | wc -l) files into $OUT"
grep -o '"bundleDigest": "[^"]*"' "$OUT/PROVENANCE.json"
for entry in "${CORES[@]}"; do
  grep -o '"digest": *"[^"]*"' "$OUT/semantic-core/${entry%% *}/toolchain.json"
done

# Regenerate the embedded bundle module so the crate and schemas/vendored/
# cannot drift (FR-069-AC-5: the resolver runs from memory under `wasm`).
VERSIONS="$(for entry in "${CORES[@]}"; do echo "${entry%% *}"; done)" python3 - <<'PY'
import os, pathlib
base=pathlib.Path('schemas/vendored')
versions=os.environ['VERSIONS'].split()
src=pathlib.Path('src/semantic/vendored.rs').read_text()
head, _, _ = src.partition('/// Semantic-core versions with an embedded bundle')
const=lambda v: 'SEMANTIC_CORE_' + v.replace('.', '_')
body='/// Semantic-core versions with an embedded bundle, ascending.\npub const SEMANTIC_CORE_VERSIONS: &[&str] = &[' + ', '.join(f'"{v}"' for v in versions) + '];\n'
for v in versions:
    core=sorted(p.name for p in (base/'semantic-core'/v).glob('*.json') if p.name!='toolchain.json')
    body+=f'\n/// `(file name, bytes)` for every schema of semantic-core {v}, sorted by name;\n/// `toolchain.json` is provenance, not a schema, and is excluded.\npub const {const(v)}: &[(&str, &str)] = &[\n' + ''.join(f'    ("{n}", include_str!("../../schemas/vendored/semantic-core/{v}/{n}")),\n' for n in core) + '];\n'
body+='\n/// The embedded bundle for `version`, if any.\npub fn semantic_core_bundle(version: &str) -> Option<&\'static [(&\'static str, &\'static str)]> {\n    match version {\n' + ''.join(f'        "{v}" => Some({const(v)}),\n' for v in versions) + '        _ => None,\n    }\n}\n'
pathlib.Path('src/semantic/vendored.rs').write_text(head+body)
PY
echo "regenerated src/semantic/vendored.rs"
