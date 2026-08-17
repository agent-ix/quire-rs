---
id: FR-053
title: "Obligation Record"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-050"
    type: "extends"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-052"
    type: "references"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-014"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-017"
    type: "implements"
---
# FR-053: Obligation Record

## Description

`quire-rs` SHALL derive an **obligation record** for every normative statement a
module declares as one, and SHALL surface those records on the existing JSON
interfaces. An obligation record carries the statement's id, a deterministic
content hash of the statement, the declared verification method, any declared
parameters, and the declared criticality.

ADR 0011 names the obligation record the contract between **quire**, which
states obligations, and **quoin**, which audits their discharge. Neither side
has such a record today. `quire properties --json` carries property *shapes*;
[FR-050](./FR-050-declarative-coverage-computation.md) coverage carries the
AC→TC *rollup*. Neither carries a content hash, so nothing downstream can detect
a **suspect link** — evidence bound to a statement that has since been reworded
— and neither carries a verification method, so nothing can enforce **method
conformance**, where a requirement whose declared method is analysis is
discharged by a unit test.

An obligation is **derived, never authored**. There is no obligation document
and no `quire obligations` command. Choosing an acceptance criterion's
verification method *is* minting the obligation; the engine reads what the
author already wrote.

### Obligation sources are module data

The engine SHALL NOT know that `Verification` is a column, that `Test` is a
method, or that an acceptance criterion is the thing being obliged. A module
declares its obligation sources under the existing `traceability:` model:

```yaml
traceability:
  obligations:
  - name: acceptance-criterion
    target: acceptance-criterion       # reuses a declared trace_target
    statement_column: Criteria
    method_column: Verification
    criticality_column: Priority       # optional
  - name: nfr-metric
    archetype: NFR
    section: Measurement and Evaluation
    id_format: "{document}-M-{row}"
    statement_column: Metric
    method_column: Method
    parameters:
      target: Target
      threshold: Threshold
```

Each entry SHALL resolve its minting documents in exactly one of two ways, and
declaring both or neither SHALL be a load-time error:

- **`target:`** — names an existing
  [FR-050](./FR-050-declarative-coverage-computation.md) `trace_target`. The
  archetype, section and id column come from it, so an acceptance criterion is
  not declared twice and the obligation id is by construction the same id the
  coverage rollup and every trace tag already key on.
