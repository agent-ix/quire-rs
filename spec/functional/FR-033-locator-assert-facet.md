---
id: FR-033
title: "Locator Assert Facet"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-011"
    type: "extends"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-032"
    type: "requires"
    cardinality: "1:1"
---

## Description

Per ADR 0004/0005, a `body_extraction` locator MAY carry an optional `assert:`
facet. The facet is **evaluated by `validate_document` ([FR-032](./FR-032-validate-document.md))** and **ignored
by extraction ([FR-011](./FR-011-body-extraction-dsl.md))** — one declaration, two postures.

The facet supports these optional keys:

- `level` — required heading level of the located section.
- `columns` — ordered list of exact table column headers the located table MUST have (case-sensitive, in order).
- `min_rows` — minimum number of data rows in the located table.
- `min_items` — minimum number of items in the located list.
- `id_column` — the table column whose cells are treated as ids.
- `id_pattern` — a regex each `id_column` cell (or located id) MUST match; supports `{field}` interpolation ([FR-034](./FR-034-assert-field-interpolation.md)).
- `matches` — a regex the located content MUST match; supports `{field}` interpolation ([FR-034](./FR-034-assert-field-interpolation.md)). Legal on every content locator EXCEPT `table_row` (tables use `columns`/`min_rows`/`id_pattern`). A locator that resolves to **no** values (missing/empty section) does NOT fire `matches` — absence is the required/content-status path's job ([FR-032](./FR-032-validate-document.md)), not this assert's.
- `choices` — a fixed set of allowed values the located **scalar** content MUST be one of (an enum constraint; case-sensitive, exact match after trim). Legal on the same scalar content locators as scalar `id_pattern` (`section_body`, `heading`, `list_item`, `frontmatter_field`); illegal on `table_row` (use `column_choices`) and `code_block`. Like `matches`, it does NOT fire when the locator resolves to no values.
- `column_choices` — a map of table column header → allowed value set; every data cell in that column MUST be one of the listed values. **`table_row` only.**
- `column_patterns` — a map of table column header → regex (with `{field}` interpolation, [FR-034](./FR-034-assert-field-interpolation.md)); every data cell in that column MUST match. **`table_row` only.**

The `assert` facet SHALL be **structurally validated at load time** alongside the
existing DSL validation (`validate_dsl`, [FR-011](./FR-011-body-extraction-dsl.md)): unknown keys, type errors, and
asserts that are nonsensical for the locator kind (e.g. `columns` on a
`section_body` locator) SHALL surface as load-time `ArchetypeLoadFailure`, not at
validate time.

> **CR-008 note:** The `matches` content-assert key (FR-033-AC-10) was added so a
> content locator can assert the *shape* of its located body against a regex —
> motivated by validating that a User Story's `## Story` section carries the
> `As a … / I want … / So that …` form. It reuses [FR-034](./FR-034-assert-field-interpolation.md) `{field}` interpolation
> and the shared `resolve_regex` (invalid-regex / unresolved-field handling). It is
> legal on every content locator except `table_row` (whose structure is asserted
> via `columns`/`min_rows`/`id_pattern`), and — like the other asserts — is
> evaluated by `validate_document` and ignored on the extract path (FR-033-AC-6).
> Crucially, `matches` does NOT fire when the locator resolves to no values: a
> missing/empty required section is reported by the required/content-status path
> ([FR-032](./FR-032-validate-document.md)), keeping the two concerns separate. See ADR 0004/0005.

