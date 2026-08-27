---
id: AP-201
title: Quire detection and minting assurance profile
type: AssuranceProfile
status: active
owner: quire-maintainers
scope: coverage collection, trace binding, row minting, diagnostics, and published schemas
impact: material evidence-interpretation impact across Quoin and Filament consumers
impact_assessments:
  - concern: a plausible coverage figure is produced from a population the engine did not read
    tier: material
    scenario: reviewers accept a backed-row percentage while trace bindings or minted rows are absent
    dimensions:
      consequence: assurance work is redirected and missing tests can be reported as covered
      reversibility: the conclusion can be retracted after rerunning with complete provenance
      scope_of_effect: every consumer of the affected coverage payload
      detectability: low unless premise metrics and silent-zero sentinels are inspected
      recovery: rerun the pinned corpus and publish a corrected record with the invalidation reason
    rationale: consumers use these measurements to prioritize test and specification work
    uncertainty: controlled-corpus coverage does not establish behavior for every repository convention
review_selection:
  mode: require
  analyses: [evidence, scope-boundary]
  rationale: changes can alter both the measured population and the interpretation of its result
lifecycle: [development, review, release, maintenance]
relationships: []
---

# Quire detection and minting assurance profile

## Purpose and Scope

This profile governs Quire's coverage collection, trace binding, row minting,
diagnostics, metric envelope, and published schemas. It does not define a
repository-wide coverage target.

## Applicability and Impact

Apply it to changes that can alter what the engine examines, matches, mints, or
reports. A wrong result is material because Quoin and human reviews use it to
decide where testing work is needed.

## Assurance Concerns

Prioritize complete populations, stable reason tokens, explicit not-computed
states, deterministic output, schema compatibility, tool provenance, and clear
separation between observation and policy.

## Selected Practices

Run `make ci`, the controlled corpus, and relevant mutation/self-tests. Require a
failure fixture and healthy control for detector changes. Compare only records
whose measurement definitions and producer configurations are compatible.

## Evidence Expectations

Retain source and corpus revisions, engine identity and capabilities, module
digest, raw payload, measurement-plan version, population counts, gap count,
and exact gate output. A zero without evidence that the population was read is
not a pass.

## Measurement Ownership

Quire owns MP-201 through MP-208 and the engine benchmark producer that records
them under `spec/evidence/measurements`. Corpus inventory and Quoin
finding-quality measurements remain in their owning repositories; the
portfolio reads those stores without moving or retyping their observations.

## Tool Reliance and Independence

Quire's tests show implementation conformance but are not independent of the
engine. The language-neutral corpus and its Python reader provide a distinct
data and implementation path; human adjudication remains necessary for policy
or semantic-quality claims.

## Exceptions and Escalation

An incompatible or incomplete measurement is reported as such and cannot be
waived into comparability. Promotion from observation or ratchet to a target or
gate requires an owner-approved plan change.
