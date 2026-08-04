---
id: Task-001
title: "FR-048 — per-check grammar severity framework"
type: Task
status: not_started
track: A
priority: P0
relationships:
  - target: ix://agent-ix/quire-rs/FR-048
    type: references
  - target: ix://agent-ix/quire-rs/TC-716
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-717
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-718
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-719
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-722
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-723
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-752
    type: verifies
---
# Task-001: FR-048 — per-check grammar severity framework

## Scope

The engine half of FR-048: a `grammar_severity` registry in `manifest.yaml`
(`<grammar>:<check>` → `off`|`warning`|`error`), merged first-wins across
modules with a `DuplicateGrammarSeverity` diagnostic (mirror the FR-043
`lexicon` merge), exposed via `Registry::grammar_severity()`, applied at
finding-emission time (absent key → `warning`; `off` → dropped before routing,
absent from `--summary` input), with the type-only `validate_document` path on
the all-default map. `--strict` global semantics untouched.

## Subtasks
- [ ] **Manifest schema + loader.** Parse/validate `grammar_severity`;
  malformed entry (unknown level, non-string key) fails module load (TC-723).
- [ ] **First-wins merge + diagnostic.** Cross-module merge and
  `DuplicateGrammarSeverity` on conflicting redeclaration only (TC-717).
- [ ] **Registry accessor.** `grammar_severity()` returns the merged map (TC-716).
- [ ] **Severity application.** Key findings by `grammar`+`check`; default
  `warning`; route per FR-042-AC-7; `off` drops pre-routing (TC-718/719/752).
- [ ] **Type-only degradation.** All-default map on `validate_document` (TC-722).

## Deliverables
- Severity map types + merge in the registry/loader modules; emission-time
  application in `src/grammar`; unit tests tagged TC-716..719, TC-722,
  TC-723, TC-752.

## Notes
- First on Track A: FR-047's routing contract builds on this.
- Deterministic map iteration (NFR-006): `BTreeMap`, not `HashMap`.
- Unblocks: Task-002 (ac grammar routing), Task-003 (CLI `--severity` helper).
