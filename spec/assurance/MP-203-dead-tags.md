---
id: MP-203
title: Dead trace-tag count
type: MeasurementPlan
status: active
owner: quire-maintainers
stage: baseline
metric: coverage.dead_tags
definition_version: coverage.dead-tags-v1
relationships: []
---

# Dead trace-tag count

## Decision Objective

Identify trace tags that bind to source symbols but match no declared row.

## Population and Scope

Include trace markers written in source for one pinned corpus entry.

## Measure Definition

`coverage.dead_tags` is the count of `untracked_symbols`, definition
`coverage.dead-tags-v1`. It is count-shaped; zero is a valid observation and
is not subject to ratio silent-zero rules.

## Collection and Provenance

Retain the raw coverage payload, tool and engine versions, module digest,
source and corpus revisions, configuration, and timestamp.

## Environment and Sampling

Keep repository scope, languages, exclusions, and module configuration fixed
for baseline comparisons.

## Interpretation and Limitations

A dead tag can be a typo, stale test, or declaration gap. The count does not
choose among those repairs and does not measure test quality.

## Comparison and Enforcement

Establish per-corpus baselines. Report additions and removals with loci; do not
turn the count into a percentage or a universal target.
