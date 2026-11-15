---
id: MP-207
title: Skeptic suspicion-rate language guard
type: MeasurementPlan
status: active
owner: quire-maintainers
stage: ratchet
metric: skeptic.suspicion_rate
definition_version: skeptic.suspicion-rate-v1
relationships: []
---

# Skeptic suspicion-rate language guard

## Decision Use

Detect a skeptic rule that broadly misreads a language and floods the corpus.

## Population

For each language and pinned corpus entry, include evidence symbols examined by
the source walk.

## Measure Definition

`skeptic.suspicion_rate` is `suspicions / candidates * 100`, definition
`skeptic.suspicion-rate-v1`. Preserve per-language counts.

## Collection Procedure

Retain raw suspicion records, configuration, and timestamp.

## Environment and Sampling

Include healthy language controls and seeded vacuity defects. Do not pool
languages with different syntax.

## Interpretation

A high rate is a review trigger, not proof the rule is wrong; adjudicate a
sample. A low rate does not establish recall.

## Comparison and Enforcement

Ratchet per pinned language corpus. Any boundary change creates a new
definition version; comparisons never assign severity.
