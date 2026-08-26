#!/usr/bin/env bash
# Cheap structural gates that do NOT depend on the engine whose output they
# protect. Added after an outside review found committed conflict markers in
# `spec/functional/index.md` and FR-065 absent from it, while `make validate`
# reported "130 documents, 0 failed" and exited 0.
#
# Self-hosting is valuable and cannot be the only oracle: a semantic validator
# that parses a document will not tell you the document is a merge artifact.
set -euo pipefail
fail=0

# 1. Conflict markers, anywhere in spec/.
if grep -rn '^<<<<<<< \|^>>>>>>> \|^=======$' spec/ --include='*.md' 2>/dev/null; then
  echo "check_spec_structure: FAIL — committed conflict markers above" >&2
  fail=1
fi

# 2. Index completeness, BOTH directions.
for dir in spec/functional spec/non-functional spec/stakeholder; do
  [ -f "$dir/index.md" ] || continue
  for f in "$dir"/*.md; do
    base=$(basename "$f")
    [ "$base" = "index.md" ] && continue
    if ! grep -q "($base)\|(\./$base)" "$dir/index.md"; then
      echo "check_spec_structure: FAIL — $base is not in $dir/index.md" >&2
      fail=1
    fi
  done
  while read -r target; do
    [ -f "$dir/$target" ] || { echo "check_spec_structure: FAIL — $dir/index.md names $target, which does not exist" >&2; fail=1; }
  done < <(grep -oE '\]\(\./([^)]+\.md)\)' "$dir/index.md" | sed 's/](\.\///;s/)//')
done

# 3. Duplicate artifact ids (#348).
#
# Two documents claiming one id silently merge in every join keyed on it — the
# resolution table, the coverage rollup, any cross-reference. `quire validate`
# reads each document against its archetype and has no reason to compare two,
# so this is engine-independent by necessity rather than by preference.
dupes=$(grep -rhoE '^id: [A-Za-z]+-[0-9]+' spec/ --include='*.md' 2>/dev/null \
        | sort | uniq -d || true)
if [ -n "$dupes" ]; then
  echo "check_spec_structure: FAIL — duplicate artifact id(s):" >&2
  echo "$dupes" >&2
  while read -r d; do
    [ -n "$d" ] && grep -rln "^$d\$" spec/ --include='*.md' | sed 's/^/    /' >&2
  done <<< "$dupes"
  fail=1
fi

# 4. Unresolved LOCAL links (#348).
#
# A relative link to a file that does not exist. ADR-0007 makes internal
# references relative markdown, so a moved or renamed document leaves a link
# that renders fine and goes nowhere. Found NINE on its first run, every one a
# document that had been renamed and a link that had not.
#
# TWO NARROWINGS, both justified independently of the count (sampled 11: 2 rule,
# 9 real):
#
#   * a destination containing an ELLIPSIS is a placeholder — FR-026 and FR-039
#     quote link SYNTAX inside an acceptance criterion to state a rule ABOUT
#     links, and U+2026 is never part of a filename;
#   * a link inside a CODE SPAN is quoted syntax, not a link — markdown does not
#     render it, so it cannot be dead. ADR-0007 is the case: it demonstrates the
#     relative form in backticks.
#
# Anchors and `ix://` targets are out of scope: the first needs a heading index,
# the second names another repository.
while IFS= read -r src; do
  while read -r target; do
    [ -z "$target" ] && continue
    case "$target" in http*|ix://*|"#"*|*…*) continue;; esac
    clean=${target%%#*}
    [ -z "$clean" ] && continue
    if [ ! -e "$(dirname "$src")/$clean" ]; then
      echo "check_spec_structure: FAIL — $src links to $target, which does not exist" >&2
      fail=1
    fi
  done < <(sed 's/`[^`]*`//g' "$src" \
           | grep -oE '\]\(([^)]+\.md(#[^)]*)?)\)' \
           | sed -E 's/^\]\((.*)\)$/\1/' || true)
done < <(find spec -name '*.md' | sort)

# 5. Frontmatter schema validation is DELIBERATELY NOT HERE.
#
# It is `quire validate`'s entire job, against the archetype the module
# declares. A second, weaker reader of the same frontmatter in shell would be a
# rule stated twice and free to disagree with itself — which is the defect this
# bundle has now found five times (#279, #342, #352, #353, #316). The four
# gates above are here precisely because they are things a semantic validator
# has no reason to check: a validator that parses a document will not tell you
# the document is a merge artifact, that two documents claim one id, or that a
# link goes nowhere.

[ "$fail" -eq 0 ] && echo "check_spec_structure: OK"
exit "$fail"
