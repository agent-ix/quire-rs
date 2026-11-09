---
id: SR-16422
title: "integrity review of CR-215 only marker forms bind (quire-rs PR 623)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-rs@9901e1f3; spec/functional/FR-050-declarative-coverage-computation.md; spec/functional/FR-051-source-symbol-extraction.md; spec/functional/FR-077-trace-search-index.md; spec/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-051
    type: references
---

# Integrity review of CR-215 (quire-rs PR 623)

## Summary

Ticket: PLAT-847. Checked id uniqueness, cross-references, the AC→TC audit and status consistency for the diff. Ids are clean:
- TC-2160..TC-2164, FR-051-AC-49/50, FR-051-CON-4 and CR-215 are absent from origin/main and from every open PR head (#622 uses FR-051-AC-47/48 and TC-2154/2155; #620/#621 TC-2150..2152; #611 CR-211..214; #615, #616, #617 and #618 add none in range).
- The audit table maps AC-11→TC-2160, AC-49→TC-2161/2162, AC-50→TC-2163 and FR-050-AC-44→TC-2164.
- Every clause of AC-49 and AC-50 has a TC.
- The removed TCs (753, 1045, 1047, 1083) are each coupled to removed machinery or to behaviour this change reverses.

The exception is TC-1047's negative control (see SR-16420 FND-005). The defects are stale cross-references and status claims.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The Requirements Traceability rows still cite removed or reversed TCs. The FR-051 row lists `TC-753 (legacy textual forms + rewrite suggestions)`, `TC-1044, TC-1045, TC-1046, TC-1047 (CR-134 ...)` and `TC-806 (CR-043 legacy comma lists bind every id)`. The FR-050 row lists `TC-1083 (#367 ...)` and `TC-1936, TC-1937 (... marker or legacy ... legacy left-endpoint case)`. The four removed ids are also not added to the `Reserved withdrawn IDs` line (spec/tests.md:173), so they can be reallocated | spec/tests.md:150; spec/tests.md:151; spec/tests.md:173 |
| FND-002 | medium | Status overclaims. The FR-051 row stays "✅ Implemented except AC-32..33" and the FR-050 row "✅ Implemented except AC-62", while AC-11, AC-49, AC-50 and FR-050-AC-44 are unimplemented (TC-2160..2164 🚧; current code still binds legacy forms and emits `marker-form-mismatch` for a name subpopulation). The rewritten TC-1044, TC-1055, TC-1056, TC-1057 and TC-1081 keep ✅ although they now describe the not-yet-built advisory | spec/tests.md:151; spec/tests.md:749 |
| FND-003 | low | TC-1894 still says "deliberately unlike the unchanged legacy-form masking on the `verifies`/`non_binding_tags` claim paths", which is the exact phrase FR-077-CON-3 was rewritten to drop. Its row was not updated with its CON | spec/tests.md:1110 |
| FND-004 | low | TC-2090 still asserts "no `RewriteSuggestion`", but FR-050-AC-62 dropped that clause and CR-215 removes the type. The TC is coupled to removed machinery and was not rewritten | spec/tests.md:1127 |
| FND-005 | low | The FR-051 narrative says the shell marker "is **not** the legacy class below, and FR-051-CON-3 does not retire it". The legacy class is gone and CON-3 no longer retires anything, so the reference dangles | spec/functional/FR-051-source-symbol-extraction.md:140 |
| FND-006 | high | `make validate` fails on the PR head (with spec-artifacts-process and spec-artifacts-iso at origin/main): 196 documents, 2 failed. The errors are `audit table: FR-051-AC-47: declared by the FR-051 coverage row but no requirement document defines it`, and the same for AC-48. The FR-051 coverage row declares `AC-1..50`, but AC-47 and AC-48 exist only on PR #622. Declare `AC-1..46, 49..50`, or rebase onto #622 after it lands | spec/tests.md:151 |

## Verdict

FND-006 blocks merge: the spec gate is red. The rest are reference and status defects, fixable in the same round as SR-16420. There are no id collisions with main or with any open PR. #622 edits the same FR-051 summary row and audit block, so whichever PR lands second rebases.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-007 | low | TC-2165's Traces To cell names only FR-051-AC-22, but the test asserts AC-49 clauses (one finding for the same id twice on one line; `unmatched_tags` unchanged) and the audit table maps it under FR-051-AC-49. Add FR-051-AC-49 to its Traces To | spec/tests.md:1156; spec/tests.md:1998 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | The withdrawn ids are now reserved and the FR-051 row is clean. Still open: the FR-050 Requirements Traceability row (spec/tests.md:148) cites `TC-1083 (#367 mixed binding channels retain the self-named subpopulation)` as live evidence, and the US-017 row (spec/tests.md:142) cites `TC-753` |
| FND-002 | fixed | 4d4877e: the FR-050 row reads "except AC-44 (🚧, TC-2164)", the FR-051 row lists AC-11/22/24/28/49/50 as 🚧, and TC-1044, TC-1055..1057 and TC-1081 are 🚧 |
| FND-003 | fixed | 4d4877e: TC-1894 reads "deliberately unlike the advisory masking of FR-051-AC-24" |
| FND-004 | fixed | 4d4877e: TC-2090 no longer mentions `RewriteSuggestion` (0 occurrences in spec/tests.md) |
| FND-005 | fixed | 4d4877e: the shell paragraph reads "It is a marker form, so FR-051-CON-3 does not retire it" |
| FND-006 | fixed | 4d4877e merges main with #622: FR-051-AC-47/48 are now defined and the audit maps them (TC-2154/2155). The coder reports `make validate` at 198 documents, 0 failed; not re-run (no full gates) |
