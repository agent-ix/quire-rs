#!/usr/bin/env python3
"""Normalize `/`-separated trace-id chains to the comma the grammar reads.

agent-ix/spec-artifacts-process#37. Every legacy trace form joins an id list on
a **comma** and nothing else. A widespread authoring convention writes a slash:

    /// NFR-002-AC-4, TC-577: the validate-document bench.

Capture group 1 stops at `NFR-002-AC-4` and `TC-577` is dropped **inside the
regex**, before any engine code sees it — so there is no diagnostic, and the
loss is invisible by construction: a dropped id never becomes a relation, so it
can never appear in `untracked_symbols`. It shows only as the row it would have
backed reading unbacked.

The decision, already taken: **do not widen the grammar.** The manifest ruled the
slash out deliberately (`manifest.yaml`, the `*-comment-id` trailing-delimiter
rationale), and a form that accepts every spelling enforces nothing. The corpus
is normalized instead.

## Why this is a classifier and not a `sed`

A blind replacement is wrong in four measured ways, and each has its own rule:

* **Prose.** `# Pull Architecture (FR-010 / FR-011)` binds nothing today and must
  keep binding nothing. Rewriting it would MINT a binding that does not exist.
* **Numeric elision.** `// FR-011-AC-6/7/8` is a recognised shorthand — the
  matrix `Traces To` contract admits `FR-016-AC-1/2/3/6` explicitly. Comma
  replacement yields `FR-011-AC-6,7,8`, which is strictly worse than the input.
* **Ids that mint nothing.** `-CON-`, `-SC-`, `-ATK-`, bare `FR-006`, `US-`,
  `StR-` all match the legacy pattern and are minted by no trace target. Binding
  one adds an `untracked_symbol`, not coverage. This is the trap #193 hit and
  backed out of, demoting six bindings that would have taken dead tags 15 -> 21.
* **Cross-repo references.** `auth/FR-008-CON-9` can never resolve in this scope.

## Usage

    python3 scripts/slash_tag_sweep.py --report reports/<date>-slash-trace-sweep.json
    python3 scripts/slash_tag_sweep.py --repo quire-rs --write

`--dry-run` is the default. Nothing is edited without `--write`.

The report is the primary record and the `.md` is a rendering of it, per the rule
`ac_corpus_sweep.py` states: a number in a report must be the census, not a
reconstruction of it. **Commit this script with its first report.** Three prior
corpus sweeps in this ecosystem shipped without a re-derivable harness — the
2026-08-04 `ac-grammar-fit` run, the `tests-md-sweep` run, and #193's 403-comment
conversion three weeks ago. Do not make it four.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

from corpus import repos, source_files

# ── The grammar, restated exactly ───────────────────────────────────────────
#
# `ID` mirrors the `*-comment-id` alternation in `spec-artifacts-process`'s
# manifest: the prefix set is closed, and a segmented sub-id is optional.
ID = r"(?:TC|IT|FR|NFR|StR|US)-\d+[A-Za-z0-9]*(?:-[A-Za-z]+-\d+)?"

# R1: the line must OPEN with a comment marker followed immediately by an id.
# This is the whole prose defence — an id that appears mid-sentence is not a tag
# today and must not become one.
ANCHOR = re.compile(rf"^\s*(?://+|\#|\*|\"\"\")\s*({ID})")

# R2: the chain that follows, joined by commas and/or slashes. A bare number is
# admitted here ONLY so R3 can see it and refuse the line.
CHAIN = re.compile(rf"(?:{ID})(?:\s*[,/]\s*(?:{ID}|\d+))+")

# R4: which id shapes a trace target actually mints. Everything else matches the
# legacy pattern and binds to nothing.
#
#   test-case                 -> TC-*, IT-*      (Test ID column)
#   acceptance-criterion      -> FR-<n>-AC-<m>
#   nfr-acceptance-criterion  -> NFR-<n>-AC-<m>
#
# `US` and `StR` criteria are deliberately not minted by the module; `-CON-`,
# `-SC-`, `-ATK-`, `-INV-` and bare requirement ids are minted by nothing.
MINTED = re.compile(r"^(?:(?:TC|IT)-\d+[A-Za-z0-9]*|(?:FR|NFR)-\d+-AC-\d+)$")

# ecaz keeps its own trace vocabulary (`ADR-085 D8`, `FR-079/005-P1`, `🟡`
# statuses) and would produce a large, wrong-looking diff. It gets a report and
# an issue, not an edit.
EXCLUDED_REPOS = {"ecaz"}

GREEN, AMBER, ELISION, PROSE = "green", "amber", "elision", "prose"


def classify_line(line: str) -> tuple[str, str | None, list[str]]:
    """Return `(class, chain, offending_ids)` for one line."""
    match = CHAIN.search(line)
    if not match or "/" not in match.group(0):
        return PROSE, None, []

    chain = match.group(0)

    # R1b — a wrapped sentence is not a tag line, even when the continuation
    # happens to begin with an id. `/// Whether the body tier has been
    # materialised (test observability,` / `/// TC-816/TC-817).` anchors
    # correctly and is prose: the `)` closes a parenthetical opened on a
    # PREVIOUS line, which is the tell. Harmless where it was found (a
    # production function cannot bind `verifies` at all, CR-061) but the same
    # shape inside a test's doc block would mint a binding nobody authored.
    after = line[match.end():].lstrip()
    if after.startswith(")") and "(" not in line[: match.start()]:
        return PROSE, match.group(0), []

    anchored = ANCHOR.match(line)
    if not anchored or anchored.start(1) != match.start():
        # R1 — the chain is not what the line opens with, so no form binds it
        # today and rewriting it would MINT a binding that does not exist.
        # Counted, not dropped: a census that hides its own refusals is how a
        # sweep reads as complete while declining most of the population.
        return PROSE, chain, []

    parts = [p.strip() for p in re.split(r"[,/]", chain)]

    if any(p.isdigit() for p in parts):
        # R3 — numeric elision. A different transform with a different risk
        # profile; expanding it mints ids that may not exist. Out of scope.
        return ELISION, chain, []

    unminted = [p for p in parts if not MINTED.match(p)]
    if unminted:
        # R4 — binding these would add untracked symbols, not coverage.
        return AMBER, chain, unminted

    return GREEN, chain, []


def rewrite_line(line: str, chain: str) -> str:
    """R6 — replace slash separators **inside the matched span only**.

    Byte-identical everywhere else: no reflow, no trailing-whitespace fixes, no
    line-length changes. A sweep whose diff touches anything it did not classify
    cannot be reviewed against its own report.
    """
    normalized = re.sub(r"\s*/\s*", ", ", chain)
    return line.replace(chain, normalized, 1)


def sweep_repo(repo: Path, write: bool) -> dict:
    counts = {GREEN: 0, AMBER: 0, ELISION: 0, PROSE: 0}
    findings: list[dict] = []
    edited_files = 0

    for path in source_files(repo):
        try:
            text = path.read_text(encoding="utf-8")
        except (UnicodeDecodeError, OSError):
            continue
        if "/" not in text:
            continue

        lines = text.splitlines(keepends=True)
        changed = False
        for i, line in enumerate(lines):
            if "/" not in line:
                continue
            verdict, chain, unminted = classify_line(line)
            if chain is None:
                continue  # no slash-joined chain on this line at all
            counts[verdict] += 1
            record = {
                "path": str(path.relative_to(repo)),
                "line": i + 1,
                "class": verdict,
                "chain": chain,
            }
            if unminted:
                record["mints_nothing"] = unminted
            if verdict == GREEN:
                record["after"] = re.sub(r"\s*/\s*", ", ", chain)
                if write:
                    lines[i] = rewrite_line(line, chain)
                    changed = True
            findings.append(record)

        if changed:
            path.write_text("".join(lines), encoding="utf-8")
            edited_files += 1

    return {
        "repo": repo.name,
        "counts": counts,
        "files_edited": edited_files,
        "findings": findings,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", default="~/dev")
    parser.add_argument("--repo", action="append", help="limit to these repos")
    parser.add_argument("--write", action="store_true", help="apply GREEN edits")
    parser.add_argument("--report", help="write the JSON census here")
    parser.add_argument(
        "--include-excluded",
        action="store_true",
        help=f"do not skip {sorted(EXCLUDED_REPOS)}",
    )
    args = parser.parse_args()

    root = Path(args.root).expanduser()
    selected = repos(root)
    if args.repo:
        wanted = set(args.repo)
        selected = [r for r in selected if r.name in wanted]
    if not args.include_excluded:
        selected = [r for r in selected if r.name not in EXCLUDED_REPOS]

    results = [sweep_repo(r, args.write) for r in selected]
    results = [r for r in results if any(r["counts"].values())]

    totals = {k: sum(r["counts"][k] for r in results) for k in (GREEN, AMBER, ELISION, PROSE)}
    census = {
        "root": str(root),
        "repos_scanned": len(selected),
        "write": args.write,
        "excluded_repos": sorted(() if args.include_excluded else EXCLUDED_REPOS),
        "totals": totals,
        "repos": results,
    }

    if args.report:
        Path(args.report).write_text(json.dumps(census, indent=1), encoding="utf-8")

    out = sys.stderr
    print(f"repos scanned                     : {len(selected)}", file=out)
    print(f"GREEN   (auto-editable)           : {totals[GREEN]}", file=out)
    print(f"AMBER   (an id mints nothing)     : {totals[AMBER]}", file=out)
    print(f"ELISION (numeric shorthand)       : {totals[ELISION]}", file=out)
    print(f"PROSE   (not a tag line)          : {totals[PROSE]}", file=out)
    print(f"mode                              : {'WRITE' if args.write else 'dry-run'}", file=out)
    for r in sorted(results, key=lambda r: -r["counts"][GREEN]):
        if r["counts"][GREEN]:
            print(f"    {r['repo']:32s} green={r['counts'][GREEN]:4d} "
                  f"amber={r['counts'][AMBER]:4d}", file=out)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
