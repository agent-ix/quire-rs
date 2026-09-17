---
id: FR-075
title: "Model feature extraction driven by the module manifest"
type: FR
verification_method: test
evidence:
  - kind: test_case
    ref: tests/semantic_model.rs
  - kind: test_case
    ref: tests/semantic_contract.rs
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-019"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-069"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-070"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-071"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-072"
    type: "requires"
    cardinality: "1:1"
---
# FR-075: Model feature extraction driven by the module manifest

## Description

When the engine extracts an object artifact whose module carries a
`semantic` block, the engine SHALL extract every model feature the artifact
declares (generalization, abstract types, presence, subsetting,
redefinition, effect frames, populations, and the
object-type sections) into typed declarations with source spans, under the
record key `model`.

The engine SHALL extract a feature only when the module manifest declares
it. An artifact that declares a feature its manifest does not declare SHALL
be refused with the artifact, the section, and the span. The engine SHALL
NOT infer or approximate a feature from any other form.

A declaration's identity is its artifact `id`; its class name is the
declared name (the artifact `title`). Clauses are written in `quire`, the only
checked clause language; the engine carries them by
clause id and never parses them
([FR-071](./FR-071-clause-and-operation-extraction.md)).

## Inputs

- The document and the `SemanticContext` of
  [FR-070](./FR-070-typed-properties-extraction.md) and
  [FR-071](./FR-071-clause-and-operation-extraction.md).
- The module `semantic.mappings` list, recorded on `SemanticModule` at load
  ([FR-069](./FR-069-semantic-module-contract-at-load.md)). The feature
  tokens are `generalization`, `abstract-types`, `presence`, `subsetting`,
  `redefinition`, and `effect-frames`.
- The object type's typed `body_extraction` DSL. A `table_row` locator in
  `yield_pattern.match` or `yield_pattern.per_match` (a fallback chain
  included) declares a model table when it sets `under_section`, its
  `assert.columns` start with one table key of the Outputs table and name
  only that table's columns, and its `assert.optional_columns` name none of
  that table's required columns.
- The document path. The Filament and Python surfaces pass the artifact
  path; `validate_document` has no path, and its spans and messages name
  `<document>`.

## Outputs

`model` on the FR-072 record, present exactly when `availability.model` is
`available`, with these optional members. Every `sourceSpan` is a
semantic-core `SourceLocus` built as FR-071 builds clause spans: `startLine`
the declaring line, `startColumn` 1, `endLine` the same line, `endColumn` one
past that line's byte length.

| Member | Declared in the artifact | Entry |
|---|---|---|
| `identity` | frontmatter `id` | `{ value, sourceSpan }` |
| `displayName` | frontmatter `title` | `{ value, sourceSpan }` |
| `supertypes` | frontmatter `relationships` entry with `type: specializes` | `{ target, sourceSpan }`, `target` verbatim |
| `abstract` | frontmatter `abstract: <bool>` | `{ value, sourceSpan }` |
| `fieldFeatures` | `## Properties` typed table columns `Presence`, `Subsets`, `Redefines` | `{ field, presence?, subsets?, redefines?, sourceSpan }` |
| `operationFrames` | `Modifies:`, `Creates:`, `Deletes:` lines under an operation, with its `Requires:` and `Ensures:` ids | `{ operation, requires, ensures, modifies, creates, deletes, sourceSpan }` |
| `population` | table `Type \| Extent` (both required) | `{ members: [{ type, extent, sourceSpan }] }` |
| `values` | table `Value \| Description` (`Value` required) | `{ value, doc?, sourceSpan }` |
| `states` | table `State \| Description` (`State` required) | `{ value, doc?, sourceSpan }` |
| `transitions` | table `From \| To \| Trigger \| Guard \| Emits` (`From`, `To`, `Trigger` required) | `{ from, to, trigger, guard?, emits?, sourceSpan }`, `emits` a name list |
| `steps` | table `Step \| Kind \| Consumes \| Emits \| Description` (`Step`, `Kind` required) | `{ name, kind, consumes?, emits?, doc?, sourceSpan }` |
| `members` | table `Member \| Multiplicity` (both required) | `{ target, multiplicity, sourceSpan }` |
| `vocabulary` | table `Term \| Description` (`Term` required) | `{ term, doc, sourceSpan }` |

`identity` and `displayName` are carried whenever `model` is present; they
declare no feature on their own.

`availability.model` is present exactly when the artifact declares at least
one model feature: a frontmatter feature, a Properties feature column, an
operation frame line, or any block under a section a declared
table owns, or any refused feature. It is `available` when no declaring
source failed, and `unavailable` with reason `entry-errors` listing the error
lines when a frontmatter feature, a model table, the Properties feature
columns, or an operation line carries an error or a refusal.

