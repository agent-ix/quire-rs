---
id: negative-003
title: "SliWithoutMeasuredField"
type: sli
object: sli
expect: semantic.record-invalid
detail: '"name":"good_event_status_ceiling"'
because: "Sli.json requires at least one field whose Type cell carries a bracketed unit; without it the indicator references no typed measurement"
---
# [negative-003] SliWithoutMeasuredField

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| success_ratio | Decimal(5,4) | 1..1 | min: 0, max: 1 |
| good_event_status_ceiling | Integer | 1..1 | min: 100, max: 599 |
