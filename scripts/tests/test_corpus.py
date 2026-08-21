"""The corpus membership rules, tested (#219).

`repos()`'s numbers are cited as authoritative in three PR bodies, the
CHANGELOG and a spec review; until this file, nothing verified the rules that
produce them — including the two behavior changes #202 shipped (`SKIP_DIRS`
members skipped at top level, and the structural `.git`-is-a-file worktree
check replacing the name regex).
"""

import subprocess
import sys
from pathlib import Path

from corpus import (
    is_test_data,
    is_worktree_sibling,
    markdown_files,
    repos,
    source_files,
    walk,
)


def make_repo(root, name, with_spec=True, git="dir"):
    repo = root / name
    (repo / "spec" if with_spec else repo).mkdir(parents=True)
    if git == "dir":
        (repo / ".git").mkdir()
    elif git == "file":
        (repo / ".git").write_text("gitdir: /elsewhere\n", encoding="utf-8")
    return repo


# ── repos() ──────────────────────────────────────────────────────────────────


def test_repos_membership_is_a_spec_directory(tmp_path):
    make_repo(tmp_path, "alpha")
    make_repo(tmp_path, "no-spec", with_spec=False)
    (tmp_path / "loose-file.md").write_text("", encoding="utf-8")

    assert [r.name for r in repos(tmp_path)] == ["alpha"]


def test_repos_skips_hidden_skipdirs_and_superseded(tmp_path):
    make_repo(tmp_path, "alpha")
    make_repo(tmp_path, ".hidden")
    # A `SKIP_DIRS` name at top level (the #202 behavior change).
    make_repo(tmp_path, "node_modules")
    # Superseded: the same repo counted twice under its old name.
    make_repo(tmp_path, "filament-ide")

    assert [r.name for r in repos(tmp_path)] == ["alpha"]


def test_repos_worktree_sibling_needs_structure_not_just_name(tmp_path):
    make_repo(tmp_path, "alpha")
    # `.git` is a file: a linked worktree — the same repo counted twice.
    make_repo(tmp_path, "alpha-task3", git="file")
    # `.git` is a directory: a real repository that merely matches the name;
    # skipping it by name alone would silently drop it from every sweep.
    make_repo(tmp_path, "legit-task42", git="dir")

    assert [r.name for r in repos(tmp_path)] == ["alpha", "legit-task42"]


def test_repos_is_sorted(tmp_path):
    for name in ("zeta", "alpha", "mid"):
        make_repo(tmp_path, name)

    assert [r.name for r in repos(tmp_path)] == ["alpha", "mid", "zeta"]


# ── is_worktree_sibling() ────────────────────────────────────────────────────


def test_worktree_sibling_true_only_for_name_plus_git_file(tmp_path):
    make_repo(tmp_path, "repo-task1", git="file")
    make_repo(tmp_path, "repo-task2", git="dir")
    make_repo(tmp_path, "plain-repo", git="file")

    assert is_worktree_sibling(tmp_path, "repo-task1") is True
    assert is_worktree_sibling(tmp_path, "repo-task2") is False
    assert is_worktree_sibling(tmp_path, "plain-repo") is False


# ── walk() and the suffix views ──────────────────────────────────────────────


def test_walk_skips_skipdirs_at_any_depth_and_filters_suffixes(tmp_path):
    repo = make_repo(tmp_path, "alpha")
    (repo / "src").mkdir()
    (repo / "src" / "lib.rs").write_text("", encoding="utf-8")
    (repo / "doc.md").write_text("", encoding="utf-8")
    (repo / "node_modules" / "dep").mkdir(parents=True)
    (repo / "node_modules" / "dep" / "index.ts").write_text("", encoding="utf-8")
    (repo / "src" / "__pycache__").mkdir()
    (repo / "src" / "__pycache__" / "m.py").write_text("", encoding="utf-8")

    everything = {p.relative_to(repo).as_posix() for p in walk(repo)}
    assert everything == {"src/lib.rs", "doc.md"}

    assert {p.name for p in walk(repo, {".rs"})} == {"lib.rs"}
    assert {p.name for p in source_files(repo)} == {"lib.rs"}
    assert {p.name for p in markdown_files(repo)} == {"doc.md"}


# ── is_test_data() ───────────────────────────────────────────────────────────


def test_is_test_data_matches_the_module_exclude_rule():
    assert is_test_data("tests/fixtures/matrix.md") is True
    assert is_test_data("nested/tests/matrix.md") is True
    assert is_test_data("nested/fixtures/matrix.md") is True
    assert is_test_data("spec/tests.md") is False
    assert is_test_data("attests/matrix.md") is False


def test_is_test_data_has_exactly_one_definition():
    # #202 extracted this rule to end the divergent copies, then
    # `classify_matrices.py` kept a local one anyway. One definition, importable.
    import classify_matrices
    import corpus

    assert classify_matrices.is_test_data is corpus.is_test_data


# ── import structure (#219) ──────────────────────────────────────────────────


def test_python_dash_m_resolves_the_flat_sibling_imports():
    # `python -m scripts.slash_tag_sweep` from the repo root: the package
    # __init__ adds `scripts/` to sys.path so `from corpus import …` resolves.
    repo_root = Path(__file__).resolve().parents[2]
    proc = subprocess.run(
        [sys.executable, "-m", "scripts.slash_tag_sweep", "--help"],
        cwd=repo_root,
        capture_output=True,
        text=True,
    )
    assert proc.returncode == 0, proc.stderr
    assert "--allow-dirty" in proc.stdout
