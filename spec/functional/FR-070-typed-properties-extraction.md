---
id: FR-070
title: "Typed Properties extraction to FieldDecl[]"
type: FR
verification_method: test
evidence:
  - kind: test_case
    ref: tests/semantic_properties.rs
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-019"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-069"
    type: "requires"
    cardinality: "1:1"
---
# FR-070: Typed Properties extraction to FieldDecl[]

## Description

For an object artifact whose module carries a `semantic` block, the engine
SHALL extract the `## Properties` section into a `FieldDecl[]` that validates
against the vendored semantic-core `FieldDecl.json`, from either the typed
table or the `sysml` fence, per the `agent-ix/quoin` FR-071 mapping.

The existing `properties` string yielded by `section_body` is preserved
unchanged beside the new array.

## Inputs

- The parsed document and its `## Properties` section.
- The module's `SemanticModule` record ([FR-069](./FR-069-semantic-module-contract-at-load.md)):
  `package`, `imports`, and the installed modules' `exports`.
- A bundle index when extraction runs over a corpus ([FR-025](./FR-025-spec-corpus-model.md)):
  object artifacts by `id` and `title`, enumerations by `id`.
- The golden fixtures vendored read-only from `agent-ix/quoin`
  `tests/fixtures/semantic-module/mapping/` at revision `3e842ce`.

## Outputs

- `fields[]`: `FieldDecl` entries in authored order, normalized (canonical
  key order, optional keys omitted).
- `fieldsForm`: `table` or `fence`.
- Row loci: 1-based line of each row or fence line, for every diagnostic.
- Diagnostics with a `semantic.*` code, a severity in `advisory`, `warning`,
  `error`, and a locus.

## Behavior

- The typed table SHALL be recognized by the exact header
  `Field | Type | Multiplicity | Constraints`; the fence SHALL be recognized by
  a fenced block tagged `sysml` directly under the heading. A section holding
  both SHALL yield no `fields` and one error `semantic.properties-both-forms`
  at the second form's start line.
- A free-column table or a bullet list under `## Properties` SHALL yield the
  warning `semantic.legacy-properties-form` with the form name, the block's
  first line, and `migration: typed-table`; a mixed section names the first
  block's form. `fields` is then `unavailable` and `properties` is unchanged.
  When the module sets `legacy_forms: error`, the severity is `error`.
- `Type` cells SHALL map: a kernel scalar name (`UUID`, `Boolean`, `Integer`,
  `Decimal`, `String`, `Timestamp`, `Duration`, `Bytes`, `JsonObject`) to that
  target; `Decimal(p,s)` to `Decimal` with `decimal: { precision, scale }`;
  bare `Decimal` to the semantic-core reader error
  `agent-ix.semantic-core.MISSING_DECIMAL_POLICY`; `<Name> [<unit>]` to the
  target with `unit`; a bundle object by `title` or `id` and an enumeration
  by `id` to `ix://<org>/<repo>/type/<Name>`; a name exported by an imported
  package to `ix://<package>/type/<Name>`; anything else to the placeholder
  `ix://<org>/<repo>/unresolved/<Token>` with the advisory
  `semantic.unresolved-type` at the row. The engine SHALL NOT emit a bare
  string for an unresolved token.
- `Multiplicity` cells SHALL map `1` and empty to `{lower: 1, upper: 1}`,
  `0..1` to `{0, 1}`, `a..b` to `{a, b}`, `1..*` to `{lower: 1}`, with the
  flags `ordered` and `unique` admitted only when `upper` is absent or greater
  than 1; `5..2` and `1 ordered` SHALL fail with `semantic.invalid-multiplicity`
  at the row.
- `Constraints` cells SHALL map a comma-separated list of `keyword: value`
  pairs and bare flags onto the closed keyword set (`min`, `max`,
  `exclusiveMin`, `exclusiveMax`, `minLength`, `maxLength`, `pattern /…/`
  with `dialect: ecma-262`, `enumValues a|b`, `nonEmpty`, `unique`,
  `format ns:name`) plus the field flag `identity`; an unknown keyword SHALL
  fail with `semantic.unknown-constraint-keyword` at the row.
- The `sysml` fence SHALL be read line by line: `attribute <name> : <Type>[<mult>]`
  and `ref item <name> : <Type>[<mult>]`, each with an optional `{ … }` whose
  content is the `Constraints` cell text and is otherwise opaque. Any other
  line (`item`, `part def`, `:>`, …) SHALL fail with
  `semantic.sysml-outside-subset` at that line.
- The table and fence forms of the same content SHALL produce identical
  normalized `FieldDecl[]`.
- Every produced entry SHALL validate against the vendored `FieldDecl.json`
  at the module's `semantic_core` version; a produced entry that does not is
  an engine defect and SHALL fail extraction, never be dropped.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-070-CON-1 | Fence recognition is line-level; brace content and any clause text are carried as opaque strings with spans, never parsed or evaluated. | Architecture | Inspection |
| FR-070-CON-2 | Resolution reads only the bundle index and the loaded modules; it performs no filesystem walk outside the corpus and no network read. | Architecture | Test |
| FR-070-CON-3 | The `properties` string and every pre-existing record key remain byte-identical for artifacts of modules with and without a `semantic` block. | Compatibility | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-070-AC-1 | The vendored `config-version.table.md` extracts to the `fields` array of `config-version.expected.json` after normalization, and every entry validates against `FieldDecl.json`. | Test |
| FR-070-AC-2 | The vendored `config-version.fence.md` extracts to the identical normalized array with `fieldsForm: fence`. | Test |
| FR-070-AC-3 | `both-forms.md` yields no `fields` and one `semantic.properties-both-forms` error whose line equals the fence's start line recorded in `both-forms.expected.json`. | Test |
| FR-070-AC-4 | Every `Type` case in `cell-cases.json` (kernel, `Decimal(10,2)`, `Duration [ms]`, object by title, enumeration by id, import, unresolved, bare `Decimal`) yields the expected target, `decimal`, `unit`, or diagnostic at the row locus. | Test |
| FR-070-AC-5 | Every `Multiplicity` case in `cell-cases.json` yields the expected `Multiplicity` or `semantic.invalid-multiplicity` at the row. | Test |
| FR-070-AC-6 | Every `Constraints` case in `cell-cases.json` yields the expected constraint list, `identity` flag, or `semantic.unknown-constraint-keyword` at the row. | Test |
| FR-070-AC-7 | Every fence-line case in `cell-cases.json` yields `semantic.sysml-outside-subset` at the fence line. | Test |
| FR-070-AC-8 | The pinned `FR-006-config-version-entity.md` copy, `legacy-bullets.md`, and `legacy-mixed.md` yield the form, line, and warning recorded in `legacy.expected.json`, with `properties` unchanged and `fields` `unavailable`; under `legacy_forms: error` the same finding is an error. | Test |
| FR-070-AC-9 | An artifact of a module without a `semantic` block yields no `fields` key, no semantic diagnostic, and a record byte-identical to the pre-change extraction. | Test |

## Dependencies

- **Upstream**: [FR-069](./FR-069-semantic-module-contract-at-load.md), [FR-025](./FR-025-spec-corpus-model.md), [FR-011](./FR-011-body-extraction-dsl.md); `agent-ix/quoin` FR-071/FR-074
- **Downstream**: [FR-072](./FR-072-semantic-extraction-surface.md); `agent-ix/filament-core-data#36`
