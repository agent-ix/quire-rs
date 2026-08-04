---
id: FR-047
title: "Acceptance-Criteria Grammar"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-042"
    type: "extends"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-043"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-044"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-014"
    type: "implements"
---
# FR-047: Acceptance-Criteria Grammar

## Description

The `iso-spec-core` grammar bundle SHALL include an **acceptance-criteria
grammar** (`ac`) registered on the [FR-042](./FR-042-requirement-grammar-check.md)
framework alongside EARS. The `ac` grammar SHALL bind to the FR
`Acceptance Criteria` section with the `Criteria` table column (the existing
table-column binding used for FR `Constraints`), and SHALL additionally bind to
AC supplement subsections (a `### <doc-id>-AC-N` heading whose body supplements
its table row).

The `ac` grammar SHALL treat every non-empty `Criteria` cell as one statement,
unlike the EARS segmenter's modal-verb filter: an acceptance criterion with no
`shall` is still a criterion and is still checked. The `ac` grammar SHALL skip
fenced code blocks and blockquotes inside supplement sections, per the FR-042
skip rules.

The `ac` grammar SHALL classify each statement into exactly one shape:

- `ears` — the statement matches an EARS pattern per the FR-042 classifier.
  EARS is the **canonical** acceptance-criteria shape;
- `given-when-then` — the statement is structured as Given/When/Then clauses
  (a leading `Given`/`When` clause and a `Then`/result clause, in prose or
  bullet form). The shape is recognized — not `unclassifiable` — so property
  extraction can consume legacy GWT cells while authoring converges on EARS;
- `unclassifiable` — neither shape matches.

For each statement, the `ac` grammar SHALL emit a finding when the statement
violates a check:

1. **unclassifiable** — the statement matches neither shape.
2. **non-singular** — the statement bundles more than one independent
   obligation: more than one `shall`, or more than one `Then` clause. A single
   criterion pairing one positive and one negative case of the same behavior
   (the `X yields a finding; Y yields none` idiom) SHALL count as one
   obligation.
3. **vague-response** — the statement's outcome clause (the response clause of
   an `ears`-shaped statement, or the `Then` clause of a
   `given-when-then`-shaped statement) uses a vague verb per the FR-042
   object-aware machinery. The check SHALL reuse the merged module lexicon
   ([FR-043](./FR-043-module-concrete-lexicon.md)) and project glossary
   ([FR-044](./FR-044-project-glossary-lexicon.md)) exactly as the EARS
   `vague-response` check does — one vague-verb implementation, two grammars.
4. **no-observable-outcome** — the statement's outcome clause names no
   externally checkable result. An outcome is observable when the clause
   carries a concrete-object signal per FR-042 (a backticked identifier, a
   numeric bound, or a lexicon term) or an observable-result verb from a
   bounded engine list (`returns`, `emits`, `yields`, `reports`, `records`,
   `rejects`, `fails`, `exits`, `persists`, `prints`, `contains`, `equals`,
   `matches`, and their inflections). A clause with none of these signals
   (e.g. `The import works correctly`) SHALL be flagged.
5. **non-canonical-shape** — the statement is `given-when-then`-shaped. The
   finding steers the author toward the canonical EARS shape, in the same
   spirit as the EARS `non-canonical-trigger` check; classification still
   succeeds, so the cell's other checks still run on its `Then` clause.

Each `ac` finding SHALL carry `grammar: "ac"`. The framework SHALL route `ac`
findings into `ValidationResult` by severity per FR-042. The rollout default
is explicit: every `ac` check ships advisory (`warning`) at most, and each
check is individually suppressible (`off`) or promotable per
[FR-048](./FR-048-per-check-grammar-severity.md).

`quire-cli` `validate --summary` SHALL surface findings for **any** grammar in
the active bundle. The summary parser SHALL group findings by the generic
prefix `[<grammar>:<check>]` — replacing the hardcoded `[ears:` prefix — so
the histogram covers every grammar and check.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-047-CON-1 | `ac` checks SHALL NOT ship promoted to `error` by default: promotion waits for a corpus baseline sweep and an explicit user gate, mirroring the FR-042 EARS rollout precedent | Operational | Inspection |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-047-AC-1 | An EARS-shaped `Criteria` cell is classified `ears`, a Given/When/Then cell is classified `given-when-then`, and a cell matching neither yields one `unclassifiable` finding. | Test (TC-707) |
| FR-047-AC-2 | A non-empty `Criteria` cell with no modal verb is still segmented and checked; an empty cell yields no statement. | Test (TC-708) |
| FR-047-AC-3 | A cell with two `shall` obligations or two `Then` clauses yields exactly one `non-singular` finding; the positive/negative pair idiom (`X yields a finding; Y yields none`) yields none. | Test (TC-709) |
| FR-047-AC-4 | A cell whose outcome clause uses a vague verb over an abstract object yields a `vague-response` finding; the same cell with the object present in the merged lexicon yields none. | Test (TC-710) |
| FR-047-AC-5 | A cell whose outcome clause carries neither a concrete-object signal nor an observable-result verb yields a `no-observable-outcome` finding; a cell naming a returned value, emitted record, or exit code yields none. | Test (TC-711) |
| FR-047-AC-6 | The `ac` grammar runs only on its bindings: the FR `Acceptance Criteria` `Criteria` column and `### <doc-id>-AC-N` supplement sections; an FR `Constraints` cell and an NFR `Statement` receive EARS findings only. | Test (TC-712) |
| FR-047-AC-7 | An `ac` finding carries `grammar: "ac"`, a stable check id, the statement excerpt, a 1-based line number, the classified shape, and a severity, and routes into `ValidationResult` per its severity. | Test (TC-713) |
| FR-047-AC-8 | `quire validate --summary` histograms findings by the generic `[<grammar>:<check>]` prefix: a corpus emitting both `[ears:*]` and `[ac:*]` findings shows both in the summary. | Test (TC-714) |
| FR-047-AC-9 | The `ac` grammar entry point is exposed through the existing grammar PyO3 surface and returns the same findings as the in-process Rust call for a fixture document. | Test (TC-715) |
| FR-047-AC-10 | A `given-when-then`-shaped cell yields one `non-canonical-shape` finding while still classifying `given-when-then` (its other checks run on the `Then` clause); an EARS-shaped cell yields none. | Test (TC-751) |

## Dependencies

- **Upstream**: [FR-042](./FR-042-requirement-grammar-check.md) (the grammar framework and its section/table binding), [FR-043](./FR-043-module-concrete-lexicon.md) / [FR-044](./FR-044-project-glossary-lexicon.md) (the lexicon consumed by `vague-response`), [FR-010](./FR-010-query-api.md) (table extraction)
- **Downstream**: [FR-048](./FR-048-per-check-grammar-severity.md) (per-check severity promotion), the authoring and review workflows consume `ac` findings; the AC quality gate feeds the coverage rollup ([FR-050](./FR-050-declarative-coverage-computation.md))
