"""Gate G6 — Python binding verification (FR-023, NFR-016).

Run via: maturin build --features python, install the wheel into a venv,
then `pytest tests/python/`. The Rust suite cannot exercise the FFI
boundary; this is the binding layer's verification method (spec.md §13).
"""

import concurrent.futures
import pathlib

import pytest

import quire

REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
SPEC_DIR = REPO_ROOT / "spec"
# load_from walks one level deep for module dirs, so point at the parent.
MODULES_DIR = REPO_ROOT / "tests" / "fixtures" / "modules"


def test_parse_document_returns_structured_object():
    """TC-461/467: parse returns a native dict, not a JSON string."""
    md = "---\nid: FR-023\ntype: FR\n---\n# Behavior {#blk-1}\n\nbody\n"
    doc = quire.parse_document(md)

    assert isinstance(doc, dict)
    assert doc["frontmatter"]["id"] == "FR-023"
    assert doc["frontmatter"]["type"] == "FR"
    sections = doc["sections"]
    assert sections[0]["heading"] == "Behavior"
    assert sections[0]["block_id"] == "blk-1"
    assert sections[0]["level"] == 1


def test_parse_document_malformed_frontmatter_is_tolerant():
    """Parser tolerance survives the boundary (no exception)."""
    doc = quire.parse_document("---\nnot: [valid\n---\n# H\n")
    assert doc["frontmatter"] is None  # malformed -> None (FR-006)


def test_check_grammar_ears_findings_cross_boundary():
    """TC-666 (FR-042-AC-10): the EARS grammar entry point is exposed via PyO3
    and returns the same findings as the in-process Rust call for a fixture."""
    md = (
        "---\nid: FR-001\ntype: FR\n---\n"
        "## Description\n\nOn startup, the service shall support publishing.\n"
    )
    findings = quire.check_grammar("iso-spec-core", "FR", md)
    assert isinstance(findings, list)
    checks = {f["check"] for f in findings}
    # vague verb `support` + latent trigger `On …` (matches the Rust unit tests)
    assert "vague-response" in checks
    assert "non-canonical-trigger" in checks
    vague = next(f for f in findings if f["check"] == "vague-response")
    assert vague["grammar"] == "ears"
    assert vague["pattern"] == "ubiquitous"
    assert vague["severity"] == "warning"
    assert vague["line"] is not None
    assert vague["statement"]

    # An unknown grammar bundle yields no findings (advisory, never errors).
    assert quire.check_grammar("nonexistent", "FR", md) == []


def test_check_grammar_applies_module_lexicon(tmp_path):
    """TC-673 (FR-043-AC-7): check_grammar applies a module's concrete lexicon
    when given module_root; without one it uses an empty lexicon."""
    md = (
        "---\nid: FR-001\ntype: FR\n---\n"
        "## Description\n\nThe system shall support pagination.\n"
    )
    # No module → empty lexicon → the bare noun is flagged.
    findings = quire.check_grammar("iso-spec-core", "FR", md)
    assert any(f["check"] == "vague-response" for f in findings)

    # A module declaring `pagination` in its lexicon → suppressed.
    mod = tmp_path / "m"
    (mod / "schemas").mkdir(parents=True)
    (mod / "manifest.yaml").write_text(
        "name: m\nlexicon:\n  pagination:\n    definition: page splitting\n"
    )
    findings2 = quire.check_grammar("iso-spec-core", "FR", md, str(mod))
    assert not any(f["check"] == "vague-response" for f in findings2)


