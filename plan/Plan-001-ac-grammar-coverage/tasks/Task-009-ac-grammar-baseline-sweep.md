---
id: Task-009
title: "Corpus cleanup — AC-grammar baseline sweep (user-gated promotion)"
type: Task
status: not_started
track: C
priority: P2
relationships:
  - target: ix://agent-ix/quire-rs/Task-003
    type: depends_on
  - target: ix://agent-ix/quire-rs/FR-047
    type: references
  - target: ix://agent-ix/quire-rs/TC-714
    type: verifies
---
# Task-009: Corpus cleanup — AC-grammar baseline sweep (user-gated promotion)

## Scope

**Corpus/quality work class — own branch (`chore/ac-grammar-baseline`), never
mixed with feature branches.** After Gate G1: run the `ac` grammar over this
repo's spec corpus, record the per-check baseline histogram (via the generic
`--summary`), fix mechanical advisory findings in this repo's ACs (wording
only — a finding that implies a requirement change gets a CR note instead),
and re-record the post-fix baseline. Precedent: the FR-042 EARS rollout
baseline (791 vague / 333 non-singular before enforcement).

## Subtasks
- [ ] **Baseline capture.** Per-check counts over `spec/**/*.md`, committed to
  this task's Notes.
- [ ] **Mechanical fixes.** Clear-cut wording fixes to AC cells; no semantic
  requirement changes without a CR note; re-validate + matrix untouched.
- [ ] **Promotion proposal.** Present per-check promotion candidates
  (`warning` → `error`, or `off` for high-noise checks) with counts. **STOP:
  promotion is user-gated (FR-047-CON-1) — do not edit any
  `grammar_severity` default without explicit sign-off.**

## Deliverables
- Baseline report (before/after counts) in this task file; corpus wording
  fixes on the cleanup branch; a promotion proposal awaiting user decision.

## Baseline (Gate G1 dry-run, 2026-08-04 — counts only, no corpus edits)

Swept `spec/**/*.md` with the `ac` grammar through the PyO3 surface at
Task-003 completion (44 FR documents; `ac` binds to FR only):

| Check | Count |
|---|---|
| `ac:unclassifiable` | 322 |
| `ac:no-observable-outcome` | 12 |
| `ac:non-canonical-shape` | 2 |
| `ac:non-singular` | 2 |
| `ac:vague-response` | 2 |
| **total** | **340** |

Heaviest documents: FR-011 (16), FR-013 (14), FR-047 (14), FR-032 (13),
FR-033 (13), FR-051 (13).

Read of the shape: the corpus authors acceptance criteria as declarative
assertions (`A manifest declaring X loads, and Registry::grammar_severity()
returns the merged map`) — neither EARS nor Given/When/Then, so
`unclassifiable` is a correct classification, not a classifier defect
(sampled against FR-042's own AC table). It does mean **`ac:unclassifiable`
would flag ~95% of the corpus if promoted**, which is the central input to the
promotion proposal below — the realistic first promotion candidates are the
low-count checks, with `unclassifiable` staying `warning` (or `off`) until the
corpus converges on EARS.

**Promotion remains user-gated (FR-047-CON-1): no `grammar_severity` default
has been authored, and none will be without explicit sign-off.**

## Notes
- Wider-ecosystem sweeps (other repos' corpora) are follow-up work owned by
  those repos — this task covers quire-rs only.
- Never tune the checker to the corpus from this branch; classifier bugs
  found here are Track A fixes.
