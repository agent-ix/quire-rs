---
id: Task-006
title: "FR-051 — trace binding (markers + legacy) and FR-045 records"
type: Task
status: not_started
track: B
priority: P0
relationships:
  - target: ix://agent-ix/quire-rs/Task-004
    type: depends_on
  - target: ix://agent-ix/quire-rs/Task-005
    type: depends_on
  - target: ix://agent-ix/quire-rs/FR-051
    type: references
  - target: ix://agent-ix/quire-rs/TC-744
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-745
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-746
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-747
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-748
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-750
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-753
    type: verifies
---
# Task-006: FR-051 — trace binding (markers + legacy) and FR-045 records

## Scope

Bind symbols to trace ids via the module-declared trace-tag grammar (Task-004
model): statically parse framework-native canonical markers
(`@pytest.mark.trace(...)`, `#[trace(...)]`, TS `trace(...)`) and the legacy
textual class (docstring bare id, `Trace:` line, line-comment id,
trace-embedding test name) with `legacy` provenance + mechanical
marker-rewrite suggestions. Mint `verifies` (dedup + diagnostic per FR-045),
`defined_in`, and `contains` relations; emit FR-045-shaped graph-node/edge
records with caller-supplied org/repo-normalized refs, plus the compact
in-process form for FR-050. Byte-identical repeat output.

## Subtasks
- [ ] **Marker parsing.** All three canonical forms, statically, forms taken
  from the declared grammar — zero hardcoded tag shapes (TC-744, TC-745).
- [ ] **Legacy class.** Recognition + `legacy` provenance + rewrite
  suggestions where derivable (TC-753).
- [ ] **Relations + dedup.** One relation per (symbol, trace id) + diagnostic
  on repeats (TC-746); `defined_in`/`contains`, deterministic order (TC-748).
- [ ] **Records.** FR-045 shapes, ingestion-fixture compatibility (TC-747);
  byte-identical repeat (TC-750).

## Deliverables
- Trace-binding + record emission in `src/symbols`; tests tagged TC-744..748,
  TC-750, TC-753.

## Notes
- **External:** the real pytest plugin / Rust proc-macro crate / npm helper
  are companion deliverables outside quire-rs — fixtures here declare the
  forms directly; nothing in this task blocks on those packages.
- Legacy recognition is migration-scoped: removal is Task-010's user gate
  (FR-051-CON-3) — do not remove here.
- Unblocks: Task-007; Task-010 (Track C, after G2).
