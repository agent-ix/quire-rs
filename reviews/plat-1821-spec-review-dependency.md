---
id: SR-2026111921333400000013
title: "Dependency review — PLAT-1821 anomaly records"
type: SpecReview
analysis: dependency
scope: "agent-ix/quire-rs; spec/functional/FR-097-anomaly-records-for-release-reporting.md, spec/usecase/US-020-adopter-reviews-release-anomalies.md, Quoin FR-162 through FR-164"
review_set: subset
---

## Summary

Reviewed the new cross-repository dependency edges and responsibility boundaries. Quoin is correctly named as report owner, but the referenced requirements do not support the fixed-anomaly output promised by the new use case.

## Verdict

**CONDITIONAL** — the use case has an unmet output dependency.

## Assurance Context

Inspected `spec/assurance/AP-201-detection-minting.md`. It does not apply: its decision boundary covers engine behavior that examines, matches, mints, or reports coverage evidence; this PR authors consumer-side anomaly records and does not change those engine behaviors.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | US-020-EX-2 depends on a Fixed Anomalies section, while Quoin FR-162 defines its report as anomalies a release still carries and explicitly omits closed fixed anomalies. FR-097 points to FR-162 through FR-164 as the source of release-note behavior, so the fixed-list promise needs an actual supporting Quoin requirement or should be removed. | spec/usecase/US-020-adopter-reviews-release-anomalies.md:32 | wrong-requirement |
