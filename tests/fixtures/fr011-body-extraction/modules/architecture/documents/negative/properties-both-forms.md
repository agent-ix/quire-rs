---
id: negative_008
title: "RecordWithBothForms"
type: data_schema
object: data_schema
expect: semantic.properties-both-forms
because: "an artifact carries one typed table or one sysml fence; the alternate form is a separate file"
---
# [negative_008] RecordWithBothForms

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| artifact_id | UUID | 1..1 | identity |

```sysml
attribute artifact_id : UUID[1..1] { identity }
```

## Schema

```json
{ "type": "object", "required": ["artifact_id"] }
```
