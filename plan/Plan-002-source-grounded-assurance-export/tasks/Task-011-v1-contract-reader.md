---
id: Task-011
title: "FR-067 — v1 assurance contract and reader"
type: Task
status: completed
track: A
priority: P0
relationships:
  - target: ix://agent-ix/quire-rs/FR-067
    type: references
  - target: ix://agent-ix/quire-rs/TC-1084
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1087
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1089
    type: verifies
---
# Task-011: FR-067 — v1 assurance contract and reader

## Scope

Define the public v1 types, hand-authored schema, deterministic serialization entry point, accepted-premise model, and schema-first reader.

## Subtasks

- [x] Write failing schema, reader-rejection, and compatibility-fixture tests.
- [x] Add strict v1 record types and the hand-authored draft-2020-12 schema.
- [x] Validate before deserializing and before exposing any typed record.
- [x] Pin the complete fixture and mutation behavior.

## Deliverables

- `schemas/output/assurance-v1.schema.json`
- Public v1 contract and reader API in `src/assurance.rs`
- Contract fixture and tests for TC-1084, TC-1087, TC-1089

## Notes

- No schema generation dependency.
- This task unblocks Task-012.
