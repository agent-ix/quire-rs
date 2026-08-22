"""The engine benchmark's ratchet and metric semantics (quire-rs#231, CR-099).

The sweeps are slow and need a corpus on disk; the semantics are what regress,
so they are tested as pure functions.
"""

from __future__ import annotations

import json
import pathlib
import sys

import pytest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))

from bench import BenchError, compare, metrics_from, pct, score, silent_zeros  # noqa: E402

MANIFEST = json.loads(
    (pathlib.Path(__file__).resolve().parents[2] / "bench" / "manifest.json").read_text()
)


def test_every_declared_metric_states_unit_population_and_method():
    """The dictionary rule of agent-ix/quoin FR-043-AC-1, applied to the
    engine's own metrics: a
    metric that does not say what it counts is not one this benchmark emits."""
    for name, spec in MANIFEST["metrics"].items():
        for field in ("unit", "population", "method", "direction"):
            assert spec.get(field), f"{name} declares no {field}"
        assert spec["direction"] in {"higher-is-better", "lower-is-better", "gate-zero"}


def test_ratchet_improves_holds_and_regresses():
    """Better rewrites the baseline; equal holds and rewrites nothing; worse
    fails, keeping the baseline so a regression cannot quietly become the new
    floor."""
    assert compare("m", "higher-is-better", 90.0, 80.0) == ("improved", 90.0)
    assert compare("m", "higher-is-better", 80.0, 80.0) == ("held", 80.0)
    assert compare("m", "higher-is-better", 70.0, 80.0) == ("regressed", 80.0)

    # Direction is per metric: fewer dead tags is better.
    assert compare("m", "lower-is-better", 3, 10) == ("improved", 3)
    assert compare("m", "lower-is-better", 30, 10) == ("regressed", 10)

    # A metric with no baseline is recorded, not scored against nothing.
    assert compare("m", "higher-is-better", 5.0, None) == ("new", 5.0)


def test_the_sentinel_is_a_gate_not_a_ratchet():
    """No baseline to beat and no tolerance to spend. A silent zero is the class
    of defect that made three published SpecReviews wrong; a benchmark that let
    it score 0.98 and pass would be measuring the wrong thing."""
    assert compare("s", "gate-zero", 0, None) == ("held", 0)
    assert compare("s", "gate-zero", 1, None) == ("regressed", 0)
    # Even against a non-zero baseline — the gate never ratchets upward.
    assert compare("s", "gate-zero", 1, 5) == ("regressed", 0)


def test_an_undeclared_metric_is_refused():
    """The dictionary is the contract. A number the benchmark does not declare
    is a number it does not emit — the same rule FR-063 applies to the engine."""
    with pytest.raises(BenchError, match="not in the dictionary"):
        score(MANIFEST, {"self": {"coverage.invented": 1}}, {})


def test_the_report_is_deterministic_and_carries_provenance():
    """Two runs over identical inputs produce identical reports (quoin
    FR-043-AC-9),
    and every row carries the unit/population/method its metric declares — so a
    reader can interrogate a number without opening the manifest."""
    observed = {"self": {"coverage.backed_pct": 51.0, "sentinel.silent_zero": 0}}
    first = score(MANIFEST, observed, {})
    second = score(MANIFEST, observed, {})
    assert first == second
    for row in first["rows"]:
        assert row["unit"] and row["population"] and row["method"]


def test_metrics_are_omitted_rather_than_zeroed_when_unreadable(capsys):
    """An engine predating a field must not score 0 for it. Reporting 0% where
    the payload carries nothing would be the silent zero this benchmark exists
    to catch, produced by the benchmark itself."""
    payload = {"totals": {"backed": 5, "total": 10}, "untracked_symbols": []}
    out = metrics_from(payload)
    assert out["coverage.backed_pct"] == 50.0
    assert "coverage.binding_read_pct" not in out
    assert "properties.specific_shaped_pct" not in out
    # …and it says why, rather than omitting silently.
    err = capsys.readouterr().err
    assert "binding_census" in err and "specific_shaped" in err


def test_the_binding_rate_reads_the_census_when_it_is_there():
    payload = {
        "totals": {"backed": 5, "total": 10, "criteria": 4, "specific_shaped": 1},
        "binding_census": [
            {"language": "rust", "candidates": 8, "bound": 6},
            {"language": "python", "candidates": 2, "bound": 0},
        ],
    }
    out = metrics_from(payload)
    assert out["coverage.binding_read_pct"] == pct(6, 10)
    assert out["properties.specific_shaped_pct"] == 25.0


def test_silent_zero_counts_only_what_the_engine_did_not_already_report():
    """The sentinel must not depend on the code path it is checking. A hollow
    metric the engine already flagged is covered; one it did not is the leak."""
    hollow = {
        "name": "coverage.backed", "state": "measured",
        "value": 0, "population": 100, "examined": 50, "matched": 0,
    }
    assert silent_zeros({"metrics": [hollow], "diagnostics": []}) == 1
    assert silent_zeros({
        "metrics": [hollow],
        "diagnostics": [{"reason": "hollow-denominator"}],
    }) == 0
    # Nothing offered is not a silent zero — it is an honest zero.
    honest = {**hollow, "examined": 0}
    assert silent_zeros({"metrics": [honest], "diagnostics": []}) == 0
    # A metric that did not run has no ratio to be hollow.
    absent = {"name": "coverage.implements", "state": "not_computed"}
    assert silent_zeros({"metrics": [absent], "diagnostics": []}) == 0

    # CR-102: a COUNT is exempt. `matched` and the value are the same fact, so
    # zero reports that none was found. coverage.implements reads an honest
    # 0 of 42 on spec-artifacts-process; before the shape existed this gate
    # read 0 only because the engine's own false-positive hollow-denominator
    # satisfied the `covered` clause -- two defects cancelling.
    counted = {
        "name": "coverage.implements", "state": "measured", "shape": "count",
        "value": 0, "population": 42, "examined": 42, "matched": 0,
    }
    assert silent_zeros({"metrics": [counted], "diagnostics": []}) == 0
    # The identical numbers as a ratio still fail, so this measures the shape
    # rather than the arithmetic.
    assert silent_zeros({
        "metrics": [{**counted, "shape": "ratio"}], "diagnostics": [],
    }) == 1
    # An engine predating CR-102 emits no shape; read it as a ratio, the
    # reading that can still fail the gate rather than silently pass it.
    assert silent_zeros({
        "metrics": [{k: v for k, v in counted.items() if k != "shape"}],
        "diagnostics": [],
    }) == 1