- **`archetype:` + `section:` + `id_format:`** — for a table whose rows mint no
  id of their own. The NFR `Measurement and Evaluation` table is the motivating
  case: 19 of 19 NFRs in this repository carry one, every row is a quantified
  obligation (metric, threshold, method), and no row has an `ID` column. The
  `id_format` template SHALL support `{document}` (the owning document's id) and
  `{row}` (the 1-based row ordinal), so `NFR-006-M-2` is derived rather than
  authored.

### The second source is not an afterthought

An NFR measurement row states one number three times over: the spec threshold,
the benchmark assertion, and the evidence gate are the same quantity. Modelling
it as an obligation is what lets a performance budget be discharged, ratcheted
and reported by the same machinery as a functional criterion, rather than by a
parallel mechanism that drifts. `parameters:` exists for exactly this — a map of
result key to source column, so `{target: "< 5ms", threshold: "< 8ms"}` travels
with the obligation instead of being re-parsed downstream. It is equally the
carrier for a t-way strength or a mutation-score floor a later method needs.

### Content hash

The hash SHALL be a SHA-256 over the **normalized** statement, rendered
lowercase hex, matching the NUL-separated `stable_id` convention already used
for FR-045 record ids and FR-051 symbol identities.

Normalization SHALL be exactly: Unicode NFC, then trim, then collapse every run
of whitespace to a single space. Nothing else.

> **This deliberately does NOT reuse the CR-017 code-span mask.**
> agent-ix/quire-rs#82 proposed it — "reuse CR-017's masking rules so a
> formatting edit doesn't churn hashes" — and it is the wrong tool, because
> CR-017 is not a formatting normalizer. `mask_code_spans` replaces a code
> span's *contents* with `x` so that a **quoted** keyword is read as a mention
> rather than a use. Applied to a hash it collapses statements that differ in
> the only place that matters:
>
> | Statement | CR-017 masked |
> |---|---|
> | ``The parser SHALL reject a `foo` token.`` | ``The parser SHALL reject a `xxx` token.`` |
> | ``The parser SHALL reject a `bar` token.`` | ``The parser SHALL reject a `xxx` token.`` |
>
> Two different obligations, one hash, and a suspect-link detector that stays
> silent through the rename it exists to catch. A backticked identifier is
> usually the *most* load-bearing token in a criterion. Whitespace normalization
> gives the property the ticket actually wanted — a re-wrap or a trailing space
> does not churn the hash — without ever equating two statements a reader would
> read differently.

### Surface

Records SHALL travel on the existing interfaces, adding no command:

- `classify_document_criteria` records gain an `obligation` field, so
  `quire properties --json` carries the obligation beside the property shape
  that criterion already has. A criterion its module declares no obligation
  source for carries `obligation: null` — classification is unchanged either
  way.

  This nested form carries `{source, statement_hash, method, parameters,
  criticality}` and **not** `id`, `statement` or `document`. Those three are
  already on the record or its enclosing object — `row_id`, `statement`, and
  `documents[].document` — and repeating them would put the same fact in two
  places on one payload, which is the shape that eventually disagrees with
  itself. It is also what keeps the classification API text-only: a criterion's
  hash, method and criticality are all readable from the criterion's own row,
  where a document path is not.
- The [FR-050](./FR-050-declarative-coverage-computation.md) coverage report
  gains an `obligations` list — ordered by source **declaration** order, then
  document path, then row ordinal (AC-9) — so the rollup and the obligations are
  one read rather than two. The list is skipped when empty, so a module
  declaring no sources emits a byte-identical payload (AC-11).

  > **CR-063** (2026-08-17) corrected this paragraph. It said "ordered by id",
  > which no code ever did and which contradicts AC-9 in the same document; and
  > AC-11 said the empty list is "present rather than absent", which contradicts
  > its own test, the `skip_serializing_if` attribute and the published schema.
  > Three statements of one fact, two of them wrong, is what a single source
  > exists to prevent.

Neither payload SHALL gain a version key
([FR-008](./FR-008-cli-boundary.md)-AC-5 stands); contract versioning lives in
the published schema artifacts.

## Inputs

- A loaded `Spec` corpus and a `Registry` whose merged traceability model
  declares one or more `obligations:` entries.
- For a `target:` entry, the resolved `TraceTarget` it names.

## Outputs

- `Obligation { source, id, document, statement, statement_hash, method,
  parameters, criticality }` — one per minting row, on
  `CoverageReport::obligations`.
- `CriterionObligation { source, statement_hash, method, parameters,
  criticality }` on `AcClassification::obligation` — the same facts minus the
  three the enclosing record already carries.

## Behavior

For each declared obligation source, in declaration order, the engine SHALL walk
the minting documents the source resolves to, read each row of the declared
section's table, and emit one record per row:

1. `id` — from the target's `id_column`, or rendered from `id_format`.
2. `statement` — the `statement_column` cell, verbatim and untruncated.
3. `statement_hash` — SHA-256 of the normalized statement.
4. `method` — the `method_column` cell with any trailing parenthetical
   annotation removed, so a `Verification` cell reading `Test (TC-707)` yields
   method `Test` while [FR-049](./FR-049-verification-reference-integrity.md)
   continues to read `TC-707` from the same cell. The engine SHALL NOT validate
   the method against any vocabulary here — that is the catalog's job
   ([FR-054](./FR-054-verification-method-catalog.md)) and the auditor's.
5. `parameters` — a `BTreeMap` of declared key to cell value; absent keys are
   omitted, never defaulted.
6. `criticality` — the `criticality_column` cell, or absent when the module
   declares no such column. Criticality is genuinely optional today: the ISO FR
   acceptance-criteria contract is `ID | Criteria | Verification` and carries no
   priority column, so declaring one is a module's choice rather than a
   precondition for obligations to exist at all.

Ordering SHALL be by `(source declaration order, document id, row ordinal)` and
every map SHALL be a `BTreeMap`, so two runs over identical inputs serialize
byte-identically ([NFR-006](../non-functional/NFR-006-determinism.md)).

A row whose `statement_column` is empty or whose id cannot be rendered SHALL be
skipped with a diagnostic naming the document and row, never emitted as a record
with an empty statement — an obligation nothing can state is not an obligation.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-053-CON-1 | The engine SHALL NOT name a verification method, a column name, or an archetype anywhere on this path. Every one is module data, so an external user's catalog and vocabulary drive their obligations exactly as this ecosystem's drive ours. | Architecture | Inspection |
| FR-053-CON-2 | An obligation record SHALL NOT carry a verdict, a threshold comparison, an evidence reference or a freshness judgement. The engine states obligations; reading evidence and judging discharge is quoin's Auditor role (ADR 0011 invariant 1). | Architecture | Inspection |
| FR-053-CON-3 | The content hash SHALL be computed from the normalized statement alone — never from the id, the document path, the line number or the method. A statement that moves file or renumbers has not changed, and a suspect-link detector that fires on a move is one that gets switched off. | Architecture | Test |
| FR-053-CON-4 | Obligation derivation SHALL NOT construct a `GrammarFinding` or carry a [FR-048](./FR-048-per-check-grammar-severity.md) `grammar_severity` key, so no obligation can fail a build. A missing method is a report, not an error, until a consuming workflow decides otherwise. | Architecture | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-053-AC-1 | A module declaring an obligation source with `target:` yields one record per row of the named trace target's table, each carrying the same id the coverage rollup mints for that row. | Test (TC-831) |
| FR-053-AC-2 | A module declaring an obligation source with `archetype:`, `section:` and `id_format:` yields one record per row of a table whose rows carry no id column, with ids rendered from `{document}` and the 1-based `{row}`. | Test (TC-832) |
| FR-053-AC-3 | An obligation source declaring both `target:` and `archetype:`, or neither, is rejected at manifest parse: the module contributes nothing, the load failure names the offending source, and no obligation is derived from the unexecutable declaration. | Test (TC-833) |
| FR-053-AC-4 | Two statements differing only in leading, trailing or internal whitespace, or in Unicode normalization form, produce the same `statement_hash`; two statements differing in any word — including a word inside an inline code span — produce different hashes. | Test (TC-834, TC-871) |
| FR-053-AC-5 | A `method_column` cell reading `Test (TC-707)` yields method `Test`, and the same cell continues to yield the reference `TC-707` to FR-049's integrity check, so the two readings of one cell do not interfere. | Test (TC-835) |
| FR-053-AC-6 | A source declaring `parameters:` yields a record whose parameter map carries one entry per declared key that has a non-empty cell, with absent keys omitted rather than present-and-empty. | Test (TC-836) |
| FR-053-AC-7 | A source declaring no `criticality_column` yields records whose criticality is absent, and the records are otherwise identical to those from the same source with a criticality column declared and empty. | Test (TC-837) |
| FR-053-AC-8 | A row whose statement cell is empty is skipped with a diagnostic naming the document and the row ordinal, and contributes no record. The diagnostic appears in the coverage report a consumer reads, not only in the derivation helper's return value. | Test (TC-838, TC-870) |
| FR-053-AC-9 | Two runs of obligation derivation over an identical corpus serialize byte-identically, and the record order is by source **declaration** order (never source name), then scope-relative document path, then row ordinal. | Test (TC-839, TC-872) |
| FR-053-AC-10 | A criterion whose module declares an obligation source over its archetype carries a populated `obligation` on its property classification record, matched to the criterion by row id; a criterion whose module declares none carries `obligation: null` and is otherwise field-for-field unchanged. | Test (TC-840) |
| FR-053-AC-13 | The obligation nested on a classification record carries no `id`, `statement` or `document` key, because the record and its enclosing object already carry all three. | Test (TC-843) |
| FR-053-AC-11 | A coverage report over a corpus with declared obligation sources carries one `obligations` entry per minting row, and a corpus with no declared sources carries an **absent** `obligations` key — the empty list serializes away — so FR-050-AC-7 byte-identity holds for every module that has not adopted them. | Test (TC-841) |
| FR-053-AC-12 | The statement hash of a criterion is unchanged when that criterion moves to a different file, changes line number, or has its id renumbered, and changes when a single word of the statement changes. | Test (TC-842) |
| FR-053-AC-14 | An `exclude:` glob on an obligation source removes the matching document's rows from **both** obligation surfaces — the coverage rollup and the `obligation` nested on a property classification record — so the two never disagree about whether a row states an obligation. Content with no path (stdin) matches no glob. | Test (TC-873) |

## Dependencies

- **Upstream**: [FR-050](./FR-050-declarative-coverage-computation.md) (the traceability model this extends, and the trace targets `target:` names), [FR-052](./FR-052-acceptance-criteria-property-classification.md) (the classification record obligations ride on), [FR-014](./FR-014-module-activation.md) (manifest loading), [FR-010](./FR-010-query-api.md) (table extraction)
- **Downstream**: [FR-054](./FR-054-verification-method-catalog.md) (the catalog the method is conformant against); `spec-artifacts-process` declares the ecosystem's obligation sources and the evidence-layer archetypes (agent-ix/spec-artifacts-process#34); quoin's evidence store keys bindings on `statement_hash` (agent-ix/quoin#79) and its auditor compares them (agent-ix/quoin#80)
