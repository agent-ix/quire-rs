---
id: NFR-003
title: "Zero unsafe Blocks in v1"
type: NFR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-004"
    type: "traces_to"
    cardinality: "1:1"
---

## Statement

`quire-rs` SHALL contain zero `unsafe` blocks in v1. The `scripts/check_unsafe_comments.sh` script inherited from `rust-lib-cookiecutter` enforces that every `unsafe {` block has a `// SAFETY:` comment within the 3 preceding lines — for v1 the simpler invariant applies: no `unsafe` blocks at all, and `scripts/unsafe_comment_baseline.txt` SHALL be empty.

If a future release introduces an `unsafe` block (e.g. for FFI), the addition requires:

1. A `// SAFETY: <reason>` comment justifying memory invariants.
2. A code review with explicit acknowledgment of the unsafe addition.
3. An ADR in `spec/assets/adr/` recording why the unsafe was needed.

> **CR (2026-06, ADR 0006):** the zero-`unsafe` invariant is now a **compile-time
> guarantee**, not just a script check. The crate root carries
> `#![cfg_attr(not(feature = "python"), forbid(unsafe_code))]`, so any first-party
> `unsafe` in the default build is a hard **compile error**. The forbid is scoped
> off for `--features python` (PyO3 macros expand to `unsafe` in-crate); that build
> stays covered by `check_unsafe_comments.sh` (AC-1/AC-4). This supersedes the
> scheduled Miri job ([NFR-012](./NFR-012-miri-ub-check.md), retired): with zero first-party `unsafe` enforced at
> compile time, there is no first-party UB surface for Miri to interpret; dependency
> advisories are covered by `cargo-audit` ([NFR-014](./NFR-014-advisory-checking.md)) and the concurrency surface by
> `loom` ([NFR-017](./NFR-017-concurrency-permutation.md)).

## Rationale

Inheriting from `agent-ix/ecaz` ([StR-004](../stakeholder/StR-004-safety-scaffolding-inheritance.md)), unsafe code is the highest-risk surface in a Rust crate. A markdown parser + templating engine has no legitimate need for unsafe in v1 — every operation can be expressed in safe Rust with negligible performance penalty.

### Scope: the `python` feature (v0.3)

The PyO3 binding ([FR-023](../functional/FR-023-python-binding-surface.md)) is feature-gated. PyO3's `#[pyclass]`/`#[pymethods]`/`#[pymodule]` macros expand to `unsafe` internally, but that unsafe is **upstream** (in the `pyo3` crate), not first-party `quire-rs` source. The zero-unsafe invariant therefore SHALL hold for **first-party source under all feature combinations**, including `--features python`: `quire-rs`'s own `src/python/` module SHALL contain no hand-written `unsafe` blocks. If a PyO3 pattern genuinely requires a first-party `unsafe` block, it follows the same three-step process above (SAFETY comment + review + ADR) and is added to the baseline with rationale.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| NFR-003-AC-1 | `bash scripts/check_unsafe_comments.sh` runs in CI and exits 0. | Demonstration |
| NFR-003-AC-2 | `scripts/unsafe_comment_baseline.txt` is empty (zero entries). | Inspection |
| NFR-003-AC-3 | A grep `rg 'unsafe\s*\{' src/` returns zero matches. | Inspection |
| NFR-003-AC-4 | `rg 'unsafe\s*\{' src/` returns zero matches **with the `python` feature enabled** (first-party binding source is unsafe-free; PyO3 macro-generated unsafe is upstream and not counted). | Inspection |
| NFR-003-AC-5 | The crate root declares `#![cfg_attr(not(feature = "python"), forbid(unsafe_code))]`; the **default** `cargo build` compiles (the compiler proves zero first-party `unsafe`), and introducing any first-party `unsafe` block/fn/`impl` makes the default build **fail to compile**. `cargo build --features python` compiles with the forbid scoped off. | Demonstration |

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| First-party `unsafe` blocks in `src/` (default build) | 0 | 0 | Static Analysis (`rg 'unsafe\s*\{' src/`) |
| First-party `unsafe` blocks with `--features python` | 0 | 0 | Static Analysis |
| `scripts/check_unsafe_comments.sh` exit status | 0 | 0 | CI Gate |
| `scripts/unsafe_comment_baseline.txt` entries | 0 (empty) | 0 | CI Gate |
| Default build compiles under `#![forbid(unsafe_code)]` | Pass | Pass | Compile-time check |

## Verification

- CI workflow `.github/workflows/ci.yml` runs `bash scripts/check_unsafe_comments.sh` as a required job.
- `make audit-unsafe` runs locally.
