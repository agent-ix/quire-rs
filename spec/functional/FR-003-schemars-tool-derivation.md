---
id: FR-003
title: "schemars-Derived JSON Schema for LLM Tool Definitions"
artifact_type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-001"
    type: "implements"
    cardinality: "1:1"
---

## Behavior

For each archetype struct, `quire-rs` SHALL expose a function:

```rust
fn schema_for(block_type: &str) -> Result<serde_json::Value, QuireError>
```

that returns a JSON Schema draft 2020-12 document derived via `schemars::schema_for!`. The derived schema SHALL match the existing reference schema in `spec-artifacts-iso/spec_artifacts_iso/schemas/<type>-frontmatter.schema.json` at the level of:

- Required fields
- Property types (string, integer, array, object)
- Pattern constraints (e.g. `^[A-Z]{2,4}-[0-9]+$` for id, `^ix://` for relationship targets)
- Const constraints (e.g. `artifact_type: "FR"`)
- Min-length / max-length constraints

Differences between the Rust-derived schema and the existing reference SHALL be documented in `spec/assets/schema-parity-notes.md`. The Rust-derived schema is the source of truth going forward.

## Acceptance

- **FR-003-AC-1**: `schema_for("fr")` returns a JSON Schema whose `required` array contains `id`, `title`, `artifact_type`.
- **FR-003-AC-2**: The `id` property's `pattern` is `^[A-Z]{2,4}-[0-9]+$`.
- **FR-003-AC-3**: The `artifact_type` property's `const` is `"FR"`.
- **FR-003-AC-4**: A snapshot test serializes the derived schema and compares byte-by-byte against the reference `fr-frontmatter.schema.json` (modulo whitespace), or documents the diff in `schema-parity-notes.md`.
