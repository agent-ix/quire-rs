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
  - kind: test_case
    ref: tests/semantic_systems.rs
  - kind: test_case
    ref: tests/semantic_features.rs
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
redefinition, effect frames, populations, the
object-type sections, the systems-model tables, the feature order, and the
dimension and unit tables) into typed declarations with source spans, under the
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
  included) declares a model table when its match key is that table's
  Member name in the Outputs table (`values`, `connection`, …), it sets
  `under_section`, its `assert.columns` start with that table's key column
  and name only that table's columns, and its `assert.optional_columns` name
  none of that table's required columns. A locator whose key names no table
  declares none. A Rust caller supplies it with
  `SemanticContext::with_body_extraction`, the one path every surface uses;
  without it no model table is declared. The same typed DSL gives the
  sections a document must carry through `RequiredSections::from_extraction`,
  the one reader of required sections; no surface scans DSL JSON keys of its
  own.
- The document path. The Filament and Python surfaces pass the artifact
  path; `validate_document` has no path, and its spans and messages name
  `<document>`.
- Each bundle artifact's declared operation names (`BundleArtifact.operations`
  on the `SemanticContext`'s bundle index): what an allocation `Source` of
  `<id>/<member>` checks `<member>` against. Every surface that supplies a
  bundle — the Rust API, the Filament `semanticBundle` JSON request, and the
  Python `bundle` argument — supplies `operations` for every artifact it
  lists; the field is required, not defaulted (Outputs). Corpus mode
  (`BundleIndex::from_documents`) derives it itself, from
  `clauses::declared_operation_names`. When `<id>` names the artifact's own
  document (a self-reference), `<member>` is checked the same way, against
  that same `declared_operation_names` scan run on the artifact's own text,
  not against the bundle at all.
- `declared_operation_names`' name list contains one entry per `### <name>`
  heading under a document's `## Operations` section: every heading whose
  text is an `Identifier`, read from the heading itself, independent of
  whether that operation's own body parses without error and independent of
  whether any *other* operation's body has an error. `extract_operations`'s
  own `operations` (FR-071 Outputs) is a separate list: the whole document's
  operations parsed as full declarations, present only when every one of
  them parses without error. A `<member>` check against a heading that
  exists but whose body is malformed still lifts; only a `<member>` naming
  no heading at all is refused.

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
| `operationFrames` | `Modifies:`, `Creates:`, `Deletes:` lines under an operation, with its `Pre:` and `Post:` ids | `{ operation, pre, post, modifies, creates, deletes, sourceSpan }` |
| `population` | table `Type \| Extent` (both required) | `{ members: [{ type, extent, sourceSpan }] }` |
| `values` | table `Value \| Description` (`Value` required) | `{ value, doc?, sourceSpan }` |
| `states` | table `State \| Description` (`State` required) | `{ value, doc?, sourceSpan }` |
| `transitions` | table `From \| To \| Trigger \| Guard \| Emits` (`From`, `To`, `Trigger` required) | `{ from, to, trigger, guard?, emits?, sourceSpan }`, `emits` a name list |
| `steps` | table `Step \| Kind \| Consumes \| Emits \| Description` (`Step`, `Kind` required) | `{ name, kind, consumes?, emits?, doc?, sourceSpan }` |
| `members` | table `Member \| Multiplicity` (both required) | `{ target, multiplicity, sourceSpan }` |
| `vocabulary` | table `Term \| Description` (`Term` required) | `{ term, doc, sourceSpan }` |
| `part` | table `Owner \| Declared Type \| Multiplicity` (all required), one row | `{ owner, declaredType, multiplicity, sourceSpan }` |
| `port` | table `Owner \| Direction \| Interface \| Multiplicity` (all required), one row | `{ owner, direction, interfaceType, multiplicity, sourceSpan }` |
| `connection` | table `Source \| Source Multiplicity \| Target \| Target Multiplicity \| Direction` (`Source`, `Target`, `Direction` required), one row | `{ sourceEnd, targetEnd, flowDirection, sourceSpan }`, each end `{ type, multiplicity? }` |
| `allocation` | table `Source \| Target` (both required), one row | `{ sourceElement, targetElement, sourceSpan }` |
| `featureOrder` | table `Feature \| Kind` (both required) | `{ name, kind, sourceSpan }`, in row order |
| `dimension` | table `Base` (required), one row | `{ base, sourceSpan }`, `base` a boolean |
| `terms` | table `Dimension \| Exponent` (both required) | `{ dimension, exponent, sourceSpan }`, `dimension` a `TypeRef` `{ target }` resolved as a dimension token, `exponent` an integer, in row order |
| `unit` | table `Dimension \| Target \| Scale \| Offset` (`Dimension`, `Scale`, `Offset` required, `Target` optional), one row | `{ dimension, target?, scale, offset, sourceSpan }`, `dimension` and `target` each a `TypeRef` `{ target }` (`dimension` resolved as a dimension token, `target` as a unit token), `scale` and `offset` canonical rational strings |

