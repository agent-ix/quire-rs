---
id: negative-005
title: "ControlWithoutEffectiveness"
type: control
object: control
expect: semantic.record-invalid
because: "Control.json requires an effectiveness row, so effectiveness is stated and never assumed"
---
# [negative-005] ControlWithoutEffectiveness

## Mappings

| Set | Clause |
|---|---|
| example.invalid/std@1 | EX-1 |

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| control_id | UUID | 1..1 | identity |
| owner | String | 1..1 | minLength: 1 |

## Invariants

### OwnerIsNamed

```ocl
context ControlWithoutEffectiveness
inv OwnerIsNamed:
  self.owner->notEmpty()
```
