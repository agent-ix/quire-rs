#!/usr/bin/env python3
"""Compare per-repository coverage snapshots (FR-050-AC-31).

Take two snapshots with the same explicit repository root and module, then
compare the readable intersection. Unreadable repositories are retained in
each snapshot's ``unreadable`` list and are never scored as zero.

Examples::

    python3 scripts/overfit_check.py --root /approved/corpus --snapshot before.json
    # Change the engine.
    python3 scripts/overfit_check.py --root /approved/corpus --snapshot after.json
    python3 scripts/overfit_check.py --compare before.json after.json

The root is explicit so a broad corpus walk is always a deliberate invocation.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path
from typing import Any

sys.path.insert(0, str(Path(__file__).resolve().parent))

from corpus import repos  # noqa: E402

CONCENTRATION_FLOOR = 0.5


# implements: FR-050-AC-31
def _nonnegative_int(value: Any) -> bool:
    return isinstance(value, int) and not isinstance(value, bool) and value >= 0


# implements: FR-050-AC-31
def measure(quire: str, module: str | None, repo: Path) -> dict[str, int] | None:
    """Return one repository's metrics, or ``None`` when its corpus is unreadable."""
    cmd = [quire, "coverage", "--scope", str(repo), "--json"]
    if module:
        cmd.extend(["--module", module])
    try:
        done = subprocess.run(cmd, capture_output=True, text=True, timeout=300, check=False)
    except (OSError, subprocess.TimeoutExpired):
        return None
    if done.returncode != 0 or not done.stdout.strip():
        return None
    try:
        payload = json.loads(done.stdout)
    except json.JSONDecodeError:
        return None

    totals = payload.get("totals") if isinstance(payload, dict) else None
    if not isinstance(totals, dict):
        return None
    backed, total = totals.get("backed"), totals.get("total")
    if not _nonnegative_int(backed) or not _nonnegative_int(total):
        return None

    census = payload.get("binding_census", [])
    if not isinstance(census, list):
        return None
    candidates = bound = 0
    for row in census:
        if not isinstance(row, dict):
            return None
        row_candidates, row_bound = row.get("candidates"), row.get("bound")
        if not _nonnegative_int(row_candidates) or not _nonnegative_int(row_bound):
            return None
        candidates += row_candidates
        bound += row_bound

    result = {
        "backed": backed,
        "total": total,
        "dead_tags": len(payload.get("untracked_symbols", [])),
        "suspicions": len(payload.get("suspicions", [])),
    }
    if census and candidates:
        result["bound"] = bound
        result["candidates"] = candidates
    return result


# implements: FR-050-AC-31
def snapshot(quire: str, module: str | None, root: Path, limit: int = 0) -> dict[str, Any]:
    """Measure repositories under ``root`` and preserve unreadable names."""
    if limit < 0:
        raise ValueError("limit must be non-negative")
    if not root.is_dir():
        raise ValueError(f"repository root is not a directory: {root}")
    all_repos = repos(root)
    if limit:
        all_repos = all_repos[:limit]

    entries: dict[str, dict[str, int]] = {}
    unreadable: list[str] = []
    for repo in all_repos:
        metrics = measure(quire, module, repo)
        if metrics is None:
            unreadable.append(repo.name)
            print(f"  {repo.name:38s} unreadable", file=sys.stderr)
            continue
        entries[repo.name] = metrics
        print(f"  {repo.name:38s} {metrics['backed']}/{metrics['total']}", file=sys.stderr)
    return {
        "schema_version": 1,
        "repos": entries,
        "unreadable": sorted(unreadable),
        "population": len(all_repos),
    }


# implements: FR-050-AC-31
def pct(num: int, den: int) -> float:
    return round((num * 100.0) / den, 2) if den else 0.0


# implements: FR-050-AC-31
def _validate_snapshot(data: Any, label: str) -> dict[str, Any]:
    if not isinstance(data, dict) or data.get("schema_version") != 1:
        raise ValueError(f"{label} snapshot has unsupported or missing schema_version")
    entries = data.get("repos")
    unreadable = data.get("unreadable")
    population = data.get("population")
    if not isinstance(entries, dict) or not isinstance(unreadable, list):
        raise ValueError(f"{label} snapshot must contain repos and unreadable collections")
    if not _nonnegative_int(population):
        raise ValueError(f"{label} snapshot population must be a non-negative integer")
    if population != len(entries) + len(unreadable):
        raise ValueError(f"{label} snapshot population does not match its readable/unreadable entries")
    if not all(isinstance(name, str) and name for name in unreadable):
        raise ValueError(f"{label} snapshot has an invalid unreadable repository name")
    if len(set(unreadable)) != len(unreadable) or set(entries) & set(unreadable):
        raise ValueError(f"{label} snapshot repeats a repository across its populations")
    for name, row in entries.items():
        if not isinstance(name, str) or not name or not isinstance(row, dict):
            raise ValueError(f"{label} snapshot has an invalid repository record")
        if not _nonnegative_int(row.get("backed")) or not _nonnegative_int(row.get("total")):
            raise ValueError(f"{label} snapshot has invalid backed/total metrics for {name}")
    return data


