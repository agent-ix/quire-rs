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

And one measured way a *correct-looking* replacement is still wrong (#217):

* **A rewrite that cannot bind.** `// TC-473 / FR-024-AC-4 + NFR-006:` rewrites
  to a clean comma list whose ` + NFR-006` tail still stops the grammar at the
  first id — three of #208's edits were placebos measured exactly so. GREEN
  judges the **rewritten** line, not the slash-span (rule R7).

## The census unit

One record per **slash-joined chain occurrence**, not per line. A line may hold
several chains — `// TC-001, TC-002 — see FR-003/FR-004` holds a comma chain
and then a slash chain — and a census that sees only the first match silently
under-counts (#217). Files that cannot be read are **counted refusals**, never
silent skips: a census that hides its own refusals reads as complete while
declining part of the population.

## Usage

    python3 scripts/slash_tag_sweep.py --report reports/<date>-slash-trace-sweep.json
    python3 scripts/slash_tag_sweep.py --repo quire-rs --write

`--dry-run` is the default. Nothing is edited without `--write`, and `--write`
refuses a repository whose git worktree is dirty — an in-place corpus edit must
be revertable by `git checkout`, so mixing it into uncommitted work is refused
per repo and counted. Override deliberately with `--allow-dirty`.

With `--write`, the census carries **both** `totals` (measured before any edit)
and `totals_after_write` (re-measured after), so every headline number is
re-derivable from the one committed artifact (#217; the #208 report's
before-numbers were re-derivable from nothing).

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
import subprocess
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

# R2: a chain of ids joined by commas and/or slashes. A bare number is admitted
# here ONLY so R3 can see it and refuse the chain.
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

# R7: what follows the chain decides whether the REWRITTEN line can bind. A
# ` + <id>` tail stops the comma grammar at the chain's first id, so a slash
# rewrite in front of one is a placebo — the three #208 shipped were measured
# unbacked (`FR-024-AC-4`, `FR-025-AC-4`, `FR-027-AC-6`).
UNBINDABLE_TAIL = re.compile(rf"\s*\+\s*({ID})")

# ecaz keeps its own trace vocabulary (`ADR-085 D8`, `FR-079/005-P1`, `🟡`
# statuses) and would produce a large, wrong-looking diff. It gets a report and
# an issue, not an edit.
EXCLUDED_REPOS = {"ecaz"}

GREEN, AMBER, ELISION, PROSE = "green", "amber", "elision", "prose"
CLASSES = (GREEN, AMBER, ELISION, PROSE)


def classify_line(line: str) -> list[dict]:
    """Every slash-joined chain on `line`, each classified separately.

    Returns one record per chain: `{"class", "chain", "start", "end"}` plus
    `"mints_nothing"` (AMBER, R4) or `"unbindable_tail"` (AMBER, R7) where they
    apply. Iterates ALL chains (#217): a line whose first chain is comma-joined
    and whose second is slash-joined is found and counted, not skipped.
    """
    records: list[dict] = []
    anchored = ANCHOR.match(line)
    for match in CHAIN.finditer(line):
        chain = match.group(0)
        if "/" not in chain:
            continue
        record = {
            "chain": chain,
            "start": match.start(),
            "end": match.end(),
        }
        record["class"] = _classify_chain(line, match, anchored)
        if record["class"] == AMBER:
            parts = [p.strip() for p in re.split(r"[,/]", chain)]
            unminted = [p for p in parts if not MINTED.match(p)]
            if unminted:
                record["mints_nothing"] = unminted
            else:
                tail = UNBINDABLE_TAIL.match(line[match.end() :])
                record["unbindable_tail"] = tail.group(1) if tail else None
        records.append(record)
    return records


def _classify_chain(line: str, match: re.Match, anchored: re.Match | None) -> str:
    """One chain's verdict, rules R1..R4 and R7 in order."""
    # R1b — a wrapped sentence is not a tag line, even when the continuation
    # happens to begin with an id. `/// Whether the body tier has been
    # materialised (test observability,` / `/// TC-816/TC-817).` anchors
    # correctly and is prose: the `)` closes a parenthetical opened on a
    # PREVIOUS line, which is the tell. Harmless where it was found (a
    # production function cannot bind `verifies` at all, CR-061) but the same
    # shape inside a test's doc block would mint a binding nobody authored.
    after = line[match.end() :].lstrip()
    if after.startswith(")") and "(" not in line[: match.start()]:
        return PROSE

    if anchored is None or anchored.start(1) != match.start():
        # R1 — the chain is not what the line opens with, so no form binds it
        # today and rewriting it would MINT a binding that does not exist.
        # Counted, not dropped.
        return PROSE

    parts = [p.strip() for p in re.split(r"[,/]", match.group(0))]

    if any(p.isdigit() for p in parts):
        # R3 — numeric elision. A different transform with a different risk
        # profile; expanding it mints ids that may not exist. Out of scope.
        return ELISION

    if any(not MINTED.match(p) for p in parts):
        # R4 — binding these would add untracked symbols, not coverage.
        return AMBER

    if UNBINDABLE_TAIL.match(line[match.end() :]):
        # R7 — GREEN is a claim about the REWRITTEN line. A ` + <id>` tail
        # stops the comma grammar at the first id, so the rewrite would be a
        # placebo: byte-diff without a binding. Refused like any other chain
        # that needs a per-line decision.
        return AMBER

    return GREEN


def rewrite_line(line: str, start: int, end: int) -> str:
    """R6 — replace slash separators **inside the matched span only**.

    A span replace, not `str.replace(chain, …, 1)` (#217): the first occurrence
    of the chain's text is not necessarily the classified occurrence. Byte
    identical everywhere else: no reflow, no trailing-whitespace fixes, no
    line-length changes. A sweep whose diff touches anything it did not
    classify cannot be reviewed against its own report.
    """
    normalized = re.sub(r"\s*/\s*", ", ", line[start:end])
    return line[:start] + normalized + line[end:]


def sweep_repo(repo: Path, write: bool) -> dict:
    counts = {klass: 0 for klass in CLASSES}
    findings: list[dict] = []
    refusals: list[dict] = []
    edited_files = 0

    for path in source_files(repo):
        try:
            text = path.read_text(encoding="utf-8")
        except (UnicodeDecodeError, OSError) as exc:
            # R5 — an unreadable file is a counted refusal, never a silent
            # skip. The docstring's census rule: a census that hides its own
            # refusals reads as complete while declining part of the
            # population (#217).
            refusals.append(
                {
                    "path": str(path.relative_to(repo)),
                    "reason": f"{type(exc).__name__}: {exc}",
                }
            )
            continue
        if "/" not in text:
            continue

        lines = text.splitlines(keepends=True)
        changed = False
        for i, line in enumerate(lines):
            if "/" not in line:
                continue
            records = classify_line(line)
            for record in records:
                counts[record["class"]] += 1
                entry = {
                    "path": str(path.relative_to(repo)),
                    "line": i + 1,
                    "class": record["class"],
                    "chain": record["chain"],
                }
                for key in ("mints_nothing", "unbindable_tail"):
                    if record.get(key):
                        entry[key] = record[key]
                if record["class"] == GREEN:
                    entry["after"] = re.sub(r"\s*/\s*", ", ", record["chain"])
                findings.append(entry)
            if write:
                # Rewrites apply right-to-left so earlier spans stay valid.
                greens = [r for r in records if r["class"] == GREEN]
                for record in sorted(greens, key=lambda r: -r["start"]):
                    lines[i] = rewrite_line(lines[i], record["start"], record["end"])
                    changed = True

        if changed:
            path.write_text("".join(lines), encoding="utf-8")
            edited_files += 1

    return {
        "repo": repo.name,
        "counts": counts,
        "files_edited": edited_files,
        "unreadable_files": refusals,
        "findings": findings,
    }


def dirty_reason(repo: Path) -> str | None:
    """Why `--write` refuses this repo, or `None` when its worktree is clean."""
    try:
        proc = subprocess.run(
            ["git", "-C", str(repo), "status", "--porcelain"],
            capture_output=True,
            text=True,
            check=True,
        )
    except (OSError, subprocess.CalledProcessError):
        return "not a git repository (or `git status` failed)"
    if proc.stdout.strip():
        return "dirty git worktree"
    return None


def aggregate(results: list[dict]) -> dict:
    return {klass: sum(r["counts"][klass] for r in results) for klass in CLASSES}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", default="~/dev")
    parser.add_argument("--repo", action="append", help="limit to these repos")
    parser.add_argument("--write", action="store_true", help="apply GREEN edits")
    parser.add_argument("--report", help="write the JSON census here")
    parser.add_argument(
        "--allow-dirty",
        action="store_true",
        help="R8 override: edit a repo even when its git worktree is dirty",
    )
    parser.add_argument(
        "--include-excluded",
        action="store_true",
        help=f"do not skip {sorted(EXCLUDED_REPOS)}",
    )
    args = parser.parse_args()

    root = Path(args.root).expanduser()
    # Two corpus numbers, named so they cannot be conflated (#217/#219): the
    # #208 report published `repos_scanned` labelled as "enumerated by
    # scripts/corpus.py", and the two differ by every exclusion below.
    enumerated = repos(root)
    selected = enumerated
    if args.repo:
        wanted = set(args.repo)
        selected = [r for r in selected if r.name in wanted]
    if not args.include_excluded:
        selected = [r for r in selected if r.name not in EXCLUDED_REPOS]

    results = []
    write_refusals: list[dict] = []
    for repo in selected:
        write_here = args.write
        if args.write and not args.allow_dirty:
            # R8 — an in-place edit must be revertable by `git checkout`, so a
            # dirty worktree is refused (per repo, counted) rather than mixed
            # into uncommitted work. `--allow-dirty` overrides deliberately.
            reason = dirty_reason(repo)
            if reason is not None:
                write_here = False
                write_refusals.append({"repo": repo.name, "reason": reason})
                print(
                    f"REFUSED --write for {repo.name}: {reason} "
                    f"(census taken; pass --allow-dirty to edit anyway)",
                    file=sys.stderr,
                )
        results.append(sweep_repo(repo, write_here))

    kept = [r for r in results if any(r["counts"].values()) or r["unreadable_files"]]

    totals = aggregate(results)
    unreadable_total = sum(len(r["unreadable_files"]) for r in results)
    census = {
        "root": str(root),
        "repos_enumerated": len(enumerated),
        "repos_scanned_after_exclusions": len(selected),
        "write": args.write,
        "excluded_repos": sorted(() if args.include_excluded else EXCLUDED_REPOS),
        "unreadable_files": unreadable_total,
        "totals": totals,
        "repos": kept,
    }
    if write_refusals:
        census["write_refusals"] = write_refusals

    if args.write:
        # Census discipline (#217): `totals` above is the BEFORE census —
        # classification always reads the pre-edit line. Re-measure after the
        # edits so both headline numbers live in the one committed artifact.
        after = [sweep_repo(r, write=False) for r in selected]
        census["totals_after_write"] = aggregate(after)

    if args.report:
        Path(args.report).write_text(json.dumps(census, indent=1), encoding="utf-8")

    out = sys.stderr
    print(f"repos enumerated                  : {len(enumerated)}", file=out)
    print(f"repos scanned (after exclusions)  : {len(selected)}", file=out)
    print(f"unreadable files (refused, counted): {unreadable_total}", file=out)
    print(f"GREEN   (auto-editable)           : {totals[GREEN]}", file=out)
    print(f"AMBER   (refused per R4/R7)       : {totals[AMBER]}", file=out)
    print(f"ELISION (numeric shorthand)       : {totals[ELISION]}", file=out)
    print(f"PROSE   (not a tag line)          : {totals[PROSE]}", file=out)
    print(
        f"mode                              : {'WRITE' if args.write else 'dry-run'}",
        file=out,
    )
    if args.write:
        after_totals = census["totals_after_write"]
        print(
            f"GREEN after write                 : {after_totals[GREEN]}",
            file=out,
        )
    for r in sorted(kept, key=lambda r: -r["counts"][GREEN]):
        if r["counts"][GREEN]:
            print(
                f"    {r['repo']:32s} green={r['counts'][GREEN]:4d} "
                f"amber={r['counts'][AMBER]:4d}",
                file=out,
            )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
