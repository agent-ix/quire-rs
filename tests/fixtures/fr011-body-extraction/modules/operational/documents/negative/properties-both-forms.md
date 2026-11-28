---
id: negative-007
title: "ConfigurationWithBothForms"
type: configuration
object: configuration
expect: semantic.properties-both-forms
detail: 'a second Properties form'
because: "an artifact carries one typed table or one sysml fence; the alternate form is a separate file"
---
# [negative-007] ConfigurationWithBothForms

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| storage_root | String | 1..1 | minLength: 1 |

```sysml
attribute storage_root : String[1..1] { minLength: 1 }
```
