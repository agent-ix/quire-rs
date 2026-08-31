---
id: Task-013
title: "FR-068 — authoritative source projection"
type: Task
status: completed
track: A
priority: P0
relationships:
  - target: ix://agent-ix/quire-rs/Task-012
    type: depends_on
  - target: ix://agent-ix/quire-rs/FR-068
    type: references
  - target: ix://agent-ix/quire-rs/StR-007
    type: references
  - target: ix://agent-ix/quire-rs/TC-1091
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1092
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1093
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1094
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1095
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1096
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1097
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1098
    type: verifies
---
# Task-013: FR-068 — authoritative source projection

## Scope

Project the existing corpus, obligations, source symbols, bindings, and module-declared required relations into strict, source-grounded v1 records.

## Subtasks

- [x] Write failing record, locator, relation-bijection, state, and isolation tests.
- [x] Add minimal read-only corpus/registry accessors where the public contract already names them.
- [x] Preserve authoritative record values and stable identity tuples.
- [x] Emit explicit availability/freshness states without assurance verdicts.

## Deliverables

- Artifact, obligation, symbol, relation, and observation projections
- TC-1091..TC-1098 evidence

## Notes

- Do not parse frontmatter, query Markdown, or harvest source tags in this module.
- This task unblocks Task-014.
