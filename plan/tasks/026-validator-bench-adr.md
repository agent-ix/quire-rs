# Task 026: Validator Crate Bench + ADR (NFR-009)

Status: blocked on Gate G1 (parser parity); MUST be done BEFORE Task 005 implementation starts

## Scope

Bench candidate JSON Schema validator crates against representative spec-artifacts-* schemas. Record decision in `spec/assets/adr/0001-validator-crate.md`. Pin the chosen crate in `Cargo.toml` per NFR-009.

This task is the ADR step before Task 005 (archetype loader) commits to a validator implementation. **The choice is load-bearing for NFR-001 perf.**

## Subtasks

- [ ] **Bench harness.** `benches/validator_choice.rs` runs each candidate against:
  - 8 ISO archetype schemas (FR, NFR, StR, US, IT, TC, AC, CON)
  - 2 app archetype schemas (ApplicationSpec, MasterRequirements)
  - 7 process archetype schemas
  - Mix of valid + invalid inputs per archetype
- [ ] **Candidates.** `jsonschema`, `boon`, and any other reasonable alternatives.
- [ ] **Measurements.** Median validation time, 99th percentile, allocations per validate, build time.
- [ ] **Decision.** Update `spec/assets/adr/0001-validator-crate.md` with bench numbers and chosen crate.
- [ ] **Cargo.toml pin.** Add chosen crate with tilde-pin per NFR-009.

## Owns

NFR-009-AC-2 (ADR exists with bench numbers).

## Dependencies

Gate G1 (parser parity proves the parsing surface is real; bench needs real archetypes to be loadable).

## Unblocks

Task 005 (loader can commit to a specific validator).

## Deliverables

- `benches/validator_choice.rs`
- `spec/assets/adr/0001-validator-crate.md` (filled in)
- `Cargo.toml` validator dep with tilde pin

## Primary Tests

TC-331.

## Notes

- This task is the gate between exploratory exploration (Tasks 001-004) and concrete render-path implementation (Task 005+).
- Run the bench on the canonical baseline runner (Apple Silicon M2 Pro) AND on CI (Ubuntu x86_64) to anchor both per-runner baselines.
- If neither candidate hits NFR-001, this task surfaces the gap early — we either relax the NFR (CR required) OR write a custom validator (R2 fallback).
