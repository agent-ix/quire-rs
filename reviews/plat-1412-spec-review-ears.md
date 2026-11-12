---
id: SR-20261103118158
title: "EARS conformance of FR-082 (quire-rs PR 645)"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-rs@326eb1bd; spec/functional/FR-082-criticality-trace-column.md (Description statements)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-082
    type: references
---

# EARS conformance of FR-082 (quire-rs PR 645)

## Summary

Ticket: PLAT-1412. This review checks the normative statements in FR-082's Description against EARS patterns. The unwanted-behaviour clause ("the engine SHALL NOT default, infer, inherit ... or write one back") conforms.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | low | The main statement is compound: "The trace model SHALL expose ... and `quire trace` output SHALL include the column". It joins two SHALLs with two different subjects in two repos. The missing-value rule ("A requirement with no `criticality` is reported with the literal value `unassigned`") is normative but has no SHALL and no subject. Fix: one statement per subject: "When a requirement has no `criticality`, the engine SHALL ...". Move the CLI clause to quire-cli. | spec/functional/FR-082-criticality-trace-column.md:22-26 | wrong-requirement |

## Verdict

Low only. Fix it together with the base and scope-boundary findings.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | still-open | The compound half is fixed (the CLI clause is gone). The core positive behaviour ('The engine reads the value ... and carries it as written. When the document has no such field, the engine leaves the key absent.') still has no SHALL; only the FR-053 and No-inference clauses do. One-line fix: 'The engine SHALL carry ... When ..., the engine SHALL leave the key absent.' |
| FND-001 | fixed | Round 2 (dfcf08f): 'and SHALL carry it as written. When the document has no such field, the engine SHALL leave the key absent.' |
