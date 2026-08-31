---
id: Task-012
title: "FR-067 — deterministic premise inventory and exporter"
type: Task
status: not_started
track: A
priority: P0
relationships:
  - target: ix://agent-ix/quire-rs/Task-011
    type: depends_on
  - target: ix://agent-ix/quire-rs/FR-067
    type: references
  - target: ix://agent-ix/quire-rs/NFR-006
    type: references
  - target: ix://agent-ix/quire-rs/TC-1085
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1086
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-1088
    type: verifies
---
# Task-012: FR-067 — deterministic premise inventory and exporter

## Scope

Construct source and active-module/schema premises from caller-selected immutable identity and a loaded Registry, failing atomically on incomplete inputs.

## Subtasks

- [ ] Write red tests for every invalid premise and deterministic ordering case.
- [ ] Expose only the minimal read-only Registry access needed for active archetypes.
- [ ] Canonicalize schema JSON and compute semantic SHA-256 digests.
- [ ] Prove source-revision isolation and byte identity.

## Deliverables

- Validated source/module/schema premise construction
- TC-1085, TC-1086, TC-1088 evidence

## Notes

- No Git or filesystem discovery; callers provide the identity and loaded Registry.
- This task unblocks Task-013.
