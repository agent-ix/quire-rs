---
id: negative_020
title: "OrderFulfilmentWithStateList"
type: process
object: process
expect: semantic.feature-not-extractable
because: "the States section holds only the declared State table; a bullet list there is a form the manifest does not declare"
---
# [negative_020] OrderFulfilmentWithStateList

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| correlation_id | UUID | 1..1 | identity |

## Workflow

| Step | Kind | Consumes | Emits | Description |
|---|---|---|---|---|
| receive_order | event | OrderPlaced | | Start a run keyed by a fresh correlation id |

## States

- Reserving
- Capturing
