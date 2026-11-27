---
id: RC-004
title: "NoCategoryStated"
type: data_classification
object: data_classification
---
# [RC-004] NoCategoryStated

## Handling

- Encrypted at rest and in transit.
- Never written to application logs.

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| classification_id | UUID | 1..1 | identity |
| level | String | 1..1 | minLength: 1 |
