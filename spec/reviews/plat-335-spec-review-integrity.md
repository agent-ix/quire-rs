---
id: SR-16432
title: "Integrity review of the emitter relocation and FR-051-AC-47/48 (quire-rs PR 622)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-rs; spec/functional/FR-051-source-symbol-extraction.md, spec/tests.md (PR 622 diff only, head 21408e0)"
review_set: subset
---

## Summary

Tickets: PLAT-334, PLAT-335. This review checks consistency after the deletions and against the upstream contract quoin FR-120, which FR-051-AC-47 cites as its authority (emitter rule E-2). It also checks id collisions with main and with every open quire-rs PR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-051-AC-47 cites `ix://agent-ix/quoin/FR-120` E-2 for the TypeScript rule. quoin main FR-120's symbol table still says, for vitest/jest, `An enclosing describe does not qualify its members; a class qualifies as Class.method.` AC-47 is correct and FR-120 is wrong. vitest/jest register tests only through `it`/`test`, and the TS extractor gives a class only to non-test methods (src/symbols/typescript.rs:62-65). ix-trace-ts FR-001 also says `there is no class-qualified form`. The cited authority still contradicts the AC that cites it. Fix in quoin: drop `a class qualifies as Class.method` from the FR-120 vitest/jest row. No quoin PR is open for it (quoin PR 776 amended only E-4). | spec/functional/FR-051-source-symbol-extraction.md:235 |

## Verdict

Examined: FR-051-AC-47, FR-051-AC-48, the PLAT-334 / PLAT-335 note, the FR-051 summary row (spec/tests.md:149), TC-2154 and TC-2155 (spec/tests.md:1153-1154), the audit rows (spec/tests.md:1987-1988), and the removal of the FR-079/FR-080 summary rows, TC rows, AC-audit rows and CON-audit rows.

After the deletions the tables are consistent. Every new AC has one TC and one audit row. No row names a removed id. `quire validate` exits 0 on all requirement documents and `spec/tests.md`.

Id collisions: none. FR-051-AC-47/48 and TC-2154/2155 appear nowhere on main or in open PRs 609, 611, 615, 616, 617, 618, 620, 621 and 623. PR 623 uses FR-051-AC-49/50, CR-215 and TC-2160..TC-2164. PR 620 uses TC-2150..TC-2152.

Merge-order note, not a defect: PR 623 edits the same FR-051 summary row (`AC-1..46` to `AC-1..50`) and the same AC table end. A trial merge shows 4 conflict hunks. Whichever PR lands second must write the union, `AC-1..50` with both TC sets and both 🚧 exceptions. The PR merges cleanly with main and with PR 620. FR-051-AC-48 has no staleness clause.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | deferred | Commit f043565 rewords FR-051-AC-47 to cite `ix://agent-ix/quoin/FR-120` E-2 only for the emitters' name equality with the pairs, and names `ix-trace-py` FR-001 and `ix-trace-ts` FR-001 as the owners. AC-47 no longer depends on FR-120's wrong vitest/jest row. That row (`a class qualifies as Class.method`) is still on quoin main, and the fix belongs in agent-ix/quoin, outside this PR. No quoin PR is open for it. |
