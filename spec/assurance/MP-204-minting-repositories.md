---
id: MP-204
title: Minting repository census
type: MeasurementPlan
status: active
owner: quire-maintainers
stage: baseline
metric: coverage.minting_repos
definition_version: coverage.minting-repos-v1
relationships: []
---

# Minting repository census

## Decision Objective

Expose repositories in a corpus whose declared matrix paths can mint test rows.

## Population and Scope

Include every repository walked in the named corpus entry.

## Measure Definition

`coverage.minting_repos` counts repositories with the declared test-case
summary surface, definition `coverage.minting-repos-v1`. Store matched and
examined repository counts.

## Collection and Provenance

Retain the raw coverage envelope and the source, corpus, engine, module,
configuration, and timestamp provenance.

## Environment and Sampling

Pin corpus identity and module declarations. Record inaccessible repositories
as incomplete population rather than dropping them.

## Interpretation and Limitations

Minting capability says rows can enter the denominator; it does not show that
rows are backed or tests are good.

## Comparison and Enforcement

Use as a baseline and population-integrity signal. Population changes are
flagged, never rendered as bare quality regressions.
