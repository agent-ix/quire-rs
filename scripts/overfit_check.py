#!/usr/bin/env python3
"""Cross-corpus overfit check (quire-rs#237, FR-050-AC-31).

An improvement tuned against `filament-ide-rs` might be an improvement to the
engine, or it might be an improvement to `filament-ide-rs`. One corpus cannot
tell those apart. This sweeps the 241 repositories `scripts/corpus.py`
enumerates and reports **how a gain is distributed** — because that is the
question, and a single ecosystem-wide average hides the answer.

    python3 scripts/overfit_check.py --snapshot before.json
    …change the engine…
    python3 scripts/overfit_check.py --snapshot after.json
    python3 scripts/overfit_check.py --compare before.json after.json

`workflow_dispatch` only, never push-triggered: a 241-repo sweep is minutes of
work and a gate that runs on every push is a gate somebody disables.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from corpus import repos  # noqa: E402

# A gain is "concentrated" when this fraction or more of the total improvement
# comes from a single repository. Not a pass/fail line — the script reports the
# number and names the repo, because whether that is overfitting depends on
# what changed, and a script cannot know that.
CONCENTRATION_FLOOR = 0.5


def measure(quire: str, module: str | None, repo: Path) -> dict | None:
    """One repository's numbers, or None when it could not be read.

    None rather than zeros, for the reason the whole programme exists: a repo
    the engine could not read, scored as 0, is indistinguishable from a repo
    with nothing in it.
    """
    cmd = [quire, "coverage", "--scope", str(repo), "--json"]
    if module:
        cmd += ["--module", module]
    try:
        done = subprocess.run(cmd, capture_output=True, text=True, timeout=300, check=False)
    except subprocess.TimeoutExpired:
        return None
    if done.returncode != 0 or not done.stdout.strip():
        return None
    try:
        payload = json.loads(done.stdout)
    except json.JSONDecodeError:
        return None

    totals = payload.get("totals", {})
    backed, total = totals.get("backed", 0), totals.get("total", 0)
    census = payload.get("binding_census", [])
    candidates = sum(c["candidates"] for c in census)
    bound = sum(c["bound"] for c in census)
    out = {
        "backed": backed,
        "total": total,
        "dead_tags": len(payload.get("untracked_symbols", [])),
        "suspicions": len(payload.get("suspicions", [])),
    }
    # Omitted, not zeroed, when the engine predates the field or the repo has
    # no evidence symbols — the two are different and neither is 0%.
    if census and candidates:
        out["bound"] = bound
        out["candidates"] = candidates
    return out


def snapshot(quire: str, module: str | None, root: Path, limit: int) -> dict:
    entries: dict[str, dict] = {}
    unreadable: list[str] = []
    all_repos = repos(root)
    if limit:
        all_repos = all_repos[:limit]
    for repo in all_repos:
        got = measure(quire, module, repo)
        if got is None:
            unreadable.append(repo.name)
            print(f"  {repo.name:38s} unreadable", file=sys.stderr)
            continue
        entries[repo.name] = got
        print(f"  {repo.name:38s} {got['backed']}/{got['total']}", file=sys.stderr)
    return {
        "repos": entries,
        # Carried, not dropped: a sweep that silently shrank its own population
        # would show every remaining repo improving.
        "unreadable": sorted(unreadable),
        "population": len(all_repos),
    }


def pct(num: int, den: int) -> float:
    return round((num * 100.0) / den, 2) if den else 0.0


def compare(before: dict, after: dict) -> dict:
    """Per-repo deltas plus the distribution statistics that answer the question."""
    shared = sorted(set(before["repos"]) & set(after["repos"]))
    rows = []
    for name in shared:
        b, a = before["repos"][name], after["repos"][name]
        rows.append(
            {
                "repo": name,
                "backed_delta": a["backed"] - b["backed"],
                "backed_pct_before": pct(b["backed"], b["total"]),
                "backed_pct_after": pct(a["backed"], a["total"]),
                "dead_tags_delta": a["dead_tags"] - b["dead_tags"],
            }
        )

    gains = [r for r in rows if r["backed_delta"] > 0]
    losses = [r for r in rows if r["backed_delta"] < 0]
    total_gain = sum(r["backed_delta"] for r in gains)
    top = max(gains, key=lambda r: r["backed_delta"], default=None)
    concentration = (top["backed_delta"] / total_gain) if top and total_gain else 0.0

    return {
        "rows": rows,
        "shared": len(shared),
        "only_before": sorted(set(before["repos"]) - set(after["repos"])),
        "only_after": sorted(set(after["repos"]) - set(before["repos"])),
        "improved": len(gains),
        "regressed": len(losses),
        "unchanged": len(rows) - len(gains) - len(losses),
        "total_gain": total_gain,
        "top_repo": top["repo"] if top else None,
        "top_gain": top["backed_delta"] if top else 0,
        "concentration": round(concentration, 3),
    }


def render(diff: dict) -> str:
    lines = [
        f"repositories compared: {diff['shared']}",
        f"  improved  {diff['improved']}",
        f"  regressed {diff['regressed']}",
        f"  unchanged {diff['unchanged']}",
        f"  net rows backed: {diff['total_gain']:+d}",
    ]
    if diff["only_before"] or diff["only_after"]:
        lines.append(
            f"  population moved: {len(diff['only_before'])} dropped, "
            f"{len(diff['only_after'])} appeared — the comparison is over the "
            f"{diff['shared']} present in both"
        )
    if diff["top_repo"]:
        lines.append(
            f"  concentration: {diff['concentration']:.0%} of the gain is "
            f"{diff['top_repo']} ({diff['top_gain']:+d} rows)"
        )
        if diff["concentration"] >= CONCENTRATION_FLOOR:
            lines.append(
                "  ^ most of the improvement is one repository. That is what "
                "overfitting looks like — check whether the change is about the "
                "engine or about that corpus before ratcheting on it."
            )
    for row in sorted(diff["rows"], key=lambda r: r["backed_delta"]):
        if row["backed_delta"]:
            lines.append(
                f"    {row['repo']:38s} {row['backed_delta']:+5d} rows "
                f"({row['backed_pct_before']}% → {row['backed_pct_after']}%)"
            )
    return "\n".join(lines)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--snapshot", type=Path, help="write a snapshot here")
    ap.add_argument("--compare", type=Path, nargs=2, metavar=("BEFORE", "AFTER"))
    ap.add_argument("--quire", default="quire")
    ap.add_argument("--module", default=None)
    ap.add_argument("--root", type=Path, default=Path.home() / "dev")
    ap.add_argument("--limit", type=int, default=0,
                    help="sweep only the first N repos (smoke runs)")
    args = ap.parse_args()

    if args.compare:
        before = json.loads(args.compare[0].read_text())
        after = json.loads(args.compare[1].read_text())
        print(render(compare(before, after)))
        return 0

    if not args.snapshot:
        ap.error("one of --snapshot or --compare is required")
    snap = snapshot(args.quire, args.module, args.root, args.limit)
    args.snapshot.write_text(json.dumps(snap, indent=1, sort_keys=True) + "\n")
    readable = len(snap["repos"])
    print(
        f"{readable} of {snap['population']} repositories read "
        f"({len(snap['unreadable'])} unreadable) → {args.snapshot}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
