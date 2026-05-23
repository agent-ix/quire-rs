---
id: FR-002
title: "Schema Validation Pipeline: JSON Merge → Compiled Validator → Render"
artifact_type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-001"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-004"
    type: "implements"
    cardinality: "1:1"
---

## Behavior

`quire-rs` SHALL expose:

```rust
pub fn apply_patch(
    archetype: &CompiledArchetype,
    current: &serde_json::Value,
    patch: &serde_json::Value,
) -> Result<serde_json::Value, QuireError>;
```

The function operates on JSON values directly (`serde_json::Value`); it does NOT require typed Rust structs per archetype. Behavior:

1. **Merge** `patch` onto `current` using deep JSON merge semantics: patch wins per-key; nested objects merge recursively; arrays are replaced wholesale. Merge is a pure function over JSON values.
2. **Validate** the *merged result* — never the patch in isolation — against the archetype's pre-compiled JSON Schema validator. Cross-field invariants and `required` constraints depend on the full merged shape.
3. On success, return the merged-and-validated `Value`. On failure, return `QuireError::SchemaViolation` with the merged-shape field path and the violating constraint.

Patches MAY add new fields to objects whose schema allows them. Patches MAY NOT introduce fields the schema disallows (`additionalProperties: false`); those raise `SchemaViolation`.

The JSON Schema validator implementation uses a high-performance pre-compiled validator (e.g. the `jsonschema` crate's `JSONSchema::compile()` once, then `validate()` per call). Compilation happens at archetype load time (FR-013), not per `apply_patch` call.

## Acceptance

- **FR-002-AC-1**: Given a `Value` containing `{ "title": "old", "body": "content" }` and a patch `{ "title": "new" }`, the merged-validated result is `{ "title": "new", "body": "content" }` — array-and-object merge semantics preserve siblings.
- **FR-002-AC-2**: Given a current with `title: "valid"` and a patch `{ "title": "" }` against a schema requiring `title` `minLength: 1`, the merged-validated call returns `SchemaViolation` with field path `title` — caught because the merge produced an invalid `title`, even though the patch alone has the field set.
- **FR-002-AC-3**: A patch introducing an unknown key on an object where the schema sets `additionalProperties: false` raises `SchemaViolation` naming the unknown key.
- **FR-002-AC-4**: A proptest fuzzes patches across all archetypes in the test corpus and confirms `apply_patch` returns a valid `Value` (per schema) or a typed error — never a panic.
- **FR-002-AC-5**: Per-call cost of `apply_patch` (excluding schema-compile, which is amortized at load) is dominated by JSON merge and validation; criterion bench shows median below 100 µs for a typical (~4 KB) artifact.
