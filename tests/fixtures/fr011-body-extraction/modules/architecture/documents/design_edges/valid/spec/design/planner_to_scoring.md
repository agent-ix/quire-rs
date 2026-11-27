---
id: planner_to_scoring
title: "PlannerToScoringConnection"
type: connection
object: connection
relationships:
  - target: "ix://agent-ix/design-fixture/FR-002"
    type: "realizes_requirement"
---
# [planner_to_scoring] PlannerToScoringConnection

## Connection

| Source | Source Multiplicity | Target | Target Multiplicity | Direction |
|---|---|---|---|---|
| planner_query_out | 1..1 | score_query_in | 1..1 | source-to-target |
