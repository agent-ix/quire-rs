"""Cross-corpus overfit statistics (quire-rs#237, CR-101).

The sweep is minutes of work over 241 repositories; the *distribution
arithmetic* is what decides whether a gain generalizes, so it is tested as a
pure function.
"""

from __future__ import annotations

import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))

from overfit_check import CONCENTRATION_FLOOR, compare, pct, render  # noqa: E402


def snap(**repos):
    return {"repos": repos, "unreadable": [], "population": len(repos)}


def test_a_gain_spread_across_repositories_is_not_concentrated():
    """The shape of a genuine engine improvement: many repositories move a
    little. This is what generalization looks like."""
    before = snap(
        a={"backed": 10, "total": 100, "dead_tags": 0},
        b={"backed": 10, "total": 100, "dead_tags": 0},
        c={"backed": 10, "total": 100, "dead_tags": 0},
        d={"backed": 10, "total": 100, "dead_tags": 0},
    )
    after = snap(
        a={"backed": 15, "total": 100, "dead_tags": 0},
        b={"backed": 15, "total": 100, "dead_tags": 0},
        c={"backed": 15, "total": 100, "dead_tags": 0},
        d={"backed": 15, "total": 100, "dead_tags": 0},
    )
    diff = compare(before, after)
    assert diff["improved"] == 4
    assert diff["regressed"] == 0
    assert diff["total_gain"] == 20
    assert diff["concentration"] == 0.25
    assert diff["concentration"] < CONCENTRATION_FLOOR
    assert "overfitting" not in render(diff)


def test_a_gain_from_one_repository_is_named_as_such():
    """The shape of a change tuned to one corpus: the ecosystem total moves and
    241 repositories did not. A single average would report this as a win."""
    before = snap(
        a={"backed": 10, "total": 100, "dead_tags": 0},
        b={"backed": 10, "total": 100, "dead_tags": 0},
        c={"backed": 10, "total": 100, "dead_tags": 0},
    )
    after = snap(
        a={"backed": 60, "total": 100, "dead_tags": 0},
        b={"backed": 10, "total": 100, "dead_tags": 0},
        c={"backed": 11, "total": 100, "dead_tags": 0},
    )
    diff = compare(before, after)
    assert diff["total_gain"] == 51
    assert diff["top_repo"] == "a"
    assert diff["concentration"] >= CONCENTRATION_FLOOR
    text = render(diff)
    assert "overfitting" in text
    assert "a (+50 rows)" in text


def test_regressions_are_counted_separately_from_gains():
    """A change that lifts most repositories while breaking one is a different
    fact from one that lifts all of them, and the report must not net them into
    a single number."""
    before = snap(
        a={"backed": 10, "total": 100, "dead_tags": 0},
        b={"backed": 10, "total": 100, "dead_tags": 0},
    )
    after = snap(
        a={"backed": 20, "total": 100, "dead_tags": 0},
        b={"backed": 2, "total": 100, "dead_tags": 0},
    )
    diff = compare(before, after)
    assert (diff["improved"], diff["regressed"], diff["unchanged"]) == (1, 1, 0)
    # `total_gain` is the GAIN, not the net — a regression is not spent against it.
    assert diff["total_gain"] == 10
    assert "-8 rows" in render(diff)


def test_a_moving_population_is_reported_not_hidden():
    """A sweep that silently shrank its own population would show every
    remaining repository improving. The comparison is over the intersection and
    says so."""
    before = snap(
        a={"backed": 10, "total": 100, "dead_tags": 0},
        gone={"backed": 0, "total": 100, "dead_tags": 0},
    )
    after = snap(
        a={"backed": 12, "total": 100, "dead_tags": 0},
        fresh={"backed": 99, "total": 100, "dead_tags": 0},
    )
    diff = compare(before, after)
    assert diff["shared"] == 1
    assert diff["only_before"] == ["gone"]
    assert diff["only_after"] == ["fresh"]
    assert "population moved" in render(diff)
    # The newcomer's 99 does not count as a gain.
    assert diff["total_gain"] == 2


def test_no_change_reports_no_concentration():
    before = snap(a={"backed": 10, "total": 100, "dead_tags": 0})
    diff = compare(before, before)
    assert diff["total_gain"] == 0
    assert diff["concentration"] == 0.0
    assert diff["top_repo"] is None
    assert "concentration" not in render(diff)


def test_pct_does_not_divide_by_zero():
    """A repository that mints nothing is 0%, and that is a fact rather than a
    crash — 153 of 237 repositories were in exactly that state (#72)."""
    assert pct(0, 0) == 0.0
    assert pct(1, 4) == 25.0
