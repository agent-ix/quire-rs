---
id: positive_001
title: "ReturnLifecycle"
type: state_machine
object: state_machine
relationships:
  - type: specializes
    target: OrderLifecycle
abstract: true
---
# [positive_001] ReturnLifecycle

A return reuses the order lifecycle's context and declares its own states,
transitions, and operation frames. An exchange replaces the returned order
lines with a new order, which publishes OrderPlaced.

## Properties

| Field | Type | Multiplicity | Constraints | Presence | Subsets | Redefines |
|---|---|---|---|---|---|---|
| order_id | UUID | 1..1 | identity | required | | |
| items | String | 0..* | | | | |
| returned_items | String | 0..* | | | items | |
| current_state | ReturnStatus | 1..1 | | required | | current_state |
| reason | String | 0..1 | | optional | | |
| exchanged | Boolean | 1..1 | | required | | |

## Invariants

### ReturnedItemsAreAtMostTheItems

```quire
size(self.returned_items) <= size(self.items)
```

### ExchangedReturnHasReturnedItems

```quire
self.exchanged implies size(self.returned_items) >= 1
```

## Operations

### exchange

Replace the returned lines with a new order.

Pre: ReturnedItemsAreAtMostTheItems
Post: ExchangedReturnHasReturnedItems
Modifies: self.current_state, self.returned_items, self.exchanged
Creates: Order
Deletes: OrderLine

## States

| State | Description |
|---|---|
| Requested | The customer asked to return items |
| Exchanged | The returned lines were replaced by a new order |

## Transitions

| From | To | Trigger | Guard | Emits |
|---|---|---|---|---|
| Requested | Exchanged | exchange | ReturnedItemsAreAtMostTheItems | OrderPlaced |
