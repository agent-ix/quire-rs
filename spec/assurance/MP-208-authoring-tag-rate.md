---
id: MP-208
title: Evidence-symbol tag authoring rate
type: MeasurementPlan
status: active
owner: quire-maintainers
stage: observe
metric: authoring.tag_rate
definition_version: authoring.tag-rate-v1
relationships: []
---

# Evidence-symbol tag authoring rate

## Decision Objective

Distinguish tests that carry no trace tag from tests whose authored tag exists
but the declared grammar cannot read it.

## Population and Scope

For each language, include every evidence symbol counted by `binding_census`.
Preserve language rows when interpreting a repository; a pooled total is only a
portfolio summary.

## Measure Definition

`authoring.tag_rate` is `tagged / candidates`. A candidate is tagged when its
attached annotation block carries a generic id-shaped token, or when a declared
form bound it. The invariant is `bound <= tagged <= candidates`.

## Collection and Provenance

Collect the counts and `unmatched_example` from the raw `quire coverage --json`
payload. Retain CLI and engine identity, capabilities, source revision, module
digest, command configuration, timestamp, and raw-payload digest.

## Environment and Sampling

Exercise a no-tag control, an unread-tag case, and a mixed population in every
supported language. Record corpus and module revisions.

## Interpretation and Limitations

`tagged - bound` routes work to the instrument or declaration;
`candidates - tagged` routes work to repository authors. The generic pattern is
deliberately broader than declared forms and may over-route ambiguous tokens to
instrument review; it never absolves the instrument based on its own grammar.

## Comparison and Enforcement

Remain at `observe`. Compare only matching definition and producer
configurations. No universal target or repository-wide percentage is declared.
