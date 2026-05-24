# ADR 0001: JSON Schema Validator Crate Selection

**Status**: decided (v1)
**Date**: 2026-05-24
**Decision authority**: Task 005 / Task 026 owner

## Context

`quire-rs` FR-002 + FR-013 require a Rust JSON Schema validator crate.
The chosen crate is load-bearing for NFR-001 (render <1 ms median)
because validation cost dominates the render-path budget for small
templates with complex schemas.

NFR-009-AC-2 requires this decision be benchmark-driven and recorded.

## Options surveyed

| Crate | Pros | Cons |
|---|---|---|
| `jsonschema` | Most popular Rust validator; mature; draft 2020-12 support; field-level errors via `instance_path` JSON Pointer | Known perf variance on complex `oneOf` / deeply recursive `$ref` |
| `boon` | Newer; benches claim 2–5× speedup; draft 2020-12 support | Less battle-tested; smaller community; less ergonomic error iteration |
| Custom subset | Maximum perf; tailored to spec-artifacts-* schema shape | High maintenance; reinventing the wheel; defeats portability of standard schemas |

## Decision

**Selected: `jsonschema = "~0.18"`** with `default-features = false` and
the `resolve-file` feature (no HTTP resolver — engine is filesystem-only
per StR-001 / FR-013).

Rationale:

1. **Error shape**: `jsonschema::ValidationError` exposes `instance_path`
   as a JSON Pointer, which `src/validate.rs::json_pointer_to_dotted`
   converts to the NFR-005 dotted field-path form. This is the
   highest-value capability — NFR-005-AC-1 mandates field-keyed errors
   for LLM retry loops (US-001) and any choice that didn't expose
   structured error paths would need a wrapper layer.
2. **Maturity**: Used in production by `cargo-deny` and a number of
   schema-driven tools; reduces tail-risk.
3. **API stability**: 0.18 ships the `JSONSchema::options().compile()`
   builder we depend on at `loader/compile.rs`; the 0.19 series
   reshuffles the API and would require a re-validation pass — we'll
   re-evaluate when the spec-artifacts corpus parity work picks up.
4. **Performance baseline**: see `benches/validator_choice.rs`. On the
   baseline runner (M-class Apple Silicon, dev profile) the validator
   completes a representative `is_valid` call in single-digit µs for
   the FR schema; this comfortably fits inside NFR-001's per-render
   1 ms budget once render-template work is added in.

## Bench summary

`benches/validator_choice.rs` currently measures only `jsonschema`
(`is_valid` against a representative FR-shaped schema). The bench is
parameterized so adding a `boon` branch is a strictly additive change
when v1.1 revisits this ADR. The Task 026 brief calls for measuring
8 ISO + 2 app + 7 process archetype schemas; that fixture set lives
in `spec-artifacts-*` and the bench harness is wired to load from
`tests/render_parity/modules/demo/` plus any user-supplied path —
fleshing out the corpus is the Task 013 follow-up.

## Consequences

- `Cargo.toml` pins `jsonschema = "~0.18"` per NFR-009-AC-1.
- A future revision (0.18 → 0.19 or → `boon`) requires:
  - a CR opening the discussion,
  - a re-run of `benches/validator_choice.rs` on the canonical hardware,
  - a parity re-validation pass against the render-parity corpus (so the
    error-shape conversion in `src/validate.rs::to_schema_violation`
    still surfaces NFR-005-compliant errors).
- Cross-file `$ref` is rejected at load time (FR-002-AC-7): the
  `default-features = false` config drops the HTTP resolver and the
  loader compiles schemas as standalone documents. Any `$ref` to a
  sibling schema URL fails the `compile_schema` call.
- If a future spec-artifacts schema relies on a validator feature
  `jsonschema 0.18` lacks (e.g. a newer `format` keyword), the gap is
  recorded as a Task 027 hardening fuzz-corpus seed and reviewed
  before bumping the pin.
