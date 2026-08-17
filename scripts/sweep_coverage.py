#!/usr/bin/env python3
"""Re-derive the dead-trace-tag population across the ecosystem.

quire-rs#78: the numbers in #72 ("153 of 237 repos mint zero test-case targets,
1,014 written trace tags bind to nothing") were measured through a corpus walk
that could not see any file named `tests.md`. Type-driven membership landed in
v0.26.0; this re-measures against a released engine.

Two things the earlier sweep got wrong, both fixed here:

* **The engine must be released.** CR-061 (v0.27.0) widened `trace::bind` to
  benchmarks and fuzz targets, so a tag on a `criterion_group!` bench resolves
  where it previously did not. Numbers taken on an older engine are stale on
  arrival. `--engine` is recorded in the output.
* **The module must not be the stale installed copy.** `~/.ix/filament/modules`
  lags the source repository — at the time of writing it was missing the Phase D
  comma-list trace patterns (205 ids across 17 repos) and the `tests/**`
  exclusions on the FR/NFR archetype targets. Pass `--module` pointing at the
  source tree so the model is the current one, and record which.

Dead tags are `untracked_symbols`: a trace marker written in source whose id no
trace target ever minted. A repo "mints zero test-case targets" when none of the
three declared matrix paths carries a `## Test Case Summary` — the section the
`test-case` target reads.

Usage:
  python3 sweep_coverage.py --quire <bin> --module <dir> [root]
"""

from __future__ import annotations

import argparse
import json
import pathlib
import re
import subprocess
import sys

# Same dedupe rules as classify_matrices.py — a worktree or a `-task<N>` scratch
# copy is the same repo counted twice. The 2026-08-14 sweep reported 184 matrices
# where the truth was 183 for exactly this reason.
TASK_COPY = re.compile(r"-task\d+$")
SUPERSEDED = {"filament-ide"}
DECLARED_PATHS = ("spec/tests.md", "spec/matrix.md", "spec/evals.md")
SUMMARY = re.compile(r"^##\s+Test Case Summary\s*$", re.MULTILINE)


def repos(root: pathlib.Path) -> list[pathlib.Path]:
    out = []
    for child in sorted(root.iterdir()):
        if not child.is_dir() or child.name.startswith("."):
            continue
        if TASK_COPY.search(child.name) or child.name in SUPERSEDED:
            continue
        if (child / "spec").is_dir():
            out.append(child)
    return out


def mints_test_cases(repo: pathlib.Path) -> bool:
    """A `test-case` target reads `## Test Case Summary` from one of the three
    declared matrix paths. No summary at any of them, no minted ids."""
    for rel in DECLARED_PATHS:
        path = repo / rel
        if not path.is_file():
            continue
        if SUMMARY.search(path.read_text(encoding="utf-8", errors="replace")):
            return True
    return False


def coverage(quire: str, repo: pathlib.Path, module: str | None) -> dict | None:
    cmd = [quire, "coverage", "--scope", str(repo), "--json"]
    if module:
        cmd += ["--module", module]
    try:
        done = subprocess.run(cmd, capture_output=True, text=True, timeout=600)
    except subprocess.TimeoutExpired:
        return {"error": "timeout"}
    if done.returncode != 0 or not done.stdout.strip():
        return {"error": (done.stderr or "no output").strip().splitlines()[-1][:200]}
    try:
        return json.loads(done.stdout)
    except json.JSONDecodeError as error:
        return {"error": f"unparseable json: {error}"}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("root", nargs="?", default="~/dev")
    parser.add_argument("--quire", required=True, help="path to the quire binary")
    parser.add_argument("--module", help="module directory supplying the traceability model")
    parser.add_argument("--engine", default="", help="engine version string, recorded in output")
    args = parser.parse_args()

    root = pathlib.Path(args.root).expanduser()
    rows = []

    for repo in repos(root):
        report = coverage(args.quire, repo, args.module)
        row = {"repo": repo.name, "mints_test_cases": mints_test_cases(repo)}
        if report is None or "error" in report:
            row["error"] = (report or {}).get("error", "unknown")
        else:
            untracked = report.get("untracked_symbols", [])
            groups = report.get("groups", [])
            # Measured, not inferred: a repo mints test-case ids when the engine
            # actually produced a group for one of the test-case targets. The
            # file-layout heuristic above answers a different question and
            # disagrees under archetype binding, where a matrix outside the three
            # declared paths mints for the first time.
            targets = sorted({g["target"] for g in groups})
            row.update(
                {
                    "dead_tags": len(untracked),
                    "dead_tag_ids": len({u["trace_id"] for u in untracked}),
                    "unbacked_rows": len(report.get("unbacked_rows", [])),
                    "status_lies": len(report.get("status_lies", [])),
                    "groups": len(groups),
                    "targets": targets,
                    "mints_measured": any(t.startswith("test-case") for t in targets),
                    "totals": report.get("totals", {}),
                }
            )
        rows.append(row)
        print(f"  {row['repo']:34s} {row.get('dead_tags', row.get('error'))}", file=sys.stderr)

    print(json.dumps({"engine": args.engine, "module": args.module, "repos": rows}, indent=1))

    ok = [r for r in rows if "error" not in r]
    failed = [r for r in rows if "error" in r]
    zero_mint = [r for r in ok if not r["mints_measured"]]
    zero_layout = [r for r in ok if not r["mints_test_cases"]]
    dead = sum(r["dead_tags"] for r in ok)
    dead_ids = sum(r["dead_tag_ids"] for r in ok)
    with_dead = [r for r in ok if r["dead_tags"]]

    out = sys.stderr
    print(f"\nengine                            : {args.engine or 'unrecorded'}", file=out)
    print(f"module                            : {args.module or 'discovered'}", file=out)
    print(f"repos with a spec/ directory      : {len(rows)}", file=out)
    print(f"  reported cleanly                : {len(ok)}", file=out)
    print(f"  errored                         : {len(failed)}", file=out)
    print(f"repos minting zero test-case ids  : {len(zero_mint)}  (measured)", file=out)
    print(f"  by declared-path layout alone   : {len(zero_layout)}", file=out)
    print(f"dead trace tags (occurrences)     : {dead}", file=out)
    print(f"dead trace tags (distinct ids)    : {dead_ids}", file=out)
    print(f"repos carrying any dead tag       : {len(with_dead)}", file=out)
    for r in sorted(with_dead, key=lambda r: -r["dead_tags"])[:15]:
        print(f"      {r['repo']:30s} {r['dead_tags']:5d}  mints={r['mints_test_cases']}", file=out)
    for r in failed:
        print(f"      ERROR {r['repo']:26s} {r['error']}", file=out)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
