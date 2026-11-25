---
id: US-022
title: "Review the impact of a source change"
type: US
relationships:
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-007"
    type: "traces_to"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-099"
    type: "exercises"
---
# US-022: Review the impact of a source change

## Story

**As a** developer preparing a code review or verification run
**I want** to see which tests and requirement-linked code relate to a source change, with reasons and uncovered areas
**So that** I can direct review and verification while understanding where the available traceability is incomplete.

## Context

Quire-RS can extract source symbols and trace relations, but a developer still
needs to connect a particular base-to-head source change to those relations and
the tests that claim them. Without that view, the developer must manually
compare source changes with requirement links and test declarations. A useful
impact result must show the source evidence behind each selection and make
unavailable or unlinked areas visible to assurance reviewers.

## Acceptance

- **US-022-AC-1**: **Given** a source change in a symbol linked to a requirement
  criterion and tests that declare the same criterion id, **when** the developer
  asks for change impact, **then** the result identifies those tests and shows
  the changed source, implementation, and criterion id that connect each test.
- **US-022-AC-2**: **Given** a changed source area with no bound requirement link,
  or source or specification data that the analysis cannot inspect, **when** the
  developer asks for change impact, **then** the result identifies the unlinked
  or unavailable area without claiming that no tests are affected.

## Dependencies (Contextual)

This story exercises the change-impact analysis in
[FR-099](../functional/FR-099-diff-impact-analysis.md). It traces to
[StR-007](../stakeholder/StR-007-source-grounded-assurance-data.md) because the
reviewer needs interpretable findings grounded in their source locations and
relationships.
