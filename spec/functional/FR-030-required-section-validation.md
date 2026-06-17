---
id: FR-030
title: "Required Section Completeness Validation"
type: FR
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

> **CR note (superseded by FR-032 / FR-033, ADR 0004):** The `required_sections`
> manifest field and the list-based contract described below are **retired**.
> Markdown structural validation is now performed by `validate_document` (FR-032)
> over the unified archetype's `body_extraction` asserts (FR-033), which subsume
> required-section completeness (presence at level, non-empty, no placeholder) and
> add table-column / row-count / list-item / id assertions. This FR is retained for
> history and traceability; new work targets FR-032/FR-033. See ADR 0004 and ADR 0005.
>
> **Decision note (2026-06-04):** The placeholder sentinel set was **reduced** —
> bare `none` and `n/a` are NOT sentinels (they reject legitimate content like
> `Upstream: none`). The authoritative set and matching rules live in FR-032
> (FR-032-AC-7/AC-8). Empty/header-only tables and item-less lists report reason
> **`empty`** (and absent locators report **`missing`**) — NOT `placeholder`; see
> FR-032-AC-9. The prose below is corrected to match.

## Description

`quire-rs` validation SHALL enforce manifest `required_sections` for rendered markdown artifacts in addition to frontmatter JSON Schema validation.

For any archetype with `required_sections`, validation SHALL parse the artifact body and verify that each required heading exists at the declared level, has non-empty content before the next peer-or-higher heading, and does not consist only of placeholder/default text.

Placeholder content (reason `placeholder`) SHALL include (authoritative set in
FR-032-AC-7):

- `TODO` / `TBD` (case-insensitive prefix)
- unresolved template markers — a whole-value `{{...}}`
- case-insensitive whole-value `placeholder`
- generic empty-state phrases such as whole-value `none specified`

Bare `none` and `n/a` are **not** placeholders. Empty markdown tables (header-only)
or lists (no items) are **not** reported as `placeholder` — they report reason
`empty`; a section/locator that does not resolve at all reports reason `missing`.

The engine SHALL expose this as a durable validation path used by CLI validation and library consumers. Frontmatter schema success is necessary but not sufficient for a document to validate.

Template authors SHOULD stop emitting friendly placeholder defaults for required sections. If a required input is missing, rendering SHOULD fail before producing a document that would later fail required-section validation.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-030-AC-1 | An FR document missing `## Acceptance Criteria` fails validation with a diagnostic naming that required section. | Test |
| FR-030-AC-2 | An FR document whose `## Specification` contains only `TODO` or unresolved `{{...}}` text fails validation even when frontmatter JSON Schema validation passes. | Test |
| FR-030-AC-3 | An NFR document whose `## Measurement and Evaluation` table has headers but no substantive measurement rows fails validation. | Test |
| FR-030-AC-4 | A valid FR document with all required sections populated with substantive content passes required-section validation. | Test |
| FR-030-AC-5 | Archetypes without `required_sections` continue to validate by frontmatter schema rules without a required-section diagnostic. | Test |
| FR-030-AC-6 | Diagnostics include archetype name, section name, heading level, and reason (`missing`, `empty`, or `placeholder`). | Test |

## Dependencies

- **Upstream**: FR-002, FR-005, FR-013
- **Downstream**: FR-032, FR-033 (supersede)
