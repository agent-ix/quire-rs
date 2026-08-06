---
id: Task-004
title: "FR-050 — `traceability:` model loading (shared dependency)"
type: Task
status: completed
track: B
priority: P0
relationships:
  - target: ix://agent-ix/quire-rs/FR-050
    type: references
  - target: ix://agent-ix/quire-rs/TC-732
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-733
    type: verifies
---
# Task-004: FR-050 — `traceability:` model loading (shared dependency)

## Scope

The declarative model that three downstream tasks consume (extract first —
shared dependency): parse the manifest `traceability:` section into typed
declarations — trace targets (archetype + section + table + id column,
incl. auxiliary trace sources outside the corpus walk), document references
(column/annotation pattern → target kinds), status vocabulary
(`complete`/`pending`/`failed` classes), and the trace-tag grammar reference —
merged across modules and exposed on the `Registry`. Absent section → model
undeclared; malformed section → module-load failure.

## Subtasks
- [x] **Declaration types.** Typed model structs; no FR/AC/TC semantics in
  the engine — everything comes from the declaration.
- [x] **Loader + validation.** Load, shape-validate (TC-733), expose via a
  `Registry` accessor (TC-732).
- [x] **Fixture modules.** ISO-shaped fixture AND a non-ISO fixture (different
  archetype/id pattern/status values) for TC-727/739/745 downstream.

## Deliverables
- Model types + loader + accessor; two fixture modules; tests tagged TC-732,
  TC-733.

## Implementation record (2026-08-04)

- `src/traceability.rs` holds the model: `TraceTarget` (archetype-minted or
  auxiliary-`document`-minted, section + `id_column`), `DocumentReference`
  (section + column + capturing `pattern` + `targets`, optional
  `row_id_column`), `StatusVocabulary` (`column` + complete/pending/failed
  value lists with `class_of`), and `TraceTagGrammar` (canonical `markers` and
  the sunset-gated `legacy` forms, each with a capturing pattern).
- Every struct is `deny_unknown_fields`, so a typo inside `traceability:` is a
  module-load failure; `validate()` additionally rejects empty/duplicate names,
  a target declaring both or neither of `archetype`/`document`, a reference to
  an undeclared target, a non-capturing pattern, a status value in two classes,
  and a legacy form rewriting to an undeclared marker.
- `Registry::traceability()` returns `Option<&TraceabilityModel>` — `None` when
  nothing is declared, which is what FR-050-AC-9's diagnostic will read.
  Cross-module merge is first-wins by entry name (loader `merge_traceability`),
  with the first declared `status` winning.
- Fixtures live in `tests/fixtures/traceability/{iso,alt}` — deliberately NOT
  under `tests/fixtures/modules/`, which is a shared search root other tests
  load wholesale (a fixture declaring its own `FR` shadowed the ISO one and
  broke the determinism test).

## Notes
- The real ISO declaration ships in `spec-artifacts-iso` (external follow-up);
  this repo's tasks run on the fixtures.
- Unblocks: Task-006 (trace-tag grammar), Task-007 (reconciliation),
  Task-008 (reference declarations).
