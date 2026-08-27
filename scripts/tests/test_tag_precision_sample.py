import json

import pytest

from tag_precision_sample import (
    CalibrationError,
    assess,
    digest,
    render,
    select_sample,
)


def candidates():
    return [
        {"id": f"p-{index}", "stratum": "production-symbol"} for index in range(5)
    ] + [{"id": f"m-{index}", "stratum": "module-scope"} for index in range(3)]


def test_sample_is_deterministic_and_stratified():
    rows = candidates()
    first = select_sample(rows, {"production-symbol": 3, "module-scope": 2}, "seed")
    second = select_sample(
        list(reversed(rows)), {"production-symbol": 3, "module-scope": 2}, "seed"
    )
    assert first == second
    by_id = {row["id"]: row for row in rows}
    assert sum(by_id[row_id]["stratum"] == "production-symbol" for row_id in first) == 3
    assert sum(by_id[row_id]["stratum"] == "module-scope" for row_id in first) == 2


def frame_and_rulings():
    frame = {
        "date": "2026-08-27",
        "seed": "seed",
        "provenance": {"cli": "1", "engine": "2", "module_revision": "abc"},
        "population": {"production-symbol": 10, "module-scope": 2},
        "sample_ids": ["p", "m"],
        "candidates": [
            {
                "id": "p",
                "stratum": "production-symbol",
                "repo": "a",
                "path": "a.py",
                "line": 1,
                "occurrences": [{"line": 1}],
            },
            {
                "id": "m",
                "stratum": "module-scope",
                "repo": "b",
                "path": "b.rs",
                "line": 2,
                "occurrences": [{"line": 3}],
            },
        ],
    }
    frame_hash = digest(json.dumps(frame, sort_keys=True, separators=(",", ":")))
    rulings = {
        "frame_sha256": frame_hash,
        "decision": "retain-current-rule",
        "recall_effect": "none; no matcher change",
        "locality_effect": "none; loci are unchanged",
        "rulings": [
            {
                "id": "p",
                "ruling": "authored-tag",
                "rationale": "tag opens the doc comment",
            },
            {
                "id": "m",
                "ruling": "ambiguous",
                "rationale": "module banner is context dependent",
            },
        ],
    }
    return frame, rulings


def test_ambiguity_stays_in_denominator_and_forms_an_upper_bound():
    frame, rulings = frame_and_rulings()
    result = assess(frame, rulings)
    assert result["population"] == 12
    assert result["precision_lower"] == pytest.approx(10 / 12)
    assert result["precision_upper"] == 1
    assert result["unresolved"] == []
    rendered = render(frame, rulings, result)
    assert "83.3%–100.0%" in rendered
    assert "retain-current-rule" in rendered


def test_unresolved_is_explicit_not_excluded():
    frame, rulings = frame_and_rulings()
    rulings["rulings"][1] = {
        "id": "m",
        "ruling": "unresolved",
        "rationale": "needs repository owner context",
    }
    result = assess(frame, rulings)
    assert result["strata"]["module-scope"]["sample"] == 1
    assert result["strata"]["module-scope"]["counts"]["unresolved"] == 1
    assert result["unresolved"] == ["m"]
    assert "Explicit unresolved rows" in render(frame, rulings, result)


def test_missing_rationale_and_wrong_frame_are_refused():
    frame, rulings = frame_and_rulings()
    rulings["rulings"][0]["rationale"] = ""
    with pytest.raises(CalibrationError, match="no rationale"):
        assess(frame, rulings)
    rulings["rulings"][0]["rationale"] = "restored"
    rulings["frame_sha256"] = "wrong"
    with pytest.raises(CalibrationError, match="frame digest"):
        assess(frame, rulings)
