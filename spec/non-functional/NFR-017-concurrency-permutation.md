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

`shuttle` (randomized scheduling for larger state spaces) is NOT adopted, and the reconsideration clause this paragraph used to carry — reconsider "if a future version introduces actual shared-mutable concurrency (a cache, a pool with shared state)" — is now **resolved explicitly** (CR-047): the [FR-025](../functional/FR-025-spec-corpus-model.md) lazy body cache is that future version, and its state space is **one once-init cell × two threads** — exactly loom's exhaustive small-scope sweet spot, not the large randomized space shuttle exists for. Shuttle stays not adopted; the cache is covered by a loom permutation instead (AC-4). One modeling limit is recorded: the production primitive is `std::sync::OnceLock`, which loom cannot instrument (loom ships no OnceLock model and cannot see into std's internals), so the loom permutation models the once-cell **contract** — exactly-once init, racers agree on the stored value — with loom primitives, and the real primitive is raced for real under the [NFR-018](./NFR-018-ffi-sanitizer-lanes.md) TSAN lane (`tests/corpus_concurrency.rs`, TC-816).

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| NFR-017-AC-1 | A `#[cfg(loom)]` test models the parallel parse collection and passes under loom's exhaustive interleaving (no race; identical, path-sorted output across all schedules). | Test |
| NFR-017-AC-2 | The loom test scope (file/thread count) is bounded so the lane completes within the CI timeout (≤ 30 min, parity with [NFR-012](./NFR-012-miri-ub-check.md)). | Analysis |
| NFR-017-AC-3 | `.github/workflows/ci.yml` runs the loom lane on weekly schedule + workflow_dispatch + tag push; a `make loom` target reproduces it locally. | Inspection |
| NFR-017-AC-4 | A loom permutation covers concurrent **first-touch of one document's lazy body** ([FR-025-AC-8](../functional/FR-025-spec-corpus-model.md), CR-047): two threads race the once-cell, the init runs exactly once, and both racers observe the identical value, under every interleaving. The std `OnceLock` production primitive is modeled by contract (loom limitation, see *Relationship to shuttle*) and raced for real under the [NFR-018](./NFR-018-ffi-sanitizer-lanes.md) TSAN lane. | Test (TC-815) |

> **CR-047 note (2026-08-15):** The FR-025 lazy body cache introduces the
> first sanctioned interior mutability in `src/corpus` (a per-document
> once-init cell behind `Arc<SpecInner>`), which is the concurrency the
> original loom permutation did not model — [FR-024-AC-9](../functional/FR-024-parallel-repo-walk.md)'s
> blanket no-`Mutex`/`RwLock`/`Atomic` audit was standing in for it. That AC
> is narrowed to the walk fan-out and the audit widened with a named
> exemption list; this NFR gains AC-4 so the risk the audit stood in for is
> covered by an exhaustive permutation instead of a ban, and the shuttle
> paragraph's "reconsider if a cache appears" clause is resolved above
> (agent-ix/quire-rs#93, umbrella #90).

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Data races found by loom in small-scope parallel parse | 0 | 0 | deterministic-simulation (loom, exhaustive interleaving) |
| Output identical + path-sorted across all loom schedules | Pass | Pass | deterministic-simulation (loom, exhaustive interleaving) |
| loom lane completes within CI timeout | ≤ 30 min | ≤ 30 min | performance-benchmarking (loom lane wall-clock) |

## Verification

- `cargo test --features loom --test concurrency` (or equivalent) visible in CI; passing run on tag push.
