---
id: MP-205
title: Specific property-shape census
type: MeasurementPlan
status: active
owner: quire-maintainers
stage: baseline
metric: properties.specific_shaped_pct
definition_version: properties.specific-shaped-v1
relationships: []
---

# Specific property-shape census

## Decision Objective

Measure how many binding acceptance criteria expose a specific property shape
that downstream test-quality analysis can inspect.

## Population and Scope

Include binding acceptance criteria in one pinned corpus entry.

## Measure Definition

`properties.specific_shaped_pct` is `specific_shaped / criteria * 100`,
definition `properties.specific-shaped-v1`; retain both counts.

## Collection and Provenance

Collect from the raw properties payload and retain engine, source, corpus,
module, configuration, timestamp, and raw-evidence digest.

## Environment and Sampling

Keep property idioms and module vocabulary fixed for comparison.

## Interpretation and Limitations

Specific shape makes an oracle inspectable; it does not prove the requirement
or test is semantically correct.

## Comparison and Enforcement

Baseline per pinned corpus. Refuse unlike definitions or configurations and
flag population changes. No repository-wide target is declared.
