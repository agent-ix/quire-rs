"""Cross-repository coverage distribution checks for FR-050-AC-31."""

from __future__ import annotations

import pathlib
import sys

import pytest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))

import overfit_check  # noqa: E402
from overfit_check import compare, render, snapshot  # noqa: E402


def snap(**repos):
    return {
        "schema_version": 1,
        "repos": repos,
        "unreadable": [],
        "population": len(repos),
    }


@pytest.mark.trace("FR-050-AC-31")
def test_snapshot_records_unreadable_repositories_instead_of_zeroing(tmp_path, monkeypatch):
    paths = [pathlib.Path("/corpus/readable"), pathlib.Path("/corpus/unreadable")]
    monkeypatch.setattr(overfit_check, "repos", lambda root: paths)

    def measured(quire, module, repo):
        if repo.name == "unreadable":
            return None
        return {"backed": 4, "total": 8, "dead_tags": 0}

    monkeypatch.setattr(overfit_check, "measure", measured)
    result = snapshot("quire", None, tmp_path, limit=0)

    assert result["population"] == 2
    assert result["repos"] == {"readable": {"backed": 4, "total": 8, "dead_tags": 0}}
    assert result["unreadable"] == ["unreadable"]


@pytest.mark.trace("FR-050-AC-31")
def test_compare_reports_distribution_and_uses_only_shared_population():
    before = snap(
        a={"backed": 10, "total": 100, "dead_tags": 0},
        b={"backed": 20, "total": 100, "dead_tags": 0},
        gone={"backed": 50, "total": 100, "dead_tags": 0},
    )
    before["unreadable"] = ["lost-after"]
    before["population"] += 1
    after = snap(
        a={"backed": 15, "total": 100, "dead_tags": 0},
        b={"backed": 5, "total": 100, "dead_tags": 0},
        fresh={"backed": 30, "total": 100, "dead_tags": 0},
    )
    after["unreadable"] = ["lost-before"]
    after["population"] += 1

    result = compare(before, after)

    assert result["shared"] == 2
    assert result["only_before"] == ["gone"]
    assert result["only_after"] == ["fresh"]
    assert result["unreadable_before"] == ["lost-after"]
    assert result["unreadable_after"] == ["lost-before"]
    assert (result["improved"], result["regressed"], result["unchanged"]) == (1, 1, 0)
    assert result["total_gain"] == 5
    assert result["total_loss"] == 15
    assert result["net_gain"] == -10
    assert result["top_repo"] == "a"
    assert result["concentration"] == 1.0
    report = render(result)
    assert "population moved" in report
    assert "unreadable" in report
    assert "net rows backed: -10" in report
    assert "a (+5 rows)" in report
    assert any(
        line.lstrip().startswith("b ") and "-15 rows" in line
        for line in report.splitlines()
    )


@pytest.mark.trace("FR-050-AC-31")
def test_compare_zero_gain_has_zero_concentration():
    before = snap(a={"backed": 0, "total": 0, "dead_tags": 0})
    result = compare(before, before)

    assert result["improved"] == 0
    assert result["regressed"] == 0
    assert result["unchanged"] == 1
    assert result["total_gain"] == 0
    assert result["total_loss"] == 0
    assert result["net_gain"] == 0
    assert result["top_repo"] is None
    assert result["concentration"] == 0.0
