#!/usr/bin/env python3
"""Run the `ac` grammar over a corpus of repos and report per-check fire rates.

This is the harness behind `~/dev/reports/2026-08-04-ac-grammar-fit.md`. That
run's script was never committed, which is why the CR-014 decision could not be
re-derived without rebuilding it (agent-ix/quire-rs#18, #19). It lives here now.

Usage:

    # build the measurement wheel first — `remeasure` re-enables the
    # `no-observable-outcome` check CR-014 retired, so it can be re-measured
    # with the mechanical fixes applied. Omit the feature for a shipping sweep.
    maturin build --release --features python,remeasure --out dist
    pip install --force-reinstall --no-deps --no-index --find-links dist quire

    python3 scripts/ac_corpus_sweep.py \
        --root ~/dev \
        --module ~/dev/spec-artifacts-iso/spec_artifacts_iso \
        --out /tmp/sweep.json

`--extra-verbs FILE` writes a scratch module declaring one `observable_verbs`
entry per line and merges it alongside the real module, so a vocabulary
hypothesis can be tested without an engine change (ADR 0009).

`--mine-verbs N` skips the sweep and instead prints the N most frequent
third-person-singular verb stems in the corpus's AC cells that the engine does
not already know — the derivation of the report's "corpus-mined verbs" list,
which was also never recorded.

Output JSON: {docs, cells_by_archetype, cells_by_repo, cells, findings}. `cells`
is every AC cell seen, so a *recall* sample can be drawn from the cells a check
did **not** flag — the original run sampled precision only.
"""

from __future__ import annotations

import argparse
import json
import os
import random
import re
import sys
import tempfile
from collections import Counter
from pathlib import Path

import quire  # type: ignore[import-not-found]

BUNDLE = "iso-spec-core"
# The archetypes `ac::check` binds to (src/grammar/ac.rs).
AC_ARCHETYPES = {"FR", "NFR", "US", "StR", "IT"}

FRONTMATTER_TYPE = re.compile(r"^type:\s*['\"]?([A-Za-z][A-Za-z0-9_-]*)", re.MULTILINE)
AC_SECTION = re.compile(r"^##\s+Acceptance Criteria\s*$", re.MULTILINE)
NEXT_H2 = re.compile(r"^##\s+", re.MULTILINE)
# Skip vendored / build trees that would otherwise dominate the walk.
SKIP_DIRS = {
    ".git", "node_modules", "target", "dist", "build", ".venv", "venv",
    "__pycache__", ".mypy_cache", ".pytest_cache", ".next", "vendor",
    # Worktree checkouts are byte-copies of a repo already counted. Nested ones
    # matter most: `ecaz` alone carries 20 of them under `.worktrees/` and
    # `.claude/worktrees/`, so leaving them in multiplies that repo's every
    # diagnostic by ~20 and makes one repo look like a corpus-wide trend. This
    # mirrors `spec-artifacts-process/scripts/testmatrix_sweep.py`, which has
    # always deduped; this harness did not, and the gap inflated the CR-013
    # ecaz scope by 19x (1,524 findings walked vs 127 in the real `spec/`).
    "worktrees", ".worktrees",
    # The ticket runner materializes a full checkout at
    # `.ticket-runner/<org>-<repo>-<ticket>/` while a ticket is in flight and
    # removes it afterwards. Left in, a sweep's result depends on whether a
    # ticket happened to be running: one StR count moved 440 -> 453 and back
    # within a single session.
    ".ticket-runner",
}

# Sibling checkouts (`<repo>-task<N>`) are the third worktree shape; they are
# top-level directories, so a dirname skip cannot catch them.
WORKTREE_SIBLING = re.compile(r"^.+-task\d+$")


def frontmatter_type(text: str) -> str | None:
    """The document's `type:` field, read from the frontmatter block only."""
    if not text.startswith("---"):
        return None
    end = text.find("\n---", 3)
    if end == -1:
        return None
    m = FRONTMATTER_TYPE.search(text[3:end])
    return m.group(1) if m else None


