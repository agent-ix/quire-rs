---
id: MP-202
title: Backed trace census
type: MeasurementPlan
status: active
owner: quire-maintainers
stage: branch-comparison
metric: coverage.backed_pct
definition_version: coverage.backed-v1
relationships: []
---

# Backed trace census

## Decision Objective

Show how a change affects the fraction of declared matrix rows reconciled to
evidence within one pinned corpus.

## Population and Scope

Include reference rows examined by the declared model for one corpus entry.
Keep each repository and corpus revision separate.

## Measure Definition

`coverage.backed_pct` is `backed / total * 100`, definition
`coverage.backed-v1`. Store numerator and denominator. This is a structural
trace census, never a repository-wide test-quality target.

## Collection and Provenance

Collect the raw coverage envelope and record engine, module digest, source and
corpus revisions, configuration, timestamp, and raw-evidence digest.

## Environment and Sampling

Use the exact repository, scope, module set, and exclusions named by the corpus
entry. A changed population remains visible.

## Interpretation and Limitations

Backing shows a readable relation, not that the test is meaningful or passing.
A population correction may lower the percentage without degrading quality.

## Comparison and Enforcement

Compare branches only under the same definition and producer configuration.
Always flag denominator movement. Caller policy may ratchet a pinned corpus;
this plan defines no universal target.
