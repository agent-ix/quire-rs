---
id: negative_013
title: "OrderFulfilmentWithUnknownStepKind"
type: process
object: process
expect: semantic.invalid-model-cell
because: "a step Kind is one of command, event, decision, compensation, wait; retry is none of them"
---
# [negative_013] OrderFulfilmentWithUnknownStepKind

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| order_id | UUID | 1..1 | identity |

## Workflow

| Step | Kind | Consumes | Emits | Description |
|---|---|---|---|---|
| reserve_stock | command | OrderPlaced | | Reserve stock for every line |
| reserve_again | retry | | | Try the reservation again |
