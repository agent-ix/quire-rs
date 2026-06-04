# ADR 0005: Structural-expressability scope and deferred capabilities

**Status**: Proposed
**Date**: 2026-06-04
**Decision authority**: TBD

## Context

ADR 0004 makes `body_extraction` asserts the structural validator and direct
markdown the authoring path, replacing Jinja templates. A template (a generator)
can *describe* structures that a static validate/extract DSL cannot. We audited
every capability a template expresses that the new format cannot, and decided
each one explicitly. This ADR records those decisions together with the
**acknowledged limitation** and the **intended future improvement** for each, so
the scope boundary is a deliberate, documented line rather than an accident of
what the DSL happened to support.

The audit deliberately excludes the generation→validation *timing* shift (a
template guarantees structure at generation; validation checks it after) — that
is the premise of ADR 0004, not a per-feature loss.

## Decision

| # | Capability a template could express | Decision | Limitation acknowledged | Future intent |
|---|---|---|---|---|
| 1 | **Conditional / variant structure** — required shape depends on a frontmatter field (the manifest `variants[]` `selector`) | **Forbid.** Model subtypes as **separate archetypes**, one per shape. No `when:`/selector in `body_extraction`. | One archetype describes exactly one required shape; a "kind with subtypes" fragments into N archetypes. | Revisit only if a concrete need appears; if so, port the `selector` concept as a `when:` guard on conditional assert-blocks. Not built now (keeps each archetype simple and independently testable). |
| 2 | **Fixed section order** | **Drop enforcement** for now. Sections must be present and well-formed; sequence is free. | Two docs with sections in different order both validate; diffs/readability not enforced. (Never enforced today either — order was a template side effect; `required_sections` was unused.) | Add an `order:` assert (implicit from `match` key order, which is already an `IndexMap`, or an explicit list) and/or a canonicalizing formatter. |
| 3 | **Body↔frontmatter cross-references** — e.g. AC-id prefix equals the doc's own `id`; H1 = `[{id}] {title}` | **Add `{field}` interpolation** into asserts (e.g. `id_pattern: '^{id}-AC-\d+$'`). Adopted for traceability/completeness. | Interpolation resolves only from this document's frontmatter; not a general expression language. | Extend interpolation surface if more cross-field rules are needed. |
| 4 | **Deep nested topology** — required nesting ≥3 levels (section ▸ subsection ▸ table) | **2-level ceiling for now.** Plus **enforce per-level heading uniqueness** (no two headings share text at the same level) so flat name-addressing is sound. | Strict path-containment ("table only counts under X under Y") and required trees ≥3 deep are not expressible; addressing relies on names. | Add recursive `children` / `under_path` containment; when path-scoping lands, **relax uniqueness from per-level to per-parent (sibling)**. |
| 5 | **Default / fallback content** — supply text for a missing field | **Drop.** No generation step; a missing required section is an error, not a fill. | No archetype-supplied default body text. | None intended; the starter-boilerplate role moves to the per-archetype authoring **skeleton** (ADR 0004 / FR-029 recast). |
| 6 | **Computed / derived content** — auto-number CON/AC ids, render an array into a table, build the frontmatter block | **Drop auto-generation; validate correctness instead.** Contiguity + well-formedness asserts (and #3 interpolation for own-id prefixing) enforce what generation used to guarantee. | Authors write these literally; no engine-side generation. | None intended; covered by validation. |

Net-new format mechanisms introduced by these decisions: the **`assert:` facet**
(`level`, `columns`, `min_rows`, `min_items`, `id_column`, `id_pattern`),
**`{field}` interpolation** (#3), and the **per-level heading-uniqueness** rule
(#4). Everything else is forbid or drop.

## Consequences

- The expressability boundary is explicit and FR-backed: new FRs specify the
  `assert` facet, `{field}` interpolation, and heading uniqueness; the forbid/drop
  items are documented non-goals, not gaps to be "fixed" ad hoc in
  implementation.
- Deferred items (#1 `when:`, #2 `order:`, #4 recursion/path-scoping + per-parent
  uniqueness) each have a named future shape, so a later need has a designed
  landing spot rather than a redesign.
- `spec-artifacts-iso` archetypes must fit the 2-level ceiling and keep headings
  unique per level (lint-able); FR/NFR/StR/US/IT/TC/AC/CON already do.
- Any future proposal to add conditional structure (#1) or deep recursion (#4)
  amends this ADR.
