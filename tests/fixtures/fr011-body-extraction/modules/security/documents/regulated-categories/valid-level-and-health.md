---
id: RC-012
title: "HealthAtRestrictedLevel"
type: data_classification
object: data_classification
regulated_categories: [health]
---
# [RC-012] HealthAtRestrictedLevel

## Handling

- Encrypted at rest and in transit.
- Never written to application logs.
- Handled at the restricted confidentiality level.

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| classification_id | UUID | 1..1 | identity |
| level | String | 1..1 | minLength: 1 |
