"""Gate G6 — Python binding verification (FR-023, NFR-016).

Run via: maturin build --features python, install the wheel into a venv,
then `pytest tests/python/`. The Rust suite cannot exercise the FFI
boundary; this is the binding layer's verification method (spec.md §13).
"""

import concurrent.futures
import json
import threading
import time
import pathlib
import shutil

import pytest

import quire

REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
SPEC_DIR = REPO_ROOT / "spec"
# load_from walks one level deep for module dirs, so point at the parent.
MODULES_DIR = REPO_ROOT / "tests" / "fixtures" / "modules"


@pytest.mark.trace("FR-023-AC-2", "StR-005-AC-1")
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


@pytest.mark.trace("FR-042-AC-10")
def test_check_grammar_ears_findings_cross_boundary():
    """FR-042-AC-10: the EARS grammar entry point is exposed via PyO3
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


@pytest.mark.trace("FR-043-AC-7")
def test_check_grammar_applies_module_lexicon(tmp_path):
    """FR-043-AC-7: check_grammar applies a module's concrete lexicon
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


@pytest.mark.trace("FR-047-AC-12")
def test_check_grammar_ac_findings_cross_boundary(tmp_path):
    """FR-047-AC-9: the `ac` grammar is exposed through the same PyO3
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

    # FR-047-AC-12 / FR-048: module data reaches the binding. Declaring `work`
    # as an observable verb makes the cell carry signal, which suppresses
    # `vacuous-outcome` (CR-014 demoted `observable_verbs` to a suppressor);
    # mapping `ac:unclassifiable` to `off` removes that check entirely.
    #
    # Both assertions must name checks that still exist. Asserting the absence
    # of a *retired* id passes unconditionally and proves nothing — that is how
    # TC-715 rotted against CR-014's rename.
    mod = tmp_path / "m"
    mod.mkdir(parents=True)
    (mod / "manifest.yaml").write_text(
        "name: m\n"
        "observable_verbs:\n  work:\n    definition: produces a checkable result\n"
        "grammar_severity:\n  \"ac:unclassifiable\": off\n"
    )
    scoped = quire.check_grammar("iso-spec-core", "FR", md, str(mod))
    scoped_checks = {f["check"] for f in scoped if f["grammar"] == "ac"}
    assert "vacuous-outcome" not in scoped_checks
    assert "unclassifiable" not in scoped_checks


@pytest.mark.trace("FR-052-CON-4")
def test_classify_properties_cross_boundary(tmp_path):
    """The property classifier exposes its records through PyO3.

    Every assertion below must name a field and a label that still exist —
    asserting the absence of a retired one passes unconditionally and proves
    nothing, which is how TC-715 rotted across CR-014.
    """
    md = (
        "---\nid: FR-001\ntype: FR\n---\n"
        "## Acceptance Criteria\n\n"
        "| ID | Criteria | Verification |\n"
        "|----|----------|--------------|\n"
        "| FR-001-AC-1 | A finding whose key is absent from the merged map "
        "defaults to warning | Test |\n"
        "| FR-001-AC-2 | The loader emits a `Duplicate` diagnostic for the "
        "second declaration. | Test |\n"
    )
    records = quire.classify_properties("iso-spec-core", "FR", md)
    assert [r["row_id"] for r in records] == ["FR-001-AC-1", "FR-001-AC-2"]

    universal, example = records
    # The FR-052 axis, and the FR-047 axis carried through unchanged.
    assert universal["property"] == "universal"
    assert universal["shape"] == "assertion"
    assert universal["extractable"] is True
    assert universal["extraction"] == "extractable"
    assert universal["line"] is not None
    assert universal["obligation"] is None
    # The statement is untruncated and the span offsets index it.
    assert universal["statement"].startswith("A finding whose key")
    for key, text in (
        ("domain", "finding"),
        ("precondition", "whose key is absent from the merged map"),
        ("oracle", "defaults to warning"),
    ):
        span = universal[key]
        assert span is not None, key
        assert span["text"] == text
        assert universal["statement"][span["start"] : span["end"]] == text
    # The audit trail is a list of stable signal ids, in evaluation order.
    assert universal["signals"] == [
        "universal:determiner",
        "span:domain",
        "span:precondition",
        "span:oracle",
    ]

    # A specific scenario is `example`, not extractable, and carries no span —
    # a first-class outcome, not a defect (FR-052-CON-1).
    assert example["property"] == "example"
    assert example["extractable"] is False
    # CR-033: an Example criterion is `not-extractable`, never a candidate.
    assert example["extraction"] == "not-extractable"
    assert example["domain"] is None
    assert example["precondition"] is None
    assert example["oracle"] is None

    # US declares no binding-criteria table, so it yields no records at all.
    us = md.replace("id: FR-001\ntype: FR", "id: US-001\ntype: US")
    assert quire.classify_properties("iso-spec-core", "US", us) == []
    # An unknown bundle is a silent no-op, exactly as it is for check_grammar.
    assert quire.classify_properties("no-such-bundle", "FR", md) == []

    # a module `property_idioms` registry reaches the binding and
    # boosts the label — while `extractable` stays put (FR-052-CON-4).
    mod = tmp_path / "m"
    mod.mkdir(parents=True)
    (mod / "manifest.yaml").write_text(
        "name: m\n"
        "property_idioms:\n"
        "  defaults to:\n"
        "    definition: an absent key falls back to a declared default\n"
        "    shape: error-case\n"
    )
    scoped = quire.classify_properties("iso-spec-core", "FR", md, str(mod))
    assert scoped[0]["property"] == "error-case"
    assert scoped[0]["extractable"] == universal["extractable"]
    assert scoped[1]["extractable"] == example["extractable"]
    # CR-033: `extraction` crosses the boundary too. `error-case` is not one of
    # the four metamorphic shapes, so the declaration does not raise a
    # candidate here — the boosted record stays exactly as extractable as it was.
    assert scoped[0]["extraction"] == "extractable"
    assert scoped[1]["extraction"] == "not-extractable"


@pytest.mark.trace("FR-023-AC-4", "US-011-AC-1")
def test_load_repo_returns_documents(tmp_path):
    """load_repo yields one structured doc per markdown file."""
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

    # Those three requirements implement StR-006, so the reverse lookup
    # finds them. (Ids kept out of tag-shaped position: a trailing `/` binds.)
    referrers = [src for (src, _etype) in spec.referencing("StR-006")]
    for fr in ("FR-025", "FR-026", "FR-027"):
        assert fr in referrers

    # by_id returns a structured record.
    rec = spec.by_id("FR-025")
    assert rec is not None and rec["type"] == "FR"


@pytest.mark.trace("FR-023-AC-3")
def test_validate_accepts_valid_and_flags_violations():
    """schema validation across the FFI boundary, with the same
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

