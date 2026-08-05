---
id: Plan-001
title: "quire-rs — AC grammar + declarative traceability coverage (FR-047..FR-051)"
type: Plan
status: active
relationships:
  - target: ix://agent-ix/quire-rs/FR-047
    type: references
  - target: ix://agent-ix/quire-rs/FR-048
    type: references
  - target: ix://agent-ix/quire-rs/FR-049
    type: references
  - target: ix://agent-ix/quire-rs/FR-050
    type: references
  - target: ix://agent-ix/quire-rs/FR-051
    type: references
  - target: ix://agent-ix/quire-rs/US-017
    type: references
---
# Implementation Plan: AC grammar + declarative traceability coverage

TDD plan for implementing FR-047..FR-051 (spec branch `spec/ac-grammar-coverage`,
reviewed in SR-002, matrix 438/438). **Out of scope by instruction:** ADR-0010
(SMT consistency analysis) — the ADR is `Proposed` with no decision; no
`quire-analyze` work appears in this plan.

The plan isolates **two work classes that never share a branch or a task**:

- **Feature implementation** (Tracks A and B) — engine code for FR-047..051.
- **Corpus/quality cleanups** (Track C) — baseline sweeps and legacy trace-tag
  migration that the new checks *surface*. These run on their own branches,
  after the feature gates, and their enforcement/removal steps are user-gated
  (FR-047-CON-1, FR-051-CON-3).

## Requirements Summary

### Functional Requirements
- [ ] **FR-047**: Acceptance-criteria grammar `ac` on the FR-042 framework —
  EARS-canonical shape, GWT recognized-but-steered, unclassifiable /
  non-singular / vague-response / no-observable-outcome / non-canonical-shape
  checks, supplement-section skip rules, generic `[<grammar>:<check>]` summary,
  PyO3 parity. (AC-1..11, CON-1)
- [ ] **FR-048**: Per-check grammar severity — `grammar_severity` manifest
  registry (`off`|`warning`|`error`), first-wins merge + diagnostic,
  `--severity` CLI override incl. repeatable form and malformed-entry
  rejection, type-only all-default path, `--strict` unchanged. (AC-1..10)
- [ ] **FR-049**: Verification-reference integrity — model-driven
  `dangling-trace-reference` in `validate_bundle`, posture-degradable,
  auxiliary trace-source harvest. (AC-1..8)
- [ ] **FR-050**: Declarative coverage computation — `traceability:` manifest
  model, generic reconciliation (unbacked rows, status lies, untracked
  symbols, per-group counts), `quire coverage` JSON report, byte-identical.
  (AC-1..9, CON-1..2)
- [ ] **FR-051**: Source symbol extraction with relations — Rust/Python/TS
  syntax-level adapters, stable identities, framework-native markers as the
  canonical trace form, legacy textual class with provenance + rewrite
  suggestions, FR-045-shaped records. (AC-1..11, CON-1..3)

### User Stories
- [ ] **US-017**: Agent verifies requirement coverage deterministically
  (exercises FR-049/050/051; illustrative examples only).

### Cross-cutting NFRs (existing, apply to all new code)
- **NFR-006** determinism: `BTreeMap`/`IndexMap` where iteration order is
  observable (`src/parser`, `src/extract`, plus all new report/record output);
  Property TCs TC-731/738/750.
- **NFR-003/NFR-020 purity posture**: no network/service I/O, no execution of
  extracted code (FR-050-CON-2, FR-051-CON-1, backed by TC-756).

## Dependency Graph

### Core dependency edges
- `FR-042/043/044 (shipped) -> FR-047`
  Reason: the `ac` grammar registers on the FR-042 framework and reuses the
  FR-043/044 merged lexicon for `vague-response`.
- `FR-048 -> FR-047 (rollout)`
  Reason: FR-047 ships advisory-at-most with per-check suppression/promotion;
  the severity map (FR-048) is the mechanism. FR-047's classifier does not
  need FR-048 to *run*, but its finding-routing contract (TC-713, TC-718)
  lands cleanly only after the severity map exists — build FR-048 first.
- `FR-050 (model loading) -> FR-051 (marker binding)`
  Reason: trace-tag/marker forms are module data declared in the
  `traceability:` model; the extractor carries no hardcoded tag forms.
- `FR-051 (symbol graph) -> FR-050 (reconciliation)`
  Reason: the rollup consumes `verifies` relations from the extractor.
  Note the split: FR-050's *model loading* is upstream of FR-051, FR-050's
  *reconciliation* is downstream — decompose FR-050 accordingly.
