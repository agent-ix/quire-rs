---
id: KPI-SYN-1
title: "OnTimeDeliveryRate"
type: kpi
object: kpi
metric: "Orders delivered within the promised window (% of delivered orders)"
target: "97% or higher, measured weekly"
threshold: "Alert below 95% for two consecutive weeks"
---
<!-- kpi authoring skeleton (spec-objects-enterprise). Contract (manifest
     body_extraction asserts):
     - Frontmatter MUST carry id, title, type: kpi, object: kpi, plus the prose
       fields `metric` and `target`; `threshold` is optional but recommended.
       Those strings are the human-facing form; the typed record below is
       the authority.
     - "Properties" (H2): typed rows declaring the MEASURE, not a reading of
       it. Kpi.json enforces three rules: at least one row carries a unit in
       its Type cell (`Decimal(5,2) [%]`, `Integer [1]`, `Duration [d]`), a
       dimensionless measure using the UCUM unity symbol `1`; NO row carries
       `identity`, because a definition has no per-occurrence identity; and NO
       row targets `Timestamp`, because a definition has no instant. A row
       named `observed_at` or an `observation_id` identity row is exactly the
       record this schema exists to refuse.
     - "Invariants" (H2): one `### <clauseId>` per clause with one ```ocl```
       fence.
     A KPI performs nothing (`operations` forbidden) and carries no goal
     (`targets` forbidden — that belongs to the objective that sets it). -->
# [KPI-SYN-1] OnTimeDeliveryRate

On-time delivery rate measures the share of delivered orders whose proof of
delivery falls within the delivery window promised at checkout. It is computed
weekly from carrier delivery scans joined to the promise recorded at Order
Capture, excluding orders the customer rescheduled. The KPI is owned by the
Fulfillment Platform group. A breach of the alert threshold for two consecutive
weeks triggers a carrier-performance review and pauses onboarding of new
delivery promises in the affected region.

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| delivered_within_promise | Decimal(5,2) [%] | 1..1 | min: 0, max: 100 |
| delivered_order_count | Integer [1] | 1..1 | min: 0 |
| evaluation_window | Duration [d] | 1..1 | |
| measured_capability | OrderFulfillment | 0..* unique | |

## Relationships

| Name | Verb | Target | Multiplicity |
|---|---|---|---|

## Invariants

The rules the OnTimeDeliveryRate definition carries. They constrain the
measure, never a reading of it.

### RateIsAProportion

```ocl
context OnTimeDeliveryRate
inv RateIsAProportion:
  self.delivered_within_promise >= 0 and self.delivered_within_promise <= 100
```

### WindowIsPositive

```ocl
context OnTimeDeliveryRate
inv WindowIsPositive:
  self.evaluation_window > 0
```
