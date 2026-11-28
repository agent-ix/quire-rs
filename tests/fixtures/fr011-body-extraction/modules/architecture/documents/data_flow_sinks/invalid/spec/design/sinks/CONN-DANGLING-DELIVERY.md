---
id: CONN-DANGLING-DELIVERY
title: Dangling Delivery Connection
type: connection
object: connection
relationships:
  - target: SINK-MISSING
    type: delivers_to
---
# [CONN-DANGLING-DELIVERY] Dangling Delivery Connection

## Connection

| Source | Source Multiplicity | Target | Target Multiplicity | Direction |
|---|---|---|---|---|
| dangling_delivery_out | 1..1 | dangling_delivery_in | 1..1 | source-to-target |
