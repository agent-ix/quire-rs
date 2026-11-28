---
id: TM-1
title: Threat model
type: threat_model
object: threat_model
method: stride_per_element
reviewed_by: Person
relationships:
  - type: scopes
    target: CONN-1
  - type: scopes
    target: SINK-1
  - type: considers
    target: THREAT-1
---
# Threat model

## Scope
The system.

## Assumptions
None.

## Out of scope
External systems.

## Properties
| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| threat_model_id | UUID | 1..1 | identity |
