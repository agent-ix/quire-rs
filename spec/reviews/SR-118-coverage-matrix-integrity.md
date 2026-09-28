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
