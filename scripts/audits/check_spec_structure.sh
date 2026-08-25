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
  prefix=$(basename "$dir" | tr '[:lower:]' '[:upper:]' | cut -c1-3)
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

[ "$fail" -eq 0 ] && echo "check_spec_structure: OK"
exit "$fail"
