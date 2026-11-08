---
id: AP-201
title: Quire detection and minting assurance profile
type: AssuranceProfile
status: active
owner: quire-maintainers
schema_version: 0.2
profile_kind: general
scope: coverage collection, trace binding, row minting, diagnostics, and published schemas
impact_assessments:
  - id: coverage-population-interpretation
    scenario: reviewers accept a backed-row percentage while trace bindings or minted rows are absent
    severity: material
review_policy:
  mode: require
  operations: [spec-review]
relationships: []
---

# Quire detection and minting assurance profile

## Decision Boundary

This profile governs Quire's coverage collection, trace binding, row minting,
diagnostics, metric envelope, and published schemas. It applies to changes that
can alter what the engine examines, matches, mints, or reports. It does not
define a repository-wide coverage target.

## Impact Scenarios

The material scenario is a plausible coverage figure produced from a population
the engine did not read. Reviewers may accept a backed-row percentage while
trace bindings or minted rows are absent, redirecting assurance work and making
missing tests appear covered. The conclusion can be retracted by rerunning with
complete provenance; recovery is to rerun the pinned corpus and publish a
corrected record with the invalidation reason. Detectability is low unless
premise metrics and silent-zero sentinels are inspected. Controlled-corpus
coverage does not establish behavior for every repository convention.

## Evidence Policy

Run `make ci`, the controlled corpus, and relevant mutation/self-tests. Detector
changes need a failure fixture and healthy control. Compare only records whose
measurement definitions and producer configurations are compatible. Retain
engine capabilities, raw payload, measurement-plan version, population counts,
gap count, and exact gate output. A zero without evidence that the population
was read is not a pass.

Quire's tests show implementation conformance but are not independent of the
engine. The language-neutral corpus is read by the QA corpus Rust crate and by
Quire's separate Rust test harness. Their independent case loading and grading
paths can expose disagreement; bounds assertions that call the QA crate
directly share its implementation and provide no second bounds oracle. Human
adjudication remains necessary for policy or semantic-quality claims.

## Measurement Policy

Quire owns MP-201 through MP-208 and the engine benchmark producer that records
them under `spec/evidence/measurements`. Corpus inventory and Quoin
finding-quality measurements remain in their owning repositories; the
portfolio reads those stores without moving or retyping their observations.
Promotion from observation or ratchet to a target or gate requires an
owner-approved plan change.

## Exceptions

An incompatible or incomplete measurement is reported as such and cannot be
waived into comparability.

> **CR-190 note (STD-227):** The QA reader moved from Python to Rust. This
> statement names the remaining independent paths and the shared bounds API.
