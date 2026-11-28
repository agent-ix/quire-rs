---
id: negative_002
title: "RecordThatDeclaresBehaviour"
type: data_schema
object: data_schema
expect: semantic.record-invalid
because: "DataSchema.json forbids `operations`: a record declares data, not behaviour"
---
# [negative_002] RecordThatDeclaresBehaviour

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| artifact_id | UUID | 1..1 | identity |

## Schema

```json
{ "type": "object", "required": ["artifact_id"] }
```

## Operations

### recompute_digest

Recompute the record's digest — behaviour a data schema may not declare.

Returns: String[1..1]
