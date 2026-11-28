---
id: CAP-901
title: "CapabilityWithoutIdentity"
type: capability
object: capability
expect: semantic.record-invalid
---
<!-- NEGATIVE fixture. A capability is a node in the architecture graph, so its
     declaration carries at least one identity field. Capability.json requires
     one. -->
# [CAP-901] CapabilityWithoutIdentity

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| name | String | 1..1 | minLength: 1 |

## Sub-capabilities

- **Something** — a child capability with no parent identity to hang from.
