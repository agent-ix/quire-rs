#!/usr/bin/env python3
"""Fail closed when the governed YAML call-site census drifts (TC-1830)."""

from __future__ import annotations

import argparse
import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
DECLARATION = 'serde_yaml = { package = "yaml_serde", version = "=0.10.7" }'
SKIP_PARTS = {".git", "target", "tools"}


def occurrences(path: pathlib.Path) -> int:
    text = path.read_text(errors="strict")
    if "yaml_serde::" in text:
        raise ValueError(f"{path.relative_to(ROOT)} uses the package name directly; keep serde_yaml alias")
    return len(re.findall(r"\bserde_yaml::", text))


def load_expected() -> dict[str, object]:
    return json.loads((ROOT / "quality/yaml-callsite-census.json").read_text())


def actual_callsites() -> dict[str, int]:
    found: dict[str, int] = {}
    for path in ROOT.rglob("*.rs"):
        relative = path.relative_to(ROOT)
        if any(part in SKIP_PARTS for part in relative.parts):
            continue
        count = occurrences(path)
        if count:
            found[relative.as_posix()] = count
    return dict(sorted(found.items()))


def audit() -> dict[str, object]:
    expected_doc = load_expected()
    rows = expected_doc["callsites"]
    assert isinstance(rows, list)
    expected = {str(row["path"]): int(row["count"]) for row in rows}
    actual = actual_callsites()
    errors: list[str] = []
    if actual != dict(sorted(expected.items())):
        missing = sorted(set(expected) - set(actual))
        unexpected = sorted(set(actual) - set(expected))
        changed = sorted(
            path for path in set(actual) & set(expected) if actual[path] != expected[path]
        )
        if missing:
            errors.append(f"missing classified callsites: {missing}")
        if unexpected:
            errors.append(f"unclassified callsites: {unexpected}")
        if changed:
            errors.append(
                "call counts changed: "
                + ", ".join(f"{path} expected={expected[path]} actual={actual[path]}" for path in changed)
            )

    required = set(map(str, expected_doc["required_production_classes"]))
    covered = {
        str(class_name)
        for row in rows
        if row["scope"] == "production"
        for class_name in row["classes"]
    }
    if covered != required:
        errors.append(
            f"production class coverage differs: missing={sorted(required - covered)} "
            f"unexpected={sorted(covered - required)}"
        )

    for relative in ["Cargo.toml", "fuzz/Cargo.toml"]:
        text = (ROOT / relative).read_text()
        if DECLARATION not in text:
            errors.append(f"{relative} lacks exact YAML alias declaration")

    report: dict[str, object] = {
        "schema": expected_doc["schema"],
        "dependency_alias": DECLARATION,
        "excluded_standalone_tool": "tools/yaml-differential",
        "required_production_classes": sorted(required),
        "covered_production_classes": sorted(covered),
        "callsites": [
            {
                "path": row["path"],
                "scope": row["scope"],
                "count": actual.get(str(row["path"]), 0),
                "classes": row["classes"],
            }
            for row in rows
        ],
        "errors": errors,
    }
    return report


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()
    try:
        report = audit()
    except (OSError, ValueError, json.JSONDecodeError, AssertionError) as error:
        print(f"check_yaml_callsites: FAIL — {error}", file=sys.stderr)
        return 1
    if args.json:
        print(json.dumps(report, indent=2, sort_keys=True))
    if report["errors"]:
        for error in report["errors"]:
            print(f"check_yaml_callsites: FAIL — {error}", file=sys.stderr)
        return 1
    if not args.json:
        print("check_yaml_callsites: OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
