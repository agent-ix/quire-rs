---
id: SR-20261112155213
title: "integrity review of FR-058 where selector ACs and Test Matrix rows (quire-rs PR 652)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-rs@b04cbdb; spec/functional/FR-058-upward-trace-completeness.md; spec/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-058
    type: references
---

# Integrity review of FR-058 where selector (quire-rs PR 652)

## Summary

Ticket: PLAT-1475. Checked completeness, consistency and atomicity of FR-058-AC-12..20, CON-3 and their Test Matrix rows. Every new AC has exactly one Test Case row, and each TC row names its ACs. Every new AC has an audit row in "AC → TC Coverage Audit". The coverage range was updated to AC-1..20 and CON-1..3. TC ids TC-2248..TC-2255 continue directly after TC-2247 and are not used anywhere else. The validator reports 0 failed and passes the FR-050-AC-58 audit check. CON-3 has no TC row because it is Inspection, the same as CON-2.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The FR-058 coverage row's status still reads "✅ Implemented", but eight of its TCs (TC-2248..TC-2255) are "🚧 Open". A reader of the coverage table sees FR-058 as done while the `where` selector has no implementation. Fix: mark the row as partially implemented (for example "🚧 AC-12..20 open") until the implementation PR lands. | spec/tests.md:157 |
| FND-002 | low | AC-16 combines two unrelated behaviours: the ordering of `where` and `exclude`, and the dead-vocabulary rule being unchanged by `where`. A failure in one does not explain the other, and TC-2251 tests both. Fix: split the dead-vocabulary sentence into its own AC (or move it to AC-20, which already covers "unchanged without `where`"). | spec/functional/FR-058-upward-trace-completeness.md AC-16 |
| FND-003 | low | Broken link. The new section links FR-082 as `./FR-082-requirement-criticality-in-the-coverage-matrix.md`, but the file is `spec/functional/FR-082-coverage-matrix-criticality.md`, so the link 404s. The validator does not check relative Markdown links, which is why the gate stays green. Fix: link `./FR-082-coverage-matrix-criticality.md`. | spec/functional/FR-058-upward-trace-completeness.md:89 |

## Verdict

Mergeable with respect to integrity. All three findings are low: wording, status and link fixes.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | low | The PR description is stale after the fix round. It still says `from: nfr-document`, "match is trim + case-fold", "FR-058-AC-12..20" and "TC-2248..TC-2255", and it does not mention AC-21, AC-22 or the self-report. These repos squash-merge, so the description can become the commit message on main. Fix: update the PR body to `from: NFR`, ASCII case folding, AC-12..22 and TC-2248..TC-2257, and mention the self-report. | quire-rs PR 652 description |

## Dispositions

Round 1, checked at a5fc8b0. Renumbering check: every mention of FR-058-AC-12..22 and TC-2248..TC-2257 in `spec/` is consistent. FR-058 has AC rows 12-22. Each TC row's "Verifies" cell names the renumbered AC (AC-16→TC-2251, AC-17→TC-2252, AC-18→TC-2253, AC-19→TC-2254, AC-20→TC-2255, AC-21→TC-2256, AC-22→TC-2257). The audit rows match, and the coverage row says AC-1..22. TC-2256 and TC-2257 do not exist on main. No other document in `spec/` cites FR-058-AC-12+ by number. The validator gives 0 failed.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The coverage row status reads "🚧 AC-1..11 Implemented; AC-12..22 (TC-2248..TC-2257) Open". |
| FND-002 | fixed | AC-16 now covers only `exclude` (TC-2251), and AC-17 covers only the unchanged dead-vocabulary rule (TC-2252). |
| FND-003 | fixed | The link is `./FR-082-coverage-matrix-criticality.md`, and the file exists. |
