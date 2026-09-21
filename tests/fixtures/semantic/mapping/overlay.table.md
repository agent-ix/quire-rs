---
id: FR-005
title: ConfigOverlay Entity
object: entity
type: FR
---

# FR-005: ConfigOverlay Entity

## Description

The service SHALL apply configuration overlays as `ConfigOverlay` entities layered onto a `ConfigVersion`.

## Properties

| Field | Type | Multiplicity | Constraints |
|-------|------|--------------|-------------|
| id | UUID | 1 | identity |
| priority | Integer | 1 | min: 0 |
| label | String | 1 | nonEmpty |
| settings | JsonObject | 1 | |
| baseVersion | ConfigVersion | 0..1 | |
| appliedAt | Timestamp | 1 | |
| appliedBy | String | 1 | maxLength: 32 |

## Relationships

- `version`: references → ConfigVersion (FR-006)

## Invariants

### sealed

```ocl
context ConfigOverlay inv sealed: self.priority = self.priority@pre and self.settings = self.settings@pre and self.label = self.label@pre
```
