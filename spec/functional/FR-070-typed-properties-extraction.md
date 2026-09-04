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
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-011"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-025"
    type: "requires"
    cardinality: "1:1"
---
# FR-070: Typed Properties extraction to FieldDecl[]

## Description

When the engine extracts an object artifact whose module carries a
`semantic` block, the engine SHALL extract the `## Properties` section into a
`FieldDecl[]` that validates against the vendored semantic-core
`FieldDecl.json`, from either the typed table or the `sysml` fence, per the
`agent-ix/quoin` FR-071 mapping.

The engine SHALL leave the existing `properties` string yielded by
`section_body` unchanged beside the new array.

## Inputs

- The parsed document and its `## Properties` section.
- A `SemanticContext`: the module's `SemanticModule` record
  ([FR-069](./FR-069-semantic-module-contract-at-load.md)), the corpus-relative
  `path` of the document, and a `BundleIndex`
  `{ objects: [{ id, names[] }], enumerations: [{ id, names[] }], imports: { <package>: exports[] } }`.
  Every surface supplies the index explicitly: corpus extraction
  ([FR-025](./FR-025-spec-corpus-model.md)) builds it from the loaded corpus,
  where an object's `names` are its `id`, its `title`, and its frontmatter
  `name` when present, and `imports` come from the loaded modules' `exports`;
  the Filament API takes it as the optional `semanticBundle` input; a caller
  that has none passes an empty index, which is an explicit state, not a
  default.
- The golden fixtures vendored read-only from `agent-ix/quoin`
  `tests/fixtures/semantic-module/mapping/` and
  `tests/fixtures/semantic-module/corpus/config-service/` at revision
  `3e842ce`, plus a quire-rs-authored `config-version.bundle.json` index
  (`{ objects: [{ id: "FR-006", names: ["ConfigVersion"] }, { id: "FR-005", names: ["ConfigOverlay"] }] }`)
  under which the golden cases run.

## Outputs

- `fields[]`: `FieldDecl` entries in authored order, normalized (canonical
  key order, optional keys omitted).
- `fieldsForm`: `table` or `fence`.
- A row locus (1-based line of the table row or fence line) on every
  diagnostic.
- Diagnostics with a `semantic.*` or `agent-ix.semantic-core.*` code, a
  severity in `advisory`, `warning`, `error`, and a locus.
- The `fields` availability state ([FR-072](./FR-072-semantic-extraction-surface.md)):
  `available` with the array, or `unavailable` with a reason.

## Behavior

Form recognition:

- The engine SHALL treat a table whose header row is exactly
  `Field | Type | Multiplicity | Constraints` (cells trimmed, optional
  backticks stripped) as the typed table, a fenced block tagged `sysml`
  directly under the heading as the fence, and any other table or any bullet
  list as a legacy form (`free-column-table`, `bullet-list`).
- If the section holds a typed table and a fence, or a typed form followed by
  a legacy block, then the engine SHALL emit the error
  `semantic.properties-both-forms` at the second form's start line and set
  `fields` `unavailable` with reason `both-forms`.
- If the section holds only legacy forms, then the engine SHALL emit
  `semantic.legacy-properties-form` with the form name, the first block's
  line, and `migration: typed-table`, at severity `warning`, or `error` when
  the module sets `legacy_forms: error`; `fields` is `unavailable` with
  reason `legacy-form`.
- If the document has two `## Properties` headings, then the engine SHALL
  emit `semantic.duplicate-section` at the second heading and set `fields`
  `unavailable` with reason `duplicate-section`.
- A typed table with a header and no rows SHALL yield `fields` `available`
  and empty.
- If any row or fence line carries an `error`, then the engine SHALL set
  `fields` `unavailable` with reason `row-errors` listing the loci and SHALL
  NOT emit a partial array; a consumer never receives a type missing a field.

Type cells, resolved in this precedence:

1. A kernel scalar name (`UUID`, `Boolean`, `Integer`, `Decimal`, `String`,
   `Timestamp`, `Duration`, `Bytes`, `JsonObject`) maps to that target;
   `Decimal(p,s)` adds `decimal: { precision, scale }`; bare `Decimal` is the
   error `agent-ix.semantic-core.MISSING_DECIMAL_POLICY`; `<Name> [<unit>]`
   adds `unit` and is `agent-ix.semantic-core.UNIT_NOT_ALLOWED` when the
   target is not `Integer`, `Decimal`, or `Duration`.
2. A bundle object whose `id` equals the token.
3. A bundle object one of whose `names` equals the token; two objects
   matching is the error `semantic.ambiguous-type` naming both.
4. A bundle enumeration whose `id` or `names` match the token.
5. A name exported by a package in `imports` that a loaded module provides.

Matches at steps 2..4 map to `ix://<org>/<repo>/type/<Name>` where
`<org>/<repo>` is the bundle's package (the `SemanticContext` `package`
segments); step 5 maps to `ix://<package>/type/<Name>`. The engine
maps any other token to the placeholder `ix://<org>/<repo>/unresolved/<Token>`
with the advisory `semantic.unresolved-type` at the row carrying `reason`
`unknown-token`, `no-bundle-index` (empty index), or `import-unresolved`;
a token that is not a semantic-core `Identifier` is instead the error
`semantic.invalid-type-token`. The engine SHALL NOT emit a bare string for an
unresolved token.

