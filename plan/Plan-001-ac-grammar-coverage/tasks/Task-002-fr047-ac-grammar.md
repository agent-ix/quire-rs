---
id: Task-002
title: "FR-047 — acceptance-criteria grammar (`ac`)"
type: Task
status: not_started
track: A
priority: P0
relationships:
  - target: ix://agent-ix/quire-rs/Task-001
    type: depends_on
  - target: ix://agent-ix/quire-rs/FR-047
    type: references
  - target: ix://agent-ix/quire-rs/TC-707
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-708
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-709
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-710
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-711
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-712
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-713
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-715
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-751
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-754
    type: verifies
---
# Task-002: FR-047 — acceptance-criteria grammar (`ac`)

## Scope

Register the `ac` grammar on the FR-042 framework: bindings to the FR
`Acceptance Criteria` `Criteria` column and `### <doc-id>-AC-N` supplement
sections; every non-empty cell is a statement (no modal-verb filter); shape
classification `ears` (canonical) / `given-when-then` / `unclassifiable`; the
five checks (unclassifiable, non-singular with the positive/negative pair
idiom, lexicon-backed vague-response, no-observable-outcome with the bounded
observable-verb list, non-canonical-shape); fenced-block/blockquote skip
inside supplements; findings carry `grammar: "ac"` + check id + excerpt +
line + shape + severity and route per the Task-001 severity map; PyO3 parity.

## Subtasks
- [ ] **Bindings + segmentation.** Criteria-column and supplement-section
  binding; empty-cell/no-modal handling (TC-708, TC-712); supplement skip
  rules (TC-754).
- [ ] **Shape classifier.** EARS reuse + GWT recognition + unclassifiable
  (TC-707); non-canonical-shape steer with checks still running on the `Then`
  clause (TC-751).
- [ ] **Checks.** non-singular + pair idiom (TC-709); vague-response via the
  merged FR-043/FR-044 lexicon — one implementation, two grammars (TC-710);
  no-observable-outcome signals (TC-711).
- [ ] **Finding shape + routing.** (TC-713) via the FR-048 severity map.
- [ ] **PyO3 surface.** Same findings as the in-process call (TC-715).

## Deliverables
- `src/grammar` `ac` module + registration; fixtures; unit/integration tests
  tagged TC-707..713, TC-715, TC-751, TC-754.

## Notes
- Rollout default is advisory: no check ships above `warning` (FR-047-CON-1);
  promotion happens only via Task-009's user gate.
- Extend FR-042's segmenter/clause machinery in place — do not fork it.
- Unblocks: Task-003; Task-009 (Track C sweep, after G1).
