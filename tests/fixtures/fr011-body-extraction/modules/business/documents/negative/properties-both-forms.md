---
id: negative_006
title: "CustomerWithBothForms"
type: entity
object: entity
expect: semantic.properties-both-forms
because: "an artifact carries one typed table or one sysml fence; the alternate form is a separate file"
---
# [negative_006] CustomerWithBothForms

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| customer_id | UUID | 1..1 | identity |

```sysml
attribute customer_id : UUID[1..1] { identity }
```