# implements: FR-050-AC-31
def compare(before: dict[str, Any], after: dict[str, Any]) -> dict[str, Any]:
    """Compare two snapshots over the readable intersection, without netting losses."""
    before = _validate_snapshot(before, "before")
    after = _validate_snapshot(after, "after")
    before_repos = before["repos"]
    after_repos = after["repos"]
    shared = sorted(set(before_repos) & set(after_repos))
    rows = []
    for name in shared:
        old, new = before_repos[name], after_repos[name]
        rows.append(
            {
                "repo": name,
                "backed_delta": new["backed"] - old["backed"],
                "backed_pct_before": pct(old["backed"], old["total"]),
                "backed_pct_after": pct(new["backed"], new["total"]),
                "dead_tags_delta": new.get("dead_tags", 0) - old.get("dead_tags", 0),
            }
        )

    gains = [row for row in rows if row["backed_delta"] > 0]
    losses = [row for row in rows if row["backed_delta"] < 0]
    total_gain = sum(row["backed_delta"] for row in gains)
    total_loss = sum(-row["backed_delta"] for row in losses)
    net_gain = total_gain - total_loss
    top = max(gains, key=lambda row: row["backed_delta"], default=None)
    concentration = top["backed_delta"] / total_gain if top and total_gain else 0.0
    only_before = sorted(set(before_repos) - set(after_repos))
    only_after = sorted(set(after_repos) - set(before_repos))

    return {
        "rows": rows,
        "shared": len(shared),
        "only_before": only_before,
        "only_after": only_after,
        "unreadable_before": sorted(before["unreadable"]),
        "unreadable_after": sorted(after["unreadable"]),
        "population_before": before["population"],
        "population_after": after["population"],
        "improved": len(gains),
        "regressed": len(losses),
        "unchanged": len(rows) - len(gains) - len(losses),
        "total_gain": total_gain,
        "total_loss": total_loss,
        "net_gain": net_gain,
        "top_repo": top["repo"] if top else None,
        "top_gain": top["backed_delta"] if top else 0,
        "concentration": round(concentration, 3),
    }


# implements: FR-050-AC-31
def render(diff: dict[str, Any]) -> str:
    """Render distribution counts, net/gross deltas and population changes."""
    lines = [
        f"repositories compared: {diff['shared']}",
        f"  improved  {diff['improved']}",
        f"  regressed {diff['regressed']}",
        f"  unchanged {diff['unchanged']}",
        f"  gross gain: {diff['total_gain']:+d} rows",
        f"  gross loss: {-diff['total_loss']:+d} rows",
        f"  net rows backed: {diff['net_gain']:+d}",
    ]
    if diff["only_before"] or diff["only_after"] or diff["population_before"] != diff["population_after"]:
        lines.append(
            f"  population moved: {len(diff['only_before'])} readable repositories dropped, "
            f"{len(diff['only_after'])} appeared; compare uses only the {diff['shared']} "
            "readable repositories present in both snapshots"
        )
    if diff["unreadable_before"] or diff["unreadable_after"]:
        lines.append(
            f"  unreadable: before={len(diff['unreadable_before'])} "
            f"{diff['unreadable_before']}; after={len(diff['unreadable_after'])} "
            f"{diff['unreadable_after']}"
        )
    if diff["top_repo"]:
        lines.append(
            f"  concentration: {diff['concentration']:.0%} of gross gain is "
            f"{diff['top_repo']} ({diff['top_gain']:+d} rows)"
        )
        if diff["concentration"] >= CONCENTRATION_FLOOR:
            lines.append(
                "  ^ most of the improvement is one repository. Check whether the "
                "change is about the engine or that corpus before ratcheting on it."
            )
    for row in sorted(diff["rows"], key=lambda item: (item["backed_delta"], item["repo"])):
        if row["backed_delta"]:
            lines.append(
                f"    {row['repo']:38s} {row['backed_delta']:+5d} rows "
                f"({row['backed_pct_before']}% -> {row['backed_pct_after']}%)"
            )
    return "\n".join(lines)


# implements: FR-050-AC-31
def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    operation = parser.add_mutually_exclusive_group(required=True)
    operation.add_argument("--snapshot", type=Path, help="write a snapshot here")
    operation.add_argument("--compare", type=Path, nargs=2, metavar=("BEFORE", "AFTER"))
    parser.add_argument("--quire", default="quire")
    parser.add_argument("--module", default=None)
    parser.add_argument("--root", type=Path, help="explicit root whose child repositories are enumerated")
    parser.add_argument("--limit", type=int, default=0, help="smoke-run only the first N repositories")
    args = parser.parse_args()

    if args.compare:
        try:
            before = json.loads(args.compare[0].read_text(encoding="utf-8"))
            after = json.loads(args.compare[1].read_text(encoding="utf-8"))
            print(render(compare(before, after)))
        except (OSError, json.JSONDecodeError, ValueError) as error:
            print(f"overfit_check: {error}", file=sys.stderr)
            return 1
        return 0

    if args.root is None:
        parser.error("--root is required for a snapshot; choose a bounded corpus root explicitly")
    try:
        snap = snapshot(args.quire, args.module, args.root.expanduser(), args.limit)
    except (OSError, ValueError) as error:
        print(f"overfit_check: {error}", file=sys.stderr)
        return 1
    try:
        args.snapshot.write_text(json.dumps(snap, indent=1, sort_keys=True) + "\n", encoding="utf-8")
    except OSError as error:
        print(f"overfit_check: cannot write snapshot: {error}", file=sys.stderr)
        return 1
    print(
        f"{len(snap['repos'])} of {snap['population']} repositories read "
        f"({len(snap['unreadable'])} unreadable) -> {args.snapshot}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
