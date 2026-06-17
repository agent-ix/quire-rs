---
id: FR-035
title: "Per-Level Heading Uniqueness"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-032"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-005"
    type: "requires"
    cardinality: "1:1"
---

## Description

Per ADR 0005 (#4), `validate_document` (FR-032) SHALL enforce **per-level heading
uniqueness**: within a single document, no two headings at the **same heading
level** may share identical heading text.

This rule makes the flat, name-based locator addressing (`after_heading`,
`under_section`) unambiguous: `(heading text, level)` is unique, and since asserts
can pin `level` (FR-033), a locator resolves to exactly one section. Headings with
the same text at *different* levels are permitted (the level disambiguates).

The rule is compatible with multi-yield extraction (`iterate_over`, FR-011):
iterated child units (e.g. `### Algorithm A`, `### Algorithm B`) carry **distinct**
heading text and therefore do not collide.

Violations SHALL produce a line-numbered diagnostic with reason
`duplicate-heading`, naming both the repeated text and the level.

> Scope note: this is per-level (global within the document at each level), not
> per-parent. ADR 0005 records the intent to relax to per-parent (sibling)
> uniqueness once recursive path-scoping lands; until then per-level is the rule
> that keeps name-based addressing sound.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-035-AC-1 | A document with two `## Description` headings fails with reason `duplicate-heading` naming the text and level 2. | Test |
| FR-035-AC-2 | A document with a `## Properties` (level 2) and a `### Properties` (level 3) passes the uniqueness rule (different levels). | Test |
| FR-035-AC-3 | A document using `iterate_over` with distinct child headings (`### A`, `### B`) passes; introducing a duplicate `### A` fails. | Test |
| FR-035-AC-4 | The duplicate-heading diagnostic includes a line number for the offending (second) heading. | Test |

## Dependencies

- **Upstream**: FR-032 (requires), FR-005 (requires)
- **Downstream**: none
