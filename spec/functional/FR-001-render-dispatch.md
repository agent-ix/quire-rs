---
id: FR-001
title: "Render Dispatch: Generic Engine Over (Schema, Template, Data)"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-001"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-001"
    type: "implements"
    cardinality: "1:1"
---

> **RETIRED (render removal — 2026-06-04):** The render/templating feature is
> **removed** from `quire-rs` (no backward-compatibility layer). This FR (generic
> render dispatch over `(schema, template, data)`) is **retired**: there is no
> `render`/`render_by_name`, no MiniJinja environment, and no byte-parity-with-Python
> contract. The retained engine validates data ([FR-002](./FR-002-schema-validation-pipeline.md), unchanged) and byte-splices
> blocks (FR-022) without rendering. This document is kept for history and
> traceability only; its acceptance criteria are dropped from the required-coverage
> tally. The retirement and rationale are recorded in `spec.md` §2bis. Quality gate
> **G2** (render parity) is retired with it. New work does not target this FR.

## Description

`quire-rs` SHALL expose a generic render API that knows nothing about specific archetypes — `FR`, `NFR`, `ADR`, etc. are data shipped by Filament, not types compiled into the engine.

The primary entry point:

```rust
pub fn render(archetype: &CompiledArchetype, data: &serde_json::Value)
    -> Result<String, QuireError>;
```

where `CompiledArchetype` carries:
- A pre-compiled JSON Schema validator (built from a JSON Schema document loaded at archetype-load time — see [FR-013](./FR-013-archetype-loader.md))
- A pre-parsed MiniJinja template (compiled once at load time)
- A stable archetype name + version for diagnostics

The function SHALL:

1. Validate `data` against the compiled JSON Schema. Violations return `QuireError::SchemaViolation` with a field-keyed list (see [NFR-005](../non-functional/NFR-005-actionable-schema-errors.md)).
2. Render the validated `data` through the pre-parsed MiniJinja template using the long-lived strict-undefined `Environment` (see [FR-004](./FR-004-minijinja-strict-environment.md)).
3. Return the rendered markdown on success.

The engine SHALL NOT panic on any input. It SHALL be re-entrant and safe for concurrent calls. A `CompiledArchetype` is `Send + Sync` and may be shared across threads.

A convenience function `render_by_name(registry: &ArchetypeRegistry, name: &str, data: &Value)` looks up the archetype by name and dispatches to `render`. Name resolution is the responsibility of the registry ([FR-013](./FR-013-archetype-loader.md), [FR-014](./FR-014-module-activation.md)).

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-001-AC-1 | (RETIRED) For each archetype in the Filament v1 corpus (currently `spec-artifacts-iso` + `spec-artifacts-app` + `spec-artifacts-process` = 17 types), feeding the same `data` JSON through `quire-rs::render` and the Python Jinja2 reference produces byte-identical output. | Test |
| FR-001-AC-2 | (RETIRED) `render_by_name(registry, "nonexistent", ...)` returns `Err(QuireError::UnknownArchetype { name: "nonexistent" })`. | Test |
| FR-001-AC-3 | (RETIRED) `render` against a CompiledArchetype with data missing a required field returns `Err(QuireError::SchemaViolation)` whose first violation has the field path and references the schema's `required` constraint. | Test |
| FR-001-AC-4 | (RETIRED) A proptest exercises `render` from 64 concurrent threads against a fixed `CompiledArchetype` and asserts no panic, no UB, byte-identical outputs for identical input. | Test |
| FR-001-AC-5 | (RETIRED) Adding a new archetype to Filament (i.e., a new schema + template pair) is a data-only change — no `quire-rs` source code change is required. | Inspection |

## Dependencies

- **Upstream**: [US-001](../usecase/US-001-llm-emits-validated-patch.md), [StR-001](../stakeholder/StR-001-single-rust-engine.md)
- **Downstream**: none (retired)
