---
id: NFR-012
title: "miri Undefined-Behavior Check on Schedule"
type: NFR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/non-functional/NFR-003"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-004"
    type: "implements"
    cardinality: "1:1"
---

> **RETIRED (2026-06, CR — supersedes the Miri job; see ADR 0006).** The Miri job
> is removed. First-party UB is now a **compile-time impossibility**: [NFR-003-AC-5](./NFR-003-zero-unsafe.md)
> makes the default build carry `#![forbid(unsafe_code)]`, so any first-party
> `unsafe` is a hard compile error (Miri's runtime check is moot — there is nothing
> first-party for it to interpret). Miri's only remaining rationale here was
> **dependency** UB, but Miri-over-deps is low-signal/high-noise — it aborts on a
> **Stacked-Borrows false-positive inside rayon's thread pool** (a sound, upstream-
> Miri-tested crate), ran >1h, and `rust-lib-cookiecutter` treats "miri-on-main" as
> opt-in. Dependency unsoundness is covered by **cargo-audit** (RUSTSEC advisories,
> [NFR-014](./NFR-014-advisory-checking.md)) + tight pins ([NFR-009](./NFR-009-dependency-pinning.md)); the concurrency surface by **loom** ([NFR-017](./NFR-017-concurrency-permutation.md)).
> All ACs below are retired (un-bolded; excluded from the AC→TC integrity tally).

## Statement

**RETIRED (2026-06, CR — supersedes the Miri job; see ADR 0006).** The original
requirement below is retired and kept for history only.

`quire-rs` SHALL run `cargo +nightly miri test --lib` on a CI schedule (weekly + workflow_dispatch + on every tag push). miri detects undefined behavior at runtime — even with zero first-party `unsafe` ([NFR-003](./NFR-003-zero-unsafe.md)), miri can catch UB introduced by dependency crates (validator, MiniJinja internals, serde, etc.).

### Operational policy

- A CI job (`miri` in `.github/workflows/ci.yml`) runs on weekly schedule, `workflow_dispatch`, and on every tag push.
- The job uses the nightly toolchain via `dtolnay/rust-toolchain@nightly` + `rustup component add miri`.
- Test suite under miri is restricted to `--lib` (library tests only) — integration tests are too slow under miri.
- A miri violation is a P0 bug; the offending dependency is either upgraded, replaced, or the affected code path is restructured.

### Scope: the FFI boundary (v0.3)

miri has **no support for the CPython C-API foreign calls** introduced by the PyO3 binding ([FR-023](../functional/FR-023-python-binding-surface.md)). The miri job therefore runs the **default feature set (without `python`)** — it covers the pure-Rust engine (parser, loader, render, validate, corpus, resolution) but cannot execute or check the binding layer. UB and memory-safety in the FFI layer is covered instead by Python-level tests (the pytest harness, [FR-023](../functional/FR-023-python-binding-surface.md) TCs) and the scheduled sanitizer lanes ([NFR-018](./NFR-018-ffi-sanitizer-lanes.md), ASAN/TSAN on the built extension). This division is deliberate: miri for the safe-Rust core, sanitizers for the FFI boundary.

### Known cost

miri runs roughly 10–100× slower than normal `cargo test`. Library test suite size SHALL stay small enough for the miri job to complete within 30 minutes (GitHub Actions default timeout). If the suite outgrows that budget, the `miri` job runs a curated subset under a `miri` test marker.

## Rationale

Even safe Rust can encounter UB through unsound dep crates. miri's runtime checking is the cheapest way to catch this. Running on schedule (not per-PR) keeps PR latency low while still surfacing issues before release.

## Acceptance Criteria

- NFR-012-AC-1 (RETIRED): `.github/workflows/ci.yml` contains a `miri:` job that runs `cargo +nightly miri test --lib` on weekly schedule + workflow_dispatch + tag push.
- NFR-012-AC-2 (RETIRED): The job uses caching (`Swatinem/rust-cache@v2`) to amortize the nightly toolchain install.
- NFR-012-AC-3 (RETIRED): Test suite under `cargo miri test --lib` completes in under 30 minutes on the GitHub-hosted Ubuntu runner.
- NFR-012-AC-4 (RETIRED): A miri-flagged UB violation is recorded as a P0 issue with the offending stack trace.
- NFR-012-AC-5 (RETIRED): The `miri` job runs without the `python` feature; a doc note records that the FFI layer is out of miri's scope and is covered by [NFR-018](./NFR-018-ffi-sanitizer-lanes.md) sanitizer lanes + the [FR-023](../functional/FR-023-python-binding-surface.md) pytest harness.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Miri job (RETIRED) — UB violations on safe-Rust core | 0 | 0 | Scheduled CI Gate (retired; superseded by `forbid(unsafe_code)`) |
| First-party UB surface remaining after `#![forbid(unsafe_code)]` | 0 | 0 | Compile-time check ([NFR-003](./NFR-003-zero-unsafe.md)) |

## Verification

- CI workflow visible in repo; passing run on tag push.
- `make miri` local target reproduces the CI invocation.