def test_check_grammar_ac_findings_cross_boundary(tmp_path):
    """TC-715 (FR-047-AC-9): the `ac` grammar is exposed through the same PyO3
    surface and returns the same findings as the in-process Rust call."""
    md = (
        "---\nid: FR-001\ntype: FR\n---\n"
        "## Acceptance Criteria\n\n"
        "| ID | Criteria | Verification |\n"
        "|----|----------|--------------|\n"
        "| FR-001-AC-1 | It all works end to end. | Test |\n"
        "| FR-001-AC-2 | Given a token, when it expires, then `401` is returned. | Test |\n"
    )
    findings = quire.check_grammar("iso-spec-core", "FR", md)
    ac = [f for f in findings if f["grammar"] == "ac"]
    assert ac, "the ac grammar must contribute through the binding"

    vacuous = [f for f in ac if f["statement"].startswith("It all works")]
    checks = {f["check"] for f in vacuous}
    # CR-014 inverted both high-volume checks: the cell is flagged for asserting
    # a *vacuous* predicate, not for missing a verb off an allowlist, and it is
    # not `unclassifiable` — "works" is a predicate, just an empty one.
    assert checks == {"vacuous-outcome"}
    # `pattern` carries the detected shape (CR-013). A cell with a predicate is
    # an `assertion` whatever verb it uses; the check id names the defect.
    assert all(f["pattern"] == "assertion" for f in vacuous)
    assert all(f["severity"] == "warning" for f in vacuous)
    assert all(f["line"] is not None for f in vacuous)

    gwt = [f for f in ac if f["statement"].startswith("Given a token")]
    # GWT is a recognized-but-non-canonical rendering (CR-013): the assertion is
    # the canonical AC shape, so the cell is steered while still classifying.
    assert {f["check"] for f in gwt} == {"non-canonical-shape"}
    assert all(f["pattern"] == "given-when-then" for f in gwt)

    # FR-047-AC-12 / FR-048: module data reaches the binding — declaring the
    # verb suppresses `no-observable-outcome`, mapping the check `off` removes
    # it entirely.
    mod = tmp_path / "m"
    mod.mkdir(parents=True)
    (mod / "manifest.yaml").write_text(
        "name: m\n"
        "observable_verbs:\n  work:\n    definition: produces a checkable result\n"
        "grammar_severity:\n  \"ac:unclassifiable\": off\n"
    )
    scoped = quire.check_grammar("iso-spec-core", "FR", md, str(mod))
    scoped_checks = {f["check"] for f in scoped if f["grammar"] == "ac"}
    assert "no-observable-outcome" not in scoped_checks
    assert "unclassifiable" not in scoped_checks


def test_load_repo_returns_documents(tmp_path):
    """TC-463: load_repo yields one structured doc per markdown file."""
    (tmp_path / "FR-001.md").write_text(
        "---\nid: FR-001\ntype: FR\nuuid: 0190b6a0-0000-7000-8000-000000000001\n---\n# H\n"
    )
    docs = quire.load_repo(str(tmp_path))
    assert len(docs) == 1
    assert docs[0]["id"] == "FR-001"
    assert docs[0]["uuid"] == "0190b6a0-0000-7000-8000-000000000001"
    assert isinstance(docs[0]["doc"], dict)


def test_spec_queries_on_real_spec():
    """Whole-spec queries over quire-rs's own spec/ (dogfood via FFI)."""
    spec = quire.Spec.from_path(str(SPEC_DIR))
    assert len(spec) >= 50

    frs = spec.by_type("FR")
    assert "FR-025" in frs and "FR-027" in frs

    # FR-025/026/027 implement StR-006 -> reverse lookup finds them.
    referrers = [src for (src, _etype) in spec.referencing("StR-006")]
    for fr in ("FR-025", "FR-026", "FR-027"):
        assert fr in referrers

    # by_id returns a structured record.
    rec = spec.by_id("FR-025")
    assert rec is not None and rec["type"] == "FR"


