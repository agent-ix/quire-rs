---
id: FR-032
title: "Markdown Document Validation (validate_document)"
type: FR
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

> **CR note (composed type+object validation — 2026-06-16):** `validate_document`
> now validates BOTH the `type` archetype (the artifact archetype) AND the
> frontmatter `object:` archetype (an object archetype from a spec-objects-*
> module), when an `object:` key is present. Object resolution requires the
> registry/module set, so the composed entry point is
> `validate_document_in_registry(registry, archetype, doc_text)`; the 2-arg
> `validate_document(archetype, doc_text)` remains the type-only path (no
> registry → no object layer) for callers without a registry. Composition is
> **always on** when the registry-aware entry point is used: `object:` absent ⇒
> no object diagnostics, exactly as before. A resolved object archetype's
> `body_extraction` asserts run in the same asserting posture as the `type`
> archetype and merge into the same **error** list. An `object:` naming an
> archetype the registry does not know is a **warning** (advisory), not an
> error. `ValidationResult` gains a typed `warnings: Vec<ValidationWarning>`
> field alongside `errors`; `is_valid` stays `errors.is_empty()` (warnings never
> fail validation). See AC-11..13 and the new `ValidationReason::UnknownObjectType`.

## Description

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
5. **(composed, registry-aware entry point only)** If frontmatter carries an `object:` string, resolve an archetype by that name from the **same registry/module set** used to resolve `type` (across artifact_types AND object_types). When resolved, run its `body_extraction` asserts in the same asserting posture (step 3) and merge any failures into `errors`. When the name is unknown to the registry, emit one **warning** (reason `unknown-object-type`) — never an error. When `object:` is absent, no object-layer work happens.

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
`duplicate-heading`). Object-layer **warnings** carry reason `unknown-object-type`.
The function is the single engine entry point; CLI,
wheel, and wasm surfaces call it without re-implementing logic.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-032-AC-1 | A conformant FR document (all required locators resolved, asserts satisfied, frontmatter valid) returns `is_valid == true` with no errors. | Test |
| FR-032-AC-2 | A document missing a `required` section fails with a diagnostic naming the archetype, the section, and reason `missing`. The diagnostic's `line` field is `Option<usize>` and is `None` when the section is wholly absent (no heading to point at); it is `Some(n)` when the locator resolved to a place but the content is empty/placeholder. (Parity with quire-cli CR-003.) | Test |
| FR-032-AC-3 | A document whose required `## Specification` contains only `TODO`/`{{...}}` fails with reason `placeholder`, even when frontmatter JSON Schema validation passes. | Test |
| FR-032-AC-4 | Frontmatter that violates `frontmatter_schema_ref` fails with reason `frontmatter`, independent of body structure. | Test |
| FR-032-AC-5 | The markdown path (`validate_document`) and the legacy context/data path (FR-002) are distinct entry points; selecting the context path validates a JSON object and does not parse markdown. | Test |
| FR-032-AC-6 | An archetype with no `body_extraction` validates by frontmatter schema + heading-uniqueness only, emitting no body-structure diagnostics. | Test |
| FR-032-AC-7 | The placeholder sentinel set is exactly `{TODO, TBD (case-insensitive prefix), {{…}} (whole-value), placeholder (whole-value, case-insensitive), none specified (whole-value, case-insensitive), empty}`. A required section whose content is `TODO: …` or a whole-value `{{ id }}` fails with reason `placeholder`; a section whose content is substantive prose that merely contains the word `todo` mid-sentence or an embedded `{{x}}` token does not. | Test |
| FR-032-AC-8 | A required section whose only content is `none` or `n/a` (e.g. `Upstream: none`) is treated as substantive and passes (these are not sentinels) — proving the reduced set does not reject legitimate content. | Test |
| FR-032-AC-9 | A required `table_row` locator resolving to a header-only table fails with reason `empty` (not `placeholder`); a required `list_item` locator resolving to a list with no items fails with reason `empty`; a locator that does not resolve at all fails with reason `missing`. | Test |
| FR-032-AC-10 | An `assert` facet on a resolved locator is evaluated regardless of the locator's `required` flag: an optional (`required: false`) locator that resolves to content still has its `assert` checked (failing → reason `assert`); when the optional locator does not resolve, no assert runs and no diagnostic is emitted. | Test |
| FR-032-AC-11 | When a document carries a frontmatter `object:` string naming an archetype the registry resolves (across artifact_types AND object_types), `validate_document_in_registry` runs that object archetype's `body_extraction` asserts in the same asserting posture as the `type` archetype; any failures are errors (reason `missing`/`empty`/`placeholder`/`assert`), merged into the same `errors` list as the type errors (so they fail validation / exit 1) and tagged with the object archetype name. The `type` archetype path is unchanged and still always produces hard errors. An `object:` absent ⇒ no object-layer diagnostics (type-only, exactly as the 2-arg path). | Test |
| FR-032-AC-12 | When a document carries a frontmatter `object:` naming an archetype the registry does not resolve, `validate_document_in_registry` emits exactly one warning (reason `unknown-object-type`, message `unknown object type '<x>' declared in frontmatter object`) into `ValidationResult::warnings` — not an error. `is_valid` remains `true` for an otherwise-conformant document (warnings are advisory and never fail validation). | Test |
| FR-032-AC-13 | `ValidationResult` exposes both `errors: Vec<ValidationError>` and `warnings: Vec<ValidationWarning>`; `is_valid == errors.is_empty()` regardless of warnings. A document that is type-conformant and object-conformant returns no object errors; a document whose object archetype's required mermaid `diagram` body_extraction is missing returns an object error while the `type` portion still passes independently. | Test |

## Dependencies

- **Upstream**: FR-002 (extends), FR-005 (requires), FR-011 (requires), FR-030 (supersedes)
- **Downstream**: FR-033, FR-034, FR-035, FR-037, FR-038
