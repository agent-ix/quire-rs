---
id: positive_004
title: "CustomerAccount"
type: entity
object: entity
---
# [positive_004] CustomerAccount

A customer account with one relationship row per verb an entity admits.

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| account_id | UUID | 1..1 | identity |
| balance | Money | 1..1 | |

## Relationships

| Name | Verb | Target | Multiplicity |
|---|---|---|---|
| balance | contains | value_object_001 | 1..1 |
| lifecycle | owns | state_machine_001 | 1..1 |
| last_order | references | aggregate_root_001 | 0..1 |
| referrer | references | positive_004 | 0..1 |
| shared_terms | references | value_object_001 | 0..* |
