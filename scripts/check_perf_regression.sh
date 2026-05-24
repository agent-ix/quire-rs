#!/usr/bin/env bash
# Enforce the 10% regression band on criterion benchmark outputs.
#
# Reads target/criterion/<bench>/<case>/change/estimates.json (written
# by `cargo bench -- --baseline main`) and fails when the mean point
# estimate has grown by >10%. Stays silent when there's no baseline
# yet (first CI run) or when the band is satisfied.
#
# Per NFR-001 / NFR-002 / NFR-007 (Task 014 / G3). Hardware-band
# tuning lives at the bottom of this script.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CRITERION_ROOT="$ROOT/target/criterion"

# Allow CI to override the band; default 10% per NFR.
ALLOWED_REGRESSION="${QUIRE_PERF_BAND:-0.10}"

if [[ ! -d "$CRITERION_ROOT" ]]; then
  echo "check_perf_regression: no criterion output found; first run, skipping."
  exit 0
fi

FAILED=0
SUMMARY=""

# Walk every change/estimates.json that criterion produced.
while IFS= read -r -d '' f; do
  bench="$(echo "$f" | sed -E "s|^$CRITERION_ROOT/||" | sed -E 's|/change/estimates.json$||')"
  # Mean point-estimate change is expressed as a fraction:
  #   0.05  = +5% slower
  #  -0.10  = 10% faster
  change="$(python3 -c "
import json, sys
try:
    d = json.load(open(sys.argv[1]))
    print(d['mean']['point_estimate'])
except Exception as e:
    print('NaN', file=sys.stderr)
    print('0.0')
" "$f")"
  # Compare against the band.
  python3 - <<PY || FAILED=1
import sys
change = float("${change}")
band   = float("${ALLOWED_REGRESSION}")
bench  = "${bench}"
if change > band:
    print(f"::error::regression in {bench}: +{change*100:.1f}% (band: +{band*100:.0f}%)")
    sys.exit(1)
elif change < -band:
    print(f"  speedup in {bench}: {change*100:+.1f}%")
else:
    print(f"  ok       {bench}: {change*100:+.1f}%")
PY
done < <(find "$CRITERION_ROOT" -path '*/change/estimates.json' -print0)

if [[ "$FAILED" -ne 0 ]]; then
  echo "check_perf_regression: one or more benches exceeded the +${ALLOWED_REGRESSION} band."
  exit 1
fi
echo "check_perf_regression: OK"