Every `semantic.feature-not-extractable` diagnostic carries `sourceSpan`, the
span of the declaring line, and `section`: `frontmatter`, `preamble` (the
body before the first `##` heading), the `##` heading text, or
`Operations / <operation>`.

## Behavior

Manifest gating:

- The engine SHALL extract a frontmatter, column, or operation-line feature
  only when `semantic.mappings` names its token: `specializes` needs
  `generalization`, `abstract` needs `abstract-types`, each column needs
  `presence`, `subsetting`, or `redefinition`, and
  `Modifies:`/`Creates:`/`Deletes:` need `effect-frames`. `Requires:` and
  `Ensures:` are the FR-071 contract lines and need no token.
- The engine SHALL extract a model table only when a declared table locator
  owns the `##` section holding it (`under_section` equals the heading) and
  the table header conforms to the locator: the header is an ordered
  subsequence of `assert.columns` that contains every column not named in
  `assert.optional_columns`. Several locators may own one section.
- If an artifact declares a feature its manifest does not declare, then the
  engine SHALL emit the error `semantic.feature-not-extractable` at the
  declaring line with `reason` the feature name, `sourceSpan`, `section`, and
  a message naming the artifact path, the section, and the feature; the
  owning kind SHALL be `unavailable` and carry no entry. An artifact declares
  an undeclared table feature when:
  - a table whose header starts with one table key of the Outputs table and
    names only that table's columns sits under a section no declared locator
    owns, or before the first `##` heading;
  - a block (table, fence, or list) under a section a declared locator owns
    does not conform to any owning locator; the refusal names the first
    owning locator's feature at the block's first line;
  - a typed Properties table with feature columns sits before the first `##`
    heading.

Frontmatter:

- A `relationships` entry whose `type` is `specializes` SHALL yield one
  `supertypes` entry whose `target` is the entry's `target` string, at the
  line of the entry. A second entry with the same target is
  `semantic.duplicate-model-entry`; a non-string target is
  `semantic.invalid-model-cell`.
- `abstract` SHALL be a boolean; any other value is
  `semantic.invalid-model-cell` at its line.

Properties columns:

- When a table header is `Field | Type | Multiplicity | Constraints`
  followed by one or more of `Presence`, `Subsets`, `Redefines` (each at
  most one time, in any order), the engine SHALL read it as the FR-070 typed
  table and map each row's first four cells as FR-070 maps them.
- If a column after the typed four-column prefix is not `Presence`,
  `Subsets`, or `Redefines`, or repeats one, then the engine SHALL emit
  `semantic.invalid-model-cell` at the header line with `section`
  `Properties` and `sourceSpan`, and set `fields` `unavailable` with reason
  `invalid-feature-column`; the table is not read as any other form.
- A `Presence` cell SHALL be `required` or `optional`; a `Subsets` cell a
  comma-separated list of `Identifier`s; a `Redefines` cell one `Identifier`.
  An empty cell declares nothing. Any other value is
  `semantic.invalid-model-cell` at the row and fails the `fields` kind as a
  row error does.
- A row with at least one non-empty feature cell SHALL yield one
  `fieldFeatures` entry, in row order.

Operation lines:

- `Modifies:`, `Creates:`, and `Deletes:` SHALL each list comma-separated
  names, each an `Identifier` or a dotted path of `Identifier`s; any other
  name is `semantic.invalid-model-cell` at the line. A second line of the
  same key is `semantic.duplicate-operation-line`.
- An operation carrying at least one `Modifies:`, `Creates:`, or `Deletes:`
  line SHALL yield one `operationFrames` entry spanning its heading line,
  whose `requires` and `ensures` list the ids of its FR-071 `Requires:` and
  `Ensures:` lines.

Tables:

- A `Value`, `State`, `Step`, `From`, `To`, or `Trigger` cell SHALL be an
  `Identifier`; a `Description` cell is carried verbatim, and an empty one
  is absent.
- A `Kind` cell SHALL be one of `command`, `event`, `decision`,
  `compensation`, `wait`. `Consumes` and `Emits` cells (in `steps` and in
  `transitions`) SHALL list comma-separated names, carried verbatim as a
  name list; an empty cell is absent.
- A cell of a column the header omits under `optional_columns` is absent.
- A transition `From` or `To` that names no `states` entry of the artifact
  is `semantic.unknown-state`, checked only when the `states` table carries
  no error or refusal; a `Trigger` that names no operation of the
  artifact is `semantic.unknown-trigger`; a `Guard` that names no invariant
  clause is `semantic.dangling-clause-ref`.
- A `Multiplicity` or `Extent` cell SHALL map as the FR-070 multiplicity
  cell; a `Type` cell SHALL resolve as the FR-070 type cell.
- A second entry with the same `Value`, `State`, `Step`, `Term`, `Member`,
  or `Type` in one table is `semantic.duplicate-model-entry`.