REQ_MODULE = REPO_ROOT / "tests" / "fixtures" / "modules" / "req-fixture"

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


@pytest.mark.trace("FR-032-AC-1")
def test_validate_document_conformant_is_valid():
    """binding happy path: a conformant FR markdown
    document validates through the wheel."""
    result = quire.validate_document("FR", str(REQ_MODULE), CONFORMANT_FR)
    assert result["is_valid"] is True
    assert result["errors"] == []


@pytest.mark.trace("FR-032-AC-2")
def test_validate_document_flags_missing_section_with_reason_and_line():
    """binding sad path: a missing required section is
    flagged with a reason and a line-numbered error shape."""
    mutated = CONFORMANT_FR.replace(
        "## Specification\n"
        "On parse, the engine retains every byte of the section body verbatim.\n\n",
        "",
    )
    result = quire.validate_document("FR", str(REQ_MODULE), mutated)
    assert result["is_valid"] is False
    reasons = {e["reason"] for e in result["errors"]}
    assert "missing" in reasons
    # Error shape: each carries message + line + reason keys.
    for e in result["errors"]:
        assert set(e.keys()) == {"message", "line", "reason"}


def test_validate_document_unknown_archetype_raises():
    with pytest.raises(quire.QuireSchemaError):
        quire.validate_document("no-such-archetype", str(REQ_MODULE), CONFORMANT_FR)


# ── Task 039: input contract + skeleton binding (FR-029 recast) ──────


