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
MODULES_DIR = REPO_ROOT / "tests" / "render_parity" / "modules"


def test_parse_document_returns_structured_object():
    """TC-461/467: parse returns a native dict, not a JSON string."""
    md = "---\nid: FR-023\nartifact_type: FR\n---\n# Behavior {#blk-1}\n\nbody\n"
    doc = quire.parse_document(md)

    assert isinstance(doc, dict)
    assert doc["frontmatter"]["id"] == "FR-023"
    assert doc["frontmatter"]["artifact_type"] == "FR"
    sections = doc["sections"]
    assert sections[0]["heading"] == "Behavior"
    assert sections[0]["block_id"] == "blk-1"
    assert sections[0]["level"] == 1


def test_parse_document_malformed_frontmatter_is_tolerant():
    """Parser tolerance survives the boundary (no exception)."""
    doc = quire.parse_document("---\nnot: [valid\n---\n# H\n")
    assert doc["frontmatter"] is None  # malformed -> None (FR-006)


def test_load_repo_returns_documents(tmp_path):
    """TC-463: load_repo yields one structured doc per markdown file."""
    (tmp_path / "FR-001.md").write_text(
        "---\nid: FR-001\nartifact_type: FR\nuuid: 0190b6a0-0000-7000-8000-000000000001\n---\n# H\n"
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


def test_gil_released_under_concurrency():
    """NFR-016: two threads each loading the spec run concurrently
    (GIL released during the Rust work), so they don't serialize."""

    def load():
        return len(quire.Spec.from_path(str(SPEC_DIR)))

    with concurrent.futures.ThreadPoolExecutor(max_workers=2) as ex:
        results = list(ex.map(lambda _: load(), range(4)))
    assert all(r >= 50 for r in results)
