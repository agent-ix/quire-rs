---
id: MP-206
title: Silent-zero integrity sentinel
type: MeasurementPlan
status: active
owner: quire-maintainers
stage: gate
metric: sentinel.silent_zero
definition_version: sentinel.silent-zero-v1
relationships: []
---

# Silent-zero integrity sentinel

## Decision Objective

Prevent a ratio metric from appearing valid when its instrument matched none
of a non-empty examined population and emitted no diagnostic.

## Population and Scope

Include every ratio-shaped metric emitted for every corpus entry in one run.
Count-shaped metrics and genuinely empty populations are excluded.

## Measure Definition

`sentinel.silent_zero` counts metrics with `examined > 0`, `matched = 0`, and
no accompanying diagnostic, definition `sentinel.silent-zero-v1`.

## Collection and Provenance

Evaluate complete raw payloads and retain engine capabilities, module digest,
source and corpus revisions, configuration, timestamp, and payload digest.

## Environment and Sampling

Exercise positive controls and mutations that remove the diagnostic or blind
the instrument.

## Interpretation and Limitations

The sentinel establishes measurement integrity, not quality. A diagnostic can
still be wrong and requires its own validation.

## Comparison and Enforcement

Gate at exactly zero with no tolerance or ratchet. A missing capability or
incomplete run is a refusal, not a zero.