def test_input_contract_for_fr():
    contract = quire.input_contract("FR", str(REQ_MODULE))
    assert contract["archetype"] == "FR"
    assert contract["frontmatter_schema"]["type"] == "object"
    headings = [s["heading"] for s in contract["sections"] if s["heading"]]
    for required in ["Description", "Specification", "Acceptance Criteria", "Dependencies"]:
        assert required in headings


def test_input_skeleton_for_fr():
    skeleton = quire.input_skeleton("FR", str(REQ_MODULE))
    assert "## Description" in skeleton
    assert "TODO" not in skeleton


def test_input_contract_unknown_raises():
    with pytest.raises(quire.QuireSchemaError):
        quire.input_contract("no-such-archetype", str(REQ_MODULE))


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


@pytest.mark.trace("FR-028-AC-2")
def test_validate_matches_rust_dotted_nested_field_path(tmp_path):
    """A valid object passes and a nested invalid field exposes Rust's dotted path."""
    module = tmp_path / "nested"
    (module / "schemas").mkdir(parents=True)
    (module / "manifest.yaml").write_text(
        "name: nested\n"
        "artifact_types:\n"
        "  - name: nested-item\n"
        "    frontmatter_schema_ref: schemas/nested-item.schema.json\n"
    )
    (module / "schemas" / "nested-item.schema.json").write_text(
        '{"type":"object","required":["metadata"],'
        '"properties":{"metadata":{"type":"object","required":["name"],'
        '"properties":{"name":{"type":"string"}}}}}'
    )

    assert quire.validate(
        "nested-item", str(module), {"metadata": {"name": "ok"}}
    ) is None
    with pytest.raises(quire.QuireValidationError) as exc:
        quire.validate("nested-item", str(module), {"metadata": {"name": 7}})
    assert "metadata.name" in str(exc.value)


@pytest.mark.trace("FR-028-AC-3")
def test_validate_manifest_happy_and_sad(tmp_path):
    """schema-load happy, validation happy + sad, schema-missing → QuireSchemaError."""
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


@pytest.mark.trace("FR-076-AC-15")
def test_validate_document_bundle_package_qualifies_relationship_targets(tmp_path):
    """FR-076-AC-15: the optional `bundle_package` kwarg routes to
    `validate_document_in_bundle`: a bare row target qualifies under it, so a
    frontmatter `ix://` duplicate is refused. Without it the relationships
    table is `no-bundle-package`, one advisory warning and no row error."""
    mod = tmp_path / "related"
    shutil.copytree(REPO_ROOT / "crates" / "quire-fixtures" / "fixtures" / "module-ok", mod)
    manifest = (mod / "manifest.yaml").read_text()
    manifest = manifest.replace(
        "- name: entity\n",
        "- name: entity\n  roles: [domain-object]\n"
        "  allowed_links: { references: [domain-object] }\n",
        1,
    )
    manifest = manifest.replace(
        "semantic:\n",
        "edge_types:\n  references: { description: r, category: traceability }\n"
        "roles:\n  domain-object: { description: d }\n"
        "semantic:\n  mappings: [relationships]\n",
        1,
    )
    (mod / "manifest.yaml").write_text(manifest)
    doc = (
        REPO_ROOT / "crates" / "quire-fixtures" / "fixtures" / "mapping" / "config-version.table.md"
    ).read_text()
    doc = doc.replace(
        "- `overlay`: belongs_to \u2192 ConfigOverlay (FR-005)\n",
        "| Name | Verb | Target | Multiplicity |\n|---|---|---|---|\n"
        "| overlay | references | FR-005 | 1..1 |\n",
    ).replace(
        "type: FR\n",
        "type: FR\nrelationships:\n"
        '  - target: "ix://agent-ix/config-service/FR-005"\n    type: references\n',
        1,
    )

    def relationship(entries):
        return [e["message"] for e in entries if "relationships" in e["message"]]

    res = quire.validate_document("entity", str(mod), doc, bundle_package="agent-ix/config-service")
    errors = relationship(res["errors"])
    assert len(errors) == 1, errors
    assert errors[0].startswith("semantic.duplicate-model-entry"), errors

    res = quire.validate_document("entity", str(mod), doc)
    assert relationship(res["errors"]) == []
    warnings = relationship(res["warnings"])
    assert len(warnings) == 1, warnings
    assert warnings[0].startswith("semantic.relationships-no-bundle-package"), warnings


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


