---
id: NFR-010
title: "Public API Stability and Semver Policy"
type: NFR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-001"
    type: "traces_to"
    cardinality: "1:1"
---

## Statement

`quire-rs` SHALL follow semantic versioning for its public Rust API:

- **Patch (0.y.Z+1)**: bug fixes, performance improvements, internal refactors. No API change.
- **Minor (0.y+1.0)**: new public items, new variants on `#[non_exhaustive]` enums, new Cargo features. No removal or signature change of existing items.
- **Major (X+1.0.0)**: any breaking change — removed items, changed signatures, removed variants, behavior change that breaks the parity suite without explicit upstream coordination.

### Pre-1.0 caveat

While `quire-rs` is `0.y.z`, the **minor** position carries the semantic weight of major (per Cargo convention). Breaking changes between `0.1.x` and `0.2.x` are allowed but SHALL be documented in `CHANGELOG.md`.

### Public surface

Public surface is **everything** exposed via `pub use` in `src/lib.rs`. Items marked `pub(crate)` or below are internal. Items marked `#[doc(hidden)]` are public-by-Rust-rules but not part of the stable surface.

### Non-exhaustive enums

`QuireError`, `Diagnostic`, `Locator`, and other public enums SHALL be marked `#[non_exhaustive]` to allow variant additions without breaking changes.

### Type bounds in public API

`Send + Sync` is a stability guarantee for `Registry`, `CompiledArchetype`, `QuireDocument`. Removing these bounds is breaking.

## Rationale

Downstream consumers (Filament editor, CLI tools, future LLM adapters) need stability guarantees to depend on `quire-rs` without churn. Semver lets them set version constraints in their `Cargo.toml`.

## Acceptance Criteria

- **NFR-010-AC-1**: `src/lib.rs` re-exports the documented public surface; `Cargo.toml` declares the crate version following the policy above.
- **NFR-010-AC-2**: Public enums are marked `#[non_exhaustive]` (verified by a static check or a compile-fail test for an exhaustive match outside the crate).
- **NFR-010-AC-3**: `CHANGELOG.md` exists; each release entry classifies changes as Added / Changed / Deprecated / Removed / Fixed / Security.
- **NFR-010-AC-4**: A `cargo-semver-checks` invocation against the previous published version reports no unexpected breaks.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| `cargo-semver-checks` unexpected breaks vs previous published version | 0 | 0 | CI Gate |
| Public enums marked `#[non_exhaustive]` | all | all | Static Analysis / compile-fail test |
| `CHANGELOG.md` entries classified (Added/Changed/Deprecated/Removed/Fixed/Security) | every release | every release | Inspection |
| Version bump matches semver policy for the change class | Pass | Pass | Inspection |

## Verification

- `cargo-semver-checks` run in CI (opt-in via workflow_dispatch initially; promote to PR gate post-1.0).
- `CHANGELOG.md` reviewed at release time.
