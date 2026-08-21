"""The slash-sweep classifier/rewriter corpus (#217).

Every class the harness can assign, the multi-chain line the first-match
classifier silently dropped, the span-replace the docstring always claimed,
the R7 placebo guard, and the counted-refusal census rule.
"""

import subprocess

import pytest
from slash_tag_sweep import (
    AMBER,
    ELISION,
    GREEN,
    PROSE,
    classify_line,
    dirty_reason,
    rewrite_line,
    sweep_repo,
)

# ── classify_line ────────────────────────────────────────────────────────────


def classes(line):
    return [r["class"] for r in classify_line(line)]


def test_green_anchored_minted_chain():
    records = classify_line("// TC-577 / NFR-002-AC-4: the bench.")
    assert [r["class"] for r in records] == [GREEN]
    assert records[0]["chain"] == "TC-577 / NFR-002-AC-4"


def test_amber_chain_with_unminted_ids():
    records = classify_line("// FR-006/FR-007-CON-1: neither id is minted")
    assert [r["class"] for r in records] == [AMBER]
    assert records[0]["mints_nothing"] == ["FR-006", "FR-007-CON-1"]


def test_elision_numeric_shorthand():
    assert classes("// FR-011-AC-6/7/8: the recognised shorthand") == [ELISION]


def test_prose_unanchored_chain():
    assert classes("# Pull Architecture (FR-010 / FR-011)") == [PROSE]


def test_prose_wrapped_parenthetical_r1b():
    # The `)` closes a parenthetical opened on a PREVIOUS line — the line
    # anchors correctly and is still prose.
    assert classes("/// TC-816/TC-817).") == [PROSE]


def test_comma_first_slash_second_chain_is_counted():
    # The #217 defect: `CHAIN.search` saw only the comma chain, returned
    # PROSE-with-no-chain, and the line vanished from the census entirely.
    records = classify_line("// TC-001, TC-002 — see FR-003/FR-004")
    assert len(records) == 1, "the slash chain must be found and counted"
    assert records[0]["chain"] == "FR-003/FR-004"
    assert records[0]["class"] == PROSE  # mid-line: R1 refuses the rewrite


def test_every_chain_on_a_line_is_classified():
    records = classify_line("// TC-1/TC-2: see also FR-3/FR-4")
    assert [r["class"] for r in records] == [GREEN, PROSE]
    assert [r["chain"] for r in records] == ["TC-1/TC-2", "FR-3/FR-4"]


def test_r7_unbindable_plus_tail_is_not_green():
    # The #208 placebo shape, verbatim: the rewrite would produce a clean
    # comma list whose ` + NFR-006` tail still stops the grammar at the first
    # id (measured: FR-024-AC-4 / FR-025-AC-4 / FR-027-AC-6 stayed unbacked).
    records = classify_line(
        "// TC-473 / FR-024-AC-4 + NFR-006: path-sorted, byte-identical."
    )
    assert [r["class"] for r in records] == [AMBER]
    assert records[0]["unbindable_tail"] == "NFR-006"


def test_r7_parenthetical_tail_stays_green():
    # The repaired form of the same line: the extra id moved into prose, the
    # chain binds, GREEN is honest again.
    records = classify_line(
        "// TC-473 / FR-024-AC-4: path-sorted, byte-identical (NFR-006)."
    )
    assert [r["class"] for r in records] == [GREEN]


def test_line_without_slash_chain_yields_no_record():
    assert classify_line("// TC-473, FR-024-AC-4: comma only") == []
    assert classify_line("let ratio = a / b;") == []


# ── rewrite_line ─────────────────────────────────────────────────────────────


def test_rewrite_replaces_inside_the_span_only():
    # The same chain text occurs twice; a first-occurrence `str.replace` would
    # edit the wrong one when handed the second span.
    line = "// TC-1/TC-2: TC-1/TC-2 again"
    first, second = classify_line(line)
    assert (first["class"], second["class"]) == (GREEN, PROSE)
    assert rewrite_line(line, second["start"], second["end"]) == (
        "// TC-1/TC-2: TC-1, TC-2 again"
    )
    assert rewrite_line(line, first["start"], first["end"]) == (
        "// TC-1, TC-2: TC-1/TC-2 again"
    )


def test_rewrite_is_byte_identical_outside_the_span():
    line = "//   TC-9 / FR-1-AC-2:   trailing   spaces   "
    (record,) = classify_line(line)
    rewritten = rewrite_line(line, record["start"], record["end"])
    assert rewritten == "//   TC-9, FR-1-AC-2:   trailing   spaces   "


# ── sweep_repo: census discipline ────────────────────────────────────────────


def test_unreadable_file_is_a_counted_refusal(tmp_path):
    (tmp_path / "good.rs").write_text("// TC-1/TC-2: fine\n", encoding="utf-8")
    (tmp_path / "bad.rs").write_bytes(b"\xff\xfe not utf-8 // TC-3/TC-4\n")

    result = sweep_repo(tmp_path, write=False)

    assert result["counts"][GREEN] == 1
    assert [r["path"] for r in result["unreadable_files"]] == ["bad.rs"]
    assert "UnicodeDecodeError" in result["unreadable_files"][0]["reason"]


def test_write_edits_green_spans_in_place(tmp_path):
    path = tmp_path / "lib.rs"
    path.write_text(
        "// TC-1/TC-2: green\n"
        "// FR-006/FR-007-CON-1: amber stays\n"
        "# Pull Architecture (FR-010 / FR-011)\n",
        encoding="utf-8",
    )

    result = sweep_repo(tmp_path, write=True)

    assert result["files_edited"] == 1
    assert path.read_text(encoding="utf-8") == (
        "// TC-1, TC-2: green\n"
        "// FR-006/FR-007-CON-1: amber stays\n"
        "# Pull Architecture (FR-010 / FR-011)\n"
    )


def test_dry_run_never_writes(tmp_path):
    path = tmp_path / "lib.rs"
    before = "// TC-1/TC-2: green\n"
    path.write_text(before, encoding="utf-8")

    result = sweep_repo(tmp_path, write=False)

    assert result["counts"][GREEN] == 1
    assert path.read_text(encoding="utf-8") == before


# ── dirty_reason: the --write precondition ───────────────────────────────────


def git(*args, cwd):
    subprocess.run(
        ["git", *args],
        cwd=cwd,
        check=True,
        capture_output=True,
        env={
            "GIT_AUTHOR_NAME": "t",
            "GIT_AUTHOR_EMAIL": "t@t",
            "GIT_COMMITTER_NAME": "t",
            "GIT_COMMITTER_EMAIL": "t@t",
            "PATH": "/usr/bin:/bin",
            "HOME": str(cwd),
        },
    )


@pytest.fixture
def git_repo(tmp_path):
    git("init", "-q", cwd=tmp_path)
    return tmp_path


def test_dirty_worktree_is_refused(git_repo):
    (git_repo / "lib.rs").write_text("// TC-1/TC-2\n", encoding="utf-8")
    assert dirty_reason(git_repo) == "dirty git worktree"


def test_clean_worktree_is_allowed(git_repo):
    (git_repo / "lib.rs").write_text("// TC-1/TC-2\n", encoding="utf-8")
    git("add", ".", cwd=git_repo)
    git("commit", "-q", "-m", "seed", cwd=git_repo)
    assert dirty_reason(git_repo) is None


def test_non_git_directory_is_refused(tmp_path):
    reason = dirty_reason(tmp_path)
    assert reason is not None and "git" in reason
