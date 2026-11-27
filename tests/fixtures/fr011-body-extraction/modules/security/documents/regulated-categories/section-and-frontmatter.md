---
id: RC-011
title: "SectionAndFrontmatter"
type: data_classification
object: data_classification
regulated_categories: [health]
---
# [RC-011] SectionAndFrontmatter

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
