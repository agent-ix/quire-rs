---
id: Task-004
title: "FR-050 — `traceability:` model loading (shared dependency)"
type: Task
status: not_started
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
- [ ] **Declaration types.** Typed model structs; no FR/AC/TC semantics in
  the engine — everything comes from the declaration.
- [ ] **Loader + validation.** Load, shape-validate (TC-733), expose via a
  `Registry` accessor (TC-732).
- [ ] **Fixture modules.** ISO-shaped fixture AND a non-ISO fixture (different
  archetype/id pattern/status values) for TC-727/739/745 downstream.

## Deliverables
- Model types + loader + accessor; two fixture modules; tests tagged TC-732,
  TC-733.

## Notes
- The real ISO declaration ships in `spec-artifacts-iso` (external follow-up);
  this repo's tasks run on the fixtures.
- Unblocks: Task-006 (trace-tag grammar), Task-007 (reconciliation),
  Task-008 (reference declarations).
