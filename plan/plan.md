# Implementation Plan: quire-rs (v0.2 — block-model refinement)

Generated from `~/dev/quire-rs/spec/` via `/spec-to-plan`, then refined post-discovery audit. The v0.2 plan restores the block-addressable artifact model that was central to `INPUT.md` but had drifted out of the v0.1 spec. See `spec/spec.md` § 2bis (Drift Audit) for the full record of what's added, modified, and stripped vs v0.1.

## Core invariant

**Markdown is canonical.** The on-disk `.md` is the source of truth. Blocks are *parsed from* markdown. Edits change one block's data → re-render that block via its template → splice new bytes back into the `.md` via writeback. Frontmatter and untouched blocks stay byte-identical.

## Requirements Summary

### Stakeholder Requirements

- [x] **StR-001** Single generic Rust engine for parse + render + edit + writeback; archetype-as-data; offline (no network deps); adding a block type is a data-only change.
- [x] **StR-002** Byte-parity render output vs. Python Jinja2 reference (spec-artifacts-iso/app/process), CI-gated.
- [x] **StR-003** Parse parity with TS `agent-ix/quire` + Py `agent-ix/quire-py` acceptance fixtures.
- [x] **StR-004** Safety scaffolding inherited from `rust-lib-cookiecutter` (clippy MSRV, deny.toml, // SAFETY: enforcement, CI gates) — kept in sync via backport.

### User Stories

- [x] **US-001** LLM emits a validated patch targeting a single block; on-disk JSON Schema (via `schema_for(block_type)`) is the tool contract; engine writes back updated markdown with only that block's bytes changed.
- [x] **US-002** Developer calls `parse_document(md)` and receives a `QuireDocument` structurally equivalent to quire-py.
- [x] **US-003** Extractor evaluates `body_extraction` DSL against a parsed doc.
- [x] **US-004** Filament editor receives a block-level patch, validates, re-renders the block, writes back markdown — all well under a frame budget.
- [x] **US-005** CI fails on render-parity regression against Python reference fixtures.

### Functional Requirements

**Parser (foundation):**
- [x] **FR-005** `parse_document` API + `QuireDocument`/`QuireSection`/`Block` shape
- [x] **FR-006** Frontmatter extraction with malformed-fallback (incl. BOM strip)
- [x] **FR-007** Fenced-code-block-aware heading walk
- [x] **FR-008** Byte-exact section content slicing
- [x] **FR-009** Slug-line ID generation (legacy section ID; block IDs from `{#blk-id}`)
- [x] **FR-010** Query API: `section`, `sections`, `tables`, `lists`, `diagrams`, `search`
- [x] **FR-019** Stable block IDs via Pandoc heading attribute `## Heading {#blk-id}` — survives parse → reparse round-trip
- [x] **FR-020** Block data model: `Block { id, type, schema_version, data }` parsed from canonical markdown

**Loader:**
- [x] **FR-013** Block-type registry: filesystem-first loader of `blocks/<type>/{schema,template}`, IX_SCHEMA_PATH, symlink-loop guarded, `Send + Sync`
- [x] **FR-014** Module activation: multiple block-type modules coexist; collisions diagnosed

**Render + Edit + Writeback:**
- [x] **FR-001** Render dispatch — whole-artifact (assemble from per-block templates) + per-block (re-render one block's bytes)
- [x] **FR-003** `schema_for(block_type)` surfaces the JSON Schema for one block's patch shape
- [x] **FR-004** Strict MiniJinja env; `{% include %}` disabled at v1
- [x] **FR-012** Corpus-driven render-parity suite (block-by-block writeback flow)
- [x] **FR-021** Block edit API: `apply_block_patch(doc, block_id, patch)` + `replace_block(doc, block_id, new_data)`
- [x] **FR-022** Writeback primitives: `update_section(doc, heading, new_content)` + `update_block(doc, block_id, new_bytes)` — port + extend TS `updateSection`

**Extract:**
- [x] **FR-011** DSL evaluator: 6 Locator primitives + single/multi-yield
- [x] **FR-016** Secondary/fallback locator chains

### Non-Functional Requirements

- [x] **NFR-001** Render median <1ms per archetype (criterion bench; CI regression gate)
- [x] **NFR-002** Parse 5MB <500ms median (criterion bench; CI regression gate)
- [x] **NFR-003** Zero unsafe blocks in v1 (`audit-unsafe` baseline empty)
- [x] **NFR-004** License hygiene (`cargo deny check licenses`)
- [x] **NFR-005** Field-keyed actionable error format; no raw validator/serde leak
- [x] **NFR-006** Determinism: identical input → byte-identical output (proptest 10 000×, no observable HashMap)
- [x] **NFR-007** Archetype load cost amortized (compile once; per-call render does no disk I/O)
- [x] **NFR-009** Dependency version pinning per the validator-choice ADR
- [x] **NFR-010** API stability + CHANGELOG.md
- [x] **NFR-011** Fuzz: cargo-fuzz targets for parse, frontmatter, apply_block_patch, extract_dsl, load_manifest, load_schema, update_block
- [x] **NFR-012** miri UB check (weekly + tag-push)
- [x] **NFR-013** Mutation testing (cargo-mutants weekly)
- [x] **NFR-014** Advisory checking (cargo-audit daily + on PR)

### Stripped from v0.1 (not in INPUT.md)

- ~~FR-015~~ Relationship harvesting — agent-added without discovery basis
- ~~FR-017~~ Diagnostic Collection API (public collector) — internal-only now, payload-of-QuireError
- ~~FR-018~~ IxUriResolver — agent-added without discovery basis
- ~~NFR-008~~ Tracing instrumentation — agent-added without discovery basis

---

## Dependency Graph

### Core edges

- `FR-006, FR-007, FR-008, FR-009 -> FR-005`
  Reason: `parse_document` orchestrates frontmatter extraction, fence-aware heading walk, byte-exact slicing, and slug-ID generation. Build the primitives first; `parse_document` ties them together.
- `FR-005 -> FR-010`
  Reason: Query API operates on a `QuireDocument` returned by `parse_document`.
- `FR-013 -> FR-014`
  Reason: Module activation is a layer over the loader's per-module discovery.
- `FR-013 -> FR-001, FR-002, FR-003, FR-004, FR-012`
  Reason: Render-side requires `CompiledArchetype` instances produced by the loader (compiled schemas + parsed templates). `FR-003` (schema_for) just surfaces the loaded schema. `FR-004` env is built at load time. `FR-012` parity harness loads archetypes from a corpus manifest.
- `FR-013, FR-014 -> FR-001`
  Reason: Render dispatch needs a registry with possibly multi-module activation semantics.
- `FR-005, FR-010 -> FR-011`
  Reason: DSL locators (`section_body`, `code_block`, etc.) are all expressed in terms of the parsed document's query API.
- `FR-013 -> FR-011`
  Reason: DSL is loaded from manifest at archetype-load time and validated structurally then (XOR `match`/`iterate_over`, unknown-key rejection).
- `FR-011 -> FR-016`
  Reason: Fallback locators are a per-locator extension within the DSL evaluator.

### Shared dependencies

- **`QuireDocument` / `QuireSection` types (FR-005)** are consumed by FR-010, FR-011, FR-019..022. Build the types once with full `Send + Sync + Eq + Serialize` derives before any consumer.
- **`CompiledArchetype` struct (FR-013)** is consumed by FR-001, FR-002, FR-003, FR-004, FR-011, FR-012. Single owning type; reference-counted clone (`Arc<Inner>`).
- **JSON Schema compile path (FR-013)** is the boundary for FR-002 (validate) + FR-003 (surface). Pick a single validator crate (likely `jsonschema`) at FR-013 time.
- **`Diagnostic` enum** appears in FR-013, FR-014, FR-016, FR-011. Define once with non-exhaustive variant set; consumers match.
- **`QuireError` enum** appears in every FR's signature. Define early; expand variants as new FRs land.

### Cross-cutting NFRs

- **NFR-003 (zero unsafe)**: enforced by `make audit-unsafe`; applies to all source files. Already wired up via scaffold.
- **NFR-004 (license hygiene)**: `make deny`; applies to dep additions. Already wired up.
- **NFR-005 (error shape)**: applies to every error path in FR-001..016. Best implemented alongside `QuireError` itself.
- **NFR-006 (determinism)**: applies to render path (FR-001, FR-004), parse path (FR-005..010), extract path (FR-011, FR-016), and writeback path (FR-021, FR-022). Verified by proptest in each path's task.
- **NFR-001 (render <1ms)**: gates FR-001 + FR-004.
- **NFR-002 (parse <500ms)**: gates FR-005..010.
- **NFR-007 (load cost amortized)**: gates FR-013 + FR-014.

---

## Test Plan

### Unit Tests (50)

**Parser (TC-001, 012-017, 022-026):** empty/preamble/heading cases, frontmatter happy/malformed/no-fence/BOM (×2), backtick + tilde + unclosed fences, byte-exact slicing edge cases (whitespace, CRLF), slug normalization + line-index isolation, Unicode/degenerate-slug variants.

**Query API (TC-027 compile, TC-028 parity, TC-029 complexity).**

**Render dispatch (TC-003-008):** valid byte-equal, unknown-archetype error, missing-required field, strict template-field error, concurrent dispatch.

**Schema (TC-007, 007b, 009, 009b, 062, 170, 171):** merge-then-validate semantics, additionalProperties, schema surface, cross-file $ref rejection, $defs/$ref cycles.

**Strict env (TC-010, 011, 160):** Strict UndefinedBehavior, env cost, include rejection.

**DSL (TC-072 each locator, TC-070 iterate_over, TC-073 missing-required, TC-150-152 load-time validation).**

**Fallback locators (TC-110-113):** legacy vs canonical path, optional-miss, domain-object parity.

**Loader (TC-080-082, 130-135):** empty env, ISO load, bad schema_ref, symlink loop, IX_SCHEMA_PATH dedup, Send+Sync, path-is-file, module-name collision, missing-name dir-default.

**Errors (TC-054, 055):** Display shape, snapshot stability.

**Validator config (TC-205):** invalid-merged-value path.

### Integration Tests (19)

**Smoke (TC-200):** `cargo add quire-rs` + use-as-library.
**Shell-out audit (TC-201).**
**Multi-corpus equivalence (TC-060):** behavior identical across Filament/hand/test on-disk corpora.
**Concurrent render (TC-008).**
**Multi-module activation (TC-090, 092, 094):** load 3 modules; collision strict mode; baseline union.
**LLM tool round-trip (TC-061).**
**Archetype loader real-corpus (TC-081, 083 bench, 084 no-IO audit, 085 dep audit, 133).**
**Module collision (TC-091).**
**DSL real-fixture (TC-018, 019, 040).**
**Adding a new archetype is a data-only change (TC-005, TC-039).**

### Parity Tests (7)

**TS reference transliterated (TC-020).**
**quire-py structural equivalence (TC-021).**
**Render parity sweep (TC-030).**
**Fallback locator parity (TC-113).**
**Query API parity (TC-028).**
**Render-parity regression catch (TC-041).**

### Benchmarks (7)

- TC-042 (render per-archetype <1ms), TC-042b (apply_patch <100µs), TC-052 (parse 5MB <500ms), TC-083 (load <100ms), TC-120 (10k sequential renders), TC-011 (env cost), TC-206 (patch+render <1ms).

### Property Tests (8)

- TC-002 (parse no-panic), TC-002b (apply_patch fuzz no-panic), TC-024 (roundtrip), TC-029 (no quadratic walks), TC-053 (5MB roundtrip), TC-056 (render determinism), TC-057 (parse determinism).

### Static / Snapshot (13)

- TC-009 (schema surface snapshot), TC-031 (corpus.yaml exists), TC-050 (audit-unsafe), TC-051 (cargo deny), TC-058 (no HashMap audit), TC-062 (no schemars dep), TC-085 (no net deps), TC-121 (no recompile audit), TC-131 (IX_SCHEMA_PATH dedup), TC-202 (parity-notes.md), TC-203 (cookiecutter baseline), TC-204 (CI workflow), TC-201 (no shell-out).

### Compile-time assertions (2)

- TC-027 (Query API signatures compile), TC-132 (`Registry: Send + Sync`).

### Soak Test (1)

- TC-122 (1M-render memory-flat soak).

---

## Remaining Work

### Critical Path (Track A — serial)

The critical path runs through parser → loader → render → parity gate. Everything else either feeds in (Track B) or hangs off it (Track C).

```
A1 Parser primitives (FR-006..009)
   ↓
A2 parse_document (FR-005) + Query API (FR-010)
   ↓
A3 Parser parity gate (TC-020, TC-021)
   ↓
A4 Archetype loader (FR-013)
   ↓
A5 Module activation (FR-014)
   ↓
A6 Strict MiniJinja env (FR-004) + schema_for (FR-003)
   ↓
A7 Schema validation pipeline (FR-002)
   ↓
A8 Render dispatch (FR-001)
   ↓
A9 Parity harness + 1-archetype proof (FR-012, partial)
   ↓
A10 Render parity gate (TC-030 against FR archetype)
   ↓
A11 Full parity sweep (all 17 archetypes)
   ↓
A12 Perf gates (NFR-001, NFR-002, NFR-007)
```

#### Gate G1: Parser parity (after A3)

- **Measures:** Does Rust parser produce structurally equivalent `QuireDocument` to TS/Py reference across all transliterated fixtures?
- **Pass criteria:** TC-020 + TC-021 100% pass.
- **If fails:** Stop. The whole extract + writeback stack downstream depends on parser correctness. Investigate the diverging fixture, fix parser, re-run.

#### Gate G2: Render parity proof (after A10)

- **Measures:** Does the engine + loader + env produce byte-equal output to Python reference for at least one full archetype (FR)?
- **Pass criteria:** TC-030 for FR archetype passes byte-exact.
- **If fails:** Stop before scaling to remaining 16 archetypes. Likely root causes: template parse divergence, Strict undefined handling, JSON-merge edge case, validator state coupling. Diagnose with diff before adding more archetypes.

#### Gate G3: Perf NFRs (after A12)

- **Measures:** Render p50 + parse p50 + load p50 against NFR targets.
- **Pass criteria:** NFR-001 (<1ms), NFR-002 (<500ms / 5MB), NFR-007 (<100ms load). 10% regression bands set as baselines.
- **If fails:** Defer Track C until perf root cause identified. The data-driven design absorbs much of the cost via compile-once-render-fast; a >5× miss suggests a structural problem (e.g. validator chosen poorly).

### Track B: Parallel (independent, can start immediately)

These have no dependency on the critical path beyond shared types that are defined in A1/A2.

- **B1: Error shape (NFR-005)** — `QuireError` + `format_violation` helper + snapshot tests. Touches every FR but the surface is small. Define early so all FRs use it.
- **B2: Determinism harness (NFR-006)** — proptest infrastructure + HashMap audit. Independent of any specific FR; runs against everything in CI once FRs land.
- **B3: Scaffold polish (StR-004)** — verify cookiecutter inheritance (TC-203); add render-parity-notes.md placeholder; confirm `make ci` gates wired (TC-050, 051).
- **B4: Inter-tool contract (spec §17)** — backport notes to ix-cli spec when that repo is touched; this side is already documented.

### Track C: Post-render-gate (after G2)

Once the render path proves byte-parity, these become safe to implement in parallel with A11/A12.

- **C1: DSL locators + single-yield (FR-011 partial)** — 6 Locator primitives + `match` yield. Depends on parser (A2).
- **C2: DSL multi-yield (FR-011 advanced)** — depends on C1.
- **C4: Fallback locators (FR-016)** — depends on C1.

### Parallel Execution Summary

```
Time →
A1 Parser primitives ────────────╮
   A2 parse_document + Query ────┤
      Gate G1 (parser parity)    │      B1 Error shape           B3 Scaffold polish
         A4 Loader ──────────────┤      B2 Determinism harness   B4 Inter-tool contract
            A5 Module activation │
               A6 Env + schema_for
                  A7 Schema validate
                     A8 Render dispatch
                        A9 Parity harness
                           Gate G2 (render proof)
                              A11 Full parity sweep ── C1 DSL locators ──┐
                              A12 Perf NFRs                              C4 Fallback ──┐
                                                                 C2 DSL advanced ──── C3 Edges
                                                                 Gate G3 (perf)
```

---

## Task File Mapping

| Task file | Track / step | Owns | Status |
|---|---|---|---|
| `tasks/001-parser-primitives.md` | A1 | FR-006, FR-007, FR-008, FR-009 | ✅ complete |
| `tasks/002-parse-document.md` | A2 | FR-005 | ✅ complete |
| `tasks/003-query-api.md` | A2 | FR-010 | ✅ complete |
| `tasks/004-parser-parity-gate.md` | A3 / Gate G1 | TC-020, TC-021 | ✅ complete — **G1 PASS** |
| `tasks/005-archetype-loader.md` | A4 | FR-013 | ✅ complete |
| `tasks/006-module-activation.md` | A5 | FR-014 | ✅ complete |
| `tasks/007-minijinja-env.md` | A6 | FR-004 | ✅ complete |
| `tasks/008-schema-surface.md` | A6 | FR-003 | ✅ complete |
| `tasks/009-schema-validation.md` | A7 | FR-002 | ✅ complete |
| `tasks/010-render-dispatch.md` | A8 | FR-001 | ✅ complete |
| `tasks/011-parity-harness.md` | A9 | FR-012 | ✅ complete |
| `tasks/012-render-parity-gate.md` | A10 / Gate G2 | TC-030 (FR archetype proof) | ✅ complete — **G2 PASS** |
| `tasks/013-full-parity-sweep.md` | A11 | FR-012 (full corpus) | ✅ complete |
| `tasks/014-perf-gates.md` | A12 / Gate G3 | NFR-001, NFR-002, NFR-007 | ✅ complete — **G3 PASS** |
| `tasks/015-dsl-locators.md` | C1 | FR-011 (locators + match yield) | ✅ complete |
| `tasks/016-dsl-advanced.md` | C2 | FR-011 (iterate_over) | ✅ complete |
| `tasks/018-fallback-locators.md` | C4 | FR-016 | ✅ complete |
| `tasks/019-error-shape.md` | B1 | NFR-005 | ✅ complete |
| `tasks/020-determinism-harness.md` | B2 | NFR-006 | ✅ complete |
| `tasks/021-scaffold-polish.md` | B3 | StR-004 | ✅ complete |
| `tasks/025-static-audits.md` | B (cross-cutting) | static audit scripts | ✅ complete |
| `tasks/026-validator-bench-adr.md` | B (cross-cutting) | NFR-009 | ✅ complete (`jsonschema ~0.18`) |
| `tasks/027-hardening-suite.md` | B (cross-cutting) | NFR-011/012/013/014 | ✅ complete |

### v0.2 Block-model tasks (Track D — added post-discovery audit)

These logical tasks were executed on the spec-refinement branch (`feat/spec-refinement-block-model`); see commits for evidence.

| Task | Track / gate | Owns | Status |
|---|---|---|---|
| D1: Block-stable IDs via Pandoc `{#blk-id}` | D / parser layer | FR-019 | ✅ complete (commit `2afba4c`) |
| D2: Block data model + Registry alias | D / loader layer | FR-020 | ✅ complete (commits `2afba4c` + `1067def`) |
| D3: Writeback (`update_section` + `update_block`) | D | FR-022 | ✅ complete (commit `2afba4c`) |
| D4: Block edit API (`apply_block_patch` + `replace_block`) | D / Gate G4 | FR-021 | ✅ complete (commit `1067def`) — **G4 PASS** |
| D5: Strip agent-added subsystems (FR-015/017/018/NFR-008) | D / cleanup | spec + src purge | ✅ complete (commits `3b96c89` + `5b67538`) |

Total: 23 task files on disk + 5 logical v0.2 tasks. All tracks complete; all gates PASS.

### Gate G4: Block round-trip integrity (after D4)

- **Measures:** Does parse → apply_block_patch → writeback preserve frontmatter + untouched blocks byte-identical, change only the patched block's bytes, and survive reparse with block_id stability?
- **Pass criteria:** TC-440..443 + TC-420..425 + TC-430..435 all pass.
- **Status:** ✅ PASS — `make ci` green across 192 lib + 92 integration tests.

---

## Coordination Rules

- **Do not start tasks blocked by a gate** until the gate is marked Pass in this file.
- **Critical-path tasks merge serially.** Tracks B can merge whenever ready.
- **Track C tasks freeze if Gate G2 fails.** Re-evaluate the design before re-baselining.
- **Update `plan/tasks/README.md` on each task transition** (not started → in progress → complete).
- **Spec changes during implementation:** if a task discovers an AC that's wrong, open a CR; do not silently change the spec.
- **Test-first:** every task's first deliverable is the failing test set; second is the implementation that turns them green; third is the refactor + benches if applicable.
