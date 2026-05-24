# Test Matrix

## Overview

This matrix maps every Acceptance Criterion in `~/dev/quire-rs/spec/` to one or more Test Cases. Coverage status reflects intent (DRAFT) — implementation tasks are tracked separately via `/spec-to-plan`.

The spec was revised after authoring to reflect the **archetype-as-data** model: archetypes (schemas + templates + manifests) are loaded from the local filesystem at engine startup, never compiled into the engine. Sync from Filament to disk is owned by `ix-cli`; `quire-rs` is filesystem-only.

## Test Matrix Rules

1. **Coverage Rule**: Every acceptance criterion (AC) has at least one test case.
2. **Option Permutation Rule**: Each archetype loaded from the corpus is exercised by the parity sweep.
3. **Constraint Boundary Rule**: JSON Schema constraints (pattern, minLength, enum, const) are tested at boundary values via at least one fixture per archetype.
4. **Error Path Rule**: Every `QuireError` variant has at least one negative test.
5. **State Transition Rule**: Not applicable — engine is stateless beyond `Registry` lifecycle.
6. **Edge Case Rule**: Parser edge cases (unclosed fence, malformed YAML, level skips, empty input) and loader edge cases (missing schema_ref, duplicate archetype name, empty search path) have dedicated TCs.

---

## Requirements Traceability

### Stakeholder Requirement Coverage

| Stakeholder Req | Trace to US/FR | Test/Validation | Coverage Status |
|-----------------|----------------|-----------------|-----------------|
| StR-001 Single generic engine | US-001..005, all FRs | TC-001, TC-040, TC-060 | ✅ Complete |
| StR-002 Render parity | US-005, FR-012, NFR-006 | TC-030 (corpus sweep) | ✅ Complete |
| StR-003 Parse parity | US-002, FR-005..010, NFR-006 | TC-020, TC-021 | ✅ Complete |
| StR-004 Safety scaffolding | NFR-003, NFR-004 | TC-050, TC-051 | ✅ Complete |

### User Story Coverage

| User Story | Acceptance Criteria | Test Cases | Coverage Status |
|------------|---------------------|------------|-----------------|
| US-001 LLM patch + render | AC-1..4 | TC-009, TC-003, TC-006, TC-061 | ✅ Complete |
| US-002 Developer parses doc | AC-1..3 | TC-001, TC-029, TC-002 | ✅ Complete |
| US-003 Extractor evaluates DSL | AC-1..3 | TC-018, TC-019, TC-040 | ✅ Complete |
| US-004 Editor patch + render | AC-1..3 | TC-007, TC-042 | ✅ Complete |
| US-005 CI detects regression | AC-1..4 | TC-030, TC-031, TC-041 | ✅ Complete |

### Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Coverage Status |
|----------------|---------------------|------------|-----------------|
| FR-001 Generic render dispatch | AC-1..5 | TC-003, TC-004, TC-006, TC-008, TC-005 (data-only-change) | ✅ Complete |
| FR-002 JsonValue merge-validate | AC-1..5 | TC-007, TC-007b (additional-props), TC-002b (proptest), TC-042b (bench) | ✅ Complete |
| FR-003 schema_for surfaces fs schema | AC-1..4 | TC-009, TC-009b (unknown), TC-061, TC-062 (no schemars dep) | ✅ Complete |
| FR-004 Strict MiniJinja env | AC-1..3 | TC-010, TC-008, TC-011 | ✅ Complete |
| FR-005 parse_document API | AC-1..4 | TC-001, TC-029 | ✅ Complete |
| FR-006 Frontmatter fallback | AC-1..4 | TC-012, TC-013, TC-014 | ✅ Complete |
| FR-007 Fenced-block walk | AC-1..4 | TC-015, TC-016, TC-017 | ✅ Complete |
| FR-008 Byte-exact slicing | AC-1..3 | TC-022, TC-023, TC-024 | ✅ Complete |
| FR-009 Slug-line ID | AC-1..5 | TC-025, TC-026 | ✅ Complete |
| FR-010 Query API | AC-1..3 | TC-027, TC-028, TC-029 | ✅ Complete |
| FR-011 DSL (6 locators + yields) | AC-1..5 | TC-018, TC-019, TC-040, TC-070 (iterate_over), TC-071 (emit_edges), TC-072 (each locator), TC-073 (corpus sweep) | ✅ Complete |
| FR-012 Corpus parity suite | AC-1..5 | TC-030 (sweep), TC-031 (corpus.yaml), TC-041 (regression), TC-039 (data-only-extension) | ✅ Complete |
| FR-013 Archetype loader | AC-1..6 | TC-080 (empty env), TC-081 (load iso), TC-082 (bad schema_ref), TC-083 (bench), TC-084 (no IO post-load), TC-085 (no net deps) | ✅ Complete |
| FR-014 Module activation | AC-1..5 | TC-090 (multi-module), TC-091 (collision), TC-092 (strict), TC-093 (version), TC-094 (17-baseline union) | ✅ Complete |
| FR-015 Edge harvesting + dedup | AC-1..5 | TC-100 (sugar), TC-101 (dedup diag), TC-102 (parent alias), TC-103 (unresolved), TC-104 (parity vs python) | ✅ Complete |
| FR-016 Fallback locators | AC-1..4 | TC-110 (legacy path), TC-111 (canonical path), TC-112 (optional miss), TC-113 (domain parity) | ✅ Complete |