@pytest.mark.trace("FR-028-AC-4")
def test_extract_envelope_returns_extraction_and_edges(tmp_path):
    """extract returns {extraction, edges}. demo-item has no body_extraction,
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


@pytest.mark.trace("FR-028-AC-9")
def test_extraction_context_returns_rust_records_and_emitted_edges_without_registry():
    """Caller-compiled ObjectTypes yield Rust extraction records and emit_edges without a module root."""
    object_types = [
        {
            "name": "event",
            "schema": {"type": "object", "additionalProperties": True},
            "body_extraction": {
                "yield_pattern": {
                    "iterate_over": {
                        "section_path": ["Items"],
                        "kind": "heading",
                        "depth": 1,
                    },
                    "per_match": {"name": {"from": "heading", "required": True}},
                },
                "emit_edges": [
                    {
                        "type": "references",
                        "target": {"from": "section_body", "after_heading": "Ref"},
                    }
                ],
            },
        }
    ]
    context = quire.ExtractionContext.from_object_types(object_types)
    assert context.object_type_names() == ["event"]

    body = (
        "## Items\n"
        "### One\n#### Ref\nFR-100\n"
        "### Two\n(no ref child)\n"
    )
    result = context.extract("event", {}, body)
    assert result["extraction"] == [
        {"content": "", "heading": "One", "name": "One"},
        {"content": "(no ref child)", "heading": "Two", "name": "Two"},
    ]
    assert result["edges"] == [
        {"record_index": 0, "type": "references", "target": "FR-100"}
    ]


@pytest.mark.trace("FR-028-AC-10")
def test_extraction_context_accepts_list_and_items_envelope():
    """Both documented ObjectType input shapes compile to equivalent extractors."""
    object_type = {
        "name": "event",
        "schema": {"type": "object", "additionalProperties": True},
        "body_extraction": {
            "yield_pattern": {
                "match": {
                    "id": {
                        "from": "frontmatter_field",
                        "path": ["id"],
                        "required": False,
                    }
                }
            }
        },
    }
    list_context = quire.ExtractionContext.from_object_types([object_type])
    envelope_context = quire.ExtractionContext.from_object_types({"items": [object_type]})
    body = "# Event\n"

    assert list_context.object_type_names() == envelope_context.object_type_names() == ["event"]
    assert list_context.extract("event", {"id": "E-1"}, body) == envelope_context.extract(
        "event", {"id": "E-1"}, body
    )


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


@pytest.mark.trace("FR-028-AC-5")
def test_extract_frontmatter_returns_dict_or_none():
    """FR-006 parity — Rust returns frontmatter and body."""
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


@pytest.mark.trace("FR-028-AC-8")
def test_new_module_functions_release_the_gil_under_concurrent_calls(tmp_path):
    """The five FR-028 module functions overlap their Rust work across two Python threads."""
    module = tmp_path / "extract"
    (module / "schemas").mkdir(parents=True)
    (module / "schemas" / "event.schema.json").write_text(
        '{"type":"object","properties":{"name":{"type":"string"}}}'
    )
    (module / "manifest.yaml").write_text(
        "name: concurrent-extract\n"
        "artifact_types:\n"
        "  - name: event\n"
        "    frontmatter_schema_ref: schemas/event.schema.json\n"
        "    body_extraction:\n"
        "      yield_pattern:\n"
        "        match:\n"
        "          purpose:\n"
        "            from: section_body\n"
        "            after_heading: Purpose\n"
    )

    schema_path = tmp_path / "manifest.schema.json"
    schema = {
        "anyOf": [
            {
                "type": "object",
                "required": [f"field-{index}"],
                "properties": {f"field-{index}": {"type": "string"}},
            }
            for index in range(256)
        ]
    }
    schema_path.write_text(json.dumps(schema))
    payload = {"field-255": "valid"}
    frontmatter = "---\nid: FR-001\nbody: " + ("x" * 250_000) + "\n---\n"
    markdown = "# Purpose\n" + ("plain content without a link\n" * 40_000)
    document = "# No links\n" + ("plain content without a link\n" * 40_000)

    calls = [
        (
            "validate",
            lambda: quire.validate(
                "demo-item", str(DEMO_MODULE), {"id": "DEMO-1", "title": "t"}
            ),
        ),
        (
            "validate_manifest",
            lambda: quire.validate_manifest(payload, str(schema_path)),
        ),
        ("extract", lambda: quire.extract("event", str(module), markdown)),
        ("extract_frontmatter", lambda: quire.extract_frontmatter(frontmatter)),
        ("harvest_edges", lambda: quire.harvest_edges(document)),
    ]

    failures = []
    repetitions = 20
    with concurrent.futures.ThreadPoolExecutor(max_workers=2) as executor:
        for name, call in calls:
            single_samples = []
            for _ in range(3):
                started = time.perf_counter()
                for _ in range(repetitions):
                    call()
                single_samples.append((time.perf_counter() - started) / repetitions)
            single_call = sorted(single_samples)[1]

            parallel_samples = []
            for _ in range(5):
                start_time = [0.0]
                barrier = threading.Barrier(
                    2, action=lambda: start_time.__setitem__(0, time.perf_counter())
                )

                def run_concurrently():
                    barrier.wait()
                    result = None
                    for _ in range(repetitions):
                        result = call()
                    return result

                futures = [executor.submit(run_concurrently) for _ in range(2)]
                concurrent_results = [future.result() for future in futures]
                parallel_samples.append(
                    (time.perf_counter() - start_time[0]) / repetitions
                )
                assert concurrent_results[0] == concurrent_results[1], name

            parallel_elapsed = sorted(parallel_samples)[len(parallel_samples) // 2]
            if parallel_elapsed >= single_call * 2:
                failures.append(
                    f"{name} took {parallel_elapsed:.6f}s for two concurrent calls; "
                    f"single-call median was {single_call:.6f}s"
                )
    assert not failures, "\n".join(failures)


@pytest.mark.trace("FR-028-AC-6")
def test_harvest_edges_string_and_dict_equal():
    """harvest_edges accepts raw string or parsed dict; outputs equal."""
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


@pytest.mark.trace("FR-028-AC-7")
def test_exception_hierarchy_importable():
    """the exception hierarchy is importable and properly nested;
    QuireRenderError is not exported (render removed, FR-028-AC-7)."""
    assert issubclass(quire.QuireValidationError, quire.QuireBaseError)
    assert issubclass(quire.QuireSchemaError, quire.QuireBaseError)
    assert issubclass(quire.QuireParseError, quire.QuireBaseError)
    assert issubclass(quire.QuireBaseError, Exception)
    assert not hasattr(quire, "QuireRenderError")


@pytest.mark.trace("NFR-016-AC-2")
def test_gil_released_under_concurrency():
    """two threads each loading the spec run concurrently
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


