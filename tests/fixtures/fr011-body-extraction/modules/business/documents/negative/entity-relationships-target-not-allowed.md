---
id: negative_023
title: "CustomerContainingAnEntity"
type: entity
object: entity
expect: semantic.invalid-model-cell
because: "an entity contains nested entities and value objects; the target negative_023 is an entity"
---
# [negative_023] CustomerContainingAnEntity

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| customer_id | UUID | 1..1 | identity |

## Relationships

| Name | Verb | Target | Multiplicity |
|---|---|---|---|
| sub_account | contains | negative_023 | 0..* |
