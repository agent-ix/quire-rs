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

An acceptance criterion is a **verification statement**, not an obligation: the
requirement already stated the obligation, and the criterion states what is
observed under what input. The **assertion** is therefore the single canonical
acceptance-criteria shape — it carries the test oracle directly, whereas an
obligation restates the requirement one level down and a Given/When/Then cell is
a second rendering of the same assertion (CR-013).

The `ac` grammar SHALL classify each statement into exactly one shape. The
classification is **structural**, used to locate the outcome clause the checks
below read; only the `assertion` shape is canonical:

- `assertion` — the statement asserts an outcome directly, e.g. "A finding whose
  key is absent from the merged map defaults to warning". Its outcome clause is
  the whole statement. **Canonical.**
- `obligation` — the statement matches an EARS pattern per the FR-042
  classifier, i.e. it states an obligation rather than an observation. Its
  outcome clause is the response clause after the modal verb.
- `given-when-then` — the statement is structured as Given/When/Then clauses (a
  leading `Given`/`When` clause and a `Then`/result clause, in prose or bullet
  form). Its outcome clause is the `Then` clause, so a GWT cell's other checks
  still run.
- `unstructured` — none of the above: the statement carries no modal, no
  Given/When/Then structure, and no observable signal.

For each statement, the `ac` grammar SHALL emit a finding when the statement
violates a check:

1. **unclassifiable** — the statement is `unstructured`: it carries no modal, no
   Given/When/Then structure, and no observable signal (no concrete-object
   signal, no lexicon term, no observable-result verb), so it names neither an
   input nor an outcome and there is nothing to test with (e.g. `It all works
   end to end`).
2. **non-singular** — the statement bundles more than one independent
   obligation: more than one `shall`, or more than one `Then` clause. A single
   criterion pairing one positive and one negative case of the same behavior
   (the `X yields a finding; Y yields none` idiom) SHALL count as one
   obligation.
3. **vague-response** — the statement's outcome clause (the whole statement for
   an `assertion`, the response clause of an `obligation`, or the `Then` clause
   of a `given-when-then` statement) uses a vague verb per the FR-042
   object-aware machinery. The check SHALL reuse the merged module lexicon
   ([FR-043](./FR-043-module-concrete-lexicon.md)) and project glossary
   ([FR-044](./FR-044-project-glossary-lexicon.md)) exactly as the EARS
   `vague-response` check does — one vague-verb implementation, two grammars.
4. **no-observable-outcome** — the statement's outcome clause names no
   externally checkable result. An outcome is observable when the clause
   carries a concrete-object signal per FR-042 (a backticked identifier, a
   numeric bound, or a lexicon term) or an **observable-result verb**. The
   observable-verb vocabulary is module data per ADR 0009 (concrete
   vocabulary is module data): the engine SHALL ship a built-in default set
   (`returns`, `emits`, `yields`, `reports`, `records`, `rejects`, `fails`,
   `exits`, `persists`, `prints`, `contains`, `equals`, `matches`, and their
   inflections), and a module MAY extend or override it via an
   `observable_verbs` registry in its `manifest.yaml`, merged across modules
   first-wins with the built-in defaults at lowest precedence — the same
   merge pattern as the FR-043 `lexicon`. A clause with none of these
   signals (e.g. `The import works correctly`) SHALL be flagged.
5. **non-canonical-shape** — the statement is `obligation`-shaped or
   `given-when-then`-shaped. The finding steers the author toward the canonical
   assertion shape, in the same spirit as the EARS `non-canonical-trigger`
   check; classification still succeeds, so the cell's other checks still run on
   its outcome clause. An `assertion` cell yields none.

