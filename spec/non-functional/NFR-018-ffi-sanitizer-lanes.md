---
id: NFR-018
title: "FFI Sanitizer Lanes (TSAN + ASAN on the Python Extension)"
type: NFR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-023"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/non-functional/NFR-016"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-004"
    type: "implements"
    cardinality: "1:1"
---

## Statement

The PyO3 binding (FR-023) introduces a C-ABI boundary against the CPython runtime that **miri cannot reach** (NFR-012 scope note). `quire-rs` SHALL run **ThreadSanitizer (TSAN)** and **AddressSanitizer (ASAN)** against the built `python`-feature extension on a scheduled CI lane to cover the two FFI risk surfaces miri leaves uncovered:

- **TSAN** — data races in the **GIL-release window** (FR-023 / NFR-016): when a binding entry point releases the GIL and runs the rayon parse, TSAN verifies no Rust thread races with the Python runtime or with another released-GIL call.
- **ASAN** — memory errors in **object handoff**: use-after-free, leaks, or refcount errors when Rust constructs Python objects and hands ownership across the boundary.

This NFR supersedes the v1 §19 decision to skip `-Z sanitizer=address|thread` ("marginal value for safe Rust above miri"). That rationale held for a pure-Rust crate; it does not hold across an FFI boundary with a shared interpreter runtime.

### Operational policy

- Both sanitizers run on a **scheduled lane** (weekly + workflow_dispatch + tag push), NOT per-PR — building the extension and an instrumented test run under each sanitizer is too slow for PR latency, matching the miri/fuzz/mutants cadence.
- TSAN target: a two-thread Python harness each calling `quire.load_repo` (the GIL-release concurrency case, TC-464) plus a concurrent `parse_document` mix, built with `-Z sanitizer=thread` on nightly.
- ASAN target: the object-handoff test set (parse/extract/validate/render results crossing to Python, then dropped) built with `-Z sanitizer=address`, run under an ASAN-aware CPython where feasible (or `PYTHONMALLOC=malloc` to expose allocations).
- A sanitizer-detected race/leak/UAF is a P0 bug with a committed reproducer (parity with NFR-011-AC-4).

### Known cost / limitations

- Sanitizers require a nightly toolchain and a rebuild of the extension + dependencies with instrumentation; the lane is heavier than miri. Bounded by running only the focused FFI concurrency/handoff tests, not the full suite.
- Some CPython internals produce ASAN noise; a suppressions file (`asan.supp`) SHALL be maintained for known-benign interpreter allocations, with each suppression carrying a one-line rationale (parity with the deny.toml advisory-ignore policy).

## Acceptance Criteria

- **NFR-018-AC-1**: A scheduled CI lane builds the `python`-feature extension with `-Z sanitizer=thread` and runs the two-thread `load_repo` harness; zero races reported.
- **NFR-018-AC-2**: A scheduled CI lane builds with `-Z sanitizer=address` and runs the object-handoff test set; zero leaks / use-after-free in first-party handoff code (interpreter-internal noise suppressed via `asan.supp` with rationale).
- **NFR-018-AC-3**: Both lanes run on weekly schedule + workflow_dispatch + tag push (not per-PR); `make sanitize` reproduces them locally.
- **NFR-018-AC-4**: A sanitizer-detected race/leak/UAF is recorded as a P0 issue with a committed reproducer.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| TSAN data races in GIL-release window (two-thread `load_repo` harness) | 0 | 0 | Scheduled CI Gate (`-Z sanitizer=thread`) |
| ASAN leaks / use-after-free in first-party object handoff | 0 | 0 | Scheduled CI Gate (`-Z sanitizer=address`) |
| Both lanes run weekly + workflow_dispatch + tag push | Pass | Pass | Inspection (CI workflow) |
| Sanitizer-detected race/leak/UAF committed as P0 reproducer | Pass | Pass | Inspection |

## Verification

- Sanitizer jobs visible in `.github/workflows/ci.yml`; passing run on tag push; `asan.supp` checked in with rationale comments.
