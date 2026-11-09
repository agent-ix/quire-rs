---
id: FR-081
title: "Record-sections extraction to a module's own record keys"
type: FR
verification_method: test
evidence:
  - kind: test_case
    ref: tests/it/semantic_record_sections.rs
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-019"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-072"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-075"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-076"
    type: "requires"
    cardinality: "1:1"
---
# FR-081: Record-sections extraction to a module's own record keys

## Description

When the engine extracts an object artifact whose module carries a
`semantic` block that names the mapping token `record-sections`, the engine
SHALL read each named section of the artifact that names a record key of the
object type's `data_schema` (a typed table, a bullet list, or one inline
value) into the output key `record`, with a source span per key under
`recordSources` and a per-key state under `availability.recordKeys`.

The grammar follows `agent-ix/quoin` FR-121 (record-key sections), FR-122
(architecture record keys) and FR-123 (enumeration `Values` and model-table
sections). The golden fixtures are owned by this repository's `quire-fixtures`
crate (`fixtures/record-sections`); quoin holds no copy (FR-121-CON-2,
FR-122-CON-2, FR-123-CON-1). This requirement extends
[FR-076](./FR-076-relationships-extraction.md) the way FR-076 extends
[FR-072](./FR-072-semantic-extraction-surface.md): it adds output keys beside
the declaration arrays and does not change them.

## Inputs

- The document and the `SemanticContext` of
  [FR-070](./FR-070-typed-properties-extraction.md).
- The object type's resolved `data_schema` and the schema documents its
  `$ref`s name (`SemanticContext::with_record_schema`). Each property's JSON
  type, `$ref` and `anyOf` branches direct the reading; a `$ref` is followed
  through the supplied documents and the embedded semantic-core bundle. A
  `$ref` to neither makes its property unreadable (the section is prose).
- The context's relation vocabulary, which names the typed relation keys
  FR-076 derives, and its `table_row` locators, which name the model-table
  keys FR-075 fills.
- The artifact's own frontmatter `id`, `object` and `## Operations` names,
  and the bundle index and package, as FR-076 resolves a `Target`.

## Outputs

- `record`: an object of the extracted keys, each a JSON value its property
  accepts. Present exactly when at least one key is extracted.
- `recordSources`: one `{ key, sourceSpan }` per extracted key, in document
  order, `sourceSpan` a `SourceLocus` from the heading line to the last
  non-blank line of the section. The record is sealed, so no span rides in it.
- `availability.recordKeys`: per key, `{ state, reason?, lossy }`. Present
  when the artifact names the token and a section names, or fails to name, a
  key.
- Diagnostics in the FR-075 shape, with the codes of FR-076.

`declaration_record` carries the `record` keys beside the declaration arrays,
so the record a module's data schema validates is the one a document produced.

## Behavior

Token and key resolution:

- The engine SHALL read record-key sections only when `semantic.mappings`
  names `record-sections` and a `data_schema` is supplied.
- If `semantic.mappings` does not name `record-sections`, or no `data_schema`
  is supplied, then the engine SHALL treat every section as prose to this
  mapping, with no diagnostic and no `availability.recordKeys`.
- The engine SHALL match a `##` heading to a property of the record by
  lowercasing both and removing spaces, hyphens and underscores, so
  `## Retry Policy` names `retryPolicy` and `## Associated Types` names
  `associated_types`.
- If two properties normalize equal and a heading names them, then the engine
  SHALL emit `semantic.feature-not-extractable` at the heading with `reason`
  `key-ambiguous` and read neither.
- The engine SHALL read a heading only as a key no other mapping owns. The
  owned keys are `fields`, `clauses`, `operations`, `relations`,
  `featureOrder` and the FR-075 systems members; each typed relation key
  FR-076 derives; each key a declared FR-075 table locator fills (`values`,
  `states`, `transitions`, `steps`, `members`, `vocabulary`), whatever its
  heading normalizes to; and, under `agent-ix/spec-objects-business`, the keys
  quoin FR-123 lists as having no authored source: `owner` of a
  `nested_entity`, `ubiquitous_language` of a `domain`, and `members` of a
  `domain` or `population`. The object type is the one the surface names
  (`SemanticContext::with_object_type`, the Python `objectType`) or the
  relation vocabulary's.
