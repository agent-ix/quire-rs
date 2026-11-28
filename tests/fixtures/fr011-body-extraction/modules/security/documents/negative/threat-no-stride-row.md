---
id: negative-004
title: "UnclassifiedThreat"
type: threat
object: threat
stride_category: Spoofing
vector: "unspecified"
expect: semantic.record-invalid
because: "Threat.json requires a stride_category row; a threat nobody classified is not a threat statement"
---
# [negative-004] UnclassifiedThreat

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| threat_id | UUID | 1..1 | identity |
| vector | String | 1..1 | minLength: 1 |
