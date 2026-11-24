---
id: US-020
title: "Adopter reviews fixed and known anomalies for a release"
type: US
relationships:
  - target: "ix://agent-ix/quire-rs/FR-097"
    type: "traces_to"
---
# US-020: Adopter reviews fixed and known anomalies for a release

## Story

**As an** adopter evaluating a Quire-rs release
**I want** release-note anomaly lists to come from the repository's validated records
**So that** I can assess the release using the supplier's current, reviewable findings.

## Context

The 2026-11-23 regulated standards review identifies an open behavioral-mutation
finding (FND-009) and reports that hardening CI is dispatch-only. No retained
behavioral-mutation result exists. Mutation runs are currently backlogged, so
the record must remain open without claiming a run passed. Release-note anomaly
lists must come from the records, not a second manually maintained summary.

## Acceptance Examples (Illustrative)

### US-020-EX-1: Known finding remains visible

- **Given** an assessed or identified Anomaly record without a fixed resolution
- **When** release notes are generated
- **Then** the finding appears in the Known Anomalies section with its current status

### US-020-EX-2: Fixed finding is identified as fixed

- **Given** an Anomaly record resolved as fixed with fix-verification evidence
- **When** release notes are generated
- **Then** the finding appears in the Fixed Anomalies section

## Dependencies (Contextual)

The Anomaly artifact lifecycle is defined by `spec-artifacts-process`. This story
does not assign anomaly severity or decide safety impact.