- Any other cell error is `semantic.invalid-model-cell` at the row.
- A second declared table of the same feature in one artifact is
  `semantic.duplicate-section` at its header line and fails that feature.

General:

- If any model feature carries an error or a refusal, then the engine SHALL
  set `availability.model` to `unavailable` with reason `entry-errors`
  listing the error lines, and SHALL omit the whole `model` from the record.
- The engine SHALL NOT read a table as a model feature unless a declared
  locator owns its section and its header conforms to that locator.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-075-CON-1 | An artifact that declares no model feature SHALL yield a record byte-identical to the FR-072 record: no `model` key and no `availability.model` key. | Compatibility | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-075-AC-1 | Under `mappings: [generalization, abstract-types]`, an artifact with two `specializes` relationships and `abstract: true` yields two `supertypes` entries with verbatim targets and entry-line spans, `abstract` `{ value: true }` at its line, and `availability.model` `available`; a non-boolean `abstract`, a repeated `specializes` target (`semantic.duplicate-model-entry`), and a non-string target (`semantic.invalid-model-cell`) each set `availability.model` `unavailable` and omit `model`; clean frontmatter features beside a refused operation line also omit `model`. | Test |
| FR-075-AC-2 | Under `mappings: [presence, subsetting, redefinition]`, a typed table with `Presence`, `Subsets`, `Redefines` columns yields the FR-070 `fields` plus one `fieldFeatures` entry per row declaring a feature, with row spans; `availability.model` is `available`; a `Presence` of `maybe` yields `semantic.invalid-model-cell`, `fields` `unavailable`, and no `model`; an unknown or repeated feature column yields only `semantic.invalid-model-cell` at the header with `section` `Properties`; a typed feature table before the first `##` heading is refused with `section` `preamble`. | Test |
| FR-075-AC-3 | Under `mappings: [effect-frames]`, an operation with `Requires:`, `Ensures:`, `Modifies:`, `Creates:`, and `Deletes:` lines yields `pre`/`post` and one `operationFrames` entry at its heading; a `Pre:` line fills no `pre`; a second `Requires:` yields `semantic.duplicate-operation-line`; a dangling `Requires:` id yields `semantic.dangling-clause-ref`; a second `Modifies:` line yields `semantic.duplicate-operation-line`; a frame name that is not an `Identifier` path yields `semantic.invalid-model-cell`; each error sets `availability.model` `unavailable` and omits `model`. | Test |
| FR-075-AC-4 | A `population` artifact whose object type declares a `Type \| Extent` table extracts each member with its resolved `TypeRef`, its `extent` multiplicity, and its row span. | Test |
| FR-075-AC-5 | Under declared table locators, `Values`, `States`, `Transitions`, `Steps`, `Members`, and `Ubiquitous Language` tables extract to their typed entries with row spans; a transition to an undeclared state yields `semantic.unknown-state`, an unknown trigger `semantic.unknown-trigger`, an unknown guard `semantic.dangling-clause-ref`, a repeated value `semantic.duplicate-model-entry`, an unknown step kind `semantic.invalid-model-cell`, and a second table under a declared section `semantic.duplicate-section`; `emits` is a name list; a failed `states` table suppresses `semantic.unknown-state`; a transitions table omitting its optional columns extracts, and one omitting a required column is refused; a mermaid fence and a bullet list under declared sections are refused with their section and line. | Test |
| FR-075-AC-6 | Each feature of AC-1..AC-5 authored under a manifest that does not declare it yields the error `semantic.feature-not-extractable` at the declaring line, with the feature as `reason`, `sourceSpan` of that line, `section`, and a message naming the artifact path and section, and the owning kind `unavailable` with no entry; a model table under an unowned section or before the first `##` heading is refused the same way. | Test |
| FR-075-AC-7 | An artifact declaring no model feature yields no `model` and no `availability.model`, every existing semantic case record is unchanged, and every record of AC-1..AC-6 validates against `semantic-v1.schema.json`. | Test |
| FR-075-AC-8 | A record carrying `model` carries `identity` with the frontmatter `id` and `displayName` with the frontmatter `title`, each spanning its frontmatter line. | Test |
| FR-075-AC-9 | `validate_document` and Filament extraction read a feature their module manifest declares (a mapping token or a `table_row` locator) and refuse one it does not; `validate_document` refusals name `<document>`, Filament refusals name the artifact path. | Test |

## Dependencies

- **Upstream**: [FR-069](./FR-069-semantic-module-contract-at-load.md), [FR-070](./FR-070-typed-properties-extraction.md), [FR-071](./FR-071-clause-and-operation-extraction.md), [FR-072](./FR-072-semantic-extraction-surface.md)
- **Downstream**: `agent-ix/filament-core-data#148` (domain package lift); `agent-ix/spec-objects-business` (manifest `mappings` and `table_row` locators)
