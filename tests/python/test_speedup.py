"""TC-456 / StR-005-AC-3 — the binding path is materially faster than
parsing a repo in pure Python.

The spec's reference is the pure-Python `filament_parser` loader. When
that package isn't importable (it lives in a sibling repo), we fall back
to an equivalent pure-Python baseline — read + YAML frontmatter parse +
heading scan per file — which is representative of the per-file work a
Python loader does. The binding (`quire.Spec.from_path`, all-Rust load +
resolve) must beat it by a wide margin.

Run: pytest tests/python/test_speedup.py -s  (the `-s` prints the ratio).
"""

import re
import time

import pytest

import quire

try:  # prefer the real reference if it's checked out
    from filament_parser.loader import load_directory as _filament_load  # type: ignore

    HAVE_FILAMENT = True
except Exception:  # pragma: no cover - depends on sibling repo presence
    HAVE_FILAMENT = False

DOC_COUNT = 400
TARGET_SPEEDUP = 5.0
_HEADING = re.compile(r"^#{1,6}\s+\S", re.MULTILINE)


def _make_corpus(root, n):
    import yaml  # noqa: F401  (ensure available; used in baseline)

    for i in range(n):
        (root / f"FR-{i:04}.md").write_text(
            f"---\nid: FR-{i:04}\ntype: FR\n"
            f"uuid: 0190b6a0-0000-7000-8000-{i:012}\n"
            "relationships:\n"
            '  - target: "StR-001"\n'
            "    type: implements\n"
            "---\n"
            f"# Behavior\n\nProse for document {i}.\n\n"
            "## Acceptance\n\n- **AC-1**: a criterion.\n- **AC-2**: another.\n\n"
            "## Notes\n\n```rust\nfn x() {}\n```\n"
        )


def _python_baseline(root):
    """Read + parse frontmatter (yaml) + scan headings per file, into an
    id-indexed dict — what a naive pure-Python loader does."""
    import yaml

    index = {}
    for path in sorted(root.glob("**/*.md")):
        if path.name in ("README.md", "tests.md"):
            continue
        text = path.read_text(encoding="utf-8")
        fm = None
        if text.startswith("---\n"):
            end = text.find("\n---\n", 4)
            if end != -1:
                try:
                    fm = yaml.safe_load(text[4:end])
                except yaml.YAMLError:
                    fm = None
        headings = _HEADING.findall(text)
        key = (fm or {}).get("id") or path.as_posix()
        index[key] = (fm, len(headings))
    return index


def _time(fn, *args):
    best = float("inf")
    for _ in range(3):
        t0 = time.perf_counter()
        fn(*args)
        best = min(best, time.perf_counter() - t0)
    return best


def test_binding_load_beats_pure_python(tmp_path, capsys):
    _make_corpus(tmp_path, DOC_COUNT)
    root = str(tmp_path)

    py = _time(_python_baseline, tmp_path)
    rs = _time(lambda r: len(quire.Spec.from_path(r)), root)

    # Sanity: both saw the whole corpus.
    assert len(quire.Spec.from_path(root)) == DOC_COUNT

    ratio = py / rs if rs > 0 else float("inf")
    with capsys.disabled():
        print(
            f"\n[TC-456] {DOC_COUNT} docs — python baseline {py*1e3:.1f} ms, "
            f"quire.Spec.from_path {rs*1e3:.1f} ms → {ratio:.1f}x"
        )

    assert ratio >= TARGET_SPEEDUP, (
        f"binding load only {ratio:.1f}x the pure-Python baseline "
        f"(target ≥ {TARGET_SPEEDUP}x)"
    )


@pytest.mark.skipif(not HAVE_FILAMENT, reason="filament_parser not installed")
def test_speedup_vs_real_filament_parser(tmp_path, capsys):  # pragma: no cover
    """When the real reference is present, compare against it directly."""
    _make_corpus(tmp_path, DOC_COUNT)
    root = str(tmp_path)
    import pathlib

    py = _time(lambda: list(_filament_load(pathlib.Path(root))))
    rs = _time(lambda r: len(quire.Spec.from_path(r)), root)
    ratio = py / rs if rs > 0 else float("inf")
    with capsys.disabled():
        print(f"\n[TC-456 vs filament_parser] → {ratio:.1f}x")
    assert ratio >= TARGET_SPEEDUP