Multiplicity cells: `n` and empty map to `{ lower: n, upper: n }` (empty is
`1`); `n..m` to `{ n, m }`; `n..*` to `{ lower: n }`; `*` to `{ lower: 0 }`;
the flags `ordered` and `unique` are admitted only when `upper` is absent or
greater than 1; `lower` greater than `upper`, a non-integer bound, or a flag
on a single value is the error `semantic.invalid-multiplicity` at the row.

Constraints cells: entries separated by commas outside a `/…/` pattern;
`keyword: value` pairs and bare flags from the closed set `min`, `max`,
`exclusiveMin`, `exclusiveMax`, `minLength`, `maxLength`, `pattern /…/`
(the value runs from the first `/` to the last `/` of the entry, carried
opaque with `dialect: ecma-262`, never compiled), `enumValues a|b`
(`|`-separated, no spaces), `nonEmpty`, `unique`, `format ns:name`, and the
field flags `identity` and `nullable`; an unknown keyword is the error
`semantic.unknown-constraint-keyword` at the row.

Reader rules carried from semantic-core: a duplicate field name is
`agent-ix.semantic-core.DUPLICATE_NAME` at the second row; `identity` on a
field whose multiplicity is not `1..1` is `IDENTITY_NOT_SINGLE`; `identity`
on `JsonObject` is `IDENTITY_ON_JSON_OBJECT`; `decimal` on a non-`Decimal`
target is `DECIMAL_ON_NON_DECIMAL`; flags on a non-collection are
`FLAGS_ON_NON_COLLECTION`.

Fence form: the engine SHALL read the fence line by line;
`attribute <name> : <Type>[<mult>]` and `ref item <name> : <Type>[<mult>]`,
each with an optional trailing `{ … }` whose content is taken as the
`Constraints` cell text and is otherwise opaque; any other non-blank line
(`item`, `part def`, `:>`, …) is the error `semantic.sysml-outside-subset` at
that line.

Normalization: the table and fence forms of the same content SHALL produce
identical normalized `FieldDecl[]`. Every produced entry SHALL validate
against the vendored `FieldDecl.json` at the module's `semantic_core`
version; an entry that does not is an engine defect and SHALL fail extraction
with `semantic.internal-invalid-decl`, never be dropped.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-070-CON-1 | The engine SHALL recognize fences line by line and carry brace content and pattern text as opaque strings, never parsed or compiled. | Architecture | Test |
| FR-070-CON-2 | The engine SHALL resolve types from the `BundleIndex` and the loaded modules only, with no filesystem walk and no network read on this path. | Architecture | Test |
| FR-070-CON-3 | For an artifact of a module without a `semantic` block, the extracted record SHALL equal the checked-in pre-change baseline byte-for-byte and carry no `semantic` key. | Compatibility | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-070-AC-1 | Under `config-version.bundle.json`, the vendored `config-version.table.md` extracts to the normalized `fields` of `config-version.expected.json`, every entry validating against `FieldDecl.json`. | Test |
| FR-070-AC-2 | The vendored `config-version.fence.md` extracts to the identical normalized array with `fieldsForm: fence`. | Test |
| FR-070-AC-3 | `both-forms.md` yields `fields` `unavailable` (`both-forms`) and one `semantic.properties-both-forms` error at the fence start line recorded in `both-forms.expected.json`; a typed table followed by a bullet list yields the same code at the list's line; two `## Properties` headings yield `semantic.duplicate-section`. | Test |
| FR-070-AC-4 | Every `Type` case in `cell-cases.json` yields the expected target, `decimal`, `unit`, placeholder, or diagnostic at the row; a token matching two bundle names yields `semantic.ambiguous-type` naming both; an empty index yields `unresolved` with reason `no-bundle-index`; a non-`Identifier` token yields `semantic.invalid-type-token`. | Test |
| FR-070-AC-5 | Every `Multiplicity` case in `cell-cases.json`, plus `*`, `0..*`, `0..0`, `2..2 unique`, and `a..b`, yields the expected `Multiplicity` or `semantic.invalid-multiplicity` at the row. | Test |
| FR-070-AC-6 | Every `Constraints` case in `cell-cases.json`, plus `nullable`, `pattern: /a,b|c/`, and a duplicated field name, yields the expected constraints, flags, or diagnostic at the row, with the pattern text carried verbatim. | Test |
| FR-070-AC-7 | Every fence-line case in `cell-cases.json` yields `semantic.sysml-outside-subset` at the fence line, and a fence line with brace content of arbitrary UTF-8 text maps that text as the `Constraints` cell. | Test |
| FR-070-AC-8 | The pinned `FR-006-config-version-entity.md` copy, `legacy-bullets.md`, and `legacy-mixed.md` yield the form, line, and warning of `legacy.expected.json`, with `properties` unchanged and `fields` `unavailable` (`legacy-form`); under `legacy_forms: error` the same finding is an error. | Test |
| FR-070-AC-9 | An artifact of a module without a `semantic` block yields a record equal to the checked-in baseline byte-for-byte, with no `semantic` key and no semantic diagnostic. | Test |
| FR-070-AC-10 | A row error in any one row yields `fields` `unavailable` with reason `row-errors` naming every erroring locus and no partial array; a typed header with zero rows yields `available` and empty. | Test |

## Dependencies

- **Upstream**: [FR-069](./FR-069-semantic-module-contract-at-load.md), [FR-025](./FR-025-spec-corpus-model.md), [FR-011](./FR-011-body-extraction-dsl.md); `agent-ix/quoin` FR-071/FR-074
- **Downstream**: [FR-071](./FR-071-clause-and-operation-extraction.md), [FR-072](./FR-072-semantic-extraction-surface.md); `agent-ix/filament-core-data#36`
