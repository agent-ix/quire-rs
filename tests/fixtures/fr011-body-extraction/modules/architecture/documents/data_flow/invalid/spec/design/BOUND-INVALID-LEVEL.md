---
id: BOUND-INVALID-LEVEL
title: "BoundaryWithInvalidTrustLevel"
type: trust_boundary
object: trust_boundary
trust_level: partly
---
# [BOUND-INVALID-LEVEL] BoundaryWithInvalidTrustLevel

## Boundary

```mermaid
flowchart LR
  inside[Protected zone] --> outside[Outside]
```

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| boundary_id | UUID | 1..1 | identity |
| trust_level | String | 1..1 | minLength: 1 |

## Invariants

Every crossing is addressed.

### EveryCrossingAddressed

```ocl
context BoundaryWithInvalidTrustLevel
inv EveryCrossingAddressed:
  self.boundary_id <> ''
```
