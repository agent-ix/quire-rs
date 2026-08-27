import json
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent.parent))

from export_measurements import build_collection


def test_collection_is_plan_derived_and_keeps_corpus_boundaries():
    manifest = json.loads(
        (pathlib.Path(__file__).resolve().parents[2] / "bench" / "manifest.json").read_text()
    )
    manifest = {"metrics": {"coverage.dead_tags": manifest["metrics"]["coverage.dead_tags"]}}
    observed = {
        "first": {"coverage.dead_tags": 2},
        "second": {"coverage.dead_tags": 0},
    }
    raw = {
        "first": {"identity": "first-sha", "payload": {"untracked_symbols": [1, 2]}},
        "second": {"identity": "second-sha", "payload": {"untracked_symbols": []}},
    }
    collection = build_collection(
        manifest,
        observed,
        raw,
        timestamp="2026-08-27T00:00:00.000Z",
        source_revision="source-sha",
        tool_version="quire test",
        consumer=pathlib.Path("/consumer"),
        module=None,
    )

    assert collection["collectionId"].startswith("quire-bench-20260827000000000-")
    assert [row["dimensions"]["corpus"] for row in collection["observations"]] == [
        "first",
        "second",
    ]
    assert all(row["planId"] == "MP-203" for row in collection["observations"])
    assert all(row["definitionVersion"] == "coverage.dead-tags-v1" for row in collection["observations"])
    assert collection["rawEvidence"] == raw