- `FR-050 (model: document-reference declarations) -> FR-049`
  Reason: FR-049 resolves references declared by the same model; it needs the
  loader + declaration types, not the rollup.
- `FR-038 (shipped) -> FR-049`
  Reason: findings ride `validate_bundle` postures (`Strict`/`Okf`).

### Shared dependencies (extract first)
- **`traceability:` model loader + declaration types** (FR-050-AC-1/2) is
  consumed by three downstream items (FR-051 marker binding, FR-050
  reconciliation, FR-049 reference integrity). It is Task-004, first on
  Track B.
- **Statement/outcome-clause machinery** (FR-042's segmenter, clause split,
  object-aware vague check) is reused by FR-047 — extend in place, do not
  fork.

### Cross-cutting constraints
- `NFR-006` applies to: `ac` finding order, coverage report JSON, symbol/
  relation record order (TC-731/738/750 are Property tests).
- `FR-050-CON-2`/`FR-051-CON-1` purity applies to `src/coverage`/`src/symbols`
  (working module names) — enforced by the TC-756 static audit.

## External Dependencies (tracked; NOT tasks in this repo)

Tracked entries per SR-002 FND-004 — each has an owning repo and explicit
blocking relationships to the tasks it gates. Do not close a gated task's
"end-to-end" exit criterion while its EXT entry is `open`.

