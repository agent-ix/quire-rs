---
id: ADR-0006
title: "Compile-time forbid(unsafe_code); retire the Miri job"
type: ADR
---

# ADR 0006: Compile-time `forbid(unsafe_code)`; retire the Miri job

**Status**: Accepted
**Date**: 2026-06-06
**Decision authority**: Peter Krenesky

## Context

quire-rs is a markdown parser + schema/document validator + body-extraction engine
with optional PyO3 bindings (`python` feature) and a `wasm` feature. NFR-003 mandates
**zero first-party `unsafe`** in v1, historically verified by a script
(`check_unsafe_comments.sh` + an empty baseline). NFR-012 additionally specified a
scheduled **Miri** job — explicitly *not* for first-party `unsafe` (there is none),
but to catch UB in **dependency** crates (rayon, serde, regex, jsonschema).

During the v0.3.6 stabilization the Miri job proved a poor fit:

- It ran **>1 hour** and **aborted on a Stacked-Borrows false-positive inside rayon's
  thread-pool internals** (`src/corpus/walk.rs` `par_iter`) — not a quire-rs bug.
  rayon is a sound, widely-used crate with its own upstream Miri CI.
- A deep safety eval established **zero first-party `unsafe`** three independent ways:
  exhaustive grep (all `unsafe` syntaxes + `transmute`/`get_unchecked`/`MaybeUninit`/
  `from_raw`/`ptr::`/`UnsafeCell` → nothing), the **empty**
  `scripts/unsafe_comment_baseline.txt`, and the **compiler accepting
  `#![forbid(unsafe_code)]`** (default build compiles; `--features python` compiles
  with the forbid scoped off). The byte-splice writeback is plain safe `String`/`Vec`;
  the rayon region is shared-nothing (owned `Outcome` per task; zero
  `Mutex`/`RwLock`/`Atomic`/`UnsafeCell`/`unsafe impl Send/Sync`, enforced by
  `scripts/audits/check_no_shared_mutable.sh`).
- `rust-lib-cookiecutter` lists **"miri-on-main" as opt-in** ("heavy CI lanes, add
  per-project when justified"), so Miri is not part of the golden-path floor.

## Decision

1. **Make first-party safety a compile-time guarantee.** Keep
   `#![cfg_attr(not(feature = "python"), forbid(unsafe_code))]` at the crate root.
   Any first-party `unsafe` in the default build is now a **hard compile error** —
   stronger than catching its UB after the fact. The forbid is scoped off for
   `--features python` because PyO3 macros expand to `unsafe` in-crate; that build
   stays covered by `check_unsafe_comments.sh` (NFR-003-AC-1/AC-4). This strengthens
   NFR-003 (new AC-5).

2. **Retire the Miri job (NFR-012).** Remove the `miri:` CI job, the `make miri`
   target, and Miri from the `hardening` composite. With zero first-party `unsafe`
   enforced at compile time, there is no first-party UB surface for Miri to
   interpret; its only remaining rationale (dependency UB) is **low-signal/
   high-noise** (the rayon false-positive is the proof) and is the upstream crates'
   responsibility.

## Residual safety posture

| Surface | Guard |
|---|---|
| First-party `unsafe` / UB (default build) | `#![forbid(unsafe_code)]` — compile-time impossible |
| First-party `unsafe` on the `python` build | `check_unsafe_comments.sh` + empty baseline (NFR-003) |
| Dependency unsoundness (known advisories) | `cargo-audit` / RUSTSEC (NFR-014) + tight version pins (NFR-009) |
| Concurrency (the rayon fan-out) | `loom` exhaustive interleaving (NFR-017) + `check_no_shared_mutable.sh` |
| FFI boundary (PyO3) | TSAN/ASAN sanitizer lanes (NFR-018) + the FR-023 pytest harness |

## Consequences

- **Positive**: no >1h flaky tag job; no rayon false-positives; a *stronger* (compile-
  time) first-party guarantee; alignment with the cookiecutter baseline.
- **Negative / accepted**: loss of runtime UB scanning of dependencies' `unsafe`. This
  is acceptable — those crates are top-tier and upstream-Miri-tested, known advisories
  are caught by cargo-audit, and any future first-party `unsafe` (e.g. new FFI) forces
  a deliberate `#[allow(unsafe_code)]` + the NFR-003 three-step process, at which point
  reintroducing Miri SHALL be reconsidered.
