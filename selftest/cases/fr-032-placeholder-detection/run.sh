#!/bin/sh
# trace: FR-032-AC-3
set -eu

stderr_file="$CASE_TMP/validate.stderr"
if "$QUIRE" validate "$CASE_TMP/input/FR-001.md" \
    --module "$CASE_TMP/input/module" >/dev/null 2>"$stderr_file"; then
    status=0
else
    status=$?
fi

if [ "$status" -ne 3 ]; then
    printf 'expected validation exit 3; got %s\n' "$status"
    exit 1
fi

found_placeholder=0
while IFS= read -r line; do
    case "$line" in
        *placeholder*) found_placeholder=1 ;;
    esac
done < "$stderr_file"

if [ "$found_placeholder" -ne 1 ]; then
    printf 'validation error did not name placeholder\n'
    exit 1
fi

printf 'placeholder rejected\n'