### Non-Functional Requirement Coverage

| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
|--------------------|---------------------|---------------------|--------|
| NFR-001 Render <1ms | criterion bench (median) | TC-042 + per-archetype bench under corpus sweep | ✅ Complete |
| NFR-002 Parse 5MB <500ms | criterion bench (median) | TC-052, TC-053 | ✅ Complete |
| NFR-003 Zero unsafe | static check (audit-unsafe) | TC-050 | ✅ Complete |
| NFR-004 License hygiene | cargo deny check licenses | TC-051 | ✅ Complete |
| NFR-005 Actionable errors | unit + snapshot | TC-006, TC-054, TC-055 | ✅ Complete |
| NFR-006 Determinism | proptest (render + parse 100x) | TC-056, TC-057, TC-058 | ✅ Complete |
| NFR-007 Load cost amortized | criterion bench + tracing audit | TC-083, TC-120, TC-121 (no recompile), TC-122 (soak) | ✅ Complete |

---

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---------|-------|------|----------|-----------|--------|
| TC-001 | parse_document handles empty + preamble-only + nested headings | Unit | P0 | FR-005-AC-1..3, US-002 | 🚧 |
| TC-002 | parse_document does not panic on 10k random inputs | Property | P0 | FR-005-AC-4 | 🚧 |
| TC-002b | apply_patch proptest fuzz never panics | Property | P0 | FR-002-AC-4 | 🚧 |
| TC-003 | render against compiled FR archetype byte-equals Python reference | Integration | P0 | FR-001-AC-1, US-001-AC-2 | 🚧 |
| TC-004 | render_by_name("unknown") returns UnknownArchetype | Unit | P0 | FR-001-AC-2 | 🚧 |
| TC-005 | Adding new archetype to corpus requires no Rust change | Integration | P0 | FR-001-AC-5, StR-001-AC-4 | 🚧 |
| TC-006 | render returns field-keyed SchemaViolation on missing required | Unit | P0 | FR-001-AC-3, NFR-005-AC-1 | 🚧 |
| TC-007 | apply_patch merges then validates merged result | Unit | P0 | FR-002-AC-1..2, US-004-AC-1..2 | 🚧 |
| TC-007b | apply_patch rejects unknown key under additionalProperties:false | Unit | P0 | FR-002-AC-3 | 🚧 |
| TC-008 | render is thread-safe under 64-thread concurrency | Integration | P1 | FR-001-AC-4, FR-004-AC-2 | 🚧 |
| TC-009 | schema_for returns the on-disk schema byte-identical | Snapshot | P0 | FR-003-AC-1, US-001-AC-4 | 🚧 |
| TC-009b | schema_for unknown archetype returns UnknownArchetype | Unit | P1 | FR-003-AC-2 | 🚧 |
| TC-010 | Strict mode reports missing template field as TemplateError | Unit | P0 | FR-004-AC-1 | 🚧 |
| TC-011 | Renderer environment cost measured (one-time) | Bench | P2 | FR-004-AC-3 | 🚧 |
| TC-012 | extract_frontmatter happy path | Unit | P0 | FR-006-AC-2 | 🚧 |
| TC-013 | extract_frontmatter malformed YAML returns body fallback | Unit | P0 | FR-006-AC-3 | 🚧 |
| TC-014 | extract_frontmatter unterminated fence returns body fallback | Unit | P1 | FR-006-AC-4 | 🚧 |
| TC-015 | Backtick fence blocks heading parsing inside | Unit | P0 | FR-007-AC-1 | 🚧 |
| TC-016 | Unclosed fence: trailing lines are not parsed as headings | Unit | P1 | FR-007-AC-2 | 🚧 |
| TC-017 | Tilde fence behaves identically to backtick fence | Unit | P1 | FR-007-AC-3 | 🚧 |
| TC-018 | extract evaluates api_endpoint DSL on real fixture | Integration | P0 | FR-011-AC-1, US-003-AC-1 | 🚧 |
| TC-019 | extract code_block (language: json) byte-equals fenced content | Integration | P0 | FR-011 (code_block locator), US-003-AC-2 | 🚧 |
| TC-020 | TS reference test suite transliterated; all pass | Parity | P0 | StR-003-AC-2 | 🚧 |
| TC-021 | quire-py vs quire-rs structural equivalence on real corpus | Parity | P1 | StR-003-AC-3 | 🚧 |
| TC-022 | Section content preserves leading/trailing whitespace | Unit | P0 | FR-008-AC-1 | 🚧 |
| TC-023 | CRLF and LF endings preserved in section content | Unit | P1 | FR-008-AC-2 | 🚧 |
| TC-024 | Roundtrip: reconstructing body from sections equals input | Property | P0 | FR-008-AC-3, NFR-006 | 🚧 |
| TC-025 | Slug normalization (lowercase, alphanum-dash, trim) | Unit | P0 | FR-009-AC-1..3 | 🚧 |
| TC-026 | Line index ignores frontmatter offset | Unit | P0 | FR-009-AC-4..5 | 🚧 |
| TC-027 | Query API module-level signatures compile and re-export | Compile | P0 | FR-010-AC-1 | 🚧 |
| TC-028 | Query API parity sweep against TS fixtures | Parity | P0 | FR-010-AC-2 | 🚧 |
| TC-029 | Query API complexity: no quadratic walks | Property | P1 | FR-010-AC-3 | 🚧 |
| TC-030 | Corpus parity sweep: every archetype × every fixture byte-equals Python reference | Parity | P0 | FR-012-AC-1..2, StR-002, US-005-AC-1..3 | 🚧 |
| TC-031 | tests/render_parity/corpus.yaml exists and lists v1 modules | Static | P0 | FR-012-AC-1 | 🚧 |
| TC-039 | Adding archetype to corpus.yaml + fixtures extends suite with no Rust change | Integration | P0 | FR-012-AC-5 | 🚧 |
| TC-040 | extract sweep across all 87+ object archetypes from 6 source repos | Integration | P0 | FR-011-AC-5, US-003 | 🚧 |
| TC-041 | Parity suite catches deliberate template mutation | Regression | P0 | FR-012-AC-3, US-005-AC-4 | 🚧 |
| TC-042 | Bench: render per-archetype median <1 ms (sweep across corpus) | Bench | P0 | NFR-001-AC-1..2 | 🚧 |
| TC-042b | Bench: apply_patch median <100 µs (typical artifact) | Bench | P1 | FR-002-AC-5 | 🚧 |
| TC-050 | check_unsafe_comments.sh exits 0; baseline empty | Static | P0 | NFR-003 | 🚧 |
| TC-051 | cargo deny check licenses exits 0 | Static | P0 | NFR-004 | 🚧 |
| TC-052 | Bench: parse_document 5 MB median <500 ms | Bench | P0 | NFR-002-AC-1 | 🚧 |
| TC-053 | Bench: 5 MB document round-trips byte-for-byte | Property | P0 | NFR-002-AC-3 | 🚧 |
| TC-054 | QuireError::Display contains all four required tuple elements | Unit | P0 | NFR-005-AC-1, US-001-AC-3 | 🚧 |
| TC-055 | QuireError snapshot pins canonical error per archetype | Snapshot | P1 | NFR-005-AC-3 | 🚧 |
| TC-056 | Determinism: render 100x across threads → byte-identical | Property | P0 | NFR-006-AC-1 | 🚧 |
| TC-057 | Determinism: parse 100x → Eq | Property | P0 | NFR-006-AC-2 | 🚧 |
| TC-058 | Static audit: no HashMap in render/parse code paths | Static | P1 | NFR-006-AC-3 | 🚧 |
| TC-060 | Registry behavior identical across three on-disk corpora (Filament/hand/test) | Integration | P0 | StR-001-AC-5 | 🚧 |
| TC-061 | LLM tool-call schema round-trip: schema_for → tool input → render | Integration | P1 | US-001-AC-2..3 | 🚧 |
| TC-062 | Cargo.lock has no schemars dependency | Static | P1 | FR-003-AC-4 | 🚧 |
| TC-070 | DSL multi-yield (iterate_over) emits one record per iteration unit | Unit | P0 | FR-011-AC-2 | 🚧 |
| TC-071 | DSL emit_edges produces one edge per declared target | Unit | P0 | FR-011-AC-3 | 🚧 |
| TC-072 | Each of 6 Locator primitives exercised by ≥1 unit test | Unit | P0 | FR-011-AC-1 | 🚧 |
| TC-073 | DSL required:true missing field returns MissingField | Unit | P0 | FR-011-AC-4 | 🚧 |
| TC-080 | Registry::from_env() with no IX_SCHEMA_PATH and no default dir → empty registry, no error | Unit | P0 | FR-013-AC-1 | 🚧 |
| TC-081 | IX_SCHEMA_PATH pointing at spec-artifacts-iso loads all 8 ISO archetypes | Integration | P0 | FR-013-AC-2 | 🚧 |
| TC-082 | Manifest with missing schema_ref produces ArchetypeLoadError; siblings still load | Integration | P0 | FR-013-AC-3 | 🚧 |
| TC-083 | Bench: Registry::load_from baseline corpus < 100 ms median | Bench | P0 | FR-013-AC-4, NFR-007-AC-1 | 🚧 |
| TC-084 | After load, render does no disk I/O (verified via strace / tracing audit) | Static | P0 | FR-013-AC-5 | 🚧 |
| TC-085 | Cargo.lock has no HTTP/RPC client crates | Static | P0 | FR-013-AC-6, StR-001-AC-3 | 🚧 |
| TC-090 | Two paths each with a module: both modules present in module_names() | Integration | P0 | FR-014-AC-1 | 🚧 |
| TC-091 | Duplicate archetype across modules → DuplicateArchetype diagnostic + first-wins | Integration | P0 | FR-014-AC-2 | 🚧 |
| TC-092 | load_strict on duplicate-archetype input returns ArchetypeCollision | Integration | P0 | FR-014-AC-3 | 🚧 |
| TC-093 | manifest.yaml version queryable via module_version() | Unit | P1 | FR-014-AC-4 | 🚧 |
| TC-094 | Loading iso + app + process modules yields union of 17 archetypes | Integration | P0 | FR-014-AC-5 | 🚧 |
| TC-100 | depends_on/parent/template_for sugar fields emit canonical edges | Unit | P0 | FR-015-AC-1 | 🚧 |
| TC-101 | Duplicate edge from sugar + relationships block deduped with diagnostic | Unit | P0 | FR-015-AC-2 | 🚧 |
| TC-102 | parent_process sugar alias maps to edge_type "parent" | Unit | P1 | FR-015-AC-3 | 🚧 |
| TC-103 | Unresolvable bare ID emits UnresolvedRelationshipTarget; edge preserved | Unit | P1 | FR-015-AC-4 | 🚧 |
| TC-104 | Edge harvesting parity vs filament-parser-lib relationships.py | Parity | P0 | FR-015-AC-5 | 🚧 |
| TC-110 | Fallback chain resolves via second locator + emits FallbackLocatorUsed | Unit | P0 | FR-016-AC-1 | 🚧 |
| TC-111 | Fallback chain resolves via first locator + no fallback diagnostic | Unit | P0 | FR-016-AC-2 | 🚧 |
| TC-112 | Fallback chain all-miss with required:false omits key | Unit | P1 | FR-016-AC-3 | 🚧 |
| TC-113 | domain object_type from ix-spec-objects with legacy heading: parity vs python | Parity | P0 | FR-016-AC-4 | 🚧 |
| TC-120 | Bench: 10 000 sequential renders after load → median <1ms, zero I/O | Bench | P0 | NFR-007-AC-2 | 🚧 |
| TC-121 | Tracing audit: zero Template::parse and zero JSONSchema::compile during render | Static | P0 | NFR-007-AC-3 | 🚧 |
| TC-122 | Long-running soak: registry memory footprint flat over 1 M renders | Soak | P1 | NFR-007-AC-4 | 🚧 |
| TC-130 | Loader symlink-loop detected; warning emitted; cycle skipped | Integration | P0 | FR-013-AC-7 | 🚧 |
| TC-131 | Duplicate IX_SCHEMA_PATH entries: modules loaded once | Integration | P0 | FR-013-AC-8 | 🚧 |
| TC-132 | Registry: Send + Sync (compile-time assertion) | Compile | P0 | FR-013-AC-9 | 🚧 |
| TC-133 | Path-entry-is-a-file: warning emitted; other entries process | Integration | P1 | FR-013-AC-10 | 🚧 |
| TC-134 | Two modules same name → DuplicateModuleName diag + first-wins | Integration | P0 | FR-014-AC-6 | 🚧 |
| TC-135 | Manifest without name uses parent dir name + diagnostic | Unit | P1 | FR-014-AC-7 | 🚧 |
| TC-140 | Edge dedup metadata: first wins; dropped reported in diagnostic | Unit | P0 | FR-015-AC-6 | 🚧 |
| TC-141 | harvest_edges deterministic across 64 threads | Property | P0 | FR-015-AC-7 | 🚧 |
| TC-150 | DSL with both match and iterate_over → ArchetypeLoadError at load | Unit | P0 | FR-011-AC-6 | 🚧 |
| TC-151 | DSL with unknown key → ArchetypeLoadError at load | Unit | P0 | FR-011-AC-7 | 🚧 |
| TC-152 | iterate_over.section_path missing → empty records + IterateRootMissing | Unit | P0 | FR-011-AC-8 | 🚧 |
| TC-160 | Template with {% include %} → ArchetypeLoadError | Unit | P0 | FR-004-AC-4 | 🚧 |
| TC-170 | Schema with internal $ref + $defs (recursive) compiles + validates | Unit | P0 | FR-002-AC-6 | 🚧 |
| TC-171 | Schema with cross-file $ref → ArchetypeLoadError at load | Unit | P0 | FR-002-AC-7 | 🚧 |
| TC-180 | extract_frontmatter handles BOM-prefixed input (with FM) | Unit | P0 | FR-006-AC-5 | 🚧 |
| TC-181 | extract_frontmatter handles BOM-prefixed input (no FM) | Unit | P0 | FR-006-AC-6 | 🚧 |
| TC-190 | Slug for non-ASCII heading "Café Menu" → "caf-menu-L<n>" | Unit | P0 | FR-009-AC-6 | 🚧 |
| TC-191 | Slug for degenerate "!!!" heading → "-L<n>" | Unit | P1 | FR-009-AC-7 | 🚧 |
| TC-200 | Smoke: `cargo add quire-rs` + `use quire_rs::parse_document` compiles & links in a hello-world consumer | Integration | P0 | StR-001-AC-1, US-002-AC-1 | 🚧 |
| TC-201 | Static grep: no `std::process::Command` invocations targeting python/node/npm/pip in src/ | Static | P0 | StR-001-AC-2 | 🚧 |
| TC-202 | Doc exists: spec/assets/render-parity-notes.md documents any known whitespace exceptions vs Python reference | Static | P1 | StR-002-AC-2 | 🚧 |
| TC-203 | Static: clippy.toml / deny.toml / rustfmt.toml / scripts/check_unsafe_comments.sh byte-equal rust-lib-cookiecutter baseline (or documented MSRV bump) | Static | P0 | StR-004-AC-1, StR-004-AC-2 | 🚧 |
| TC-204 | CI workflow includes render_parity job (not just test job) | Static | P0 | US-005-AC-2, US-005-AC-3, StR-002-AC-3 | 🚧 |
| TC-205 | A patch making merged value invalid (title="") returns SchemaViolation, not a render error | Unit | P0 | US-004-AC-2 | 🚧 |
| TC-206 | Bench: bench_patch_render_fr median < 1ms for typical FR | Bench | P1 | US-004-AC-3 | 🚧 |

