---
id: RC-005
title: "PatientNamedFields"
type: data_classification
object: data_classification
---
# [RC-005] PatientNamedFields

## Handling

- Encrypted at rest and in transit.
- Never written to application logs.

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| classification_id | UUID | 1..1 | identity |
| level | String | 1..1 | minLength: 1 |
| patient_name | String | 1..1 | minLength: 1 |