`owner`, `sourceElement`, `targetElement`, and a connection end's `type` are
semantic-core `SemanticId`s; `declaredType` and `interfaceType` are
`TypeRef`s. The keys of `part`, `port`, `connection`, and `allocation` other
than `sourceSpan` are also members of the declaration record the object
type's data schema validates, and so is `featureOrder`, as the list of its
entries' `name`s in row order.

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
  `Modifies:`/`Creates:`/`Deletes:` need `effect-frames`. `Pre:` and
  `Post:` are the FR-071 contract lines and need no token.
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
  whose `pre` and `post` list the ids of its FR-071 `Pre:` and
  `Post:` lines.

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

Dimension and unit tables:

- The engine SHALL extract a `dimension`, `terms` or `unit` table under the
  rules of Manifest gating. An artifact SHALL declare at most one `dimension`
  table, at most one `terms` table and at most one `unit` table, and SHALL NOT
  declare both a `dimension` table and a `unit` table (a dimension artifact and
  a unit artifact are different object types). The second table of one feature
  is `semantic.duplicate-section` at its header line, as for any feature; when
  a `dimension` table and a `unit` table are both declared, the later one is
  `semantic.duplicate-section` at its header line, and the engine extracts the
  earlier table's kind and omits the later's.
- A `dimension` or `unit` table SHALL hold exactly one row. A table with no
  row is `semantic.invalid-model-cell` at its header line; each row after the
  first is `semantic.duplicate-model-entry` at that row.
- A `Base` cell SHALL be `true` or `false`; any other value is
  `semantic.invalid-model-cell` at the row.
- The engine SHALL resolve a `Dimension` cell (an `Identifier`) of a `terms`
  row or a `unit` row as a *dimension token*, and a non-empty `Target` cell
  (an `Identifier`; an empty cell leaves `target` absent, a root unit) as a
  *unit token*, by the rules of
  [FR-070](./FR-070-typed-properties-extraction.md) "Unit and dimension
  tokens", which this requirement does not restate: a bare token or a
  qualified token `<org>/<repo>::<Name>`, matched by artifact `id` first and
  then by `names` (title or frontmatter `name`) among the own bundle and the
  imported packages' artifacts, the matched artifact's frontmatter `object`
  being `dimension` for a dimension token and `unit` for a unit token. A match
  maps to `ix://<package>/type/<Name>` (`<package>` the owner of the matched
  artifact, `<Name>` its `id`), so dimensions of two owners stay two
  identities. Token resolution relates no two dimensions: an equivalence
  between dimensions of two owners is declared by the unit of compilation
  (QSpec FR-142 "Dimension equivalence") and carried by FCD FR-146's
  `dimensionEquivalences`, never inferred from a token match. A token that
  names no artifact of the required object type maps to the placeholder
  `ix://<org>/<repo>/unresolved/<Token>` (`<Token>` the `Identifier` the token
  names) with the advisory `semantic.unresolved-type` (`reason`
  `unknown-token`, `no-bundle-index` or `import-unresolved` as FR-070 states)
  at the row, and the owning kind stays `available` with `lossy` true; two
  artifacts matching one token are the error `semantic.ambiguous-type` naming
  both, which fails the kind as a row error does. A token that is neither an
  `Identifier` nor a qualified token is `semantic.invalid-type-token`.