> **CR-013 note:** This replaces the original FR-047 decision that EARS is the
> canonical acceptance-criteria shape (and that GWT is the only non-canonical
> one). The Gate G1 baseline over this repo's own spec produced 340 `ac`
> findings across 44 FR documents, **322 of them `unclassifiable`** — every one
> a correct report that a declarative assertion is not an obligation. Only 18
> were substantive. Classifying the corpus by quantifier instead of style showed
> 50.8% of its 327 acceptance criteria are already property-shaped (42.5%
> universally quantified, 8.3% metamorphic). An AC's testability depends on
> whether it names an input and an observable outcome, not on which prose style
> it wears, and the assertion shape supplies the test oracle directly. The
> canonical shape is therefore the assertion; `unclassifiable` now means
> "structureless", and `non-canonical-shape` fires on obligations as well as
> GWT. Roughly 5 cells in this corpus are affected. Shape conformance is not
> made configurable: FR-048's per-check severity map (`ac:non-canonical-shape:
> off`) is the opt-out, as for every other check.

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
| FR-047-CON-2 | The canonical shape SHALL NOT be made configurable per module: a `preferred_shape`-style option would reintroduce the plurality this FR removes. Suppression uses the FR-048 per-check severity map, the same mechanism every other check uses (CR-013) | Architecture | Inspection |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-047-AC-1 | A `Criteria` cell asserting an outcome is classified `assertion` (canonical), an obligation-shaped cell `obligation`, a Given/When/Then cell `given-when-then`, and a cell with no modal, no Given/When/Then structure, and no observable signal is classified `unstructured` and yields one `unclassifiable` finding. | Test (TC-707) |
| FR-047-AC-2 | A non-empty `Criteria` cell with no modal verb is still segmented and checked; an empty cell yields no statement. | Test (TC-708) |
| FR-047-AC-3 | A cell with two `shall` obligations or two `Then` clauses yields exactly one `non-singular` finding; the positive/negative pair idiom (`X yields a finding; Y yields none`) yields none. | Test (TC-709) |
| FR-047-AC-4 | A cell whose outcome clause uses a vague verb over an abstract object yields a `vague-response` finding; the same cell with the object present in the merged lexicon yields none. | Test (TC-710) |
| FR-047-AC-5 | A cell whose outcome clause carries neither a concrete-object signal nor an observable-result verb yields a `no-observable-outcome` finding; a cell naming a returned value, emitted record, or exit code yields none. | Test (TC-711) |
| FR-047-AC-6 | The `ac` grammar runs only on its bindings: the FR `Acceptance Criteria` `Criteria` column and `### <doc-id>-AC-N` supplement sections; an FR `Constraints` cell and an NFR `Statement` receive EARS findings only. | Test (TC-712) |
| FR-047-AC-7 | An `ac` finding carries `grammar: "ac"`, a stable check id, the statement excerpt, a 1-based line number, the classified shape, and a severity, and routes into `ValidationResult` per its severity. | Test (TC-713) |
| FR-047-AC-8 | `quire validate --summary` histograms findings by the generic `[<grammar>:<check>]` prefix: a corpus emitting both `[ears:*]` and `[ac:*]` findings shows both in the summary. | Test (TC-714) |
| FR-047-AC-9 | The `ac` grammar entry point is exposed through the existing grammar PyO3 surface and returns the same findings as the in-process Rust call for a fixture document. | Test (TC-715) |
| FR-047-AC-10 | An `obligation`-shaped cell and a `given-when-then`-shaped cell each yield one `non-canonical-shape` finding while still classifying as that shape (their other checks run on their outcome clause); an `assertion` cell yields none. | Test (TC-751) |
| FR-047-AC-11 | Fenced code blocks and blockquotes inside a `### <doc-id>-AC-N` supplement section are skipped: statements inside them are not segmented and yield no `ac` findings, while the surrounding supplement prose is still checked. | Test (TC-754) |
| FR-047-AC-12 | The observable-verb vocabulary is module data: a module's `observable_verbs` registry merges first-wins over the built-in defaults (a module-added verb suppresses `no-observable-outcome`), and with no module declaration the built-in default set applies unchanged. | Test (TC-757) |

## Dependencies

- **Upstream**: [FR-042](./FR-042-requirement-grammar-check.md) (the grammar framework and its section/table binding), [FR-043](./FR-043-module-concrete-lexicon.md) / [FR-044](./FR-044-project-glossary-lexicon.md) (the lexicon consumed by `vague-response`), [FR-010](./FR-010-query-api.md) (table extraction)
- **Downstream**: [FR-048](./FR-048-per-check-grammar-severity.md) (per-check severity promotion), the authoring and review workflows consume `ac` findings; the AC quality gate feeds the coverage rollup ([FR-050](./FR-050-declarative-coverage-computation.md))
