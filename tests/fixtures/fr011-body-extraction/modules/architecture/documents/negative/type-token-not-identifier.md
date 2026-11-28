---
id: negative_010
title: "RecordWithBadTypeToken"
type: data_schema
object: data_schema
expect: semantic.invalid-type-token
because: "a Type cell holds a kernel scalar or an Identifier naming another declaration"
---
# [negative_010] RecordWithBadTypeToken

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| artifact_id | UUID | 1..1 | identity |
| stored_at | Object Store Location | 1..1 | |

## Schema

```json
{ "type": "object", "required": ["artifact_id", "stored_at"] }
```