---

## Option Permutation Matrix

The render dispatch is generic over `(CompiledArchetype, data)`. The corpus parity sweep (TC-030) exercises every archetype × every fixture; no separate option matrix is needed at the engine level. Permutation is encoded in the on-disk corpus.

| Test Case | archetype source | data validity | Expected |
|-----------|-------------------|----------------|----------|
| TC-030 | every archetype in corpus.yaml | valid (Python reference accepted) | byte-equal markdown |
| TC-004 | unknown name | (any) | UnknownArchetype |
| TC-006 | fr | missing required | SchemaViolation(field path) |
| TC-091 | duplicate across modules | (any) | DuplicateArchetype diagnostic + first-wins |

---

## Constraint Boundary Tests

Schema constraints come from the on-disk JSON Schema files. Boundary tests sit inside per-archetype fixture pairs (one valid-boundary, one beyond-boundary per archetype). Each `expected.md` for a valid-boundary case is generated by the Python reference; each beyond-boundary case is verified to raise `SchemaViolation` with the expected field path.

| Constraint family | Where verified |
|-------------------|----------------|
| `pattern` (regex) | Per-archetype fixtures + TC-006 |
| `minLength` / `maxLength` | Per-archetype fixtures + TC-007 |
| `const` (e.g. artifact_type) | Per-archetype fixtures |
| `required` array | TC-006, TC-073 (DSL side) |
| `additionalProperties: false` | TC-007b |
| `enum` | Per-archetype fixtures |

