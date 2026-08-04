---
id: Task-008
title: "FR-049 — verification-reference integrity in validate_bundle"
type: Task
status: not_started
track: B
priority: P0
relationships:
  - target: ix://agent-ix/quire-rs/Task-004
    type: depends_on
  - target: ix://agent-ix/quire-rs/FR-049
    type: references
  - target: ix://agent-ix/quire-rs/TC-724
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-725
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-726
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-727
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-728
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-729
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-730
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-731
    type: verifies
---
# Task-008: FR-049 — verification-reference integrity in `validate_bundle`

## Scope

Extend bundle validation (FR-038) with the model-driven
`dangling-trace-reference` check: for each document-reference declaration in
the Task-004 model, extract referenced ids from the declared cells (multiple
annotations resolve independently), resolve them against the union of
bundle-document ids and auxiliary-trace-source ids (targeted scan, FR-044
harvester pattern), and report unresolved ids with document path + declaration
provenance. Posture-degradable exactly like `dangling-reference` (error under
`Strict`, warning under `Okf`). No model declared → zero findings.
Deterministic finding order.

## Subtasks
- [ ] **Declared extraction.** Pattern/column/target kinds from the model —
  no ISO hardcoding (TC-727); multi-annotation cells (TC-730).
- [ ] **Resolution set.** Bundle docs + auxiliary source harvest (TC-724,
  TC-728).
- [ ] **Findings + postures.** (TC-725, TC-726); no-model no-op (TC-729);
  deterministic order (TC-731, Property).

## Deliverables
- Check in `src/corpus/validate.rs` alongside `dangling-reference`; tests
  tagged TC-724..731.

## Notes
- Depends only on Task-004 (model loader) — parallel to Task-005/006.
- Real-world activation on ISO repos awaits the `spec-artifacts-iso`
  declaration (external); fixtures cover the engine contract.
