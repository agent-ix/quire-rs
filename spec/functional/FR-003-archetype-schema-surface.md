---
id: FR-003
title: "Archetype JSON Schema Surfaced to LLM Consumers"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-001"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-013"
    type: "requires"
    cardinality: "1:1"
---

## Description

`quire-rs` SHALL expose the JSON Schema document associated with any loaded archetype so that LLM tool-call definitions and downstream consumers can read the same schema the engine validates against.

```rust
pub fn schema_for<'a>(
    registry: &'a ArchetypeRegistry,
    name: &str,
) -> Result<&'a serde_json::Value, QuireError>;
```

The returned `Value` is the **same JSON Schema document loaded from Filament** ([FR-013](./FR-013-archetype-loader.md)). The engine does NOT derive schemas from Rust structs — schemas are data, owned by Filament, authored as JSON Schema draft 2020-12 files. The engine merely surfaces them.

For downstream Rust consumers who want typed bindings for a specific archetype, they MAY use `schemars` themselves (out of scope for the engine) or use `serde_json::from_value` against a hand-written typed struct. quire-rs does not generate Rust types from JSON Schemas.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-003-AC-1 | `schema_for(registry, "fr")` returns the JSON Schema document loaded from `spec-artifacts-iso/spec_artifacts_iso/schemas/fr-frontmatter.schema.json` byte-identical to the source file (modulo `serde_json` normalization of key order — see [NFR-006](../non-functional/NFR-006-determinism.md)). | Test |
| FR-003-AC-2 | `schema_for(registry, "nonexistent")` returns `Err(QuireError::UnknownArchetype)`. | Test |
| FR-003-AC-3 | An LLM agent test fixture consumes the returned schema as a tool-call argument schema; emitting a structurally-valid patch + sending to `apply_patch` succeeds; emitting a structurally-invalid patch + sending to `apply_patch` produces a `SchemaViolation` with a field path the agent can use to retry. | Test |
| FR-003-AC-4 | No Rust source file under `src/` derives schemas via `schemars::schema_for!` for archetype-specific types. The `schemars` crate is NOT a `quire-rs` dependency (verified by `Cargo.toml` audit). | Inspection |

## Dependencies

- **Upstream**: [US-001](../usecase/US-001-llm-emits-validated-patch.md), [FR-013](./FR-013-archetype-loader.md) (load-time schema compilation)
- **Downstream**: [FR-002](./FR-002-schema-validation-pipeline.md) (consumes the same compiled schema for validation)
