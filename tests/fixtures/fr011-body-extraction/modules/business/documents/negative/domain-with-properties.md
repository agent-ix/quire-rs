---
id: negative_004
title: "OrderManagementWithFields"
type: domain
object: domain
expect: semantic.record-invalid
because: "a domain declares a boundary, not data; Domain.json is sealed and admits no fields"
---
# [negative_004] OrderManagementWithFields

## Bounded Context

The context boundary this domain owns.

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| context_id | UUID | 1..1 | identity |
