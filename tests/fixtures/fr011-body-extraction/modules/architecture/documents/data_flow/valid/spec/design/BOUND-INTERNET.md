---
id: BOUND-INTERNET
title: "InternetBoundary"
type: trust_boundary
object: trust_boundary
trust_level: untrusted
---
# [BOUND-INTERNET] InternetBoundary

## Boundary

```mermaid
flowchart LR
  private[Protected zone] --> public[Internet]
```

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| boundary_id | UUID | 1..1 | identity |
| trust_level | String | 1..1 | minLength: 1 |

## Invariants

Every crossing leaves the protected zone.

### LeavesProtectedZone

```ocl
context InternetBoundary
inv LeavesProtectedZone:
  self.boundary_id <> ''
```
