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

### Known cost

miri runs roughly 10–100× slower than normal `cargo test`. Library test suite size SHALL stay small enough for the miri job to complete within 30 minutes (GitHub Actions default timeout). If the suite outgrows that budget, the `miri` job runs a curated subset under a `miri` test marker.

## Rationale

Even safe Rust can encounter UB through unsound dep crates. miri's runtime checking is the cheapest way to catch this. Running on schedule (not per-PR) keeps PR latency low while still surfacing issues before release.

## Acceptance Criteria

- **NFR-012-AC-1**: `.github/workflows/ci.yml` contains a `miri:` job that runs `cargo +nightly miri test --lib` on weekly schedule + workflow_dispatch + tag push.
- **NFR-012-AC-2**: The job uses caching (`Swatinem/rust-cache@v2`) to amortize the nightly toolchain install.
- **NFR-012-AC-3**: Test suite under `cargo miri test --lib` completes in under 30 minutes on the GitHub-hosted Ubuntu runner.
- **NFR-012-AC-4**: A miri-flagged UB violation is recorded as a P0 issue with the offending stack trace.

## Verification

- CI workflow visible in repo; passing run on tag push.
- `make miri` local target reproduces the CI invocation.