- The engine SHALL treat a heading that names an owned key, or no key, as
  prose.
- If a heading names a property whose schema shape no mapping can read (not a
  scalar, an object, an array of scalars or of objects, nor `associated_types`
  as a `TypeRef[]`, or a `$ref` that no supplied document resolves), then the
  engine SHALL emit `semantic.feature-not-extractable` at the heading with
  `reason` `schema-unreadable` and report the key `unavailable` with that
  reason. A `$ref` to a fragment of the same document is followed.
- If the artifact holds a second section naming the same key, then the engine
  SHALL emit `semantic.duplicate-section` at the second heading with `reason`
  `second-section`.
- The engine SHALL NOT read a record key from frontmatter, and SHALL omit a
  key whose section is absent, reporting it `not_applicable`.

Section shape by key type:

- An object-valued key is one table with one data row; an array of objects is
  one table, one element per data row; a scalar is a body of exactly one
  inline-code value; an array of scalars is one bullet list with one value per
  item, bare or in backticks.
- If a section's first block is not of its key's form, or it holds a further
  list, fence, table or diagram, then the engine SHALL emit
  `semantic.feature-not-extractable` at that block's first line with `reason`
  the key name.
- If an object-valued key's table holds a second data row, then the engine
  SHALL emit `semantic.invalid-model-cell` at it with `reason` `second-row`;
  with no data row, at the header with `reason` `no-row`. An array-valued key
  with a header and no row yields `[]` and is `available`.
- If a section holds no block (and for a scalar key no text), then the engine
  SHALL emit the warning `semantic.record-key-no-block` at the heading with
  `reason` `no-block` and report the key `not_applicable`.

Columns and cells:

- The engine SHALL match each header cell to one property of the key's object
  schema under the normalization above. A header cell matching none is
  `semantic.invalid-model-cell` with `reason` `column-unknown`; two matching
  one property, `column-duplicate`, both at the header line.
- The engine SHALL strip surrounding backticks and type a cell by its
  property: `integer` and `number` by their literal (else `reason` `cell-type`),
  `boolean` by `true` or `false`, a string, an enumeration or an `anyOf` of
  those as the authored text, an `anyOf` of an integer and a string as the
  integer for a decimal literal and else the string. It SHALL NOT check an
  enumeration or `format`; record validation does.
- The engine SHALL extract `unknown`, `not_assessed` and `not_applicable` as
  that exact string, and SHALL omit the field of an empty cell with no default.
- A non-empty cell whose property is an object or an array of objects is
  `semantic.invalid-model-cell` with `reason` `nested-object-cell` at its row.
- A `SemanticId` property takes its value from the cell by the FR-076 `Target`
  rules, with the same `target-not-id` refusal, the `semantic.unresolved-target`
  advisory under no bundle index, and no `allowed_links` check (no verb names
  one). With no bundle package the key is `unavailable` with reason
  `no-bundle-package` and one advisory `semantic.unresolved-target` carrying
  that reason.
- Cell checks run left to right in row order; the first failing check of a
  cell is its only diagnostic.

Availability:

- If any header, row, block or section of a key carries an error, then the
  engine SHALL report the key `unavailable` with reason
  `entry-errors: lines <lines>` as FR-076 formats it, with no value and no
  `recordSources` entry. Otherwise the key is `available` and not lossy.

Keys with their own form (module `agent-ix/spec-objects-architecture`, quoin
FR-122), when the record schema declares the key:

- `provider` and `exposes` follow the scalar form above, one `SemanticId`.
  `delivery`, `versioning`, `registration` and `stability` follow the
  one-row-table form.
- `routes` is a table `Method | Path | Operation`. If an `Operation` cell names
  no operation of the same artifact's `## Operations`, then the engine SHALL
  emit `semantic.invalid-model-cell` at the row with `reason`
  `route-operation-unknown`.