---

## Edge Cases

| ID | Description | Related Req | Test Case | Risk if Untested |
|----|-------------|-------------|-----------|------------------|
| EC-001 | Empty markdown input | FR-005 | TC-001 | parse_document panics or malformed doc |
| EC-002 | Markdown with no headings (preamble only) | FR-005 | TC-001 | preamble lost |
| EC-003 | Heading inside fenced code block | FR-007 | TC-015 | False positive heading splits content |
| EC-004 | Unclosed fenced code block | FR-007 | TC-016 | Trailing content split into phantom sections |
| EC-005 | Frontmatter with invalid YAML | FR-006 | TC-013 | parse_document returns Err instead of body-fallback |
| EC-006 | Frontmatter without closing fence | FR-006 | TC-014 | Body lost or partial returned |
| EC-007 | Heading level skip (`## A` then `#### B`) | FR-007 | TC-020 | Section tree mis-nested |
| EC-008 | CRLF line endings | FR-008 | TC-023 | Content slice loses CR |
| EC-009 | Title with leading/trailing whitespace | FR-009 | TC-025 | Slug ID has stray dashes |
| EC-010 | 5 MB document | NFR-002 | TC-052, TC-053 | Quadratic walk; OOM |
| EC-011 | Patch with additionalProperties violation | FR-002 | TC-007b | Silently accepted |
| EC-012 | Concurrent renders | FR-001, NFR-006 | TC-008, TC-056 | Data race, non-determinism |
| EC-013 | Empty IX_SCHEMA_PATH + missing default dir | FR-013 | TC-080 | Engine fails instead of empty-registry semantics |
| EC-014 | Manifest with broken schema_ref | FR-013 | TC-082 | All-or-nothing failure instead of partial load |
| EC-015 | Two modules defining same archetype name | FR-014 | TC-091, TC-092 | Silent shadow vs documented diagnostic |
| EC-016 | Frontmatter sugar field + structured relationships block both name same edge | FR-015 | TC-101 | Duplicate edges in graph |
| EC-017 | Document uses legacy heading variant | FR-016 | TC-110 | Silent data loss |
| EC-018 | Hot-path render re-reads disk | NFR-007 | TC-084, TC-121 | Per-call cost balloons |

