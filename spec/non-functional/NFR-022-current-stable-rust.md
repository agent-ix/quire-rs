---
id: NFR-022
title: "Current stable Rust qualification"
type: NFR
quality_attribute: maintainability
relationships:
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-004"
    type: "constrains"
---
# NFR-022: Current stable Rust qualification

## Statement

`quire-rs` SHALL qualify all first-party Rust targets on current stable Rust
1.98.1 and advance through the governed transition below when a newer stable
release becomes available.

## Scope

- Applies to `Cargo.toml` `rust-version`, `rust-toolchain.toml`, `clippy.toml`,
  the default crate, Python feature/bindings, WASM target, test targets, audit
  tools, benchmarks, and packaging tools required by repository gates.
- Does not promise support for an older compiler merely because an existing
  file, dependency, or inherited scaffold names one.
- Does not treat rustfmt output changes or repairable Clippy findings as tool
  incompatibility.

## Rationale

The prior 1.75 declaration propagated from scaffold history without a current
compatibility decision, while the selected toolchain had independently moved
to 1.94.1. Preserving either number because it already exists freezes new work
around configuration rather than observed needs. Current stable is the default;
an older hold is an exceptional, evidenced response to an actual required-tool
failure.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| NFR-022-AC-1 | `Cargo.toml`, `rust-toolchain.toml`, and `clippy.toml` name exact Rust 1.98.1, and the compiler, formatter, and Clippy report the 1.98.1 toolchain during the qualification run. | static-quality |
| NFR-022-AC-2 | The default, all-target, Python-feature, WASM, test, documentation, audit, benchmark-build, and packaging gates required by this repository compile or run on Rust 1.98.1 without weakening a gate. | integration-testing |
| NFR-022-AC-3 | A stable release newer than 1.98.1 triggers the same compatibility matrix within seven calendar days and either updates all three declarations together or records the bounded hold in AC-4. | integration-testing |
| NFR-022-AC-4 | A compiler hold names the required failing tool and command, retains the reproduced failure and successful older control, links an upstream issue, names an owner and rerun trigger, and expires no later than 30 days after recording. | inspection |
| NFR-022-AC-5 | A proposed hold based only on formatting drift, a repairable lint finding, existing repository metadata, or an untested compatibility concern is rejected. | property-testing |

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Rust declarations equal current accepted stable | 3 of 3 | 3 of 3 | static-quality |
| Required repository Rust gate classes passing on 1.98.1 | all | all | integration-testing |
| Days from a newer stable release to compatibility result | at most 7 | 7 | integration-testing |
| Compiler hold attribution and reproduction fields present | all | all | inspection |
| Compiler hold lifetime | at most 30 days | 30 days | inspection |
| Formatting-, lint-, existing-pin-, or speculation-only holds accepted | 0 | 0 | property-testing |

## Verification

- A static test parses all three toolchain declarations and compares their
  exact value with the compiler output captured by the gate.
- The repository's real default, Python, WASM, test, documentation, audit,
  benchmark-build, and packaging commands run without changing their flags or
  thresholds merely to accommodate the new compiler.
- A transition-policy test evaluates current, newer-compatible,
  newer-required-tool-incompatible, expired, unattributed, formatting-only,
  lint-only, existing-pin-only, and speculative cases.
- Every new Rust requirement test imports `ix_trace_rs::trace` and uses bare
  `#[trace("TC-...", "NFR-022-AC-...")]` markers.

## Dependencies

- **Upstream**: Rust 1.98.1, released 2026-09-01, is the accepted current stable
  observation for this decision.
- **Downstream**: [ADR-0012](../assets/adr/0012-yaml-engine-maintenance-and-parity.md)
  and all future Rust dependency/tooling decisions consume this baseline.
