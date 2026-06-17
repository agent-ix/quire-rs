---
id: NFR-016
title: "Python Binding Overhead Bounded; abi3 Stability"
type: NFR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-011"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-023"
    type: "requires"
    cardinality: "1:1"
---

## Statement

The PyO3 binding ([FR-023](../functional/FR-023-python-binding-surface.md)) SHALL add bounded overhead over the equivalent in-crate Rust call:

- **Per-crossing overhead** for a single `quire.parse_document(text)` call SHALL be **< 50 µs** above the cost of the equivalent `quire_rs::parse_document` Rust call, measured on a small (≤ 4 KB) document where the crossing cost is not masked by parse work ([US-011](../usecase/US-011-python-parses-repo-via-bindings.md)-PC-2).
- **GIL release**: binding entry points performing non-trivial Rust work (`load_repo`, corpus construction, large-document parse) SHALL release the GIL for the duration of the Rust computation, so two Python threads each issuing such a call complete in **< 2× single-call wall-clock** ([US-011-AC-5](../usecase/US-011-python-parses-repo-via-bindings.md), [FR-023-AC-5](../functional/FR-023-python-binding-surface.md)).
- **abi3 stability**: wheels SHALL be built against the CPython stable ABI (abi3) such that **one** wheel, built once, imports on at least two different CPython 3.x minor versions without rebuild ([StR-005-AC-5](../stakeholder/StR-005-native-python-bindings.md), [FR-023-AC-6](../functional/FR-023-python-binding-surface.md)).

## Rationale

The "optimal speed" mandate ([StR-005](../stakeholder/StR-005-native-python-bindings.md)) is only met if the FFI boundary itself is cheap and does not re-serialize data. A 50 µs ceiling keeps per-call overhead negligible against any real parse, and GIL release is what lets a multi-threaded Python orchestrator actually parallelize across the boundary instead of serializing on it. abi3 keeps the packaging burden to one wheel per platform rather than one per CPython minor version — a release-pipeline cost that, left unbounded, would erode the maintainability win of consolidating on one engine.

## Acceptance Criteria

- **NFR-016-AC-1**: A bench compares `quire.parse_document(small_doc)` (Python, warm) against `quire_rs::parse_document(small_doc)` (Rust) and asserts the per-call delta is < 50 µs median.
- **NFR-016-AC-2**: A test spawns two Python threads each calling `quire.load_repo` on the same corpus and asserts combined wall-clock < 2× a single call (GIL released, [FR-023-AC-5](../functional/FR-023-python-binding-surface.md)).
- **NFR-016-AC-3**: CI builds one abi3 wheel and imports + smoke-tests it under two CPython 3.x minor versions; both succeed ([FR-023-AC-6](../functional/FR-023-python-binding-surface.md)).
- **NFR-016-AC-4**: A test confirms returned objects carry no reference forcing a re-parse on the Python side (the data is materialized in Rust and handed over once) — paired with [FR-023-AC-7](../functional/FR-023-python-binding-surface.md) (no subprocess/socket on the data path).

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Per-crossing overhead, `quire.parse_document` vs Rust call (≤ 4 KB doc) | < 50 µs median | 50 µs median | Criterion / pytest-benchmark |
| Two-thread `quire.load_repo` combined wall-clock (GIL released) | < 2× single-call | < 2× single-call | Load Benchmark |
| abi3 wheel imports across CPython 3.x minor versions | ≥ 2 versions, one wheel | ≥ 2 versions | CI Gate (wheel-build lane) |
| Returned objects forcing Python-side re-parse | 0 | 0 | Inspection |

## Verification

- Boundary micro-bench in `benches/py_overhead.rs` (or a maturin-built pytest-benchmark harness) on every PR with the `python` feature.
- abi3 cross-version import test runs in the wheel-build CI lane.
