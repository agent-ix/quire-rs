"""Focused FR-028 GIL-release regression test for extract_frontmatter."""

import concurrent.futures
import threading
import time

import pytest

import quire


@pytest.mark.trace("FR-028-AC-8")
def test_extract_frontmatter_releases_gil_for_two_threads():
    """Two Rust frontmatter parses overlap instead of serializing under the GIL."""
    fields = "".join(f"field_{index}: value\n" for index in range(20_000))
    markdown = f"---\n{fields}---\nbody\n"
    iterations = 4

    def operation():
        return quire.extract_frontmatter(markdown)

    started = time.perf_counter()
    for _ in range(iterations):
        operation()
    single_worker_elapsed = time.perf_counter() - started

    barrier = threading.Barrier(2)

    def concurrent_worker():
        barrier.wait()
        for _ in range(iterations):
            operation()

    started = time.perf_counter()
    with concurrent.futures.ThreadPoolExecutor(max_workers=2) as executor:
        futures = [executor.submit(concurrent_worker) for _ in range(2)]
        for future in futures:
            future.result()
    concurrent_elapsed = time.perf_counter() - started

    assert concurrent_elapsed < 1.8 * single_worker_elapsed, (
        f"extract_frontmatter serialized: two-worker {concurrent_elapsed:.4f}s, "
        f"one-worker {single_worker_elapsed:.4f}s"
    )
