---
id: PART-DELIVERS
title: Part Deliveries
type: part
object: part
relationships:
  - target: SINK-LLM-PROMPT
    type: delivers_to
  - target: SINK-TOOL-CALL
    type: delivers_to
  - target: SINK-MESSAGE
    type: delivers_to
  - target: SINK-DATASTORE
    type: delivers_to
---
# [PART-DELIVERS] Part Deliveries

## Part

| Owner | Declared Type | Multiplicity |
|---|---|---|
| search_service | triage_processor | 1..1 |
