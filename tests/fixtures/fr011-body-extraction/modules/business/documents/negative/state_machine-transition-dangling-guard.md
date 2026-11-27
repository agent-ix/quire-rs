---
id: negative_012
title: "OrderLifecycleWithDanglingGuard"
type: state_machine
object: state_machine
expect: semantic.dangling-clause-ref
because: "a transition Guard names an invariant clause of the same artifact; NoSuchClause is not one"
---
# [negative_012] OrderLifecycleWithDanglingGuard

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

| State | Description |
|---|---|
| Draft | The order is being assembled |
| Placed | The customer committed to the order |

## Transitions

| From | To | Trigger | Guard | Emits |
|---|---|---|---|---|
| Draft | Placed | place | NoSuchClause | |
