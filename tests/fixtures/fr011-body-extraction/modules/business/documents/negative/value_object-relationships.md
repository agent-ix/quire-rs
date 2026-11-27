---
id: negative_025
title: "MoneyWithRelationships"
type: value_object
object: value_object
expect: semantic.record-invalid
because: "a value object record admits no relations key, so a Relationships table on it fails the ValueObject schema"
---
# [negative_025] MoneyWithRelationships

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| amount_minor | Integer | 1..1 | min: 0 |
| currency | String | 1..1 | minLength: 3, maxLength: 3 |

## Relationships

| Name | Verb | Target | Multiplicity |
|---|---|---|---|
| currency_code | references | enumeration_001 | 1..1 |
