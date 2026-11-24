---
id: FR-097
title: "Maintain first-party anomaly records for release reporting"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/US-020"
    type: "satisfies"
  - target: "ix://agent-ix/quire-rs/NFR-010"
    type: "requires"
---
# FR-097: Maintain first-party anomaly records for release reporting

Quire-rs owns the first-party Anomaly records for findings in this repository.
Quoin owns selection and rendering of those records
in a dossier or release report; this requirement does not add a second local
release-note generator.

## Description

The Quire-rs repository SHALL retain one validated `Anomaly` artifact under
`spec/anomalies/` for each applicable open finding from the 2026-11-23 review.
These records SHALL be the source data supplied to Quoin's anomaly report and
release-note generation, as specified by [Quoin FR-136](ix://agent-ix/quoin/FR-136)
and [Quoin FR-162](ix://agent-ix/quoin/FR-162) through
[Quoin FR-164](ix://agent-ix/quoin/FR-164). Quire-rs SHALL preserve authored
anomaly state without deriving severity, safety impact, or a release decision.

## Inputs

- The Quire-rs `Anomaly` artifacts under `spec/anomalies/`.
- The finding identity and review evidence that support each record.

## Outputs

- Validated first-party anomaly records available for Quoin's report and
  release-note generation.

## Behavior

- Every open finding assigned to Quire-rs in the 2026-11-23 regulated standards
  review SHALL have a distinct Anomaly artifact with a stable ID, title,
  observed behavior, and source finding reference.
- An Anomaly without a human assessment SHALL use `status: identified` and
  `safety_impact: unassessed`; the record SHALL omit severity until a person
  assesses it.
- A record SHALL move to `resolved` with `resolution: fixed` only after its
  required Fix Verification evidence is authored. A ticket merge or scheduled
  work does not itself establish this resolution.
- Quire-rs maintainers SHALL place records under `spec/anomalies/`, where Quoin
  can validate and consume the repository's source artifacts.
- This repository SHALL NOT assign anomaly severity or safety impact, infer
  that no anomalies exist, or promote a finding to fixed.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-097-CON-1 | Quoin dossier and release reporting SHALL consume the validated Anomaly records as its source data; a separately maintained Quire-rs anomaly list is not authoritative. | Data Integrity | Inspection |
| FR-097-CON-2 | This repository SHALL preserve human-authored severity, safety impact, status, resolution, and release rationale without assigning or revising them. | Human Authority | Inspection |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-097-AC-1 | Every applicable open Quire-rs finding from the 2026-11-23 review has exactly one Anomaly artifact under `spec/anomalies/`, with a unique ID and a source finding reference. | Inspection |
| FR-097-AC-2 | The FND-009 record identifies the mutation workflow as dispatch-only, states that no behavioral-mutation run has been executed or passed, and remains `identified` while the run is backlogged. | Inspection |
| FR-097-AC-4 | Each record without a human assessment has `status: identified`, `safety_impact: unassessed`, and no severity value. | Test |
| FR-097-AC-5 | A record resolved as fixed includes the Anomaly archetype's required Fix Verification section; changing a ticket state or merging code alone does not satisfy the record. | Test |
| FR-097-AC-6 | Quoin can validate and consume all Quire-rs anomaly records without a duplicate manual fixed/known list in this repository. | Test |

## Dependencies

- **Upstream**: the `Anomaly` artifact lifecycle from
  [spec-artifacts-process FR-014](ix://agent-ix/spec-artifacts-process/FR-014),
  including its status-dependent assessment, resolution, and fix-verification
  sections; [Quoin FR-136](ix://agent-ix/quoin/FR-136) and
  [Quoin FR-162](ix://agent-ix/quoin/FR-162) through
  [Quoin FR-164](ix://agent-ix/quoin/FR-164) own report and release-note
  selection.

> **CR-422 note (2026-11-24):** This change makes first-party
> Quire-rs Anomaly records the source data for Quoin's shared report and
> release-note behavior. It does not add a duplicate local generator, run
> mutation tests, or alter CI dispatch policy.
