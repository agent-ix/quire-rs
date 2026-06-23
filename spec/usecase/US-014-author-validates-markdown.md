---
id: US-014
title: "LLM Authors an Artifact as Markdown and Validates It"
type: US
relationships:
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-001"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-031"
    type: "exercises"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-032"
    type: "exercises"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-033"
    type: "exercises"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-034"
    type: "exercises"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-035"
    type: "exercises"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-042"
    type: "exercises"
---

## Story

As an **LLM agent** authoring a new spec artifact, I want to write the artifact
**directly as markdown** (filling the archetype's skeleton) and call
`validate_document(archetype, doc_text)` to confirm it conforms — required
sections at the right level, tables with the right columns, ids prefixed with the
document's own id — **without a render/template step**, so that hand-authored
markdown gets a deterministic structural gate.

## Context

This complements [US-009](./US-009-llm-creates-new-artifact.md) (render-from-data). Per ADR 0004, direct-markdown is the
default authoring path; the render path ([US-009](./US-009-llm-creates-new-artifact.md), `render_by_name`) remains for
legacy template use this revision. Validation runs the unified archetype's
`body_extraction` locators in an asserting posture ([FR-031](../functional/FR-031-unified-archetype-shape.md)/[FR-032](../functional/FR-032-validate-document.md)/[FR-033](../functional/FR-033-locator-assert-facet.md)),
resolves `{field}` interpolation from frontmatter ([FR-034](../functional/FR-034-assert-field-interpolation.md)), and enforces per-level
heading uniqueness ([FR-035](../functional/FR-035-per-level-heading-uniqueness.md)). Validation and extraction share one `body_extraction`
declaration.

## Acceptance

- **US-014-AC-1**: A conformant authored FR markdown artifact passes `validate_document` with no errors (exercises [FR-031](../functional/FR-031-unified-archetype-shape.md), [FR-032](../functional/FR-032-validate-document.md)).
- **US-014-AC-2**: An artifact missing a required section, with wrong Acceptance-Criteria table columns, or an AC id not prefixed with the document's own id fails with a line-numbered diagnostic (exercises [FR-032](../functional/FR-032-validate-document.md), [FR-033](../functional/FR-033-locator-assert-facet.md), [FR-034](../functional/FR-034-assert-field-interpolation.md)).
- **US-014-AC-3**: An artifact with two same-level headings of identical text fails with a `duplicate-heading` diagnostic (exercises [FR-035](../functional/FR-035-per-level-heading-uniqueness.md)).
- **US-014-AC-4**: The same archetype's `body_extraction` both validates the document and extracts its record (one declaration, two postures) (exercises [FR-031](../functional/FR-031-unified-archetype-shape.md)).

## Efficiency Analysis

**Round trips:** 1 (author writes markdown → `validate_document` → pass/fail).

**LLM context cost:** the per-archetype skeleton + asserts ([FR-029](../functional/FR-029-archetype-input-contract.md) recast) — a
fixed per-archetype read that amortizes across many artifacts in one task; no
per-artifact JSON-context serialization.