---

## AC → TC Coverage Audit

Comprehensive, post-audit explicit mapping. Every AC defined in the spec is listed below with its covering TC(s). This section is the authoritative coverage source; the summary tables above are convenience views.

### Stakeholder Requirements

| AC | Covering TC(s) |
|---|---|
| StR-001-AC-1 | TC-200 |
| StR-001-AC-2 | TC-201 |
| StR-001-AC-3 | TC-085 |
| StR-001-AC-4 | TC-005 |
| StR-001-AC-5 | TC-060 |
| StR-002-AC-1 | TC-030 |
| StR-002-AC-2 | TC-202 |
| StR-002-AC-3 | TC-204 |
| StR-003-AC-1 | TC-020 |
| StR-003-AC-2 | TC-020 |
| StR-003-AC-3 | TC-021 |
| StR-004-AC-1 | TC-203 |
| StR-004-AC-2 | TC-050, TC-051, TC-203 |
| StR-004-AC-3 | TC-203 (process AC; verified by inheritance audit) |

### User Stories

| AC | Covering TC(s) |
|---|---|
| US-001-AC-1 | TC-009 |
| US-001-AC-2 | TC-003 |
| US-001-AC-3 | TC-006 |
| US-001-AC-4 | TC-009 |
| US-002-AC-1 | TC-200 |
| US-002-AC-2 | TC-020, TC-021 |
| US-002-AC-3 | TC-002 |
| US-003-AC-1 | TC-018 |
| US-003-AC-2 | TC-019 |
| US-003-AC-3 | TC-073, TC-040 |
| US-004-AC-1 | TC-007 |
| US-004-AC-2 | TC-205 |
| US-004-AC-3 | TC-206 |
| US-005-AC-1 | TC-031 |
| US-005-AC-2 | TC-204 |
| US-005-AC-3 | TC-204 |
| US-005-AC-4 | TC-041 |

