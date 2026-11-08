---
id: SR-16312
title: "PLAT-321 audit-table completeness spec-review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs; spec/tests.md; spec/functional/FR-050-declarative-coverage-computation.md; PLAT-321; PR598"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/FR-050", type: references}
---

# PLAT-321 audit-table completeness spec-review

## Summary

Ticket: PLAT-321. Spec review of PR agent-ix/quire-rs#598 (head 6fe84930), with
the integrity and EARS sub-analyses applied to the new AC and the corpus edits.
The reviewer spot-checked 32 added audit rows and TC re-traces, comparing AC text
with TC text, and confirmed their tags in source. Of those, 28 fit and 4 are
defects (below). It also checked the four coverage-row edits against the criteria
each document defines.

## Examined Units

- **FR-050-AC-58** (examined), spec/functional/FR-050-declarative-coverage-computation.md:292. The audit-table completeness criterion.
- **FR-050-AC-55** (context_only), spec/functional/FR-050-declarative-coverage-computation.md:289. Retirement is module-declared vocabulary; an undeclared `Withdrawn` remains live.
- **FR-011 coverage row** (examined), spec/tests.md:76. `AC-1..2, 4..8, 13..21`. The document defines 1, 2, 4..8 and 13..21, so the row matches.
- **FR-067 coverage row** (examined). `AC-1..5, 7 (AC-6 withdrawn)`. AC-6 is a `Withdrawn.` tombstone, so the row is honest.
- **FR-074 coverage row** (examined). `AC-1..9, 11..12 (AC-10 never defined)`. The document defines 1..9, 11 and 12, so the row is honest.
- **FR-050 coverage row** (examined). `AC-1..58`. The document defines 1..58, so the row is honest.
- **Spot-checked audit rows that fit** (examined): FR-061-AC-10, FR-062-AC-1/5/12/15/17, FR-067-AC-3/7, FR-068-AC-2/5/9, FR-069-AC-4/8/12, FR-070-AC-3/9/14, FR-071-AC-5/6, FR-072-AC-4/6/8, NFR-019-AC-2, NFR-021-AC-1/4, FR-013-AC-18 (TC-2006), FR-050-AC-52 (TC-2008), FR-050-AC-54 (TC-2010 🚧), FR-072-AC-9 (TC-2011 🚧), and TC-970..981 to FR-074-AC-1..12.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | [trace] Two added audit rows map a retired criterion as if it were live. NFR-006-AC-1 reads `**RETIRED (render removal — 2026-06-04):** ...`, but the new row maps it to TC-056 ("render 100x across threads", 🚧). FR-071-AC-9 reads `Removed: ... the id is not reused`, but the new row maps it to TC-1851. The gate forced both rows because its tombstone rule misses these forms (code-review FND-004). Fix: write them as retired rows, like `NFR-012-AC-1..5 \| RETIRED (...)`, and fix the rule. | spec/tests.md:2195; spec/tests.md:2123 |
| FND-002 | low | [trace] The FR-070-AC-6 audit row lists only TC-1621, a generic schema-validation case. It leaves out TC-1615, the dedicated ✅ case for that AC's own text (cell-cases.json Constraints, `nullable`, `pattern`, duplicate field), which is tagged at tests/it/semantic_properties.rs:398. Fix: `\| FR-070-AC-6 \| TC-1615, TC-1621 \|`. | spec/tests.md:2106 |
| FND-003 | medium | [soundness] FR-050-AC-58 makes a statement opening with `Withdrawn` a tombstone unconditionally. FR-050-AC-55 says retirement uses only module-declared values, and that an undeclared `Withdrawn` remains live. The two ACs of one FR disagree about what retires a criterion. The check also hard-codes its vocabulary (section titles, tombstone words, directory names) in the public engine library rather than taking it as module data. Fix: AC-58 takes its tombstone vocabulary from the declared retirement vocabulary (or states why the audit check differs), and the code follows. | spec/functional/FR-050-declarative-coverage-computation.md:292; spec/functional/FR-050-declarative-coverage-computation.md:289 |
| FND-004 | low | [other] The PR adds provenance ceremony to the spec. The CR-196 note records a commit SHA (`cc510d2`) and a measurement history (111 named, three misreadings, 98 real). The integrity paragraph this PR rewrote keeps the retired hand-kept `Count: 316 ... = 447` chain and the `grep-verified, SR-118 FND-001, round 2` provenance, although it now says the engine enforces completeness. Fix: drop the SHA and the history from the note, keeping the rule and its reason, and delete the retired count chain and the provenance tag from the paragraph. | spec/functional/FR-050-declarative-coverage-computation.md:1102-1117; spec/tests.md:2264 |

## Verdict

Changes requested. The coverage-row edits are honest about undefined and
withdrawn criteria. FR-011's row lists only defined ids, though unlike FR-074 it
does not annotate the gaps. The TC-970..981 re-trace to FR-074 is correct and
matches the existing tags. TC-2010 and TC-2011 are honestly 🚧. FND-001 and
FND-003 should be fixed before merge.
