"""Focused FR-023 load_repo GIL-release contract test."""

import concurrent.futures
import threading
import time

import pytest

import quire


@pytest.mark.trace("FR-023-AC-5")
def test_concurrent_load_repo_calls_release_the_gil(tmp_path):
    """Two Python threads can run the Rust repository walk concurrently."""
    root = tmp_path / "spec"
    root.mkdir()
    body = "\n".join(
        f"Line {index}: representative requirement prose." for index in range(600)
    )
    for index in range(128):
        (root / f"FR-{index:03}.md").write_text(
            f"---\nid: FR-{index:03}\ntype: FR\n---\n# Requirement {index}\n\n{body}\n"
        )

    def load():
        docs = quire.load_repo(str(root))
        assert len(docs) == 128

    iterations = 3
    started = time.perf_counter()
    for _ in range(iterations):
        load()
    single_worker_elapsed = time.perf_counter() - started

    barrier = threading.Barrier(2)

    def concurrent_worker():
        barrier.wait()
        for _ in range(iterations):
            load()

    started = time.perf_counter()
    with concurrent.futures.ThreadPoolExecutor(max_workers=2) as executor:
        futures = [executor.submit(concurrent_worker) for _ in range(2)]
        for future in futures:
            future.result()
    concurrent_elapsed = time.perf_counter() - started

    assert concurrent_elapsed < 2 * single_worker_elapsed, (
        f"load_repo serialized: two-worker {concurrent_elapsed:.4f}s, "
        f"one-worker {single_worker_elapsed:.4f}s"
    )


@pytest.mark.trace("NFR-018-AC-1")
def test_two_python_threads_stress_parse_document_under_tsan():
    """Concurrent binding parses stay value-stable under ThreadSanitizer."""
    document = (
        "---\nid: FR-018\ntype: FR\n---\n# Requirement\n\n"
        + "## Acceptance\n\n- **AC-1**: a concurrent parse input.\n" * 300
    )
    barrier = threading.Barrier(2)

    def parse_repeatedly():
        barrier.wait()
        parsed = None
        for _ in range(32):
            parsed = quire.parse_document(document)
        return parsed

    with concurrent.futures.ThreadPoolExecutor(max_workers=2) as executor:
        left, right = [future.result() for future in (
            executor.submit(parse_repeatedly),
            executor.submit(parse_repeatedly),
        )]

    assert left == right
