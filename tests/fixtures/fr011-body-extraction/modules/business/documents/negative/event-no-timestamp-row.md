---
id: negative_003
title: "OrderPlacedWithoutOccurrence"
type: event
object: event
expect: semantic.record-invalid
because: "Event.json requires at least one occurrence field whose type targets Timestamp"
---
# [negative_003] OrderPlacedWithoutOccurrence

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| order_id | UUID | 1..1 | |
| line_count | Integer | 1..1 | min: 1 |

## Schema

```json
{ "type": "object" }
```
