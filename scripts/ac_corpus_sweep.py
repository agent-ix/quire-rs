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

`--properties` runs the FR-052 property-shape classifier instead of the `ac`
grammar, over exactly the same deduplicated corpus. The census it prints — cells
per `PropertyShape`, the `extractable` ratio, and the per-repo breakdown — is the
**primary record** of the property-extractable ratio: CR-022's recorded defect
was quoting a differently-derived distribution table against a check's own count,
so the number in a report must be this census and not a reconstruction of it.
Output JSON: {mode, module_root, docs, records, cells_by_archetype,
cells_by_repo}, one record per classified criterion.

Every `--properties` record carries the criterion's `Verification` cell, joined
on `row_id`, and the census prints **two** recall denominators: every criterion
marked not extractable, and only those whose `Verification` names a test.
A criterion verified by Inspection, Demonstration or Analysis is legitimately
not a property, so counting it as a recall miss understates recall.
`--verified-by-test` restricts the recall sampling frame to the second;
`--verification-vocab N` prints the vocabulary the corpus actually contains, so
the test/non-test call can be judged against the data rather than assumed
(agent-ix/quire-rs#45).

The three `recall-*` cargo features are measured through this same harness: build
one wheel per combination (`maturin build --features python,<combo>`), sweep, and
attribute each classification by its `recall:<rule>:…` signal. They are
default-off and never ship enabled; what ships is decided by that sweep.

`--strip-idioms` materialises a copy of `--module` with its `property_idioms:`
block removed and sweeps against that instead. Two runs — one with, one without —
plus `--compare A.json B.json` give the FR-052-CON-4 check (`extractable` must be
identical in both) and the *fallthrough rate* (of the cells the registry labels
metamorphic, how many degrade to `Universal` without it). The design rests on
that degradation being lossless; this measures it rather than assuming it.
"""

from __future__ import annotations

import argparse
import json
import os
import random
import re
import shutil
import sys
import tempfile
from collections import Counter
from pathlib import Path

import quire  # type: ignore[import-not-found]

BUNDLE = "iso-spec-core"
# The archetypes `ac::check` binds to (`BINDINGS`, src/grammar/ac.rs). CR-020
# narrowed the binding to FR/NFR/StR — US and IT criteria are demonstrations of
# a flow, not claims about the system, and unbinding them was the whole point of
# that CR. This constant kept the pre-CR-020 set, so every archetype-bucketed
# figure taken from this harness since then counted US and IT documents the
# engine never looked at: the walk yielded them, `criteria_cells` counted their
# rows into the denominator, and `check_grammar` returned nothing for them.
AC_ARCHETYPES = {"FR", "NFR", "StR"}

# The FR-052 shapes that assert a metamorphic relation rather than a plain
# quantification. Only these can be *sharpened* by a module idiom, so they are
# the frame the fallthrough measurement is drawn from.
METAMORPHIC_SHAPES = ("round-trip", "idempotence", "ordering", "invariant")

FRONTMATTER_TYPE = re.compile(r"^type:\s*['\"]?([A-Za-z][A-Za-z0-9_-]*)", re.MULTILINE)
AC_SECTION = re.compile(r"^##\s+Acceptance Criteria\s*$", re.MULTILINE)
# StR carries its binding criteria under `## Validation Criteria` (CR-020), so
# the `Verification` join below reads both headings; `criteria_cells` above
# deliberately does not, and that divergence is why the census is the primary
# record and this file's own cell count is not.
CRITERIA_SECTION = re.compile(r"^##\s+(?:Acceptance|Validation)\s+Criteria\s*$", re.MULTILINE)
NEXT_H2 = re.compile(r"^##\s+", re.MULTILINE)

# A `Verification` cell **indicates Test** when it names a test at all. The
# predicate is a word-boundary match on `test`/`tests` rather than membership of
# a fixed set, because the corpus vocabulary is open: 1,478 distinct values over
# 10,798 cells, of which `Test`, `Unit Test` and `Integration Test` are only
# 56%, and the long tail is overwhelmingly `Test (TC-985)`, `Test
# (tests/core/query.test.ts)`, `Schema Test`, `API Test`, `Security Test` — all
# of them tests. `--verification-vocab` prints what the corpus actually
# contains, so this predicate can be re-judged against the data rather than
# trusted.
VERIFICATION_IS_TEST = re.compile(r"\btests?\b", re.IGNORECASE)
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


def is_worktree_sibling(root: Path, name: str) -> bool:
    """True for a `<repo>-task<N>` directory that really is a git worktree.

    The name alone is not proof: a normal repository may legitimately be called
    `something-task42`, and skipping it would silently drop it from every sweep
    with no diagnostic — the same class of invisible under-count this harness
    was already fixed for twice. A linked worktree stores `.git` as a *file*
    containing a `gitdir:` pointer, whereas a normal clone has a `.git`
    directory, so the structure distinguishes them exactly.
    """
    if not WORKTREE_SIBLING.match(name):
        return False
    return (root / name / ".git").is_file()


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


def split_table_row(row: str) -> list[str]:
    """A markdown table row's cells, respecting escaped pipes."""
    parts = re.split(r"(?<!\\)\|", row)
    return [p.strip() for p in parts[1:-1]] if len(parts) >= 3 else []


def verification_by_row_id(text: str) -> dict[str, str]:
    """`{criterion id: Verification cell}` for every binding-criteria table.

    The engine's classification record carries `row_id` but not `Verification`
    — the grammar has no reason to read that column — so the join happens here,
    on the criterion id the record already carries at 100% coverage.

    A criterion whose row has no `Verification` cell simply does not appear; the
    caller reports those as `unknown` rather than assuming either answer, which
    is the same discipline the `extractable` census applies to `Unclassified`.
    """
    out: dict[str, str] = {}
    for m in CRITERIA_SECTION.finditer(text):
        rest = text[m.end():]
        nxt = NEXT_H2.search(rest)
        section = rest[: nxt.start()] if nxt else rest
        rows = [ln.strip() for ln in section.splitlines() if ln.strip().startswith("|")]
        if len(rows) < 2:
            continue
        headers = [h.lower() for h in split_table_row(rows[0])]
        try:
            id_col = headers.index("id")
            v_col = headers.index("verification")
        except ValueError:
            continue
        for row in rows[2:]:  # rows[1] is the |---|---| separator
            cells = split_table_row(row)
            if max(id_col, v_col) < len(cells) and cells[id_col]:
                out[cells[id_col]] = cells[v_col]
    return out


def verification_class(value: str | None) -> str:
    """`test`, `non-test`, or `unknown` for one `Verification` cell."""
    if value is None or not value.strip():
        return "unknown"
    return "test" if VERIFICATION_IS_TEST.search(value) else "non-test"


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
            if is_worktree_sibling(root, rel.parts[0]):
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


def strip_idioms_module(module: Path) -> str:
    """A copy of `module` with its top-level `property_idioms:` block removed.

    The registry-off half of the fallthrough measurement. Copying rather than
    editing in place keeps the real module untouched, and copying the *whole*
    tree (rather than symlinking the manifest's neighbours) keeps every
    `*_schema_ref` path resolvable, which is what `Registry::load_module` walks.

    The strip is textual because the block must vanish exactly as an author
    deleting it would leave it: a YAML round-trip would also reorder and requote
    the rest of the manifest, and then a difference between the two runs could
    not be attributed to the registry alone.
    """
    dest = Path(tempfile.mkdtemp(prefix="ac-sweep-no-idioms-"), module.name)
    shutil.copytree(module, dest)
    manifest = dest / "manifest.yaml"
    text = manifest.read_text(encoding="utf-8")
    # From `property_idioms:` up to the next top-level key (a line starting in
    # column 0 with a non-space, non-`#` character).
    stripped = re.sub(
        r"^property_idioms:.*?(?=^[^\s#])", "", text, flags=re.MULTILINE | re.DOTALL
    )
    if stripped == text:
        print(f"warning: no property_idioms block in {manifest}", file=sys.stderr)
    manifest.write_text(stripped, encoding="utf-8")
    return str(dest)


def sweep_properties(root: Path, module_root: str | None) -> dict:
    """Classify every binding criterion in the corpus by FR-052 property shape.

    One `classify_properties` call per document — the engine's own criteria
    collection, not this file's `criteria_cells` reimplementation, so the census
    denominator is the engine's and cannot drift from it.
    """
    docs = 0
    records: list[dict] = []
    cells_by_archetype: Counter[str] = Counter()
    cells_by_repo: Counter[str] = Counter()

    for repo, path, arch, text in iter_docs(root):
        docs += 1
        try:
            found = quire.classify_properties(BUNDLE, arch, text, module_root)
        except Exception as exc:  # a malformed doc must not abort the sweep
            print(f"skip {path}: {exc}", file=sys.stderr)
            continue
        verification = verification_by_row_id(text)
        for i, r in enumerate(found):
            cells_by_archetype[arch] += 1
            cells_by_repo[repo] += 1
            declared = verification.get(r["row_id"]) if r["row_id"] else None
            records.append({
                "repo": repo, "path": path, "archetype": arch, "index": i,
                "row_id": r["row_id"], "statement": r["statement"],
                "line": r["line"], "shape": r["shape"], "property": r["property"],
                "extractable": r["extractable"], "signals": list(r["signals"]),
                "domain": r["domain"], "precondition": r["precondition"],
                "oracle": r["oracle"],
                # The honest-denominator join (agent-ix/quire-rs#45): a criterion
                # verified by Inspection, Demonstration or Analysis is
                # legitimately not a property, and counting it as a recall miss
                # understates recall.
                "verification": declared,
                "verification_class": verification_class(declared),
            })

    return {
        "mode": "properties",
        "module_root": module_root,
        "docs": docs,
        "cells_by_archetype": dict(cells_by_archetype),
        "cells_by_repo": dict(cells_by_repo),
        "records": records,
    }


def print_property_census(run: dict, top_repos: int) -> None:
    """The census. This *is* the primary record of the extractable ratio."""
    records = run["records"]
    total = len(records)
    print(f"docs {run['docs']}  repos {len(run['cells_by_repo'])}  criteria {total}")
    print(f"module_root {run['module_root']}")
    print()

    print(f"{'property shape':16} {'cells':>8} {'share':>8} {'extractable':>12}")
    by_shape = Counter(r["property"] for r in records)
    for shape, n in by_shape.most_common():
        ex = sum(1 for r in records if r["property"] == shape and r["extractable"])
        share = f"{100 * n / total:.1f}%" if total else "—"
        print(f"{shape:16} {n:8d} {share:>8} {ex:12d}")

    extractable = sum(1 for r in records if r["extractable"])
    spanned = sum(1 for r in records if r["oracle"])
    print()
    print(f"extractable      {extractable:8d} {100 * extractable / total:7.1f}%" if total else "")
    print(f"oracle span      {spanned:8d} {100 * spanned / total:7.1f}%" if total else "")
    print()
    print("criteria by archetype: " + ", ".join(
        f"{k} {v}" for k, v in Counter(run["cells_by_archetype"]).most_common()))

    print()
    print("── Verification, the honest denominator ──")
    by_class = Counter(r["verification_class"] for r in records)
    for cls in ("test", "non-test", "unknown"):
        n = by_class[cls]
        share = f"{100 * n / total:.1f}%" if total else "—"
        print(f"{cls:12} {n:8d} {share:>8}")
    test_pool = [r for r in records if r["verification_class"] == "test"]
    missed_all = [r for r in records if not r["extractable"]]
    missed_test = [r for r in test_pool if not r["extractable"]]
    print()
    print(f"recall denominator, unrestricted : {len(missed_all):8d} criteria marked not extractable")
    print(f"recall denominator, Verification=Test: {len(missed_test):5d} "
          f"({100 * len(missed_test) / len(missed_all):.1f}% of it)"
          if missed_all else "")
    ex_test = sum(1 for r in test_pool if r["extractable"])
    if test_pool:
        print(f"extractable within Verification=Test: {ex_test} of {len(test_pool)} "
              f"({100 * ex_test / len(test_pool):.1f}%)")

    print()
    print(f"{'repo':40} {'criteria':>9} {'extractable':>12} {'rate':>7}")
    per_repo: Counter[str] = Counter(r["repo"] for r in records)
    ex_repo: Counter[str] = Counter(r["repo"] for r in records if r["extractable"])
    for repo, n in per_repo.most_common(top_repos):
        print(f"{repo:40} {n:9d} {ex_repo[repo]:12d} {100 * ex_repo[repo] / n:6.1f}%")


def print_verification_vocab(run: dict, top_n: int) -> None:
    """The `Verification` vocabulary the corpus actually contains.

    Printed rather than assumed: the ISO trio (Inspection / Demonstration /
    Analysis) is only part of the non-test tail, and a fixed set would silently
    misclassify `Storybook`, `Code Review`, `Static Analysis` and `CI Check`.
    Read this table before trusting `VERIFICATION_IS_TEST`.
    """
    values = Counter(
        r["verification"] for r in run["records"] if r["verification"] is not None)
    total = sum(values.values())
    print(f"\n─── Verification vocabulary — {len(values)} distinct values over "
          f"{total} cells ───\n")
    print(f"{'n':>7}  {'class':9}  value")
    for value, n in values.most_common(top_n):
        print(f"{n:7d}  {verification_class(value):9}  {value!r}")
    non_test = Counter({v: n for v, n in values.items()
                        if verification_class(v) == "non-test"})
    print(f"\n─── the non-test tail — {sum(non_test.values())} cells, "
          f"{len(non_test)} distinct ───\n")
    for value, n in non_test.most_common(top_n):
        print(f"{n:7d}  {value!r}")


def sample_properties(run: dict, n: int, seed: int, verified_by_test: bool = False) -> None:
    """Four seeded hand-labelling frames, each drawn from its own stream.

    Separate streams so adding a frame does not reshuffle the others, and
    separate frames because one rate cannot answer for the others: precision over
    the marked-extractable cells, recall over the marked-not, span quality over
    the records a generator would actually consume, and idiom recall over
    `Example` — the cells dropped from extraction entirely, which is where an
    open-set miss becomes silent coverage loss rather than a degraded label.
    """
    records = run["records"]
    # The recall frame is the only one whose denominator is a judgement rather
    # than a fact: a criterion verified by Inspection, Demonstration or Analysis
    # is legitimately not a property, so counting it as a miss understates
    # recall. `--verified-by-test` restricts the frame to criteria the author
    # said would be tested; both denominators stay available, and the census
    # above prints their sizes, so a report can state the pair.
    recall_pool = [r for r in records if not r["extractable"]]
    recall_label = "RECALL — marked NOT extractable"
    if verified_by_test:
        recall_pool = [r for r in recall_pool if r["verification_class"] == "test"]
        recall_label += ", Verification=Test only"
    frames = [
        ("PRECISION — marked extractable",
         [r for r in records if r["extractable"]]),
        (recall_label, recall_pool),
        ("SPAN QUALITY — records carrying an oracle span",
         [r for r in records if r["oracle"]]),
        ("IDIOM RECALL — labelled Example (dropped from extraction)",
         [r for r in records if r["property"] == "example"]),
    ]
    for offset, (label, pool) in enumerate(frames):
        rng = random.Random(seed + offset)
        print(f"\n─── {label} — {min(n, len(pool))} of {len(pool)} ───")
        for r in rng.sample(pool, min(n, len(pool))):
            print(f"\n  [{r['repo']}] {r['row_id']}  {r['property']}"
                  f"  extractable={r['extractable']}"
                  f"  verification={r['verification']!r}")
            print(f"    {r['statement'][:400]}")
            if r["oracle"]:
                print(f"    domain:       {(r['domain'] or {}).get('text')}")
                print(f"    precondition: {(r['precondition'] or {}).get('text')}")
                print(f"    oracle:       {r['oracle']['text']}")
            print(f"    signals: {', '.join(r['signals'])}")


def compare_runs(path_a: Path, path_b: Path) -> int:
    """FR-052-CON-4 and the fallthrough rate, from two `--properties` runs.

    `A` is the registry-on run, `B` the registry-off one. CON-4 says a
    criterion's `extractable` value may not depend on a module-declared idiom, so
    any key differing on `extractable` is a constraint violation, not a
    measurement. Fallthrough is what the "degradation, not loss" assumption
    reduces to: of the cells `A` labels metamorphic, the share `B` still labels
    `Universal` — still extractable, only less specifically shaped.
    """
    a = json.loads(path_a.read_text())
    b = json.loads(path_b.read_text())

    def keyed(run: dict) -> dict[tuple, dict]:
        return {(r["path"], r["index"]): r for r in run["records"]}

    ka, kb = keyed(a), keyed(b)
    shared = sorted(set(ka) & set(kb))
    print(f"A {path_a}  ({len(ka)} criteria, module {a['module_root']})")
    print(f"B {path_b}  ({len(kb)} criteria, module {b['module_root']})")
    print(f"shared keys: {len(shared)}")
    if len(ka) != len(kb) or len(shared) != len(ka):
        print("WARNING: the two runs do not cover the same criteria; "
              "the corpus moved under the measurement", file=sys.stderr)

    differ = [k for k in shared if ka[k]["extractable"] != kb[k]["extractable"]]
    print()
    print(f"FR-052-CON-4 — extractable differs on {len(differ)} of {len(shared)} criteria"
          f"  → {'HOLDS' if not differ else 'VIOLATED'}")
    for k in differ[:10]:
        print(f"  {k}  A={ka[k]['extractable']} B={kb[k]['extractable']}  "
              f"{ka[k]['statement'][:120]}")

    relabelled = [k for k in shared if ka[k]["property"] != kb[k]["property"]]
    print(f"property label differs on {len(relabelled)} of {len(shared)} criteria")
    print()
    print(f"{'A (registry on)':16} {'B (registry off)':16} {'cells':>7}")
    transitions = Counter(
        (ka[k]["property"], kb[k]["property"]) for k in relabelled)
    for (pa, pb), n in transitions.most_common():
        print(f"{pa:16} {pb:16} {n:7d}")

    print()
    print(f"{'metamorphic in A':16} {'cells':>7} {'→ universal in B':>18} {'fallthrough':>12}")
    for shape in METAMORPHIC_SHAPES:
        pool = [k for k in shared if ka[k]["property"] == shape]
        fell = [k for k in pool if kb[k]["property"] == "universal"]
        rate = f"{100 * len(fell) / len(pool):.1f}%" if pool else "—"
        print(f"{shape:16} {len(pool):7d} {len(fell):18d} {rate:>12}")
    meta = [k for k in shared if ka[k]["property"] in METAMORPHIC_SHAPES]
    fell = [k for k in meta if kb[k]["property"] == "universal"]
    rate = f"{100 * len(fell) / len(meta):.1f}%" if meta else "—"
    print(f"{'total':16} {len(meta):7d} {len(fell):18d} {rate:>12}")

    # A metamorphic *label* is not extraction coverage: `extractable` is derived
    # from the structural shape, so a criterion the registry labels
    # `Idempotence` while the structure reads `Example` is shaped but not
    # extractable. CON-4 makes that gap immovable by the registry — which is the
    # point — so it has to be reported on its own rather than read as a loss
    # between the two runs.
    decorative = [k for k in meta if not ka[k]["extractable"]]
    print(f"metamorphic label without extraction coverage (in A): {len(decorative)}"
          f" of {len(meta)}")
    return 0


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
    ap.add_argument("--properties", action="store_true",
                    help="classify criteria by FR-052 property shape instead of "
                         "running the `ac` grammar")
    ap.add_argument("--strip-idioms", action="store_true",
                    help="sweep against a copy of --module with its "
                         "`property_idioms:` block removed (the registry-off run)")
    ap.add_argument("--top-repos", type=int, default=25, metavar="N",
                    help="rows in the per-repo table; the JSON always carries all")
    ap.add_argument("--verified-by-test", action="store_true",
                    help="restrict the RECALL sampling frame to criteria whose "
                         "`Verification` cell names a test; Inspection / "
                         "Demonstration / Analysis criteria are legitimately "
                         "not properties. The census prints both denominators "
                         "either way")
    ap.add_argument("--verification-vocab", type=int, default=0, metavar="N",
                    help="print the N commonest `Verification` values found in "
                         "the corpus, with the test/non-test call for each")
    ap.add_argument("--compare", type=Path, nargs=2, default=None,
                    metavar=("A.json", "B.json"),
                    help="compare a registry-on and a registry-off --properties "
                         "run: FR-052-CON-4 and the fallthrough rate")
    args = ap.parse_args()

    if args.compare:
        return compare_runs(*args.compare)

    root = args.root.expanduser().resolve()

    if args.mine_verbs:
        for stem, n in mine_verbs(root, args.mine_verbs):
            print(f"{n:6d}  {stem}")
        return 0

    module_roots = []
    if args.module:
        module = args.module.expanduser()
        module_roots.append(
            strip_idioms_module(module) if args.strip_idioms else str(module))
    elif args.strip_idioms:
        print("--strip-idioms needs --module", file=sys.stderr)
        return 2
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

    if args.properties:
        run = sweep_properties(root, module_root)
        print_property_census(run, args.top_repos)
        if args.verification_vocab:
            print_verification_vocab(run, args.verification_vocab)
        if args.sample:
            sample_properties(run, args.sample, args.seed, args.verified_by_test)
        if args.out:
            args.out.write_text(json.dumps(run))
            print(f"\nwrote {args.out}")
        return 0

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
