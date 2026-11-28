---
id: RC-001
title: "HealthAndPersonal"
type: data_classification
object: data_classification
regulated_categories: [health, personal]
---
# [RC-001] HealthAndPersonal

## Handling

- Encrypted at rest and in transit.
- Never written to application logs.

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| classification_id | UUID | 1..1 | identity |
| level | String | 1..1 | minLength: 1 |
