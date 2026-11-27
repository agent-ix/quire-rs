---
id: negative_018
title: "CustomerWithUnknownPresence"
type: entity
object: entity
expect: semantic.invalid-model-cell
because: "a Presence cell is required or optional; sometimes is neither"
---
# [negative_018] CustomerWithUnknownPresence

## Properties

| Field | Type | Multiplicity | Constraints | Presence |
|---|---|---|---|---|
| customer_id | UUID | 1..1 | identity | required |
| nickname | String | 0..1 | | sometimes |
