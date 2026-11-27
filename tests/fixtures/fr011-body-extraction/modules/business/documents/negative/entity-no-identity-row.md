---
id: negative_002
title: "CustomerWithoutIdentity"
type: entity
object: entity
expect: semantic.record-invalid
because: "Entity.json requires at least one identity field"
---
# [negative_002] CustomerWithoutIdentity

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| email | String | 1..1 | minLength: 3 |
| display_name | String | 1..1 | |
