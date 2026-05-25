---
id: NFR-012
title: "miri Undefined-Behavior Check on Schedule"
artifact_type: NFR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/non-functional/NFR-003"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-004"
    type: "implements"
    cardinality: "1:1"
---

## Statement

`quire-rs` SHALL run `cargo +nightly miri test --lib` on a CI schedule (weekly + workflow_dispatch + on every tag push). miri detects undefined behavior at runtime — even with zero first-party `unsafe` (NFR-003), miri can catch UB introduced by dependency crates (validator, MiniJinja internals, serde, etc.).

### Operational policy

- A CI job (`miri` in `.github/workflows/ci.yml`) runs on weekly schedule, `workflow_dispatch`, and on every tag push.
- The job uses the nightly toolchain via `dtolnay/rust-toolchain@nightly` + `rustup component add miri`.
- Test suite under miri is restricted to `--lib` (library tests only) — integration tests are too slow under miri.
- A miri violation is a P0 bug; the offending dependency is either upgraded, replaced, or the affected code path is restructured.

### Scope: the FFI boundary (v0.3)

miri has **no support for the CPython C-API foreign calls** introduced by the PyO3 binding (FR-023). The miri job therefore runs the **default feature set (without `python`)** — it covers the pure-Rust engine (parser, loader, render, validate, corpus, resolution) but cannot execute or check the binding layer. UB and memory-safety in the FFI layer is covered instead by Python-level tests (the pytest harness, FR-023 TCs) and the scheduled sanitizer lanes (NFR-018, ASAN/TSAN on the built extension). This division is deliberate: miri for the safe-Rust core, sanitizers for the FFI boundary.

### Known cost

miri runs roughly 10–100× slower than normal `cargo test`. Library test suite size SHALL stay small enough for the miri job to complete within 30 minutes (GitHub Actions default timeout). If the suite outgrows that budget, the `miri` job runs a curated subset under a `miri` test marker.

## Rationale

Even safe Rust can encounter UB through unsound dep crates. miri's runtime checking is the cheapest way to catch this. Running on schedule (not per-PR) keeps PR latency low while still surfacing issues before release.

## Acceptance Criteria

- **NFR-012-AC-1**: `.github/workflows/ci.yml` contains a `miri:` job that runs `cargo +nightly miri test --lib` on weekly schedule + workflow_dispatch + tag push.
- **NFR-012-AC-2**: The job uses caching (`Swatinem/rust-cache@v2`) to amortize the nightly toolchain install.
- **NFR-012-AC-3**: Test suite under `cargo miri test --lib` completes in under 30 minutes on the GitHub-hosted Ubuntu runner.
- **NFR-012-AC-4**: A miri-flagged UB violation is recorded as a P0 issue with the offending stack trace.
- **NFR-012-AC-5**: The `miri` job runs without the `python` feature; a doc note (in the workflow or `spec/spec.md` §19) records that the FFI layer is out of miri's scope and is covered by NFR-018 sanitizer lanes + the FR-023 pytest harness.

## Verification

- CI workflow visible in repo; passing run on tag push.
- `make miri` local target reproduces the CI invocation.
