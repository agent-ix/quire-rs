#!/usr/bin/env bash
# Enforce the 10% regression band on criterion benchmark outputs.
#
# Reads target/criterion/<bench>/<case>/change/estimates.json (written
# by `cargo bench -- --baseline main`) and fails when the mean point
# estimate has grown by >10%. Stays silent when there's no baseline
# yet (first CI run) or when the band is satisfied.
#
# Per NFR-001 / NFR-002 / NFR-007 (Task 014 / G3). The band is
# overridable via the QUIRE_PERF_BAND env var.
#
# The diff math runs in a separate Python helper that takes its
# arguments via argv only — bench paths never go through shell
# interpolation, eliminating the injection risk if a bench name
# ever carried a `'` or `"`.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CRITERION_ROOT="$ROOT/target/criterion"
ALLOWED_REGRESSION="${QUIRE_PERF_BAND:-0.10}"

if [[ ! -d "$CRITERION_ROOT" ]]; then
  echo "check_perf_regression: no criterion output found; first run, skipping."
  exit 0
fi

exec python3 "$ROOT/scripts/_check_perf_regression.py" \
  "$CRITERION_ROOT" \
  "$ALLOWED_REGRESSION"
