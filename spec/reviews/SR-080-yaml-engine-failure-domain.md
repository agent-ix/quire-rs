---
id: SR-080
title: "Failure-domain review of the YAML engine decision"
type: SpecReview
analysis: failure-domain
scope: "ADR-0012, NFR-009"
review_set: all
---

## Summary

Parity, MSRV, license, advisory, unsafe, and wrong-resolution failures block the
change. The exact legacy-backend pin nevertheless lacks a declared operational
state if a new advisory arrives before a Rust-1.82 migration is available.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-440 | high | An advisory against `yaml_serde 0.10.2` or `unsafe-libyaml 0.2.11` reopens the ADR, but the candidate does not state that affected releases are blocked while reopened or identify the permitted recovery choices. Reopening alone is workflow state, not fail-closed product behavior. | ADR-0012:168-189; NFR-009-AC-9 | missing-requirement |

## Failure posture

Wrong package/version, semantic differential, parity failure, typed-consumer
failure, MSRV failure, and current advisory/license failure are otherwise
explicit and testable.

## Repeat-review disposition

| ID | Disposition |
| --- | --- |
| FND-440 | Fixed: ADR-0012 now blocks affected releases while reopened and limits recovery to a newly reviewed engine/MSRV/removal decision or a separate expiring security waiver. |
