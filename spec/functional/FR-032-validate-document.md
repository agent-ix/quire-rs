---
id: FR-032
title: "Markdown Document Validation (validate_document)"
artifact_type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-002"
    type: "extends"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-005"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-011"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-030"
    type: "supersedes"
    cardinality: "1:1"
---

## Behavior

Per ADR 0004, `quire-rs` SHALL expose `validate_document(archetype, doc_text) ->
ValidationResult` that validates an authored **markdown document** against a
unified archetype (FR-031). This is the **default** validation path; the
pre-existing context/data validation (a JSON object against the archetype schema,
FR-002) remains available as a distinct, explicitly selected path.

`validate_document` SHALL:

1. Parse the document via `parse_document` (FR-005).
2. Validate the frontmatter against `frontmatter_schema_ref`.
3. Run the archetype's `body_extraction` locators in an **asserting posture**: each `required: true` locator MUST resolve to content that is non-empty and not placeholder-only; each locator's optional `assert` facet (FR-033) MUST hold; `{field}` interpolation (FR-034) is resolved during assert evaluation.
4. Enforce the per-level heading-uniqueness rule (FR-035).

Placeholder content (rejected in required locators) SHALL include `TODO`, `TBD`,
unresolved `{{...}}`, case-insensitive `placeholder`, generic empty-state phrases
such as `none specified`, and empty tables/lists with no substantive cells/items.

Frontmatter-schema success is **necessary but not sufficient**. Diagnostics SHALL
be line-numbered and name the archetype, the offending locator/section, and a
reason (`missing`, `empty`, `placeholder`, `assert`, `frontmatter`, or
`duplicate-heading`). The function is the single engine entry point; CLI,
wheel, and wasm surfaces call it without re-implementing logic.

## Acceptance

- **FR-032-AC-1**: A conformant FR document (all required locators resolved, asserts satisfied, frontmatter valid) returns `is_valid == true` with no errors.
- **FR-032-AC-2**: A document missing a `required` section fails with a line-numbered diagnostic naming the archetype, the section, and reason `missing`.
- **FR-032-AC-3**: A document whose required `## Specification` contains only `TODO`/`{{...}}` fails with reason `placeholder`, even when frontmatter JSON Schema validation passes.
- **FR-032-AC-4**: Frontmatter that violates `frontmatter_schema_ref` fails with reason `frontmatter`, independent of body structure.
- **FR-032-AC-5**: The markdown path (`validate_document`) and the legacy context/data path (FR-002) are distinct entry points; selecting the context path validates a JSON object and does not parse markdown.
- **FR-032-AC-6**: An archetype with no `body_extraction` validates by frontmatter schema + heading-uniqueness only, emitting no body-structure diagnostics.