### Functional Requirements

| AC | Covering TC(s) |
|---|---|
| FR-001-AC-1 | TC-003 |
| FR-001-AC-2 | TC-004 |
| FR-001-AC-3 | TC-006 |
| FR-001-AC-4 | TC-008 |
| FR-001-AC-5 | TC-005 |
| FR-002-AC-1 | TC-007 |
| FR-002-AC-2 | TC-007 |
| FR-002-AC-3 | TC-007b |
| FR-002-AC-4 | TC-002b |
| FR-002-AC-5 | TC-042b |
| FR-002-AC-6 | TC-170 |
| FR-002-AC-7 | TC-171 |
| FR-003-AC-1 | TC-009 |
| FR-003-AC-2 | TC-009b |
| FR-003-AC-3 | TC-061 |
| FR-003-AC-4 | TC-062 |
| FR-004-AC-1 | TC-010 |
| FR-004-AC-2 | TC-008 |
| FR-004-AC-3 | TC-011 |
| FR-004-AC-4 | TC-160 |
| FR-005-AC-1 | TC-001 |
| FR-005-AC-2 | TC-001 |
| FR-005-AC-3 | TC-001 |
| FR-005-AC-4 | TC-002 |
| FR-006-AC-1 | TC-012 |
| FR-006-AC-2 | TC-012 |
| FR-006-AC-3 | TC-013 |
| FR-006-AC-4 | TC-014 |
| FR-006-AC-5 | TC-180 |
| FR-006-AC-6 | TC-181 |
| FR-007-AC-1 | TC-015 |
| FR-007-AC-2 | TC-016 |
| FR-007-AC-3 | TC-017 |
| FR-007-AC-4 | TC-020 |
| FR-008-AC-1 | TC-022 |
| FR-008-AC-2 | TC-023 |
| FR-008-AC-3 | TC-024 |
| FR-009-AC-1 | TC-025 |
| FR-009-AC-2 | TC-025 |
| FR-009-AC-3 | TC-025 |
| FR-009-AC-4 | TC-026 |
| FR-009-AC-5 | TC-026 |
| FR-009-AC-6 | TC-190 |
| FR-009-AC-7 | TC-191 |
| FR-010-AC-1 | TC-027 |
| FR-010-AC-2 | TC-028 |
| FR-010-AC-3 | TC-029 |
| FR-011-AC-1 | TC-072 |
| FR-011-AC-2 | TC-070 |
| FR-011-AC-3 | TC-071 |
| FR-011-AC-4 | TC-073 |
| FR-011-AC-5 | TC-040 |
| FR-011-AC-6 | TC-150 |
| FR-011-AC-7 | TC-151 |
| FR-011-AC-8 | TC-152 |
| FR-012-AC-1 | TC-031 |
| FR-012-AC-2 | TC-030 |
| FR-012-AC-3 | TC-041 |
| FR-012-AC-4 | TC-041 (script existence subsumed by regression check) |
| FR-012-AC-5 | TC-039 |
| FR-013-AC-1 | TC-080 |
| FR-013-AC-2 | TC-081 |
| FR-013-AC-3 | TC-082 |
| FR-013-AC-4 | TC-083 |
| FR-013-AC-5 | TC-084 |
| FR-013-AC-6 | TC-085 |
| FR-013-AC-7 | TC-130 |
| FR-013-AC-8 | TC-131 |
| FR-013-AC-9 | TC-132 |
| FR-013-AC-10 | TC-133 |
| FR-014-AC-1 | TC-090 |
| FR-014-AC-2 | TC-091 |
| FR-014-AC-3 | TC-092 |
| FR-014-AC-4 | TC-093 |
| FR-014-AC-5 | TC-094 |
| FR-014-AC-6 | TC-134 |
| FR-014-AC-7 | TC-135 |
| FR-015-AC-1 | TC-100 |
| FR-015-AC-2 | TC-101 |
| FR-015-AC-3 | TC-102 |
| FR-015-AC-4 | TC-103 |
| FR-015-AC-5 | TC-104 |
| FR-015-AC-6 | TC-140 |
| FR-015-AC-7 | TC-141 |
| FR-016-AC-1 | TC-110 |
| FR-016-AC-2 | TC-111 |
| FR-016-AC-3 | TC-112 |
| FR-016-AC-4 | TC-113 |

