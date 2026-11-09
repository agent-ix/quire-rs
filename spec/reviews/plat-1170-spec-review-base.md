---
id: SR-16326
title: "Base spec review of PLAT-1170 FR-050-AC-61 (quire-rs PR 610)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs; spec/functional/FR-050-declarative-coverage-computation.md, spec/tests.md (PR 610 diff only)"
review_set: subset
---

## Summary

Ticket: PLAT-1170. PR head `9581b22`. Base checklist over the PR's spec-only diff: the CR-207 note and FR-050-AC-61 row in FR-050, plus the FR-050 summary row, the TC-2083 row and the FR-050-AC-61 audit row in `spec/tests.md`. `quire validate` exits 0 on both files, and FR-050-AC-61 gets no grammar warning. Open PR 609 (PLAT-313) mints the same three ids for different content, so the two PRs cannot both merge as written. No hash, pin, digest, tracking record, vendored file or new compatibility layer was introduced. Applicable assurance profile: `spec/assurance/AP-201-detection-minting.md` (`review_policy: require spec-review`). This was a direct review, not an enforced ix-flow run. The enforced acceptance needs a human acknowledgement.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | CR-207, FR-050-AC-61 and TC-2083 are also minted by open PR 609 (PLAT-313, `id_column` as one name or a list), with different meanings. One PR must renumber before merge. The other open PRs (606 CR-205/TC-2081/FR-069-AC-14, 607 CR-206/TC-2082/FR-051-AC-31) do not collide. | spec/functional/FR-050-declarative-coverage-computation.md:233, spec/functional/FR-050-declarative-coverage-computation.md:307, spec/tests.md:1130 |
| FND-002 | low | The FR-050 summary row still reads `✅ Implemented` and lists AC-1..61, but its exceptions note does not mention AC-61, whose only test (TC-2083) is `🚧`. | spec/tests.md:148 |

## Verdict

Examined: FR-050-AC-61 (testable: it names a reason, a severity, four fields and three negative controls, and each can fail), the CR-207 note, and the tests.md summary, TC and audit rows. The audit row and the summary range (AC-1..61) agree with the new AC. Not mergeable until FND-001 is resolved by renumbering in whichever PR lands second.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Commit 606a83c renumbers the PR to CR-209, FR-050-AC-62 and TC-2090. PR 609 now uses CR-208, FR-050-AC-61 and TC-2084..2089. PRs 606, 607, 608 and 611 and `main` use none of the three ids. |
| FND-002 | fixed | The FR-050 summary row now reads `AC-1..60, 62` with status `✅ Implemented except AC-62 (🚧, TC-2090 unimplemented)`. The `, 62` continuation is in the `audit_table` bare-range grammar. |
