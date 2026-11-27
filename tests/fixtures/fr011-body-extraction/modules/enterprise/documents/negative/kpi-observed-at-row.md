---
id: KPI-901
title: "KpiWithObservationInstant"
type: kpi
object: kpi
metric: "Orders delivered within the promised window"
target: "97% or higher"
expect: semantic.record-invalid
---
<!-- NEGATIVE fixture. A KPI declaration carrying the instant of a reading:
     `observed_at : Timestamp` is what an observation has and a definition does
     not. Kpi.json forbids every `Timestamp`-targeted field. -->
# [KPI-901] KpiWithObservationInstant

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| delivered_within_promise | Decimal(5,2) [%] | 1..1 | min: 0, max: 100 |
| observed_at | Timestamp | 1..1 | |
