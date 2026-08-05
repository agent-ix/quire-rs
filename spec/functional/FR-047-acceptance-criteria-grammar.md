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
framework alongside EARS. The `ac` grammar SHALL bind to the `Acceptance
Criteria` section's `Criteria` table column of **every requirement archetype
that carries one** — FR, NFR, US, StR, IT — rather than to `FR` alone: the
criteria of a user story or a quality attribute are verification statements of
the same kind, and an ecosystem survey found them authored in the same shape
(CR-014). It SHALL additionally bind to AC supplement subsections (a
`### <doc-id>-AC-N` heading whose body supplements its table row).

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
- `unstructured` — none of the above **and no predicate at all**: no modal or
  copula, no inflected or irregular verb form, no declared observable-result
  verb, and no concrete-object signal (CR-014).

For each statement, the `ac` grammar SHALL emit a finding when the statement
violates a check:

1. **unclassifiable** — the statement carries **no predicate at all**: no modal
   or copula, no inflected verb form, no irregular past form, no declared
   observable-result verb, and no concrete-object signal. A bare noun phrase
   (`Structural evaluation`, `Type Check`, `Criterion`) asserts nothing and
   cannot be tested. The check is deliberately *structural* — it does not ask
   whether the outcome is a good one, only whether the cell states an outcome
   (CR-014).
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
4. **vacuous-outcome** — the statement's outcome clause is headed by a
   **vacuous predicate** and carries nothing else to check: the engine SHALL
   ship a built-in vacuity set (`works`, `working`, `behaves`, `functions`,
   `work correctly`, `is correct`, `is successful`, `is fine`, `is ok`) that a
   module MAY extend via a `vacuous_predicates` registry in its
   `manifest.yaml`, merged first-wins with the built-in defaults at lowest
   precedence — the same pattern as the FR-043 `lexicon` and the
   `observable_verbs` registry. The finding SHALL be suppressed when the clause
   carries a concrete-object signal (a backticked identifier or a numeric
   bound), a lexicon term, or a declared **observable-result verb**: the engine
   SHALL ship a built-in observable-verb set and a module MAY extend it via an
   `observable_verbs` registry (ADR 0009), and those verbs are what tell a
   vacuous cell (`Navigation works`) from a substantive one (`Volumes are
   correctly mounted into the container`).

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

> **CR-014 note:** An ecosystem fit check (report:
> `~/dev/reports/2026-08-04-ac-grammar-fit.md`) ran this grammar through the
> PyO3 binding over 5,027 requirement documents in 199 repos — 11,919
> acceptance-criteria cells — and found two of its checks unusable as specified.
> `no-observable-outcome` fired on **51% of all cells at ~35% sampled
> precision**, and `unclassifiable` on **43.5% at ~12%**. The cause is
> structural, not a tuning gap: observability in acceptance criteria lives in an
> open-ended verb space (1,201 distinct stems in the corpus; the built-in 13
> covered 14.5%), and an allowlist cannot close an open set — declaring 73
> corpus-mined verbs cut findings 60% but left precision at ~30%/~12%, still
> flagging *"Semantic search ranks by relevance"* and *"Cache does not exceed
> max_size entries"*.
>
> The tests are therefore inverted. `vacuous-outcome` detects a **closed** set of
> vacuous predicates instead of requiring membership of an open one, and
> `unclassifiable` asks the structural question — is there a predicate at all —
> instead of asking whether a verb is on a list. Measured over the same corpus:
> **25 and 39 findings** respectively (0.21% and 0.33% of cells) against 11,269
> before, with no true positive lost in sampling. `non-canonical-shape`
> (2,047 findings, ~95% precision) and the low-volume checks are unchanged.
>
> The same check also settled the binding: `FR`-only reached 76.9% of AC-bearing
> documents, and sampled US and NFR criteria are the same shape as FR ones, so
> the grammar now binds to every requirement archetype carrying an
> `Acceptance Criteria` table. Bullet-form AC sections remain unsegmented by any
> grammar — recorded as future work, not addressed here.

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
| FR-047-AC-5 | A cell headed by a vacuous predicate with nothing else to check (`Navigation works`) yields a `vacuous-outcome` finding; the same predicate alongside a concrete-object signal, a lexicon term, or a declared observable-result verb (`Volumes are correctly mounted into the container`) yields none. | Test (TC-711) |
| FR-047-AC-6 | The `ac` grammar runs on the `Acceptance Criteria` `Criteria` column of every requirement archetype that carries one (FR, NFR, US, StR, IT) plus `### <doc-id>-AC-N` supplement sections; an FR `Constraints` cell and an NFR `Statement` receive EARS findings only. | Test (TC-712) |
| FR-047-AC-7 | An `ac` finding carries `grammar: "ac"`, a stable check id, the statement excerpt, a 1-based line number, the classified shape, and a severity, and routes into `ValidationResult` per its severity. | Test (TC-713) |
| FR-047-AC-8 | `quire validate --summary` histograms findings by the generic `[<grammar>:<check>]` prefix: a corpus emitting both `[ears:*]` and `[ac:*]` findings shows both in the summary. | Test (TC-714) |
| FR-047-AC-9 | The `ac` grammar entry point is exposed through the existing grammar PyO3 surface and returns the same findings as the in-process Rust call for a fixture document. | Test (TC-715) |
| FR-047-AC-10 | An `obligation`-shaped cell and a `given-when-then`-shaped cell each yield one `non-canonical-shape` finding while still classifying as that shape (their other checks run on their outcome clause); an `assertion` cell yields none. | Test (TC-751) |
| FR-047-AC-11 | Fenced code blocks and blockquotes inside a `### <doc-id>-AC-N` supplement section are skipped: statements inside them are not segmented and yield no `ac` findings, while the surrounding supplement prose is still checked. | Test (TC-754) |
| FR-047-AC-12 | Both vocabularies are module data: a module's `observable_verbs` registry merges first-wins over the built-in defaults (a module-added verb suppresses `vacuous-outcome` and gives the cell a predicate), a module's `vacuous_predicates` registry likewise extends the built-in vacuity set, and with no module declaration both built-in default sets apply unchanged. | Test (TC-757) |

## Dependencies

- **Upstream**: [FR-042](./FR-042-requirement-grammar-check.md) (the grammar framework and its section/table binding), [FR-043](./FR-043-module-concrete-lexicon.md) / [FR-044](./FR-044-project-glossary-lexicon.md) (the lexicon consumed by `vague-response`), [FR-010](./FR-010-query-api.md) (table extraction)
- **Downstream**: [FR-048](./FR-048-per-check-grammar-severity.md) (per-check severity promotion), the authoring and review workflows consume `ac` findings; the AC quality gate feeds the coverage rollup ([FR-050](./FR-050-declarative-coverage-computation.md))