> **CR-010 note:** The `choices`, `column_choices`, and `column_patterns` assert keys
> (FR-033-AC-11..13) were added to support **per-value enum/regex validation**, motivated by a
> `Review` findings table whose `Severity` column must be one of `low|medium|high`. The
> existing `matches` key validates a *scalar* content locator but is deliberately illegal on
> `table_row` (whose structure is asserted via `columns`/`min_rows`/`id_pattern`), so a table
> column's values could not be constrained at all. `choices` adds a first-class enum constraint
> for scalar locators (cleaner than a `^(a|b|c)$` regex), and `column_choices`/`column_patterns`
> extend per-value validation to individual table columns. All three are evaluated by
> `validate_document` and ignored on the extract path (FR-033-AC-6), reuse the shared
> `resolve_regex` (`column_patterns`) and `{field}` interpolation, and — like `matches` — do
> NOT fire when the located value is absent. See ADR 0004/0005.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-033-AC-1 | A `section_body` locator with `assert: {level: 2}` fails validation when the resolved heading is not level 2; passes when it is. | Test |
| FR-033-AC-2 | A `table_row` locator with `assert: {columns: [ID, Criteria, Verification]}` fails when the table's headers differ in text or order; passes on exact match. | Test |
| FR-033-AC-3 | `assert: {min_rows: 1}` fails on a header-only table; `assert: {min_items: 1}` fails on an empty list. | Test |
| FR-033-AC-4 | `assert: {id_column: ID, id_pattern: '^[A-Z]{2,4}-\d+-AC-\d+$'}` fails when any id cell does not match; passes when all match. | Test |
| FR-033-AC-5 | A load-time-invalid assert (unknown key, or `columns` on `section_body`) produces an `ArchetypeLoadFailure` naming the archetype and locator; the registry does not silently accept it. | Test |
| FR-033-AC-6 | Extraction of the same locator ignores the `assert` facet entirely (extracted value is identical with and without `assert`). | Test |
| FR-033-AC-7 | The assert-key × locator-kind legality matrix is enforced at load time. For each cell: `level` is legal on `section_body` and `heading`; `columns`, `min_rows`, `id_column`, `column_choices`, `column_patterns` are legal on `table_row`; `min_items` is legal on `list_item`; `id_pattern` is legal on `table_row`, `heading`, `section_body`, `list_item`, and `frontmatter_field` (scalar); `matches` is legal on every locator kind EXCEPT `table_row` (CR-008); `choices` is legal on `heading`, `section_body`, `list_item`, and `frontmatter_field` (scalar) and illegal on `table_row` and `code_block` (CR-010). Every illegal cell (e.g. `columns` on `section_body`, `min_items` on `table_row`, `level` on `table_row`, `matches` on `table_row`, `choices` on `table_row`, `column_choices` on `section_body`) produces an `ArchetypeLoadFailure` naming the archetype, locator, and offending key; every legal cell loads. A table-driven test exercises each cell. | Test |
| FR-033-AC-8 | `id_column` resolution precedence on a `table_row` locator is: explicit `assert.id_column` → the locator's `column` parameter → column index 0. A test with all three present resolves to `assert.id_column`; with `id_column` absent resolves to `column`; with both absent resolves to column 0. | Test |
| FR-033-AC-9 | `id_pattern` applies to non-table locators against the located scalar value: on a `heading` it matches the heading text, on a `section_body` the section's first line / id token, on a `list_item` each item, and on a `frontmatter_field` the scalar value. A mismatch fails with reason `assert`; a match passes. (`id_column` is meaningless and illegal on these kinds per FR-033-AC-7.) | Test |
| FR-033-AC-10 | A `section_body` locator with `assert: {matches: '<regex>'}` fails with reason `assert` (line-numbered at the located section) when the located content does NOT match the regex, and passes when it does — e.g. a `## Story` body carrying the `As a … / I want … / So that …` shape passes a `matches: '(?is)as an?\b.+i want\b.+so that\b'` assert, and a body lacking it fails. A locator that resolves to no values (missing/empty section) does NOT fire `matches`. `matches` is illegal on `table_row` at load time (per the FR-033-AC-7 legality matrix); the pattern supports `{field}` interpolation ([FR-034](./FR-034-assert-field-interpolation.md)). | Test |
| FR-033-AC-11 | A scalar locator (`heading`/`section_body`/`list_item`/`frontmatter_field`) with `assert: {choices: [low, medium, high]}` fails with reason `assert` when the located value is none of the listed values, and passes on an exact (trimmed) match. A locator that resolves to no values does NOT fire `choices`. `choices` is illegal on `table_row` and `code_block` at load time (CR-009). | Test |
| FR-033-AC-12 | A `table_row` locator with `assert: {column_choices: {Severity: [low, medium, high]}}` fails with reason `assert` when any data cell in the `Severity` column is not one of the listed values, and passes when every cell is. A column header named in `column_choices` that is absent from the table fails with an explicit "column not found" reason. `column_choices` is illegal on every non-`table_row` locator at load time. | Test |
| FR-033-AC-13 | A `table_row` locator with `assert: {column_patterns: {ID: '^FND-\d+$'}}` fails with reason `assert` when any data cell in the `ID` column does not match the regex, and passes when every cell matches; the regex supports `{field}` interpolation ([FR-034](./FR-034-assert-field-interpolation.md)). A named column absent from the table fails with an explicit "column not found" reason. `column_patterns` is illegal on every non-`table_row` locator at load time. | Test |

## Dependencies

- **Upstream**: [FR-011](./FR-011-body-extraction-dsl.md) (extends), [FR-032](./FR-032-validate-document.md) (requires)
- **Downstream**: [FR-034](./FR-034-assert-field-interpolation.md)