- `associated_types` is a table `Type | Multiplicity`, one `TypeRef` per row by
  the FR-070 `Type` and `Multiplicity` cell grammars. The engine SHALL NOT take
  `associated_types` from an `interface` `## Contract` fence.
- `records` and `endianness` are lowered from the YAML fence under `## Layout`:
  top-level `endianness`; each `record_types[]` entry to a `RecordLayout`
  `{ name, tag, size, fields }` and each `fields[]` entry to a `LayoutField`
  `{ name, offset, size, type }`. A `tag` is the authored text, so `0x21` stays
  `"0x21"`. A string `LayoutField.size` is kept as the identifier when exactly
  one record of the fence has a field of that name; if none, the engine SHALL
  emit `semantic.invalid-model-cell` at the fence line with `reason`
  `size-unknown-field`, and if two or more, `size-ambiguous`. Other `## Layout`
  keys (`format`, `magic`) stay out of the record. A `## Records` or
  `## Endianness` section is prose.
- `thresholds` is a table `Scope | Metric | Limit | Window | Status |
  Retry After | Behavior`; `Status`, `Retry After` and `Behavior` build the
  row's `on_exceeded` object by the schema's types, an empty `Retry After` or
  `Behavior` omitting that field. If a `Status` cell is empty, then the engine
  SHALL emit `semantic.invalid-model-cell` at the row with `reason`
  `status-required`.

Model-table keys (quoin FR-123):

- Where the record schema declares the key and a declared FR-075 table locator
  fills it, the engine SHALL copy the FR-075 `model` entries of `values`,
  `states`, `transitions`, `steps`, `members` and `vocabulary` to the record
  key of the same name, each element without its `sourceSpan`, in row order. The key is `available`
  (`[]` for a header-only table), `unavailable` with the model's reason when
  the model is, and `not_applicable` when the table is absent. Its
  `recordSources` span is the key's whole section, from its heading to its
  last non-blank line, as for any other key; `recordSources` stays in document
  order. A model entry whose key
  the schema does not declare stays in `model` and adds no record key.
- The engine SHALL resolve a `Member` cell of an `aggregate_root` to a
  `SemanticId` by the FR-075 declaration references, admitting only a
  declaration whose object type carries a role of the construct
  `references.members`, and SHALL copy `members` as `RelationDecl[]`. The
  `verb` is the one composite verb of the registry that the object type's
  `allowed_links` admit for the target's object type or role, and `category`
  and `composite` come from that verb's registry entry. An unresolved
  reference is `semantic.unknown-reference`, `semantic.reference-kind-mismatch`
  or `semantic.ambiguous-reference`; if no composite verb or more than one
  admits the target, then the engine SHALL emit `semantic.invalid-model-cell`
  at the row with `reason` `member-verb`. With no bundle package, no bundle
  index or no relation vocabulary the key is `unavailable` with that reason.
- If an `## Relationships` row of an object type that declares the `members`
  locator carries a composite verb, then the engine SHALL emit
  `semantic.duplicate-model-entry` at the row with `reason`
  `declared-in-members`.
- The engine SHALL omit the `doc` of a `vocabulary` term whose `Description`
  is empty, in `model` and in the record alike.
- A transition `Guard` that is a clause id SHALL copy as the semantic-core
  `ClauseRef` `{ language: "quire", clauseId }` with no `sourceSpan`; an inline
  `guardExpression` is not a `Transition` field and is not copied.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-081-CON-1 | The engine SHALL NOT infer, default or derive a score, status, acceptance or identity a document did not state. | Safety | Test |
