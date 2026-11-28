---
id: PRIN-901
title: "PrincipleThatDeclaresData"
type: principle
object: principle
expect: semantic.record-invalid
---
<!-- NEGATIVE fixture. A principle declares a rule, not data. Principle.json
     forbids `fields`, so a `## Properties` table is refused by the seal. -->
# [PRIN-901] PrincipleThatDeclaresData

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| principle_id | UUID | 1..1 | identity |

## Rationale

A principle that declares its own data is indistinguishable from an entity, and
the graph loses the reason the type exists.

## Invariants

### RuleIsStated

```ocl
context PrincipleThatDeclaresData
inv RuleIsStated:
  self.principle_id->notEmpty()
```
