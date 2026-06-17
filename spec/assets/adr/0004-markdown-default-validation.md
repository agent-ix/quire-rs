---
id: ADR-0004
title: "Direct-markdown authoring; body_extraction asserts as the structural validator"
type: ADR
---

# ADR 0004: Direct-markdown authoring; `body_extraction` asserts as the structural validator

**Status**: Proposed
**Date**: 2026-06-04
**Decision authority**: TBD

## Context

Spec authoring has been "an agent produces a JSON context → `quire render`
expands a Jinja template → markdown." Empirical evaluation (n=5 per condition
plus a 40-trial length×encoding sweep) found that **authoring markdown directly**
ties the best schema-driven render path on wall-time, is cheaper on tokens once
the per-archetype contract amortizes across many artifacts in one task, and that
structured authoring of long tables is reliable on the current model. The only
capability the render path provided that direct authoring lacked was
**deterministic structural validation** of the produced markdown.

`quire-rs` already specs that validation on the *older* `required_sections`
model:

- [FR-030](../../functional/FR-030-required-section-validation.md) (Required Section Completeness Validation): parse the body, enforce
  each `required_sections` heading at its level, non-empty, no placeholder text;
  frontmatter-schema success is necessary-but-not-sufficient. Specced, unimplemented.
- [FR-029](../../functional/FR-029-archetype-input-contract.md) (Archetype Input Contract Surface): expose a per-archetype contract
  (frontmatter schema + required_sections + template variables) so an agent can
  populate every section before rendering. Specced, unimplemented.

ADR 0003 unifies the archetype shape around `body_extraction`. The locator DSL
already addresses sections/tables/lists/code-blocks and carries `required`, so
it is a strictly richer structural contract than the `required_sections` list
(it can assert table columns, row counts, list-item counts, heading level, and
id patterns — see ADR 0005 #3).

## Decision

1. **`body_extraction` is the structural validator.** Validation runs the
   archetype's locators in an asserting posture: each `required: true` locator
   must resolve, and an optional `assert:` facet on a locator checks shape
   (`level`, `columns`, `min_rows`, `min_items`, `id_column`, `id_pattern`).
   `assert` is ignored by the extractor and validated at load time. A new
   `validate_document(archetype, doc_text) -> ValidationResult` (engine API,
   `src/validate_document.rs`) performs `parse_document` + locator asserts +
   the per-level heading-uniqueness rule (ADR 0005 #4), emitting line-numbered
   diagnostics.

2. **Markdown validation is the default.** The default `validate` path takes a
   **markdown document** and runs `validate_document` + frontmatter-schema
   validation. The pre-existing context/data validation (validate a JSON object
   against the archetype schema) remains available behind an explicit `--json`
   selector at the CLI / a distinct binding.

3. **This supersedes [FR-030](../../functional/FR-030-required-section-validation.md) and recasts [FR-029](../../functional/FR-029-archetype-input-contract.md).** [FR-030](../../functional/FR-030-required-section-validation.md)'s
   `required_sections` model is replaced by `body_extraction` asserts. [FR-029](../../functional/FR-029-archetype-input-contract.md)'s
   "input contract" becomes a **skeleton/example** derived from the frontmatter
   schema + the body_extraction asserts (no template dependency) — what the
   authoring skill hands the agent. Both supersessions are recorded as CR notes
   on the existing FRs (CR-002 pattern, [FR-024](../../functional/FR-024-parallel-repo-walk.md)), not silent edits.

`validate_document` is an engine/API function in `quire-rs`. Every surface —
quire-cli (`quire validate <doc.md>`), `filament_parser` via the wheel,
quire-wasm — is a thin caller of the same function; no validation logic is
duplicated in a surface.

## Options considered

1. **Implement [FR-030](../../functional/FR-030-required-section-validation.md) as specced (`required_sections` list).** Rejected: ADR
   0003 retires `required_sections`; the list cannot express tables/lists/ids,
   and it would be a second contract beside `body_extraction`.
2. **Keep render-from-context as the authoring path; validate the context.**
   Rejected on the evaluation result — direct authoring is equal-or-cheaper and
   removes the template as a maintained artifact (future deprecation).
3. **Selected: markdown-default validation via `body_extraction` asserts**,
   `--json` for the legacy context path.

## Consequences

- **Structure is checked, not guaranteed by construction.** A template made a
  malformed artifact unrepresentable; validation catches it post-hoc with an
  author/re-author loop. Accepted per the evaluation; a canonicalizing markdown
  formatter is a possible future addition (also see ADR 0005 #2, section order).
- [FR-029](../../functional/FR-029-archetype-input-contract.md)/[FR-030](../../functional/FR-030-required-section-validation.md) are marked superseded with pointers here; new FRs specify
  the assert facet, interpolation, heading-uniqueness, and `validate_document`.
- The authoring skill (`/specify`) changes from "produce JSON context → render"
  to "fill the skeleton → write markdown → `quire validate` loop."
- `spec/tests.md` AC→TC coverage must extend to the new validation FRs.
