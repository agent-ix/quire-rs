---
id: order_schema
title: "OrderRecord"
type: data_schema
object: data_schema
relationships:
  - target: "ix://agent-ix/design-fixture/FR-004"
    type: "realizes_requirement"
---
# [order_schema] OrderRecord

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| order_id | UUID | 1..1 | identity |

## Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "OrderRecord",
  "type": "object",
  "required": ["order_id"],
  "properties": { "order_id": { "type": "string", "format": "uuid" } },
  "additionalProperties": false
}
```
