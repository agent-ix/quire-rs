#!/usr/bin/env python3
"""Re-derive the dead-trace-tag population across the ecosystem.

quire-rs#78: the numbers in #72 ("153 of 237 repos mint zero test-case targets,
1,014 written trace tags bind to nothing") were measured through a corpus walk
that could not see any file named `tests.md`. Type-driven membership landed in
v0.26.0; this re-measures against a released engine.

Two things the earlier sweep got wrong, both fixed here:

* **The engine must be the pinned one, and it must say so itself.** CR-061
  (v0.27.0) widened `trace::bind` to benchmarks and fuzz targets, so a tag on a
  `criterion_group!` bench resolves where it previously did not. Numbers taken
  on an older engine are stale on arrival.
* **The module must not be the stale installed copy.** `~/.ix/filament/modules`
  lags the source repository — at the time of writing it was missing the Phase D
  comma-list trace patterns (205 ids across 17 repos) and the `tests/**`
  exclusions on the FR/NFR archetype targets. Pass `--module` pointing at the
  source tree so the model is the current one, and record which.

## Provenance is measured, never typed (#265)

This script used to take `--engine`, a **string the operator typed**, and record
it in the output. Unverified, so it was not provenance — it made a report *look*
sourced, which is worse than a report that admits it is not. It also took
`--quire <bin>`, defaulting to whatever was on `PATH`; the installed binary was
16 releases behind this tree at the time of writing, and `Makefile:246` had
already stated the correct rule for `make validate` two years running.

Both are gone. The binary is **built from the consuming workspace at its pinned
rev**, and the engine version is read from the `engine` block of the payloads it
emits (quire-cli#68). A binary that cannot say what it links cannot be swept
with, and a binary missing a capability the sweep depends on **aborts naming the
token** rather than omitting a metric and printing the rest.

Dead tags are `untracked_symbols`: a trace marker written in source whose id no
trace target ever minted. A repo "mints zero test-case targets" when none of the
three declared matrix paths carries a `## Test Case Summary` — the section the
`test-case` target reads.

Usage:
  python3 sweep_coverage.py --consumer ../quire-cli --module <dir> [root]
"""

from __future__ import annotations

import argparse
import json
import pathlib
import re
import subprocess
import sys

# Dedupe rules live in `corpus.py` — a worktree or a `-task<N>` scratch copy is
# the same repo counted twice. The 2026-08-14 sweep reported 184 matrices where
# the truth was 183 for exactly this reason. This script previously carried its
# own copy of the rules which had neither `SKIP_DIRS` nor verified worktree
# detection, so its numbers excluded less than the other harnesses' did.
from check_engine import Drift, assert_capabilities, build_engine, reported_engine
from corpus import repos

DECLARED_PATHS = ("spec/tests.md", "spec/matrix.md", "spec/evals.md")
SUMMARY = re.compile(r"^##\s+Test Case Summary\s*$", re.MULTILINE)

# What this sweep's own numbers rest on. `binding_census` is the premise behind
# every backed/total figure it prints — without it the sweep cannot say whether
# the binder read a single test in a repo, and a coverage percentage over an
# unread corpus is the exact shape that produced four passes of wrong answers.
REQUIRED_CAPABILITIES = ("binding_census",)


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
    parser.add_argument(
        "--consumer",
        default="../quire-cli",
        help="workspace to build the engine from, at its pinned rev (default: ../quire-cli)",
    )
    parser.add_argument("--module", help="module directory supplying the traceability model")
    args = parser.parse_args()

    here = pathlib.Path(__file__).resolve().parent.parent
    consumer = pathlib.Path(args.consumer).expanduser()
    if not consumer.is_absolute():
        consumer = (here.parent / consumer.name).resolve()

    try:
        quire = build_engine(consumer, release=True)
    except Drift as error:
        print(f"sweep_coverage: {error}", file=sys.stderr)
        return 1

    root = pathlib.Path(args.root).expanduser()
    rows = []
    engine: str | None = None

    for repo in repos(root):
        report = coverage(quire, repo, args.module)
        row = {"repo": repo.name, "mints_test_cases": mints_test_cases(repo)}
        if report is None or "error" in report:
            row["error"] = (report or {}).get("error", "unknown")
        else:
            # Provenance off the FIRST clean payload, then held. Checked on
            # every subsequent one: a sweep is a long-running loop over hundreds
            # of repositories, and a binary swapped underneath it halfway
            # through would otherwise be reported as one measurement.
            try:
                version, capabilities = reported_engine(report)
                if engine is None:
                    assert_capabilities(capabilities, list(REQUIRED_CAPABILITIES))
                    engine = version
                    print(f"engine: {version} ({', '.join(capabilities)})", file=sys.stderr)
                elif version != engine:
                    raise Drift(
                        f"{repo.name} was measured by engine {version} while the "
                        f"sweep began on {engine}. The binary changed mid-run, so "
                        f"these rows are not one measurement."
                    )
            except Drift as error:
                print(f"sweep_coverage: {error}", file=sys.stderr)
                return 1
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

    # No clean payload means nothing verified the instrument, and every figure
    # below would be a zero over an unmeasured corpus — the silent-zero the
    # capability abort exists to prevent, arrived at by a different route. The
    # first version printed `"engine": null` and exited 0.
    if engine is None:
        print(
            f"sweep_coverage: not one of {len(rows)} repositories produced a "
            f"readable payload, so the engine was never identified and nothing "
            f"here was measured. Refusing to print a zero over an unmeasured "
            f"corpus.",
            file=sys.stderr,
        )
        return 1

    print(json.dumps(
        {"engine": engine, "module": args.module, "consumer": str(consumer), "repos": rows},
        indent=1,
    ))

    ok = [r for r in rows if "error" not in r]
    failed = [r for r in rows if "error" in r]
    zero_mint = [r for r in ok if not r["mints_measured"]]
    zero_layout = [r for r in ok if not r["mints_test_cases"]]
    dead = sum(r["dead_tags"] for r in ok)
    dead_ids = sum(r["dead_tag_ids"] for r in ok)
    with_dead = [r for r in ok if r["dead_tags"]]

    out = sys.stderr
    print(f"\nengine (from the payload)         : {engine or 'no clean payload'}", file=out)
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
