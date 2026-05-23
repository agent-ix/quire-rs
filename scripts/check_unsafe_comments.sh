#!/usr/bin/env bash
# Enforce that every `unsafe {` block in src/ has a `// SAFETY:` comment within
# the 3 lines preceding it. Pre-existing exemptions live in the baseline file
# below; regenerate with `--update-baseline`.
set -euo pipefail

baseline_file="scripts/unsafe_comment_baseline.txt"
update_baseline=false

if [[ "${1:-}" == "--update-baseline" ]]; then
  update_baseline=true
fi

if [[ ! -d src ]]; then
  exit 0
fi

unsafe_lines=()
while IFS= read -r line; do
  unsafe_lines+=("$line")
done < <(grep -rEn 'unsafe[[:space:]]*\{' src 2>/dev/null || true)

if [[ ${#unsafe_lines[@]} -eq 0 ]]; then
  exit 0
fi

missing_lines=()
missing=0
for entry in "${unsafe_lines[@]}"; do
  file=${entry%%:*}
  rest=${entry#*:}
  line=${rest%%:*}

  start=$(( line > 3 ? line - 3 : 1 ))
  if ! sed -n "${start},${line}p" "$file" | grep -q '// SAFETY:'; then
    missing_lines+=("${file}:${line}")
  fi
done

if [[ "$update_baseline" == true ]]; then
  if [[ ${#missing_lines[@]} -eq 0 ]]; then
    : > "$baseline_file"
    echo "wrote empty ${baseline_file}"
  else
    printf '%s\n' "${missing_lines[@]}" | sort -u > "$baseline_file"
    echo "wrote ${baseline_file} with ${#missing_lines[@]} entries"
  fi
  exit 0
fi

if [[ ${#missing_lines[@]} -eq 0 ]]; then
  exit 0
fi

if [[ ! -f "$baseline_file" ]]; then
  echo "missing unsafe comment baseline: ${baseline_file}" >&2
  echo "run: bash scripts/check_unsafe_comments.sh --update-baseline" >&2
  exit 1
fi

for entry in "${missing_lines[@]}"; do
  if ! grep -Fxq "$entry" "$baseline_file"; then
    echo "missing SAFETY comment near ${entry}" >&2
    missing=1
  fi
done

exit "$missing"