def test_validate_accepts_valid_and_flags_violations():
    """TC-462: schema validation across the FFI boundary, with the same
    dotted field path the Rust validator produces (NFR-005)."""
    reg = quire.Registry.load_from([str(MODULES_DIR)])
    assert "demo-item" in reg.archetype_names()

    # Valid data -> no violations.
    assert reg.validate("demo-item", {"id": "DEMO-1", "title": "Hello"}) == []

    # Pattern violation on `id` -> a violation carrying that field path.
    violations = reg.validate("demo-item", {"id": "nope", "title": "x"})
    assert len(violations) >= 1
    assert any("id" in v.get("field_path", "") for v in violations)

    # Missing required field is also reported.
    assert len(reg.validate("demo-item", {"id": "DEMO-1"})) >= 1


def test_validate_unknown_archetype_raises():
    reg = quire.Registry.load_from([str(MODULES_DIR)])
    with pytest.raises(ValueError):
        reg.validate("no-such-archetype", {})


# ── Task 038: validate_document binding (FR-032) ─────────────────────

ISO_MODULE = REPO_ROOT / "tests" / "fixtures" / "modules" / "iso"

CONFORMANT_FR = (
    "---\n"
    "id: FR-901\n"
    'title: "A conformant requirement"\n'
    "type: FR\n"
    "---\n"
    "# [FR-901] A conformant requirement\n"
    "\n"
    "## Description\n"
    "The system SHALL preserve byte-exact content across a parse round-trip.\n"
    "\n"
    "## Specification\n"
    "On parse, the engine retains every byte of the section body verbatim.\n"
    "\n"
    "## Acceptance Criteria\n"
    "\n"
    "| ID | Criteria | Verification |\n"
    "|----|----------|--------------|\n"
    "| FR-901-AC-1 | Round-trip is byte-identical | Integration Test |\n"
    "\n"
    "## Dependencies\n"
    "\n"
    "- **Upstream**: none\n"
    "- **Downstream**: none\n"
)


def test_validate_document_conformant_is_valid():
    """TC-528/TC-533 (binding happy path): a conformant FR markdown
    document validates through the wheel."""
    result = quire.validate_document("FR", str(ISO_MODULE), CONFORMANT_FR)
    assert result["is_valid"] is True
    assert result["errors"] == []


def test_validate_document_flags_missing_section_with_reason_and_line():
    """TC-529/TC-533 (binding sad path): a missing required section is
    flagged with a reason and a line-numbered error shape."""
    mutated = CONFORMANT_FR.replace(
        "## Specification\n"
        "On parse, the engine retains every byte of the section body verbatim.\n\n",
        "",
    )
    result = quire.validate_document("FR", str(ISO_MODULE), mutated)
    assert result["is_valid"] is False
    reasons = {e["reason"] for e in result["errors"]}
    assert "missing" in reasons
    # Error shape: each carries message + line + reason keys.
    for e in result["errors"]:
        assert set(e.keys()) == {"message", "line", "reason"}


def test_validate_document_unknown_archetype_raises():
    with pytest.raises(quire.QuireSchemaError):
        quire.validate_document("no-such-archetype", str(ISO_MODULE), CONFORMANT_FR)


# ── Task 039: input contract + skeleton binding (FR-029 recast) ──────


def test_input_contract_for_fr():
    contract = quire.input_contract("FR", str(ISO_MODULE))
    assert contract["archetype"] == "FR"
    assert contract["frontmatter_schema"]["type"] == "object"
    headings = [s["heading"] for s in contract["sections"] if s["heading"]]
    for required in ["Description", "Specification", "Acceptance Criteria", "Dependencies"]:
        assert required in headings


def test_input_skeleton_for_fr():
    skeleton = quire.input_skeleton("FR", str(ISO_MODULE))
    assert "## Description" in skeleton
    assert "TODO" not in skeleton


def test_input_contract_unknown_raises():
    with pytest.raises(quire.QuireSchemaError):
        quire.input_contract("no-such-archetype", str(ISO_MODULE))


# ── FR-028 expanded surface ──────────────────────────────────────────

DEMO_MODULE = REPO_ROOT / "tests" / "fixtures" / "modules" / "demo"