def criteria_cells(text: str) -> list[str]:
    """Every non-empty cell of the `Criteria` column under `## Acceptance Criteria`.

    Mirrors `ac::criteria_cells` closely enough for a denominator, but it is a
    reimplementation, not the engine: use it for sampling frames and ratios, and
    quote engine finding counts for anything normative.
    """
    m = AC_SECTION.search(text)
    if not m:
        return []
    rest = text[m.end():]
    nxt = NEXT_H2.search(rest)
    section = rest[: nxt.start()] if nxt else rest

    rows = [ln.strip() for ln in section.splitlines() if ln.strip().startswith("|")]
    if len(rows) < 2:
        return []

    def split_row(row: str) -> list[str]:
        # Respect escaped pipes; drop the leading/trailing empties.
        parts = re.split(r"(?<!\\)\|", row)
        return [p.strip() for p in parts[1:-1]] if len(parts) >= 3 else []

    headers = split_row(rows[0])
    try:
        col = next(i for i, h in enumerate(headers) if h.lower() == "criteria")
    except StopIteration:
        return []

    cells = []
    for row in rows[2:]:  # rows[1] is the |---|---| separator
        parts = split_row(row)
        if col < len(parts) and parts[col]:
            cells.append(parts[col])
    return cells


def iter_docs(root: Path):
    """Yield (repo, relative_path, archetype, text) for every requirement doc."""
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS]
        for fn in filenames:
            if not fn.endswith(".md"):
                continue
            path = Path(dirpath) / fn
            try:
                text = path.read_text(encoding="utf-8")
            except (OSError, UnicodeDecodeError):
                continue
            arch = frontmatter_type(text)
            if arch not in AC_ARCHETYPES:
                continue
            rel = path.relative_to(root)
            if WORKTREE_SIBLING.match(rel.parts[0]):
                continue
            yield rel.parts[0], str(rel), arch, text


def scratch_module(verbs: list[str]) -> str:
    """A throwaway module declaring `observable_verbs`, merged over the built-ins."""
    d = tempfile.mkdtemp(prefix="ac-sweep-verbs-")
    lines = ["name: ac-sweep-verbs", "observable_verbs:"]
    for v in verbs:
        lines.append(f"  {v}:")
        lines.append("    definition: corpus-mined observable-result verb")
    Path(d, "manifest.yaml").write_text("\n".join(lines) + "\n")
    return d


