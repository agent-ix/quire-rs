---
id: SR-20277
title: "gap-analysis of PLAT-1420/PLAT-1485 (quire-rs PR 658)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs@c9ff0947; spec/functional/FR-083-clause-set-publishability.md; spec/functional/FR-073-generic-clause-sets.md; spec/tests.md; tests/it/clause_publishable.rs; src/clauses.rs; src/loader/manifest.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-083
    type: references
  - target: ix://agent-ix/quire-rs/FR-073
    type: references
---

# Gap analysis of PLAT-1420/PLAT-1485 (quire-rs PR 658)

## Summary

Ticket: PLAT-1420. Plan completion: not assessed. `quoin matrix --repo .` (quoin 0.30.1): FR-083-AC-1..7 and AC-9 tagged; FR-083-AC-8 untagged; FR-073-AC-6 tagged. Each FR-083 AC has a trace-tagged test with an assertion that fails on the defect it names (mutation-checked for AC-1, AC-3, AC-6). No stub or coverage inflation found in the diff.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | The TC-2317 row is spliced with the old TC-2248 row: it traces FR-058-AC-12/15 and is marked done, and the TC-2248 row no longer exists (lost in #657). So `quoin matrix` reports FR-083-AC-8 untagged, and the FR-058 summary still cites TC-2248. Fix: restore the TC-2248 row from 03033ae~1, and rewrite TC-2317 to trace FR-083-AC-8 as done. | spec/tests.md:1242 | missing-requirement |
| FND-002 | low | The tests.md status rows are stale. The FR-083 coverage row says AC-1..8 (the spec has AC-9) and is still pending. TC-2313..2316 are still marked pending. TC-2313 says the missing-rights error names the `clause_sets` key, but AC-2 (and the code) names the file. Fix: change the row to AC-1..9 and implemented, mark TC-2313..2317 done, and correct the TC-2313 wording. | spec/tests.md:165,1238-1241 | wrong-requirement |
| FND-003 | low | The second clause of AC-8 (a module with no `clause_sets` returns empty) is asserted in the test tagged TC-2314/AC-3/AC-5, not in the AC-8 test. Fix: move the assertion into the TC-2317 test, or tag it AC-8. | tests/it/clause_publishable.rs:139-141,190-203 | wrong-requirement |

## Verdict

Code satisfies FR-083 AC-1..9 and amended FR-073-AC-6. Not ready until FND-001 is fixed: the spec table loses a TC-2248 row and leaves AC-8 untraced. FND-002 and FND-003 can be fixed in the same round.
