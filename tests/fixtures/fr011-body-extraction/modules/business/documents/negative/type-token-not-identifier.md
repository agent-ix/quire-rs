---
id: negative_008
title: "CustomerWithBadTypeToken"
type: entity
object: entity
expect: semantic.invalid-type-token
because: "a Type cell holds a kernel scalar or an Identifier naming another declaration"
---
# [negative_008] CustomerWithBadTypeToken

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| customer_id | UUID | 1..1 | identity |
| shipping_address | Shipping Address | 1..1 | |
