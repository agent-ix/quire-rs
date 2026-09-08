---
id: SR-084
title: "Risk and complexity review of the YAML engine decision"
type: SpecReview
analysis: risk-complexity
scope: "ADR-0012, NFR-009, NFR-002"
review_set: all
---

## Summary

The alias keeps implementation complexity small, but it changes every YAML
consumer and remains on the legacy backend. Semantic parity alone does not guard
the existing parse/validation latency contract.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-446 | medium | ADR-0012 and NFR-009 omit NFR-002's 5 MB parse, typical-artifact validation, and >10% regression gates from required migration evidence. A zero semantic differential can still land a load-bearing latency regression. | ADR-0012:138-153; NFR-002-AC-1..AC-4; NFR-009:46-50 | missing-requirement |

## Risk posture

The selected version minimizes API churn but freezes the old backend. The
decision therefore needs both the existing performance gate and the explicit
security contingency in FND-440.

## Repeat-review disposition

| ID | Disposition |
| --- | --- |
| FND-446 | Fixed: ADR-0012 evidence item 6 and NFR-009-AC-13/TC-1828 bind the unchanged NFR-002 thresholds and stored-baseline regression gate. |
