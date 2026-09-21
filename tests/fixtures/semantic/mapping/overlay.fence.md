---
id: FR-005
title: ConfigOverlay Entity
object: entity
type: FR
---

# FR-005: ConfigOverlay Entity

## Description

The service SHALL apply configuration overlays as `ConfigOverlay` entities layered onto a `ConfigVersion`.

## Properties

```sysml
attribute id : UUID[1] { identity }
attribute priority : Integer[1] { min: 0 }
attribute label : String[1] { nonEmpty }
attribute settings : JsonObject[1]
ref item baseVersion : ConfigVersion[0..1]
attribute appliedAt : Timestamp[1]
attribute appliedBy : String[1] { maxLength: 32 }
```

## Relationships

- `version`: references → ConfigVersion (FR-006)

## Invariants

### sealed

```ocl
context ConfigOverlay inv sealed: self.priority = self.priority@pre and self.settings = self.settings@pre and self.label = self.label@pre
```
