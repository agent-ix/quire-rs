---
id: Task-014
title: "FR-067/FR-068 — boundary and compatibility gates"
type: Task
status: completed
track: B
priority: P0
relationships:
  - target: ix://agent-ix/quire-rs/Task-013
    type: depends_on
  - target: ix://agent-ix/quire-rs/FR-067
    type: references
  - target: ix://agent-ix/quire-rs/FR-068
    type: references
  - target: ix://agent-ix/quire-rs/IT-001
    type: references
  - target: ix://agent-ix/quire-rs/TC-1090
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1099
    type: verifies
---
# Task-014: FR-067/FR-068 — boundary and compatibility gates

## Scope

Protect the architectural boundary and prove the new export does not mutate existing output contracts.

## Subtasks

- [x] Add a static source-dependency audit for forbidden parse, command, network, and persistence calls.
- [x] Extend output-contract tests to keep coverage-v1/properties-v1 version-free.
- [x] Run formatting, lint, full tests, supply-chain, static audits, and spec validation; record the pre-existing platform-specific full-suite failure separately.

## Deliverables

- `tests/assurance_boundary.rs`
- Legacy-output regression coverage and full local gate evidence

## Notes

- The Quoin side of IT-001 is downstream; this task supplies the pinned producer schema and fixture it consumes.
