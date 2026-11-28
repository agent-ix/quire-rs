---
id: negative_024
title: "CustomerWithBadMultiplicity"
type: entity
object: entity
expect: semantic.invalid-model-cell
because: "a Multiplicity is an integer, n..m, n..* or *; many is none of them"
---
# [negative_024] CustomerWithBadMultiplicity

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| customer_id | UUID | 1..1 | identity |

## Relationships

| Name | Verb | Target | Multiplicity |
|---|---|---|---|
| lifecycle | owns | state_machine_001 | many |