- An `Exponent` cell SHALL be a canonical integer: `0`, or an optional `-`
  then a non-zero digit followed by digits; any other value is
  `semantic.invalid-model-cell` at the row. A second `terms` row naming the
  same resolved target is `semantic.duplicate-model-entry`.
- A `Scale` or `Offset` cell SHALL be a canonical rational string: an
  integer in the `Exponent` form, or `n/d` with `n` a non-zero canonical
  integer, `d` an integer of at least 2 in canonical form, and `n` and `d`
  coprime. `1/1000`, `-3/2`, `0` and `-3` are canonical; `2/4`, `1/1`, `-0`,
  `0/5`, `01/3` and `1.5` are `semantic.invalid-model-cell` at the row.
- The engine checks each cell and each table of one artifact, and extracts a
  self-targeting unit, a unit cycle, a zero `Scale` and a base dimension that
  has a `terms` table unchanged, because those rules concern the graph of all
  the dimension and unit artifacts of a document. Those rules (a base dimension
  without terms, ascending terms, cycles, a nonzero scale, a target of the
  same dimension, one root per dimension) are specified once, with their
  diagnostics, as the reader rules of `agent-ix/filament-core-data` FR-146
  "Dimensions" and "Units" (`agent-ix.semantic-ir.INVALID_UNIT_GRAPH`), which
  apply to the document the frontend lifts from these extracted values; this
  engine specifies no second copy.
- The engine SHALL carry the rows in the entries of the Outputs table in row
  order and SHALL NOT compute, reduce or compare any dimension or unit.

Systems-model tables:

- An artifact SHALL declare at most one of the `part`, `port`,
  `connection`, and `allocation` tables; a second is
  `semantic.duplicate-section` at its header line.
- A `part`, `port`, `connection`, or `allocation` table SHALL hold exactly
  one row. A table with no row is `semantic.invalid-model-cell` at its header
  line; each row after the first is `semantic.duplicate-model-entry` at that
  row.
- An `Owner`, `Source`, or `Target` cell SHALL name an object `id` (an
  ASCII letter, then ASCII letters, digits, or `_`) or an
  `ix://<org>/<repo>/<id>` identity; an allocation `Source` MAY instead name
  `<id>/<member>` or `ix://<org>/<repo>/<id>/<member>`, `<member>` an
  `Identifier`. Any other value, an empty cell or a hyphenated id included,
  is `semantic.invalid-model-cell` at the row. A bare `id` lowers to
  `ix://<package>/<id>`, `<package>` the bundle's package, else the
  module's.
- The cell resolves as an FR-076 relationship target does, through the one
  shared resolver: an own-package `id` SHALL resolve to the artifact itself
  or to a bundle artifact, and an identity in a package the module imports
  resolves unchecked. An own-package `id` no bundle artifact carries, or an
  identity in a package the module does not import, is
  `semantic.unknown-reference` at the row. A bundle with no artifacts lowers
  an own-package cell unchecked with the advisory
  `semantic.unresolved-target`, reason `no-bundle-index`, emitted only when
  the row lowers. The `<member>` of `<id>/<member>` is not checked when the
  identity is imported or unchecked (no bundle index).