def mine_verbs(root: Path, top_n: int) -> list[tuple[str, int]]:
    """Most frequent `-s` stems in AC cells that the engine does not already match.

    Reproduces the report's vocabulary experiment. A stem counts as unknown when
    the engine reports no observable verb in the bare word, which is exactly the
    membership test the retired check used.
    """
    counts: Counter[str] = Counter()
    for _repo, _path, _arch, text in iter_docs(root):
        for cell in criteria_cells(text):
            for word in re.findall(r"\b([a-z]{3,})s\b", cell.lower()):
                counts[word] += 1

    known = set()
    unknown: list[tuple[str, int]] = []
    for stem, n in counts.most_common():
        if stem in known:
            continue
        # Probe the engine: a lone `<stem>s` in a criteria cell is `unstructured`
        # (→ `unclassifiable`) only when nothing in it carries signal. If the
        # engine already knows the verb, the cell classifies as an assertion.
        doc = (
            "---\nid: FR-001\ntype: FR\n---\n"
            "## Acceptance Criteria\n\n"
            "| ID | Criteria | Verification |\n|---|---|---|\n"
            f"| FR-001-AC-1 | The system {stem}s | Test |\n"
        )
        checks = {f["check"] for f in quire.check_grammar(BUNDLE, "FR", doc)}
        if "unclassifiable" in checks:
            unknown.append((stem, n))
        if len(unknown) >= top_n:
            break
    return unknown


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--root", type=Path, default=Path.home() / "dev")
    ap.add_argument("--module", type=Path, default=None,
                    help="module dir (with manifest.yaml) or a root of module dirs")
    ap.add_argument("--out", type=Path, default=None, help="write full JSON here")
    ap.add_argument("--extra-verbs", type=Path, default=None,
                    help="file of observable verbs, one per line, merged as a module")
    ap.add_argument("--mine-verbs", type=int, default=0, metavar="N",
                    help="print the N most frequent unknown -s stems and exit")
    ap.add_argument("--sample", type=int, default=0, metavar="N",
                    help="print N random findings per check for hand-labelling")
    ap.add_argument("--sample-clean", type=int, default=0, metavar="N",
                    help="print N random cells no check flagged (a recall frame)")
    ap.add_argument("--seed", type=int, default=20260806,
                    help="sampling seed; fixed so a sample is reproducible")
    args = ap.parse_args()

    root = args.root.expanduser().resolve()

    if args.mine_verbs:
        for stem, n in mine_verbs(root, args.mine_verbs):
            print(f"{n:6d}  {stem}")
        return 0

    module_roots = [str(args.module.expanduser())] if args.module else []
    if args.extra_verbs:
        verbs = [ln.strip() for ln in args.extra_verbs.read_text().splitlines() if ln.strip()]
        module_roots.append(scratch_module(verbs))
        print(f"declaring {len(verbs)} extra observable verbs", file=sys.stderr)

    # `check_grammar` takes one module_root. To merge several, point it at a
    # directory whose children are the modules.
    if len(module_roots) > 1:
        merged = tempfile.mkdtemp(prefix="ac-sweep-modules-")
        for i, m in enumerate(module_roots):
            os.symlink(m, Path(merged, f"m{i}"))
        module_root: str | None = merged
    else:
        module_root = module_roots[0] if module_roots else None

    docs = 0
    findings: list[dict] = []
    cells: list[dict] = []
    cells_by_archetype: Counter[str] = Counter()
    cells_by_repo: Counter[str] = Counter()

    for repo, path, arch, text in iter_docs(root):
        docs += 1
        for cell in criteria_cells(text):
            cells.append({"repo": repo, "path": path, "archetype": arch, "statement": cell})
            cells_by_archetype[arch] += 1
            cells_by_repo[repo] += 1
        try:
            found = quire.check_grammar(BUNDLE, arch, text, module_root)
        except Exception as exc:  # a malformed doc must not abort the sweep
            print(f"skip {path}: {exc}", file=sys.stderr)
            continue
        for f in found:
            findings.append({
                "repo": repo, "path": path, "archetype": arch,
                "grammar": f["grammar"], "check": f["check"],
                "pattern": f.get("pattern"), "line": f.get("line"),
                "statement": f.get("statement", ""),
            })

    total_cells = sum(cells_by_archetype.values())
    print(f"docs {docs}  repos {len(cells_by_repo)}  cells {total_cells}")
    print()
    print(f"{'check':34} {'findings':>9} {'rate/cells':>11}")
    by_check = Counter(f"{f['grammar']}:{f['check']}" for f in findings)
    for check, n in by_check.most_common():
        rate = f"{100 * n / total_cells:.1f}%" if total_cells else "—"
        print(f"{check:34} {n:9d} {rate:>11}")
    print(f"{'total':34} {len(findings):9d}")
    print()
    print("cells by archetype: " + ", ".join(
        f"{k} {v}" for k, v in cells_by_archetype.most_common()))

    rng = random.Random(args.seed)
    if args.sample:
        for check in sorted(by_check):
            pool = [f for f in findings if f"{f['grammar']}:{f['check']}" == check]
            print(f"\n─── {check} — {min(args.sample, len(pool))} of {len(pool)} ───")
            for f in rng.sample(pool, min(args.sample, len(pool))):
                print(f"  [{f['repo']}] {f['statement'][:160]}")

    if args.sample_clean:
        flagged = {f["statement"] for f in findings}
        clean = [c for c in cells if c["statement"] not in flagged]
        print(f"\n─── unflagged cells — {min(args.sample_clean, len(clean))} "
              f"of {len(clean)} (recall frame) ───")
        for c in rng.sample(clean, min(args.sample_clean, len(clean))):
            print(f"  [{c['repo']}] {c['statement'][:160]}")

    if args.out:
        args.out.write_text(json.dumps({
            "docs": docs,
            "cells_by_archetype": dict(cells_by_archetype),
            "cells_by_repo": dict(cells_by_repo),
            "cells": cells,
            "findings": findings,
        }))
        print(f"\nwrote {args.out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
