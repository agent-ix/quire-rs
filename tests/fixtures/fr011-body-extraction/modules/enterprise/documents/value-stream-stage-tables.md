---
id: VS-SYN-1
title: Store replenishment
type: value_stream
object: value_stream
---
# Store replenishment

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| stream_id | UUID | 1..1 | identity |

## Stages

| Stage ID | Order | Name | Value |
|---|---:|---|---|
| stage_detect | 1 | Detect demand | Identify stock below the synthetic threshold |
| stage_replenish | 2 | Replenish stock | Transfer stock to the store |

## Stage Capability Bindings

| Stage ID | Capability | Process |
|---|---|---|
| stage_replenish | Inventory planning | Restock inventory |

## Stage Outcomes

| Stage ID | Outcome ID | Name | Description |
|---|---|---|---|
| stage_detect | complete | Demand identified | Demand is below the synthetic threshold |
| stage_replenish | complete | Stock received | The store has replenished stock |

## Invariants

### HasStages

```quire
present(self.stages)
```
