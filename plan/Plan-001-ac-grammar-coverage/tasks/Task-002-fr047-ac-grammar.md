---
id: Task-002
title: "FR-047 — acceptance-criteria grammar (`ac`)"
type: Task
status: completed
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
  - target: ix://agent-ix/quire-rs/TC-757
    type: verifies
---
# Task-002: FR-047 — acceptance-criteria grammar (`ac`)

## Scope

Register the `ac` grammar on the FR-042 framework: bindings to the FR
`Acceptance Criteria` `Criteria` column and `### <doc-id>-AC-N` supplement
sections; every non-empty cell is a statement (no modal-verb filter); shape
classification `ears` (canonical) / `given-when-then` / `unclassifiable`; the
five checks (unclassifiable, non-singular with the positive/negative pair
idiom, lexicon-backed vague-response, no-observable-outcome with the
module-data `observable_verbs` vocabulary over built-in defaults,
non-canonical-shape); fenced-block/blockquote skip
inside supplements; findings carry `grammar: "ac"` + check id + excerpt +
line + shape + severity and route per the Task-001 severity map; PyO3 parity.

## Subtasks
- [x] **Bindings + segmentation.** Criteria-column and supplement-section
  binding; empty-cell/no-modal handling (TC-708, TC-712); supplement skip
  rules (TC-754).
- [x] **Shape classifier.** EARS reuse + GWT recognition + unclassifiable
  (TC-707); non-canonical-shape steer with checks still running on the `Then`
  clause (TC-751).
- [x] **Checks.** non-singular + pair idiom (TC-709); vague-response via the
  merged FR-043/FR-044 lexicon — one implementation, two grammars (TC-710);
  no-observable-outcome signals with the module-data `observable_verbs`
  registry first-wins over built-in defaults, ADR-0009 pattern (TC-711,
  TC-757).
- [x] **Finding shape + routing.** (TC-713) via the FR-048 severity map.
- [x] **PyO3 surface.** Same findings as the in-process call (TC-715).

## Deliverables
- `src/grammar` `ac` module + registration; fixtures; unit/integration tests
  tagged TC-707..713, TC-715, TC-751, TC-754.

## Implementation record (2026-08-04)

- `src/grammar/ac.rs` holds the grammar; `check_document_grammar` now runs
  **both** grammars of the `iso-spec-core` bundle and lets each grammar's own
  binding decide whether it contributes.
- Binding is literal to the FR text: archetype `FR` only — the
  `Acceptance Criteria` `Criteria` column plus every `### <doc-id>-AC-N`
  section found anywhere in the document tree.
- Shape classification reuses `ears::classify`; EARS wins when both shapes
  could match (`When … shall …` is an EARS event pattern, not a GWT trigger).
- `vague-response` shares one implementation with EARS: `ears.rs` grew
  `vague_verb_in_clause` (bare outcome clause, no `shall` anchor) and both
  paths run the same `judge_vague_verb` object-awareness over one weak-verb
  alternation. The EARS-side behaviour is unchanged.
- `no-observable-outcome` reads the outcome clause — after the modal for an
  `ears` cell, after `Then` for a GWT cell, the whole cell when unclassifiable
  (so `The import works correctly` is caught, per FR-047-AC-5's own example).
- `observable_verbs` is module data (ADR 0009): a `BTreeMap<String,
  ObservableVerbDef>` manifest registry merged first-wins across modules and
  layered **over** the built-in defaults by `ObservableVerbs::with_module_verbs`,
  which inflects each verb. No `DuplicateObservableVerb` diagnostic was added:
  FR-047 specifies the first-wins merge only, and no AC covers a diagnostic.
- The PyO3 `check_grammar` surface needed no signature change — module data
  reaches it through the same `module_root` registry. Verified by building the
  wheel and running `tests/python/` (TC-715 green).

## Notes
- Rollout default is advisory: no check ships above `warning` (FR-047-CON-1);
  promotion happens only via Task-009's user gate.
- Extend FR-042's segmenter/clause machinery in place — do not fork it.
- Unblocks: Task-003; Task-009 (Track C sweep, after G1).
