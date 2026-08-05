---
id: Task-006
title: "FR-051 — trace binding (markers + legacy) and FR-045 records"
type: Task
status: completed
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
- [x] **Marker parsing.** All three canonical forms, statically, forms taken
  from the declared grammar — zero hardcoded tag shapes (TC-744, TC-745).
- [x] **Legacy class.** Recognition + `legacy` provenance + rewrite
  suggestions where derivable (TC-753).
- [x] **Relations + dedup.** One relation per (symbol, trace id) + diagnostic
  on repeats (TC-746); `defined_in`/`contains`, deterministic order (TC-748).
- [x] **Records.** FR-045 shapes, ingestion-fixture compatibility (TC-747);
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

## Implementation record (2026-08-04)

- `src/symbols/trace.rs` binds symbols to trace ids from the module-declared
  grammar only. Binding is restricted to **test symbols**: FR-051 attaches
  markers to the test symbol, and binding containers would let a Rust
  `mod tests` block inherit every marker nested inside it.
- Canonical markers are matched over the symbol's attached span (annotation
  block + declaration + body), statically — nothing is imported, built, or run.
  Marker ids come from the quoted arguments in capture group 1; an unquoted
  argument list is taken whole so a bare-identifier form still binds.
- Dedup: attachments are collected per trace id, a canonical marker wins over a
  legacy form, and an id attached more than once mints **one** relation and
  **one** diagnostic naming every form that attached it (the first cut emitted
  one diagnostic per extra attachment, which over-reports against AC-6).
- Two small model additions were needed and are documented in
  `src/traceability.rs`: `TraceLegacyForm::id_format` (rebuild `TC-741` from a
  `tc741_extracts` test name — otherwise that legacy form cannot yield an id at
  all) and `TraceMarkerForm::template` (what makes a rewrite suggestion
  "derivable" per FR-051-AC-11; without a template no suggestion is emitted).
- Records: `graph_records` emits FR-045 `CoreGraphNodeRef`/`CoreGraphEdgeRef`
  values with `ref`s normalized under the caller's org/repo, symbol nodes typed
  `source_symbol`, and `verifies`/`defined_in`/`contains` edges. Ids are stable
  digests and ordering is total, so repeated emission is byte-identical.
- TC-744..748, TC-750, TC-753 green; `make ci` green.
