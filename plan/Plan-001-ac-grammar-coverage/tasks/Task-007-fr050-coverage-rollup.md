---
id: Task-007
title: "FR-050 — coverage reconciliation + `quire coverage` report"
type: Task
status: completed
track: B
priority: P0
relationships:
  - target: ix://agent-ix/quire-rs/Task-004
    type: depends_on
  - target: ix://agent-ix/quire-rs/Task-006
    type: depends_on
  - target: ix://agent-ix/quire-rs/FR-050
    type: references
  - target: ix://agent-ix/quire-rs/TC-734
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-735
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-736
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-737
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-738
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-739
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-740
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-756
    type: verifies
---
# Task-007: FR-050 — coverage reconciliation + `quire coverage` report

## Scope

The generic reconciliation over declared targets, declared references, and
scanned source tags: unbacked rows, status lies, untracked symbols, and
per-minting-document backed/total counts, emitted as byte-identical JSON.
Engine API takes a loaded `Spec`, a `Registry` with a declared model, and the
Task-006 symbol graph; auxiliary trace sources (tests.md-style matrices) are
harvested per the declaration. No model declared → distinct diagnostic (the
CLI exit is wired in quire-cli). Includes the TC-756 static boundary audit
covering FR-050-CON-2 and FR-051-CON-1 (no network/service I/O, no execution
of extracted code).

## Subtasks
- [x] **Reconciler.** Targets × references × `verifies` relations; the engine
  knows nothing of "AC"/"TC" (TC-734..736).
- [x] **Counts + report.** Per-group backed/total summing to bundle totals
  (TC-737); deterministic JSON (TC-738).
- [x] **Genericity.** Non-ISO fixture model end-to-end (TC-739).
- [x] **No-model diagnostic.** Distinct error, not an empty report (TC-740).
- [x] **Static boundary audit.** TC-690-pattern test over `src/coverage` +
  `src/symbols` (TC-756).

## Deliverables
- `src/coverage` (working name) + report types; tests tagged TC-734..740,
  TC-756.

## Notes
- Verdict policy (PASS/CONDITIONAL/FAIL) and SpecReview authoring stay in the
  `gap-analysis` workflow (FR-050-CON-1) — the engine emits data only.
- **External:** `quire coverage` CLI command lands in `quire-cli`; ISO model
  declaration lands in `spec-artifacts-iso`; dogfooding over process-module
  artifacts additionally awaits `spec-artifacts-process` FR-003 + its future
  ISO traceability declaration. Fixture-based tests block on none of these.
- Closes Gate G2 with the Property TCs (TC-731 in Task-008, TC-738, TC-750).

## Implementation record (2026-08-04)

- `src/coverage.rs` reconciles declared targets × declared reference rows ×
  `verifies` relations. `compute(spec, registry, graph, root)` returns a
  `CoverageReport` (unbacked rows, status lies, untracked symbols, per-group
  backed/total counts, totals) or `CoverageError::ModelUndeclared` — a distinct
  diagnostic rather than an empty report (TC-740). `to_json()` is the
  `quire coverage` stdout payload; the CLI command itself is EXT-3.
- **Backing rule (worth stating, since the AC leaves it open):** a reference
  row is answerable for its own row id *and* the ids its cell references. A
  matrix row counts as backed when a test binds the row's own id; an AC row
  counts when a test binds the TC it names. Anything else would make the ISO
  shape structurally unbackable.
- A shared `src/corpus/declared_tables.rs` scanner now serves both FR-049 and
  FR-050, so the two consumers cannot drift on which rows they see; FR-049 was
  refactored onto it.
- Determinism: every collection is sorted and deduped, groups come from a
  `BTreeMap`, and `totals` is the sum over `groups` — TC-738 asserts
  byte-identical JSON across repeated runs.
- TC-756 static boundary audit covers `src/coverage.rs`, all of
  `src/symbols/`, and the corpus scanners. `std::fs` is explicitly **allowed**
  (FR-050-CON-2 names local source trees as an input); the forbidden set is
  network/service/execution surface. Two comment rewordings were needed to keep
  the textual audit honest rather than weakening the needle list.
- TC-734..TC-740, TC-756 green (`tests/coverage_rollup.rs`); `make ci` green.
