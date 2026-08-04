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

## Notes
- Wider-ecosystem sweeps (other repos' corpora) are follow-up work owned by
  those repos — this task covers quire-rs only.
- Never tune the checker to the corpus from this branch; classifier bugs
  found here are Track A fixes.
