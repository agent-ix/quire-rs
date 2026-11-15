---
id: MP-206
title: Silent-zero integrity sentinel
type: MeasurementPlan
status: active
owner: quire-maintainers
stage: gate
ground_truth_kind: mechanical
statistical_design:
  population: Every eligible ratio-shaped metric emitted for each corpus entry in one run.
  sampling: Complete deterministic census of the run's eligible metric rows.
  repetitions: 1
  estimator: count
  error_model: Deterministic comparison of each metric row and its diagnostics.
  uncertainty: No sampling uncertainty; the gate counts missing diagnostic records in a complete census.
  decision_rule:
    comparator: eq
    threshold: 0
negative_controls:
  - kind: suppressed-observation
    description: Keep an eligible metric row with examined > 0 and matched = 0 in the raw payload, but suppress its diagnostic; the sentinel must count that row as silent zero.
metric: sentinel.silent_zero
definition_version: sentinel.silent-zero-v1
relationships: []
---

# Silent-zero integrity sentinel

## Decision Use

Prevent a ratio metric from appearing valid when its instrument matched none
of a non-empty examined population and emitted no diagnostic.

## Population

Include every ratio-shaped metric emitted for every corpus entry in one run.
Count-shaped metrics and genuinely empty populations are excluded.

## Measure Definition

`sentinel.silent_zero` counts metrics with `examined > 0`, `matched = 0`, and
no accompanying diagnostic, definition `sentinel.silent-zero-v1`.

## Collection Procedure

Evaluate complete raw payloads and retain engine capabilities,
configuration, and timestamp.

## Environment and Sampling

Exercise positive controls and mutations that remove the diagnostic or blind
the instrument.

## Interpretation

The sentinel establishes measurement integrity, not quality. A diagnostic can
still be wrong and requires its own validation.

## Comparison and Enforcement

Gate at exactly zero with no tolerance or ratchet. A missing capability or
incomplete run is a refusal, not a zero.
