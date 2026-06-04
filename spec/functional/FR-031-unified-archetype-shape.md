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

The manifest field `required_sections` is **retired**: it SHALL NOT be read as a
validation contract; its intent moves to `body_extraction` asserts (FR-033). A
manifest that still declares `required_sections` SHALL load with a non-fatal
`Diagnostic` noting the field is ignored and pointing to `body_extraction`. No
dual-read fallback is provided.

`Registry` lookup by archetype name (FR-013) is unchanged.

## Acceptance

- **FR-031-AC-1**: A manifest archetype declaring `template_ref` + `frontmatter_schema_ref` + `body_extraction` compiles to one `CompiledArchetype` with `is_renderable() == true` and a resolvable body contract.
- **FR-031-AC-2**: A manifest archetype declaring `body_extraction` but no `template_ref` compiles successfully with `is_renderable() == false`, and is still validatable and extractable.
- **FR-031-AC-3**: `defaults.id_pattern`, `allowed_links`, `has_plugin`, and `grammar_ref` are retained on the compiled archetype and readable via accessors.
- **FR-031-AC-4**: `frontmatter_schema_ref` and `data_schema` are both retained as distinct compiled validators (frontmatter vs extracted record); neither is collapsed into the other.
- **FR-031-AC-5**: A manifest archetype that still declares `required_sections` loads, the field is ignored for validation, and exactly one non-fatal diagnostic names the archetype and points to `body_extraction`.
- **FR-031-AC-6**: `Registry::archetype(name)` resolves a unified archetype identically to the pre-unification path (same name keying, same first-wins semantics).
