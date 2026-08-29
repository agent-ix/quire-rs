---
id: MP-201
title: Coverage binding-read premise
type: MeasurementPlan
status: active
owner: quire-maintainers
stage: observe
metric: coverage.binding_read_pct
definition_version: coverage.binding-read-v1
relationships: []
---

# Coverage binding-read premise

## Decision Objective

Determine whether the binder read enough of each language's evidence-symbol
population for downstream coverage figures to be interpretable.

## Population and Scope

For each language in one coverage invocation, include every evidence-symbol
candidate examined by the declared trace forms. Preserve language rows; do not
average them into a repository score.

## Measure Definition

`coverage.binding_read_pct` is `bound / candidates * 100`. Definition
`coverage.binding-read-v1` records both counts. Zero bound with candidates is
an unambiguous premise failure. The current 5% boundary only triggers the
uncertainty-shaped `low-symbol-binding` observation; it is not a health target
and does not diagnose whether sparse tagging or a pattern mismatch caused it.

## Collection and Provenance

Collect from the `binding_census` in the raw `quire coverage --json` payload.
Record CLI and engine versions, capabilities, source revision, module digest,
command configuration, timestamp, and raw-payload digest.

## Environment and Sampling

Record language, repository revision, module set, exclusions, and corpus pin.
Use zero, below-boundary, exactly-at-boundary, and above-boundary controls.

## Interpretation and Limitations

This is a premise metric, not test coverage. Low values cannot distinguish a
working binder over mostly untagged tests from a near-miss marker convention.
Do not compare pooled values across different language populations.

## Comparison and Enforcement

Remain at `observe`. The zero-bound reason is factual; the 5% observation is
advisory and non-diagnostic. Changing its boundary requires a new definition
version and comparative corpus evidence. It never gates backed percentage.
