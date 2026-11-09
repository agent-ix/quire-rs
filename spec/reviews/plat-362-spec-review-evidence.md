---
id: SR-16363
title: "Evidence review of PLAT-362 shell symbol adapter (quire-rs PR 614)"
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-rs; spec/functional/FR-051-source-symbol-extraction.md, spec/tests.md (PR 614 diff only, head 63ac3f0)"
review_set: subset
---

## Summary

Ticket: PLAT-362. This pass checked the verification method of each new AC against its TC. FR-051-AC-34..38 and AC-40 are behaviours an Integration test can observe and fail, and TC-2101..2105 and TC-2107 match them one to one. One clause in AC-39 is a negative property that a test is the wrong method for.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | AC-39 makes "the shell adapter executes, sources and sandboxes nothing" a Test obligation (TC-2106, "nothing is executed"). An integration test cannot fail on that property in any meaningful way, and "sandboxes nothing" has no observable. FR-051-CON-1's no-execution clause, already verified by the static boundary audit TC-756, is the right owner. Drop the clause from AC-39, or verify it by Inspection under CON-1. | spec/functional/FR-051-source-symbol-extraction.md:206, spec/tests.md:1139, spec/tests.md:505 |

## Verdict

Examined: FR-051-AC-34..40 and TC-2101..2107. Methods and priorities are consistent. AC-40/TC-2107 is a true end-to-end check through the coverage report and includes the negative control (same marker outside the glob backs nothing). That control is the evidence the declaration-driven design needs.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The no-execution clause left the AC. FR-051-CON-1 now says "the shell adapter never executes or sources the script it reads", verified by the TC-756 static boundary audit, whose row now says "shell adapter included". AC-45 holds only the observable diagnostic. |
