---
id: FR-033
title: "Locator Assert Facet"
artifact_type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-011"
    type: "extends"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-032"
    type: "requires"
    cardinality: "1:1"
---

## Behavior

Per ADR 0004/0005, a `body_extraction` locator MAY carry an optional `assert:`
facet. The facet is **evaluated by `validate_document` (FR-032)** and **ignored
by extraction (FR-011)** — one declaration, two postures.

The facet supports these optional keys:

- `level` — required heading level of the located section.
- `columns` — ordered list of exact table column headers the located table MUST have (case-sensitive, in order).
- `min_rows` — minimum number of data rows in the located table.
- `min_items` — minimum number of items in the located list.
- `id_column` — the table column whose cells are treated as ids.
- `id_pattern` — a regex each `id_column` cell (or located id) MUST match; supports `{field}` interpolation (FR-034).

The `assert` facet SHALL be **structurally validated at load time** alongside the
existing DSL validation (`validate_dsl`, FR-011): unknown keys, type errors, and
asserts that are nonsensical for the locator kind (e.g. `columns` on a
`section_body` locator) SHALL surface as load-time `ArchetypeLoadFailure`, not at
validate time.

## Acceptance

- **FR-033-AC-1**: A `section_body` locator with `assert: {level: 2}` fails validation when the resolved heading is not level 2; passes when it is.
- **FR-033-AC-2**: A `table_row` locator with `assert: {columns: [ID, Criteria, Verification]}` fails when the table's headers differ in text or order; passes on exact match.
- **FR-033-AC-3**: `assert: {min_rows: 1}` fails on a header-only table; `assert: {min_items: 1}` fails on an empty list.
- **FR-033-AC-4**: `assert: {id_column: ID, id_pattern: '^[A-Z]{2,4}-\\d+-AC-\\d+$'}` fails when any id cell does not match; passes when all match.
- **FR-033-AC-5**: A load-time-invalid assert (unknown key, or `columns` on `section_body`) produces an `ArchetypeLoadFailure` naming the archetype and locator; the registry does not silently accept it.
- **FR-033-AC-6**: Extraction of the same locator ignores the `assert` facet entirely (extracted value is identical with and without `assert`).
