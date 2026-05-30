---
id: FR-030
title: "Required Section Completeness Validation"
artifact_type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-002"
    type: "extends"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-005"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-013"
    type: "requires"
    cardinality: "1:1"
---

## Behavior

`quire-rs` validation SHALL enforce manifest `required_sections` for rendered markdown artifacts in addition to frontmatter JSON Schema validation.

For any archetype with `required_sections`, validation SHALL parse the artifact body and verify that each required heading exists at the declared level, has non-empty content before the next peer-or-higher heading, and does not consist only of placeholder/default text.

Placeholder content SHALL include:

- `TODO`
- `TBD`
- unresolved template markers such as `{{...}}`
- case-insensitive `placeholder`
- generic empty-state phrases such as `none specified`
- empty markdown tables or lists with no substantive cells/items

The engine SHALL expose this as a durable validation path used by CLI validation and library consumers. Frontmatter schema success is necessary but not sufficient for a document to validate.

Template authors SHOULD stop emitting friendly placeholder defaults for required sections. If a required input is missing, rendering SHOULD fail before producing a document that would later fail required-section validation.

## Acceptance

- **FR-030-AC-1**: An FR document missing `## Acceptance Criteria` fails validation with a diagnostic naming that required section.
- **FR-030-AC-2**: An FR document whose `## Specification` contains only `TODO` or unresolved `{{...}}` text fails validation even when frontmatter JSON Schema validation passes.
- **FR-030-AC-3**: An NFR document whose `## Measurement and Evaluation` table has headers but no substantive measurement rows fails validation.
- **FR-030-AC-4**: A valid FR document with all required sections populated with substantive content passes required-section validation.
- **FR-030-AC-5**: Archetypes without `required_sections` continue to validate by frontmatter schema rules without a required-section diagnostic.
- **FR-030-AC-6**: Diagnostics include archetype name, section name, heading level, and reason (`missing`, `empty`, or `placeholder`).