| ID | Deliverable | Owning repo | Status | Gates (blocking relationship) |
|---|---|---|---|---|
| EXT-1 | ISO `traceability:` model declaration (+ `ac`/`grammar_severity` ISO defaults) | `agent-ix/spec-artifacts-iso` | open | Real-world activation of Task-007/Task-008 output on ISO repos, and Task-009's sweep over ISO-declared severity defaults. Engine tasks run on fixture modules — not blocked for implementation, blocked for rollout. |
| EXT-2 | `FR-003` master-requirements surface + future ISO traceability declaration | `agent-ix/spec-artifacts-process` | open | Task-007 dogfooding of `quire coverage` over process-module artifacts (matrix/plan docs). Fixture-based TCs not blocked. |
| EXT-3 | CLI wiring: `--severity` flag, generic `--summary` parser, `quire coverage` command | `agent-ix/quire-cli` (release-coupled) | open | End-to-end runs of TC-714/720/721/755/740: Task-003 and Task-007 deliver the engine APIs, their CLI-level exit criteria complete only with the quire-cli PR. |
| EXT-4a | pytest plugin registering the `trace` marker | companion package (repo TBD, owner: user decision) | open | Python-side migrations in downstream repos; no quire-rs task blocked (fixtures declare forms statically). |
| EXT-4b | Rust no-op `#[trace]` proc-macro support crate | companion crate (repo TBD, owner: user decision) | open | **Hard prerequisite of Task-010** (this repo's legacy-tag migration rewrites Rust tests to `#[trace(...)]`). Task-010 stops if unpublished. |
| EXT-4c | npm vitest/jest `trace()` helper | companion package (repo TBD, owner: user decision) | open | TS-side migrations in downstream repos; no quire-rs task blocked. |
| EXT-5 | ADR-0010 / SMT (`quire-analyze`) | excluded — ADR is Proposed, no decision | n/a | Gates nothing in this plan. |

## Test Plan

TC definitions live in `spec/tests.md` (TC-707..756, all 🚧). Grouped by module:

### Grammar (`src/grammar`) — Unit
- [ ] TC-707..711, TC-751, TC-754, TC-757 (FR-047): shape classification,
  every-cell segmentation, non-singular + pair idiom, lexicon-backed
  vague-response, no-observable-outcome with module-data `observable_verbs`
  over built-in defaults, non-canonical-shape, supplement fenced/quote skip.
- [ ] TC-712, TC-713 (FR-047): binding scope; finding fields + severity routing.
- [ ] TC-716..719, TC-722, TC-723, TC-752 (FR-048): registry load/merge/
  accessor, default-warning, type-only all-default, malformed manifest, `off`.

### Grammar — Integration (engine API + quire-cli follow-up)
- [ ] TC-714 (generic `[<grammar>:<check>]` summary), TC-715 (PyO3 parity),
  TC-720 (`--severity` override + repeatable), TC-721 (`--strict` unchanged),
  TC-755 (malformed `--severity` rejected).

### Traceability model + reference integrity (`src/corpus`) — Unit/Integration
- [ ] TC-732, TC-733 (FR-050): model load, malformed/absent.
- [ ] TC-724..730 (FR-049): resolved/dangling/posture/model-driven/aux-source/
  no-model/multi-annotation. TC-731 (Property): deterministic findings.

### Symbol extraction (`src/symbols`) — Unit
- [ ] TC-741..746, TC-749, TC-753 (FR-051): adapters, identity stability,
  test classification, canonical markers, module-data forms, dedup+diagnostic,
  per-file degradation, legacy forms + rewrite suggestions.
- [ ] TC-747, TC-748 (FR-051): FR-045 record shapes, `defined_in`/`contains`.
  TC-750 (Property): byte-identical repeat.

### Coverage rollup (`src/coverage`) — Integration/Property
- [ ] TC-734..737, TC-739, TC-740 (FR-050): unbacked rows, status lies,
  untracked symbols, per-group counts, non-ISO model, no-model diagnostic.
  TC-738 (Property): byte-identical JSON.

### Verification (constraints)
- [ ] TC-756 (Static): boundary audit — no network/service I/O, no execution
  of extracted code in the coverage/symbols modules (TC-690 pattern).
- [ ] FR-047-CON-1, FR-051-CON-3: Inspection — user-gated promotion/removal,
  recorded in Track C task exit criteria, never automatic.

## Remaining Work

### Remaining Dependency Graph
```
Track A (grammar)              Track B (traceability)
Task-001 FR-048 severity       Task-004 FR-050 model loader     Task-005 FR-051 adapters
      |                              |         \                      |
Task-002 FR-047 ac grammar           |          Task-008 FR-049       |
      |                              +------+   (needs 004 only)      |
Task-003 CLI-surface support                \                         |
      |                                      Task-006 FR-051 trace binding + records
   [Gate G1]                                 (needs 004 + 005)
      |                                              |
      |                                      Task-007 FR-050 reconciliation + report
      |                                      (needs 004 + 006)
      |                                              |
      |                                          [Gate G2]
      |                                              |
Task-009 (C) baseline sweep                  Task-010 (C) legacy trace-tag migration
(own branch, user-gated promotion)           (own branch, user-gated removal;
                                              external: Rust #[trace] crate)
```

### Track A: Grammar feature (serial; branch `task/ac-grammar-severity`)
#### A1 (Task-001): FR-048 per-check severity framework
- **Scope:** `grammar_severity` manifest registry, first-wins merge +
  `DuplicateGrammarSeverity`, `Registry::grammar_severity()`, severity
  application at finding-emission, `off` drop, type-only all-default path.
- **Difficulty:** Medium — **Exit:** TC-716..719, TC-722, TC-723, TC-752 green.

#### A2 (Task-002): FR-047 `ac` grammar
- **Scope:** `ac` grammar registration + bindings (Criteria column, AC
  supplement sections), shape classifier, five checks, supplement skip rules,
  finding shape + routing, PyO3 surface.
- **Difficulty:** Hard — **Exit:** TC-707..713, TC-751, TC-754, TC-757,
  TC-715 green.

#### A3 (Task-003): CLI-surface support (engine side)
- **Scope:** generic `[<grammar>:<check>]` finding prefix + summary-grouping
  API; `--severity` parse/merge helper (repeatable, malformed rejection)
  exposed for the CLI. The `quire-cli` PR itself is external.
- **Difficulty:** Easy — **Exit:** TC-714, TC-720, TC-721, TC-755 green
  against the engine API (CLI end-to-end lands with the quire-cli PR).

#### Gate G1: grammar fixture + baseline readiness — **PASSED 2026-08-04**
Track A TCs green (TC-707..723, TC-751..755, TC-757 minus the EXT-3 CLI
end-to-end halves), `make ci` green, PyO3 wheel built and `tests/python/`
green. Dry-run sweep over this repo's `spec/`: 340 `ac` findings across 44 FR
documents, recorded in Task-009 — no corpus edits. No false-positive class
found (sampled); the dominant `unclassifiable` count reflects genuinely
non-EARS corpus authoring, which is Track C's user-gated problem, not a
classifier defect.

- **Measures:** `ac` findings over the fixture corpus AND a dry-run sweep over
  this repo's own `spec/` (counts only, no edits).
- **Pass criteria:** zero false-positive classes on fixtures; sweep count
  recorded in the Task-009 body; all Track A TCs green; `make ci` green.
- **If fails:** fix classifier/checks before any Track C sweep — never tune
  the corpus to the checker.

### Track B: Traceability feature (parallelizable with A; branch `task/traceability-coverage`)
#### B1 (Task-004): FR-050 `traceability:` model loader — shared dependency, first
#### B2 (Task-005): FR-051 language adapters + symbol identities (parallel-ready)
#### B3 (Task-006): FR-051 trace binding + relations + FR-045 records (after B1+B2)
#### B4 (Task-008): FR-049 verification-reference integrity (after B1; parallel to B2/B3)
#### B5 (Task-007): FR-050 reconciliation + `quire coverage` report (after B1+B3)

#### Gate G2: determinism + genericity
- **Measures:** Property TCs (TC-731, TC-738, TC-750) and the non-ISO fixture
  models (TC-727, TC-739, TC-745); TC-756 static audit.
- **Pass criteria:** all green under `make ci`; byte-identical repeat runs.
- **If fails:** ordering/hardcoding bug — fix before Track C migration, since
  the migration trusts extractor output.

### Track C: Corpus/quality cleanups (post-gate; SEPARATE branches; never mixed with A/B)
#### C1 (Task-009): AC-grammar baseline sweep (branch `chore/ac-grammar-baseline`)
- After G1. Sweep, record baseline, fix advisory findings in this repo's spec
  where mechanical; **promotion to `error` is user-gated (FR-047-CON-1)**.
#### C2 (Task-010): Legacy trace-tag migration (branch `chore/trace-tag-migration`)
- After G2. Apply marker-rewrite suggestions to this repo's tests; **legacy-
  recognition removal is user-gated (FR-051-CON-3)**; Rust `#[trace]` support
  crate is an external prerequisite.

## Parallel Execution Summary
```
Agent 1 (Track A): 001 ──> 002 ──> 003 ──> [G1] ─────────────> (C: 009)
Agent 2 (Track B): 004 ──> 006 ──────────> 007 ──> [G2] ─────> (C: 010)
Agent 3 (Track B): 005 ──/    008 (after 004, anytime)
```
Tracks A and B share no files beyond `src/lib.rs` wiring (single-writer:
merge A before B or rebase). Track C starts only after its gate and always on
its own branch.

## Task File Mapping

| Task | Track | Owns | Depends on | Status |
|---|---|---|---|---|
| [Task-001](./tasks/Task-001-fr048-severity-framework.md) | A | FR-048 | — | completed |
| [Task-002](./tasks/Task-002-fr047-ac-grammar.md) | A | FR-047 | Task-001 | completed |
| [Task-003](./tasks/Task-003-cli-surface-support.md) | A | FR-047-AC-8, FR-048-AC-5/6/10 | Task-002 | completed |
| [Task-004](./tasks/Task-004-fr050-traceability-model.md) | B | FR-050-AC-1/2 | — | completed |
| [Task-005](./tasks/Task-005-fr051-symbol-adapters.md) | B | FR-051-AC-1/2/3/9 | — | not_started |
| [Task-006](./tasks/Task-006-fr051-trace-binding-records.md) | B | FR-051-AC-4..8/10/11 | Task-004, Task-005 | not_started |
| [Task-007](./tasks/Task-007-fr050-coverage-rollup.md) | B | FR-050-AC-3..9, CON-2 | Task-004, Task-006 | not_started |
| [Task-008](./tasks/Task-008-fr049-reference-integrity.md) | B | FR-049 | Task-004 | not_started |
| [Task-009](./tasks/Task-009-ac-grammar-baseline-sweep.md) | C | corpus cleanup (FR-047-CON-1) | Task-003 (G1) | not_started |
| [Task-010](./tasks/Task-010-legacy-trace-tag-migration.md) | C | corpus cleanup (FR-051-CON-3) | Task-007 (G2) | not_started |

## Coordination Rules

- **Class isolation is hard:** feature work (A/B) and corpus cleanups (C)
  never share a branch, commit, or PR. A feature PR that also "fixes" corpus
  findings will be split.
- **User gates:** promoting any `ac` check beyond `warning` (FR-047-CON-1) and
  removing legacy tag recognition (FR-051-CON-3) require explicit user
  sign-off — the tasks stop and ask.
- **Spec freeze:** FR-047..051 are frozen for implementation; a spec error
  found mid-implementation gets a CR note (CR-002 pattern), never a silent
  edit.
- **External follow-ups** (`spec-artifacts-iso`, `spec-artifacts-process`,
  `quire-cli`, marker packages) are tracked in the External Dependencies
  table; engine tasks use fixture modules so none block Tracks A/B.
- **Merge sequencing:** A before B (or rebase B), C branches cut from main
  after the corresponding feature merge.
- **Quality bars:** `make ci` (fmt, clippy `-D warnings`, tests, deny, unsafe
  audit) green per task; no `HashMap` in order-observable paths (NFR-006);
  tests carry tracking tags for their TCs.
