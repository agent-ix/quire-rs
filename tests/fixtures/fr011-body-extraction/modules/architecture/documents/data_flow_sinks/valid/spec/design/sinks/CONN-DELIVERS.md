---
id: CONN-DELIVERS
title: Connection Deliveries
type: connection
object: connection
relationships:
  - target: SINK-UI-SCREEN
    type: delivers_to
  - target: SINK-LOG
    type: delivers_to
  - target: SINK-EXPORT
    type: delivers_to
  - target: SINK-ANALYTICS
    type: delivers_to
  - target: CLASS-PHI
    type: carries
---
# [CONN-DELIVERS] Connection Deliveries

## Connection

| Source | Source Multiplicity | Target | Target Multiplicity | Direction |
|---|---|---|---|---|
| sink_source_out | 1..1 | sink_target_in | 1..1 | source-to-target |
