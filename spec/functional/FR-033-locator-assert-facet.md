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
- `optional_columns` — a subset of `columns` the document MAY omit ([CR-023](#cr-023-note)). The declared order still holds: the headers MUST be an ordered subsequence of `columns` containing every non-optional column. **`table_row` only.**
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
| FR-033-AC-11 | A scalar locator (`heading`/`section_body`/`list_item`/`frontmatter_field`) with `assert: {choices: [low, medium, high]}` fails with reason `assert` when the located value is none of the listed values, and passes on an exact (trimmed) match. A locator that resolves to no values does NOT fire `choices`. `choices` is illegal on `table_row` and `code_block` at load time (CR-010). | Test |
| FR-033-AC-12 | A `table_row` locator with `assert: {column_choices: {Severity: [low, medium, high]}}` fails with reason `assert` when any data cell in the `Severity` column is not one of the listed values, and passes when every cell is. A column header named in `column_choices` that is absent from the table fails with an explicit "column not found" reason. `column_choices` is illegal on every non-`table_row` locator at load time. | Test |
| FR-033-AC-13 | A `table_row` locator with `assert: {column_patterns: {ID: '^FND-\d+$'}}` fails with reason `assert` when any data cell in the `ID` column does not match the regex, and passes when every cell matches; the regex supports `{field}` interpolation ([FR-034](./FR-034-assert-field-interpolation.md)). A named column absent from the table fails with an explicit "column not found" reason. `column_patterns` is illegal on every non-`table_row` locator at load time. | Test |
| FR-033-AC-14 | A `table_row` locator with `assert: {columns: [ID, Criteria, Priority, Verification], optional_columns: [Priority]}` passes whether or not the table carries the `Priority` column, and still fails when a *non*-optional column is absent or the header order differs. `optional_columns` is illegal on every non-`table_row` locator, and illegal when it names a column absent from `columns`, at load time. | Test |
| FR-033-AC-15 | A `column_choices` or `column_patterns` entry naming a column declared in `optional_columns` does NOT fire when that column is absent from the table (no "column not found" failure); when the column IS present it is checked exactly as any other column, so a disallowed value still fails with reason `assert`. | Test |
| FR-033-AC-16 | A row-scoped assert failure — `column_choices`, `column_patterns`, `id_pattern` — reports the offending **row's own** document line rather than the table's start line, and carries the row's `id_column` cell as `row_id` when the assert declares one; the rendered message leads with that id. A table-scoped failure (`columns`, `min_rows`, table-not-found, column-not-found) keeps the located section's line and carries no `row_id`. An assert declaring no `id_column`, or a table lacking that header, yields a row line and no id — never a guessed one. | Test (TC-991) |

<a id="cr-023-note"></a>

> **CR-097 note (2026-08-22):** AC-16 is new — a row-scoped failure says which
> row. `agent-ix/quire-cli#58`, epic `agent-ix/quoin#197`.
>
> **What the number counts:** `[assert]` findings from
> `quire validate --scope . "spec/**/*.md" "plan/**/*.md" "reviews/**/*.md"`
> over `agent-ix/filament-ide-rs` @ `fc5d644`, under `quire 0.29.0` / engine
> `v0.42.0` / `spec-artifacts-process v0.23.0`: **496 findings, 15 of which
> carried a row id anywhere, and exactly ONE distinct line per document.** Every
> assert against `spec/service/matrix/tests.md` reported `line 25`. So 481
> findings read like this, twice in a row, byte-identical, naming neither of the
> two rows they came from:
>
> ```
> spec/service/matrix/tests.md: line 25: [TestMatrix] 'test_cases': column 'Type' cell 'Inspection' is not one of [...]
> spec/service/matrix/tests.md: line 25: [TestMatrix] 'test_cases': column 'Type' cell 'Inspection' is not one of [...]
> ```
>
> `Inspection` appears 112 times across that corpus. There was no way to act on
> an individual one without hand-diffing the table against the vocabulary — and
> not even the machine surface disambiguated them, since the two records were
> equal field for field.
>
> **The engine already had both facts.** It walks `table.rows` to evaluate a
> column assert, so the row index was in hand; `parse_table_with_lines` (CR-089,
> which fixed the same defect for coverage records) already returns per-row
> lines and the assert evaluator was simply calling `parse_table` instead. And
> `LocatorAssert::id_column` — *"table column whose cells are treated as ids"* —
> has been declared by every matrix contract in the ecosystem since FR-033
> shipped.
>
> **No new declaration, and no guessing.** The row is named by the column the
> contract already said identifies it. An assert declaring no `id_column`, or a
> table lacking that header, gets the row's line and no id rather than a
> first-cell heuristic — the engine knows nothing of "TC", and inferring an id
> column would put it back in the business of knowing.
>
> **Measured after, on the same corpus** — population: `[assert]` findings over
> `filament-ide-rs/spec/**` whose document type the process module resolves,
> which is a **narrower set than the 496** above (that run also walked `plan/`
> and `reviews/`), so these are not two ends of one delta:
>
> | | |
> |---|---|
> | assert findings | **144** |
> | carrying a row id | **144** |
> | documents with more than one distinct assert line | **11 of 11** |
>
> Before, that last row was 0 of 11.
>
> **`id_pattern`'s failure deliberately carries no `row_id`.** The id cell *is*
> the offending value and already appears in the message; naming it twice would
> read as two facts. That failure gains the row line, which is what it was
> missing.

> **CR-023 note:** `optional_columns` (FR-033-AC-14) was added because `columns`
> is an exact match, which leaves a contract no way to ask for a column an
> existing corpus never authored. The concrete case is the TestMatrix `Priority`
> column (agent-ix/spec-artifacts-process#14): 49 of 169 ecosystem matrices carry
> real test-case rows with no priority anywhere in the document. The only ways
> out were to fail all 49 forever, or to write an invented priority into each —
> fabricating planning data to satisfy a checker, which is the failure mode this
> whole programme exists to catch. Making the column declarable-optional fixes
> the contract instead of the corpus. Order is still enforced and only the
> *declared* column is forgiving, so this does not weaken `columns` into "any
> subset will do".
>
> FR-033-AC-15 completes the key: the motivating contract declares `Priority`
> optional **and** constrains its values with `column_choices: [P0..P4]`. Left
> alone, the per-value check would report "column not found" (FR-033-AC-12) on
> precisely the documents `optional_columns` exists to admit, so a per-value
> constraint on a declared-optional column is skipped while that column is
> absent and enforced normally once it is authored.

## Dependencies

- **Upstream**: [FR-011](./FR-011-body-extraction-dsl.md) (extends), [FR-032](./FR-032-validate-document.md) (requires)
- **Downstream**: [FR-034](./FR-034-assert-field-interpolation.md)