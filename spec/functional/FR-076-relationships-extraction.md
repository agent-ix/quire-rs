---
id: FR-076
title: "Relationships extraction to RelationDecl[]"
type: FR
verification_method: test
evidence:
  - kind: test_case
    ref: tests/it/semantic_relations.rs
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-019"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-040"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-041"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-070"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-072"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-075"
    type: "requires"
    cardinality: "1:1"
---
# FR-076: Relationships extraction to RelationDecl[]

## Description

When the engine extracts an object artifact whose module carries a
`semantic` block that names the mapping token `relationships`,
the engine SHALL extract the `## Relationships` section, authored as one table with the
columns `Name`, `Verb`, `Target`, and `Multiplicity`, to a semantic-core
`RelationDecl[]` with a name and a source span per row, under the record
keys `relations` and `relationSources`, with `availability.relations`.

The grammar follows `agent-ix/quoin` FR-104 (quoin PRs #553, #555 and #558); the
relocated fixtures `relationships.md`, `relationships.expected.json`, and
`relationships-cases.json` (CR-182), whose bytes are unchanged from
`agent-ix/quoin` FR-104's own fixtures with ownership transferred to quire-rs's
`quire-fixtures` crate, are the oracle. A relationship's category and
composition come only from the FR-040 `edge_types` registry entry of its
verb ([FR-040](./FR-040-object-edge-vocabulary.md)); an inverse
label is refused in favour of its forward verb
([FR-041](./FR-041-authorable-inverse-edges.md)).

## Inputs

- The document and the `SemanticContext` of
  [FR-070](./FR-070-typed-properties-extraction.md), with the module
  `semantic.mappings` and `semantic.imports`
  ([FR-069](./FR-069-semantic-module-contract-at-load.md)).
- The optional `RelationVocabulary` on the context: the extracting object
  type's name, the merged `edge_types` registry (verb →
  `{ category, inverse? }`) with its FR-041 inverse index, each object
  type's `roles`, and each object type's `allowed_links` (verb → target
  tokens). `validate_document` builds it from the loaded `Registry`: its
  merged `edge_types` and inverse index, and the `roles` and
  `allowed_links` of every active archetype, under the validated
  archetype's name. The Filament surface supplies none until the registry
  snapshot carries `edge_types` and `roles`
  (`agent-ix/filament-core-service#32`). The Python entry supplies one
  only when the request carries `relationVocabulary`.
