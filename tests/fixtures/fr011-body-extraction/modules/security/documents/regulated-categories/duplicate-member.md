---
id: RC-008
title: "DuplicateMember"
type: data_classification
object: data_classification
regulated_categories: [health, health]
---
# [RC-008] DuplicateMember

## Handling

- Encrypted at rest and in transit.
- Never written to application logs.

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| classification_id | UUID | 1..1 | identity |
| level | String | 1..1 | minLength: 1 |