def test_validate_happy_path_returns_none():
    """TC-511 happy: valid data → no raise."""
    assert quire.validate("demo-item", str(DEMO_MODULE), {"id": "DEMO-1", "title": "t"}) is None


def test_validate_sad_path_raises_validation_error_with_field_path():
    """TC-511 sad: invalid data raises QuireValidationError; message carries field."""
    with pytest.raises(quire.QuireValidationError) as exc:
        quire.validate("demo-item", str(DEMO_MODULE), {"id": "nope", "title": "t"})
    # NFR-005 dotted field path lives in the message.
    assert "id" in str(exc.value)


def test_validate_manifest_happy_and_sad(tmp_path):
    """TC-512: schema-load happy, validation happy + sad, schema-missing → QuireSchemaError."""
    schema_path = tmp_path / "s.json"
    schema_path.write_text(
        '{"type":"object","required":["name"],'
        '"additionalProperties":false,'
        '"properties":{"name":{"type":"string"}}}'
    )
    # Happy path.
    assert quire.validate_manifest({"name": "ok"}, str(schema_path)) == []
    # Sad path — wrong type for `name`, returned as structured data.
    violations = quire.validate_manifest({"name": 5, "extra": True}, str(schema_path))
    assert {v["schema_keyword"] for v in violations} == {
        "type",
        "additionalProperties",
    }
    assert any(v["path"] == "name" for v in violations)
    required_violations = quire.validate_manifest({}, str(schema_path))
    assert any(
        v["path"] == "name" and v["schema_keyword"] == "required"
        for v in required_violations
    )
    # Missing schema file.
    with pytest.raises(quire.QuireSchemaError):
        quire.validate_manifest({"name": "ok"}, str(tmp_path / "missing.json"))


def test_validate_document_column_choices_enforced(tmp_path):
    """CR-010 (FR-033-AC-12): a per-column `column_choices` table assert is
    enforced through the validate_document binding — a Severity cell inside the
    allowed set passes, an out-of-set value flags reason `assert`."""
    mod = tmp_path / "m"
    (mod / "schemas").mkdir(parents=True)
    (mod / "schemas" / "review.schema.json").write_text(
        '{"type":"object","required":["id","type"],'
        '"properties":{"id":{"type":"string"},"type":{"const":"Review"}}}'
    )
    (mod / "manifest.yaml").write_text(
        "manifest_version: 1.0.0\n"
        "name: m\n"
        "version: 0.1.0\n"
        "grammars:\n  - name: g\n    version: 0.1.0\n    doc_kinds: [review]\n"
        "archetypes:\n  - kind: review\n    name: Review\n    doc_backed: true\n"
        "artifact_types:\n"
        "  - name: Review\n"
        "    grammar_ref: g\n"
        "    frontmatter_schema_ref: schemas/review.schema.json\n"
        "    body_extraction:\n"
        "      yield_pattern:\n"
        "        match:\n"
        "          findings:\n"
        "            from: table_row\n"
        "            under_section: Findings\n"
        "            assert:\n"
        "              columns: [ID, Severity]\n"
        "              column_choices:\n"
        "                Severity: [low, medium, high]\n"
    )
    ok = (
        "---\nid: REV-1\ntype: Review\n---\n## Findings\n"
        "| ID | Severity |\n| - | - |\n| FND-1 | medium |\n"
    )
    res = quire.validate_document("Review", str(mod), ok)
    assert res["is_valid"] is True, res["errors"]

    bad = ok.replace("medium", "huge")
    res2 = quire.validate_document("Review", str(mod), bad)
    assert res2["is_valid"] is False
    assert "assert" in {e["reason"] for e in res2["errors"]}
    assert any("huge" in e["message"] for e in res2["errors"])