| FR-081-CON-2 | An artifact whose module does not name `record-sections` SHALL yield a record byte-identical to the FR-076 record: no `record`, `recordSources` or `availability.recordKeys` key. | Compatibility | Test |
| FR-081-CON-3 | The golden fixtures of quoin FR-121, FR-122 and FR-123 SHALL live in the `quire-fixtures` crate and in no other repository. | Integrity | Inspection |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-081-AC-1 | A lifecycle fixture whose `## Assessment` table has the header `Severity \| Likelihood \| Exposure \| Controllability \| Rationale` and one row extracts an `assessment` object with a `recordSources` entry spanning the section, and `declaration_record` validates against the fixture's synthetic record schema; an axis authored `not_assessed` is that string, an empty `Exposure` cell is absent, and a record with no `## Assessment` has no `assessment` key and `availability.recordKeys` `not_applicable`. The quoin criteria that name `spec-objects-safety` schemas (`HazardAssessment.json`, `Hazard.json`) are not run here: that module is not fetchable by the repository's fixture mechanism, and its schemas are never copied. | Test |
| FR-081-AC-2 | A `failure_mode` `## Analysis` table extracts an `analysis` object; `## Status` holding `` `accepted` `` and a `## Provenance` table extract `status` and `provenance`, the same fixture without `## Provenance` extracts none, and a frontmatter `status: accepted` yields no `status`. | Test |
| FR-081-AC-3 | A `## Evidence` table `Target \| Kind` extracts `evidence` in row order with each target resolved by the FR-076 `Target` rules (the no-index advisory included); a header-only table extracts `[]` and is `available`. | Test |
| FR-081-AC-4 | The cases `column-unknown`, `column-duplicate`, `second-row`, `no-row`, `nested-object-cell` and `cell-type` each yield `semantic.invalid-model-cell` with that `reason` at the row or header, a second `## Assessment` yields `semantic.duplicate-section` `second-section`, and each key is `unavailable` with `entry-errors: lines <lines>`, no value and no `recordSources` entry. | Test |
| FR-081-AC-5 | A bullet list under a table-form key and a list after a valid table each yield `semantic.feature-not-extractable` with `reason` the key name; a prose-only record-key section yields the warning `semantic.record-key-no-block` and `not_applicable`. | Test |
| FR-081-AC-6 | Without the token the AC-1 fixture yields no `record`, no diagnostic and no `availability.recordKeys`; under the token a heading that names no key (`## Notes`) and one that names an owned key (a typed relation key, a key a declared table locator fills) are prose with no diagnostic. | Test |
| FR-081-AC-7 | A record with a camelCase key (`retryPolicy`) is reached by `## Retry Policy`; two properties that normalize equal yield `semantic.feature-not-extractable` `key-ambiguous`. | Test |
| FR-081-AC-8 | `provider` and `exposes` sections each holding one inline id extract one `SemanticId`; the `delivery`, `versioning`, `registration` and `stability` one-row tables extract objects, an omitted optional cell omits its field, and a bullet-list `## Registration` is `semantic.feature-not-extractable`. | Test |
| FR-081-AC-9 | A `## Routes` table extracts `routes`; a route naming an operation the artifact does not declare yields `route-operation-unknown`. | Test |
| FR-081-AC-10 | A `## Associated Types` table `Type \| Multiplicity` extracts `associated_types` as `TypeRef[]`; an `interface` whose `## Contract` fence lists `associated_types` that its table omits extracts none from the fence. | Test |
| FR-081-AC-11 | A `binary_format` `## Layout` fence extracts `endianness` `little` and two `RecordLayout`s with `tag: 0x21` as `"0x21"`, `size: payload_len` kept as the identifier resolving to the other record's field, and `magic` absent; a `size` naming no field yields `size-unknown-field`, one naming a field in two records `size-ambiguous`; a `## Endianness` section is prose. | Test |
| FR-081-AC-12 | A `## Thresholds` row `per_token \| requests \| 60 \| 1m \| 429 \| true \|` extracts a `Threshold` with `on_exceeded` `{ status: 429, retry_after: true }` and no `behavior`; the same row with an empty `Status` yields `status-required` at the row. | Test |
| FR-081-AC-13 | `semantic-v1.schema.json` admits `record`, `recordSources` and `availability.recordKeys`; every record of AC-1..AC-12 validates against it, every existing semantic case record is unchanged (FR-076-AC-9), and `declaration_record` carries the `record` keys. | Test |
| FR-081-AC-14 | Every top-level property of every object-type record schema that `agent-ix/spec-objects-architecture` ships is owned by exactly one of quoin FR-071, FR-072, FR-104, FR-121, FR-122, or quire-rs FR-075, FR-076 or this requirement, and none is declared not-authored; a property no owner claims fails the test (quoin FR-122-AC-7, TC-2115). | Test |
| FR-081-AC-15 | An `enumeration` `## Values` table `Value \| Description` with the rows `draft \| Not yet reviewed` and `final \|` copies to `values` as two elements, the second with no `doc`, with a `recordSources` entry spanning the section and `values` `available`, and no `semantic.invalid-model-cell` `column-unknown` or second reading of the section; a header-only table copies `[]`; a bullet list is `semantic.feature-not-extractable` `values`; a repeated `Value` and an invalid `Value` are refused and the key is `unavailable` with the model's `entry-errors` reason. A `state_machine` copies `states` and `transitions` (a clause-id `Guard` as `{ language: "quire", clauseId }`, `Emits` as one `SemanticId`); a `process` copies `steps` with `consumes` and `emits` as `SemanticId[]`, and its `## States` table stays in `model` and adds no `states` key to a record whose schema declares none; `recordSources` follows the document order of the sections. | Test |
| FR-081-AC-16 | An `aggregate_root` `## Members` with `OrderLine \| 0..*` (a `nested_entity`) and `Money \| 3..3` (a `value_object`) copies two `RelationDecl`s, `aggregates` and `contains`, both composite with the target `SemanticId`; a member no composite verb admits yields `member-verb`, an unknown name `semantic.unknown-reference`, a member of a type the construct `references.members` does not admit `semantic.reference-kind-mismatch`, and an `aggregates` row in `## Relationships` yields `semantic.duplicate-model-entry` `declared-in-members`; with no bundle index the key is `unavailable` with reason `no-bundle-index`. A `domain` `## Ubiquitous Language` table copies `vocabulary` with an empty `Description` as an absent `doc` (record validation refuses it), and reads no `ubiquitous_language`; a `nested_entity` `## Parent` and `## Owner` yield no `owner` and no diagnostic; a `repository` `persists` row extracts `persists` while `## Persists` and `## Emits` sections are prose; an `event` `## Source` inline id extracts `source`. | Test |
| FR-081-AC-17 | A property whose schema is a `#/$defs` fragment `$ref` is read; a property whose shape no mapping reads (an array of arrays, a `TypeRef[]` other than `associated_types`) named by a heading yields `semantic.feature-not-extractable` `schema-unreadable` and the key `unavailable`; a `## Associated Types` table in a module other than the architecture module is read; a `thresholds` row with an invalid `Status` is refused once, as `cell-type`. | Test |

## Dependencies

- **Upstream**: [FR-070](./FR-070-typed-properties-extraction.md), [FR-072](./FR-072-semantic-extraction-surface.md), [FR-075](./FR-075-model-feature-extraction.md), [FR-076](./FR-076-relationships-extraction.md); `agent-ix/quoin` FR-121, FR-122, FR-123
- **Quoin FR-123 coverage**: AC-1 and AC-9 by FR-081-AC-15, AC-2 by FR-081-AC-15, AC-3 and AC-5 by FR-081-AC-15, AC-4 and AC-6 and AC-7 by FR-081-AC-16, and AC-8 only in part: the ownership audit runs over `spec-objects-architecture` (FR-081-AC-14) but not over `spec-objects-business`, whose package the fixture mechanism does not fetch and whose schemas this repository never copies. Quoin's business-module schema validations (`EnumValue.json`, `Enumeration.json`, `Transition.json`, ...) are likewise not run here; the cases validate against synthetic roots.
- **FR-075 CR-192 note**: corrected to say `owner` is never extracted and `Event.source` is read by this requirement.
- **Downstream**: `agent-ix/spec-objects-architecture`, `agent-ix/spec-objects-safety`, `agent-ix/quoin`
