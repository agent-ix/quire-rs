---
id: FR-016
title: "Secondary / Fallback Locators for Author-Variant Tolerance"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-011"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/spec-objects-business"
    type: "implements"
    cardinality: "1:1"
---

## Description

`quire-rs` SHALL support **multiple fallback locators per field** so that one DSL entry tolerates author variants (e.g. canonical `## Entities` vs. legacy `## Entity List`) without failing extraction.

### DSL shape

A locator value MAY be either:

- A **single locator** (existing shape) — `{ from: section_body, after_heading: "Entities" }`
- A **fallback chain** — `[ <locator>, <locator>, ... ]` evaluated in order; the first locator whose result is non-empty wins. If all fail, behavior follows the `required` flag (FR-011): error if required, omit if optional.

The chain semantics are deterministic and ordered — author intent dictates priority.

### Diagnostics

When a fallback chain successfully resolves via a non-first locator, the evaluator emits `Diagnostic::FallbackLocatorUsed { key, position, locator_repr }` so authoring tools can surface "this artifact uses a legacy locator" hints to upgrade documents.

### Public API

The existing `extract(doc, dsl)` (FR-011) handles fallback chains transparently. No new top-level functions. The `Locator` type internally accepts either a single primitive or a `FallbackChain(Vec<Locator>)`:

```rust
pub enum Locator {
    Single(LocatorPrimitive),
    Fallback(Vec<LocatorPrimitive>),
}
```

YAML serialization handles both shapes via untagged deserialization.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-016-AC-1 | A DSL entry `entities: [{from: section_body, after_heading: Entities}, {from: section_body, after_heading: Entity List}]` against a document with `## Entity List` resolves to the legacy section's content and emits `Diagnostic::FallbackLocatorUsed { key: "entities", position: 1 }`. | Test |
| FR-016-AC-2 | The same DSL against a document with `## Entities` resolves via the first locator and emits no fallback diagnostic. | Test |
| FR-016-AC-3 | The same DSL against a document with neither heading and `required: false` omits the key without error. | Test |
| FR-016-AC-4 | A parity test loads the `domain` object_type from `spec-objects-business` (which uses fallback chains for `entities` and `ubiquitous_language`) and confirms extraction matches the filament-parser-lib Python reference on documents using each variant. | Test |

## Dependencies

- **Upstream**: requires FR-011; implements spec-objects-business
- **Downstream**: none