### Non-Functional Requirements

| AC | Covering TC(s) |
|---|---|
| NFR-001-AC-1 | TC-042 |
| NFR-001-AC-2 | TC-042 (sweep across corpus) |
| NFR-001-AC-3 | TC-042 (regression-gate assertion) |
| NFR-002-AC-1 | TC-052 |
| NFR-002-AC-2 | TC-052 (regression-gate assertion) |
| NFR-002-AC-3 | TC-053 |
| NFR-003-AC-1 | TC-050 |
| NFR-003-AC-2 | TC-050 |
| NFR-003-AC-3 | TC-050 |
| NFR-004-AC-1 | TC-051 |
| NFR-004-AC-2 | TC-051 |
| NFR-004-AC-3 | TC-051 |
| NFR-005-AC-1 | TC-054 |
| NFR-005-AC-2 | TC-054 |
| NFR-005-AC-3 | TC-055 |
| NFR-006-AC-1 | TC-056 |
| NFR-006-AC-2 | TC-057 |
| NFR-006-AC-3 | TC-058 |
| NFR-007-AC-1 | TC-083 |
| NFR-007-AC-2 | TC-120 |
| NFR-007-AC-3 | TC-121 |
| NFR-007-AC-4 | TC-122 |

**Coverage status: 141 / 141 ACs covered (100%).**

