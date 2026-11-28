---
id: CAP-903
title: "CapabilityWithABadTypeToken"
type: capability
object: capability
expect: semantic.invalid-type-token
---
<!-- NEGATIVE fixture. A `Type` cell whose token is not an `Identifier`. -->
# [CAP-903] CapabilityWithABadTypeToken

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| capability_id | UUID | 1..1 | identity |
| broken | 9NotAnIdentifier | 1..1 | |

## Sub-capabilities

- **Something** — a child capability.
