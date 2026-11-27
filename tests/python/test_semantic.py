"""FR-072 Python binding parity (TC-1635, TC-1644 Python half).

`quire.extract_semantic` and the additive `semantic` payload of
`quire.extract_filament_core` must equal the Rust output recorded in
`tests/fixtures/semantic/cases.expected.json` for every case of
`tests/fixtures/semantic/cases.json`, as JSON values.
"""

import copy
import json
from pathlib import Path

import pytest
import quire

FIXTURES = Path(__file__).resolve().parents[1] / "fixtures" / "semantic"
# The mapping/module-ok fixture set lives in the `quire-fixtures` workspace
# crate (PLAT-901), not under tests/fixtures/: quire-rs owns it because it is
# the parser that defines the forms it exercises, and quoin (its original
# home) resolves it as a dependency rather than holding a copy.
FIXTURE_CRATE = Path(__file__).resolve().parents[2] / "crates" / "quire-fixtures" / "fixtures"


def _cases():
    cases = json.loads((FIXTURES / "cases.json").read_text())["cases"]
    return cases


def _expected():
    return json.loads((FIXTURES / "cases.expected.json").read_text())


@pytest.mark.trace("FR-072-AC-6")
def test_extract_semantic_matches_rust_for_every_case():
    """binding output equals the Rust oracle, case by case."""
    expected = _expected()
    for case in _cases():
        result = quire.extract_semantic(case["input"])
        assert isinstance(result, dict), case["name"]
        assert result == expected[case["name"]], case["name"]


@pytest.mark.trace("NFR-021-AC-4")
def test_extract_semantic_is_deterministic():
    """Python half: repeated runs agree including diagnostic order."""
    for case in _cases():
        first = quire.extract_semantic(case["input"])
        second = quire.extract_semantic(case["input"])
        assert first == second, case["name"]
        assert [d["code"] for d in first["diagnostics"]] == [d["code"] for d in second["diagnostics"]]


@pytest.mark.trace("FR-075-AC-20")
def test_physical_model_unit_matches_rust_golden():
    """The Python binding matches the Rust record for dimension and unit output."""
    request = json.loads((FIXTURES / "physical-model-unit.input.json").read_text())
    expected = json.loads((FIXTURES / "physical-model-unit.expected.json").read_text())
    first = quire.extract_semantic(request)
    second = quire.extract_semantic(request)
    assert first == expected
    assert second == expected


@pytest.mark.trace(
    "FR-075-AC-16",
    "FR-075-AC-17",
    "FR-075-AC-18",
    "FR-075-AC-19",
    "FR-075-AC-20",
)
def test_physical_model_cases_match_complete_rust_records():
    """The Python binding matches the full Rust records for every physical-model case."""
    cases = json.loads(
        (FIXTURES / "physical-model-cases.input.json").read_text()
    )["cases"]
    expected = json.loads(
        (FIXTURES / "physical-model-cases.expected.json").read_text()
    )
    assert len(cases) == len(expected)
    for case in cases:
        name = case["name"]
        record = quire.extract_semantic(case["request"])
        assert record == expected[name], name


def _physical_request(section, columns, rows, objects=(), artifacts=()):
    request = copy.deepcopy(
        json.loads((FIXTURES / "physical-model-unit.input.json").read_text())
    )
    table = "| " + " | ".join(columns) + " |\n|" + "|".join("---" for _ in columns) + "|\n"
    table += "".join("| " + " | ".join(row) + " |\n" for row in rows)
    request["markdown"] = (
        "---\nid: U-001\ntitle: Unit\nobject: unit\n---\n# U-001: Unit\n\n"
        f"## {section}\n\n{table}"
    )
    feature = "dimension" if columns == ["Base"] else "terms" if "Exponent" in columns else "unit"
    request["bodyExtraction"]["yield_pattern"]["match"] = {
        feature: {
            "from": "table_row",
            "under_section": section,
            "assert": {
                "columns": columns,
                "optional_columns": ["Target"] if "Target" in columns else [],
            },
        }
    }
    request["bundle"]["objects"] = list(objects)
    request["bundle"]["artifacts"] = list(artifacts)
    return request


@pytest.mark.trace("FR-075-AC-16")
def test_physical_dimension_record_matches_rust_behavior():
    request = _physical_request("Basis", ["Base"], [["false"]])
    record = quire.extract_semantic(request)
    assert record["availability"]["model"]["state"] == "available"
    assert record["model"]["dimension"]["base"] is False


@pytest.mark.trace("FR-075-AC-17")
def test_physical_terms_alias_duplicate_matches_rust_behavior():
    objects = [
        {"id": "D1", "names": ["Length"]},
    ]
    artifacts = [{"id": "D1", "object": "dimension", "operations": []}]
    request = _physical_request(
        "Powers",
        ["Dimension", "Exponent"],
        [["Length", "1"], ["D1", "2"]],
        objects,
        artifacts,
    )
    record = quire.extract_semantic(request)
    assert record["availability"]["model"]["state"] == "unavailable"
    assert "model" not in record
    duplicate = next(
        d for d in record["diagnostics"] if d["code"] == "semantic.duplicate-model-entry"
    )
    assert duplicate["line"] == 13


