---
id: FR-005
title: ConfigOverlay
object: entity
type: FR
---

# FR-005: ConfigOverlay

## Properties

| Field | Type | Multiplicity | Constraints |
|-------|------|--------------|-------------|
| id | UUID | 1 | identity |

## Relationships

| Name | Verb | Target | Multiplicity |
|------|------|--------|--------------|
| baseVersion | references | FR-006 | 1..1 |
| lineItems | contains | FR-007 | 0..* |
| priorOverlay | references | FR-005 | 0..1 |