---

## Coverage Gaps

| Gap ID | Description | Risk Level | Mitigation |
|--------|-------------|------------|------------|
| GAP-001 | DSL evaluator parity test (TC-040) needs a curated fixture document per object_type across all 87+ types; some fixtures may not yet exist in the source repos. | Medium | Track per-type fixture availability in `tests/extract_parity/coverage.md`; missing fixtures are P1 follow-ups. |
| GAP-002 | Python Jinja2 reference renderer is not byte-stable across Jinja2 minor versions in all whitespace cases. | Low | StR-002-AC-2 documents known whitespace exceptions; pin reference's Jinja2 version. |
| GAP-003 | Cross-machine determinism (arm64 vs x86_64 byte parity) is implied but not explicitly benched. | Low | Add an arm64 + x86_64 CI matrix as a P2 enhancement. |
| GAP-004 | The relationship resolver (FR-015) is caller-supplied; quire-rs does not ship a default. Test fixtures use an in-test stub resolver. | Low | Document the resolver contract clearly; provide reference stub in test utilities. |
| GAP-005 | Sync from Filament to disk is out of scope (lives in `ix-cli`). Integration tests confirm quire-rs is correct against the on-disk state regardless of how it got there. | None | No mitigation needed — by design. |

---

## Test Execution Summary

All tests are DRAFT — pending implementation via `/spec-to-plan` → `/implement-plan`.

| Category | Total | Passed | Failed | Blocked | Coverage |
|----------|-------|--------|--------|---------|----------|
| Unit | 50 | 0 | 0 | 50 | 0% |
| Integration | 19 | 0 | 0 | 19 | 0% |
| Parity | 7 | 0 | 0 | 7 | 0% |
| Bench | 7 | 0 | 0 | 7 | 0% |
| Property | 8 | 0 | 0 | 8 | 0% |
| Static / Snapshot | 13 | 0 | 0 | 13 | 0% |
| Compile | 2 | 0 | 0 | 2 | 0% |
| Soak | 1 | 0 | 0 | 1 | 0% |
| **Total** | **104** | **0** | **0** | **104** | **0%** |
