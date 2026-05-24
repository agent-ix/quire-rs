# ADR 0001: JSON Schema Validator Crate Selection

**Status**: pending bench (decided at Task 005 implementation start)
**Date**: TBD
**Decision authority**: Task 005 owner

## Context

`quire-rs` FR-002 + FR-013 require a Rust JSON Schema validator crate. The chosen crate is load-bearing for NFR-001 (render <1 ms median) because validation cost dominates the render-path budget for small templates with complex schemas.

NFR-009-AC-2 requires this decision be benchmark-driven and recorded.

## Options

| Crate | Pros | Cons |
|---|---|---|
| `jsonschema` | Most popular; mature; draft 2020-12 support; good error messages | Known perf variance on complex `oneOf` / recursive `$ref` |
| `boon` | Newer; benchmarks claim 2-5× faster; draft 2020-12 support | Less battle-tested; smaller community |
| Custom (subset only) | Maximum perf; tailored to spec-artifacts-* schema shape | High maintenance; reinventing wheel; defeat purpose of standard schemas |

## Bench plan

At Task 005 start, bench all three against a representative set of spec-artifacts-* schemas + valid + invalid input:

- 8 ISO archetype schemas
- 2 app archetype schemas
- 7 process archetype schemas
- 5+ object archetype schemas (smaller, from spec-objects-*)

Measure:
- Median validation time per (schema, input) pair on baseline runner
- 99th percentile
- Memory allocation per validation
- Build / compile time for the crate

## Decision

_To be filled in by Task 005 owner after bench. Document the choice with concrete numbers and the per-archetype median._

## Consequences

- `Cargo.toml` pins the chosen crate per NFR-009
- A future revision (e.g. jsonschema 0.18 → 0.19) requires a CR + parity re-run
- If neither `jsonschema` nor `boon` hits NFR-001, the fallback is a custom validator for the subset of features actually used — listed as a v1.1 risk in R2 (see analysis findings)
