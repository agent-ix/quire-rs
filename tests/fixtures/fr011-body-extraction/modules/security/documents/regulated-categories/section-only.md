---
id: RC-010
title: "SectionOnly"
type: data_classification
object: data_classification
---
# [RC-010] SectionOnly

## Handling

- Encrypted at rest and in transit.
- Never written to application logs.

## Regulated Categories

- health

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| classification_id | UUID | 1..1 | identity |
| level | String | 1..1 | minLength: 1 |
