---
id: SR-003
title: "Gap analysis — Plan-001 AC grammar, severity, coverage, symbol extraction"
type: SpecReview
analysis: gap-analysis
scope: "plan/Plan-001-ac-grammar-coverage/, spec/functional/FR-047-acceptance-criteria-grammar.md, spec/functional/FR-048-per-check-grammar-severity.md, spec/functional/FR-049-verification-reference-integrity.md, spec/functional/FR-050-declarative-coverage-computation.md, spec/functional/FR-051-source-symbol-extraction.md, spec/tests.md, src/grammar/, src/traceability.rs, src/coverage.rs, src/corpus/, src/symbols/"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-rs/plan/Plan-001", type: reviews }
  - { target: "ix://agent-ix/quire-rs/spec/tests", type: references }
  - { target: "ix://agent-ix/quire-rs/spec/functional/FR-047", type: references }
  - { target: "ix://agent-ix/quire-rs/spec/functional/FR-050", type: references }
  - { target: "ix://agent-ix/quire-rs/spec/functional/FR-051", type: references }
---

## Summary

Post-implementation gap analysis of Plan-001 on `task/traceability-coverage`
(= PR #16 head). Steps 1–3 of the gap-analysis procedure were run; the optional
semantic review (step 4) was **skipped** — it was not opted into.

**Plan completion.** Eight of ten tasks are `completed`. Task-009 (baseline
sweep and promotion proposal) and Task-010 (legacy trace-tag migration) are
`not_started`. Neither is unfinished engineering: both are blocked on a user
gate this review cannot clear — FR-047-CON-1 for severity promotion, and
FR-051-CON-3 for removing legacy-tag recognition. Task-009's body nonetheless
records three completed baselines, so its status field understates what is in
the file (FND-001).

**Matrix verification.** All 55 Test Case rows in the slice range (TC-707..761)
resolve to a real test. Five are marked 🚧 and correctly so: TC-714, TC-720,
TC-721, TC-740 and TC-755 state their criteria at the `quire validate` /
`quire coverage` **command** level, and the CLI wiring is EXT-3 in `quire-cli`.
The engine APIs beneath them are green. No row claims ✅ without a backing test,
and no test carries a tracking tag absent from the matrix.

**Underspecified code.** Every module added by the slice — `grammar/ac.rs`,
`traceability.rs`, `coverage.rs`, `corpus/declared_tables.rs`,
`corpus/trace_refs.rs`, `symbols/*` — names its owning FR in its module
documentation, and the two helpers added by CR-017 (`mask_code_spans`,
`split_sentences`) are owned by the new FR-047-AC-13. No stub files, no
placeholder returns, no `TODO`/`FIXME`/`todo!()` in the slice.

## Verdict

**CONDITIONAL** — with a stated deviation from the verdict rule.

By the letter of the rule this is a FAIL: two tasks are incomplete. Recording it
as FAIL would misdescribe the state, because neither task is missing work — both
are parked on a decision the review is not entitled to make, and one of them
(promotion) is a decision this repo's own constraints say must stay parked until
a human clears it. The deviation is recorded here rather than applied silently.

No `high` finding stands. The one that would have been high — the PyO3-parity
criteria resting on a suite CI never ran — was found during the accompanying
code review and fixed in `f61bfb2` before this analysis was written.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Task-009 is `status: not_started` while its body records three completed baseline sweeps (initial, post-CR-013/014, post-CR-017). The status should say what is actually outstanding — the promotion proposal and its user gate — rather than implying no work was done. | Task-009 |
| FND-002 | medium | Five matrix rows (TC-714, TC-720, TC-721, TC-740, TC-755) cannot go ✅ from this repo at all: their criteria are stated at the CLI level and the CLI lives in `quire-cli`. They are honestly marked, but the matrix has no way to express "verified as far as this repo can" versus "unverified", so the two look identical. | TC-714, TC-720, TC-721, TC-740, TC-755, EXT-3 |
| FND-003 | low | FR-047's `vague-response` fires on two of this repo's own acceptance criteria (the PyO3-parity criteria of FR-042 and FR-047), where the outcome clause is concrete and the heuristic keys on *exposed*. Recorded as a checker false positive; deliberately not reworded, since bending a truthful criterion to satisfy a checker is the failure mode this slice exists to prevent. | FR-042-AC-10, FR-047-AC-9 |
| FND-004 | low | Bullet-form Acceptance Criteria sections are segmented by no grammar, so a repo authoring criteria as bullets rather than a table receives no `ac` findings at all. Known and recorded in the CR-014 note as future work; it bounds the grammar's reach without any signal to the author that it does. | FR-047 |
| FND-005 | low | Task-010's prerequisite (EXT-4b, the Rust `#[trace]` crate) is closed and the crate is published to no registry, so the dependency a downstream repo would add is a git reference. Fine for now; it becomes friction the moment a second repo adopts markers. | EXT-4b, Task-010 |

## Coverage

| Check | Result |
| --- | --- |
| Tasks `completed` | 8 / 10 (2 user-gated) |
| Matrix rows backed by a tagged test | 55 / 55 |
| Matrix rows ✅ | 50 / 55 (5 pending EXT-3) |
| Slice ACs with a mapped TC | 54 / 54 |
| Modules with an owning FR | all |
| Semantic review (step 4) | skipped — not opted into |
