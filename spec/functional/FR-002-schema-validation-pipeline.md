---
id: FR-002
title: "Schema Validation Pipeline: Merge-Then-Validate Semantics"
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

When `quire_rs::apply_patch(current: TypedBlock, patch: serde_json::Value) -> Result<TypedBlock, QuireError>` merges a partial patch onto a current block:

1. The patch is merged into the current block's `data` using deep JSON merge semantics — patch wins per-key; nested objects merge recursively; arrays are replaced wholesale.
2. The **merged result** is then re-validated against the schema — never the patch in isolation. Cross-field invariants depend on the full value; validating only the patch would miss violations introduced by the merge.
3. Successful validation returns the typed `TypedBlock` with the new data. Failure returns `QuireError::SchemaViolation` referencing the merged shape.

Patches MAY add new fields to optional objects. Patches MAY NOT add fields the schema disallows (`additionalProperties: false` for nested objects) — those raise validation errors.

## Acceptance

- **FR-002-AC-1**: Given an `FrData` with `relationships: [{target, type, cardinality}]` and a patch updating only `relationships[0].cardinality`, the merged-validated result has all three fields preserved.
- **FR-002-AC-2**: Given a current `FrData` with `title: "valid"` and a patch `{ title: "" }`, the merged-validated result returns `QuireError::SchemaViolation { field: "data.title", message: "title must be at least 1 character" }` — caught because the merge made the field invalid even though the patch alone has the field set.
- **FR-002-AC-3**: A patch that introduces an unknown key on a relationship object (where `additionalProperties: false`) raises a validation error naming the unknown key.
