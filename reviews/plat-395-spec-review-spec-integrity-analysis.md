---
id: SR-16315
title: "Spec review of FR-047 CR-197 and FR-047-AC-19 (quire-rs PR 597)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-rs@66459e84; spec/functional/FR-047-acceptance-criteria-grammar.md; spec/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-047
    type: reviews
---

# Spec review of FR-047 CR-197

## Summary

Ticket: PLAT-395. Reviewed the check-1 text change, the new FR-047-AC-19 row, the CR-197 note,
the TC-2030 row and the AC-to-TC mapping. CR-197 and TC-2030 are each new and used only in this
change. The AC-19 row is atomic and testable, and its examples match the regex.

## Verdict

**CHANGES REQUIRED.** The CR-197 note and the check-1 prose state the rule's effect without a
measurement, and the measurement contradicts them.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The CR-197 note says about 12 of the 50 findings were imperatives and that the check "gains" a branch for them, which implies most are cleared. Check 1 still cites "fires on 50 (0.19%)", the count before the rule. Measured on a 27,214-cell frame with the same exclusions, the rule takes findings from 49 to 46. It removes 3 imperatives, all correct, and changes no true positive. Precision goes from 77.6% to 82.6%. Imperatives without a preposition, or with a verb outside the list, still fire. Record the measured after-count and say that the branch clears only imperatives that carry a prepositional complement. | spec/functional/FR-047-acceptance-criteria-grammar.md:103-107,525-535 |
| FND-002 | low | FR-047-AC-19 sits between AC-14 and AC-15 in the criteria table. Every other row is in numeric order, so a reader scanning for AC-19 after AC-18 will not find it there. Move the row after FR-047-AC-18. | spec/functional/FR-047-acceptance-criteria-grammar.md:562 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1 (head 155b534): check 1 cites the 27,214-cell frame and 49 findings; the CR-202 note (renumbered from CR-197) records 38 true, 49 -> 46, the 3 cleared cells and the classes still firing. Re-measurement confirms the numbers. |
| FND-002 | fixed | Round 1: the FR-047-AC-19 row now follows FR-047-AC-18. |
