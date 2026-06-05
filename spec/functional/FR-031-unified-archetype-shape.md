---
id: FR-031
title: "Unified Archetype Shape"
artifact_type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-013"
    type: "extends"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-011"
    type: "requires"
    cardinality: "1:1"
---

> **CR note (no backward-compatibility — 2026-06-04):** This FR's original
> `required_sections` handling ("loads, ignored, one non-fatal diagnostic") is
> **superseded** by the project's no-deprecated-format-fallback directive (see
> filament-core FR-035 CR-002). The deprecated keys `required_sections` and
> `variants` are now **hard-rejected** (`ArchetypeLoadFailure`), not tolerated.
> The Behavior and FR-031-AC-5 below are updated to the hard-rejection contract.
>
> **CR note (render removal — 2026-06-04):** The render/templating feature is
> **removed** (no backward-compatibility layer). The unified shape drops
> `template_ref`, the `is_renderable()` accessor, and the renderability concept.
> `template_ref` joins `required_sections`/`variants` as a hard-rejected deprecated
> field. FR-031-AC-1/AC-2 below are recast from renderability assertions to the
> validate/extract surface. See `spec.md` §2bis.

## Behavior

Per ADR 0003, `quire-rs` SHALL compile every manifest archetype into a **single**
`CompiledArchetype`, removing the `artifact_type` / `object_type` distinction
from the compiled model.

A compiled archetype MAY carry, all optional except `name`:

- `frontmatter_schema_ref` — JSON Schema validating the document frontmatter (input).
- `body_extraction` — the locator DSL (FR-011), which drives **both** extraction and validation (FR-032).
- `data_schema` — JSON Schema validating the *extracted record* (a distinct validator from frontmatter).
- carry-over fields with no DSL representation: `defaults.id_pattern`, `allowed_links`, `has_plugin`, `grammar_ref`.

Validatability and extractability SHALL be **derived from which parts are present**
(a `frontmatter_schema_ref` makes the archetype validatable; a `body_extraction`
makes it extractable and structurally validatable), not from a declared kind. The
render feature is removed: there is no `template_ref` field, no `is_renderable()`
accessor, and no renderability concept on the compiled archetype.

The manifest fields `required_sections`, `variants`, and `template_ref` are
**retired**: required-section intent moves to `body_extraction` asserts (FR-033),
variants to separate archetypes (ADR 0005 #1), and `template_ref` is removed with
the render feature. A manifest that still declares any of these fields SHALL be
**rejected** with an `ArchetypeLoadFailure` naming the archetype and the
deprecated field; the archetype does NOT load. There is **no tolerate/ignore path
and no dual-read** of the deprecated shape (no backward-compatibility layer).

`Registry` lookup by archetype name (FR-013) is unchanged.

## Acceptance

- **FR-031-AC-1**: A manifest archetype declaring `frontmatter_schema_ref` + `body_extraction` compiles to one `CompiledArchetype` that is validatable (frontmatter schema) and extractable (resolvable body contract); no renderability concept is exposed.
- **FR-031-AC-2**: A manifest archetype declaring `frontmatter_schema_ref` but no `body_extraction` compiles successfully and is validatable by frontmatter schema; `body_extraction()` returns `None`.
- **FR-031-AC-3**: `defaults.id_pattern`, `allowed_links`, `has_plugin`, and `grammar_ref` are retained on the compiled archetype and readable via accessors.
- **FR-031-AC-4**: `frontmatter_schema_ref` and `data_schema` are both retained as distinct compiled validators (frontmatter vs extracted record); neither is collapsed into the other.
- **FR-031-AC-5**: A manifest archetype that still declares `required_sections`, `variants`, or `template_ref` is **rejected** with an `ArchetypeLoadFailure` naming the archetype and the deprecated field; it does NOT load (no tolerate/ignore path).
- **FR-031-AC-6**: `Registry::archetype(name)` resolves a unified archetype identically to the pre-unification path (same name keying, same first-wins semantics).
