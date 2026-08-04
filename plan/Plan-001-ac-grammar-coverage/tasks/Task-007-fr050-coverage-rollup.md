---
id: Task-007
title: "FR-050 — coverage reconciliation + `quire coverage` report"
type: Task
status: not_started
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
- [ ] **Reconciler.** Targets × references × `verifies` relations; the engine
  knows nothing of "AC"/"TC" (TC-734..736).
- [ ] **Counts + report.** Per-group backed/total summing to bundle totals
  (TC-737); deterministic JSON (TC-738).
- [ ] **Genericity.** Non-ISO fixture model end-to-end (TC-739).
- [ ] **No-model diagnostic.** Distinct error, not an empty report (TC-740).
- [ ] **Static boundary audit.** TC-690-pattern test over `src/coverage` +
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
