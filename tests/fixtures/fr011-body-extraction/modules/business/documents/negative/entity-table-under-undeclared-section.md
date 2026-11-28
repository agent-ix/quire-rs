---
id: negative_017
title: "CustomerWithUndeclaredStates"
type: entity
object: entity
expect: semantic.feature-not-extractable
because: "the entity manifest declares no States table, so a State table under any section is a form the manifest does not declare"
---
# [negative_017] CustomerWithUndeclaredStates

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| customer_id | UUID | 1..1 | identity |

## Lifecycle

| State | Description |
|---|---|
| Active | The customer may place orders |
