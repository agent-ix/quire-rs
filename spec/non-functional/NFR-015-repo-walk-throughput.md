---
id: NFR-015
title: "Repository Walk Throughput Scales With Cores"
type: NFR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-011"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-024"
    type: "requires"
    cardinality: "1:1"
---

## Statement

`RepoLoad::load_repo` over a corpus of **1,000 markdown documents** (representative spec-artifact sizes, ~2–20 KB each) SHALL complete in:

- **p50 < 600 ms** single-threaded, and
- **p50 < 200 ms** on 8 threads,

on the canonical baseline runner (Apple Silicon M2 Pro, release build). The parse fan-out ([FR-024](../functional/FR-024-parallel-repo-walk.md)) SHALL exhibit near-linear speedup with core count up to 8 cores (parallel efficiency ≥ 0.6), confirming the rayon parallelization is effective and not serialized on a shared bottleneck.

The directory walk component is I/O-bound and excluded from the scaling claim; the claim is about the parse fan-out, which dominates total time at this corpus size.

### Cross-runner policy

Per-runner baselines stored separately (Apple Silicon M2 Pro canonical; Ubuntu x86_64 CI allowance +50%). Same regression-gate policy as [NFR-001](./NFR-001-render-latency.md)/[NFR-002](./NFR-002-parse-latency.md).

## Rationale

`filament_parser` loads whole spec repos; the per-file Python path is sequential and GIL-bound. The entire point of pushing the walk into Rust ([StR-005](../stakeholder/StR-005-native-python-bindings.md), [US-011](../usecase/US-011-python-parses-repo-via-bindings.md)) is to recover the parallelism the GIL denied. A throughput target that did not assert *scaling* would let a technically-parallel-but-contended implementation pass while delivering no real speedup. The 8-thread target plus the parallel-efficiency floor make the scaling property itself the gate.

## Acceptance Criteria

- **NFR-015-AC-1**: A criterion bench `bench_load_repo_1k` loads a synthetic 1,000-document corpus and measures `load_repo` at 1 and 8 threads; medians below 600 ms and 200 ms respectively.
- **NFR-015-AC-2**: The bench computes parallel efficiency `T1 / (8 × T8)` and asserts ≥ 0.6.
- **NFR-015-AC-3**: A regression test compares against a stored baseline; >10% slowdown on either thread count fails CI.
- **NFR-015-AC-4**: A correctness assertion on the same corpus confirms `documents.len() == 1000` and the output ordering is path-sorted and identical across both thread counts (ties [FR-024-AC-4](../functional/FR-024-parallel-repo-walk.md) / [NFR-006](./NFR-006-determinism.md) to the throughput bench).

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| `load_repo` p50, 1,000-doc corpus, single-threaded (canonical M2 Pro) | < 600 ms | 600 ms | Criterion Benchmark |
| `load_repo` p50, 1,000-doc corpus, 8 threads | < 200 ms | 200 ms | Criterion Benchmark |
| Parallel efficiency `T1 / (8 × T8)` | ≥ 0.6 | ≥ 0.6 | Criterion Benchmark |
| Same-runner regression vs stored baseline (either thread count) | no slowdown | < 10% slowdown | Criterion Benchmark |
| Output correctness: `documents.len()` and path-sorted ordering across thread counts | 1000, identical | 1000, identical | Inspection (correctness assertion) |

## Verification

- Criterion benches in `benches/load_repo.rs` execute on every PR with a stored baseline. Thread count is set via rayon pool configuration in the bench harness.
