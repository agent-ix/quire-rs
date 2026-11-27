---
id: negative-008
title: "AssetWithBothForms"
type: asset
object: asset
expect: semantic.properties-both-forms
because: "an artifact carries one typed table or one sysml fence; the alternate form is a separate file"
---
# [negative-008] AssetWithBothForms

## Description

An asset that authors both Properties forms in one artifact.

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| asset_id | UUID | 1..1 | identity |

```sysml
attribute asset_id : UUID[1..1] { identity }
```
