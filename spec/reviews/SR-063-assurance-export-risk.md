---
id: SR-063
title: "Risk and complexity review of source-grounded assurance export"
type: SpecReview
analysis: risk-complexity
scope: "StR-007, FR-067, FR-068"
review_set: all
---

## Summary

The principal risk is silent compatibility or identity drift at the new offline contract. Golden-schema, mutation, deterministic-order, and source-digest tests are the named mitigations; no unmitigated high-risk or high-volatility item remains.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No open risk or volatility issues | - |

## Risk Register

| Requirement | Tech Risk | Volatility | Drivers | Mitigation |
| --- | --- | --- | --- | --- |
| StR-007 | medium | low | Cross-repository evidence-chain integrity | Pinned-source fixture and consumer contract |
| FR-067 | medium | low | Versioned schema and canonical digest compatibility | Hand-authored schema, v1 golden, premise mutation tests |
| FR-068 | medium | medium | Many existing authoritative record shapes and module-declared applicability | Typed variants, bijection/property tests, static boundary audit |

## Top Hazards

1. A payload field or state token changes without a v2 contract; TC-1089 pins every v1 field.
2. A locator names bytes other than those loaded; TC-1091/TC-1092 recompute exact digests.
3. Production scope is mistaken for evidence; TC-1095 keeps `implements` and `verifies` as distinct variants.

Failure-domain gaps: none; see SR-059.
