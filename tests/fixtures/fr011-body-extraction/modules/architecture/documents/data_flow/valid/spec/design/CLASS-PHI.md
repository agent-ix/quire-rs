---
id: CLASS-PHI
title: "HealthData"
type: data_classification
object: data_classification
regulated_categories: [health]
---
# [CLASS-PHI] HealthData

## Handling

- Encrypted at rest and in transit.

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| classification_id | UUID | 1..1 | identity |
| level | String | 1..1 | minLength: 1 |
