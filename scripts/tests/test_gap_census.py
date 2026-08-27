"""FR-066 census partition tests (agent-ix/quire-rs#277)."""

from __future__ import annotations

import copy
import json
import pathlib
import sys

import pytest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))

from gap_census import (  # noqa: E402
    CensusError,
    Target,
    aggregate,
    classify_repo,
    engine_identity,
    render_markdown,
    scan_rows,
    write_reports,
)


def repo_fixture(tmp_path: pathlib.Path, *, tags: bool = True) -> pathlib.Path:
    repo = tmp_path / "sample"
    (repo / "spec").mkdir(parents=True, exist_ok=True)
    (repo / "src").mkdir(exist_ok=True)
    (repo / "spec" / "tests.md").write_text(
        """---
id: TestMatrix-001
type: TestMatrix
---
# Tests

## Wrong Section

| Test ID | Title | Type | Status |
|---|---|---|---|
| TC-002 | declaration | Unit | 🚧 |

## Test Case Summary

| Test ID | Title | Type | Status |
|---|---|---|---|
| TC-001 | backed | Unit | ✅ |
| TC-003 | unread | Unit | 🚧 |
| TC-004 | mismatch | Unit | 🚧 |
| TC-005 | exempt | Analysis | ✅ |
| TC-006 | absent | Unit | ✅ |

## Constraints

| ID | Constraint |
|---|---|
| FR-001-CON-1 | unknown class |

## Malformed Width But Authored

| ID | Criteria |
|---|---|
| FR-002-AC-1 | criterion | extra cell |
""",
        encoding="utf-8",
    )
    source = "// TC-003\n" if tags else "// no authored trace tags\n"
    (repo / "src" / "lib.rs").write_text(source, encoding="utf-8")
    if tags:
        (repo / "src" / "checks.py").write_text("# TC-004\n", encoding="utf-8")
    return repo


def target() -> Target:
    return Target("test-case", "TestMatrix", ("Test Case Summary",), "Test ID", ())


def report(*, zero_tags: bool = False) -> dict:
    unbacked = ["TC-003", "TC-004", "TC-005", "TC-006"]
    return {
        "totals": {"total": 5, "backed": 1},
        "minted_targets": [
            {
                "id": row_id,
                "target": "test-case",
                "document": "spec/tests.md",
                "line": line,
                "backed": row_id == "TC-001",
            }
            for row_id, line in zip(
                ["TC-001", "TC-003", "TC-004", "TC-005", "TC-006"],
                [17, 18, 19, 20, 21],
            )
        ],
        "unbacked_rows": [{"target_ids": [row]} for row in unbacked],
        "no_symbol_rows": [{"target_ids": ["TC-005"]}],
        "status_lies": [{"target_ids": ["TC-006"]}],
        "diagnostics": []
        if zero_tags
        else [{"reason": "low-symbol-binding", "value": "rust"}],
        "unmatched_tags": []
        if zero_tags
        else [
            {
                "trace_id": "TC-003",
                "language": "rust",
                "path": "src/lib.rs",
                "line": 1,
                "symbol": "tests::unread",
            },
            {
                "trace_id": "TC-004",
                "language": "python",
                "path": "src/checks.py",
                "line": 1,
                "symbol": "test_mismatch",
            },
        ],
        "binding_census": [
            {
                "language": "rust",
                "candidates": 10,
                "tagged": 0 if zero_tags else 1,
                "bound": 2 if zero_tags else 0,
                "forms": ["rust-trace-attribute"],
            }
        ],
    }


def classified_fixture(tmp_path: pathlib.Path) -> dict:
    return classify_repo(repo_fixture(tmp_path), report(), [target()])


def test_tc1062_four_populations_keep_their_units(tmp_path):
    """TC-1062"""
    result = classified_fixture(tmp_path)
    assert result["populations"] == {
        "P1_evidence_symbols": 10,
        "P2_tagged_symbols": 1,
        "P3_authored_rows": 8,
        "P4_minted_rows": 5,
    }


def test_tc1063_authored_scan_is_independent_of_minting(tmp_path):
    """TC-1063"""
    authored, minted = scan_rows(repo_fixture(tmp_path), [target()])
    assert {row.row_id for row in authored.values()} == {
        "TC-001",
        "TC-002",
        "TC-003",
        "TC-004",
        "TC-005",
        "TC-006",
        "FR-001-CON-1",
        "FR-002-AC-1",
    }
    assert {row.row_id for row in minted.values()} == {
        "TC-001",
        "TC-003",
        "TC-004",
        "TC-005",
        "TC-006",
    }


