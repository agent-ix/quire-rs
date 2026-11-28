---
id: KPI-903
title: "KpiWithoutAMeasure"
type: kpi
object: kpi
metric: "Something about deliveries"
target: "Better than now"
expect: semantic.record-invalid
---
<!-- NEGATIVE fixture. No row carries a unit, so nothing says what is measured
     or in what. Kpi.json requires at least one unit-bearing field; a
     dimensionless measure uses the UCUM unity symbol `1`. -->
# [KPI-903] KpiWithoutAMeasure

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| delivered_within_promise | Decimal(5,2) | 1..1 | min: 0, max: 100 |
| note | String | 1..1 | minLength: 1 |
