---
id: FR-075
title: "Model feature extraction driven by the module manifest"
type: FR
verification_method: test
evidence:
  - kind: test_case
    ref: tests/semantic_model.rs
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
redefinition, operation contracts, effect frames, populations, and the
object-type sections) into typed declarations with source spans, under the
record key `model`.

The engine SHALL extract a feature only when the module manifest declares
it. An artifact that declares a feature its manifest does not declare SHALL
be refused with the artifact, the section, and the span. The engine SHALL
NOT infer or approximate a feature from any other form.

A declaration's identity is its artifact `id`; its class name is the
declared name (the artifact `title`). Clauses are Quire clauses; the engine
carries them by clause id and never parses them
([FR-071](./FR-071-clause-and-operation-extraction.md)).

## Inputs

- The document and the `SemanticContext` of
  [FR-070](./FR-070-typed-properties-extraction.md) and
  [FR-071](./FR-071-clause-and-operation-extraction.md).
- The module `semantic.mappings` list, recorded on `SemanticModule` at load
  ([FR-069](./FR-069-semantic-module-contract-at-load.md)). The feature
  tokens are `generalization`, `abstract-types`, `presence`, `subsetting`,
  `redefinition`, `operation-contracts`, and `effect-frames`.
- The object type's `body_extraction` DSL. Every `table_row` locator with
  `under_section: <Section>` and `assert.columns` is a declared table.

## Outputs

`model` on the FR-072 record, present only when the artifact declares at
least one model feature, with these optional members. Every `sourceSpan` is a
semantic-core `SourceLocus` built as FR-071 builds clause spans: `startLine`
the declaring line, `startColumn` 1, `endLine` the same line, `endColumn` one
past that line's byte length.

| Member | Declared in the artifact | Entry |
|---|---|---|
| `supertypes` | frontmatter `relationships` entry with `type: specializes` | `{ target, sourceSpan }`, `target` verbatim |
| `abstract` | frontmatter `abstract: <bool>` | `{ value, sourceSpan }` |
| `fieldFeatures` | `## Properties` typed table columns `Presence`, `Subsets`, `Redefines` | `{ field, presence?, subsets?, redefines?, sourceSpan }` |
| `operationFrames` | `Requires:`, `Ensures:`, `Modifies:`, `Creates:`, `Deletes:` lines under an operation | `{ operation, requires, ensures, modifies, creates, deletes, sourceSpan }` |
| `population` | table `Type \| Extent` | `{ members: [{ type, extent, sourceSpan }] }` |
| `values` | table `Value \| Description` | `{ value, doc?, sourceSpan }` |
| `states` | table `State \| Description` | `{ value, doc?, sourceSpan }` |
| `transitions` | table `From \| To \| Trigger \| Guard \| Emits` | `{ from, to, trigger, guard?, emits?, sourceSpan }` |
| `steps` | table `Step \| Kind \| Consumes \| Emits \| Description` | `{ name, kind, consumes?, emits?, doc?, sourceSpan }` |
| `members` | table `Member \| Multiplicity` | `{ target, multiplicity, sourceSpan }` |
| `vocabulary` | table `Term \| Description` | `{ term, doc, sourceSpan }` |

`availability.model` carries the FR-072 availability of the frontmatter and
section features; it is present exactly when `model` is present or a model
feature was refused.

## Behavior

Manifest gating:

- The engine SHALL extract a frontmatter, column, or operation-line feature
  only when `semantic.mappings` names its token: `specializes` needs
  `generalization`, `abstract` needs `abstract-types`, each column needs
  `presence`, `subsetting`, or `redefinition`, `Requires:`/`Ensures:` need
  `operation-contracts`, and `Modifies:`/`Creates:`/`Deletes:` need
  `effect-frames`.
- The engine SHALL extract a model table only when the object type's
  `body_extraction` declares a `table_row` locator whose `under_section`
  equals the `##` section holding the table and whose `assert.columns` equal
  the table header, in order.
- If an artifact declares a feature its manifest does not declare, then the
  engine SHALL emit the error `semantic.feature-not-extractable` at the
  declaring line with `reason` the feature name, and a message naming the
  artifact path, the section (`frontmatter`, or the `##` heading), and the
  feature; the owning kind SHALL be `unavailable` and carry no entry.

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
- A `Presence` cell SHALL be `required` or `optional`; a `Subsets` cell a
  comma-separated list of `Identifier`s; a `Redefines` cell one `Identifier`.
  An empty cell declares nothing. Any other value is
  `semantic.invalid-model-cell` at the row and fails the `fields` kind as a
  row error does.
- A row with at least one non-empty feature cell SHALL yield one
  `fieldFeatures` entry, in row order.

Operation lines:

