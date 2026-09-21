"""FR-072 Python binding parity (TC-1635, TC-1644 Python half).

`quire.extract_semantic` and the additive `semantic` payload of
`quire.extract_filament_core` must equal the Rust output recorded in
`tests/fixtures/semantic/cases.expected.json` for every case of
`tests/fixtures/semantic/cases.json`, as JSON values.
"""

import json
from pathlib import Path

import quire

FIXTURES = Path(__file__).resolve().parents[1] / "fixtures" / "semantic"


def _cases():
    return json.loads((FIXTURES / "cases.json").read_text())["cases"]


def _expected():
    return json.loads((FIXTURES / "cases.expected.json").read_text())


# The digest the registry owner recorded over Entity.json (FR-069: Quire
# passes it through, never minting a second one).
SNAPSHOT_DIGEST = "sha256:8692992e186f40a73b78fd1b0915f0fe78e05a2b00782fb1df58c951a37c91d5"


def test_extract_semantic_matches_rust_for_every_case():
    """TC-1635: binding output equals the Rust oracle, case by case."""
    expected = _expected()
    for case in _cases():
        result = quire.extract_semantic(case["input"])
        assert isinstance(result, dict), case["name"]
        assert result == expected[case["name"]], case["name"]


def test_extract_semantic_is_deterministic():
    """TC-1644 (Python half): repeated runs agree including diagnostic order."""
    for case in _cases():
        first = quire.extract_semantic(case["input"])
        second = quire.extract_semantic(case["input"])
        assert first == second, case["name"]
        assert [d["code"] for d in first["diagnostics"]] == [d["code"] for d in second["diagnostics"]]


def test_extract_filament_core_carries_the_semantic_record():
    """TC-1635: the Filament API payload is additive and equals the record."""
    entity = json.loads((FIXTURES / "module-ok" / "schemas" / "Entity.json").read_text())
    markdown = (FIXTURES / "mapping" / "overlay.table.md").read_text()
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
                    "moduleId": "config-service-fixture",
                    "semantic": {
                        "contractVersion": "1.0.0",
                        "semanticCore": "0.1.0",
                        "package": "agent-ix/config-service-fixture",
                        "exports": ["entity"],
                        "imports": {},
                        "schemaDigest": SNAPSHOT_DIGEST,
                    },
                }
            ],
        }
    )
    node = next(n for n in result["nodes"] if n["objectType"] == "entity")
    data = json.loads(node["dataJson"])
    # The Filament record equals the library record for the same inputs.
    expected = quire.extract_semantic(
        {
            "markdown": markdown,
            "module": {
                "contractVersion": "1.0.0",
                "semanticCore": "0.1.0",
                "package": "agent-ix/config-service-fixture",
                "exports": ["entity"],
                "imports": {},
            },
            "path": "spec/functional/FR-006.md",
            "sourceIdentity": "ix://agent-ix/config-service/spec",
            "bundle": bundle,
            "schemaDigest": SNAPSHOT_DIGEST,
        }
    )
    assert data["semantic"] == expected
    assert data["semantic"]["formatVersion"] == 1
    assert len(data["semantic"]["fields"]) == 7
    assert data["semantic"]["availability"]["fields"]["state"] == "available"
    assert all(d["severity"] in {"info", "warning", "error"} for d in result["diagnostics"])


def test_extract_semantic_refuses_unsupported_versions():
    """FR-069 through the binding entry: refused before any record."""
    import pytest

    case = _cases()[0]["input"]
    bad = dict(case, module=dict(case["module"], semanticCore="9.9.9"))
    with pytest.raises(TypeError, match="semantic.unsupported-semantic-core"):
        quire.extract_semantic(bad)


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
                    "semantic": {"contractVersion": "2.0.0", "semanticCore": "0.1.0", "package": "agent-ix/x"},
                }
            ],
        }
    )
    assert any(d["code"] == "semantic.unsupported-contract-version" for d in result["diagnostics"])
    assert all(n["objectType"] != "entity" for n in result["nodes"])
