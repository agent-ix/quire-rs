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

from bench import (  # noqa: E402
    BenchError,
    compare,
    metrics_from,
    pct,
    score,
    selected,
    silent_zeros,
)

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


def test_the_report_carries_no_time_varying_field_and_states_its_provenance():
    """The report is byte-stable across runs (quoin FR-043-AC-9), and every row
    carries the unit/population/method its metric declares — so a reader can
    interrogate a number without opening the manifest.

    **Asserts the absence directly (CR-103).** This was two `score()` calls
    compared for equality, which cannot fail: `score` is pure over its
    arguments, so nothing you could change inside it breaks that assertion
    short of deliberately injecting a clock. The guard it was reaching for is
    "no field varies with time", and that is what it now checks — a `now`,
    `timestamp` or `generated_at` added to a row fails here, where the
    equality form would have sailed through two calls a microsecond apart.
    """
    observed = {"self": {"coverage.backed_pct": 51.0, "sentinel.silent_zero": 0}}
    report = score(MANIFEST, observed, {})

    assert report["rows"], "an empty report asserts nothing below"
    for row in report["rows"]:
        assert row["unit"] and row["population"] and row["method"]
        stamped = sorted(
            k for k in row
            if any(t in k.lower() for t in ("time", "date", "stamp", "now", "generated", "ran_at"))
        )
        assert not stamped, f"time-varying field(s) in a scored row: {stamped}"

    # Serializable with sorted keys and no float drift, which is what makes the
    # byte-comparison in `make bench` meaningful rather than incidental.
    once = json.dumps(report, sort_keys=True)
    assert once == json.dumps(score(MANIFEST, observed, {}), sort_keys=True)


def test_metrics_are_omitted_rather_than_zeroed_when_unreadable(capsys):
    """An engine predating a field must not score 0 for it. Reporting 0% where
    the payload carries nothing would be the silent zero this benchmark exists
    to catch, produced by the benchmark itself."""
    payload = {"totals": {"backed": 5, "total": 10}, "untracked_symbols": []}
    out = metrics_from(payload)
    assert out["coverage.backed_pct"] == 50.0
    assert "coverage.binding_read_pct" not in out
    assert "authoring.tag_rate" not in out
    assert "properties.specific_shaped_pct" not in out
    assert "coverage.minting_repos" not in out
    # …and it says why, rather than omitting silently.
    err = capsys.readouterr().err
    assert "binding_census" in err and "specific_shaped" in err


def test_the_binding_rate_reads_the_census_when_it_is_there():
    payload = {
        "totals": {"backed": 5, "total": 10, "criteria": 4, "specific_shaped": 1},
        "binding_census": [
            {"language": "rust", "candidates": 8, "tagged": 7, "bound": 6},
            {"language": "python", "candidates": 2, "tagged": 1, "bound": 0},
        ],
        "metrics": [
            {
                "name": "minting.section_hit_rate",
                "state": "measured",
                "value": 3,
                "population": 3,
            }
        ],
    }
    out = metrics_from(payload)
    assert out["coverage.binding_read_pct"] == pct(6, 10)
    assert out["authoring.tag_rate"] == pct(8, 10)
    assert out["properties.specific_shaped_pct"] == 25.0
    assert out["coverage.minting_repos"] == 1


def test_specific_shape_reads_the_current_metrics_envelope():
    payload = {
        "totals": {"backed": 1, "total": 2},
        "binding_census": [],
        "metrics": [
            {
                "name": "coverage.specific_shaped",
                "state": "measured",
                "value": 3,
                "population": 12,
            },
            {
                "name": "minting.section_hit_rate",
                "state": "measured",
                "value": 1,
                "population": 1,
            },
        ],
    }
    assert metrics_from(payload)["properties.specific_shaped_pct"] == 25.0


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


def test_a_corpus_can_scope_which_metrics_it_is_scored_on(capsys):
    """CR-103: `quoin` and `spec-artifacts-process` are carried for LANGUAGE
    coverage, not for their coverage figures.

    Their `backed_pct` moves whenever somebody writes a spec row, and
    ratcheting a tree this repository does not control would train everyone to
    run `bench-update` reflexively — which is how a ratchet stops being one.
    An entry with no `metrics` key is scored on everything, as before.
    """
    measured = {
        "coverage.backed_pct": 72.9,
        "sentinel.silent_zero": 0,
        "skeptic.suspicion_rate": 0.0,
    }
    gates = ["sentinel.silent_zero", "skeptic.suspicion_rate"]

    assert selected({"name": "quoin", "metrics": gates}, measured) == {
        "sentinel.silent_zero": 0,
        "skeptic.suspicion_rate": 0.0,
    }
    # No allowlist means every metric, so existing entries are unaffected.
    assert selected({"name": "self"}, measured) == measured

    # A declared metric the payload could not supply is named, not silently
    # dropped — the same rule `metrics_from` follows for an unreadable field.
    out = selected({"name": "quoin", "metrics": gates + ["coverage.implements_pct"]}, measured)
    assert "coverage.implements_pct" not in out
    assert "coverage.implements_pct" in capsys.readouterr().err


def test_suspicion_rate_is_the_language_coverage_guard():
    """CR-103: the number that would have caught the v0.44.0 misread.

    549 suspicions over 551 TypeScript candidates is a rule reading the wrong
    language, not a corpus full of vacuous tests. Verified end to end: reverting
    the guard-list fix moves `quoin/skeptic.suspicion_rate` from 0.0 to 99.1 and
    `make bench` exits 1.
    """
    misread = {
        "totals": {"backed": 1, "total": 2},
        "binding_census": [{"language": "typescript", "candidates": 551, "bound": 374}],
        "suspicions": [{"kind": "vacuous-under-guard"} for _ in range(549)],
    }
    assert metrics_from(misread)["skeptic.suspicion_rate"] == 99.64

    # The shipped shape: two genuine positives in nine hundred symbols.
    honest = {
        "totals": {"backed": 1, "total": 2},
        "binding_census": [{"language": "rust", "candidates": 883, "bound": 591}],
        "suspicions": [{"kind": "vacuous-under-guard"}, {"kind": "vacuous-under-guard"}],
    }
    assert metrics_from(honest)["skeptic.suspicion_rate"] == 0.23

    # No evidence symbols means no rate, not 0% — the silent zero this
    # benchmark exists to catch, produced by the benchmark itself.
    empty = {"totals": {}, "binding_census": [], "suspicions": []}
    assert "skeptic.suspicion_rate" not in metrics_from(empty)
