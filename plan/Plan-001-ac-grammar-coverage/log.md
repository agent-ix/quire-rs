---
type: log
title: "Plan-001 — Update Log"
description: "Chronological log of changes to the Plan-001 bundle."
---
# Plan-001 — Update Log

## History

* **2026-08-04** — Plan created from the `spec/ac-grammar-coverage` slice
  (SR-002-reviewed, matrix 437/437); scoped to FR-047..FR-051 + US-017.
  Decomposed into 10 tasks: Track A (grammar feature, Task-001..003 + Gate
  G1), Track B (traceability feature, Task-004..008 + Gate G2), Track C
  (corpus/quality cleanups on separate branches, Task-009/010, user-gated per
  FR-047-CON-1 / FR-051-CON-3). ADR-0010/SMT excluded (Proposed, no
  decision). External dependencies recorded: `spec-artifacts-iso` traceability
  declaration, `spec-artifacts-process` FR-003, `quire-cli` wiring, companion
  marker packages (pytest plugin, Rust proc-macro crate, npm helper).
* **2026-08-04** — SR-002 FND-004 resolution (user-authorized): external
  dependencies converted to tracked entries EXT-1..EXT-5 with owning-repo
  attribution and explicit blocking relationships to gated tasks —
  EXT-1 `spec-artifacts-iso` traceability declaration (gates Task-007/008
  rollout + Task-009 defaults), EXT-2 `spec-artifacts-process` FR-003
  (gates Task-007 dogfooding), EXT-3 `quire-cli` wiring (gates Task-003/007
  CLI-level exit criteria), EXT-4a/4b/4c companion marker packages (EXT-4b
  Rust `#[trace]` crate is a hard Task-010 prerequisite). Also wired the
  FND-005 rework into Task-002: FR-047-AC-12/TC-757 (module-data
  `observable_verbs`).
* **2026-08-04** — Task-001 (Track A, branch `task/ac-grammar-severity`)
  completed: FR-048 per-check `grammar_severity` framework — manifest registry
  with key/level validation, first-wins merge + `DuplicateGrammarSeverity`,
  `Registry::grammar_severity()`, emission-time application with `off` dropped
  pre-routing, type-only all-default degradation, PyO3 parity. TC-716..719,
  TC-722, TC-723, TC-752 green; `make ci` green. `spec/tests.md` statuses stay
  🚧 until Gate G1 (TC-718's end-to-end prose needs the Task-002 `ac` grammar).
* **2026-08-04** — Task-002 (Track A) completed: the `ac` grammar
  (`src/grammar/ac.rs`) registers on the `iso-spec-core` bundle with the FR
  `Criteria`-column and `### <doc-id>-AC-N` supplement bindings, three shapes,
  and the five checks; `vague-response` shares one implementation with EARS
  via `ears::vague_verb_in_clause`; `observable_verbs` ships as a module-data
  registry over built-in defaults (ADR 0009). TC-707..713, TC-751, TC-754,
  TC-757 green in Rust; TC-715 verified against a built wheel
  (`tests/python/test_bindings.py`). `make ci` green.
* **2026-08-04** — Task-003 (Track A) completed and **Gate G1 passed**: generic
  `[<grammar>:<check>]` summary histogram (`summarize_findings`), repeatable
  `--severity` parse/merge with a usage diagnostic (`merge_severity_overrides`
  / `SeverityOverrideError`), one shared `is_severity_key` shape definition,
  `--strict` untouched. TC-714, TC-720, TC-721, TC-755 green against the engine
  API (their CLI end-to-end halves land with the EXT-3 `quire-cli` PR). G1
  dry-run baseline (340 `ac` findings over 44 FR docs) recorded in Task-009.
* **2026-08-04** — Task-004 (Track B) completed: the declarative
  `traceability:` model (`src/traceability.rs`) — trace targets incl. auxiliary
  documents, document references with capturing patterns, status vocabulary,
  and the marker/legacy trace-tag grammar — parses, shape-validates at module
  load, merges first-wins across modules, and is exposed as
  `Registry::traceability()`. ISO-shaped and non-ISO fixtures live in
  `tests/fixtures/traceability/`. TC-732, TC-733 green; `make ci` green.

