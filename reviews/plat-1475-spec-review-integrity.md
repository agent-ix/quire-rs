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
