---
id: ADR-0003
title: "Unify artifact_type and object_type into a single archetype shape"
type: ADR
---

# ADR 0003: Unify `artifact_type` and `object_type` into a single archetype shape

**Status**: Proposed
**Date**: 2026-06-04
**Decision authority**: TBD

## Context

A module manifest currently describes a renderable/extractable kind with **two
disjoint shapes** (see `src/loader/manifest.rs`, `src/loader/compile.rs`):

- **`artifact_type`** — the *render* side: `template_ref` (Jinja `.md.j2`) +
  `frontmatter_schema_ref` (JSON Schema) + passthrough `required_sections`,
  `defaults.id_pattern`, `allowed_links`, `grammar_ref`.
- **`object_type`** — the *extract* side: `data_schema` (JSON Schema over the
  extracted record) + `body_extraction` (the locator DSL) + `allowed_links`,
  `has_plugin`.

They compile to two distinct `CompiledArchetype` paths and frequently live in
two different modules (`spec-artifacts-iso` ships the FR/NFR/… artifact_types;
`spec-objects-business` ships domain/entity/… object_types). Defining "one kind
fully" — its frontmatter, its body structure, how it's read back — means
maintaining two records that cannot drift-check against each other.

Two observations make the split unnecessary:

1. The `body_extraction` DSL already *locates* every structure a validator
   needs (sections, tables, lists, code blocks) and each Locator carries
   `required`. Structural validation is "run the locators in an asserting
   posture" — so the extract side can also carry the validation contract (see
   ADR 0004).
2. With direct-markdown authoring (ADR 0004) the template stops being the
   authoring mechanism, so the artifact/object distinction collapses to "one
   archetype that has a frontmatter schema and a body contract."

## Decision

Define **one archetype shape**. An archetype carries:

- `frontmatter_schema_ref` — validates the document's frontmatter (input).
- `body_extraction` — the locator DSL, which drives **both** `extract` and
  `validate` (asserts per ADR 0004).
- `data_schema` — validates the *extracted record* (a distinct validation point
  from frontmatter; retained, not merged away).
- Carry-over fields that previously lived on only one side and have no DSL
  representation: `defaults.id_pattern` (ID allocation), `allowed_links`,
  `has_plugin`, `grammar_ref`.
- `template_ref` — **optional, legacy.** Render still works this revision;
  removal is tracked separately (ADR 0004, future template deprecation).

`required_sections` is **retired** — its intent moves into `body_extraction`
asserts. No deprecated-format fallback is added (migrate, don't dual-read).

In the loader, the two compiled paths in `src/loader/compile.rs` merge into one
`CompiledArchetype` that can hold frontmatter schema + body_extraction +
optional template + carry-over fields. `Registry` lookup is unaffected (still by
archetype name).

## Options considered

1. **Keep the split; add validation to `artifact_type` via `required_sections`.**
   Rejected: keeps two shapes and two modules per kind, and `required_sections`
   is a weaker contract than the locator DSL (no table-column/min-rows/list
   assertions, no extraction reuse). Drift between the render contract and the
   extract contract stays unmanaged.
2. **Merge everything into `object_type`, drop `artifact_type` entirely now.**
   Rejected *for this revision only* — render is still used; `template_ref`
   stays optional until template deprecation lands. The end-state is one shape
   regardless.
3. **Selected: one archetype shape, `body_extraction`-centric, template
   optional.** One declaration per kind, one compiled path, the body contract
   shared by validate and extract.

## Consequences

- `CompiledArchetype` gains optional fields (template may be absent). Renderable
  vs validatable is a property of which fields are present, surfaced via
  accessors (mirror the existing `is_renderable()`).
- `module-manifest.schema.json` (filament-core, FR-035) and `quire.validate_manifest`
  must accept the unified shape; `required_sections`/`variants` are removed from
  the manifest contract.
- `spec-artifacts-iso` archetypes are re-authored as the unified shape with
  `body_extraction` blocks for all eight kinds.
- The "where does this kind's contract live" question now has one answer.
- Conditional/variant structure is **not** carried into the unified shape — see
  ADR 0005 (#1): subtypes become separate archetypes rather than `variants` on
  one.
