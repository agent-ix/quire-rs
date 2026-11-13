---
id: SR-20261113045740
title: "PLAT-1520 quire-rs PR664 spec-review EARS analysis"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-rs PR664 (git diff origin/main...HEAD); spec/functional/FR-084-two-way-relation-coverage-report.md; spec/functional/FR-085-cross-repository-edge-resolution.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-084
    type: references
  - target: ix://agent-ix/quire-rs/FR-085
    type: references
---

## Summary

Ticket: PLAT-1520. Requirement-statement grammar pass over the two new FRs and their ACs. FR-085 states its normative rules with SHALL: kind mismatch SHALL NOT satisfy and SHALL be reported, and a dangling finding SHALL name the reference. Its ACs are single, testable assertions. FR-084's ACs are concrete Given/then vectors.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-084's Description holds no SHALL statement. The report is introduced as "This FR adds a report", and every rule (row selection, state, verification aggregation, ordering, relation naming error) is descriptive prose. The obligation lives only in the ACs, unlike FR-058 and FR-085. Fix: one SHALL statement for the report, for example "quire-rs SHALL produce, for each declared relation, one row per obliged document carrying ...". | spec/functional/FR-084-two-way-relation-coverage-report.md:18 |
| FND-002 | low | FR-084-AC-14 joins two checks under one Test verification: the report is correct over a fixture with unrelated names, and the engine source contains no hazard/threat/mitigation/safety-classification name. The second is a source-content property that FR-058-CON-2 already validates by Inspection. Fix: keep the fixture behaviour as the AC and drop the source-content clause, which FR-058-CON-2 covers. | spec/functional/FR-084-two-way-relation-coverage-report.md:125 |

## Verdict

Minor. Both findings are low. The FR-085 statements and the AC vectors of both FRs are well formed and falsifiable.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |
| FND-002 | fixed | - |
