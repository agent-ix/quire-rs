---
id: FR-001
title: "Render Dispatch: Block Type → Schema-Validated Render"
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

`quire_rs::render(block_type: &str, data: serde_json::Value) -> Result<String, QuireError>` SHALL:

1. Look up `block_type` in the archetype registry — return `QuireError::UnknownBlockType` if not found.
2. Deserialize `data` into the archetype's typed struct via `serde_json::from_value`. Deserialization errors return `QuireError::SchemaViolation` with the field path.
3. Run field-level and cross-field validators via `garde::Validate::validate`. Validation errors return `QuireError::SchemaViolation` with a list of field-keyed messages.
4. Pass the validated typed value as the template context to the archetype's MiniJinja template, rendered from the long-lived `Environment` configured with `UndefinedBehavior::Strict`.
5. Return the rendered markdown string on success.

The function SHALL NOT panic on any input. The function SHALL be re-entrant and safe for concurrent calls.

## Acceptance

- **FR-001-AC-1**: `render("fr", valid_fr_value)` returns `Ok(markdown)` where `markdown` byte-equals the Python Jinja2 reference output for the same input.
- **FR-001-AC-2**: `render("nonexistent", ...)` returns `Err(QuireError::UnknownBlockType { name: "nonexistent" })`.
- **FR-001-AC-3**: `render("fr", value_missing_required_title)` returns `Err(QuireError::SchemaViolation)` whose first violation has field path `data.title` and the message references the missing field.
- **FR-001-AC-4**: `render("fr", value_with_template_field_unknown_to_schema)` returns `Ok(markdown)` if the schema accepts the value (additionalProperties: true) — template-side missing-field errors are caught by strict mode separately (see FR-004).
- **FR-001-AC-5**: A proptest exercises `render` from 1000 concurrent threads with a mix of valid and invalid inputs; no panic, no UB, no data race.
