---
id: negative_009
title: "OrderLifecycleWithStateDiagram"
type: state_machine
object: state_machine
expect: semantic.feature-not-extractable
because: "the States section holds only the declared State table; a mermaid diagram there is a form the manifest does not declare"
---
# [negative_009] OrderLifecycleWithStateDiagram

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| current_state | OrderStatus | 1..1 | |
| placed_at | Timestamp | 0..1 | |

## Invariants

### DraftOrderHasNoPlacementTime

```quire
self.current_state = OrderManagement::OrderStatus::Draft implies not present(self.placed_at)
```

## Operations

### place

Convert a draft order into a binding purchase request.

## States

```mermaid
stateDiagram-v2
    [*] --> Draft
    Draft --> Placed: place
```

## Transitions

| From | To | Trigger | Guard | Emits |
|---|---|---|---|---|
| Draft | Placed | place | | OrderPlaced |