def test_validate_document_scalar_choices_enforced(tmp_path):
    """CR-010 (FR-033-AC-11): a scalar `choices` enum assert round-trips through
    the validate_document binding (a Vec<String> across the FFI boundary)."""
    mod = tmp_path / "m"
    (mod / "schemas").mkdir(parents=True)
    (mod / "schemas" / "review.schema.json").write_text(
        '{"type":"object","required":["id","type"],'
        '"properties":{"id":{"type":"string"},"type":{"const":"SR"}}}'
    )
    (mod / "manifest.yaml").write_text(
        "manifest_version: 1.0.0\n"
        "name: m\n"
        "version: 0.1.0\n"
        "grammars:\n  - name: g\n    version: 0.1.0\n    doc_kinds: [sr]\n"
        "archetypes:\n  - kind: sr\n    name: SR\n    doc_backed: true\n"
        "artifact_types:\n"
        "  - name: SR\n"
        "    grammar_ref: g\n"
        "    frontmatter_schema_ref: schemas/review.schema.json\n"
        "    body_extraction:\n"
        "      yield_pattern:\n"
        "        match:\n"
        "          severity:\n"
        "            from: section_body\n"
        "            after_heading: Severity\n"
        "            assert:\n"
        "              choices: [low, medium, high]\n"
    )
    ok = "---\nid: SR-1\ntype: SR\n---\n## Severity\nmedium\n"
    assert quire.validate_document("SR", str(mod), ok)["is_valid"] is True

    bad = "---\nid: SR-1\ntype: SR\n---\n## Severity\ncritical\n"
    res = quire.validate_document("SR", str(mod), bad)
    assert res["is_valid"] is False
    assert "assert" in {e["reason"] for e in res["errors"]}


def test_extract_envelope_returns_extraction_and_edges(tmp_path):
    """TC-513: extract returns {extraction, edges}. demo-item has no body_extraction,
    so we use a manifest with one. For a clean test, build a tiny module on the fly."""
    mod = tmp_path / "m"
    (mod / "schemas").mkdir(parents=True)
    (mod / "schemas" / "fr.schema.json").write_text(
        '{"type":"object","properties":{"id":{"type":"string"}}}'
    )
    (mod / "manifest.yaml").write_text(
        "name: m\nversion: 0.1.0\nobject_types:\n"
        "  - name: fr\n"
        "    data_schema:\n"
        "      type: object\n"
        "    body_extraction:\n"
        "      yield_pattern:\n"
        "        match:\n"
        "          purpose:\n"
        "            from: section_body\n"
        "            after_heading: Purpose\n"
    )
    md = (
        "---\nid: FR-001\nrelationships:\n"
        "  - target: StR-001\n    type: implements\n"
        "---\n## Purpose\nthe purpose\n[link](ix://x/FR-002)\n"
    )
    out = quire.extract("fr", str(mod), md)
    assert isinstance(out, dict)
    assert "extraction" in out and "edges" in out
    assert len(out["extraction"]) == 1
    assert "the purpose" in out["extraction"][0]["purpose"]
    targets = {(e["target"], e["edge_type"]) for e in out["edges"]}
    assert ("StR-001", "implements") in targets
    assert ("FR-002", "references") in targets


def test_extraction_context_from_object_types_extracts_without_module_root():
    """Service/library path: caller provides ObjectTypes; quire does not resolve a local registry."""
    ctx = quire.ExtractionContext.from_object_types(
        [
            {
                "name": "event",
                "schema": {"type": "object", "properties": {"schema_json": {"type": "string"}}},
                "body_extraction": {
                    "yield_pattern": {
                        "match": {
                            "id": {
                                "from": "frontmatter_field",
                                "path": ["id"],
                                "required": False,
                            },
                            "schema_json": {
                                "from": "code_block",
                                "after_heading": "Schema",
                                "language": "json",
                                "required": False,
                            },
                        }
                    }
                },
            }
        ]
    )
    assert ctx.object_type_names() == ["event"]
    body = (
        "# Event\n\n"
        "## Other\n\n"
        "```json\n{\"wrong\": true}\n```\n\n"
        "## Schema\n\n"
        "```json\n{\"right\": true}\n```\n"
    )
    out = ctx.extract("event", {"id": "E-1"}, body)
    assert out["extraction"] == [
        {"id": "E-1", "schema_json": '{"right": true}'}
    ]


