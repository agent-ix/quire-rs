---
id: negative_001
title: "MoneyWithIdentity"
type: value_object
object: value_object
expect: semantic.record-invalid
because: "a value object has no identity of its own; ValueObject.json admits 0 identity fields"
---
# [negative_001] MoneyWithIdentity

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| money_id | UUID | 1..1 | identity |
| amount | Decimal(19,2) | 1..1 | |