- A resolved bundle artifact SHALL have an object type the cell admits. A
  part `Owner` names any declared object type but a `part`, `port`,
  `connection`, or `allocation` — those four have no owner effective type
  (QSpec FR-152) — and a declarer with no object type at all is also
  refused. A port `Owner` and an allocation `Target` name a `part`; a
  connection `Source` or `Target` names a `port`; an allocation `Source`
  names a `part`, a `port`, or by `<id>/<member>` any declared object type
  but the same four systems-record kinds a part `Owner` refuses — an
  operation belongs to a type (QSpec FR-152's "effective view"), and a
  systems record structurally declares no `## Operations` section, so it
  never has a member to name. A declarer of no object type at all, or one
  of those four kinds where the role refuses them, is
  `semantic.reference-kind-mismatch` at the row. An imported or unchecked
  identity's object type is not checked.
- An allocation `Source` of `<id>/<member>` names an operation the `<id>`
  bundle artifact declares: `<member>` SHALL be one of its
  `BundleArtifact.operations`, checked only once `<id>`'s object type has
  passed the kind check above. A `<member>` the artifact does not declare
  is `semantic.unknown-reference` at the row.
- A part `Owner` SHALL NOT name the part itself; one that does is
  `semantic.reference-kind-mismatch` at the row.
- The declaration record SHALL carry at most one systems-model record; a
  `SemanticExtraction` holding more than one (e.g. deserialized with both
  `model.part` and `model.port`) makes `declaration()` and
  `declaration_record()` return an error.
- A `Declared Type` or `Interface` cell SHALL map as the FR-070 type cell,
  and a `Multiplicity` cell as the FR-070 multiplicity cell. A `Source
  Multiplicity` or `Target Multiplicity` cell maps as the FR-070 multiplicity
  cell when non-empty; an empty one gives the end no `multiplicity`.
- A port `Direction` cell SHALL be one of `in`, `out`, `inout`; a connection
  `Direction` cell one of `source-to-target`, `target-to-source`,
  `bidirectional`. Any other value is `semantic.invalid-model-cell` at the
  row.

Feature order:

- A `Feature` cell SHALL be an `Identifier` and a `Kind` cell one of `field`,
  `operation`; any other value is `semantic.invalid-model-cell` at the row. A
  second row with the same `Feature` is `semantic.duplicate-model-entry`.
- When the table carries no error, each row SHALL name a declared field (the
  FR-070 `fields`) or operation (the FR-071 `operations`) of the artifact, of
  its `Kind`. A row naming a declaration of the other kind is
  `semantic.feature-kind-mismatch` at the row; a row naming neither is
  `semantic.unknown-feature` at the row.
- Each declared field and operation SHALL have a row; one with no row of
  either `Kind` is `semantic.missing-feature` at the table's header line, so
  a row of the wrong kind yields only `semantic.feature-kind-mismatch`.
- A kind whose availability is `unavailable` is not checked: neither its
  declarations nor any row whose `Kind`, or the other `Kind`, it is.

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
| FR-075-AC-3 | Under `mappings: [effect-frames]`, an operation with `Pre:`, `Post:`, `Modifies:`, `Creates:`, and `Deletes:` lines yields `pre`/`post` and one `operationFrames` entry at its heading; a second `Pre:` yields `semantic.duplicate-operation-line`; a dangling `Pre:` id yields `semantic.dangling-clause-ref`; a second `Modifies:` line yields `semantic.duplicate-operation-line`; a frame name that is not an `Identifier` path yields `semantic.invalid-model-cell`; each error sets `availability.model` `unavailable` and omits `model`. | Test |
| FR-075-AC-4 | A `population` artifact whose object type declares a `Type \| Extent` table extracts each member with its resolved `TypeRef`, its `extent` multiplicity, and its row span. | Test |
| FR-075-AC-5 | Under declared table locators, `Values`, `States`, `Transitions`, `Steps`, `Members`, and `Ubiquitous Language` tables extract to their typed entries with row spans; a transition to an undeclared state yields `semantic.unknown-state`, an unknown trigger `semantic.unknown-trigger`, an unknown guard `semantic.dangling-clause-ref`, a repeated value `semantic.duplicate-model-entry`, an unknown step kind `semantic.invalid-model-cell`, and a second table under a declared section `semantic.duplicate-section`; `emits` is a name list; a failed `states` table suppresses `semantic.unknown-state`; a transitions table omitting its optional columns extracts, and one omitting a required column is refused; a mermaid fence and a bullet list under declared sections are refused with their section and line. | Test |
| FR-075-AC-6 | Each feature of AC-1..AC-5 authored under a manifest that does not declare it yields the error `semantic.feature-not-extractable` at the declaring line, with the feature as `reason`, `sourceSpan` of that line, `section`, and a message naming the artifact path and section, and the owning kind `unavailable` with no entry; a model table under an unowned section or before the first `##` heading is refused the same way. | Test |
| FR-075-AC-7 | An artifact declaring no model feature yields no `model` and no `availability.model`, every existing semantic case record is unchanged, and every record of AC-1..AC-6 validates against `semantic-v1.schema.json`. | Test |
| FR-075-AC-8 | A record carrying `model` carries `identity` with the frontmatter `id` and `displayName` with the frontmatter `title`, each spanning its frontmatter line. | Test |
| FR-075-AC-9 | `validate_document` and Filament extraction read a feature their module manifest declares (a mapping token or a `table_row` locator) and refuse one it does not; `validate_document` refusals name `<document>`, Filament refusals name the artifact path. | Test |
| FR-075-AC-10 | A Rust caller that builds `SemanticContext` with `with_body_extraction` and a typed `body_extraction` declaring a `Values` locator extracts the table, and the record equals the one `extract_semantic_json` returns for the same request; the same context without `with_body_extraction` refuses the table with `semantic.feature-not-extractable`. | Test |
| FR-075-AC-11 | `RequiredSections::from_extraction` over a typed `body_extraction` marks exactly the `Properties`, `Invariants`, and `Operations` headings a required locator (any primitive, including a fallback-chain member, under `under_section` or `after_heading`) sits under, wherever the locator is declared (`yield_pattern.match`, `yield_pattern.per_match`, or an `emit_edges` target), for a DSL authored as JSON with the module key names. | Test |
| FR-075-AC-12 | Under declared locators, a `part`, `port`, `connection`, and `allocation` table each extract to their typed entry with resolved `SemanticId`s, `TypeRef`s, multiplicities, direction, and row span; an empty end multiplicity cell, or end multiplicity columns the locator omits, give no end `multiplicity`; an allocation `Source` of `<id>/<member>` extracts when `<id>` declares the operation `<member>`; own-package `ix://` and imported identities resolve, and a non-imported package yields `semantic.unknown-reference`; a name no bundle artifact carries, and an allocation `Source` `<id>/<member>` whose `<id>` artifact does not declare `<member>`, yield `semantic.unknown-reference`; a bundle with no artifacts lowers a name unchecked with the advisory `semantic.unresolved-target`, and a failing row carries no advisory; a part `Owner` naming a `part`, `port`, `connection`, `allocation`, or a declarer with no object type, a Port `Owner` or allocation `Target` naming a non-part, an allocation `Source` naming neither a part nor a port, and a part `Owner` naming the part itself each yield `semantic.reference-kind-mismatch`; a record holding more than one systems-model record makes `declaration_record()` return an error; an unknown port direction, an unknown connection direction, a hyphenated id, an empty table (at its header), and `<id>/<member>` in an `Owner` or a connection end each yield `semantic.invalid-model-cell`; a second row yields `semantic.duplicate-model-entry`; a second systems-model table yields `semantic.duplicate-section`; each error sets `availability.model` `unavailable` with no `model`; a locator's match key, not its columns, names the table it declares; an undeclared `Source \| Target` table is refused as `allocation`. | Test |
| FR-075-AC-13 | Under the published `@agent-ix/spec-objects-architecture` module, resolved as a dependency at test time (PLAT-901), `validate_document` of each of the `part`, `port`, `connection`, and `allocation` skeletons yields zero errors, and each declaration record carries exactly the keys its data schema requires. | Test |
| FR-075-AC-14 | Under a declared `Feature \| Kind` locator, an artifact with one field and two operations whose `Features` table interleaves them yields `model.featureOrder` in row order with row spans; its declaration record carries `featureOrder` as the names in row order and satisfies spec-objects-architecture's `Interface.json`. | Test |
| FR-075-AC-15 | A `Features` row naming no declared feature yields `semantic.unknown-feature`, a row whose `Kind` is not the declared kind `semantic.feature-kind-mismatch`, an unknown `Kind` `semantic.invalid-model-cell`, and a repeated `Feature` `semantic.duplicate-model-entry`, each at its row; a declared feature with no row yields `semantic.missing-feature` at the table header, and a wrong-kind row yields no `semantic.missing-feature`; each sets `availability.model` `unavailable` with no `model`. When `operations` is `unavailable`, rows naming operations as either kind and operations without rows yield no feature diagnostic. | Test |
| FR-075-AC-16 | Under a declared `Base` locator, an artifact whose `Base` table row is `true` yields `model.dimension` `{ base: true }` with the row span and `availability.model` `available`; `false` yields `base: false`; `yes`, an empty cell, a table with no row (`semantic.invalid-model-cell` at the header) and a second row (`semantic.duplicate-model-entry` at that row) are each refused and fail the model kind with `entry-errors`; two `Base` tables in one artifact yield `semantic.duplicate-section` at the second table's header line. | Test |
| FR-075-AC-17 | Under a declared `Dimension \| Exponent` locator and a bundle with dimension artifacts `Length` (title `Length`) and `Time`, rows `Length \| 1` and `Time \| -2` yield `model.terms` of two entries in row order with `dimension` targets `ix://<org>/<repo>/type/Length` and `.../Time` and exponents `1` and `-2`; a row `Speed \| 1` where `Speed` is a unit artifact yields the placeholder `ix://<org>/<repo>/unresolved/Speed` with `semantic.unresolved-type` reason `unknown-token` and `lossy` true; `Length \| 01`, `Length \| 1.5` and `Length \| x` are `semantic.invalid-model-cell`; a repeated `Length` row is `semantic.duplicate-model-entry`; two dimension artifacts both titled `Length` give `semantic.ambiguous-type` naming both; a non-`Identifier` `Dimension` cell gives `semantic.invalid-type-token`. | Test |
| FR-075-AC-18 | Under a declared `Dimension \| Target \| Scale \| Offset` locator with `Target` optional, the row `Time \| s \| 1/1000 \| 0` yields `model.unit` `{ dimension: Time's identity, target: s's identity, scale: "1/1000", offset: "0" }` with the row span, where `s` is a unit artifact; a root row `Time \| \| 1 \| 0` yields no `target`; a locator without the `Target` column yields no `target`; `Scale` or `Offset` of `-3/2`, `-3` and `0` are accepted; `2/4`, `1/1`, `-0`, `0/5`, `01/3`, `1.5`, `1/0` and an empty cell are `semantic.invalid-model-cell` at the row; a `Target` naming a dimension artifact yields the unit-kind placeholder with `semantic.unresolved-type`; a table with no row or two rows is refused as in AC-16; a unit whose target is itself, whose graph has a cycle or whose `Scale` is `0` extracts unchanged (the engine decides no unit-graph rule); under a qualified `Dimension` cell `acme/time-units::Time` and a qualified `Target` cell `acme/time-units::s` (both held by `importedArtifacts`) the row yields the identities `ix://acme/time-units/type/Time` and `ix://acme/time-units/type/s`, while the same bare cell `Time` held by both the own bundle and an imported package yields the own identity. | Test |
| FR-075-AC-19 | Under a manifest declaring none of the `Base`, `Dimension \| Exponent` and `Dimension \| Target \| Scale \| Offset` locators, an artifact carrying each table yields `semantic.feature-not-extractable` at the declaring line with `reason` `dimension`, `terms` or `unit`, `sourceSpan` and `section`, and no entry; an artifact declaring none of the three tables yields no `dimension`, `terms` or `unit` member and a record byte-identical to the FR-072 record. | Test |
| FR-075-AC-20 | Every record of AC-16..AC-19 validates against `semantic-v1.schema.json`, which lists `dimension`, `terms` and `unit` with `additionalProperties: false`; the Python binding and the Rust library return equal JSON values for each case; extraction of the same input twice is byte-identical. | Test |
| FR-075-AC-21 | Two `Base` tables, two `Dimension \| Exponent` tables, and two `Dimension \| Target \| Scale \| Offset` tables in one artifact each yield `semantic.duplicate-section` at the second table's header line; an artifact declaring a `Base` table and then a unit table yields `semantic.duplicate-section` at the unit table's header line and extracts `model.dimension` only, and a unit table then a `Base` table yields it at the `Base` table's header line and extracts `model.unit` only; a `Base` table with a `terms` table is accepted and extracts both. | Test |

## Dependencies

- **Upstream**: [FR-069](./FR-069-semantic-module-contract-at-load.md), [FR-070](./FR-070-typed-properties-extraction.md), [FR-071](./FR-071-clause-and-operation-extraction.md), [FR-072](./FR-072-semantic-extraction-surface.md)
- **Downstream**: `agent-ix/filament-core-data#148` (domain package lift); `agent-ix/spec-objects-business` (manifest `mappings` and `table_row` locators); `agent-ix/spec-objects-foundation` (`dimension` and `unit` object types); `agent-ix/filament-core-data` FR-146 "Spec-bundle extraction"
