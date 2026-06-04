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

## Behavior

Per ADR 0003, `quire-rs` SHALL compile every manifest archetype into a **single**
`CompiledArchetype`, removing the `artifact_type` / `object_type` distinction
from the compiled model.

A compiled archetype MAY carry, all optional except `name`:

- `frontmatter_schema_ref` — JSON Schema validating the document frontmatter (input).
- `body_extraction` — the locator DSL (FR-011), which drives **both** extraction and validation (FR-032).
- `data_schema` — JSON Schema validating the *extracted record* (a distinct validator from frontmatter).
- carry-over fields with no DSL representation: `defaults.id_pattern`, `allowed_links`, `has_plugin`, `grammar_ref`.
- `template_ref` — optional/legacy; render (FR-001) still resolves when present.

Renderability and validatability SHALL be **derived from which parts are present**
(mirror the existing `is_renderable()` accessor), not from a declared kind.

The manifest fields `required_sections` and `variants` are **retired**: their
intent moves to `body_extraction` asserts (FR-033) and separate archetypes (ADR
0005 #1) respectively. A manifest that still declares either field SHALL be
**rejected** with an `ArchetypeLoadFailure` naming the archetype and the
deprecated field; the archetype does NOT load. There is **no tolerate/ignore path
and no dual-read** of the deprecated shape (no backward-compatibility layer).

`Registry` lookup by archetype name (FR-013) is unchanged.

## Acceptance

- **FR-031-AC-1**: A manifest archetype declaring `template_ref` + `frontmatter_schema_ref` + `body_extraction` compiles to one `CompiledArchetype` with `is_renderable() == true` and a resolvable body contract.
- **FR-031-AC-2**: A manifest archetype declaring `body_extraction` but no `template_ref` compiles successfully with `is_renderable() == false`, and is still validatable and extractable.
- **FR-031-AC-3**: `defaults.id_pattern`, `allowed_links`, `has_plugin`, and `grammar_ref` are retained on the compiled archetype and readable via accessors.
- **FR-031-AC-4**: `frontmatter_schema_ref` and `data_schema` are both retained as distinct compiled validators (frontmatter vs extracted record); neither is collapsed into the other.
- **FR-031-AC-5**: A manifest archetype that still declares `required_sections` (or `variants`) is **rejected** with an `ArchetypeLoadFailure` naming the archetype and the deprecated field; it does NOT load (no tolerate/ignore path).
- **FR-031-AC-6**: `Registry::archetype(name)` resolves a unified archetype identically to the pre-unification path (same name keying, same first-wins semantics).