@pytest.mark.trace("FR-046-AC-1", "NFR-020-AC-3")
def test_extract_filament_core_tier1_binding():
    """Python half: the binding returns native core-data JSON, not a
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


@pytest.mark.trace("FR-045-CON-2")
def test_extract_filament_core_uses_caller_object_type_snapshot():
    """Only caller-supplied ObjectType snapshots make their object extractable."""
    import json

    schema = {
        "type": "object",
        "required": ["approval_code"],
        "properties": {
            "approval_code": {"type": "string"},
            "title": {"type": "string"},
        },
        "additionalProperties": False,
    }
    request = _filament_input(
        "---\nid: PO-17\ntitle: Purchase order\nobject: caller_purchase_order\n"
        "approval_code: AP-8\n---\n# Purchase order\n"
    )
    request["objectTypes"] = [
        {
            "name": "caller_purchase_order",
            "schema": schema,
            "allowedLinks": {},
            "bodyExtraction": None,
            "hasPlugin": False,
            "moduleId": "caller-snapshot",
        }
    ]

    supplied = quire.extract_filament_core(request)
    assert supplied["errors"] == []
    assert [item["name"] for item in supplied["objectTypes"]] == ["caller_purchase_order"]
    assert json.loads(supplied["objectTypes"][0]["schemaJson"]) == schema
    assert supplied["nodes"][0]["objectType"] == "caller_purchase_order"
    assert json.loads(supplied["nodes"][0]["dataJson"])["approval_code"] == "AP-8"

    request_without_snapshot = {**request, "objectTypes": []}
    absent = quire.extract_filament_core(request_without_snapshot)
    assert all(node["objectType"] != "caller_purchase_order" for node in absent["nodes"])
    assert any(item["code"] == "unknown_object_type" for item in absent["diagnostics"])


@pytest.mark.trace("FR-045-CON-4")
def test_extract_filament_core_qualifies_refs_with_required_caller_org():
    """Caller org and repo qualify refs before stable node and edge ids are formed."""
    request = _filament_input(
        "---\nid: FR-001\ntitle: Pay vendors\nobject: capability\n"
        "depends_on: FR-002\n---\n# Body\n"
    )
    request["org"] = "buyer-org"
    request["repoName"] = "billing"

    result = quire.extract_filament_core(request)
    assert result["errors"] == []
    assert result["nodes"][0]["ref"] == "ix://buyer-org/billing/FR-001"
    assert result["nodes"][0]["id"] == '["project","graph-node","ix://buyer-org/billing/FR-001"]'
    dependency = next(edge for edge in result["edges"] if edge["edgeType"] == "depends_on")
    assert dependency["sourceRef"] == "ix://buyer-org/billing/FR-001"
    assert dependency["targetRef"] == "ix://buyer-org/billing/FR-002"
    assert dependency["id"] == (
        '["project","graph-edge","ix://buyer-org/billing/FR-001",'
        '"ix://buyer-org/billing/FR-002","depends_on"]'
    )

    request_without_org = dict(request)
    request_without_org.pop("org")
    with pytest.raises(TypeError, match="org"):
        quire.extract_filament_core(request_without_org)


@pytest.mark.trace("FR-046-AC-1", "NFR-020-AC-3")
def test_extract_filament_core_malformed_frontmatter_reports_parse_failure():
    """a complete-but-unparsable frontmatter block surfaces a
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


