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
def test_two_python_threads_stress_load_repo_under_tsan(tmp_path):
    """Concurrent repository loads exercise the Rust walk under TSAN."""
    root = tmp_path / "spec"
    root.mkdir()
    for index in range(128):
        body = "\n".join(
            f"Line {line}: concurrent load_repo input {index}." for line in range(300)
        )
        (root / f"FR-{index:03}.md").write_text(
            f"---\nid: FR-{index:03}\ntype: FR\n---\n# Requirement {index}\n\n{body}\n"
        )

    barrier = threading.Barrier(2)

    def load_repo_concurrently():
        barrier.wait()
        return quire.load_repo(str(root))

    with concurrent.futures.ThreadPoolExecutor(max_workers=2) as executor:
        futures = [
            executor.submit(load_repo_concurrently)
            for _ in range(2)
        ]
        left, right = [future.result() for future in futures]

    assert len(left) == 128
    assert len(right) == 128
    assert left == right
