---
id: OBJ-901
title: "ObjectiveWithoutAHorizon"
type: objective
object: objective
metric: "Median order-to-ship time (hours)"
target: "12 hours or less"
expect: semantic.record-invalid
---
<!-- NEGATIVE fixture. An objective with no `Timestamp` row: nothing says by
     when. Objective.json requires at least one temporal field. -->
# [OBJ-901] ObjectiveWithoutAHorizon

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| objective_id | UUID | 1..1 | identity |
| target_hours | Integer [h] | 1..1 | min: 0 |

## Invariants

### TargetIsPositive

```ocl
context ObjectiveWithoutAHorizon
inv TargetIsPositive:
  self.target_hours >= 0
```
