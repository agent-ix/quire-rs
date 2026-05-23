---
id: FR-001
title: "Render Dispatch: Generic Engine Over (Schema, Template, Data)"
artifact_type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-001"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-001"
    type: "implements"
    cardinality: "1:1"
---

## Behavior

`quire-rs` SHALL expose a generic render API that knows nothing about specific archetypes — `FR`, `NFR`, `ADR`, etc. are data shipped by Filament, not types compiled into the engine.

The primary entry point:

```rust
pub fn render(archetype: &CompiledArchetype, data: &serde_json::Value)
    -> Result<String, QuireError>;
```

where `CompiledArchetype` carries:
- A pre-compiled JSON Schema validator (built from a JSON Schema document loaded at archetype-load time — see FR-013)
- A pre-parsed MiniJinja template (compiled once at load time)
- A stable archetype name + version for diagnostics

The function SHALL:

1. Validate `data` against the compiled JSON Schema. Violations return `QuireError::SchemaViolation` with a field-keyed list (see NFR-005).
2. Render the validated `data` through the pre-parsed MiniJinja template using the long-lived strict-undefined `Environment` (see FR-004).
3. Return the rendered markdown on success.

The engine SHALL NOT panic on any input. It SHALL be re-entrant and safe for concurrent calls. A `CompiledArchetype` is `Send + Sync` and may be shared across threads.

A convenience function `render_by_name(registry: &ArchetypeRegistry, name: &str, data: &Value)` looks up the archetype by name and dispatches to `render`. Name resolution is the responsibility of the registry (FR-013, FR-014).

## Acceptance

- **FR-001-AC-1**: For each archetype in the Filament v1 corpus (currently `spec-artifacts-iso` + `spec-artifacts-app` + `spec-artifacts-process` = 17 types), feeding the same `data` JSON through `quire-rs::render` and the Python Jinja2 reference produces byte-identical output.
- **FR-001-AC-2**: `render_by_name(registry, "nonexistent", ...)` returns `Err(QuireError::UnknownArchetype { name: "nonexistent" })`.
- **FR-001-AC-3**: `render` against a CompiledArchetype with data missing a required field returns `Err(QuireError::SchemaViolation)` whose first violation has the field path and references the schema's `required` constraint.
- **FR-001-AC-4**: A proptest exercises `render` from 64 concurrent threads against a fixed `CompiledArchetype` and asserts no panic, no UB, byte-identical outputs for identical input.
- **FR-001-AC-5**: Adding a new archetype to Filament (i.e., a new schema + template pair) is a **data-only change** — no `quire-rs` source code change is required.
