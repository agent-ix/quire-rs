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
| StR-001 Single generic engine | US-002, US-003 (US-001/004/005 retired), all FRs | TC-001, TC-040, TC-060 | ✅ Complete |
| StR-002 Render parity | ~~US-005~~ (retired), FR-012 (retired), NFR-006 | TC-030 (corpus sweep) | ✅ Complete |
| StR-003 Parse parity | US-002, FR-005..010, NFR-006 | TC-020, TC-021 | ✅ Complete |
| StR-004 Safety scaffolding | NFR-003, NFR-004 | TC-050, TC-051 | ✅ Complete |
| StR-005 Native Python bindings | US-011, FR-023, FR-024, NFR-016 | TC-460, TC-461, TC-456, TC-466, TC-465 | ✅ Complete |
| StR-006 Whole-spec corpus | US-012, US-013, FR-025, FR-026, FR-027 | TC-485, TC-493, TC-488, TC-484, TC-483 | ✅ Complete |

### User Story Coverage

| User Story | Acceptance Criteria | Test Cases | Coverage Status |
|------------|---------------------|------------|-----------------|
| ~~US-001 LLM patch + render~~ | — | — | ⛔ RETIRED (render removal) |
| US-002 Developer parses doc | AC-1..3 | TC-001, TC-029, TC-002 | ✅ Complete |
| US-003 Extractor evaluates DSL | AC-1..3 | TC-018, TC-019, TC-040 | ✅ Complete |
| ~~US-004 Editor patch + render~~ | — | — | ⛔ RETIRED (render removal) |
| ~~US-005 CI detects regression~~ | — | — | ⛔ RETIRED (render removal; render byte-parity suite removed) |
| ~~US-006 LLM patches one block~~ | — | — | ⛔ RETIRED (render removal; block edits byte-splice only via FR-022) |
| ~~US-007 LLM replaces block wholesale~~ | — | — | ⛔ RETIRED (render removal; block edits byte-splice only via FR-022) |
| US-008 Multi-agent collaboration via stable block_id | AC-1..4 + PC-1..5 | TC-431, TC-432, TC-440, TC-443 (correctness) + TC-452 (perf) | ✅ Functional / 🚧 Perf bench pending |
| ~~US-009 LLM creates new artifact~~ | — | — | ⛔ RETIRED (render removal; author markdown directly + FR-032) |
| US-010 LLM extracts for RAG | AC-1..5 + PC-1..5 | TC-018, TC-019, TC-040, TC-070, TC-110, TC-152 (correctness) + TC-453, TC-454 (perf) | ✅ Functional / 🚧 Perf bench pending |
| US-011 Python parses repo via bindings | AC-1..5 + PC-1..3 | TC-463, TC-471, TC-467, TC-475, TC-464 (correctness) + TC-455, TC-469, TC-456 (perf) | ✅ Functional / 🚧 Perf bench pending |
| US-012 Agent audits whole spec | AC-1..5 + PC-1..3 | TC-493, TC-495, TC-494, TC-496, TC-485 (correctness) + TC-457, TC-458, TC-498 (perf) | ✅ Functional / 🚧 Perf bench pending |
| US-013 Agent resolves intra-spec refs | AC-1..5 + PC-1..3 | TC-486, TC-487, TC-488, TC-489, TC-490 (correctness) + TC-459, TC-492 (perf) | ✅ Functional / 🚧 Perf bench pending |
| US-014 Author validates markdown | AC-1..4 | TC-518, TC-519, TC-520, TC-521 | 🚧 Pending implementation |

### Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Coverage Status |
|----------------|---------------------|------------|-----------------|
| ~~FR-001 Generic render dispatch~~ | — | — | ⛔ RETIRED (render removal) |
| FR-002 JsonValue merge-validate | AC-1..5 | TC-007, TC-007b (additional-props), TC-002b (proptest), TC-042b (bench) | ✅ Complete |
| FR-003 schema_for surfaces fs schema | AC-1..4 | TC-009, TC-009b (unknown), TC-061, TC-062 (no schemars dep) | ✅ Complete |
| ~~FR-004 Strict MiniJinja env~~ | — | — | ⛔ RETIRED (render removal) |
| FR-005 parse_document API | AC-1..4 | TC-001, TC-029 | ✅ Complete |
| FR-006 Frontmatter fallback | AC-1..4 | TC-012, TC-013, TC-014 | ✅ Complete |
| FR-007 Fenced-block walk | AC-1..4 | TC-015, TC-016, TC-017 | ✅ Complete |
| FR-008 Byte-exact slicing | AC-1..3 | TC-022, TC-023, TC-024 | ✅ Complete |
| FR-009 Slug-line ID | AC-1..5 | TC-025, TC-026 | ✅ Complete |
| FR-010 Query API | AC-1..4 | TC-027, TC-028, TC-029, TC-589 (CR-007 escaped pipes + separator/bullet characterization) | ✅ Complete |
| FR-011 DSL (6 locators + yields + asserts) | AC-1..8, 13..21 | TC-018, TC-019, TC-040, TC-070, TC-072, TC-073, TC-563, TC-564, TC-565 (regex), TC-566 (under_section:None), TC-567 ({{…}}), TC-568 (unclosed fence), TC-569 (emit_edges), TC-583 (multiple:true) | ✅ |
| ~~FR-012 Corpus parity suite~~ | — | — | ⛔ RETIRED (render removal) |
| FR-013 Archetype loader | AC-1..6 | TC-080 (empty env), TC-081 (load iso), TC-082 (bad schema_ref), TC-083 (bench), TC-084 (no IO post-load), TC-085 (no net deps) | ✅ Complete |
| FR-014 Module activation | AC-1..5 | TC-090 (multi-module), TC-091 (collision), TC-092 (strict), TC-093 (version), TC-094 (17-baseline union) | ✅ Complete |
| FR-016 Fallback locators | AC-1..4 | TC-110 (legacy path), TC-111 (canonical path), TC-112 (optional miss), TC-113 (domain parity) | ✅ Complete |
| FR-019 Stable block IDs via Pandoc `{#blk-id}` | AC-1..3 | TC-400 (parser populates block_id), TC-401 (reparse round-trip), TC-402 (heading stripped of attr), TC-403 (no-attr → None) | ✅ Complete |
| FR-020 Block data model (block_id + block_type) | AC-1..2 | TC-410 (block_id addressable via QuireSection), TC-411 (block_type → archetype 1:1 alias) | ⚠️ Partial (no dedicated `Block` struct; v0.2 stores block_id on QuireSection + treats archetype as block_type) |
| FR-021 Block edit API | AC-1..6 | TC-420 (apply_block_patch merge/render/splice), TC-421 (replace_block), TC-422 (invalid → SchemaViolation), TC-423 (unknown block_type), TC-424 (unknown block_id), TC-425 (LLM-flow rendered == direct) | ✅ Complete |
| FR-022 Writeback primitives | AC-1..5 | TC-430 (update_section replaces content), TC-431 (update_block replaces heading+content), TC-432 (other blocks byte-identical), TC-433 (frontmatter preserved), TC-434/435 (missing heading/id → MissingField) | ✅ Complete |
| FR-023 PyO3 binding surface | AC-1..7 | TC-460 (feature-gate), TC-461 (parse parity), TC-462 (validate parity), TC-463 (load_repo via binding), TC-464 (GIL release), TC-465 (abi3 cross-version), TC-466 (no subprocess) | ✅ Complete |
| FR-028 Expanded Python surface | AC-1..8 | TC-510 (⛔ RETIRED — render removed), TC-511 (validate happy+sad), TC-512 (validate_manifest), TC-513 (extract envelope), TC-514 (extract_frontmatter), TC-515 (harvest_edges dict+str), TC-516 (exception hierarchy), TC-517 (GIL release multi-thread) | ✅ Complete |
| FR-029 Archetype input contract (recast, ADR 0004) | AC-1..6 | TC-548 (FR/NFR contract), TC-549 (NFR sections), TC-550 (iso required_sections order), TC-551 (byte-stable JSON), TC-552 (unknown→err), TC-553 (unresolved-mapping diag) | 🚧 Pending implementation |
| FR-030 Required-section validation (superseded by FR-032/FR-033, ADR 0004) | AC-1..6 | TC-529, TC-530, TC-536, TC-528, TC-533 (covered by FR-032/FR-033 TCs) | 🚧 Superseded — covered by FR-032/FR-033 |
| FR-031 Unified archetype shape | AC-1..6 | TC-522 (validatable+extractable, no renderability), TC-523 (no body_extraction → extraction None), TC-524 (defaults retained), TC-525 (two validators), TC-526 (required_sections ignored+diag), TC-527 (resolve parity) | 🚧 Pending implementation |
| FR-032 validate_document (markdown) | AC-1..10 | TC-528..533, TC-561 + TC-573 (placeholder set), TC-574 (none/n-a substantive), TC-575 (empty table/list reason), TC-576 (assert on resolved) | ✅ |
| FR-033 Locator assert facet | AC-1..9 | TC-534..539, TC-561/562 + TC-570 (legality matrix), TC-571 (id-column precedence), TC-572 (id_pattern non-table) | ✅ |
| FR-034 Assert field interpolation | AC-1..4 | TC-540 (id prefix), TC-541 (missing field diag), TC-542 (regex-escape), TC-543 (no-token static regex) | 🚧 Pending implementation |
| FR-035 Per-level heading uniqueness | AC-1..4 | TC-544 (dup L2), TC-545 (cross-level ok), TC-546 (iterate_over children), TC-547 (line number) | 🚧 Pending implementation |
| FR-036 Declarative lint rules | AC-1..5 | TC-584 (manifest→Registry::lint_rules + malformed rule fails load), TC-585 (vocab finding + annotation pass), TC-586 (archetype scoping), TC-587 (missing section/column → none), TC-588 (lint never affects extract/validate) | ✅ |
| FR-037 Base concept frontmatter schema (OKF) | AC-1..6 | TC-590 (minimal typed), TC-591 (optional desc/tags), TC-592 (missing type), TC-593 (empty type), TC-594/595/596 (mistyped desc/tags/non-string item), TC-528 (shape wired into validate_document) | ✅ |
| FR-038 OKF bundle validation (Strict vs Okf + index) | AC-1..8 | TC-600 (strict untyped→error), TC-601 (okf untyped→error), TC-602 (okf tolerates unknown type+broken link, strict rejects), TC-603 (strict conformant+complete index valid), TC-604 (index incompleteness error/warning), TC-605 (root missing okf_version), TC-606 (subdir no okf_version), TC-607 (strict mistyped description) | ✅ |
| FR-024 Parallel repo walk (load_repo) | AC-1..9 | TC-470 (N files→N docs), TC-471 (malformed→diagnostic), TC-472 (gitignore), TC-473 (path-sorted determinism), TC-474 (symlink loop), TC-475 (id derivation), TC-476 (bad root), TC-455 (bench), TC-502 (no shared mutable state) | ✅ Complete |
| FR-025 Spec corpus model | AC-1..6 | TC-480 (len), TC-481 (id index), TC-482 (dup id), TC-483 (Send+Sync), TC-484 (scope-guard surface), TC-485 (no-IO queries) | ✅ Complete |
| FR-026 Intra-spec reference resolution | AC-1..7 | TC-486 (frontmatter edge), TC-487 (ix:// edge), TC-488 (dangling), TC-489 (cross-spec dangling), TC-490 (bidirectional), TC-491 (target-id extraction), TC-492 (O(edges) proptest) | ✅ Complete |
| FR-027 Whole-spec query API | AC-1..8 | TC-493 (by_type), TC-494 (referencing), TC-495 (orphans), TC-496 (coverage), TC-497 (dangling agreement), TC-498 (sorted determinism), TC-499 (no-IO), TC-458 (bench) | ✅ Complete |

### Non-Functional Requirement Coverage

| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
|--------------------|---------------------|---------------------|--------|
| ~~NFR-001 Render <1ms~~ | — | — | ⛔ RETIRED (render removal) |
| NFR-002 Parse 5MB <500ms; validate_document <1ms | criterion bench (median) | TC-052, TC-053, TC-577 (validate_document) | ✅ |
| NFR-003 Zero unsafe | static check (audit-unsafe) | TC-050 | ✅ Complete |
| NFR-004 License hygiene | cargo deny check licenses | TC-051 | ✅ Complete |
| NFR-005 Actionable errors | unit + snapshot | TC-006, TC-054, TC-055 | ✅ Complete |
| NFR-006 Determinism | proptest (parse + validate_document + extract 100x) | TC-057, TC-058, TC-578 | ✅ |
| NFR-019 Input robustness (no panic) | fuzz + proptest | TC-579, TC-580 | ✅ |
| NFR-007 Load cost amortized | criterion bench + tracing audit | TC-083, TC-120, TC-121 (no recompile), TC-122 (soak) | ✅ Complete |
| NFR-015 Repo-walk throughput scales | criterion bench (1 + 8 threads) | TC-455 | ✅ Complete |
| NFR-016 Binding overhead + abi3 | micro-bench + cross-version import | TC-469, TC-464, TC-465, TC-467 | ✅ Complete |
| NFR-017 Concurrency permutation (loom) | loom exhaustive interleaving (scheduled lane) | TC-502, TC-503 | ✅ Complete |
| NFR-018 FFI sanitizer lanes (TSAN+ASAN) | scheduled sanitizer lanes on the extension | TC-504, TC-505 | ✅ Complete |

---

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---------|-------|------|----------|-----------|--------|
| TC-001 | parse_document handles empty + preamble-only + nested headings | Unit | P0 | FR-005-AC-1..3, US-002 | 🚧 |
| TC-002 | parse_document does not panic on 10k random inputs | Property | P0 | FR-005-AC-4 | 🚧 |
| TC-002b | apply_patch proptest fuzz never panics | Property | P0 | FR-002-AC-4 | 🚧 |
| TC-003 | render against compiled FR archetype byte-equals Python reference | Integration | P0 | FR-001-AC-1 (RETIRED), US-001-AC-2 (RETIRED) | ⛔ RETIRED — render removed |
| TC-004 | render_by_name("unknown") returns UnknownArchetype | Unit | P0 | FR-001-AC-2 (RETIRED) | ⛔ RETIRED — render removed |
| TC-005 | Adding new archetype to corpus requires no Rust change | Integration | P0 | FR-001-AC-5, StR-001-AC-4 | 🚧 |
| TC-006 | render returns field-keyed SchemaViolation on missing required | Unit | P0 | FR-001-AC-3 (RETIRED), NFR-005-AC-1 | ⛔ RETIRED — render removed |
| TC-007 | apply_patch merges then validates merged result | Unit | P0 | FR-002-AC-1..2, US-004-AC-1..2 | 🚧 |
| TC-007b | apply_patch rejects unknown key under additionalProperties:false | Unit | P0 | FR-002-AC-3 | 🚧 |
| TC-008 | render is thread-safe under 64-thread concurrency | Integration | P1 | FR-001-AC-4 (RETIRED), FR-004-AC-2 (RETIRED) | ⛔ RETIRED — render removed |
| TC-009 | schema_for returns the on-disk schema byte-identical | Snapshot | P0 | FR-003-AC-1, US-001-AC-4 | 🚧 |
| TC-009b | schema_for unknown archetype returns UnknownArchetype | Unit | P1 | FR-003-AC-2 | 🚧 |
| TC-010 | Strict mode reports missing template field as TemplateError | Unit | P0 | FR-004-AC-1 (RETIRED) | ⛔ RETIRED — render removed |
| TC-011 | Renderer environment cost measured (one-time) | Bench | P2 | FR-004-AC-3 (RETIRED) | ⛔ RETIRED — render removed |
| TC-012 | extract_frontmatter happy path | Unit | P0 | FR-006-AC-2 | 🚧 |
| TC-013 | extract_frontmatter malformed YAML returns body fallback | Unit | P0 | FR-006-AC-3 | 🚧 |
| TC-014 | extract_frontmatter unterminated fence returns body fallback | Unit | P1 | FR-006-AC-4 | 🚧 |
| TC-015 | Backtick fence blocks heading parsing inside | Unit | P0 | FR-007-AC-1 | 🚧 |
| TC-016 | Unclosed fence: trailing lines are not parsed as headings | Unit | P1 | FR-007-AC-2 | 🚧 |
| TC-017 | Tilde fence behaves identically to backtick fence | Unit | P1 | FR-007-AC-3 | 🚧 |
| TC-018 | extract evaluates api_endpoint DSL on real fixture | Integration | P0 | FR-011-AC-1, US-003-AC-1 | 🚧 |
| TC-019 | extract code_block (language: json) byte-equals fenced content | Integration | P0 | FR-011 (code_block locator), US-003-AC-2 | 🚧 |
| TC-020 | TS reference test suite transliterated; all pass | Parity | P0 | StR-003-AC-2 | 🚧 |
| TC-021 | quire-rs structural equivalence against canonical TS fixtures on real corpus | Parity | P1 | StR-003-AC-3 | 🚧 |
| TC-022 | Section content preserves leading/trailing whitespace | Unit | P0 | FR-008-AC-1 | 🚧 |
| TC-023 | CRLF and LF endings preserved in section content | Unit | P1 | FR-008-AC-2 | 🚧 |
| TC-024 | Roundtrip: reconstructing body from sections equals input | Property | P0 | FR-008-AC-3, NFR-006 | 🚧 |
| TC-025 | Slug normalization (lowercase, alphanum-dash, trim) | Unit | P0 | FR-009-AC-1..3 | 🚧 |
| TC-026 | Line index ignores frontmatter offset | Unit | P0 | FR-009-AC-4..5 | 🚧 |
| TC-027 | Query API module-level signatures compile and re-export | Compile | P0 | FR-010-AC-1 | 🚧 |
| TC-028 | Query API parity sweep against TS fixtures | Parity | P0 | FR-010-AC-2 | 🚧 |
| TC-029 | Query API complexity: no quadratic walks | Property | P1 | FR-010-AC-3 | 🚧 |
| TC-589 | `\|` in table cells is literal (escape consumed) in header/body/cell-final positions; other backslashes verbatim; borderless rows split identically; GFM alignment separators recognized; `-`/`*`/`+` bullets parse | Unit | P0 | FR-010-AC-4 | ✅ |
| TC-030 | Corpus parity sweep: every archetype × every fixture byte-equals Python reference | Parity | P0 | FR-012-AC-1..2 (RETIRED), StR-002, US-005-AC-1..3 (RETIRED) | 🚧 |
| TC-031 | tests/render_parity/corpus.yaml exists and lists v1 modules | Static | P0 | FR-012-AC-1 (RETIRED) | ⛔ RETIRED — render removed |
| TC-039 | Adding archetype to corpus.yaml + fixtures extends suite with no Rust change | Integration | P0 | FR-012-AC-5 | 🚧 |
| TC-040 | extract sweep across all 87+ object archetypes from 6 source repos | Integration | P0 | FR-011-AC-5, US-003 | 🚧 |
| TC-041 | Parity suite catches deliberate template mutation | Regression | P0 | FR-012-AC-3 (RETIRED), US-005-AC-4 (RETIRED) | ⛔ RETIRED — render removed |
| TC-042 | Bench: render per-archetype median <1 ms (sweep across corpus) | Bench | P0 | NFR-001-AC-1..2 (RETIRED) | ⛔ RETIRED — render removed |
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
| TC-072 | Each of 6 Locator primitives exercised by ≥1 unit test | Unit | P0 | FR-011-AC-1 | 🚧 |
| TC-073 | DSL required:true missing field returns MissingField | Unit | P0 | FR-011-AC-4 | 🚧 |
| TC-563 | `code_block` is section-owned: single-yield `under:X` excludes other sections; multi-yield `per_match` isolates each unit's block, required-miss → MissingField for the unit lacking one | Unit | P0 | FR-011-AC-13 | ✅ |
| TC-564 | Scanner recognizes ``` and `~~~` fences with matching-character close: `~~~mermaid` extracted as `mermaid`; cross-char fence line is content; unclosed `~~~` flushed as final block; section-owned `code_block` resolves a `~~~` block | Unit | P0 | FR-011-AC-14 | ✅ |
| TC-080 | Registry::from_env() with neither search-path env var (IX_FILAMENT_MODULES_PATH / IX_SCHEMA_PATH) set and no default dir → empty registry, no error | Unit | P0 | FR-013-AC-1 | 🚧 |
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
| TC-110 | Fallback chain resolves via second locator + emits FallbackLocatorUsed | Unit | P0 | FR-016-AC-1 | 🚧 |
| TC-111 | Fallback chain resolves via first locator + no fallback diagnostic | Unit | P0 | FR-016-AC-2 | 🚧 |
| TC-112 | Fallback chain all-miss with required:false omits key | Unit | P1 | FR-016-AC-3 | 🚧 |
| TC-113 | domain object_type from spec-objects-business with legacy heading: parity vs python | Parity | P0 | FR-016-AC-4 | 🚧 |
| TC-120 | Bench: 10 000 sequential renders after load → median <1ms, zero I/O | Bench | P0 | NFR-007-AC-2 | 🚧 |
| TC-121 | Tracing audit: zero Template::parse and zero JSONSchema::compile during render | Static | P0 | NFR-007-AC-3 | 🚧 |
| TC-122 | Long-running soak: registry memory footprint flat over 1 M renders | Soak | P1 | NFR-007-AC-4 | 🚧 |
| TC-130 | Loader symlink-loop detected; warning emitted; cycle skipped | Integration | P0 | FR-013-AC-7 | 🚧 |
| TC-131 | Duplicate IX_SCHEMA_PATH entries: modules loaded once | Integration | P0 | FR-013-AC-8 | 🚧 |
| TC-132 | Registry: Send + Sync (compile-time assertion) | Compile | P0 | FR-013-AC-9 | 🚧 |
| TC-133 | Path-entry-is-a-file: warning emitted; other entries process | Integration | P1 | FR-013-AC-10 | 🚧 |
| TC-134 | Two modules same name → DuplicateModuleName diag + first-wins | Integration | P0 | FR-014-AC-6 | 🚧 |
| TC-135 | Manifest without name uses parent dir name + diagnostic | Unit | P1 | FR-014-AC-7 | 🚧 |
| TC-150 | DSL with both match and iterate_over → ArchetypeLoadError at load | Unit | P0 | FR-011-AC-6 | 🚧 |
| TC-151 | DSL with unknown key → ArchetypeLoadError at load | Unit | P0 | FR-011-AC-7 | 🚧 |
| TC-152 | iterate_over.section_path missing → empty records + IterateRootMissing | Unit | P0 | FR-011-AC-8 | 🚧 |
| TC-160 | Template with {% include %} → ArchetypeLoadError | Unit | P0 | FR-004-AC-4 (RETIRED) | ⛔ RETIRED — render removed |
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
| TC-204 | CI workflow includes render_parity job (not just test job) | Static | P0 | US-005-AC-2 (RETIRED), US-005-AC-3 (RETIRED), StR-002-AC-3 | 🚧 |
| TC-205 | A patch making merged value invalid (title="") returns SchemaViolation, not a render error | Unit | P0 | US-004-AC-2 | 🚧 |
| TC-206 | Bench: bench_patch_render_fr median < 1ms for typical FR | Bench | P1 | US-004-AC-3 | 🚧 |
| TC-330 | Cargo.toml uses tilde/equals pins for load-bearing deps | Static | P0 | NFR-009-AC-1 | 🚧 |
| TC-331 | spec/assets/adr/0001-validator-crate.md exists with chosen crate + bench numbers | Static | P0 | NFR-009-AC-2 | 🚧 |
| TC-332 | Static: no load-bearing dep has unbounded version | Static | P0 | NFR-009-AC-3 | 🚧 |
| TC-340 | Public enums are #[non_exhaustive] | Compile | P0 | NFR-010-AC-2 | 🚧 |
| TC-341 | CHANGELOG.md exists with release entries | Static | P1 | NFR-010-AC-3 | 🚧 |
| TC-342 | cargo-semver-checks against previous tag reports no unexpected breaks | Static | P1 | NFR-010-AC-4 | 🚧 |
| TC-350 | All 6 fuzz targets compile and run cleanly for 60s on baseline | Integration | P0 | NFR-011-AC-1, NFR-011-AC-2 | 🚧 |
| TC-351 | .github/workflows/fuzz.yml runs all targets weekly | Static | P0 | NFR-011-AC-3 | 🚧 |
| TC-352 | Discovered crash reproducer committed under fuzz/corpus + regression test | Integration | P1 | NFR-011-AC-4 | 🚧 |
| TC-360 | (RETIRED — ADR 0006) miri CI job removed; first-party safety is compile-time `forbid(unsafe_code)` (NFR-003-AC-5) | Static | P0 | NFR-012-AC-1 (RETIRED) | ⊘ |
| TC-361 | (RETIRED — ADR 0006) miri job removed | Integration | P0 | NFR-012-AC-3 (RETIRED) | ⊘ |
| TC-362 | (RETIRED — ADR 0006) miri job removed | Process | P0 | NFR-012-AC-4 (RETIRED) | ⊘ |
| TC-370 | cargo-mutants config declares parser/extract/edges target paths | Static | P0 | NFR-013-AC-1 | 🚧 |
| TC-371 | CI workflow runs cargo-mutants weekly + workflow_dispatch | Static | P0 | NFR-013-AC-2 | 🚧 |
| TC-372 | mutants report uploaded as CI artifact | Static | P1 | NFR-013-AC-3 | 🚧 |
| TC-373 | mutants_baseline.txt tracks accepted survivors with rationale | Static | P1 | NFR-013-AC-4 | 🚧 |
| TC-380 | cargo-audit runs on PR + push + daily schedule | Static | P0 | NFR-014-AC-1 | 🚧 |
| TC-381 | Ignored advisory has one-line rationale in deny.toml | Static | P0 | NFR-014-AC-2 | 🚧 |
| TC-382 | Test PR adding a vulnerable crate fails audit job | Integration | P0 | NFR-014-AC-3 | 🚧 |
| TC-400 | Heading `## Behavior {#blk-7af2}` parses into QuireSection.block_id = "blk-7af2"; heading text = "Behavior" | Unit | P0 | FR-019-AC-1 | ✅ |
| TC-401 | Round-trip: parse → apply_block_patch → reparse — block_id stays "blk-7af2" | Integration | P0 | FR-019-AC-2 | ✅ |
| TC-402 | Pandoc attribute stripped from heading text on parse (no `{#…}` trailing in `QuireSection.heading`) | Unit | P0 | FR-019-AC-3 | ✅ |
| TC-403 | Heading without `{#…}` → block_id = None; heading text byte-identical to input | Unit | P0 | FR-019-AC-1 (negative) | ✅ |
| TC-410 | QuireSection.block_id is the canonical addressing primitive; find_block_by_id walks nested sections | Unit | P0 | FR-020-AC-1 | ✅ |
| TC-411 | Registry::block_type(name) returns the same CompiledArchetype as archetype(name) | Unit | P1 | FR-020-AC-2 | ✅ |
| TC-420 | apply_block_patch merges patch onto current_data → validates → renders → splices; target block bytes updated | Unit | P0 | FR-021-AC-1 | ✅ |
| TC-421 | replace_block full-replaces data + renders + splices; no merge of prior fields | Unit | P0 | FR-021-AC-2 | ✅ |
| TC-422 | apply_block_patch with merged data violating schema → SchemaViolation; no writeback | Unit | P0 | FR-021-AC-3 | ✅ |
| TC-423 | apply_block_patch with unknown block_type → UnknownArchetype | Unit | P0 | FR-021-AC-4 | ✅ |
| TC-424 | apply_block_patch with unknown block_id → MissingField; doc unchanged | Unit | P0 | FR-021-AC-5 | ✅ |
| TC-425 | LLM-flow: bytes spliced by apply_block_patch equal direct template render of merged data | Integration | P0 | FR-021-AC-6 | ✅ |
| TC-430 | update_section replaces heading's content range; heading line + frontmatter + other sections byte-identical | Unit | P0 | FR-022-AC-1 | ✅ |
| TC-431 | update_block replaces heading + content range together; addresses by block_id, finds nested blocks | Unit | P0 | FR-022-AC-2 | ✅ |
| TC-432 | After update_block, untouched blocks byte-identical (incl. trailing whitespace + nested bullets) | Unit | P0 | FR-022-AC-3 | ✅ |
| TC-433 | Frontmatter (`---\nid: …\n---\n`) byte-identical through update_section + update_block | Unit | P0 | FR-022-AC-4 | ✅ |
| TC-434 | update_section unknown heading → MissingField | Unit | P0 | FR-022-AC-5 (negative) | ✅ |
| TC-435 | update_block unknown block_id → MissingField | Unit | P0 | FR-022-AC-5 (negative) | ✅ |
| TC-440 | End-to-end: parse FR-like artifact, apply_block_patch, assert only patched block's bytes changed | Integration | P0 | FR-019..022 composite | ✅ |
| TC-441 | End-to-end: replace_block renders fresh data into existing block bytes | Integration | P0 | FR-021-AC-2, FR-022-AC-2 | ✅ |
| TC-442 | End-to-end: empty patch is idempotent (rendered bytes equal current data) | Integration | P1 | FR-021-AC-1 | ✅ |
| TC-443 | End-to-end: block_id survives parse → patch → reparse | Integration | P0 | FR-019-AC-2 | ✅ |
| TC-450 | Bench: `apply_block_patch` p50 < 1 ms on 10 KB / 5-block doc; p99 < 5 ms; memory-flat across iterations | Bench | P0 | US-006-PC-1..4 | 🚧 |
| TC-451 | Bench: `replace_block` p50 < 1 ms on 10 KB / 5-block doc; ±10% of TC-450; report crossover where replace beats patch on large blocks | Bench | P0 | US-007-PC-1, US-007-PC-4 | 🚧 |
| TC-452 | Bench: 10 sequential block patches on 20 KB doc; p50 < 10 ms; assert linear-in-N (no superlinear regression); document block_id-lookup cost on > 100-block doc | Bench | P0 | US-008-PC-1, US-008-PC-5 | 🚧 |
| TC-453 | Bench: `parse_document` + `extract` (multi-yield, ~10 records) on 10 KB doc; p50 < 2 ms | Bench | P0 | US-010-PC-1 | 🚧 |
| TC-454 | Bench: corpus-scale extract (100 docs, 10 records each) single-threaded p50 < 200 ms; 8-thread p50 < 50 ms | Bench | P1 | US-010-PC-3 | 🚧 |
| TC-455 | Bench: `load_repo` 1k-doc corpus at 1 + 8 threads; p50 < 600 ms / < 200 ms; parallel efficiency ≥ 0.6; output path-sorted | Bench | P0 | FR-024-AC-8, NFR-015-AC-1..4, US-011-PC-1 | 🚧 |
| TC-456 | Bench: 500+ doc corpus through Python binding ≥ 5× faster than pure-Python filament_parser path | Bench | P0 | StR-005-AC-3, US-011-PC-3 | 🚧 |
| TC-457 | Bench: `Spec` construct (load + resolve) for 200-artifact spec p50 < 50 ms single-thread | Bench | P0 | US-012-PC-1 | 🚧 |
| TC-458 | Bench: `by_id` / `referencing` / `orphans` sub-millisecond per query over 200-artifact corpus | Bench | P1 | FR-027-AC-8, US-012-PC-2 | 🚧 |
| TC-459 | Bench: resolve all references in 200-artifact spec p50 < 5 ms (part of construct budget) | Bench | P1 | US-013-PC-1 | 🚧 |
| TC-460 | `cargo build` (no features) and `--features python` both succeed; no pyo3 linkage in default build | Static | P0 | FR-023-AC-1, StR-005-AC-2 | 🚧 |
| TC-461 | `quire.parse_document(text)` returns frontmatter/headings/block-ids matching Rust `parse_document` | Integration | P0 | FR-023-AC-2, StR-005-AC-1 | 🚧 |
| TC-462 | `quire.validate(bad, "fr")` violation field-path equals Rust `validate` for same input | Integration | P0 | FR-023-AC-3 | 🚧 |
| TC-463 | `quire.load_repo(path)` returns one doc per `.md` + per-file diagnostics via binding | Integration | P0 | FR-023-AC-4, US-011-AC-1, US-011-AC-2 | 🚧 |
| TC-464 | Two Python threads each calling `load_repo` complete < 2× single-call (GIL released) | Integration | P0 | FR-023-AC-5, NFR-016-AC-2, US-011-AC-5 | 🚧 |
| TC-465 | One abi3 wheel imports + smoke-tests under two CPython 3.x minor versions | Integration | P0 | FR-023-AC-6, StR-005-AC-5, NFR-016-AC-3 | 🚧 |
| TC-466 | No `subprocess`/`Popen`/socket on the binding data path (static grep + runtime assert) | Static | P0 | FR-023-AC-7, StR-005-AC-4 | 🚧 |
| TC-467 | Binding returns structured objects; no Python-side markdown/frontmatter re-parse | Integration | P0 | US-011-AC-3, NFR-016-AC-4 | 🚧 |
| TC-469 | Bench: per-FFI-crossing overhead for `parse_document` < 50 µs over equivalent Rust call | Bench | P1 | NFR-016-AC-1, US-011-PC-2 | 🚧 |
| TC-470 | `load_repo` over N-file tree returns N LoadedDocuments matching direct parse_document | Integration | P0 | FR-024-AC-1 | 🚧 |
| TC-471 | One malformed file → N-1 good docs + exactly one diagnostic; no panic/error | Integration | P0 | FR-024-AC-2, US-011-AC-2 | 🚧 |
| TC-472 | `.gitignore` subtree skipped by default; parsed when WalkOptions disables ignore-files | Integration | P0 | FR-024-AC-3 | 🚧 |
| TC-473 | `documents` path-sorted; two runs byte-identical ordering + content | Property | P0 | FR-024-AC-4, NFR-006 | 🚧 |
| TC-474 | Symlink loop in tree → warning diagnostic, no infinite walk | Integration | P0 | FR-024-AC-5 | 🚧 |
| TC-475 | `LoadedDocument.id`=frontmatter `id`, `.uuid`=frontmatter `uuid` (UUID7); neither derived, no file write; missing uuid→MissingUuid, missing id→UntypedArtifact (non-fatal) | Unit | P0 | FR-024-AC-6, US-011-AC-4 | 🚧 |
| TC-476 | `root` that is a file or nonexistent → empty RepoLoad + one warning (no error/panic) | Unit | P1 | FR-024-AC-7 | 🚧 |
| TC-480 | `Spec::from_path` `len()` equals parsed-artifact count under directory | Integration | P0 | FR-025-AC-1 | 🚧 |
| TC-481 | Corpus indexes every doc by id; by_id present → Some, absent → None | Unit | P0 | FR-025-AC-2 | 🚧 |
| TC-482 | Two docs sharing an id → DuplicateArtifactId diagnostic; first-wins lookup; construct succeeds | Unit | P0 | FR-025-AC-3 | 🚧 |
| TC-483 | `Spec: Send + Sync` compile-time assertion | Compile | P0 | FR-025-AC-4, StR-006-AC-5 | 🚧 |
| TC-484 | API-surface test: corpus exposes no persistence/watcher/external-resolution method | Static | P0 | FR-025-AC-5, StR-006-AC-4 | 🚧 |
| TC-485 | Queries answer with zero filesystem read post-construction (tracing/strace audit) | Static | P0 | FR-025-AC-6, StR-006-AC-1, US-012-AC-5 | 🚧 |
| TC-486 | Frontmatter `relationships` entry to present id → Resolved edge (src/target/type) | Unit | P0 | FR-026-AC-1, US-013-AC-1 | 🚧 |
| TC-487 | `ix://` body link to present id → Resolved edge in same edge set | Unit | P0 | FR-026-AC-2, US-013-AC-2 | 🚧 |
| TC-488 | Reference to absent id → Dangling edge + queryable diagnostic; construct succeeds | Unit | P0 | FR-026-AC-3, StR-006-AC-3, US-013-AC-3 | 🚧 |
| TC-489 | Target id existing only in a different spec → Dangling; no filesystem access during resolution | Integration | P0 | FR-026-AC-4, US-013-AC-4 | 🚧 |
| TC-490 | Resolved edge appears in both `referencing(target)` and `outgoing(source)` | Unit | P0 | FR-026-AC-5, US-013-AC-5 | 🚧 |
| TC-491 | `ix://…/FR-021` and bare `FR-021` both extract target_id "FR-021"; resolve identically | Unit | P0 | FR-026-AC-6 | 🚧 |
| TC-492 | Proptest: resolution time linear in edge count; classification identical across thread counts | Property | P0 | FR-026-AC-7, US-013-PC-2, US-013-PC-3, NFR-006 | 🚧 |
| TC-493 | `by_type("FR")`/`by_type("US")` return exactly those artifacts | Unit | P0 | FR-027-AC-1, US-012-AC-1 | 🚧 |
| TC-494 | `referencing("FR-021")` returns every referencing artifact; excludes non-referencing | Unit | P0 | FR-027-AC-2, US-012-AC-3 | 🚧 |
| TC-495 | `orphans("FR","implements",Some("StR"))` returns FRs lacking that edge; excludes those with it | Unit | P0 | FR-027-AC-3, US-012-AC-2 | 🚧 |
| TC-496 | US with no resolved test-case edge returned by coverage query; one with edge excluded | Unit | P0 | FR-027-AC-4, US-012-AC-4 | 🚧 |
| TC-497 | `outgoing` includes dangling edges; every dangling edge in `dangling()`, none in any `referencing` | Unit | P0 | FR-027-AC-5 | 🚧 |
| TC-498 | Query iterators yield sorted-by-id; two runs identical sequences | Property | P0 | FR-027-AC-6, US-012-PC-3, NFR-006 | 🚧 |
| TC-499 | Zero filesystem read during any query after construction (tracing audit) | Static | P0 | FR-027-AC-7, US-012-AC-5 | 🚧 |
| TC-500 | Untyped doc (no `type` field): excluded from `by_type`, found by `by_id`, emits UntypedArtifact diagnostic | Unit | P1 | FR-027-AC-9 | 🚧 |
| TC-501 | Identical edge from frontmatter + body deduped to one; same-pair different-type kept as two | Unit | P0 | FR-026-AC-8 | 🚧 |
| TC-502 | Static audit: no Mutex/RwLock/Atomic in first-party src/; parallel parse collects owned results | Static | P0 | FR-024-AC-9 | 🚧 |
| TC-503 | loom: parallel parse collection race-free; identical path-sorted output across all interleavings | Property | P0 | NFR-017-AC-1..3 | 🚧 |
| TC-504 | TSAN lane: two-thread `load_repo` (GIL-release window) reports zero data races | Integration | P0 | NFR-018-AC-1, NFR-018-AC-3 | 🚧 |
| TC-505 | ASAN lane: FFI object-handoff test set reports zero leaks/UAF (interpreter noise suppressed) | Integration | P0 | NFR-018-AC-2, NFR-018-AC-3 | 🚧 |
| TC-506 | `rg 'unsafe {' src/` returns zero matches with `--features python` enabled | Static | P0 | NFR-003-AC-4 | 🚧 |
| TC-507 | (RETIRED — ADR 0006) miri job removed; FFI scope note moot | Static | P1 | NFR-012-AC-5 (RETIRED) | ⊘ |
| TC-582 | Crate root carries `#![cfg_attr(not(feature = "python"), forbid(unsafe_code))]`; default `cargo build` compiles (compiler proves zero first-party unsafe) and adding a first-party `unsafe` block fails the default build; `--features python` compiles with forbid scoped off | Static | P0 | NFR-003-AC-5 | 🚧 |
| TC-510 | `quire.render(archetype, module_root, data)` byte-equals `quire_rs::render_by_name` for same inputs | Integration | P0 | FR-028-AC-1 (RETIRED) | ⛔ RETIRED — render removed |
| TC-511 | `quire.validate` returns None on valid data; raises `QuireValidationError` with dotted field path on invalid | Integration | P0 | FR-028-AC-2, NFR-005 | 🚧 |
| TC-512 | `quire.validate_manifest`: happy path returns `[]`; bad payload returns structured `{path, message, schema_keyword}` records; missing schema raises `QuireSchemaError` | Integration | P0 | FR-028-AC-3 | 🚧 |
| TC-513 | `quire.extract(arch, mod_root, text)` returns `{extraction, edges}` dict; `extraction` matches Rust `extract().records` | Integration | P0 | FR-028-AC-4 | 🚧 |
| TC-514 | `quire.extract_frontmatter(text)` returns Rust-produced `{frontmatter, body}`; BOM/CRLF/malformed cases match FR-006 with no Python-side split | Integration | P0 | FR-028-AC-5 | 🚧 |
| TC-515 | `quire.harvest_edges(text)` and `quire.harvest_edges(parse_document(text))` return equal deduplicated lists | Integration | P0 | FR-028-AC-6, FR-026 | 🚧 |
| TC-516 | `Quire{Base,Render,Validation,Schema,Parse}Error` are importable and subclass `QuireBaseError` / `Exception` | Integration | P0 | FR-028-AC-7 | 🚧 |
| TC-517 | Two-thread concurrent call to each new module-level function completes wall-clock < 2× single-call | Integration | P1 | FR-028-AC-8, NFR-016-AC-2 | 🚧 |
| TC-518 | Conformant authored FR markdown passes `validate_document` with no errors (unified shape + markdown validate) | Integration | P0 | US-014-AC-1, FR-031, FR-032 | 🚧 |
| TC-519 | Authored artifact with missing section / wrong AC-table columns / mis-prefixed AC id fails with line-numbered diagnostic | Integration | P0 | US-014-AC-2, FR-032, FR-033, FR-034 | 🚧 |
| TC-520 | Authored artifact with two same-level identical-text headings fails with `duplicate-heading` diagnostic | Integration | P0 | US-014-AC-3, FR-035 | 🚧 |
| TC-521 | Same archetype `body_extraction` both validates the document and extracts its record (one declaration, two postures) | Integration | P0 | US-014-AC-4, FR-031 | 🚧 |
| TC-522 | Manifest with `frontmatter_schema_ref`+`body_extraction` compiles to one CompiledArchetype that is validatable (frontmatter schema) and extractable (resolvable body contract); no renderability concept exposed | Unit | P0 | FR-031-AC-1 | 🚧 |
| TC-523 | Manifest with `frontmatter_schema_ref` but no `body_extraction` compiles and is validatable; `body_extraction()` returns `None` | Unit | P0 | FR-031-AC-2 | 🚧 |
| TC-524 | `defaults.id_pattern`, `allowed_links`, `has_plugin`, `grammar_ref` retained on compiled archetype + readable via accessors | Unit | P0 | FR-031-AC-3 | 🚧 |
| TC-525 | `frontmatter_schema_ref` and `data_schema` retained as two distinct compiled validators; neither collapsed | Unit | P0 | FR-031-AC-4 | 🚧 |
| TC-526 | Manifest still declaring `required_sections` loads, field ignored, exactly one non-fatal diagnostic pointing to `body_extraction` | Unit | P0 | FR-031-AC-5 | 🚧 |
| TC-527 | `Registry::archetype(name)` resolves unified archetype identically to pre-unification (same keying + first-wins) | Unit | P0 | FR-031-AC-6 | 🚧 |
| TC-528 | Conformant FR document (locators resolved, asserts satisfied, frontmatter valid) → `is_valid==true`, no errors | Integration | P0 | FR-032-AC-1 | 🚧 |
| TC-529 | Document missing a `required` section → line-numbered diagnostic naming archetype + section + reason `missing` | Unit | P0 | FR-032-AC-2 | 🚧 |
| TC-530 | Required `## Specification` containing only `TODO`/`{{...}}` → reason `placeholder` even when frontmatter schema passes | Unit | P0 | FR-032-AC-3 | 🚧 |
| TC-531 | Frontmatter violating `frontmatter_schema_ref` → reason `frontmatter`, independent of body structure | Unit | P0 | FR-032-AC-4 | 🚧 |
| TC-532 | `validate_document` (markdown) vs legacy context/data path (FR-002) are distinct entry points; context path validates JSON, no markdown parse | Unit | P0 | FR-032-AC-5 | 🚧 |
| TC-533 | Archetype with no `body_extraction` validates by frontmatter schema + heading-uniqueness only; no body-structure diagnostics | Unit | P0 | FR-032-AC-6 | 🚧 |
| TC-534 | `section_body` locator `assert: {level: 2}` fails when resolved heading not level 2; passes when it is | Unit | P0 | FR-033-AC-1 | 🚧 |
| TC-535 | `table_row` locator `assert: {columns: [...]}` fails on differing header text/order; passes on exact match | Unit | P0 | FR-033-AC-2 | 🚧 |
| TC-536 | `assert: {min_rows: 1}` fails on header-only table; `assert: {min_items: 1}` fails on empty list | Unit | P0 | FR-033-AC-3 | 🚧 |
| TC-537 | `assert: {id_column, id_pattern}` fails when any id cell mismatches; passes when all match | Unit | P0 | FR-033-AC-4 | 🚧 |
| TC-538 | Load-time-invalid assert (unknown key, or `columns` on `section_body`) → `ArchetypeLoadFailure` naming archetype + locator | Unit | P0 | FR-033-AC-5 | 🚧 |
| TC-539 | Extraction ignores the `assert` facet entirely (extracted value identical with and without `assert`) | Unit | P0 | FR-033-AC-6 | 🚧 |
| TC-540 | `id_pattern: '^{id}-AC-\d+$'` with `id: FR-900` accepts `FR-900-AC-1/2`, rejects `FR-901-AC-1` | Unit | P0 | FR-034-AC-1 | 🚧 |
| TC-541 | `{field}` referencing absent frontmatter key → diagnostic naming archetype + locator + missing field; assert does not pass | Unit | P0 | FR-034-AC-2 | 🚧 |
| TC-542 | Frontmatter value with regex metacharacters (`id: A.B+`) is regex-escaped; `{id}` matches literal value | Unit | P0 | FR-034-AC-3 | 🚧 |
| TC-543 | Assert pattern with no `{field}` token behaves as plain static regex (no interpolation pass observable) | Unit | P0 | FR-034-AC-4 | 🚧 |
| TC-544 | Two `## Description` headings → reason `duplicate-heading` naming text + level 2 | Unit | P0 | FR-035-AC-1 | 🚧 |
| TC-545 | `## Properties` (L2) + `### Properties` (L3) passes uniqueness (different levels) | Unit | P0 | FR-035-AC-2 | 🚧 |
| TC-546 | `iterate_over` with distinct child headings (`### A`, `### B`) passes; duplicate `### A` fails | Unit | P0 | FR-035-AC-3 | 🚧 |
| TC-547 | Duplicate-heading diagnostic includes line number of the offending (second) heading | Unit | P0 | FR-035-AC-4 | 🚧 |
| TC-548 | `input_contract_for(registry, "FR")` returns FR frontmatter schema, required sections (Description, Specification, Acceptance Criteria, Dependencies), and populating template variables | Unit | P0 | FR-029-AC-1 | 🚧 |
| TC-549 | `input_contract_for(registry, "NFR")` returns NFR required sections + variables feeding Scope, Measurement and Evaluation, Verification | Unit | P0 | FR-029-AC-2 | 🚧 |
| TC-550 | For every `spec-artifacts-iso` archetype, contract contains each manifest `required_sections` entry exactly once, in manifest order | Integration | P0 | FR-029-AC-3 | 🚧 |
| TC-551 | Contract JSON serialization byte-identical across repeated calls against the same loaded module | Property | P0 | FR-029-AC-4 | 🚧 |
| TC-552 | `input_contract_for(registry, "nonexistent")` → `Err(QuireError::UnknownArchetype)` | Unit | P0 | FR-029-AC-5 | 🚧 |
| TC-553 | Required section whose variables cannot be mapped still yields contract with that section + unresolved-mapping diagnostic (no silent omit) | Unit | P0 | FR-029-AC-6 | 🚧 |
| TC-554 | `CompiledArchetype::body_extraction` (field) and `body_extraction()` (accessor) return `Some(ExtractionDsl)` for declaring archetypes, `None` otherwise; same parsed value validated at load | Unit | P0 | FR-013-AC-11 | 🚧 |
| TC-555 | `Registry::load_module(module_root)` loads exactly the named module, does NOT walk `module_root.parent()` siblings (sibling module not loaded) | Integration | P0 | FR-013-AC-12 | 🚧 |
| TC-556 | `load_module` against a dir with no `manifest.yaml` → zero modules + single `ArchetypeLoadFailure`; siblings not promoted | Integration | P0 | FR-013-AC-13 | 🚧 |
| TC-557 | `Diagnostic::PathTraversal{argument,path,reason}` variant: Display + to_json carry name/argument/path/reason across all three `PathTraversalReason` values | Unit | P0 | FR-013-AC-14 | 🚧 |
| TC-558 | `ExtractionContext.from_object_types([...]).extract(name,text)` returns same records + edges as Rust extractor; no module-root/`.ix` read | Integration | P0 | FR-028-AC-9, US-003-AC-4 | 🚧 |
| TC-559 | `ExtractionContext` accepts both a bare list of ObjectType dicts and the core envelope `{items: [...]}` | Integration | P0 | FR-028-AC-10 | 🚧 |
| TC-560 | Python `ExtractionContext.from_object_types([...])` extracts a real document from in-memory snapshot without reading module root / `.ix` / package manifest | Integration | P0 | US-003-AC-4 | 🚧 |
| TC-561 | Multi-yield archetype (`iterate_over` + `per_match`): `validate_document` runs each `per_match` required-locator + `assert` against every iteration unit's local scope — a conformant document passes; a unit missing a required sub-locator → reason `missing`; a unit violating an assert → reason `assert` | Unit | P0 | FR-032-AC-2, FR-033-AC-4 | 🚧 |
| TC-562 | Registry path: a module loaded via `Registry` whose archetype carries an `assert` facet (AC table `columns` + interpolated `id_pattern`) runs end-to-end through `validate_document` (manifest → load → validate); mis-prefixed id / wrong columns → reason `assert`, conformant passes | Integration | P0 | FR-033-AC-4, FR-033-AC-9 | 🚧 |
| TC-565 | Locator `regex:` projection: `(\d+)` yields capture group 1; `\d+` (no group) yields group 0; non-match drops key (required:false) or returns `MissingField` (required:true); invalid regex → empty projected value, no panic | Unit | P0 | FR-011-AC-15 | ✅ |
| TC-566 | `under_section: None` substrate: `table_row` resolves against joined section bodies using the first table (first-then-any for a required column); `list_item`/`code_block` read the joined-body substrate | Unit | P0 | FR-011-AC-16 | ✅ |
| TC-567 | Whole-value `{{ id }}` resolved value contributes no extracted value (placeholder); an embedded `{{x}}` mid-prose does not trigger the rule and surrounding content extracts normally | Unit | P0 | FR-011-AC-17 | ✅ |
| TC-568 | Unclosed fenced block (both ` ``` ` and `~~~`) flushed as the final block; trailing content is part of the block, not a phantom following block (parity with FR-007) | Unit | P0 | FR-011-AC-18 | ✅ |
| TC-569 | `emit_edges: [{from, type}]` projects one `{record_index, type, target}` edge per extracted record whose field resolves; records lacking the field emit none; distinct from `harvest_edges`; flows through the Python `extract()` envelope `edges` key | Unit | P0 | FR-011-AC-19 | ✅ |
| TC-581 | `from: heading` locator normalizes the ISO section-number prefix: `## 2. Scope` matches a `regex: ^Scope$` heading locator and a numbered master spec validates against the master-requirements archetype (parity with `section_body`/`after_heading`) | Integration | P1 | FR-011-AC-20 | ✅ |
| TC-583 | `multiple: true` keeps every located value as a JSON array (two mermaid blocks under Workflow → both yielded); absent flag → first-wins unchanged; fallback chain reads the flag from the hit primitive; multi-yield keeps per-unit lists | Unit | P0 | FR-011-AC-21 | ✅ |
| TC-584 | Manifest `lint_rules` parse typed and surface via `Registry::lint_rules()` in load order; a malformed rule (unknown `type:`) fails module load with an `ArchetypeLoadFailure` | Unit | P0 | FR-036-AC-1 | ✅ |
| TC-585 | AC Verification vocabulary rule: `Test (TC-035)` and `Inspection` cells pass, `Docs audit` yields one finding naming rule id, row, allowed set; severity mirrors rule (warning and error variants) | Unit | P0 | FR-036-AC-2 | ✅ |
| TC-586 | Rule scoped `archetypes: [FR]` yields no findings against an NFR document or an unresolvable archetype | Unit | P0 | FR-036-AC-3 | ✅ |
| TC-587 | Missing section / table / column yields zero lint findings (structure is FR-032's job) | Unit | P0 | FR-036-AC-4 | ✅ |
| TC-588 | With lint rules loaded, `extract()` and `validate_document()` results are byte-identical to the rule-free run on the same document | Unit | P0 | FR-036-AC-5 | ✅ |
| TC-570 | Assert-key × locator-kind legality matrix: each legal cell loads (`level`@section_body/heading; `columns`/`min_rows`/`id_column`@table_row; `min_items`@list_item; `id_pattern`@all listed); each illegal cell → `ArchetypeLoadFailure` naming archetype+locator+key (table-driven) | Unit | P0 | FR-033-AC-7 | ✅ |
| TC-571 | `id_column` resolution precedence: `assert.id_column` → locator `column` → column 0; all-three present resolves to `assert.id_column`; `id_column` absent → `column`; both absent → col 0 | Unit | P0 | FR-033-AC-8 | ✅ |
| TC-572 | `id_pattern` on non-table locators: matches heading text (`heading`), section first-line/id token (`section_body`), each item (`list_item`), frontmatter scalar (`frontmatter_field`); mismatch → reason `assert`, match passes | Unit | P0 | FR-033-AC-9 | ✅ |
| TC-573 | Placeholder sentinel set exact: `TODO:`/`TBD` prefix (case-insensitive) and whole-value `{{…}}`/`placeholder`/`none specified`/empty fail with reason `placeholder`; substantive prose containing `todo` mid-sentence or an embedded `{{x}}` does not | Unit | P0 | FR-032-AC-7 | ✅ |
| TC-574 | A required section whose only content is `none` or `n/a` (e.g. `Upstream: none`) is substantive and passes — proving bare `none`/`n/a` are not sentinels | Unit | P0 | FR-032-AC-8 | ✅ |
| TC-575 | Required `table_row` → header-only table fails reason `empty`; required `list_item` → item-less list fails reason `empty`; a non-resolving locator fails reason `missing` (none report `placeholder`) | Unit | P0 | FR-032-AC-9 | ✅ |
| TC-576 | `assert` on a **resolved** locator is evaluated regardless of `required`: an optional locator that resolves but violates its assert → reason `assert`; an optional locator that does not resolve runs no assert and emits no diagnostic | Unit | P0 | FR-032-AC-10 | ✅ |
| TC-577 | Bench: `bench_validate_document` on a typical FR-sized artifact median <1ms (warm registry); >10% vs baseline fails CI | Bench | P0 | NFR-002-AC-4 | ✅ |
| TC-578 | Determinism: `validate_document` + `extract` on the same input 100× across threads → equal `ValidationResult` (ordered diagnostics) + `ExtractionResult` (records+edges+diagnostics) | Property | P0 | NFR-006-AC-4 | ✅ |
| TC-579 | Fuzz: arbitrary byte slices (lossy `&str`) into `parse_document`/`validate_document`/`extract` run clean (no panic/UB) for the scheduled duration; crashes committed as regression reproducers | Integration | P0 | NFR-019-AC-1 | ✅ |
| TC-580 | Proptest: random strings (empty, fence-only, frontmatter-only, deeply nested) into `parse_document`/`validate_document`/`extract` always return a value or typed error, never panic | Property | P0 | NFR-019-AC-2 | ✅ |
| TC-590 | `validate_base_concept` on `{type: FR}` returns no errors (minimal typed concept) | Unit | P0 | FR-037-AC-1 | ✅ |
| TC-591 | `validate_base_concept` on `{type, description: str, tags: [str]}` returns no errors (optional OKF fields accepted when well-typed) | Unit | P0 | FR-037-AC-2 | ✅ |
| TC-592 | `validate_base_concept` on frontmatter omitting `type` → exactly one error, reason `frontmatter`, message names `type` | Unit | P0 | FR-037-AC-3 | ✅ |
| TC-593 | `validate_base_concept` on `{type: ""}` → exactly one error, reason `frontmatter` (`minLength: 1`) | Unit | P0 | FR-037-AC-4 | ✅ |
| TC-594 | `validate_base_concept` on `{type, description: 7}` → one error naming `description` | Unit | P0 | FR-037-AC-5 | ✅ |
| TC-595 | `validate_base_concept` on `{type, tags: "x"}` (non-array) → one error naming `tags` | Unit | P0 | FR-037-AC-5 | ✅ |
| TC-596 | `validate_base_concept` on `{type, tags: ["ok", 3]}` (non-string item) → one error | Unit | P0 | FR-037-AC-5 | ✅ |
| TC-600 | `validate_bundle_at` Strict: a doc with no `type` → `!is_valid()`, error reason `frontmatter` naming `type` (untyped is a hard error) | Integration | P0 | FR-038-AC-1 | ✅ |
| TC-601 | `validate_bundle_at` Okf: untyped doc is **still** an error (`!is_valid()`, reason `frontmatter`) | Integration | P0 | FR-038-AC-2 | ✅ |
| TC-602 | Okf tolerates unknown `type` + dangling `ix://` ref as warnings (`is_valid()`); Strict rejects both as errors | Integration | P0 | FR-038-AC-3 | ✅ |
| TC-603 | Strict: typed archetype-conformant bundle whose root index lists every sibling + declares `okf_version` → `is_valid()` | Integration | P0 | FR-038-AC-4 | ✅ |
| TC-604 | Index omitting a sibling → `index-incomplete` naming the file: error under Strict, warning under Okf | Integration | P0 | FR-038-AC-5 | ✅ |
| TC-605 | Strict: bundle-root index missing `okf_version` → `index-okf-version` error | Integration | P0 | FR-038-AC-6 | ✅ |
| TC-606 | Strict: subdirectory index without `okf_version` produces no `index-okf-version` finding; nested bundle `is_valid()` | Integration | P0 | FR-038-AC-7 | ✅ |
| TC-607 | Strict: known `type` but mistyped optional `description` → `!is_valid()`, error names `description` (base concept contract runs in bundle validation) | Integration | P0 | FR-038-AC-8 | ✅ |

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
| EC-017 | Document uses legacy heading variant | FR-016 | TC-110 | Silent data loss |
| EC-018 | Hot-path render re-reads disk | NFR-007 | TC-084, TC-121 | Per-call cost balloons |
| EC-020 | Heading with multiple `{#…}` sequences (only trailing one is the id) | FR-019 | TC-402 | Wrong block_id parsed; heading text mangled |
| EC-021 | Pandoc attribute with whitespace inside braces (`{# bad}`) — not a valid id | FR-019 | TC-403 | Garbage id slips through |
| EC-022 | apply_block_patch where merged data is valid but template render fails (Jinja error) | FR-021 | TC-422 (proxy via schema path) | Half-written markdown returned |
| EC-023 | update_block on a deeply nested block (level 4 inside level 2) | FR-022 | TC-431 | Find-by-id walks only top-level sections; nested blocks unreachable |
| EC-024 | Round-trip: patch → reparse → patch again preserves block_id stability | FR-019, FR-022 | TC-443 | Block IDs drift across edits |
| EC-025 | `load_repo` over tree with one malformed `.md` among many | FR-024 | TC-471 | One bad file aborts the whole repo load |
| EC-026 | Two artifacts in one spec sharing an id | FR-025 | TC-482 | Silent overwrite in id index; lost document |
| EC-027 | Reference whose target id is absent from the loaded set | FR-026 | TC-488 | Resolution errors instead of recording dangling |
| EC-028 | Reference whose target id exists only in a *different* spec | FR-026 | TC-489 | Corpus reaches outside the loaded set (scope violation) |
| EC-029 | Multi-threaded Python caller issuing concurrent binding calls | FR-023, NFR-016 | TC-464 | GIL not released → calls serialized, no speedup |
| EC-030 | `load_repo` root is a regular file or nonexistent path | FR-024 | TC-476 | Panic or Err instead of empty RepoLoad + warning |
| EC-031 | Parallel parse interleavings produce divergent or unsorted output | FR-024, NFR-017 | TC-503 | Non-deterministic result; hidden data race under load |
| EC-032 | GIL released during `load_repo` while Python thread touches the runtime | FR-023, NFR-018 | TC-504 | Data race between Rust thread and CPython runtime |
| EC-033 | Python object handed from Rust then dropped (refcount/lifetime) | FR-023, NFR-018 | TC-505 | Use-after-free or leak across the FFI boundary |
| EC-034 | First-party `unsafe` sneaks in via the `python` feature | NFR-003 | TC-506 | Zero-unsafe guarantee silently weakened by FFI code |

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
| StR-005-AC-1 | TC-461 |
| StR-005-AC-2 | TC-460 |
| StR-005-AC-3 | TC-456 |
| StR-005-AC-4 | TC-466 |
| StR-005-AC-5 | TC-465 |
| StR-006-AC-1 | TC-485 |
| StR-006-AC-2 | TC-493, TC-494 |
| StR-006-AC-3 | TC-488 |
| StR-006-AC-4 | TC-484, TC-489 |
| StR-006-AC-5 | TC-483 |

### User Stories

| AC | Covering TC(s) |
|---|---|
| US-002-AC-1 | TC-200 |
| US-002-AC-2 | TC-020, TC-021 |
| US-002-AC-3 | TC-002 |
| US-003-AC-1 | TC-018 |
| US-003-AC-2 | TC-019 |
| US-003-AC-3 | TC-073, TC-040 |
| US-003-AC-4 | TC-560, TC-558 |
| US-008-AC-1 | TC-440, TC-432 |
| US-008-AC-2 | TC-440 |
| US-008-AC-3 | TC-420 |
| US-008-AC-4 | TC-443, TC-401 |
| US-010-AC-1 | TC-070 |
| US-010-AC-2 | TC-073 |
| US-010-AC-3 | TC-057 |
| US-010-AC-4 | TC-152 |
| US-010-AC-5 | TC-110 |
| US-011-AC-1 | TC-463 |
| US-011-AC-2 | TC-463, TC-471 |
| US-011-AC-3 | TC-467 |
| US-011-AC-4 | TC-475 |
| US-011-AC-5 | TC-464 |
| US-011-PC-1 | TC-455 |
| US-011-PC-2 | TC-469 |
| US-011-PC-3 | TC-456 |
| US-012-AC-1 | TC-493 |
| US-012-AC-2 | TC-495 |
| US-012-AC-3 | TC-494 |
| US-012-AC-4 | TC-496 |
| US-012-AC-5 | TC-485 |
| US-012-PC-1 | TC-457 |
| US-012-PC-2 | TC-458 |
| US-012-PC-3 | TC-498 |
| US-013-AC-1 | TC-486 |
| US-013-AC-2 | TC-487 |
| US-013-AC-3 | TC-488 |
| US-013-AC-4 | TC-489 |
| US-013-AC-5 | TC-490 |
| US-013-PC-1 | TC-459 |
| US-013-PC-2 | TC-492 |
| US-013-PC-3 | TC-492 |
| US-014-AC-1 | TC-518 |
| US-014-AC-2 | TC-519 |
| US-014-AC-3 | TC-520 |
| US-014-AC-4 | TC-521 |

### Functional Requirements

| AC | Covering TC(s) |
|---|---|
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
| FR-010-AC-4 | TC-589 |
| FR-011-AC-1 | TC-072 |
| FR-011-AC-2 | TC-070 |
| FR-011-AC-4 | TC-073 |
| FR-011-AC-5 | TC-040 |
| FR-011-AC-6 | TC-150 |
| FR-011-AC-7 | TC-151 |
| FR-011-AC-8 | TC-152 |
| FR-011-AC-13 | TC-563 |
| FR-011-AC-14 | TC-564 |
| FR-011-AC-15 | TC-565 |
| FR-011-AC-16 | TC-566 |
| FR-011-AC-17 | TC-567 |
| FR-011-AC-18 | TC-568 |
| FR-011-AC-19 | TC-569 |
| FR-011-AC-20 | TC-581 |
| FR-011-AC-21 | TC-583 |
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
| FR-013-AC-11 | TC-554 |
| FR-013-AC-12 | TC-555 |
| FR-013-AC-13 | TC-556 |
| FR-013-AC-14 | TC-557 |
| FR-014-AC-1 | TC-090 |
| FR-014-AC-2 | TC-091 |
| FR-014-AC-3 | TC-092 |
| FR-014-AC-4 | TC-093 |
| FR-014-AC-5 | TC-094 |
| FR-014-AC-6 | TC-134 |
| FR-014-AC-7 | TC-135 |
| FR-016-AC-1 | TC-110 |
| FR-016-AC-2 | TC-111 |
| FR-016-AC-3 | TC-112 |
| FR-016-AC-4 | TC-113 |
| FR-019-AC-1 | TC-400, TC-403 |
| FR-019-AC-2 | TC-401, TC-443 |
| FR-019-AC-3 | TC-402 |
| FR-020-AC-1 | TC-410 |
| FR-020-AC-2 | TC-411 |
| FR-021-AC-1 | TC-420, TC-442 |
| FR-021-AC-2 | TC-421, TC-441 |
| FR-021-AC-3 | TC-422 |
| FR-021-AC-4 | TC-423 |
| FR-021-AC-5 | TC-424 |
| FR-021-AC-6 | TC-425 |
| FR-022-AC-1 | TC-430 |
| FR-022-AC-2 | TC-431, TC-440 |
| FR-022-AC-3 | TC-432 |
| FR-022-AC-4 | TC-433 |
| FR-022-AC-5 | TC-434, TC-435 |
| FR-023-AC-1 | TC-460 |
| FR-023-AC-2 | TC-461 |
| FR-023-AC-3 | TC-462 |
| FR-023-AC-4 | TC-463 |
| FR-023-AC-5 | TC-464 |
| FR-023-AC-6 | TC-465 |
| FR-023-AC-7 | TC-466 |
| FR-024-AC-1 | TC-470 |
| FR-024-AC-2 | TC-471 |
| FR-024-AC-3 | TC-472 |
| FR-024-AC-4 | TC-473 |
| FR-024-AC-5 | TC-474 |
| FR-024-AC-6 | TC-475 |
| FR-024-AC-7 | TC-476 |
| FR-024-AC-8 | TC-455 |
| FR-024-AC-9 | TC-502 |
| FR-025-AC-1 | TC-480 |
| FR-025-AC-2 | TC-481 |
| FR-025-AC-3 | TC-482 |
| FR-025-AC-4 | TC-483 |
| FR-025-AC-5 | TC-484 |
| FR-025-AC-6 | TC-485 |
| FR-026-AC-1 | TC-486 |
| FR-026-AC-2 | TC-487 |
| FR-026-AC-3 | TC-488 |
| FR-026-AC-4 | TC-489 |
| FR-026-AC-5 | TC-490 |
| FR-026-AC-6 | TC-491 |
| FR-026-AC-7 | TC-492 |
| FR-026-AC-8 | TC-501 |
| FR-027-AC-1 | TC-493 |
| FR-027-AC-2 | TC-494 |
| FR-027-AC-3 | TC-495 |
| FR-027-AC-4 | TC-496 |
| FR-027-AC-5 | TC-497 |
| FR-027-AC-6 | TC-498 |
| FR-027-AC-7 | TC-499 |
| FR-027-AC-8 | TC-458 |
| FR-027-AC-9 | TC-500 |
| FR-028-AC-2 | TC-511 |
| FR-028-AC-3 | TC-512 |
| FR-028-AC-4 | TC-513 |
| FR-028-AC-5 | TC-514 |
| FR-028-AC-6 | TC-515 |
| FR-028-AC-7 | TC-516 |
| FR-028-AC-8 | TC-517 |
| FR-028-AC-9 | TC-558 |
| FR-028-AC-10 | TC-559 |
| FR-029-AC-1 | TC-548 |
| FR-029-AC-2 | TC-549 |
| FR-029-AC-3 | TC-550 |
| FR-029-AC-4 | TC-551 |
| FR-029-AC-5 | TC-552 |
| FR-029-AC-6 | TC-553 |
| FR-030-AC-1 | TC-529 (superseded by FR-032-AC-2) |
| FR-030-AC-2 | TC-530 (superseded by FR-032-AC-3) |
| FR-030-AC-3 | TC-536 (superseded by FR-033-AC-3) |
| FR-030-AC-4 | TC-528 (superseded by FR-032-AC-1) |
| FR-030-AC-5 | TC-533 (superseded by FR-032-AC-6) |
| FR-030-AC-6 | TC-529 (superseded; diagnostic reasons covered by FR-032-AC-2/3) |
| FR-031-AC-1 | TC-522 |
| FR-031-AC-2 | TC-523 |
| FR-031-AC-3 | TC-524 |
| FR-031-AC-4 | TC-525 |
| FR-031-AC-5 | TC-526 |
| FR-031-AC-6 | TC-527 |
| FR-032-AC-1 | TC-528 |
| FR-032-AC-2 | TC-529, TC-561 |
| FR-032-AC-3 | TC-530 |
| FR-032-AC-4 | TC-531 |
| FR-032-AC-5 | TC-532 |
| FR-032-AC-6 | TC-533 |
| FR-032-AC-7 | TC-573 |
| FR-032-AC-8 | TC-574 |
| FR-032-AC-9 | TC-575 |
| FR-032-AC-10 | TC-576 |
| FR-033-AC-1 | TC-534 |
| FR-033-AC-2 | TC-535 |
| FR-033-AC-3 | TC-536 |
| FR-033-AC-4 | TC-537, TC-562 |
| FR-033-AC-5 | TC-538 |
| FR-033-AC-6 | TC-539 |
| FR-033-AC-7 | TC-570 |
| FR-033-AC-8 | TC-571 |
| FR-033-AC-9 | TC-572, TC-561, TC-562 |
| FR-034-AC-1 | TC-540 |
| FR-034-AC-2 | TC-541 |
| FR-034-AC-3 | TC-542 |
| FR-034-AC-4 | TC-543 |
| FR-035-AC-1 | TC-544 |
| FR-035-AC-2 | TC-545 |
| FR-035-AC-3 | TC-546 |
| FR-035-AC-4 | TC-547 |
| FR-036-AC-1 | TC-584 |
| FR-036-AC-2 | TC-585 |
| FR-036-AC-3 | TC-586 |
| FR-036-AC-4 | TC-587 |
| FR-036-AC-5 | TC-588 |
| FR-037-AC-1 | TC-590 |
| FR-037-AC-2 | TC-591 |
| FR-037-AC-3 | TC-592 |
| FR-037-AC-4 | TC-593 |
| FR-037-AC-5 | TC-594, TC-595, TC-596 |
| FR-037-AC-6 | TC-528 |
| FR-038-AC-1 | TC-600 |
| FR-038-AC-2 | TC-601 |
| FR-038-AC-3 | TC-602 |
| FR-038-AC-4 | TC-603 |
| FR-038-AC-5 | TC-604 |
| FR-038-AC-6 | TC-605 |
| FR-038-AC-7 | TC-606 |
| FR-038-AC-8 | TC-607 |

### Non-Functional Requirements

| AC | Covering TC(s) |
|---|---|
| NFR-002-AC-1 | TC-052 |
| NFR-002-AC-2 | TC-052 (regression-gate assertion) |
| NFR-002-AC-3 | TC-053 |
| NFR-002-AC-4 | TC-577 |
| NFR-003-AC-1 | TC-050 |
| NFR-003-AC-2 | TC-050 |
| NFR-003-AC-3 | TC-050 |
| NFR-003-AC-4 | TC-506 |
| NFR-003-AC-5 | TC-582 |
| NFR-004-AC-1 | TC-051 |
| NFR-004-AC-2 | TC-051 |
| NFR-004-AC-3 | TC-051 |
| NFR-005-AC-1 | TC-054 |
| NFR-005-AC-2 | TC-054 |
| NFR-005-AC-3 | TC-055 |
| NFR-006-AC-2 | TC-057 |
| NFR-006-AC-3 | TC-058 |
| NFR-006-AC-4 | TC-578 |
| NFR-007-AC-1 | TC-083 |
| NFR-007-AC-2 | TC-120 |
| NFR-007-AC-3 | TC-121 |
| NFR-007-AC-4 | TC-122 |
| NFR-009-AC-1 | TC-330 |
| NFR-009-AC-2 | TC-331 |
| NFR-009-AC-3 | TC-332 |
| NFR-009-AC-4 | (process AC; covered by PR-review policy, not a TC) |
| NFR-010-AC-1 | (process AC; covered by CHANGELOG.md presence in TC-341) |
| NFR-010-AC-2 | TC-340 |
| NFR-010-AC-3 | TC-341 |
| NFR-010-AC-4 | TC-342 |
| NFR-011-AC-1 | TC-350 |
| NFR-011-AC-2 | TC-350 |
| NFR-011-AC-3 | TC-351 |
| NFR-011-AC-4 | TC-352 |
| NFR-012-AC-1..5 | RETIRED (ADR 0006 — miri job removed; superseded by NFR-003-AC-5 forbid + cargo-audit NFR-014) |
| NFR-013-AC-1 | TC-370 |
| NFR-013-AC-2 | TC-371 |
| NFR-013-AC-3 | TC-372 |
| NFR-013-AC-4 | TC-373 |
| NFR-014-AC-1 | TC-380 |
| NFR-014-AC-2 | TC-381 |
| NFR-014-AC-3 | TC-382 |
| NFR-015-AC-1 | TC-455 |
| NFR-015-AC-2 | TC-455 |
| NFR-015-AC-3 | TC-455 (regression-gate assertion) |
| NFR-015-AC-4 | TC-455 (correctness assertion in bench) |
| NFR-016-AC-1 | TC-469 |
| NFR-016-AC-2 | TC-464 |
| NFR-016-AC-3 | TC-465 |
| NFR-016-AC-4 | TC-467 |
| NFR-017-AC-1 | TC-503 |
| NFR-017-AC-2 | TC-503 |
| NFR-017-AC-3 | TC-503 |
| NFR-018-AC-1 | TC-504 |
| NFR-018-AC-2 | TC-505 |
| NFR-018-AC-3 | TC-504, TC-505 |
| NFR-018-AC-4 | (process AC; covered by P0-reproducer policy, parity with NFR-011-AC-4) |
| NFR-019-AC-1 | TC-579 |
| NFR-019-AC-2 | TC-580 |

**Coverage status: 309 / 309 ACs covered (100%).** The OKF slice (2026-06-16) adds FR-037-AC-1..6 (base concept frontmatter schema, TC-590..596 + TC-528) and FR-038-AC-1..8 (OKF bundle validation, TC-600..607) — 14 ACs. v0.4 adds FR-011-AC-21 (CR-006 `multiple: true`, TC-583) and FR-036-AC-1..5 (declarative lint rules, TC-584..588). v0.2 block model added 16 ACs (FR-019..022, TC-400..443). v0.3 adds 81 ACs — StR-005/006, US-011..013, FR-023..027 (incl. review-added FR-026-AC-8, FR-027-AC-9), NFR-015/016, plus the hardening re-review (NFR-003-AC-4, FR-024-AC-9, NFR-017, NFR-018) — covered by TC-455..507 (plus reused TC-456..459). The Miri ACs (NFR-012-AC-1..5) were **retired** (ADR 0006) and the compile-time **NFR-003-AC-5** (`forbid(unsafe_code)`, TC-582) added. PC (performance criteria) for US-011..013 are tracked as benches (TC-455..459, TC-469) and marked 🚧 pending implementation, consistent with the US-006..010 perf-bench convention. The v0.3 hardening re-review (loom NFR-017, TSAN/ASAN NFR-018) is recorded in spec.md §19.

**v0.4 markdown-validation slice** adds 42 ACs — US-014 (author validates markdown), FR-029 (archetype input contract, recast by ADR 0004), FR-030 (required-section validation, superseded by FR-032/FR-033), FR-031 (unified archetype shape), FR-032 (`validate_document`), FR-033 (locator `assert` facet), FR-034 (assert field interpolation), FR-035 (per-level heading uniqueness) — covered by TC-518..553. FR-030's ACs are mapped to the FR-032/FR-033 TCs that subsume them (per its CR note). This slice also back-fills 7 ACs that a prior commit left out of the audit table — FR-013-AC-11..14, FR-028-AC-9/10, US-003-AC-4 — via TC-554..560. New v0.4 TCs are 🚧 pending implementation.

**v0.4 render-removal slice (2026-06-04)** retires the render half (no
backward-compatibility layer — see spec.md §2bis.C). **41 ACs are dropped from the
required-coverage tally** by un-bolding their definitions (so the integrity grep no
longer counts them): FR-001 (5), FR-004 (4), FR-012 (5), NFR-001 (3), US-001 (4),
US-004 (3), US-005 (4), US-006 (4), US-007 (4), US-009 (3), plus FR-028-AC-1 and
NFR-006-AC-1. (US-005 was missed in the first render-removal pass and retired in a
follow-up: its render byte-parity suite — `tests/render_parity/` — is gone.)
The retired AC ids are immutable and preserved inline in their (now RETIRED) source
docs, marked `(RETIRED)`, but are no longer bold-anchored. **16 back-fill ACs are
added:** FR-011-AC-15..19, FR-033-AC-7..9, FR-032-AC-7..10, NFR-002-AC-4,
NFR-006-AC-4, NFR-019-AC-1..2 — covered by TC-565..580. TC-561 is re-pointed off
FR-033-AC-4 onto FR-033-AC-9 (the non-table `id_pattern` case); TC-562 covers both
FR-033-AC-4 and FR-033-AC-9.

**Integrity check (grep-verified):** all **309 distinct file-defined ACs** (definition-anchored: bold `**<ID>-AC-N**:` declarations) across `stakeholder/ usecase/ functional/ non-functional/` appear in the AC→TC audit table — **0 uncovered**. Note: `FR-900-AC-1/2` appearing inside FR-034-AC-1's example prose are NOT defined ACs and are excluded from the denominator (match `**…**:` definitions, not inline mentions). Retired ACs (marked `(RETIRED)`, un-bolded) are excluded by construction. Count: 316 (pre-removal) − 41 (retired) + 16 (back-fill) + 1 (FR-011-AC-20, CR-005 heading normalization) − 5 (NFR-012-AC-1..5 retired, ADR 0006) + 1 (NFR-003-AC-5, forbid(unsafe_code)) + 1 (FR-011-AC-21, CR-006 multiple:true) + 5 (FR-036-AC-1..5, declarative lint rules) + 1 (FR-010-AC-4, CR-007 escaped pipes) + 6 (FR-037-AC-1..6, OKF base concept schema) + 8 (FR-038-AC-1..8, OKF bundle validation) = **309**.

---

## Coverage Gaps

| Gap ID | Description | Risk Level | Mitigation |
|--------|-------------|------------|------------|
| GAP-001 | DSL evaluator parity test (TC-040) needs a curated fixture document per object_type across all 87+ types; some fixtures may not yet exist in the source repos. | Medium | Track per-type fixture availability in `tests/extract_parity/coverage.md`; missing fixtures are P1 follow-ups. |
| GAP-002 | Python Jinja2 reference renderer is not byte-stable across Jinja2 minor versions in all whitespace cases. | Low | StR-002-AC-2 documents known whitespace exceptions; pin reference's Jinja2 version. |
| GAP-003 | Cross-machine determinism (arm64 vs x86_64 byte parity) is implied but not explicitly benched. | Low | Add an arm64 + x86_64 CI matrix as a P2 enhancement. |
| GAP-005 | Sync from Filament to disk is out of scope (lives in `ix-cli`). Integration tests confirm quire-rs is correct against the on-disk state regardless of how it got there. | None | No mitigation needed — by design. |

---

## Test Execution Summary

v0.1 tests (TC-001..382) — DRAFT, traced to plan tasks 001..014; many already pass against the as-implemented v0.1 surface (parser, render, validate, loader, query API).

v0.2 block-model tests (TC-400..443) — ✅ IMPLEMENTED and passing under `make ci`:
- TC-400..403: `src/parser/walk.rs` (Pandoc `{#blk-id}` parsing).
- TC-410..411: `src/ast.rs::QuireSection.block_id` + `src/registry.rs::Registry::block_type`.
- TC-420..425: `src/block_edit.rs` (6 unit tests).
- TC-430..435: `src/writeback.rs` (10 unit tests).
- TC-440..443: `tests/block_round_trip.rs` (4 integration tests).

Total v0.2 block-model assertions exercised: 24 dedicated tests + parser walk tests sharing block_id paths.

v0.3 corpus + bindings tests (TC-455..507) — DRAFT, not yet implemented. Cover the Python binding surface (FR-023), the parallel `load_repo` walk (FR-024), the `Spec` corpus + intra-spec resolution + whole-spec queries (FR-025..027), the new NFRs (walk throughput NFR-015, binding overhead NFR-016), and the v0.3 hardening re-review (loom NFR-017, TSAN/ASAN NFR-018, unsafe FFI scoping NFR-003-AC-4 (miri NFR-012 retired — ADR 0006; replaced by compile-time NFR-003-AC-5), no-shared-mutable-state FR-024-AC-9). 49 new test cases (TC-455..507; incl. TC-500/501 from spec-review and TC-502..507 from the hardening re-review).

| Category | Total | Passed | Failed | Blocked | Coverage |
|----------|-------|--------|--------|---------|----------|
| Unit | 76 | 0 | 0 | 76 | 0% |
| Integration | 38 | 0 | 0 | 38 | 0% |
| Static (hardening) | 11 | 0 | 0 | 11 | 0% |
| Process | 1 | 0 | 0 | 1 | 0% |
| Integration | 21 | 0 | 0 | 21 | 0% |
| Parity | 7 | 0 | 0 | 7 | 0% |
| Bench | 14 | 0 | 0 | 14 | 0% |
| Property | 13 | 0 | 0 | 13 | 0% |
| Static / Snapshot | 26 | 0 | 0 | 26 | 0% |
| Compile | 5 | 0 | 0 | 5 | 0% |
| Soak | 1 | 0 | 0 | 1 | 0% |
| **Total** | **189** | **0** | **0** | **189** | **0%** |