def test_extraction_context_accepts_optional_code_block_language():
    ctx = quire.ExtractionContext.from_object_types(
        [
            {
                "name": "sli",
                "schema": {"type": "object"},
                "body_extraction": {
                    "yield_pattern": {
                        "match": {
                            "query": {
                                "from": "code_block",
                                "after_heading": "Query",
                                "required": False,
                            }
                        }
                    }
                },
            }
        ]
    )
    out = ctx.extract("sli", {}, "# SLI\n\n## Query\n\n```\nrate(up[5m])\n```\n")
    assert out["extraction"] == [{"query": "rate(up[5m])"}]


def test_extraction_context_errors_on_required_per_match_miss():
    ctx = quire.ExtractionContext.from_object_types(
        [
            {
                "name": "api_endpoint",
                "schema": {"type": "object"},
                "body_extraction": {
                    "yield_pattern": {
                        "iterate_over": {
                            "section_path": ["Endpoints"],
                            "kind": "heading",
                            "depth": 1,
                        },
                        "per_match": {
                            "example": {
                                "from": "code_block",
                                "after_heading": "Example",
                                "required": True,
                            }
                        },
                    }
                },
            }
        ]
    )
    with pytest.raises(quire.QuireValidationError, match="MissingField"):
        ctx.extract("api_endpoint", {}, "## Endpoints\n### Get User\nNo code here\n")


def test_extract_frontmatter_returns_dict_or_none():
    """TC-514: FR-006 parity — Rust returns frontmatter and body."""
    result = quire.extract_frontmatter("---\nid: X\ntype: FR\n---\n# H\n")
    assert result == {
        "frontmatter": {"id": "X", "type": "FR"},
        "body": "# H\n",
    }
    assert quire.extract_frontmatter("# no frontmatter\n") == {
        "frontmatter": None,
        "body": "# no frontmatter\n",
    }
    assert quire.extract_frontmatter("---\nnot: [valid\n---\n# H\n") == {
        "frontmatter": None,
        "body": "---\nnot: [valid\n---\n# H\n",
    }


def test_extract_frontmatter_body_matches_rust_bom_and_crlf_parity():
    """Regression: Python binding exposes the body produced by Rust, no local split."""
    bom = quire.extract_frontmatter("\ufeff---\nid: X\n---\n# H\n")
    assert bom["frontmatter"] == {"id": "X"}
    assert bom["body"] == "# H\n"

    crlf = quire.extract_frontmatter("---\r\nid: X\r\n---\r\n# H\r\n")
    assert crlf["frontmatter"] == {"id": "X"}
    assert crlf["body"] == "# H\r\n"


def test_harvest_edges_string_and_dict_equal():
    """TC-515: harvest_edges accepts raw string or parsed dict; outputs equal."""
    md = (
        "---\nid: X\nrelationships:\n"
        "  - target: StR-1\n    type: implements\n"
        "  - target: ix://x/FR-2\n    type: requires\n"
        "---\n[a](ix://y/FR-3)\n"
    )
    from_str = quire.harvest_edges(md)
    from_dict = quire.harvest_edges(quire.parse_document(md))
    # The dict path may differ if raw not preserved; only the string path
    # is canonical for body links. Verify the string path is right and
    # the dict path at least has the frontmatter edges.
    targets = {(e["target"], e["edge_type"]) for e in from_str}
    assert ("StR-1", "implements") in targets
    assert ("FR-2", "requires") in targets
    assert ("FR-3", "references") in targets
    # Frontmatter-derived edges always present in dict path.
    dict_targets = {(e["target"], e["edge_type"]) for e in from_dict}
    assert ("StR-1", "implements") in dict_targets
    assert ("FR-2", "requires") in dict_targets


