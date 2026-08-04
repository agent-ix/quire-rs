---
id: Task-003
title: "Grammar CLI-surface support — generic summary + --severity helper"
type: Task
status: not_started
track: A
priority: P1
relationships:
  - target: ix://agent-ix/quire-rs/Task-002
    type: depends_on
  - target: ix://agent-ix/quire-rs/FR-047
    type: references
  - target: ix://agent-ix/quire-rs/FR-048
    type: references
  - target: ix://agent-ix/quire-rs/TC-714
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-720
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-721
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-755
    type: verifies
---
# Task-003: Grammar CLI-surface support — generic summary + `--severity` helper

## Scope

The engine-side support for the CLI-facing halves of FR-047/FR-048: the
generic `[<grammar>:<check>]` finding prefix + a summary-grouping API that
histograms any grammar (replacing the hardcoded `[ears:` assumption), and a
`--severity` entry parser/merger (repeatable entries, CLI-over-manifest
precedence, `off` vocabulary, malformed-entry rejection with a usage
diagnostic). `--strict` keeps its global escalate-on-warning semantics.

## Subtasks
- [ ] **Generic prefix + histogram API.** Findings format `[<grammar>:<check>]`;
  grouping covers `ears` and `ac` in one corpus (TC-714).
- [ ] **`--severity` helper.** Parse `<grammar>:<check>=<level>`, repeatable,
  precedence over manifest (TC-720); malformed entry → diagnostic + non-zero
  before validation (TC-755).
- [ ] **`--strict` regression.** Unchanged exit semantics (TC-721).

## Deliverables
- Engine APIs + tests tagged TC-714, TC-720, TC-721, TC-755.

## Notes
- **External dependency:** the actual `quire validate` flag/summary wiring
  lands as a `quire-cli` PR (release-coupled repo) — out of this repo's plan;
  the engine API here is what that PR consumes. End-to-end CLI verification
  completes there.
- Closes Gate G1 together with a dry-run baseline count over this repo's spec
  (recorded into Task-009's body; no corpus edits on this branch).
