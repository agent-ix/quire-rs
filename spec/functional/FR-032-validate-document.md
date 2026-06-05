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

### Placeholder sentinel set (decision — 2026-06-04)

A required locator's resolved value is **placeholder** (reason `placeholder`) when,
after trimming surrounding whitespace, it matches the sentinel set below. The set is
**reduced** from an earlier draft: bare `none` and `n/a` are **NOT** sentinels —
they reject legitimate content such as `Upstream: none`. The exact set and matching
rules are:

- `TODO` / `TBD` — case-insensitive, matched as a **prefix** (`starts-with`): a value
  that begins with `TODO`/`TBD` (e.g. `TODO: fill this in`) is placeholder.
- `{{...}}` — an unresolved template marker matched as the **whole value**: the entire
  trimmed value is a single `{{ … }}` expression. A `{{…}}` embedded in otherwise
  substantive prose is NOT placeholder.
- `placeholder` — case-insensitive, whole-value.
- `none specified` — case-insensitive, whole-value generic empty-state phrase.
- empty value — a resolved locator whose trimmed content is the empty string.

Empty/header-only **tables** and item-less **lists** are NOT reported as
`placeholder`; they are reported as `empty` (a table with a header row but no data
rows, or a list with no items) or `missing` (the section/locator did not resolve at
all). See FR-032-AC-9.

Frontmatter-schema success is **necessary but not sufficient**. Diagnostics SHALL
be line-numbered and name the archetype, the offending locator/section, and a
reason (`missing`, `empty`, `placeholder`, `assert`, `frontmatter`, or
`duplicate-heading`). The function is the single engine entry point; CLI,
wheel, and wasm surfaces call it without re-implementing logic.

## Acceptance

- **FR-032-AC-1**: A conformant FR document (all required locators resolved, asserts satisfied, frontmatter valid) returns `is_valid == true` with no errors.
- **FR-032-AC-2**: A document missing a `required` section fails with a diagnostic naming the archetype, the section, and reason `missing`. The diagnostic's `line` field is `Option<usize>` and is **`None`** when the section is wholly absent (no heading to point at); it is `Some(n)` when the locator resolved to a place but the content is empty/placeholder. (Parity with quire-cli CR-003.)
- **FR-032-AC-3**: A document whose required `## Specification` contains only `TODO`/`{{...}}` fails with reason `placeholder`, even when frontmatter JSON Schema validation passes.
- **FR-032-AC-4**: Frontmatter that violates `frontmatter_schema_ref` fails with reason `frontmatter`, independent of body structure.
- **FR-032-AC-5**: The markdown path (`validate_document`) and the legacy context/data path (FR-002) are distinct entry points; selecting the context path validates a JSON object and does not parse markdown.
- **FR-032-AC-6**: An archetype with no `body_extraction` validates by frontmatter schema + heading-uniqueness only, emitting no body-structure diagnostics.
- **FR-032-AC-7**: The placeholder sentinel set is exactly `{TODO, TBD (case-insensitive prefix), {{…}} (whole-value), placeholder (whole-value, case-insensitive), none specified (whole-value, case-insensitive), empty}`. A required section whose content is `TODO: …` or a whole-value `{{ id }}` fails with reason `placeholder`; a section whose content is substantive prose that merely contains the word `todo` mid-sentence or an embedded `{{x}}` token does **not**.
- **FR-032-AC-8**: A required section whose only content is `none` or `n/a` (e.g. `Upstream: none`) is treated as **substantive** and passes (these are not sentinels) — proving the reduced set does not reject legitimate content.
- **FR-032-AC-9**: A required `table_row` locator resolving to a header-only table fails with reason `empty` (not `placeholder`); a required `list_item` locator resolving to a list with no items fails with reason `empty`; a locator that does not resolve at all fails with reason `missing`.
- **FR-032-AC-10**: An `assert` facet on a **resolved** locator is evaluated regardless of the locator's `required` flag: an optional (`required: false`) locator that resolves to content still has its `assert` checked (failing → reason `assert`); when the optional locator does not resolve, no assert runs and no diagnostic is emitted.