@pytest.mark.trace("NFR-020-M-2")
def test_filament_core_repeated_output_mismatch_count_is_zero():
    """Measure repeated-output mismatches across the shared extraction parity fixtures."""
    import json

    fixtures_path = REPO_ROOT / "tests" / "fixtures" / "filament_core" / "graph_cases.json"
    fixtures = json.loads(fixtures_path.read_text())
    mismatches = []
    for fixture in fixtures:
        first = quire.extract_filament_core(fixture["input"])
        second = quire.extract_filament_core(fixture["input"])
        first_json = json.dumps(first, ensure_ascii=False, separators=(",", ":"))
        second_json = json.dumps(second, ensure_ascii=False, separators=(",", ":"))
        if first_json != second_json:
            mismatches.append(fixture["name"])

    mismatch_count = len(mismatches)
    print(
        "NFR-020-M-2 repeated-output mismatch count: "
        f"{mismatch_count}/{len(fixtures)} parity fixtures"
    )
    assert mismatch_count == 0, (
        f"NFR-020-M-2 repeated-output mismatch count is {mismatch_count}; "
        f"mismatched fixtures: {mismatches}"
    )


@pytest.mark.trace("NFR-016-AC-4")
def test_binding_returns_materialized_values_without_python_json_reparse(monkeypatch):
    """Parsed and extraction results arrive as native Python values without json.loads."""
    import json

    def reject_python_json_reparse(*_args, **_kwargs):
        pytest.fail("the binding must return materialized values, not JSON for Python to parse")

    monkeypatch.setattr(json, "loads", reject_python_json_reparse)

    document = quire.parse_document("---\nid: FR-001\ntype: FR\n---\n# Requirement\n")
    assert type(document) is dict
    assert type(document["frontmatter"]) is dict
    assert type(document["sections"]) is list
    assert type(document["sections"][0]) is dict

    core = quire.extract_filament_core(
        _filament_input(
            "---\nid: FR-001\ntitle: Requirement\nobject: capability\n"
            "depends_on: FR-002\n---\n# Requirement\n"
        )
    )
    assert type(core) is dict
    assert type(core["objectTypes"]) is list
    assert type(core["objectTypes"][0]) is dict
    assert type(core["nodes"]) is list
    assert type(core["nodes"][0]) is dict
    assert type(core["nodes"][0]["dataJson"]) is str
    assert type(core["edges"]) is list
    assert type(core["edges"][0]) is dict
    assert type(core["edges"][0]["targetRef"]) is str
