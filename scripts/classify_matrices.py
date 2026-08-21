#!/usr/bin/env python3
"""Classify every Test Matrix in the ecosystem by how the corpus reaches it.

quire-rs#75: before deleting the `document:` path-binding form, measure how many
real Test Matrices would become *invisible* under type-driven corpus membership.

`document:` binds a scope-relative path and `declared_tables::harvest` does a raw
`fs::read_to_string`, bypassing the walk — so it reaches a file with no
frontmatter at all. Type-driven membership requires frontmatter naming a
registered archetype. The delta between those two reaches is the gate.

Population A (reachable today, path-bound): `<repo>/spec/{tests,matrix,evals}.md`
  — the three filenames spec-artifacts-process declares.
Population B (reachable after, archetype-bound): any `.md` in the repo whose
  frontmatter `type:` is `TestMatrix`.

  LOSS = A \\ B  — matrices that go silently invisible. This is the gate.
  GAIN = B \\ A  — matrices the walk cannot see today (nested module matrices).

Usage:  python3 classify_matrices.py [root]     (default: ~/dev)
"""

from __future__ import annotations

import json
import pathlib
import re
import sys

# A worktree or a `-task<N>` scratch copy is the same repo counted twice. The
# 2026-08-14 sweep reported 184 matrices where the truth was 183 for exactly
# this reason. NB `.claude/worktrees/` as well as `.worktrees/` — ecaz uses the
# former, and missing it added 5 phantom matrices to the first run of this
# sweep. The rules live in `corpus.py`; this script used to carry a partial copy
# that omitted `.ticket-runner` and matched `-task<N>` by name alone.
from corpus import markdown_files, repos

DECLARED_PATHS = ("spec/tests.md", "spec/matrix.md", "spec/evals.md")
REGISTERED = "TestMatrix"

FRONTMATTER = re.compile(r"\A---\r?\n(.*?)\r?\n---\r?\n", re.DOTALL)
TYPE_LINE = re.compile(r"^type:\s*(.*?)\s*$", re.MULTILINE)
# A matrix with no `## Test Case Summary` mints zero `test-case` targets however
# it is bound, so losing it costs nothing. This separates notional from real loss.
SUMMARY = re.compile(r"^##\s+Test Case Summary\s*$", re.MULTILINE)


def classify(path: pathlib.Path) -> dict:
    """Three nested questions, in the order that decides visibility."""
    try:
        text = path.read_text(encoding="utf-8", errors="replace")
    except OSError as error:
        return {"readable": False, "error": str(error)}

    mints = bool(SUMMARY.search(text))
    block = FRONTMATTER.match(text)
    if not block:
        return {
            "readable": True,
            "frontmatter": False,
            "type": None,
            "registered": False,
            "mints_rows": mints,
        }

    found = TYPE_LINE.search(block.group(1))
    declared = found.group(1).strip().strip("\"'") if found else ""
    return {
        "readable": True,
        "frontmatter": True,
        "type": declared or None,
        "registered": declared == REGISTERED,
        "mints_rows": mints,
    }


def is_test_data(path: str) -> bool:
    """The same rule a module declares with `exclude: ['tests/**']`. A fixture
    matrix is not a coverage gain; spec-artifacts-process ships 30 deliberately
    malformed ones."""
    return path.startswith("tests/") or "/tests/" in path or "/fixtures/" in path


def walk_markdown(repo: pathlib.Path):
    yield from markdown_files(repo)


def main() -> int:
    root = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else "~/dev").expanduser()
    rows = []

    for repo in repos(root):
        typed = {}
        for path in walk_markdown(repo):
            info = classify(path)
            if info.get("registered"):
                typed[path.relative_to(repo).as_posix()] = info

        declared = {}
        for rel in DECLARED_PATHS:
            path = repo / rel
            if path.is_file():
                declared[rel] = classify(path)

        for rel, info in declared.items():
            rows.append(
                {
                    "repo": repo.name,
                    "path": rel,
                    "reach": "both" if rel in typed else "path-only",
                    **info,
                }
            )
        for rel in sorted(set(typed) - set(declared)):
            rows.append(
                {
                    "repo": repo.name,
                    "path": rel,
                    "reach": "archetype-only",
                    **typed[rel],
                }
            )

    print(json.dumps(rows, indent=1))

    def count(pred) -> int:
        return sum(1 for r in rows if pred(r))

    path_bound = [r for r in rows if r["reach"] in ("both", "path-only")]
    print(f"\nrepos with a spec/ directory      : {len(repos(root))}", file=sys.stderr)
    print(f"matrices at a declared path (A)   : {len(path_bound)}", file=sys.stderr)
    print(f"  no frontmatter at all  [LOSS]   : "
          f"{count(lambda r: r['reach'] == 'path-only' and not r.get('frontmatter'))}",
          file=sys.stderr)
    print(f"  frontmatter, no type   [LOSS]   : "
          f"{count(lambda r: r['reach'] == 'path-only' and r.get('frontmatter') and not r.get('type'))}",
          file=sys.stderr)
    print(f"  type, not TestMatrix   [LOSS]   : "
          f"{count(lambda r: r['reach'] == 'path-only' and r.get('type') and not r['registered'])}",
          file=sys.stderr)
    print(f"  typed TestMatrix       [kept]   : {count(lambda r: r['reach'] == 'both')}",
          file=sys.stderr)
    lost = [r for r in rows if r["reach"] == "path-only"]
    costly = [r for r in lost if r.get("mints_rows")]
    print(f"  ...of the {len(lost)} lost, minting rows today : {len(costly)}  [REAL LOSS]",
          file=sys.stderr)
    for r in sorted(costly, key=lambda r: r["repo"]):
        print(f"      {r['repo']:28s} {r['path']:15s} type={r['type']!r}", file=sys.stderr)

    gain = [r for r in rows if r["reach"] == "archetype-only"]
    real_gain = [r for r in gain if not is_test_data(r["path"])]
    print(f"matrices only archetype reaches   : {len(gain)}  [GAIN]", file=sys.stderr)
    print(f"  ...outside any test tree        : {len(real_gain)} across "
          f"{len({r['repo'] for r in real_gain})} repos", file=sys.stderr)
    # The same notional/real split the loss gets: a gained matrix with no
    # `## Test Case Summary` mints nothing, so it is not a coverage gain yet —
    # it is the matrix-layout defect surface (quoin#63) wearing a new reach.
    costly_gain = [r for r in real_gain if r.get("mints_rows")]
    print(f"  ...of those, minting rows today : {len(costly_gain)} across "
          f"{len({r['repo'] for r in costly_gain})} repos  [REAL GAIN]",
          file=sys.stderr)
    for r in sorted(costly_gain, key=lambda r: (r["repo"], r["path"])):
        print(f"      {r['repo']:28s} {r['path']}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
