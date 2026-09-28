---
id: SR-118
title: "Integrity review of the computed coverage matrix spec (CR-187)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-rs@bb9a280a4c4b4aa054c6cfbec2d2c5b69c114a54; spec/tests.md (TC-1930..TC-1941, FR-050/FR-051 coverage rows, AC to TC audit rows, integrity-check paragraph); spec/functional/FR-050-declarative-coverage-computation.md; spec/functional/FR-051-source-symbol-extraction.md; spec/log.md"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-050"
    type: reviews
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-051"
    type: reviews
---

## Summary

Ticket: PLAT-1077. This review checks the structure and traceability of the
diff.

- **Ids:** TC-1930..TC-1941 are new and unique across `spec/`, `src/` and
  `tests/`. CR-187 appears only in this change.
- **Coverage:** each of the six new ACs has at least one TC row, and each TC
  row's Traces To names an AC that exists. The AC→TC audit table carries all
  six. The FR-050/FR-051 coverage rows are widened to AC-1..51 and AC-1..27.
- **Count:** the mechanically measured defined-AC count rises by exactly 6
  (877 to 883 raw anchors, main to head).
- **Validation:** `quire validate` (installed modules) on the four touched
  files reports exactly what origin/main reports: three pre-existing
  `tests.md` status-header asserts and one pre-existing FR-051 line-139
  warning. Nothing new.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | low | The integrity paragraph now asserts "**718** distinct file-defined ACs ... **0 uncovered** (grep-verified)". Measured at head with the paragraph's own anchor rule (bold or leading-cell definitions, RETIRED lines excluded): 857 defined AC ids, 84 of them absent from the AC→TC audit table. The mismatch is pre-existing, and the +6 delta is right. But the PR re-asserts a figure it did not measure. Drop the absolute number or recount; do not extend it. | tests.md:2012 | wrong-requirement |
| FND-002 | low | TC-1938's row traces FR-050-AC-51, FR-050-AC-7 and FR-050-AC-20, but the audit-table rows for FR-050-AC-7 and FR-050-AC-20 do not list TC-1938. The trace runs one way only. | tests.md:892; tests.md:1734; tests.md:1747 | wrong-requirement |

## Verdict

Structurally sound for the new ids and traces. The two low findings are
bookkeeping and would not on their own block merge.

## New findings (disposition pass 3)

Reviewed at `agent-ix/quire-rs@a512cf52a7400a197ef68f6845a8f20db3286800`.

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-003 | low | The AC-3/AC-18/AC-21 name-set widening is traced in one direction only, the same shape as FND-002. FR-051-AC-3 and FR-051-AC-21 cite TC-1940 in their Verification cells (added in round 2), but TC-1940's Traces To lists only FR-051-AC-27, and the audit-table rows read `FR-051-AC-3 \| TC-743` and `FR-051-AC-21 \| TC-1039, TC-1040`. FR-051-AC-18's new curried form `xit.each([…])(…)` (round 3) has no TC: TC-1940 exercises only the plain `xit(` and `xtest(` calls. Fix: add FR-051-AC-3 and FR-051-AC-21 to TC-1940's Traces To and to their audit rows, and either extend TC-1940 with `xit.each([…])(…)` (then trace AC-18) or drop that example from AC-18. | tests.md:895; tests.md:1760; tests.md:1775; tests.md:1778; FR-051-AC-18 | correct-requirement-no-evidence |

## Dispositions

Reviewed at `agent-ix/quire-rs@2d79eb4f3f1769182a20f3f4827ddef1d1419e8e`.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | still-open | The line now states measured values, which is the right approach (author judgment call 3). But it says 857 while the head measures 858, because FR-051-AC-28 was added in the same commit; 84 audit gaps is still correct. A hand-kept absolute drifted inside its own fix commit. Change it to 858, or state only "every id this change adds has an audit row" plus the pre-existing gap. |
| FND-002 | fixed | 2d79eb4 |
| FND-001 | fixed | 3e54ba3 |
| FND-003 | fixed | 1b7c5d0 |
