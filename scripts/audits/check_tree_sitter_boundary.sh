#!/usr/bin/env bash
# check_tree_sitter_boundary.sh — PLAT-911, FR-051-CON-1 (related: TC-1883,
# a `scripts/audits/*.sh` check run by `make audit-static` asserting the
# symbol-extraction path reaches a grammar-driven parser library through one
# entry point; this script covers that entry point's Cargo.toml facet only —
# it does not assert TC-1883's compiler/package-manager/type-resolution
# clauses, so it does not move that row to done).
#
# `crates/quire-rust-extraction/Cargo.toml` describes itself as "the only
# place in the quire-rs workspace that names quire-code-parse/tree-sitter"
# (PLAT-843, widened PLAT-851). Nothing checked that claim — `grep -rln
# 'quire-code-parse\|tree.sitter' scripts/` returned no matches — so a second
# crate could add `tree-sitter` (or `tree-sitter-<lang>`, or `quire-code-parse`
# itself) to its own Cargo.toml, pass every gate, and silently reopen the
# boundary PLAT-843 closed. That boundary is load-bearing, not stylistic: the
# `wasm` build must be able to opt out of every grammar, since a
# `tree-sitter-<lang>` grammar's `cc`-crate build script cannot cross-compile
# to `wasm32-unknown-unknown`, and `tree-sitter` is a C dependency, so a
# second entry point widens the unsafe and supply-chain surface with nobody
# having decided to.
#
# WHAT THIS MATCHES: a genuine Cargo dependency *declaration* for
# `quire-code-parse` or `tree-sitter`/`tree-sitter-<lang>` in any git-tracked
# `Cargo.toml` other than the boundary crate's own — the inline-table key
# form (`tree-sitter-rust = "0.24"` / `tree-sitter-rust = { version = ... }`),
# the dotted-table-header form (`[dependencies.tree-sitter-rust]`), and a
# renamed dependency's `package = "tree-sitter-rust"` value, each only when
# the line is not a comment.
#
# WHAT THIS DELIBERATELY DOES NOT MATCH: prose. The root `Cargo.toml` and the
# boundary crate's own `Cargo.toml` both discuss `tree-sitter`/
# `quire-code-parse` at length in `#`-prefixed comment blocks and in the
# `description` field — legitimate, and not a dependency declaration. This
# script is comment-aware (a line whose trimmed text opens with `#` is never
# treated as a match) and only looks at git-tracked `Cargo.toml` files (`git
# ls-files`), so it never reads a build artifact, an untracked worktree, or a
# `description = "..."` string as if it were a dependency.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

EXEMPT="crates/quire-rust-extraction/Cargo.toml"

# The dependency name this boundary admits exactly one namer for: the crate
# itself (`quire-code-parse`) and its grammars (`tree-sitter`, bare, or
# `tree-sitter-<lang>`).
NAME_RE='(quire-code-parse|tree-sitter(-[A-Za-z0-9_.]+)?)'

# 1. `name = ...` at the start of a (possibly indented) line — the inline key
#    form, quoted or bare.
KEY_RE="^[[:space:]]*\"?${NAME_RE}\"?[[:space:]]*="
# 2. `[dependencies.name]` / `[dev-dependencies.name]` /
#    `[build-dependencies.name]` / `[workspace.dependencies.name]` /
#    `[target.'cfg(...)'.dependencies.name]` — the dotted-table-header form.
HEADER_RE="^[[:space:]]*\[(dependencies|dev-dependencies|build-dependencies|workspace\.dependencies|target\.[^]]+\.dependencies)\.\"?${NAME_RE}\"?\]"
# 3. `package = "name"` — an aliased dependency still names the real crate
#    through its `package` key, e.g. `ts_rust = { package = "tree-sitter-rust", ... }`.
ALIAS_RE="package[[:space:]]*=[[:space:]]*\"${NAME_RE}\""

violations=()
while IFS= read -r rel; do
  [[ "$rel" == "$EXEMPT" ]] && continue
  [[ -f "$rel" ]] || continue
  while IFS=: read -r lineno content; do
    [[ -z "$lineno" ]] && continue
    trimmed="${content#"${content%%[![:space:]]*}"}"
    [[ "$trimmed" == \#* ]] && continue
    violations+=("$rel:$lineno: $trimmed")
  done < <(grep -nE -e "$KEY_RE" -e "$HEADER_RE" -e "$ALIAS_RE" "$rel" || true)
done < <(git ls-files -- '*Cargo.toml')

if [[ "${#violations[@]}" -gt 0 ]]; then
  echo "check_tree_sitter_boundary: FAIL — a second crate names quire-code-parse/tree-sitter" >&2
  echo "  directly; the boundary crate ($EXEMPT) must stay the only namer (PLAT-843/851):" >&2
  printf '  %s\n' "${violations[@]}" >&2
  exit 1
fi

echo "check_tree_sitter_boundary: OK — quire-code-parse/tree-sitter named only by $EXEMPT"
