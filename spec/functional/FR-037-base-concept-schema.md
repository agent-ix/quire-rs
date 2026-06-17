---
id: FR-037
title: "Base Concept Frontmatter Schema (OKF)"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-032"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-002"
    type: "requires"
    cardinality: "1:1"
---

## Description

`quire-rs` SHALL provide a shared **base "concept" frontmatter schema** — the
contract every authored document satisfies *before* archetype routing. Under the
Open Knowledge Format (OKF), `type` is the discriminator that selects which
archetype validates a document; this module owns the schema that *requires* and
*types* the OKF base fields uniformly across every surface and module, with no
per-module schema duplication.

The base fields are:

- **`type`** — a string, the OKF discriminator. Required and non-empty where
  routing depends on it (`minLength: 1`).
- **`description`** — optional string; type-checked when present.
- **`tags`** — optional array of strings; type-checked (each item) when present.

`additionalProperties` is left **open**: the archetype-specific schema, run
afterward ([FR-002](./FR-002-schema-validation-pipeline.md) / [FR-032](./FR-032-validate-document.md) frontmatter validation), owns the rest of the
frontmatter. A base-concept violation is a `ValidationError` carrying reason
`frontmatter` — never a soft warning and never a CLI bail.

### Two entry points

The required-`type` posture and the typing-only posture are exposed as two
functions so the same field typing can be reused where presence of `type` is or
is not mandatory:

```rust
pub fn base_concept_schema() -> serde_json::Value;          // type REQUIRED + non-empty
pub fn validate_base_concept(frontmatter) -> Vec<ValidationError>;   // required `type`
pub fn validate_concept_shape(frontmatter) -> Vec<ValidationError>;  // typing only
```

- **`validate_base_concept`** — validates against the base schema (`type`
  required + non-empty, `description`/`tags` typed). Used where the document is
  routed *by* its `type` — the corpus / bundle layer ([FR-038](./FR-038-okf-bundle-validation.md)).
- **`validate_concept_shape`** — validates the same field typing **without**
  requiring `type`. It is wired into `validate_document` ([FR-032](./FR-032-validate-document.md)) so a document
  validated via an explicit `--archetype` override MAY legitimately carry no
  `type` and still pass, preserving the `quire-cli` [FR-004-AC-5](./FR-004-minijinja-strict-environment.md) behavior; its
  optional `description`/`tags` are still typed.

`base_concept_schema`, `validate_base_concept`, and `validate_concept_shape` are
exported from the crate root.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-037-AC-1 | `validate_base_concept` on frontmatter carrying a non-empty `type` (e.g. `type: FR`) and no other fields returns no errors. | Test |
| FR-037-AC-2 | `validate_base_concept` on frontmatter with a non-empty `type`, a string `description`, and a `tags` array of strings returns no errors (the optional OKF fields are accepted when well-typed). | Test |
| FR-037-AC-3 | `validate_base_concept` on frontmatter that omits `type` returns exactly one `ValidationError` with reason `frontmatter` whose message names `type`. | Test |
| FR-037-AC-4 | `validate_base_concept` on frontmatter with an empty `type` (`type: ""`) returns exactly one `ValidationError` with reason `frontmatter` (the `minLength: 1` constraint). | Test |
| FR-037-AC-5 | `validate_base_concept` on frontmatter with a non-string `description` (e.g. `description: 7`) returns one error naming `description`; with a non-array `tags` (e.g. `tags: "x"`) returns one error naming `tags`; with a `tags` array containing a non-string item returns one error. | Test |
| FR-037-AC-6 | `validate_document` ([FR-032](./FR-032-validate-document.md)) runs `validate_concept_shape` on the parsed frontmatter, so a conformant document whose `type` is present and whose `description`/`tags` (if any) are well-typed validates without a base-field error — confirming the shape check is wired into the per-document path and does not reject well-formed OKF base fields. (`validate_concept_shape` does not require `type`, preserving the `--archetype`-override / [FR-004-AC-5](./FR-004-minijinja-strict-environment.md) typeless-document behavior.) | Test |

## Dependencies

- **Upstream**: [FR-032](./FR-032-validate-document.md) (requires), [FR-002](./FR-002-schema-validation-pipeline.md) (requires)
- **Downstream**: [FR-038](./FR-038-okf-bundle-validation.md)