- `Requires:` and `Ensures:` SHALL list clause ids as `Pre:` and `Post:` do,
  and SHALL fill the operation's `pre` and `post`. A `Requires:` beside a
  `Pre:`, or an `Ensures:` beside a `Post:`, is
  `semantic.duplicate-operation-line`.
- `Modifies:`, `Creates:`, and `Deletes:` SHALL each list comma-separated
  names, each an `Identifier` or a dotted path of `Identifier`s; any other
  name is `semantic.invalid-model-cell` at the line. A second line of the
  same key is `semantic.duplicate-operation-line`.
- An operation carrying at least one of the five lines SHALL yield one
  `operationFrames` entry spanning its heading line.

Tables:

- A `Value`, `State`, `Step`, `From`, `To`, or `Trigger` cell SHALL be an
  `Identifier`; a `Description` cell is carried verbatim, and an empty one
  is absent.
- A `Kind` cell SHALL be one of `command`, `event`, `decision`,
  `compensation`, `wait`. `Consumes` and `Emits` cells SHALL list
  comma-separated names, carried verbatim.
- A transition `From` or `To` that names no `states` entry of the artifact
  is `semantic.unknown-state`; a `Trigger` that names no operation of the
  artifact is `semantic.unknown-trigger`; a `Guard` that names no invariant
  clause is `semantic.dangling-clause-ref`.
- A `Multiplicity` or `Extent` cell SHALL map as the FR-070 multiplicity
  cell; a `Type` cell SHALL resolve as the FR-070 type cell.
- A second entry with the same `Value`, `State`, `Step`, `Term`, `Member`,
  or `Type` in one table is `semantic.duplicate-model-entry`.
- Any other cell error is `semantic.invalid-model-cell` at the row.
- A second declared table of the same feature in one artifact is
  `semantic.duplicate-section` at its header line.

General:

- If any model feature carries an error, then the engine SHALL set
  `availability.model` to `unavailable` with reason `entry-errors` listing
  the loci, and SHALL omit `model` from the record.
- The engine SHALL NOT read a table whose header is not exactly one of the
  column sets above as a model feature.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-075-CON-1 | An artifact that declares no model feature SHALL yield a record byte-identical to the FR-072 record: no `model` key and no `availability.model` key. | Compatibility | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-075-AC-1 | Under `mappings: [generalization, abstract-types]`, an artifact with two `specializes` relationships and `abstract: true` yields two `supertypes` entries with verbatim targets and entry-line spans, and `abstract` `{ value: true }` at its line; a non-boolean `abstract` yields `semantic.invalid-model-cell`. | Test |
| FR-075-AC-2 | Under `mappings: [presence, subsetting, redefinition]`, a typed table with `Presence`, `Subsets`, `Redefines` columns yields the FR-070 `fields` plus one `fieldFeatures` entry per row declaring a feature, with row spans; a `Presence` of `maybe` yields `semantic.invalid-model-cell` and `fields` `unavailable`. | Test |
| FR-075-AC-3 | Under `mappings: [operation-contracts, effect-frames]`, an operation with `Requires:`, `Ensures:`, `Modifies:`, `Creates:`, and `Deletes:` lines yields `pre`/`post` and one `operationFrames` entry at its heading; `Requires:` beside `Pre:` yields `semantic.duplicate-operation-line`; a dangling `Requires:` id yields `semantic.dangling-clause-ref`. | Test |
| FR-075-AC-4 | A `population` artifact whose object type declares a `Type \| Extent` table extracts each member with its resolved `TypeRef`, its `extent` multiplicity, and its row span. | Test |
| FR-075-AC-5 | Under declared table locators, `Values`, `States`, `Transitions`, `Steps`, `Members`, and `Ubiquitous Language` tables extract to their typed entries with row spans; a transition to an undeclared state yields `semantic.unknown-state`, an unknown trigger `semantic.unknown-trigger`, and an unknown step kind `semantic.invalid-model-cell`. | Test |
| FR-075-AC-6 | Each feature of AC-1..AC-5 authored under a manifest that does not declare it yields the error `semantic.feature-not-extractable` at the declaring line, with the feature as `reason` and a message naming the artifact path and section, and the owning kind `unavailable` with no entry. | Test |
| FR-075-AC-7 | An artifact declaring no model feature yields no `model` and no `availability.model`, every existing semantic case record is unchanged, and every record of AC-1..AC-6 validates against `semantic-v1.schema.json`. | Test |

## Dependencies

- **Upstream**: [FR-069](./FR-069-semantic-module-contract-at-load.md), [FR-070](./FR-070-typed-properties-extraction.md), [FR-071](./FR-071-clause-and-operation-extraction.md), [FR-072](./FR-072-semantic-extraction-surface.md)
- **Downstream**: `agent-ix/filament-core-data#148` (domain package lift); `agent-ix/spec-objects-business` (manifest `mappings` and `table_row` locators)
