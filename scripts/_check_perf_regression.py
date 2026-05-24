#!/usr/bin/env python3
"""Parse criterion change/estimates.json files and enforce a band.

Invoked by scripts/check_perf_regression.sh — bench names are taken
from filesystem paths only, never from a shell-interpolated string.
"""
from __future__ import annotations

import json
import pathlib
import sys


def main(criterion_root: str, band_str: str) -> int:
    root = pathlib.Path(criterion_root)
    band = float(band_str)
    failed = 0
    seen = 0
    for f in root.rglob("change/estimates.json"):
        rel = f.relative_to(root).parent.parent  # strip /change/estimates.json
        try:
            data = json.loads(f.read_text())
            change = float(data["mean"]["point_estimate"])
        except (OSError, KeyError, ValueError) as e:
            print(f"  skip {rel} (unreadable: {e})")
            continue
        seen += 1
        if change > band:
            print(f"::error::regression in {rel}: +{change * 100:.1f}% "
                  f"(band: +{band * 100:.0f}%)")
            failed = 1
        elif change < -band:
            print(f"  speedup {rel}: {change * 100:+.1f}%")
        else:
            print(f"  ok      {rel}: {change * 100:+.1f}%")
    if seen == 0:
        print("check_perf_regression: no change/ files found "
              "(first run with --save-baseline?), skipping.")
        return 0
    if failed:
        print(f"check_perf_regression: regression band {band * 100:.0f}% "
              "exceeded by one or more benches.")
        return 1
    print("check_perf_regression: OK")
    return 0


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("usage: _check_perf_regression.py <criterion_root> <band>",
              file=sys.stderr)
        sys.exit(2)
    sys.exit(main(sys.argv[1], sys.argv[2]))
