---
id: CONN-WRONG-DELIVERY
title: Wrong Delivery Target Connection
type: connection
object: connection
relationships:
  - target: PART-DELIVERY-TARGET
    type: delivers_to
---
# [CONN-WRONG-DELIVERY] Wrong Delivery Target Connection

## Connection

| Source | Source Multiplicity | Target | Target Multiplicity | Direction |
|---|---|---|---|---|
| wrong_delivery_out | 1..1 | wrong_delivery_in | 1..1 | source-to-target |
