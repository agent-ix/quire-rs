---
id: FR-907
title: Overlay Operations
object: entity
type: FR
---

# FR-907: Overlay Operations

## Properties

| Field | Type | Multiplicity | Constraints |
|-------|------|--------------|-------------|
| id | UUID | 1 | identity |

## Invariants

### unlocked

```quire
not (self.state = "sealed")
```

### locked

```quire
result.state = "sealed"
```

## Operations

### seal

| Param | Type | Multiplicity | Constraints |
|-------|------|--------------|-------------|
| note | String | 1 | nonEmpty |
| grace | Duration [s] | 0..1 | |

Returns: ConfigOverlay[1]
Pre: unlocked
Post: locked