def test_tc1064_strict_precedence_reaches_all_six_dispositions(tmp_path):
    """TC-1064"""
    result = classified_fixture(tmp_path)
    assert result["counts"] == {
        "backed": 1,
        "instrument-unread": 1,
        "declaration-unreached": 1,
        "marker-form-mismatch": 1,
        "id-class-unminted": 2,
        "method-exempt": 1,
        "authoring-absent": 1,
    }


def test_tc1065_invariant_refuses_a_false_p4_population(tmp_path):
    """TC-1065"""
    broken = report()
    broken["totals"]["total"] = 6
    with pytest.raises(CensusError, match="disagreeing engine facts"):
        classify_repo(repo_fixture(tmp_path), broken, [target()])
    result = classified_fixture(tmp_path)
    combined = aggregate([result])
    assert (
        sum(combined["counts"].values()) == combined["populations"]["P3_authored_rows"]
    )
    assert "residual" not in combined["counts"]


def test_tc1066_status_lie_is_only_an_overlay(tmp_path):
    """TC-1066"""
    result = classified_fixture(tmp_path)
    assert result["status_lie_overlay"] == 1
    assert result["counts"]["authoring-absent"] == 1
    assert "status-lie" not in result["counts"]


def test_tc1067_zero_authored_tags_are_not_instrument_failure(tmp_path):
    """TC-1067"""
    result = classify_repo(
        repo_fixture(tmp_path, tags=False), report(zero_tags=True), [target()]
    )
    assert result["zero_tag_readable"] is True
    assert result["counts"]["instrument-unread"] == 0
    assert result["counts"]["authoring-absent"] == 3


def engine_payload(capabilities=None):
    return {
        "engine": {
            "cli": "0.30.2",
            "engine": "a14dcb2",
            "capabilities": capabilities
            or [
                "binding_census",
                "binding_census.tagged",
                "metrics_envelope",
                "minted_targets",
                "unmatched_tags",
            ],
        }
    }


def test_tc1068_engine_provenance_and_capabilities_are_refusals():
    """TC-1068"""
    identity = engine_identity(engine_payload())
    with pytest.raises(CensusError, match="binding_census.tagged"):
        engine_identity(
            engine_payload(
                [
                    "binding_census",
                    "metrics_envelope",
                    "minted_targets",
                    "unmatched_tags",
                ]
            )
        )
    changed = engine_payload()
    changed["engine"]["engine"] = "different"
    with pytest.raises(CensusError, match="changed"):
        engine_identity(changed, identity)


def payload(tmp_path):
    repo = classified_fixture(tmp_path)
    return {
        "date": "2026-08-26",
        "provenance": {
            "cli": "0.30.2",
            "engine": "a14dcb2",
            "capabilities": [
                "binding_census",
                "binding_census.tagged",
                "metrics_envelope",
                "minted_targets",
                "unmatched_tags",
            ],
            "module_sha": "abc123",
            "repos_enumerated": 1,
            "repos_scanned": 1,
            "exclusions": [],
        },
        "aggregate": aggregate([repo]),
        "repositories": [repo],
    }


def test_tc1069_reports_are_provenanced_and_byte_stable(tmp_path):
    """TC-1069"""
    data = payload(tmp_path)
    first = write_reports(data, tmp_path / "out")
    first_bytes = tuple(path.read_bytes() for path in first)
    second = write_reports(copy.deepcopy(data), tmp_path / "out")
    assert first_bytes == tuple(path.read_bytes() for path in second)
    loaded = json.loads(first[0].read_text())
    assert loaded["provenance"]["module_sha"] == "abc123"


def test_tc1070_census_workflow_is_never_a_change_gate():
    """TC-1070"""
    root = pathlib.Path(__file__).resolve().parents[2]
    workflow = (root / ".github/workflows/gap-census.yml").read_text()
    assert "schedule:" in workflow and "workflow_dispatch:" in workflow
    assert "pull_request:" not in workflow and "push:" not in workflow
    makefile = (root / "Makefile").read_text()
    ci_prerequisites = next(
        line for line in makefile.splitlines() if line.startswith("ci:")
    )
    assert "census" not in ci_prerequisites


def test_tc1071_human_report_names_locus_owner_reason_and_action(tmp_path):
    """TC-1071"""
    rendered = render_markdown(payload(tmp_path))
    for token in (
        "instrument-unread",
        "declaration-unreached",
        "marker-form-mismatch",
        "id-class-unminted",
        "method-exempt",
        "authoring-absent",
    ):
        assert token in rendered
    assert "sample/spec/tests.md" in rendered and "next action" in rendered.lower()


def test_tc1072_structural_vocabulary_is_related_not_widened():
    """TC-1072"""
    root = pathlib.Path(__file__).resolve().parents[2]
    requirement = (
        root / "spec/functional/FR-066-gap-disposition-census.md"
    ).read_text()
    assert "engineering-assurance/docs/structural-coverage.md" in requirement
    assert "does not\nchange that module's enum" in requirement