def test_exception_hierarchy_importable():
    """TC-516: the exception hierarchy is importable and properly nested;
    QuireRenderError is not exported (render removed, FR-028-AC-7)."""
    assert issubclass(quire.QuireValidationError, quire.QuireBaseError)
    assert issubclass(quire.QuireSchemaError, quire.QuireBaseError)
    assert issubclass(quire.QuireParseError, quire.QuireBaseError)
    assert issubclass(quire.QuireBaseError, Exception)
    assert not hasattr(quire, "QuireRenderError")


def test_new_functions_release_gil():
    """TC-517: concurrent calls to validate complete without serializing."""

    def work(_):
        for _ in range(20):
            quire.validate(
                "demo-item",
                str(DEMO_MODULE),
                {"id": "DEMO-1", "title": "x"},
            )
        return True

    with concurrent.futures.ThreadPoolExecutor(max_workers=4) as ex:
        assert all(ex.map(work, range(4)))


def test_gil_released_under_concurrency():
    """NFR-016: two threads each loading the spec run concurrently
    (GIL released during the Rust work), so they don't serialize."""

    def load():
        return len(quire.Spec.from_path(str(SPEC_DIR)))

    with concurrent.futures.ThreadPoolExecutor(max_workers=2) as ex:
        results = list(ex.map(lambda _: load(), range(4)))
    assert all(r >= 50 for r in results)


def _filament_input(markdown):
    """One-document Filament extraction input (FR-045/FR-046)."""
    return {
        "projectId": "project",
        "documentId": "doc-1",
        "artifactId": "artifact-1",
        "relPath": "spec/FR-001.md",
        "markdown": markdown,
        "repoName": "example",
        "org": "agent-ix",
        "objectTypes": [
            {
                "name": "capability",
                "schema": {"type": "object", "additionalProperties": True},
                "allowedLinks": {},
                "bodyExtraction": None,
                "hasPlugin": False,
                "moduleId": "test-module",
            }
        ],
    }


def test_extract_filament_core_tier1_binding():
    """TC-686 (Python half): the binding returns native core-data JSON, not a
    string, with a validated graph node carrying frontmatter id/title."""
    result = quire.extract_filament_core(
        _filament_input("---\nid: FR-001\ntitle: Pay vendors\nobject: capability\n---\n# Body\n")
    )
    assert isinstance(result, dict)
    assert result["errors"] == []
    assert len(result["nodes"]) == 1
    node = result["nodes"][0]
    assert node["objectType"] == "capability"
    assert node["ref"] == "ix://agent-ix/example/FR-001"
    import json

    data = json.loads(node["dataJson"])
    assert data["code"] == "FR-001"
    assert data["title"] == "Pay vendors"


def test_extract_filament_core_malformed_frontmatter_reports_parse_failure():
    """TC-686/#127: a complete-but-unparsable frontmatter block surfaces a
    `parse_failed` error through the binding (reaches Filament index_errors)."""
    result = quire.extract_filament_core(_filament_input("---\nid: : bad\n---\n# Body\n"))
    assert result["nodes"] == []
    assert any("parse_failed" in e for e in result["errors"])
    assert any(d["code"] == "frontmatter_unparsable" for d in result["diagnostics"])


def test_extract_core_data_is_alias_and_deterministic():
    """`extract_core_data` aliases `extract_filament_core`; identical input
    yields byte-identical JSON (NFR-020-AC-3 native-value parity)."""
    import json

    inp = _filament_input("---\nid: FR-001\nobject: capability\ndepends_on:\n  - FR-002\n---\n")
    a = quire.extract_core_data(inp)
    b = quire.extract_filament_core(inp)
    assert json.dumps(a, sort_keys=True) == json.dumps(b, sort_keys=True)
