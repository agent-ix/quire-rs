---
id: FRR-002
title: "quire-rs supplier process-risk rating: vulnerability reporting policy"
type: FeatureRiskRating
scope: supplier
rating: not_high_process_risk
status: proposed
relationships:
  - target: ix://agent-ix/quire-rs/IU-001
    type: uses_intended_use
  - target: ix://agent-ix/quire-rs/FR-097
    type: rates_feature
---

# quire-rs supplier process-risk rating: vulnerability reporting policy

## Rationale

Under IU-001, this policy does not compute the parse, validation, traceability, or coverage records produced by quire-rs. A failure could affect how a vulnerability report is submitted, but would not change those records. This proposed rating is a human decision for the stated intended use; it is not calculated.

## Assurance activities

| Activity | Acceptance criterion |
|---|---|
| Inspect the reporting-policy examples for FR-097: inspect the focused fixture result and its diagnostics. | FR-097-AC-1 |
