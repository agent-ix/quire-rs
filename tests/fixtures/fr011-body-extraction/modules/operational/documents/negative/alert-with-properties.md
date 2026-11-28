---
id: negative-004
title: "AlertWithProperties"
type: alert
object: alert
expect: semantic.record-invalid
detail: 'at fields: {} is not allowed for'
because: "Alert.json forbids fields through its seal; an alert declares a firing condition, not data"
---
# [negative-004] AlertWithProperties

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| burn_rate | Decimal(5,4) [1] | 1..1 | min: 0 |

## Invariants

### FastBurnPages

```ocl
context AlertWithProperties
inv FastBurnPages:
  self.burn_rate > 14
```
