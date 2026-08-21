#!/usr/bin/env python3
"""One definition of what counts as a repository in the `~/dev` corpus.

Every cross-repo sweep needs the same three answers — which top-level
directories are repositories, which nested directories are byte-copies of a
repository already counted, and which repositories are superseded. Before this
module there were **four** answers, and they disagreed:

| script | `SKIP_DIRS` | `-task<N>` siblings | `SUPERSEDED` |
| --- | --- | --- | --- |
| `sweep_coverage.py` | absent | name regex | yes |
| `classify_matrices.py` | partial (no `.ticket-runner`) | name regex | yes |
| `ac_corpus_sweep.py` | full | **verified** (`.git` is a file) | absent |
| `spec-artifacts-process/scripts/testmatrix_sweep.py` | full | absent | absent |

Each gap has already cost a published number. `classify_matrices.py`'s header
records a sweep reporting 184 matrices where the truth was 183;
`ac_corpus_sweep.py`'s records the CR-013 ecaz scope inflated 19x (1,524
findings walked against 127 in the real `spec/`) and an StR count that moved
440 -> 453 and back inside one session because a ticket happened to be running.

This module is the union, and it takes the *strictest correct* rule from each
source rather than the most common one — see `is_worktree_sibling`.

Import it; do not copy it. A fifth divergent copy is how the next wrong number
gets published.
"""

from __future__ import annotations

import re
from pathlib import Path
from typing import Iterator

# Vendored, generated and build trees that would otherwise dominate a walk, plus
# the three shapes a duplicate checkout takes.
SKIP_DIRS = {
    ".git",
    "node_modules",
    "target",
    "dist",
    "build",
    ".venv",
    "venv",
    "__pycache__",
    ".mypy_cache",
    ".pytest_cache",
    ".next",
    "vendor",
    # Worktree checkouts are byte-copies of a repo already counted. Nested ones
    # matter most: `ecaz` alone carries 20 of them under `.worktrees/` and
    # `.claude/worktrees/`, so leaving them in multiplies that repo's every
    # diagnostic by ~20 and makes one repo look like a corpus-wide trend.
    "worktrees",
    ".worktrees",
    # The ticket runner materializes a full checkout at
    # `.ticket-runner/<org>-<repo>-<ticket>/` while a ticket is in flight and
    # removes it afterwards. Left in, a sweep's result depends on whether a
    # ticket happened to be running.
    ".ticket-runner",
}

# Superseded repos stay on disk but are excluded from every sweep; counting
# `filament-ide` alongside `filament-ide-rs` is the same repo counted twice.
SUPERSEDED = {"filament-ide"}

# Sibling checkouts (`<repo>-task<N>`) are the third worktree shape. They are
# top-level directories, so a `SKIP_DIRS` dirname skip cannot catch them.
WORKTREE_SIBLING = re.compile(r"^.+-task\d+$")

# Source suffixes the engine's `language_of` recognises (quire-rs
# `src/symbols/mod.rs`). A file outside this set mints no symbol, so a trace tag
# written in one binds to nothing however well-formed it is.
SOURCE_SUFFIXES = {".rs", ".py", ".ts", ".tsx"}


def is_worktree_sibling(root: Path, name: str) -> bool:
    """True for a `<repo>-task<N>` directory that really is a git worktree.

    The name alone is not proof: a normal repository may legitimately be called
    `something-task42`, and skipping it would silently drop it from every sweep
    with no diagnostic — the same class of invisible under-count these harnesses
    have been fixed for twice. A linked worktree stores `.git` as a *file*
    containing a `gitdir:` pointer, whereas a normal clone has a `.git`
    directory, so the structure distinguishes them exactly.

    This is the rule `ac_corpus_sweep.py` reached and the other three did not;
    it is strictly better than the `-task\\d+$` name match they used, so it wins.
    """
    if not WORKTREE_SIBLING.match(name):
        return False
    return (root / name / ".git").is_file()


def repos(root: Path) -> list[Path]:
    """Every repository in the corpus, sorted, deduped.

    A repository is a non-hidden top-level directory carrying a `spec/`
    directory. `spec/` is the membership test because a repo with no spec has
    nothing for any of these sweeps to read.
    """
    out: list[Path] = []
    for child in sorted(root.iterdir()):
        if not child.is_dir() or child.name.startswith("."):
            continue
        if child.name in SKIP_DIRS or child.name in SUPERSEDED:
            continue
        if is_worktree_sibling(root, child.name):
            continue
        if (child / "spec").is_dir():
            out.append(child)
    return out


def walk(repo: Path, suffixes: set[str] | None = None) -> Iterator[Path]:
    """Files under `repo`, skipping every `SKIP_DIRS` segment.

    `suffixes` filters by extension; omitted, every file is yielded.
    """
    for path in repo.rglob("*"):
        if not path.is_file():
            continue
        if SKIP_DIRS & set(path.relative_to(repo).parts):
            continue
        if suffixes is not None and path.suffix not in suffixes:
            continue
        yield path


def source_files(repo: Path) -> Iterator[Path]:
    """Files the symbol extractor would open. See `SOURCE_SUFFIXES`."""
    yield from walk(repo, SOURCE_SUFFIXES)


def markdown_files(repo: Path) -> Iterator[Path]:
    yield from walk(repo, {".md"})


def is_test_data(rel_path: str) -> bool:
    """The same rule a module declares with `exclude: ['tests/**']`.

    A fixture matrix is not a coverage gain; `spec-artifacts-process` ships 30
    deliberately malformed ones.
    """
    return (
        rel_path.startswith("tests/")
        or "/tests/" in rel_path
        or "/fixtures/" in rel_path
    )
