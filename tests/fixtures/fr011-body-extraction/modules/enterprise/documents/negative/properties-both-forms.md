---
id: CAP-902
title: "CapabilityWithBothPropertiesForms"
type: capability
object: capability
expect: semantic.properties-both-forms
---
<!-- NEGATIVE fixture. One artifact carries one Properties form (FR-005-CON-2).
     The alternate is a separate file, never a second block here. -->
# [CAP-902] CapabilityWithBothPropertiesForms

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| capability_id | UUID | 1..1 | identity |

```sysml
attribute capability_id : UUID[1..1] { identity }
```

## Sub-capabilities

- **Something** — a child capability.