- The bundle package `<org>/<repo>` targets qualify under: the
  `BundleIndex` `package`, else the package of an explicit
  `ix://<org>/<repo>/…` source identity, never the semantic module's
  package. `validate_document` takes it from its caller
  (`validate_document_in_bundle`, or the Python binding's `bundle_package`);
  its other entry points, and bundle
  validation, which knows no repository identity, supply none. Filament
  supplies its `ix://<org>/<repo>/spec` source identity.
- The `BundleIndex` `artifacts`: every artifact `id` of the
  bundle with its frontmatter `object` type. Absent or empty `artifacts`
  is the no-bundle-index state. The artifact's own frontmatter `id` and
  `object` always count as a bundle artifact.
- The artifact's frontmatter `relationships` entries.
- The extracting object type's *typed relation keys* on the
  `RelationVocabulary`: `typed_keys` (verb → record key, the key one of
  `emits` or `persists`) and `relations_allowed` (whether the type's record
  admits a `relations` key). A type with no `typed_keys` behaves exactly as
  before this section existed. The surfaces that build the vocabulary supply
  both from the object type's record shape.

## Outputs

- `relations`: `RelationDecl[]` in row order, each
  `{ verb, category, composite, target, multiplicity }` validating against
  the vendored semantic-core `RelationDecl.json`.
- `relationSources`: one `{ name, sourceSpan }` per `relations` element at
  the same index (for a type with `typed_keys`, one per table row in row
  order, so every typed row keeps its `Name` and line, and the i-th entry of a
  typed key corresponds to the i-th row of that key's verb); `name` is the row's `Name` cell and `sourceSpan` the
  row's line span, built as FR-075 builds spans. This is the semantic-core
  `0.2.0` carrier: when `agent-ix/filament-core-data#155` adds `name` and
  `sourceSpan` to `RelationDecl`, they move onto each element and
  `relationSources` is removed.
- `emits`, `persists`: record keys of `SemanticId[]`, in row order, each
  present exactly when the object type's `typed_keys` names it and
  `availability.relations` is `available`. A row whose verb is a key of
  `typed_keys` lowers into that key only: its element is the row's resolved
  identity (`ix://<package>/<id>`), and it contributes nothing to
  `relations`. When `relations_allowed` is `false`, no `relations` key is
  produced.
- `availability.relations`: `{ state, reason?, lossy }`.
- Diagnostics with `code`, `severity`, `line`, `section`, `reason`,
  `sourceSpan` of the declaring line, and a message naming the artifact
  path, the section, and the feature `relationships`.

## Behavior

Gating and table shape:

- The engine SHALL read `## Relationships` as a relationship declaration
  only when `semantic.mappings` names `relationships`.
- When `semantic.mappings` does not name `relationships`, the engine SHALL
  treat `## Relationships` as prose, with no diagnostic and no
  `availability.relations`, unless a table whose header starts with `Name`
  and uses only the columns `Name`, `Verb`, `Target`, `Multiplicity` sits in
  the preamble or under any `##` section.
- If such a table sits in an artifact whose `semantic.mappings` does not
  name `relationships`, then the engine SHALL emit the error
  `semantic.feature-not-extractable` at its header line with `reason`
  `relationships` and `section` `preamble` or the heading text.
- If such a table sits in the preamble or under any `##` section other than
  `## Relationships` of an artifact whose `semantic.mappings` names
  `relationships`, then the engine SHALL emit the error
  `semantic.feature-not-extractable` at its header line with `reason`
  `relationships` and `section` `preamble` or the heading text.
- The engine SHALL read only the first table of the first
  `## Relationships` section, and only when its header columns are exactly
  `Name`, `Verb`, `Target`, `Multiplicity`, in that order.
- If that first table has any other header, or the section holds a list or
  a fence before or after the table, then the engine SHALL emit
  `semantic.feature-not-extractable` at the block's first line with
  `reason` `relationships`.
- If the section holds a second table, then the engine SHALL emit
  `semantic.duplicate-section` at its header line with `reason`
  `second-table`. If the artifact holds a second `## Relationships`
  section, then the engine SHALL emit `semantic.duplicate-section` at its
  heading line with `reason` `second-section`.
- If the `## Relationships` section holds no table, list, or fence, then
  the engine SHALL emit the warning `semantic.relationships-no-block` at
  the heading line with `reason` `no-block`.

Rows, with backticks around a cell stripped:

- The engine SHALL accept a `Name` cell only when it is an `Identifier`.
- The engine SHALL set `RelationDecl.verb` to the `Verb` cell and
  `category` to the verb's registry `category`, and set `composite` to
  `true` exactly when the registry entry declares `composite: true`, and
  `false` otherwise; `composite` is always present.
- The engine SHALL map a bare `Target` naming an artifact `id` of the bundle
  to `ix://<package>/<id>`.
- The engine SHALL carry an `ix://<org>/<repo>/<id>` target verbatim when
  `<org>/<repo>` is the bundle's package and `<id>` a bundle artifact, or
  when `<org>/<repo>` is a package in `semantic.imports`.
- When the context has no bundle index, the engine SHALL lower a bare
  `Target` in the id alphabet (an ASCII letter or digit, then ASCII letters,
  digits, `.`, `_`, `~`, `:`, or `-`) to `ix://<package>/<id>`, and an
  own-package `ix://<org>/<repo>/<id>` verbatim, without the existence and
  `allowed_links` checks. A title inside the alphabet cannot be told apart
  from an id and lowers the same way; a title outside it is refused as
  `target-not-id`.
- When a row lowers with a target checked against no bundle index, the
  engine SHALL emit the advisory `semantic.unresolved-target` at the row
  with `reason` `no-bundle-index` and a message naming the target.
- A bundle target SHALL satisfy an `allowed_links` token of the verb: `*`,
  the target's object type, or a role of that object type. A target with no
  object type satisfies only `*`. An imported target is not checked.
- `Multiplicity` SHALL be present and map as the FR-070 multiplicity cell,
  with no upper bound for `*`.

Refusals, one diagnostic per row, first failing check only, in this order:

| Check | Code | `reason` |
|---|---|---|
| `Name` is not an `Identifier` | `semantic.invalid-model-cell` | `name-not-identifier` |
| `Name` repeats an earlier row's | `semantic.duplicate-model-entry` | `duplicate-name` |
| `Verb` is only a registry `inverse` label; the message names the forward verb and the target, and for the inverse of `specializes` the FR-075 `generalization` mapping | `semantic.invalid-model-cell` | `inverse-verb` |
| `Verb` is `specializes`; the message names the FR-075 `generalization` mapping | `semantic.invalid-model-cell` | `generalization` |
| `Verb` is neither a forward key nor an inverse label | `semantic.invalid-model-cell` | `unknown-verb` |
| `Verb` is not a key of the object type's `allowed_links` | `semantic.invalid-model-cell` | `verb-not-allowed` |
| the object type's `relations_allowed` is `false` and the `Verb` is no key of `typed_keys` | `semantic.invalid-model-cell` | `verb-not-typed-key` |
| `Target` is a title, an unknown id, an own-package identity naming no artifact, or an identity of a package neither the bundle's nor imported | `semantic.invalid-model-cell` | `target-not-id` |
| a bundle target satisfies no `allowed_links` token | `semantic.invalid-model-cell` | `target-not-allowed` |
| `Multiplicity` is empty or malformed | `semantic.invalid-model-cell` | `multiplicity` |
| frontmatter `relationships` declares the same verb and target | `semantic.duplicate-model-entry` | `declared-in-frontmatter` |

A frontmatter target compares by identity: a bare id maps to
`ix://<package>/<id>`, and an `ix://<org>/<repo>/…/<id>` path to
`ix://<org>/<repo>/<id>`; an entry without `type` is `references`
([FR-026](./FR-026-intra-spec-reference-resolution.md)).

Typed relation keys:

- Where the object type's `typed_keys` names the row's verb, the engine SHALL
  lower the row into the typed key and SHALL NOT add it to `relations`. Every
  other check of this section applies to the row first and unchanged (the
  refusals table, `allowed_links`, the bundle index states, the advisories),
  so a typed row is refused for exactly the reasons a `relations` row is.
- The row's `Target` SHALL resolve through the resolver of
  [FR-075](./FR-075-model-feature-extraction.md) "Declaration references"
  in its *id-only* form: the `Target` grammar of this section is unchanged,
  and a title is `target-not-id`, not a name match. A declaration therefore
  has the same identity as a model cell naming it.
- The `emits` and `persists` keys are derived from the `## Relationships` rows
  only. The engine SHALL emit the error `semantic.duplicate-authority`, at
  the first line of the document, with `reason` `typed-key-locator`, when
  the module declares a `table_row` locator whose match key is a key of
  `typed_keys` for the artifact's object type, and with `reason`
  `frontmatter-key` when the artifact's frontmatter carries that key. The
  engine SHALL NOT merge two declaration sites.
- When `relations_allowed` is `false`, a row whose verb is not a key of
  `typed_keys` is `semantic.invalid-model-cell` with `reason`
  `verb-not-typed-key` at the row, checked after `verb-not-allowed`.
- A header-only table on a type with `typed_keys` yields `available`, each
  typed key present and empty, no `relations` and no `relationSources`.

Availability:

- While `semantic.mappings` names `relationships`, the engine SHALL report
  `not_applicable` for an artifact with no `## Relationships` section or
  with a section that holds no block, unless a refusal elsewhere in the
  artifact carries an error.
- When the section's table was read and the context carries no
  `RelationVocabulary`, the engine SHALL check no row, emit one advisory
  `semantic.relationships-no-vocabulary` at the section heading with
  `reason` `no-relation-vocabulary`, and report `unavailable` with reason
  `no-relation-vocabulary`, unless a header, block, or section error
  already makes the feature unavailable, in which case that error alone is
  reported.
- When the section's table was read, the context carries a
  `RelationVocabulary`, and no bundle package is known, the engine SHALL
  check no row, emit one advisory `semantic.relationships-no-bundle-package`
  at the section heading with `reason` `no-bundle-package`, and report
  `unavailable` with reason `no-bundle-package`, unless a header, block, or
  section error already makes the feature unavailable, in which case that
  error alone is reported. This state extends FR-104 on a surface quoin
  covers since quoin PR #558 (cases `no-bundle-package` and
  `no-bundle-package-second-table`).
- A `no-bundle-index` advisory SHALL NOT make the feature lossy or
  unavailable.
- If any row carries an error, then the engine SHALL emit no
  `no-bundle-index` advisory for the rows that passed their checks
  (quoin PR #558, case `no-bundle-index-mixed-rows`).
- When the table has a header and no rows, the engine SHALL report
  `available`, not lossy, with empty `relations` and `relationSources`.
- If any header, block, section, or row of the feature carries an error,
  then the engine SHALL report `unavailable` with reason
  `entry-errors: lines <lines>` (ascending, deduplicated, joined with `, `)
  and carry no `relations` and no `relationSources`.
- `lossy` SHALL be `true` only under `compatibility_posture: declared-lossy`.

> **CR-435 note (2026-11-08, agent-ix/quire-rs#435 and #444, PLAT-55, PLAT-35):**
> the original text lowered every row into `relations`, so a type whose record
> forbids `relations` (`process`, `repository`) failed with
> `semantic.record-invalid`, and `emits` / `persists` had no extractor. Rows
> now lower into typed keys on the types that declare them, through the
> resolver shared with FR-075, and a second declaration site is refused.
> Affected criteria: FR-076-AC-8 and -AC-14 are unchanged for types without
> `typed_keys`; the new AC-16..20 and CON-3 cover the rest. No fallback to
> `relations` exists for a type that declares `typed_keys`.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-076-CON-1 | An artifact whose module does not name `relationships` and that holds no relationship-shaped table SHALL yield a record byte-identical to the FR-075 record: no `relations`, `relationSources`, or `availability.relations` key. | Compatibility | Test |
| FR-076-CON-2 | The engine SHALL NOT derive a relationship's category or composition from the verb's spelling, the target's name, or the object type's roles. | Correctness | Test |
| FR-076-CON-3 | A typed relation key SHALL hold only resolved `SemanticId`s of bundle declarations (or unchecked imported identities as above); no row text SHALL be carried in its place, and no key SHALL have a second declaration site. | Correctness | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-076-AC-1 | The relocated fixture `relationships.md` extracts to exactly the `relations`, `relationSources`, and `availability` of `relationships.expected.json` under its recorded `context`; each `relations` element validates against semantic-core `0.2.0` `RelationDecl.json`. | Test |
| FR-076-AC-2 | The `relationships-cases.json` cases `verb-unknown`, `verb-inverse-label`, `verb-not-allowed`, and `verb-specializes` yield their expected code, line, `section`, `reason`, and `messageContains` substrings. | Test |
| FR-076-AC-3 | The cases `target-title`, `target-undeclared-id`, `target-own-package-undeclared-id`, `target-qualified-not-imported`, `target-not-allowed-object-type`, `target-not-allowed-behavioral`, and `target-qualified-import` yield their expected diagnostics, `relations`, and `relationSources`. | Test |
| FR-076-AC-4 | The cases `multiplicity-malformed`, `multiplicity-inverted`, and `multiplicity-empty` yield `semantic.invalid-model-cell` with `reason` `multiplicity` at the row. | Test |
| FR-076-AC-5 | The cases `name-not-identifier`, `duplicate-name`, and `declared-in-frontmatter` yield their expected diagnostics. | Test |
| FR-076-AC-6 | The cases `column-missing`, `column-extra`, `bullet-list`, `second-table`, `second-section`, and `list-after-table` yield their expected diagnostics with `availability.relations` `unavailable`. | Test |
| FR-076-AC-7 | The cases `mapping-not-declared`, `subset-header-unowned-section`, `preamble-table`, `prose-without-mapping`, `token-table-other-section`, and `token-preamble-table` yield their expected diagnostics and availability. | Test |
| FR-076-AC-8 | The cases `good-and-bad-rows`, `two-error-lines`, `header-only`, `section-absent`, and `first-failing-check-only` yield their expected availability, and every case of `relationships-cases.json` emits exactly `exactDiagnostics` diagnostics, each with a `sourceSpan` of its line. | Test |
| FR-076-AC-9 | Every record of AC-1..AC-8 and AC-10..AC-12 validates against `semantic-v1.schema.json`; every existing semantic case record is unchanged; `declaration_record` carries `relations` when extracted; an artifact without the token or a relationship-shaped table carries no relations keys (CON-1); renaming a verb while keeping its registry entry changes only `verb` (CON-2). | Test |
| FR-076-AC-10 | The cases `no-relation-vocabulary` and `no-relation-vocabulary-second-table` yield their expected diagnostics and `availability.relations`, with no `relations`. | Test |
| FR-076-AC-11 | The cases `no-bundle-index`, `no-bundle-index-title-target`, `no-bundle-index-own-package-identity`, and `no-bundle-index-bad-multiplicity` yield their expected diagnostics, `relations`, `relationSources`, and `availability.relations`. | Test |
| FR-076-AC-12 | The case `prose-only-section` yields the `semantic.relationships-no-block` warning at the heading and `availability.relations` `not_applicable`. | Test |
| FR-076-AC-13 | `validate_document` over a loaded registry checks relationship rows against the registry's `edge_types`, `roles`, and `allowed_links`: a row whose own-id target satisfies a role lowers with no finding, and an unregistered verb, an inverse label, and a disallowed target are errors. Filament extraction and the Python entry without `relationVocabulary` report `no-relation-vocabulary`. | Test |
| FR-076-AC-14 | With no bundle index, rows `references FR-005 1..1` and `references FR-006 many` yield exactly one diagnostic, the `multiplicity` error at line 21, and `availability.relations` `unavailable` with reason `entry-errors: lines 21`. | Test |
| FR-076-AC-15 | Targets qualify under the bundle package, else the source identity's package: through `validate_document_in_bundle` an own-package `ix://` target lowers with the `no-bundle-index` advisory and a frontmatter duplicate is refused across the bare and `ix://` forms; with no bundle package, the Python entry and `validate_document_in_registry` report `no-bundle-package` with one heading advisory; the Python `validate_document` binding's optional `bundle_package` routes to `validate_document_in_bundle`. | Test |
| FR-076-AC-16 | Under `typed_keys` `{ emits → emits }` and `relations_allowed` false, a `process` artifact with the row `done \| emits \| event-001 \| 0..1` yields `emits` `["ix://<org>/<repo>/event-001"]`, no `relations`, a `relationSources` entry with `name` `done` and the row's span, and `availability.relations` `available`; a `repository` with `orders \| persists \| aggregate-root-001 \| 0..*` under `typed_keys` `{ persists → persists }` yields `persists` `["ix://<org>/<repo>/aggregate-root-001"]` the same way; an `aggregate_root` with two `emits` rows yields two elements in row order; each record validates against `semantic-v1.schema.json`. | Test |
| FR-076-AC-17 | On a type with `typed_keys`, the rows `x \| emits \| nope \| 1`, `x \| emits \| <title of an event> \| 1`, `x \| emits \| <a non-event with a disallowing allowed_links> \| 1`, `x \| triggers \| event-001 \| 1` (a verb outside `typed_keys` while `relations_allowed` is false), and a registry inverse label each yield the FR-076 refusal of the matching row check (`target-not-id`, `target-not-id`, `target-not-allowed`, `verb-not-typed-key`, `inverse-verb`), one diagnostic per row, and `availability.relations` `unavailable` with no `emits`, `persists`, `relations` or `relationSources`; the control row with a resolving event id yields none of them. | Test |
| FR-076-AC-18 | A header-only `## Relationships` table on a type with `typed_keys` yields `available`, `emits` `[]` (or `persists` `[]`), no `relations` and no `relationSources`; on a type with `relations_allowed` true and `typed_keys` `{ emits → emits }`, a table with one `emits` row and one `references` row yields `emits` of one element and `relations` of one element, the `references` row only. | Test |
| FR-076-AC-19 | A module declaring a `table_row` locator keyed `emits` next to the `relationships` token, on a type whose `typed_keys` names `emits`, yields `semantic.duplicate-authority` with `reason` `typed-key-locator` at line 1 and no merged `emits`; an artifact whose frontmatter carries `persists:` on a `repository` yields the same code with `reason` `frontmatter-key`; the control (neither site) yields no such diagnostic. | Test |
| FR-076-AC-20 | The model-table cell `EV-001` and the relationship row target `EV-001` (and the cell `OrderPlaced`, the title of the same event) resolve to the same `SemanticId` ([FR-075](./FR-075-model-feature-extraction.md)-AC-22); a row `x \| emits \| OrderPlaced \| 1` is still `target-not-id`; a type without `typed_keys` yields a record byte-identical to AC-1 (CON-1); the corpus fixtures carry, for each of `emits` on `aggregate_root`, `emits` on `process` and `persists`, a resolving case with `issue_ref` `PLAT-55`, an unresolved case and a control. | Test |

## Dependencies

- **Upstream**: [FR-040](./FR-040-object-edge-vocabulary.md), [FR-041](./FR-041-authorable-inverse-edges.md), [FR-069](./FR-069-semantic-module-contract-at-load.md), [FR-070](./FR-070-typed-properties-extraction.md), [FR-072](./FR-072-semantic-extraction-surface.md), [FR-075](./FR-075-model-feature-extraction.md); `agent-ix/quoin` FR-104 and its mapping fixtures
- **Downstream**: `agent-ix/spec-objects-business` (`typed_keys` source: TC-097 / TC-098 fixtures), `agent-ix/filament-core-data#155` (`RelationDecl` `name` and `sourceSpan`), `agent-ix/filament-core-data#156` (`RelationDecl` lowering from extraction), `agent-ix/filament-core-service#32` (`edge_types` and `roles` in the registry snapshot), `agent-ix/quire-cli#91`, `agent-ix/quoin#557`, and `agent-ix/quire-rs#437` (callers pass the bundle package to `validate_document_in_bundle`)
