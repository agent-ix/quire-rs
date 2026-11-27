---
id: RC-009
title: "ScalarValue"
type: data_classification
object: data_classification
regulated_categories: health
---
# [RC-009] ScalarValue

## Handling

- Encrypted at rest and in transit.
- Never written to application logs.

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| classification_id | UUID | 1..1 | identity |
| level | String | 1..1 | minLength: 1 |