@pytest.mark.trace("FR-075-AC-17")
def test_physical_terms_large_exponent_matches_rust_behavior():
    exponent = "9223372036854775808"
    request = _physical_request(
        "Powers",
        ["Dimension", "Exponent"],
        [["Length", exponent]],
        [{"id": "Length", "names": ["Length"]}],
        [{"id": "Length", "object": "dimension", "operations": []}],
    )
    record = quire.extract_semantic(request)
    assert record["availability"]["model"]["state"] == "available"
    assert record["model"]["terms"][0]["exponent"] == int(exponent)


@pytest.mark.trace("FR-075-AC-18")
def test_physical_unit_large_canonical_rational_matches_rust_behavior():
    value = "100000000000000000000000000000000000001/3"
    request = _physical_request(
        "Scale", ["Dimension", "Target", "Scale", "Offset"], [["Time", "ms", value, "0"]],
        [{"id": "Time", "names": ["Time"]}, {"id": "ms", "names": ["ms"]}],
        [
            {"id": "Time", "object": "dimension", "operations": []},
            {"id": "ms", "object": "unit", "operations": []},
        ],
    )
    record = quire.extract_semantic(request)
    assert record["availability"]["model"]["state"] == "available"
    assert record["model"]["unit"]["scale"] == value


@pytest.mark.trace("FR-075-AC-19")
def test_undeclared_physical_table_refusal_matches_rust_behavior():
    request = _physical_request("Basis", ["Base"], [["true"]])
    request["bodyExtraction"]["yield_pattern"]["match"] = {}
    record = quire.extract_semantic(request)
    assert record["availability"]["model"]["state"] == "unavailable"
    assert any(
        d["code"] == "semantic.feature-not-extractable" and d["reason"] == "dimension"
        for d in record["diagnostics"]
    )


@pytest.mark.trace("FR-072-AC-6")
def test_extract_filament_core_carries_the_semantic_record():
    """the Filament API payload is additive and equals the record."""
    entity = json.loads((FIXTURE_CRATE / "module-ok" / "schemas" / "Entity.json").read_text())
    markdown = (FIXTURE_CRATE / "mapping" / "config-version.table.md").read_text()
    bundle = json.loads((FIXTURES / "config-version.bundle.json").read_text())
    result = quire.extract_filament_core(
        {
            "projectId": "p",
            "documentId": "d",
            "artifactId": "a",
            "relPath": "spec/functional/FR-006.md",
            "repoName": "config-service",
            "org": "agent-ix",
            "markdown": markdown,
            "semanticBundle": bundle,
            "objectTypes": [
                {
                    "name": "entity",
                    "dataSchema": entity,
                    "allowedLinks": {},
                    "bodyExtraction": {
                        "yield_pattern": {
                            "match": {
                                "id": {"from": "frontmatter_field", "path": ["id"], "required": True},
                                "properties": {"from": "section_body", "after_heading": "Properties", "required": True},
                            }
                        }
                    },
                    "hasPlugin": False,
                    "moduleId": "spec-objects-fixture",
                    "semantic": {
                        "contractVersion": "1.0.0",
                        "package": "agent-ix/spec-objects-fixture",
                        "exports": ["entity"],
                        "imports": {},
                    },
                }
            ],
        }
    )
    node = next(n for n in result["nodes"] if n["objectType"] == "entity")
    data = json.loads(node["dataJson"])
    assert data["semantic"] == _expected()["golden-table-available"]
    assert data["semantic"]["formatVersion"] == 1
    assert len(data["semantic"]["fields"]) == 7
    assert data["semantic"]["availability"]["fields"]["state"] == "available"
    assert all(d["severity"] in {"info", "warning", "error"} for d in result["diagnostics"])


def test_unsupported_contract_version_is_refused():
    """FR-072-AC-4 through the binding: no node for the refused snapshot."""
    result = quire.extract_filament_core(
        {
            "projectId": "p",
            "documentId": "d",
            "relPath": "spec/FR-1.md",
            "repoName": "r",
            "org": "agent-ix",
            "markdown": "---\nid: FR-1\nobject: entity\n---\n# x\n",
            "objectTypes": [
                {
                    "name": "entity",
                    "dataSchema": {"type": "object"},
                    "allowedLinks": {},
                    "bodyExtraction": None,
                    "hasPlugin": False,
                    "moduleId": "m",
                    "semantic": {"contractVersion": "2.0.0", "package": "agent-ix/x"},
                }
            ],
        }
    )
    assert any(d["code"] == "semantic.unsupported-contract-version" for d in result["diagnostics"])
    assert all(n["objectType"] != "entity" for n in result["nodes"])
