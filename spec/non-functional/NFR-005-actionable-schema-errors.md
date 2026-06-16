---
id: NFR-005
title: "Schema Validation Errors Are Field-Keyed and Actionable"
type: NFR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-001"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-002"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-001"
    type: "implements"
    cardinality: "1:1"
---

## Statement

Every `QuireError::SchemaViolation` returned by `quire-rs` SHALL carry, at minimum:

1. **Field path** — dot-notated path from the typed root to the offending field (e.g. `data.relationships[0].target`).
2. **Expected** — human-readable description of the constraint that failed (e.g. `pattern ^ix://`, `min length 1`, `enum FR | NFR | StR | US`).
3. **Observed** — the value (or value preview) that violated the constraint. Long strings are truncated at 80 characters with an ellipsis. Bytes are rendered as a hex preview.
4. **Block type** — the registered archetype name (e.g. `"fr"`) so the error is self-locating in a batch-render context.

Errors SHALL NOT leak raw `serde_json::Error`, `jsonschema::ValidationError` (or whichever validator crate is selected), or stack traces in their public `Display` form. A single helper `format_violation(violation)` produces the user-facing string.

## Rationale

LLM-driven editors (US-001) retry on validation failure. The minimum information needed for the retry to converge is the four-tuple above; without any one of them the model is guessing. Field-keyed messages also surface cleanly in editor UI without further parsing.

## Acceptance Criteria

- **NFR-005-AC-1**: Triggering each kind of violation (missing required, wrong type, pattern mismatch, length, enum) produces an error whose `Display` form contains all four elements.
- **NFR-005-AC-2**: A static check confirms `QuireError::Display` does not contain literal substrings from `serde_json::Error` or the validator crate's native debug forms.
- **NFR-005-AC-3**: A snapshot test pins one canonical error per archetype for stability.

## Verification

- Unit tests in `tests/error_shape.rs` exercise every error variant the codebase emits.
