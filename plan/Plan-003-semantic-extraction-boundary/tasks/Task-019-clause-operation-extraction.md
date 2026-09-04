---
id: Task-019
title: "FR-071 — clause and operation extraction"
type: Task
status: completed
track: A
priority: P0
relationships:
  - target: ix://agent-ix/quire-rs/Task-018
    type: depends_on
  - target: ix://agent-ix/quire-rs/FR-071
    type: references
  - target: ix://agent-ix/quire-rs/TC-1622
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1623
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1624
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1625
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1626
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1629
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1648
    type: verifies
---
# Task-019: FR-071 — clause and operation extraction

## Scope

Extract `## Invariants` and `## Operations` into `ClauseRef[]`, `clauseText`, and `OperationDecl[]` with spans from the shared fence scanner and no parsing of fence bodies.

## Subtasks

- [x] Factor the `code_block` locator's fence recognition into a shared scanner returning (open line, close line, close length, body bytes) without changing `parse_document`.
- [x] `src/semantic/clauses.rs`: heading ownership, language rules, duplicate/authority/external cases, unterminated fences, spans and `clauseText`, `sourceIdentity` default advisory.
- [x] Operations: heading, param table via FR-070 cells, `Returns`, `Pre`/`Post` lists, dangling and duplicate diagnostics; whole-kind unavailability on entry errors.
- [x] Property test for fence bodies (TC-1629).

## Deliverables

- `src/semantic/clauses.rs`, scanner refactor in `src/extract/locator.rs`, `tests/semantic_clauses.rs`

## Notes

- `parse_document` output must stay byte-identical (TC-1628 in Task-021 checks the parser golden).
