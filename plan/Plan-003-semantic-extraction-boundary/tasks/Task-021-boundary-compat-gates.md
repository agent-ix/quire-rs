---
id: Task-021
title: "NFR-021 — boundary and compatibility gates"
type: Task
status: completed
track: B
priority: P0
relationships:
  - target: ix://agent-ix/quire-rs/Task-019
    type: depends_on
  - target: ix://agent-ix/quire-rs/NFR-021
    type: references
  - target: ix://agent-ix/quire-rs/FR-070
    type: references
  - target: ix://agent-ix/quire-rs/FR-071
    type: references
  - target: ix://agent-ix/quire-rs/FR-072
    type: references
  - target: ix://agent-ix/quire-rs/TC-1619
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1620
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1627
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1628
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1636
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1640
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1641
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1642
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1649
    type: verifies
---
# Task-021: NFR-021 — boundary and compatibility gates

## Scope

Prove the boundary: no clause parsing, no network/git/persistence, wasm-safe build, parser golden unchanged, and the external WASM leg recorded by reference.

## Subtasks

- [x] Static audits over `cargo metadata` and `src/semantic/` (denylist, forbidden symbols, net/process/fs-write, brace/pattern parsers, rendering/codegen) as `tests/semantic_boundary.rs` + `scripts/audits/`.
- [x] `make ci`: add `cargo check --target wasm32-unknown-unknown --no-default-features --features wasm` (TC-1649); install the target in CI.
- [x] Parser golden comparison for TC-1628 and span agreement with the `code_block` scanner on every fixture.
- [x] Record TC-1636 as external with the quire-wasm#3 link in the matrix row.

## Deliverables

- `tests/semantic_boundary.rs`, Makefile and `.github/workflows` change, matrix status updates

## Notes

- Workflow file pushes need SSH (PAT lacks workflow scope); this session pushes over HTTPS, so the CI change may need Peter's push.
