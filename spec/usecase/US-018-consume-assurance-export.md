---
id: US-018
title: "Consume a source-grounded assurance export"
type: US
relationships:
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-007"
    type: "traces_to"
    cardinality: "1:1"
---
# US-018: Consume a source-grounded assurance export

## Story

**As an** assurance reviewer comparing evidence across repository revisions
**I want** one validated export of Quire's artifacts, obligations, source bindings, and relationships
**So that** I can audit the evidence chain without re-parsing the specification or guessing what absent data means.

## Context

Quoin currently combines obligations from `coverage --json` with a second
frontmatter reader because no Quire surface exports the bounded corpus graph.
The reader must reproduce edge direction and ownership, then join source-symbol
bindings from another shape. A stable projection lets the consumer validate one
boundary and leaves Quire's module-declared semantics intact.

## Acceptance Examples (Illustrative)

### US-018-EX-1: A claim opens at its source

- **Given** a validated export pinned to a repository revision
- **When** the reviewer follows an artifact or obligation source locator
- **Then** the cited bytes at that revision reproduce the exported digest

### US-018-EX-2: An unsupported premise is refused

- **Given** an export naming an unknown contract version or module-schema digest
- **When** the assurance tool imports it
- **Then** import stops before any claim or relationship is returned

### US-018-EX-3: Absence keeps its meaning

- **Given** one applicable required relation is absent and another relation does not apply
- **When** the reviewer inspects the export
- **Then** the first reads `missing` and the second reads `not_applicable`

## Constraints (Contextual)

The export is an immutable interchange artifact, not a persisted graph service.
Freshness verdicts remain the assurance auditor's decision.

## Dependencies (Contextual)

Upstream: [StR-007](../stakeholder/StR-007-source-grounded-assurance-data.md)
and the existing bounded corpus. Downstream: Quoin's read-only assurance case
and evidence auditor.

## Priority and Risk (Informative)

Business value is high because the export removes a competing parser from the
assurance path. The main risk is accidental semantic drift during contract
evolution.
