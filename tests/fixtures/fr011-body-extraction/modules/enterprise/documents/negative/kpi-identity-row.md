---
id: KPI-902
title: "KpiWithObservationIdentity"
type: kpi
object: kpi
metric: "Orders delivered within the promised window"
target: "97% or higher"
expect: semantic.record-invalid
---
<!-- NEGATIVE fixture. A KPI declaration carrying the identity of a reading.
     A definition has no per-occurrence identity; an observation does.
     Kpi.json admits zero identity fields. -->
# [KPI-902] KpiWithObservationIdentity

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| observation_id | UUID | 1..1 | identity |
| delivered_within_promise | Decimal(5,2) [%] | 1..1 | min: 0, max: 100 |
