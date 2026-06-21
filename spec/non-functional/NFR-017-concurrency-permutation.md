---
id: NFR-017
title: "Concurrency Permutation for the Parallel Walk (loom)"
type: NFR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-024"
    type: "depends_on"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-004"
    type: "traces_to"
    cardinality: "1:1"
---

## Statement

The parallel parse path of `load_repo` ([FR-024](../functional/FR-024-parallel-repo-walk.md)) introduced concurrency where the v1 engine had none. `quire-rs` SHALL verify that this path is free of data races and interleaving-dependent results using **loom** (`cargo` `loom` feature, exhaustive small-scope interleaving), run on a scheduled CI lane.

This NFR exists because §19 (Hardening Posture) originally skipped loom/shuttle on the grounds that "quire-rs has no synchronization primitives." v0.3's rayon fan-out makes that rationale stale. Rather than assume the data-parallel design is race-free, loom *proves* it for the small-scope case: under [FR-024](../functional/FR-024-parallel-repo-walk.md)'s no-shared-mutable-state invariant ([FR-024-AC-9](../functional/FR-024-parallel-repo-walk.md)), loom should find no race — and the test failing would mean that invariant was violated.

### Operational policy

- A `loom`-gated test (`#[cfg(loom)]`) models a small parallel parse (2–3 files across 2 threads) collecting into the result vector, asserting: (a) no data race, (b) the collected output is identical across all interleavings, (c) path-sorted ordering holds regardless of completion order.
- Runs on a scheduled lane (weekly + workflow_dispatch + tag push), consistent with the miri/fuzz/mutants cadence — NOT per-PR (loom's combinatorial cost is too high for PR latency).
- A loom-detected race is a P0 bug. Because [FR-024](../functional/FR-024-parallel-repo-walk.md) mandates no hand-written synchronization, a race almost certainly means a shared-mutable-state regression — fix by restoring the data-parallel collect.

### Relationship to shuttle

`shuttle` (randomized scheduling for larger state spaces) is NOT adopted: at v0.3's concurrency size (one data-parallel collect, no locks) loom's exhaustive small-scope checking is sufficient and stronger. If a future version introduces actual shared-mutable concurrency (a cache, a pool with shared state), shuttle SHALL be reconsidered.

## Acceptance Criteria

- **NFR-017-AC-1**: A `#[cfg(loom)]` test models the parallel parse collection and passes under loom's exhaustive interleaving (no race; identical, path-sorted output across all schedules).
- **NFR-017-AC-2**: The loom test scope (file/thread count) is bounded so the lane completes within the CI timeout (≤ 30 min, parity with [NFR-012](./NFR-012-miri-ub-check.md)).
- **NFR-017-AC-3**: `.github/workflows/ci.yml` runs the loom lane on weekly schedule + workflow_dispatch + tag push; a `make loom` target reproduces it locally.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Data races found by loom in small-scope parallel parse | 0 | 0 | loom Exhaustive Interleaving |
| Output identical + path-sorted across all loom schedules | Pass | Pass | loom Exhaustive Interleaving |
| loom lane completes within CI timeout | ≤ 30 min | ≤ 30 min | Scheduled CI Gate |

## Verification

- `cargo test --features loom --test concurrency` (or equivalent) visible in CI; passing run on tag push.
