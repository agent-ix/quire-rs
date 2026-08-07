---
id: TM-001
title: "quire-rs Test Matrix"
type: TestMatrix
---

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
| US-015 Object edge vocabulary | AC-1..4 | TC-646, TC-647, TC-648, TC-649 (exercised by FR-040 TC-641/642/645/650) | ✅ Exercised via FR-040 engine TCs |

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
| FR-032 validate_document (markdown) | AC-1..13 | TC-528..533, TC-561 + TC-573 (placeholder set), TC-574 (none/n-a substantive), TC-575 (empty table/list reason), TC-576 (assert on resolved), TC-610 (composed object error), TC-611 (unknown object → warning), TC-612 (no object key), TC-613 (composed conformant) | ✅ |
| FR-033 Locator assert facet | AC-1..13 | TC-534..539, TC-561/562 + TC-570 (legality matrix), TC-571 (id-column precedence), TC-572 (id_pattern non-table), TC-608 (CR-008 `matches` content assert), TC-633 (CR-010 `choices` scalar enum), TC-634 (`column_choices`), TC-635 (`column_patterns`) | ✅ |
| FR-034 Assert field interpolation | AC-1..4 | TC-540 (id prefix), TC-541 (missing field diag), TC-542 (regex-escape), TC-543 (no-token static regex) | 🚧 Pending implementation |
| FR-035 Per-level heading uniqueness | AC-1..4 | TC-544 (dup L2), TC-545 (cross-level ok), TC-546 (iterate_over children), TC-547 (line number) | 🚧 Pending implementation |
| FR-036 Declarative lint rules | AC-1..6 | TC-584 (manifest→Registry::lint_rules + malformed rule fails load), TC-585 (vocab finding + annotation pass), TC-586 (archetype scoping), TC-587 (missing section/column → none), TC-588 (lint never affects extract/validate), TC-609 (CR-009 `section_body_pattern`) | ✅ |
| FR-037 Base concept frontmatter schema (OKF) | AC-1..6 | TC-590 (minimal typed), TC-591 (optional desc/tags), TC-592 (missing type), TC-593 (empty type), TC-594/595/596 (mistyped desc/tags/non-string item), TC-528 (shape wired into validate_document) | ✅ |
| FR-038 OKF bundle validation (Strict vs Okf + index) | AC-1..8 | TC-600 (strict untyped→error), TC-601 (okf untyped→error), TC-602 (okf tolerates unknown type+broken link, strict rejects), TC-603 (strict conformant+complete index valid), TC-604 (index incompleteness error/warning), TC-605 (root missing okf_version), TC-606 (subdir no okf_version), TC-607 (strict mistyped description) | ✅ |
| FR-024 Parallel repo walk (load_repo) | AC-1..9 | TC-470 (N files→N docs), TC-471 (malformed→diagnostic), TC-472 (gitignore), TC-473 (path-sorted determinism), TC-474 (symlink loop), TC-475 (id derivation), TC-476 (bad root), TC-455 (bench), TC-502 (no shared mutable state) | ✅ Complete |
| FR-025 Spec corpus model | AC-1..6 | TC-480 (len), TC-481 (id index), TC-482 (dup id), TC-483 (Send+Sync), TC-484 (scope-guard surface), TC-485 (no-IO queries) | ✅ Complete |
| FR-026 Intra-spec reference resolution | AC-1..11 | TC-486 (frontmatter edge), TC-487 (ix:// edge), TC-488 (dangling), TC-489 (cross-spec dangling), TC-490 (bidirectional), TC-491 (target-id extraction), TC-492 (O(edges) proptest), TC-501 (dedup), TC-620 (rel-path edge/dangling), TC-621 (index/log excluded), TC-622 (dedup parity across sources) | ✅ Complete |
| FR-027 Whole-spec query API | AC-1..8 | TC-493 (by_type), TC-494 (referencing), TC-495 (orphans), TC-496 (coverage), TC-497 (dangling agreement), TC-498 (sorted determinism), TC-499 (no-IO), TC-458 (bench) | ✅ Complete |
| FR-039 Unlinked reference detection (ADR 0007) | AC-1..10 | TC-623 (auto-fix bare id), TC-624 (sub-id→parent file), TC-625 (inline-code conversion), TC-626 (fenced/frontmatter ignored), TC-627 (already-linked + idempotence), TC-628 (self-reference), TC-629 (unresolved→warn-only), TC-630 (ambiguous→warn-only), TC-631 (sorted determinism), TC-632 (multi-token code span skipped) | 🚧 Pending implementation |
| FR-040 Object-axis typed edge vocabulary + cross-domain targets | AC-1..11 | TC-636 (registries load + idempotent merge), TC-637 (conflict→first-wins diagnostic), TC-650 (unknown verb/role diagnostic + strict escalation), TC-638 (array→{v:["*"]} + map round-trip), TC-651 (roles parsed onto archetype), TC-639 (resolve union), TC-640 (target_satisfies name/role/"*"), TC-641 (DisallowedEdgeType warn + unknown-object fallback), TC-642 (DisallowedEdgeTarget warn + role match cross-module + skips), TC-643 (warn-only, non-blocking), TC-644 (sorted determinism), TC-645 (skeleton Relationships block) | ✅ Implemented (engine) |
| FR-041 Authorable inverse edge verbs | AC-1..5 | TC-652 (inverse index), TC-653 (Tier-1 recognition), TC-654 (precedence + DuplicateInverseEdge), TC-655 (Tier-2 forward normalization), TC-656 (warn-only determinism) | 🚧 Pending implementation |
| FR-042 Requirement-grammar check (EARS) | AC-1..10 | TC-657 (6 patterns + unclassifiable), TC-658 (non-singular + enumerated-stem exemption), TC-659 (missing-subject + StR relaxed subject), TC-660 (vague-response + passive exemption), TC-661 (non-canonical-trigger + NFR no-trigger), TC-662 (archetype/section binding), TC-663 (warning vs error severity routing), TC-664 (finding fields), TC-665 (fence/quote/reference skip), TC-666 (PyO3 parity) | 🚧 Pending implementation |
| FR-043 Module-supplied concrete lexicon | AC-1..7 | TC-667 (lexicon registry load + accessor), TC-668 (first-wins merge + DuplicateLexiconTerm), TC-669 (lexicon term suppresses vague-response; removed re-flags), TC-670 (no hardcoded noun list under empty lexicon), TC-671 (backtick/mechanism/bound hold under empty lexicon), TC-672 (registry vs type-only lexicon paths), TC-673 (PyO3 check_grammar module_root) | 🚧 Pending implementation |
| FR-044 Project Ubiquitous-Language lexicon | AC-1..7 | TC-674 (harvest Term column from Glossary `## Terms` table), TC-675 (harvest bold term from `## Ubiquitous Language` bullets), TC-676 (combined lexicon = registry keys ∪ project terms), TC-677 (validate_document_in_registry_with_lexicon injects lexicon), TC-678 (validate_bundle harvests Spec + applies combined lexicon), TC-679 (advisory: project suppression never changes is_valid), TC-680 (no glossary → empty terms → module-only path) | 🚧 Pending implementation |

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
| US-016 Consume canonical Filament extraction | Illustrative examples | TC-681..TC-690 | ✅ Complete |
| US-017 Agent verifies coverage deterministically | Illustrative examples | TC-724..TC-750, TC-753, TC-756 (via FR-049/050/051) | 🚧 Pending implementation |
| FR-045 Canonical Filament core extraction engine | AC-1..6; CON-1..4 | TC-691..TC-704, TC-690, TC-705 | ✅ Complete |
| FR-046 Filament extraction bindings | AC-1..4; CON-1..3 | TC-686, TC-687, TC-688, TC-689, TC-767 | ✅ Complete |
| FR-047 Acceptance-criteria grammar | AC-1..14; CON-1..2 | TC-707 (shape classification, assertion canonical), TC-708 (every non-empty cell segmented), TC-709 (non-singular + pair idiom), TC-710 (vague-response via lexicon), TC-711 (vacuous-outcome), TC-712 (binding), TC-713 (finding fields + routing), TC-714 (generic --summary prefix), TC-715 (PyO3 parity), TC-751 (non-canonical-shape steers obligation/GWT → assertion), TC-754 (fenced/blockquote skip in supplements), TC-757 (module-data observable/vacuity vocabularies), TC-761 (quoted keywords are mentions, not uses — CR-017), TC-763 (elided-copula predication is a predicate — CR-019) | ✅ Implemented (CLI-surface AC-8 awaits EXT-3 `quire-cli`) |
| FR-048 Per-check grammar severity | AC-1..10 | TC-716 (manifest registry + accessor), TC-717 (first-wins + DuplicateGrammarSeverity), TC-718 (per-check error routing), TC-719 (absent key → warning), TC-720 (--severity override + repeatable), TC-721 (--strict unchanged), TC-722 (type-only all-default), TC-723 (malformed entry fails load), TC-752 (`off` suppresses a check entirely), TC-755 (malformed --severity CLI entry rejected) | ✅ Implemented (CLI-surface AC-5/6/10 await EXT-3 `quire-cli`) |
| FR-049 Verification-reference integrity | AC-1..8 | TC-724 (resolved reference clean), TC-725 (dangling finding), TC-726 (posture degradation), TC-727 (model-driven pattern/column), TC-728 (auxiliary trace-source harvest), TC-729 (no model → no findings), TC-730 (multi-annotation cells), TC-731 (deterministic findings) | ✅ Implemented |
| FR-050 Declarative coverage computation | AC-1..12; CON-1..2 | TC-732 (traceability model load), TC-733 (malformed/absent model), TC-734 (unbacked rows), TC-735 (status lies), TC-736 (untracked symbols), TC-737 (per-group counts), TC-738 (byte-identical output), TC-739 (non-ISO model), TC-740 (no model → diagnostic exit), TC-758 (status marker + note, retired class), TC-759 (declared column vocabularies), TC-760 (range expansion + annotation stripping), TC-756 (CON-2 static boundary audit) | ✅ Implemented (AC-9 CLI exit awaits EXT-3 `quire-cli`) |
| FR-051 Source symbol extraction with relations | AC-1..11; CON-1..3 | TC-741 (adapter symbol extraction), TC-742 (identity stability), TC-743 (test-symbol classification), TC-744 (canonical markers bind statically), TC-745 (marker/tag forms are module data), TC-746 (duplicate-id dedup), TC-747 (FR-045 record shapes), TC-748 (defined_in/contains edges), TC-749 (unparseable-file degradation), TC-750 (byte-identical repeat), TC-753 (legacy textual forms + rewrite suggestions), TC-756 (CON-1 static boundary audit) | ✅ Implemented |
| NFR-020 Filament extraction boundary pure/deterministic | static inspection + parity tests | TC-704, TC-767, TC-690 | ✅ Complete |

---

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---------|-------|------|----------|-----------|--------|
| TC-001 | parse_document handles empty + preamble-only + nested headings | Unit | P0 | FR-005-AC-1, FR-005-AC-2, FR-005-AC-3, US-002 | 🚧 |
| TC-002 | parse_document does not panic on 10k random inputs | Property | P0 | FR-005-AC-4 | 🚧 |
| TC-002b | apply_patch proptest fuzz never panics | Property | P0 | FR-002-AC-4 | 🚧 |
| TC-003 | render against compiled FR archetype byte-equals Python reference ((RETIRED); (RETIRED)) | Integration | P0 | FR-001-AC-1, US-001-AC-2 | ⛔ RETIRED — render removed |
| TC-004 | render_by_name("unknown") returns UnknownArchetype ((RETIRED)) | Unit | P0 | FR-001-AC-2 | ⛔ RETIRED — render removed |
| TC-005 | Adding new archetype to corpus requires no Rust change | Integration | P0 | FR-001-AC-5, StR-001-AC-4 | 🚧 |
| TC-006 | render returns field-keyed SchemaViolation on missing required ((RETIRED)) | Unit | P0 | FR-001-AC-3, NFR-005-AC-1 | ⛔ RETIRED — render removed |
| TC-007 | apply_patch merges then validates merged result | Unit | P0 | FR-002-AC-1, FR-002-AC-2, US-004-AC-1, US-004-AC-2 | 🚧 |
| TC-007b | apply_patch rejects unknown key under additionalProperties:false | Unit | P0 | FR-002-AC-3 | 🚧 |
| TC-008 | render is thread-safe under 64-thread concurrency ((RETIRED); (RETIRED)) | Integration | P1 | FR-001-AC-4, FR-004-AC-2 | ⛔ RETIRED — render removed |
| TC-009 | schema_for returns the on-disk schema byte-identical | Snapshot | P0 | FR-003-AC-1, US-001-AC-4 | 🚧 |
| TC-009b | schema_for unknown archetype returns UnknownArchetype | Unit | P1 | FR-003-AC-2 | 🚧 |
| TC-010 | Strict mode reports missing template field as TemplateError ((RETIRED)) | Unit | P0 | FR-004-AC-1 | ⛔ RETIRED — render removed |
| TC-011 | Renderer environment cost measured (one-time) ((RETIRED)) | Benchmark | P2 | FR-004-AC-3 | ⛔ RETIRED — render removed |
| TC-012 | extract_frontmatter happy path | Unit | P0 | FR-006-AC-2 | 🚧 |
| TC-013 | extract_frontmatter malformed YAML returns body fallback | Unit | P0 | FR-006-AC-3 | 🚧 |
| TC-014 | extract_frontmatter unterminated fence returns body fallback | Unit | P1 | FR-006-AC-4 | 🚧 |
| TC-015 | Backtick fence blocks heading parsing inside | Unit | P0 | FR-007-AC-1 | 🚧 |
| TC-016 | Unclosed fence: trailing lines are not parsed as headings | Unit | P1 | FR-007-AC-2 | 🚧 |
| TC-017 | Tilde fence behaves identically to backtick fence | Unit | P1 | FR-007-AC-3 | 🚧 |
| TC-018 | extract evaluates api_endpoint DSL on real fixture | Integration | P0 | FR-011-AC-1, US-003-AC-1 | 🚧 |
| TC-019 | extract code_block (language: json) byte-equals fenced content ((code_block locator)) | Integration | P0 | FR-011, US-003-AC-2 | 🚧 |
| TC-020 | TS reference test suite transliterated; all pass (parity) | Integration | P0 | StR-003-AC-2 | 🚧 |
| TC-021 | quire-rs structural equivalence against canonical TS fixtures on real corpus (parity) | Integration | P1 | StR-003-AC-3 | 🚧 |
| TC-022 | Section content preserves leading/trailing whitespace | Unit | P0 | FR-008-AC-1 | 🚧 |
| TC-023 | CRLF and LF endings preserved in section content | Unit | P1 | FR-008-AC-2 | 🚧 |
| TC-024 | Roundtrip: reconstructing body from sections equals input | Property | P0 | FR-008-AC-3, NFR-006 | 🚧 |
| TC-025 | Slug normalization (lowercase, alphanum-dash, trim) | Unit | P0 | FR-009-AC-1, FR-009-AC-2, FR-009-AC-3 | 🚧 |
| TC-026 | Line index ignores frontmatter offset | Unit | P0 | FR-009-AC-4, FR-009-AC-5 | 🚧 |
| TC-027 | Query API module-level signatures compile and re-export | Compile | P0 | FR-010-AC-1 | 🚧 |
| TC-028 | Query API parity sweep against TS fixtures (parity) | Integration | P0 | FR-010-AC-2 | 🚧 |
| TC-029 | Query API complexity: no quadratic walks | Property | P1 | FR-010-AC-3 | 🚧 |
| TC-589 | `\|` in table cells is literal (escape consumed) in header/body/cell-final positions; other backslashes verbatim; borderless rows split identically; GFM alignment separators recognized; `-`/`*`/`+` bullets parse | Unit | P0 | FR-010-AC-4 | ✅ |
| TC-030 | Corpus parity sweep: every archetype × every fixture byte-equals Python reference (parity) ((RETIRED); (RETIRED)) | Integration | P0 | FR-012-AC-1, FR-012-AC-2, StR-002, US-005-AC-1, US-005-AC-2, US-005-AC-3 | 🚧 |
| TC-031 | tests/render_parity/corpus.yaml exists and lists v1 modules ((RETIRED)) | Static | P0 | FR-012-AC-1 | ⛔ RETIRED — render removed |
| TC-039 | Adding archetype to corpus.yaml + fixtures extends suite with no Rust change | Integration | P0 | FR-012-AC-5 | 🚧 |
| TC-040 | extract sweep across all 87+ object archetypes from 6 source repos | Integration | P0 | FR-011-AC-5, US-003 | 🚧 |
| TC-041 | Parity suite catches deliberate template mutation (regression) ((RETIRED); (RETIRED)) | Integration | P0 | FR-012-AC-3, US-005-AC-4 | ⛔ RETIRED — render removed |
| TC-042 | Bench: render per-archetype median <1 ms (sweep across corpus) ((RETIRED)) | Benchmark | P0 | NFR-001-AC-1, NFR-001-AC-2 | ⛔ RETIRED — render removed |
| TC-042b | Bench: apply_patch median <100 µs (typical artifact) | Benchmark | P1 | FR-002-AC-5 | 🚧 |
| TC-050 | check_unsafe_comments.sh exits 0; baseline empty | Static | P0 | NFR-003 | 🚧 |
| TC-051 | cargo deny check licenses exits 0 | Static | P0 | NFR-004 | 🚧 |
| TC-052 | Bench: parse_document 5 MB median <500 ms | Benchmark | P0 | NFR-002-AC-1 | 🚧 |
| TC-053 | Bench: 5 MB document round-trips byte-for-byte | Property | P0 | NFR-002-AC-3 | 🚧 |
| TC-054 | QuireError::Display contains all four required tuple elements | Unit | P0 | NFR-005-AC-1, US-001-AC-3 | 🚧 |
| TC-055 | QuireError snapshot pins canonical error per archetype | Snapshot | P1 | NFR-005-AC-3 | 🚧 |
| TC-056 | Determinism: render 100x across threads → byte-identical | Property | P0 | NFR-006-AC-1 | 🚧 |
| TC-057 | Determinism: parse 100x → Eq | Property | P0 | NFR-006-AC-2 | 🚧 |
| TC-058 | Static audit: no HashMap in render/parse code paths | Static | P1 | NFR-006-AC-3 | 🚧 |
| TC-060 | Registry behavior identical across three on-disk corpora (Filament/hand/test) | Integration | P0 | StR-001-AC-5 | 🚧 |
| TC-061 | LLM tool-call schema round-trip: schema_for → tool input → render | Integration | P1 | US-001-AC-2, US-001-AC-3 | 🚧 |
| TC-062 | Cargo.lock has no schemars dependency | Static | P1 | FR-003-AC-4 | 🚧 |
| TC-070 | DSL multi-yield (iterate_over) emits one record per iteration unit | Unit | P0 | FR-011-AC-2 | 🚧 |
| TC-072 | Each of 6 Locator primitives exercised by ≥1 unit test | Unit | P0 | FR-011-AC-1 | 🚧 |
| TC-073 | DSL required:true missing field returns MissingField | Unit | P0 | FR-011-AC-4 | 🚧 |
| TC-563 | `code_block` is section-owned: single-yield `under:X` excludes other sections; multi-yield `per_match` isolates each unit's block, required-miss → MissingField for the unit lacking one | Unit | P0 | FR-011-AC-13 | ✅ |
| TC-564 | Scanner recognizes ``` and `~~~` fences with matching-character close: `~~~mermaid` extracted as `mermaid`; cross-char fence line is content; unclosed `~~~` flushed as final block; section-owned `code_block` resolves a `~~~` block | Unit | P0 | FR-011-AC-14 | ✅ |
| TC-080 | Registry::from_env() with neither search-path env var (IX_FILAMENT_MODULES_PATH / IX_SCHEMA_PATH) set and no default dir → empty registry, no error | Unit | P0 | FR-013-AC-1 | 🚧 |
| TC-081 | IX_SCHEMA_PATH pointing at spec-artifacts-iso loads all 8 ISO archetypes | Integration | P0 | FR-013-AC-2 | 🚧 |
| TC-082 | Manifest with missing schema_ref produces ArchetypeLoadError; siblings still load | Integration | P0 | FR-013-AC-3 | 🚧 |
| TC-083 | Bench: Registry::load_from baseline corpus < 100 ms median | Benchmark | P0 | FR-013-AC-4, NFR-007-AC-1 | 🚧 |
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
| TC-113 | domain object_type from spec-objects-business with legacy heading: parity vs python (parity) | Integration | P0 | FR-016-AC-4 | 🚧 |
| TC-120 | Bench: 10 000 sequential renders after load → median <1ms, zero I/O | Benchmark | P0 | NFR-007-AC-2 | 🚧 |
| TC-121 | Tracing audit: zero Template::parse and zero JSONSchema::compile during render | Static | P0 | NFR-007-AC-3 | 🚧 |
| TC-122 | Long-running soak: registry memory footprint flat over 1 M renders (soak) | Integration | P1 | NFR-007-AC-4 | 🚧 |
| TC-130 | Loader symlink-loop detected; warning emitted; cycle skipped | Integration | P0 | FR-013-AC-7 | 🚧 |
| TC-131 | Duplicate IX_SCHEMA_PATH entries: modules loaded once | Integration | P0 | FR-013-AC-8 | 🚧 |
| TC-132 | Registry: Send + Sync (compile-time assertion) | Compile | P0 | FR-013-AC-9 | 🚧 |
| TC-133 | Path-entry-is-a-file: warning emitted; other entries process | Integration | P1 | FR-013-AC-10 | 🚧 |
| TC-134 | Two modules same name → DuplicateModuleName diag + first-wins | Integration | P0 | FR-014-AC-6 | 🚧 |
| TC-135 | Manifest without name uses parent dir name + diagnostic | Unit | P1 | FR-014-AC-7 | 🚧 |
| TC-150 | DSL with both match and iterate_over → ArchetypeLoadError at load | Unit | P0 | FR-011-AC-6 | 🚧 |
| TC-151 | DSL with unknown key → ArchetypeLoadError at load | Unit | P0 | FR-011-AC-7 | 🚧 |
| TC-152 | iterate_over.section_path missing → empty records + IterateRootMissing | Unit | P0 | FR-011-AC-8 | 🚧 |
| TC-160 | Template with {% include %} → ArchetypeLoadError ((RETIRED)) | Unit | P0 | FR-004-AC-4 | ⛔ RETIRED — render removed |
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
| TC-204 | CI workflow includes render_parity job (not just test job) ((RETIRED); (RETIRED)) | Static | P0 | US-005-AC-2, US-005-AC-3, StR-002-AC-3 | 🚧 |
| TC-205 | A patch making merged value invalid (title="") returns SchemaViolation, not a render error | Unit | P0 | US-004-AC-2 | 🚧 |
| TC-206 | Bench: bench_patch_render_fr median < 1ms for typical FR | Benchmark | P1 | US-004-AC-3 | 🚧 |
| TC-330 | Cargo.toml uses tilde/equals pins for load-bearing deps | Static | P0 | NFR-009-AC-1 | 🚧 |
| TC-331 | spec/assets/adr/0001-validator-crate.md exists with chosen crate + bench numbers | Static | P0 | NFR-009-AC-2 | 🚧 |
| TC-332 | Static: no load-bearing dep has unbounded version | Static | P0 | NFR-009-AC-3 | 🚧 |
| TC-340 | Public enums are #[non_exhaustive] | Compile | P0 | NFR-010-AC-2 | 🚧 |
| TC-341 | CHANGELOG.md exists with release entries | Static | P1 | NFR-010-AC-3 | 🚧 |
| TC-342 | cargo-semver-checks against previous tag reports no unexpected breaks | Static | P1 | NFR-010-AC-4 | 🚧 |
| TC-350 | All 6 fuzz targets compile and run cleanly for 60s on baseline | Integration | P0 | NFR-011-AC-1, NFR-011-AC-2 | 🚧 |
| TC-351 | .github/workflows/fuzz.yml runs all targets weekly | Static | P0 | NFR-011-AC-3 | 🚧 |
| TC-352 | Discovered crash reproducer committed under fuzz/corpus + regression test | Integration | P1 | NFR-011-AC-4 | 🚧 |
| TC-360 | (RETIRED — ADR 0006) miri CI job removed; first-party safety is compile-time `forbid(unsafe_code)` (NFR-003-AC-5) ((RETIRED)) | Static | P0 | NFR-012-AC-1 | ⛔ |
| TC-361 | (RETIRED — ADR 0006) miri job removed ((RETIRED)) | Integration | P0 | NFR-012-AC-3 | ⛔ |
| TC-362 | (RETIRED — ADR 0006) miri job removed (process) ((RETIRED)) | Manual | P0 | NFR-012-AC-4 | ⛔ |
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
| TC-403 | Heading without `{#…}` → block_id = None; heading text byte-identical to input ((negative)) | Unit | P0 | FR-019-AC-1 | ✅ |
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
| TC-434 | update_section unknown heading → MissingField ((negative)) | Unit | P0 | FR-022-AC-5 | ✅ |
| TC-435 | update_block unknown block_id → MissingField ((negative)) | Unit | P0 | FR-022-AC-5 | ✅ |
| TC-440 | End-to-end: parse FR-like artifact, apply_block_patch, assert only patched block's bytes changed (composite) | Integration | P0 | FR-019, FR-020, FR-021, FR-022 | ✅ |
| TC-441 | End-to-end: replace_block renders fresh data into existing block bytes | Integration | P0 | FR-021-AC-2, FR-022-AC-2 | ✅ |
| TC-442 | End-to-end: empty patch is idempotent (rendered bytes equal current data) | Integration | P1 | FR-021-AC-1 | ✅ |
| TC-443 | End-to-end: block_id survives parse → patch → reparse | Integration | P0 | FR-019-AC-2 | ✅ |
| TC-450 | Bench: `apply_block_patch` p50 < 1 ms on 10 KB / 5-block doc; p99 < 5 ms; memory-flat across iterations | Benchmark | P0 | US-006-PC-1, US-006-PC-2, US-006-PC-3, US-006-PC-4 | 🚧 |
| TC-451 | Bench: `replace_block` p50 < 1 ms on 10 KB / 5-block doc; ±10% of TC-450; report crossover where replace beats patch on large blocks | Benchmark | P0 | US-007-PC-1, US-007-PC-4 | 🚧 |
| TC-452 | Bench: 10 sequential block patches on 20 KB doc; p50 < 10 ms; assert linear-in-N (no superlinear regression); document block_id-lookup cost on > 100-block doc | Benchmark | P0 | US-008-PC-1, US-008-PC-5 | 🚧 |
| TC-453 | Bench: `parse_document` + `extract` (multi-yield, ~10 records) on 10 KB doc; p50 < 2 ms | Benchmark | P0 | US-010-PC-1 | 🚧 |
| TC-454 | Bench: corpus-scale extract (100 docs, 10 records each) single-threaded p50 < 200 ms; 8-thread p50 < 50 ms | Benchmark | P1 | US-010-PC-3 | 🚧 |
| TC-455 | Bench: `load_repo` 1k-doc corpus at 1 + 8 threads; p50 < 600 ms / < 200 ms; parallel efficiency ≥ 0.6; output path-sorted | Benchmark | P0 | FR-024-AC-8, NFR-015-AC-1, NFR-015-AC-2, NFR-015-AC-3, NFR-015-AC-4, US-011-PC-1 | 🚧 |
| TC-456 | Bench: 500+ doc corpus through Python binding ≥ 5× faster than pure-Python filament_parser path | Benchmark | P0 | StR-005-AC-3, US-011-PC-3 | 🚧 |
| TC-457 | Bench: `Spec` construct (load + resolve) for 200-artifact spec p50 < 50 ms single-thread | Benchmark | P0 | US-012-PC-1 | 🚧 |
| TC-458 | Bench: `by_id` / `referencing` / `orphans` sub-millisecond per query over 200-artifact corpus | Benchmark | P1 | FR-027-AC-8, US-012-PC-2 | 🚧 |
| TC-459 | Bench: resolve all references in 200-artifact spec p50 < 5 ms (part of construct budget) | Benchmark | P1 | US-013-PC-1 | 🚧 |
| TC-460 | `cargo build` (no features) and `--features python` both succeed; no pyo3 linkage in default build | Static | P0 | FR-023-AC-1, StR-005-AC-2 | 🚧 |
| TC-461 | `quire.parse_document(text)` returns frontmatter/headings/block-ids matching Rust `parse_document` | Integration | P0 | FR-023-AC-2, StR-005-AC-1 | 🚧 |
| TC-462 | `quire.validate(bad, "fr")` violation field-path equals Rust `validate` for same input | Integration | P0 | FR-023-AC-3 | 🚧 |
| TC-463 | `quire.load_repo(path)` returns one doc per `.md` + per-file diagnostics via binding | Integration | P0 | FR-023-AC-4, US-011-AC-1, US-011-AC-2 | 🚧 |
| TC-464 | Two Python threads each calling `load_repo` complete < 2× single-call (GIL released) | Integration | P0 | FR-023-AC-5, NFR-016-AC-2, US-011-AC-5 | 🚧 |
| TC-465 | One abi3 wheel imports + smoke-tests under two CPython 3.x minor versions | Integration | P0 | FR-023-AC-6, StR-005-AC-5, NFR-016-AC-3 | 🚧 |
| TC-466 | No `subprocess`/`Popen`/socket on the binding data path (static grep + runtime assert) | Static | P0 | FR-023-AC-7, StR-005-AC-4 | 🚧 |
| TC-467 | Binding returns structured objects; no Python-side markdown/frontmatter re-parse | Integration | P0 | US-011-AC-3, NFR-016-AC-4 | 🚧 |
| TC-469 | Bench: per-FFI-crossing overhead for `parse_document` < 50 µs over equivalent Rust call | Benchmark | P1 | NFR-016-AC-1, US-011-PC-2 | 🚧 |
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
| TC-620 | Relative-path body link to a loaded doc → Resolved `references` edge (text/slug-independent); rel link to no loaded path → Dangling | Unit | P0 | FR-026-AC-9 | 🚧 |
| TC-621 | Relative-path links in `index.md`/`log.md` contribute no `references` edges; the same link in an ordinary artifact is harvested | Unit | P0 | FR-026-AC-10 | 🚧 |
| TC-622 | Identical `(source, FR-002, references)` from a relative-path link and an `ix://` link / frontmatter entry dedups to one edge | Unit | P0 | FR-026-AC-11 | 🚧 |
| TC-623 | `unlinked_references`: bare `FR-008` in prose (loaded) → one `AutoFix` with token span + `suggested_link [FR-008](<rel-path>)` | Unit | P0 | FR-039-AC-1 | 🚧 |
| TC-624 | Sub-id `FR-008-CON-4` → `AutoFix` label = full token, destination = parent `FR-008` file | Unit | P0 | FR-039-AC-2 | 🚧 |
| TC-625 | Inline-code `` `FR-008` `` → `AutoFix` span covers whole code span (backticks), `suggested_link` is a plain backtick-free link | Unit | P0 | FR-039-AC-3 | 🚧 |
| TC-626 | Token inside fenced ``` block and token only in frontmatter → no finding | Unit | P0 | FR-039-AC-4 | 🚧 |
| TC-627 | Token already inside a Markdown link / `ix://` destination → no finding; re-run after applying all `AutoFix` yields none (idempotence) | Unit | P0 | FR-039-AC-5 | 🚧 |
| TC-628 | Self-reference: own H1, `id:`, own `FR-024-AC-*` rows in FR-024 → no finding; ref to other `FR-008` → finding | Unit | P0 | FR-039-AC-6 | 🚧 |
| TC-629 | Token whose parent id is absent from the loaded set → `WarnOnly { Unresolved }`, no `suggested_link` | Unit | P0 | FR-039-AC-7 | 🚧 |
| TC-630 | Token whose parent id maps to >1 loaded doc → `WarnOnly { Ambiguous }`, no `suggested_link` | Unit | P0 | FR-039-AC-8 | 🚧 |
| TC-631 | `unlinked_references` results sorted by `(path, byte_span.start)`; identical across runs and thread counts | Property | P0 | FR-039-AC-9, NFR-006 | 🚧 |
| TC-632 | A code span with >1 artifact token (`` `FR-008/FR-009` ``) yields no finding; a single-token code span still converts | Unit | P0 | FR-039-AC-10 | 🚧 |
| TC-765 | Sub-id kinds `-AC-`/`-CON-`/`-VC-` each strip to the parent id, a plain id is unchanged, a lookalike is not stripped, and a bare `StR-001-VC-2` in prose is matched whole and autofixed to its StR | Unit | P1 | FR-039-AC-11 | ✅ |
| TC-636 | A manifest declaring `edge_types` (verb → category/description/optional inverse) and `roles` (name → description) loads; merged `Registry` exposes both; identical re-declaration across two modules is silently idempotent (no diagnostic) | Unit | P0 | FR-040-AC-1 | 🚧 |
| TC-637 | Differing re-declaration of an `edge_types`/`roles` name across modules is first-wins and emits a non-fatal `DuplicateEdgeType`/`DuplicateRole` diagnostic; default load still succeeds | Unit | P0 | FR-040-AC-2, FR-014 | 🚧 |
| TC-650 | An `allowed_links` key absent from `edge_types`, or a `roles:`/target token absent from `roles`, emits a non-fatal `UnknownEdgeType`/`UnknownRole` diagnostic (default load succeeds); `load_strict` escalates AC-2/AC-3 diagnostics to errors | Unit | P0 | FR-040-AC-3, FR-014 | 🚧 |
| TC-638 | `allowed_links` array form `[calls, publishes]` normalizes to `{calls:["*"], publishes:["*"]}`; map form `{contains:[value_object]}` round-trips as an `AllowedLinks` map (CR-001, supersedes FR-031 array-only parse) | Unit | P0 | FR-040-AC-4, FR-031 | 🚧 |
| TC-651 | An object type's `roles: [..]` list is parsed onto its `CompiledArchetype` and readable via `roles()`; an archetype with no roles reads empty | Unit | P0 | FR-040-AC-5 | 🚧 |
| TC-639 | `resolve_allowed_links(T, Some(O))` returns the union of both axes; a verb on both axes unions target lists and `"*"` absorbs concrete/role tokens; `object=None` returns artifact vocab alone | Unit | P0 | FR-040-AC-6 | 🚧 |
| TC-640 | `target_satisfies` true when token == target name, token is a role the target carries, or token == `"*"`; false otherwise | Unit | P0 | FR-040-AC-7 | 🚧 |
| TC-641 | A frontmatter-harvested edge `type` not in `resolve_allowed_links` yields exactly one warning `DisallowedEdgeType` naming source+verb; in-vocabulary-only yields none; unknown `object:` falls back to artifact-axis vocab and Tier-1 still runs | Integration | P0 | FR-040-AC-8, FR-032 | 🚧 |
| TC-642 | A corpus edge whose target document's `object:` archetype/roles fail the verb's target list yields a warning `DisallowedEdgeTarget`; same verb to a target carrying the required role passes (cross-module); skipped for `"*"`, no-`object:` targets, and dangling/cross-repo targets | Integration | P0 | FR-040-AC-9 | 🚧 |
| TC-643 | Tier-1/Tier-2 findings are warnings only — they do not block extraction or FR-032 structural validation, and a corpus with disallowed edges still loads | Integration | P0 | FR-040-AC-10, FR-032 | 🚧 |
| TC-644 | Tier-1/Tier-2 diagnostics sorted by `(source, target, edge_type)`; identical across repeated runs and thread counts | Property | P0 | FR-040-AC-10, NFR-006 | 🚧 |
| TC-645 | `input_skeleton` with an optional `object` arg renders a Relationships block listing each resolved verb with category/description/targets; without `object`, only the artifact vocabulary is listed | Unit | P0 | FR-040-AC-11, FR-029 | 🚧 |
| TC-652 | The merged `Registry` exposes an inverse index mapping each declared `inverse:` label to its forward verb; a registry with no declared inverses exposes an empty index | Unit | P0 | FR-041-AC-1 | 🚧 |
| TC-653 | A frontmatter-harvested edge whose `type` is a declared inverse label is type-allowed during FR-032 (no `DisallowedEdgeType`) even when absent from `resolve_allowed_links`; a `type` that is neither a resolved key nor a known inverse still yields exactly one `DisallowedEdgeType` | Integration | P0 | FR-041-AC-2, FR-032 | 🚧 |
| TC-654 | A label that is both a forward `edge_types` key and an inverse of another verb resolves to the forward registration; two forward verbs declaring the same inverse label are first-wins and emit a non-fatal `DuplicateInverseEdge` (default load succeeds) | Unit | P0 | FR-041-AC-3 | 🚧 |
| TC-655 | A corpus inverse-verb edge `(source, I, target)` is normalized to `(target, F, source)` before `target_satisfies`; a forward-valid target passes, a forward-direction mismatch yields one `DisallowedEdgeTarget` reported with the authored inverse source/target/edge_type | Integration | P0 | FR-041-AC-4 | 🚧 |
| TC-656 | Inverse recognition and normalization are warnings only — they never block extraction or FR-032 structural validation; the inverse index and diagnostics are deterministic across runs and thread counts | Property | P0 | FR-041-AC-5, NFR-006 | 🚧 |
| TC-657 | Each EARS pattern (`ubiquitous`, `event`, `state`, `unwanted`, `optional`, `complex`) classifies from a representative statement; a statement matching no pattern is reported `unclassifiable` | Unit | P0 | FR-042-AC-1 | 🚧 |
| TC-658 | A statement with two `shall` clauses yields exactly one `non-singular` finding; an enumerated `The X SHALL:` stem followed by a numbered list yields none | Unit | P0 | FR-042-AC-2 | 🚧 |
| TC-659 | A statement with no system subject yields a `missing-subject` finding; a `StR` statement with a stakeholder subject (`The operator …`) yields none | Unit | P0 | FR-042-AC-3 | 🚧 |
| TC-660 | A statement using a vague response verb (`shall support`) yields a `vague-response` finding; a passive-voice statement (`shall be included`) yields none | Unit | P0 | FR-042-AC-4 | 🚧 |
| TC-661 | A statement leading with `On startup, … shall …` yields a `non-canonical-trigger` finding; an `NFR` statement with no trigger yields none | Unit | P0 | FR-042-AC-5 | 🚧 |
| TC-662 | A grammar runs only against its bound `(archetype, section)` pairs: an EARS rule bound to FR `Description` yields no findings against an FR `Dependencies` section or an `IT` document | Unit | P0 | FR-042-AC-6 | 🚧 |
| TC-663 | A `warning`-severity finding is recorded in `ValidationResult.warnings` and leaves `is_valid` true; the same finding promoted to `error` is recorded in `errors` and sets `is_valid` false | Unit | P0 | FR-042-AC-7, FR-032 | 🚧 |
| TC-664 | Each grammar finding carries the offending statement excerpt, a 1-based line number, the matched pattern label, and a severity | Unit | P0 | FR-042-AC-8 | 🚧 |
| TC-665 | Modal verbs inside fenced code blocks, blockquotes, and reference lines are not segmented as normative statements and yield no findings | Unit | P0 | FR-042-AC-9 | 🚧 |
| TC-666 | The grammar-check entry point exposed via PyO3 returns the same findings as the in-process Rust call for a fixture document | Integration | P0 | FR-042-AC-10, FR-023 | 🚧 |
| TC-667 | A manifest `lexicon` entry loads and `Registry::lexicon()` returns the merged map containing the term | Unit | P0 | FR-043-AC-1 | 🚧 |
| TC-668 | Two modules declaring the same term with different definitions are first-wins + emit one `DuplicateLexiconTerm`; identical redeclaration emits none | Unit | P0 | FR-043-AC-2 | 🚧 |
| TC-669 | With a lexicon containing `pagination`, `shall support pagination` yields no vague-response finding; with the term removed it yields one | Unit | P0 | FR-043-AC-3 | 🚧 |
| TC-670 | Under an empty lexicon, a bare domain noun (no backtick, no mechanism/bound) yields a vague-response finding — proving no hardcoded noun list remains | Unit | P0 | FR-043-AC-4 | 🚧 |
| TC-671 | Under an empty lexicon, backticked-identifier / mechanism / numeric-bound suppression (FR-042) still hold | Unit | P0 | FR-043-AC-5 | 🚧 |
| TC-672 | `validate_document_in_registry` applies the registry lexicon; type-only `validate_document` applies an empty lexicon (more findings) and never errors on the difference | Unit | P0 | FR-043-AC-6 | 🚧 |
| TC-673 | The `check_grammar` PyO3 binding with `module_root` applies that registry's lexicon; without one applies an empty lexicon | Integration | P0 | FR-043-AC-7, FR-023 | 🚧 |
| TC-674 | The glossary harvester collects the `Term` column of a `Glossary` artifact's `## Terms` table into the project term set | Unit | P0 | FR-044-AC-1 | 🚧 |
| TC-675 | The harvester collects the bold term of each `## Ubiquitous Language` bullet (`- **Term** — …`) into the project term set | Unit | P0 | FR-044-AC-2 | 🚧 |
| TC-676 | A combined `GrammarLexicon` contains both the registry lexicon keys and the harvested project terms; a project-only term is recognised concrete | Unit | P0 | FR-044-AC-3 | 🚧 |
| TC-677 | `validate_document_in_registry_with_lexicon` injects the supplied lexicon — a project-term object yields no vague-response, while the module-only lexicon yields one | Unit | P0 | FR-044-AC-4 | 🚧 |
| TC-678 | `validate_bundle` harvests the loaded `Spec`'s project terms and applies the combined lexicon to every document in the bundle | Integration | P0 | FR-044-AC-5, FR-027 | 🚧 |
| TC-679 | Project-glossary suppression is advisory — a doc with project-suppressed + remaining findings reports `is_valid` per its structural errors alone | Unit | P0 | FR-044-AC-6 | 🚧 |
| TC-680 | A repository with no glossary harvests an empty term set; its validation is identical to the module-only lexicon path | Unit | P0 | FR-044-AC-7 | 🚧 |
| TC-681 | Filament Tier 1 fixture emits one validated graph node with frontmatter `id` as `code` and frontmatter `title` as `title` | Unit | P0 | FR-045-AC-1 | ✅ |
| TC-682 | Filament Tier 2 fixture emits graph nodes and record-derived edges equivalent to the Rust DSL extractor for the same ObjectType snapshot | Unit | P0 | FR-045-AC-2 | ✅ |
| TC-683 | Unknown ObjectType, no-frontmatter, malformed `ix://`, duplicate-edge, and plugin-flag fixtures produce diagnostics/errors without panic | Unit | P0 | FR-045-AC-3 | ✅ |
| TC-684 | Relationship sugar and body `ix://` links produce deterministic graph edges with provenance metadata | Unit | P0 | FR-045-AC-4 | ✅ |
| TC-685 | Repeated Filament extraction over identical inputs produces byte-identical JSON ordering and stable ids | Property | P0 | FR-045-AC-5, NFR-020-AC-2 | ✅ |
| TC-686 | Python and WASM Filament extraction bindings return equivalent JSON values for shared parity fixtures | Integration | P0 | FR-046-AC-1, NFR-020-AC-3 | ✅ |
| TC-687 | `@agent-ix/quire-wasm` exports the Filament extraction API and preserves existing parse/extract/validate smoke tests and declarations | Integration | P0 | FR-046-AC-2 | 🚧 (downstream `@agent-ix/quire-wasm`) |
| TC-688 | Binding code contains no extraction-policy branches beyond input/output conversion and error mapping | Static | P0 | FR-046-AC-3 | 🚧 (inspection) |
| TC-689 | Default Rust build has no Python linkage and WASM target check succeeds with filesystem-free features | Compile | P0 | FR-046-AC-4 | 🚧 (CI wasm-target) |
| TC-690 | Static audit finds no PGlite, Electron, HTTP/auth, CloudManager sync, watcher, or embedding dependencies in extraction module/bindings | Static | P0 | FR-045-CON-1, FR-045-CON-2, FR-045-CON-3, FR-045-CON-4, FR-046-CON-1, FR-046-CON-2, FR-046-CON-3, NFR-020-AC-1 | ✅ |
| TC-705 | Malformed-frontmatter fixture (complete fence, unparsable YAML) yields a `parse_failed` error + `frontmatter_unparsable` diagnostic; absent frontmatter stays clean (`no_frontmatter`, empty errors) | Unit | P0 | FR-045-AC-6 | ✅ |
| TC-706 | `extract_frontmatter` status classification (CR-011): empty/whitespace/comment-only block (YAML null) → `Absent` (not `Malformed`); non-null non-mapping (array/scalar) → `Malformed` | Unit | P0 | FR-006-AC-7 | ✅ |
| TC-707 | Shape classification: an outcome-asserting cell classifies `assertion` (canonical), an obligation-shaped cell `obligation`, a Given/When/Then cell `given-when-then`, and a structureless cell `unstructured` + one `unclassifiable` finding | Unit | P0 | FR-047-AC-1 | ✅ |
| TC-708 | A non-empty `Criteria` cell with no modal verb is segmented and checked; an empty cell yields no statement | Unit | P0 | FR-047-AC-2 | ✅ |
| TC-709 | A cell with two `shall` obligations or two `Then` clauses yields exactly one `non-singular` finding; the positive/negative pair idiom yields none | Unit | P0 | FR-047-AC-3 | ✅ |
| TC-710 | A vague-verb outcome clause over an abstract object yields `vague-response`; the same cell with the object in the merged lexicon yields none | Unit | P0 | FR-047-AC-4, FR-043 | ✅ |
| TC-711 | A cell headed by a vacuous predicate with nothing else to check yields `vacuous-outcome`; the same predicate alongside a concrete-object signal, lexicon term, or observable-result verb yields none | Unit | P0 | FR-047-AC-5 | ✅ |
| TC-712 | The `ac` grammar runs on the `Acceptance Criteria` `Criteria` column of every requirement archetype carrying one (FR/NFR/US/StR/IT) plus `### <doc-id>-AC-N` supplements; Constraints cells and NFR Statements receive EARS findings only | Unit | P0 | FR-047-AC-6, FR-042 | ✅ |
| TC-713 | An `ac` finding carries `grammar: "ac"`, check id, excerpt, 1-based line, shape label, severity, and routes into `ValidationResult` per severity | Unit | P0 | FR-047-AC-7 | ✅ |
| TC-714 | `quire validate --summary` histograms by the generic `[<grammar>:<check>]` prefix; a corpus with both `[ears:*]` and `[ac:*]` findings shows both | Integration | P0 | FR-047-AC-8 | ✅ (CLI histogram wiring shipped in `quire-cli` v0.9.0; `tests/cli_coverage.rs`) |
| TC-715 | The `ac` grammar PyO3 entry point returns the same findings as the in-process Rust call for a fixture document | Integration | P0 | FR-047-AC-9, FR-023 | ✅ |
| TC-716 | A manifest `grammar_severity` registry loads and `Registry::grammar_severity()` returns the merged map | Unit | P0 | FR-048-AC-1 | ✅ |
| TC-717 | Conflicting `grammar_severity` redeclarations merge first-wins with one `DuplicateGrammarSeverity`; identical redeclaration emits none | Unit | P0 | FR-048-AC-2 | ✅ |
| TC-718 | With `ac:unclassifiable` mapped to `error`, an unclassifiable criteria cell lands in `ValidationResult.errors` (is_valid false) while unmapped `ears` findings stay warnings | Unit | P0 | FR-048-AC-3 | ✅ |
| TC-719 | A finding whose `<grammar>:<check>` key is absent from the merged map defaults to `warning` | Unit | P0 | FR-048-AC-4 | ✅ |
| TC-720 | `quire validate --severity ears:vague-response=error` fails a vague-only document and overrides a conflicting manifest entry; repeated `--severity` entries in one invocation apply independently | Integration | P0 | FR-048-AC-5 | ✅ (CLI flag wiring shipped in `quire-cli` v0.9.0; `tests/cli_coverage.rs`) |
| TC-721 | `--strict` semantics unchanged: exits 1 on any warning with --strict, 0 for warning-only documents without it | Integration | P0 | FR-048-AC-6 | ✅ (CLI exit-code check shipped in `quire-cli` v0.9.0; `tests/cli_coverage.rs`) |
| TC-722 | The type-only `validate_document` path applies the all-default severity map (every grammar finding is a warning) | Unit | P0 | FR-048-AC-7 | ✅ |
| TC-723 | A malformed `grammar_severity` entry (unknown level, non-string key) fails module load | Unit | P0 | FR-048-AC-8 | ✅ |
| TC-724 | An AC `Verification` reference to a TC id present in the resolution set validates with no `dangling-trace-reference` finding | Integration | P0 | FR-049-AC-1 | ✅ |
| TC-725 | A `Verification` reference to an absent TC id yields one `dangling-trace-reference` finding with document path and unresolved id | Integration | P0 | FR-049-AC-2 | ✅ |
| TC-726 | The same dangling trace reference is an error under `Strict` and a warning under `Okf` | Integration | P0 | FR-049-AC-3, FR-038 | ✅ |
| TC-727 | A fixture module declaring a different annotation pattern/column resolves references by its own declaration (no ISO behavior in the engine) | Integration | P0 | FR-049-AC-4 | ✅ |
| TC-728 | A declared auxiliary trace source outside the corpus walk (tests.md-style matrix) contributes minted trace ids via targeted scan | Integration | P0 | FR-049-AC-5 | ✅ |
| TC-729 | With no traceability model declared, `validate_bundle` emits zero `dangling-trace-reference` findings | Unit | P0 | FR-049-AC-6 | ✅ |
| TC-730 | A multi-annotation cell (`Test (TC-035, TC-036)`) resolves each id independently, reporting only unresolved ones | Unit | P0 | FR-049-AC-7 | ✅ |
| TC-731 | Repeated bundle validation yields identical `dangling-trace-reference` findings in identical order | Property | P0 | FR-049-AC-8, NFR-006 | ✅ |
| TC-732 | A manifest `traceability:` section (targets, references, status vocabulary, trace-tag grammar) loads and the `Registry` exposes the model | Unit | P0 | FR-050-AC-1 | ✅ |
| TC-733 | A malformed `traceability:` section fails module load; an absent section loads with the model undeclared | Unit | P0 | FR-050-AC-2 | ✅ |
| TC-734 | A reference row whose target has no backing `verifies` relation appears in unbacked rows with row and target ids | Integration | P0 | FR-050-AC-3 | ✅ |
| TC-735 | A `complete`-status row with no backing symbol appears in status lies; the same row with a backing symbol does not | Integration | P0 | FR-050-AC-4 | ✅ |
| TC-736 | A symbol trace tag resolving to no declared target/row appears in untracked symbols with file and symbol name | Integration | P0 | FR-050-AC-5 | ✅ |
| TC-737 | Per-minting-document backed/total counts sum to the bundle-wide totals | Unit | P0 | FR-050-AC-6 | ✅ |
| TC-738 | Repeated `quire coverage` runs over identical inputs emit byte-identical JSON | Property | P0 | FR-050-AC-7, NFR-006 | ✅ |
| TC-739 | A non-ISO fixture module (different archetype, id pattern, status values) obtains a correct rollup from its own declaration | Integration | P0 | FR-050-AC-8 | ✅ |
| TC-740 | With no declared traceability model, `quire coverage` exits non-zero with a diagnostic naming the missing declaration | Integration | P0 | FR-050-AC-9 | ✅ (CLI exit shipped in `quire-cli` v0.9.0; `tests/cli_coverage.rs`) |
| TC-741 | Each adapter (Rust/Python/TypeScript) extracts functions, test functions, and containers with language, path, qualified path, kind, and line attribute | Unit | P0 | FR-051-AC-1 | ✅ |
| TC-742 | Reformatting a fixture file leaves every symbol id unchanged; renaming a symbol changes only that symbol's id | Unit | P0 | FR-051-AC-2 | ✅ |
| TC-743 | Rust `#[test]`-family, Python `test_`, and TS `test`/`it` registrations classify as test symbols; siblings do not | Unit | P0 | FR-051-AC-3 | ✅ |
| TC-744 | Canonical markers bind statically: Python `@pytest.mark.trace(...)`, Rust `#[trace(...)]`, and TS `trace(...)` each mint one `verifies` relation per attached trace id, with no code executed | Unit | P0 | FR-051-AC-4 | ✅ |
| TC-745 | A fixture model with different marker names/patterns binds by its own declaration; with no declared forms zero `verifies` relations are minted | Unit | P0 | FR-051-AC-5 | ✅ |
| TC-746 | A trace id attached more than once to one symbol (repeated marker, or marker plus legacy tag) mints one `verifies` relation and one diagnostic | Unit | P0 | FR-051-AC-6 | ✅ |
| TC-747 | Emitted records match the FR-045 graph-record shapes with normalized refs; filament-core ingestion fixtures accept them unchanged | Integration | P0 | FR-051-AC-7, FR-045 | ✅ |
| TC-748 | `defined_in` edges link every symbol to its file and `contains` edges link containers to members, deterministically ordered | Unit | P0 | FR-051-AC-8 | ✅ |
| TC-749 | An unparseable fixture file yields a per-file diagnostic while the rest of the tree extracts normally | Unit | P0 | FR-051-AC-9 | ✅ |
| TC-750 | Repeated extraction over an identical fixture tree emits byte-identical JSON and identical record ids | Property | P0 | FR-051-AC-10, NFR-006 | ✅ |
| TC-751 | An `obligation`-shaped cell and a `given-when-then`-shaped cell each yield one `non-canonical-shape` finding while keeping their shape (other checks run on their outcome clause); an `assertion` cell yields none | Unit | P0 | FR-047-AC-10 | ✅ |
| TC-752 | A check mapped `off` (manifest or `--severity ac:vague-response=off`) records no finding in warnings, errors, or the --summary histogram; sibling checks still report | Unit | P0 | FR-048-AC-9 | ✅ |
| TC-753 | Legacy textual forms (docstring bare id, `Trace:` line, line-comment id, trace-embedding test name) bind during migration with `legacy` provenance and yield mechanical marker-rewrite suggestions where derivable | Unit | P0 | FR-051-AC-11 | ✅ |
| TC-754 | Fenced code blocks and blockquotes inside `### <doc-id>-AC-N` supplement sections are not segmented and yield no `ac` findings; surrounding supplement prose is still checked | Unit | P0 | FR-047-AC-11 | ✅ |
| TC-755 | A malformed `--severity` entry (unknown level, missing `=`, unparseable `<grammar>:<check>` key) is rejected with a usage diagnostic and a non-zero exit before validation runs | Integration | P0 | FR-048-AC-10 | ✅ (CLI usage exit shipped in `quire-cli` v0.9.0; `tests/cli_coverage.rs`) |
| TC-756 | Static boundary audit (TC-690 pattern): the coverage-rollup and symbol-extractor modules contain no network/service I/O and no execution of extracted code | Static | P0 | FR-050-CON-2, FR-051-CON-1 | ✅ |
| TC-758 | A status cell with a trailing note classes by its leading marker; a declared `retired` value classes retired, not unknown | Unit | P1 | FR-050-AC-10 | ✅ |
| TC-759 | A declared `vocabularies.test_type` is exposed on the `Registry` as core values ∪ module extensions | Unit | P1 | FR-050-AC-11 | ✅ |
| TC-760 | `expand_ranges` resolves `FR-001..FR-003` as three references; `strip_annotations` resolves a parenthetical-qualified cell as one; both off unless declared | Unit | P1 | FR-050-AC-12 | ✅ |
| TC-757 | Module `observable_verbs` and `vacuous_predicates` registries each merge first-wins over their built-in defaults (a module-added observable verb suppresses `vacuous-outcome`); with no declaration both built-in sets apply unchanged | Unit | P0 | FR-047-AC-12, FR-043 | ✅ |
| TC-762 | Module discovery loads in sorted path order regardless of directory-entry order, so every first-wins registry merge resolves a collision identically on every machine | Unit | P0 | NFR-006-AC-5 | ✅ |
| TC-761 | A quoted keyword is a mention: a cell quoting `shall` or `Given`/`When`/`Then` in a code span classifies `assertion` and yields no `non-canonical-shape`/`non-singular` finding, while the unquoted form still does; signal and lexicon checks still read inside the span, and an unbalanced backtick opens no span | Unit | P0 | FR-047-AC-13 | ✅ |
| TC-763 | An elided-copula predication is a predicate: an existential/quantifier head or a predicative adjective classifies `assertion` and yields no `unclassifiable` finding, while a bare noun phrase, a bolded heading and a dangling prose fragment each still classify `unstructured` and yield one | Unit | P1 | FR-047-AC-14 | ✅ |
| TC-766 | `Registry::with_grammar_severity` layers a surface's `--severity` map over the module-declared one: the override wins for its key, the module set is shared rather than rebuilt, and the original registry is untouched | Unit | P1 | FR-048-AC-5 | ✅ |
| TC-691 | Shared fixture: Tier 1 frontmatter node shape, stable normalized ref, `code`, `title`, and extra frontmatter data (shared fixture) | Unit | P0 | FR-045-AC-1 | ✅ |
| TC-692 | Shared fixture: Tier 2 DSL record extraction, schema validation, and record-derived edge emission (shared fixture) | Unit | P0 | FR-045-AC-2 | ✅ |
| TC-693 | Shared fixture: explicit frontmatter `edges:` preserve order, target refs, source refs, and metadata (shared fixture) | Unit | P0 | FR-045-AC-4 | ✅ |
| TC-694 | Shared fixture: relationship sugar fields emit expected edge types, normalized refs, and provenance metadata (shared fixture) | Unit | P0 | FR-045-AC-4 | ✅ |
| TC-695 | Shared fixture: `ix://` targets pass through and bare targets normalize to `ix://agent-ix/<repo>/<value>` (shared fixture) | Unit | P0 | FR-045-CON-4 | ✅ |
| TC-696 | Shared fixture: body markdown `ix://` links emit `references` graph edges and ignore external web links (shared fixture) | Unit | P0 | FR-045-AC-4 | ✅ |
| TC-697 | Shared fixture: duplicate graph edges dedupe by source/type/target, first edge wins, diagnostic emitted (shared fixture) | Unit | P0 | FR-045-AC-3 | ✅ |
| TC-698 | Shared fixtures: malformed `relationships:` entries report errors for non-map, missing target, and missing type (shared fixture) | Unit | P0 | FR-045-AC-3 | ✅ |
| TC-699 | Shared fixture: malformed explicit `edges:` entries report an extraction error without panic (shared fixture) | Unit | P0 | FR-045-AC-3 | ✅ |
| TC-700 | Shared fixture: schema validation failure prevents node emission and surfaces a field-specific error (shared fixture) | Unit | P0 | FR-045-AC-2, FR-045-AC-3 | ✅ |
| TC-701 | Shared fixtures: unknown object, no frontmatter, unsupported plugin flag, and malformed body `ix://` produce diagnostics/errors without panic (shared fixture) | Unit | P0 | FR-045-AC-3 | ✅ |
| TC-702 | Shared fixture: negative scope rejects wikilinks, `spec://`, prose cues, and `https://` as graph edges (shared fixture) | Unit | P0 | FR-045-AC-4 | ✅ |
| TC-703 | Shared corpus isolation: failing extraction fixtures do not poison later successful fixture extraction (shared fixture) | Unit | P0 | FR-045-AC-3 | ✅ |
| TC-704 | Shared corpus determinism: every canonical graph fixture produces byte-identical JSON over repeated extraction | Property | P0 | FR-045-AC-5, NFR-020-AC-2 | ✅ |
| TC-767 | Python and WASM bindings return equivalent JSON over every canonical graph fixture | Integration | P0 | FR-046-AC-1, NFR-020-AC-3 | ✅ |
| TC-768 | parser-lib shim returns core-data-valid payloads matching canonical graph fixture expectations (compatibility reference) | Integration | P0 | FR-118 compatibility reference | ✅ |
| TC-769 | Filament IDE worker merges a real quire-wasm canonical graph fixture into `CoreSyncFilePayload` (Filament IDE FR-046 reference) | Integration | P0 | Filament IDE FR-046 reference | ✅ |
| TC-502 | Static audit: no Mutex/RwLock/Atomic in first-party src/; parallel parse collects owned results | Static | P0 | FR-024-AC-9 | 🚧 |
| TC-503 | loom: parallel parse collection race-free; identical path-sorted output across all interleavings | Property | P0 | NFR-017-AC-1, NFR-017-AC-2, NFR-017-AC-3 | 🚧 |
| TC-504 | TSAN lane: two-thread `load_repo` (GIL-release window) reports zero data races | Integration | P0 | NFR-018-AC-1, NFR-018-AC-3 | 🚧 |
| TC-505 | ASAN lane: FFI object-handoff test set reports zero leaks/UAF (interpreter noise suppressed) | Integration | P0 | NFR-018-AC-2, NFR-018-AC-3 | 🚧 |
| TC-506 | `rg 'unsafe {' src/` returns zero matches with `--features python` enabled | Static | P0 | NFR-003-AC-4 | 🚧 |
| TC-507 | (RETIRED — ADR 0006) miri job removed; FFI scope note moot ((RETIRED)) | Static | P1 | NFR-012-AC-5 | ⛔ |
| TC-582 | Crate root carries `#![cfg_attr(not(feature = "python"), forbid(unsafe_code))]`; default `cargo build` compiles (compiler proves zero first-party unsafe) and adding a first-party `unsafe` block fails the default build; `--features python` compiles with forbid scoped off | Static | P0 | NFR-003-AC-5 | 🚧 |
| TC-510 | `quire.render(archetype, module_root, data)` byte-equals `quire_rs::render_by_name` for same inputs ((RETIRED)) | Integration | P0 | FR-028-AC-1 | ⛔ RETIRED — render removed |
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
| TC-646 | Authoring a doc of type `T` with `object: O`, the skeleton's Relationships block lists the union of `T`'s and `O`'s allowed verbs with category, description, and valid targets | Integration | P0 | US-015-AC-1, FR-040 | 🚧 |
| TC-647 | A `relationships[].type` in neither the artifact's nor the object's resolved vocabulary surfaces a `DisallowedEdgeType` naming the document and verb | Integration | P0 | US-015-AC-2, FR-040 | 🚧 |
| TC-648 | An edge to a target whose object-type/roles do not satisfy the verb surfaces `DisallowedEdgeTarget`; an edge whose target carries the required role passes across module boundaries | Integration | P0 | US-015-AC-3, FR-040 | 🚧 |
| TC-649 | A verb absent from merged `edge_types`, or a role absent from merged `roles`, is rejected at module load — an author cannot reference an undefined edge or role | Integration | P0 | US-015-AC-4, FR-040 | 🚧 |
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
| TC-577 | Bench: `bench_validate_document` on a typical FR-sized artifact median <1ms (warm registry); >10% vs baseline fails CI | Benchmark | P0 | NFR-002-AC-4 | ✅ |
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
| TC-608 | `section_body` `assert: {matches: '<regex>'}` — a `## Story` body with the `As a … / I want … / So that …` shape passes; a body lacking it fails with reason `assert` (line-numbered at the section); a missing section does NOT fire `matches`; `matches` rejected on `table_row` at load time (extends the TC-570 legality matrix) | Unit | P0 | FR-033-AC-10 | ✅ |
| TC-633 | CR-010 `choices` scalar enum: a `section_body`/`frontmatter_field`/`heading`/`list_item` `assert: {choices: [low, medium, high]}` passes on an exact (trimmed) value, fails reason `assert` on a non-member, and does NOT fire when the value is absent; `choices` rejected on `table_row`/`code_block` at load time (extends the TC-570 legality matrix) | Unit | P0 | FR-033-AC-11 | 🚧 |
| TC-634 | `column_choices` per-column table enum: `assert: {column_choices: {Severity: [low, medium, high]}}` passes when every `Severity` cell is a member, fails reason `assert` when any cell is not, and fails with a "column not found" reason when the named column is absent; rejected on non-`table_row` at load time | Unit | P0 | FR-033-AC-12 | 🚧 |
| TC-635 | `column_patterns` per-column table regex: `assert: {column_patterns: {ID: '^FND-\d+$'}}` passes when every `ID` cell matches, fails reason `assert` on a non-match, supports `{field}` interpolation, and fails "column not found" when absent; rejected on non-`table_row` at load time | Unit | P0 | FR-033-AC-13 | 🚧 |
| TC-609 | `section_body_pattern` lint rule: body matching `pattern` → no finding; body present but not matching → exactly one finding (default or custom `message`, severity mirrors rule); `archetypes:` scoping skips non-matching/unresolvable archetypes; missing section → no finding; `type: section_body_pattern` YAML round-trips | Unit | P0 | FR-036-AC-6 | ✅ |
| TC-764 | `forbidden_section` lint rule: section present → exactly one finding (default or custom `message`, severity mirrors rule); section absent → none; `archetypes:` scoping skips non-matching/unresolvable archetypes; `type: forbidden_section` YAML round-trips | Unit | P1 | FR-036-AC-7 | ✅ |
| TC-770 | `optional_columns: [Priority]` — a table omitting the declared-optional column validates | Unit | P0 | FR-033-AC-14 | ✅ |
| TC-771 | The same contract still accepts a table that does carry the optional column | Unit | P0 | FR-033-AC-14 | ✅ |
| TC-772 | Omitting a non-optional column still fails, so `optional_columns` does not degrade into "any subset will do" | Unit | P0 | FR-033-AC-14 | ✅ |
| TC-773 | A `column_choices` entry on a declared-optional column does not fire when that column is absent | Unit | P0 | FR-033-AC-15 | ✅ |
| TC-774 | The same `column_choices` entry is enforced once the optional column is authored | Unit | P0 | FR-033-AC-15 | ✅ |
| TC-610 | Composed type+object validation: `type: FR` + `object: process` with the FR core present but **no** `## Workflow` mermaid block → an object **error** (process required `diagram` missing) merged into `errors`, while the FR (`type`) portion passes independently; `is_valid==false` | Unit | P0 | FR-032-AC-11, FR-032-AC-13 | ✅ |
| TC-611 | Unknown object type: `type: FR` (conformant) + `object: totally-unknown` → exactly one **warning** (reason `unknown-object-type`, message names `totally-unknown`), zero errors, `is_valid==true` | Unit | P0 | FR-032-AC-12 | ✅ |
| TC-612 | No `object:` key (registry-aware entry point): `type: FR` conformant doc → no object-layer diagnostics at all (errors + warnings unchanged from the type-only path) | Unit | P0 | FR-032-AC-11 | ✅ |
| TC-613 | Composed conformant: `type: FR` + `object: process` WITH a valid `## Workflow` mermaid block → no object errors, no warnings, `is_valid==true` | Unit | P0 | FR-032-AC-13 | ✅ |

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
| US-015-AC-1 | TC-646 |
| US-015-AC-2 | TC-647 |
| US-015-AC-3 | TC-648 |
| US-015-AC-4 | TC-649 |

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
| FR-026-AC-9 | TC-620 |
| FR-026-AC-10 | TC-621 |
| FR-026-AC-11 | TC-622 |
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
| FR-032-AC-11 | TC-610, TC-612 |
| FR-032-AC-12 | TC-611 |
| FR-032-AC-13 | TC-610, TC-613 |
| FR-033-AC-1 | TC-534 |
| FR-033-AC-2 | TC-535 |
| FR-033-AC-3 | TC-536 |
| FR-033-AC-4 | TC-537, TC-562 |
| FR-033-AC-5 | TC-538 |
| FR-033-AC-6 | TC-539 |
| FR-033-AC-7 | TC-570 |
| FR-033-AC-8 | TC-571 |
| FR-033-AC-9 | TC-572, TC-561, TC-562 |
| FR-033-AC-10 | TC-608 |
| FR-033-AC-11 | TC-633 |
| FR-033-AC-12 | TC-634 |
| FR-033-AC-13 | TC-635 |
| FR-033-AC-14 | TC-770, TC-771, TC-772 |
| FR-033-AC-15 | TC-773, TC-774 |
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
| FR-036-AC-6 | TC-609 |
| FR-036-AC-7 | TC-764 |
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
| FR-039-AC-1 | TC-623 |
| FR-039-AC-2 | TC-624 |
| FR-039-AC-3 | TC-625 |
| FR-039-AC-4 | TC-626 |
| FR-039-AC-5 | TC-627 |
| FR-039-AC-6 | TC-628 |
| FR-039-AC-7 | TC-629 |
| FR-039-AC-8 | TC-630 |
| FR-039-AC-9 | TC-631 |
| FR-039-AC-10 | TC-632 |
| FR-039-AC-11 | TC-765 |
| FR-040-AC-1 | TC-636 |
| FR-040-AC-2 | TC-637 |
| FR-040-AC-3 | TC-650 |
| FR-040-AC-4 | TC-638 |
| FR-040-AC-5 | TC-651 |
| FR-040-AC-6 | TC-639 |
| FR-040-AC-7 | TC-640 |
| FR-040-AC-8 | TC-641 |
| FR-040-AC-9 | TC-642 |
| FR-040-AC-10 | TC-643, TC-644 |
| FR-040-AC-11 | TC-645 |
| FR-041-AC-1 | TC-652 |
| FR-041-AC-2 | TC-653 |
| FR-041-AC-3 | TC-654 |
| FR-041-AC-4 | TC-655 |
| FR-041-AC-5 | TC-656 |
| FR-042-AC-1 | TC-657 |
| FR-042-AC-2 | TC-658 |
| FR-042-AC-3 | TC-659 |
| FR-042-AC-4 | TC-660 |
| FR-042-AC-5 | TC-661 |
| FR-042-AC-6 | TC-662 |
| FR-042-AC-7 | TC-663 |
| FR-042-AC-8 | TC-664 |
| FR-042-AC-9 | TC-665 |
| FR-042-AC-10 | TC-666 |
| FR-043-AC-1 | TC-667 |
| FR-043-AC-2 | TC-668 |
| FR-043-AC-3 | TC-669 |
| FR-043-AC-4 | TC-670 |
| FR-043-AC-5 | TC-671 |
| FR-043-AC-6 | TC-672 |
| FR-043-AC-7 | TC-673 |
| FR-044-AC-1 | TC-674 |
| FR-044-AC-2 | TC-675 |
| FR-044-AC-3 | TC-676 |
| FR-044-AC-4 | TC-677 |
| FR-044-AC-5 | TC-678 |
| FR-044-AC-6 | TC-679 |
| FR-044-AC-7 | TC-680 |
| FR-006-AC-7 | TC-706 |
| FR-045-AC-1 | TC-681, TC-691 |
| FR-045-AC-2 | TC-682, TC-692, TC-700 |
| FR-045-AC-3 | TC-683, TC-697, TC-698, TC-699, TC-700, TC-701, TC-703 |
| FR-045-AC-4 | TC-684, TC-693, TC-694, TC-696, TC-702 |
| FR-045-AC-5 | TC-685, TC-704 |
| FR-045-AC-6 | TC-705 |
| FR-046-AC-1 | TC-686, TC-767 |
| FR-046-AC-2 | TC-687 |
| FR-046-AC-3 | TC-688 |
| FR-046-AC-4 | TC-689 |
| NFR-020-AC-1 | TC-690 |
| NFR-020-AC-2 | TC-685, TC-704 |
| NFR-020-AC-3 | TC-686, TC-767 |
| FR-047-AC-1 | TC-707 |
| FR-047-AC-2 | TC-708 |
| FR-047-AC-3 | TC-709 |
| FR-047-AC-4 | TC-710 |
| FR-047-AC-5 | TC-711 |
| FR-047-AC-6 | TC-712 |
| FR-047-AC-7 | TC-713 |
| FR-047-AC-8 | TC-714 |
| FR-047-AC-9 | TC-715 |
| FR-047-AC-10 | TC-751 |
| FR-047-AC-11 | TC-754 |
| FR-047-AC-12 | TC-757 |
| FR-047-AC-13 | TC-761 |
| FR-047-AC-14 | TC-763 |
| FR-048-AC-1 | TC-716 |
| FR-048-AC-2 | TC-717 |
| FR-048-AC-3 | TC-718 |
| FR-048-AC-4 | TC-719 |
| FR-048-AC-5 | TC-720, TC-766 |
| FR-048-AC-6 | TC-721 |
| FR-048-AC-7 | TC-722 |
| FR-048-AC-8 | TC-723 |
| FR-048-AC-9 | TC-752 |
| FR-048-AC-10 | TC-755 |
| FR-049-AC-1 | TC-724 |
| FR-049-AC-2 | TC-725 |
| FR-049-AC-3 | TC-726 |
| FR-049-AC-4 | TC-727 |
| FR-049-AC-5 | TC-728 |
| FR-049-AC-6 | TC-729 |
| FR-049-AC-7 | TC-730 |
| FR-049-AC-8 | TC-731 |
| FR-050-AC-1 | TC-732 |
| FR-050-AC-2 | TC-733 |
| FR-050-AC-3 | TC-734 |
| FR-050-AC-4 | TC-735 |
| FR-050-AC-5 | TC-736 |
| FR-050-AC-6 | TC-737 |
| FR-050-AC-7 | TC-738 |
| FR-050-AC-8 | TC-739 |
| FR-050-AC-9 | TC-740 |
| FR-050-AC-10 | TC-758 |
| FR-050-AC-11 | TC-759 |
| FR-050-AC-12 | TC-760 |
| FR-051-AC-1 | TC-741 |
| FR-051-AC-2 | TC-742 |
| FR-051-AC-3 | TC-743 |
| FR-051-AC-4 | TC-744 |
| FR-051-AC-5 | TC-745 |
| FR-051-AC-6 | TC-746 |
| FR-051-AC-7 | TC-747 |
| FR-051-AC-8 | TC-748 |
| FR-051-AC-9 | TC-749 |
| FR-051-AC-10 | TC-750 |
| FR-051-AC-11 | TC-753 |

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
| NFR-006-AC-5 | TC-762 |
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

**Coverage status: 446 / 446 ACs covered (100%).** The AC-grammar/traceability-coverage slice (FR-047..FR-051, US-017, 2026-08-04) adds FR-047-AC-1..14 (acceptance-criteria grammar `ac`: assertion-canonical shape classification with obligation/GWT recognized-but-steered via `non-canonical-shape` (CR-013; EARS was the original canon), every-cell segmentation, and the five shipped checks — `unclassifiable` (structural: no predicate at all), `non-singular`, lexicon-backed `vague-response`, `vacuous-outcome` (a closed, module-extensible `vacuous_predicates` set suppressed by any concrete signal, lexicon term, or declared observable verb) and `non-canonical-shape` (CR-014; `observable_verbs` keeps its ADR-0009 module-data role, demoted from a membership test to a suppressor); binding, fenced/blockquote skip in supplements, mention-vs-use masking (CR-017), elided-copula predication (CR-019), generic `[<grammar>:<check>]` --summary; CON-1 gates error-promotion behind a corpus baseline sweep + user sign-off — TC-707..715, TC-751, TC-754, TC-757, TC-761, TC-763), FR-048-AC-1..10 (per-check `grammar_severity` registry over `off`|`warning`|`error` + `--severity` CLI override incl. repeatable form and malformed-entry rejection, first-wins merge, type-only all-default, `off` full suppression — TC-716..723, TC-752, TC-755, TC-766), FR-049-AC-1..8 (model-driven verification-reference integrity, `dangling-trace-reference`, posture degradation, auxiliary trace-source harvest — TC-724..731), FR-050-AC-1..12 (declarative `traceability:` model + generic `quire coverage` rollup: unbacked rows, status lies, untracked symbols, per-group counts, byte-identical output; CR-015 adds the leading-marker status class, declared column vocabularies, and default-off range/annotation normalization — TC-732..740, TC-758..760), and FR-051-AC-1..11 (source-symbol extraction with stable identities; framework-native markers — pytest marker / Rust `#[trace]` attribute / TS `trace()` helper — as the canonical statically-parsed trace form with the textual forms as a sunset-gated legacy class (CON-3); `verifies`/`defined_in`/`contains` relations, FR-045-shaped records — TC-741..750, TC-753; the FR-050-CON-2/FR-051-CON-1 purity constraints are backed by the TC-756 static boundary audit, TC-690 pattern) — 55 ACs. Implementation landed 2026-08-04/05 (Plan-001 Tracks A and B, gates G1/G2 passed, amended by CR-013/CR-014/CR-015): every TC is ✅ except the five stated at the `quire validate` / `quire coverage` **command** level (TC-714, TC-720, TC-721, TC-740, TC-755, awaiting EXT-3 `quire-cli`). The canonical Filament extraction slice (FR-045/FR-046/NFR-020, US-016) adds FR-045-AC-1..6, FR-046-AC-1..4, NFR-020-AC-1..3 (14 ACs incl. FR-006-AC-7 frontmatter status, CR-011), covered by TC-681..706 + TC-767..003. The project-glossary slice (FR-044, 2026-06-23) adds FR-044-AC-1..7 (a repo's authored Ubiquitous-Language terms — a `Glossary` `## Terms` table + `## Ubiquitous Language` bullets — are harvested and composed with the module lexicon into an ad-hoc `GrammarLexicon` injected via `validate_document_in_registry_with_lexicon`; the corpus `validate_bundle` applies it per doc; advisory and a no-op when no glossary exists — TC-674..680) — 7 ACs. The module-lexicon slice (FR-043, ADR 0009, 2026-06-23) adds FR-043-AC-1..7 (modules ship a mergeable `lexicon:` registry the EARS object-aware vague-response check consumes; the engine drops its hardcoded concrete-noun list; the type-only path degrades to an empty lexicon; PyO3 `check_grammar` gains `module_root` — TC-667..673) — 7 ACs. The requirement-grammar slice (FR-042, EARS, 2026-06-22) adds FR-042-AC-1..10 (grammar-check framework with EARS as the first grammar: six-pattern classification, the non-singular/missing-subject/vague-response/non-canonical-trigger clause checks with per-archetype dialects, warning→error severity routing into `ValidationResult`, fenced/quote/reference skip, and PyO3 parity — TC-657..666) — 10 ACs. The authorable-inverse-edges slice (FR-041, ADR 0008, 2026-06-21) adds FR-041-AC-1..5 (declared `inverse:` labels become authorable as derived views of their forward edge: inverse index, Tier-1 recognition, precedence/`DuplicateInverseEdge`, Tier-2 forward normalization, warn-only determinism, TC-652..656) — 5 ACs. The object-edge-vocabulary slice (FR-040, 2026-06-20) adds FR-040-AC-1..11 (object-axis typed edge vocabulary + cross-domain role-typed targets: mergeable `edge_types`/`roles` registries with first-wins+diagnostic merge, object `roles` parsed onto the archetype, array|map `allowed_links`, union resolution, warn-tier Tier-1/Tier-2 validation, composed skeleton, TC-636..645 + TC-650/651) and US-015-AC-1..4 (author declares an object's relationship vocabulary, TC-646..649) — 15 ACs. The per-value assert slice (CR-010, 2026-06-20) adds FR-033-AC-11..13 (`choices` scalar enum + `column_choices`/`column_patterns` per-column table validation, TC-633..635) — 3 ACs. The internal-links slice (ADR 0007, 2026-06-17) adds FR-026-AC-9..11 (relative-path link edge source + index/log exclusion + dedup parity, TC-620..622) and FR-039-AC-1..10 (unlinked-reference detection & autofix suggestions, incl. AC-10 multi-token code-span skip, TC-623..632) — 13 ACs. The composed type+object validation slice (2026-06-16) adds FR-032-AC-11..13 (`validate_document_in_registry` composes the `type` archetype with the frontmatter `object:` archetype; resolved-object failures are errors, unknown-object is a warning, `ValidationResult` carries typed `warnings`) — TC-610..613, 3 ACs. The assert/lint extension slice (2026-06-16) adds FR-033-AC-10 (CR-008 `matches` content assert, TC-608) and FR-036-AC-6 (CR-009 `section_body_pattern` lint rule, TC-609) — 2 ACs. The binding-contract slice (CR-020, 2026-08-06) adds FR-036-AC-7 (`forbidden_section` lint rule — TC-764) and FR-039-AC-11 (`-VC-` sub-id kind in `parent_id` and the token regex — TC-765) — 2 ACs. The OKF slice (2026-06-16) adds FR-037-AC-1..6 (base concept frontmatter schema, TC-590..596 + TC-528) and FR-038-AC-1..8 (OKF bundle validation, TC-600..607) — 14 ACs. v0.4 adds FR-011-AC-21 (CR-006 `multiple: true`, TC-583) and FR-036-AC-1..5 (declarative lint rules, TC-584..588). v0.2 block model added 16 ACs (FR-019..022, TC-400..443). v0.3 adds 81 ACs — StR-005/006, US-011..013, FR-023..027 (incl. review-added FR-026-AC-8, FR-027-AC-9), NFR-015/016, plus the hardening re-review (NFR-003-AC-4, FR-024-AC-9, NFR-017, NFR-018) — covered by TC-455..507 (plus reused TC-456..459). The Miri ACs (NFR-012-AC-1..5) were **retired** (ADR 0006) and the compile-time **NFR-003-AC-5** (`forbid(unsafe_code)`, TC-582) added. PC (performance criteria) for US-011..013 are tracked as benches (TC-455..459, TC-469) and marked 🚧 pending implementation, consistent with the US-006..010 perf-bench convention. The v0.3 hardening re-review (loom NFR-017, TSAN/ASAN NFR-018) is recorded in spec.md §19.

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

**Integrity check (grep-verified):** all **446 distinct file-defined ACs** (definition-anchored: a bold `**<ID>-AC-N**` bullet declaration **or** an `| <ID>-AC-N |` leading cell in a `## Acceptance Criteria` table — both are definitions; the table form became the majority when the NFR sections were converted to the required table shape for spec-artifacts-iso#11) across `stakeholder/ usecase/ functional/ non-functional/` appear in the AC→TC audit table — **0 uncovered**. Note: `FR-900-AC-1/2` appearing inside FR-034-AC-1's example prose are NOT defined ACs and are excluded from the denominator (match `**…**:` definitions, not inline mentions). Retired ACs (marked `(RETIRED)`, un-bolded) are excluded by construction. Count: 316 (pre-removal) − 41 (retired) + 16 (back-fill) + 1 (FR-011-AC-20, CR-005 heading normalization) − 5 (NFR-012-AC-1..5 retired, ADR 0006) + 1 (NFR-003-AC-5, forbid(unsafe_code)) + 1 (FR-011-AC-21, CR-006 multiple:true) + 5 (FR-036-AC-1..5, declarative lint rules) + 1 (FR-010-AC-4, CR-007 escaped pipes) + 6 (FR-037-AC-1..6, OKF base concept schema) + 8 (FR-038-AC-1..8, OKF bundle validation) + 1 (FR-033-AC-10, CR-008 `matches` content assert) + 1 (FR-036-AC-6, CR-009 `section_body_pattern`) + 3 (FR-032-AC-11..13, composed type+object validation) + 3 (FR-026-AC-9..11, relative-path link edge source) + 10 (FR-039-AC-1..10, unlinked-reference detection incl. multi-token code-span skip) + 3 (FR-033-AC-11..13, CR-010 per-value enum/regex asserts) + 11 (FR-040-AC-1..11, object-axis typed edge vocabulary + cross-domain targets) + 4 (US-015-AC-1..4, author declares object relationship vocabulary) + 5 (FR-041-AC-1..5, authorable inverse edge verbs) + 10 (FR-042-AC-1..10, requirement-grammar check (EARS)) + 7 (FR-043-AC-1..7, module-supplied concrete lexicon) + 7 (FR-044-AC-1..7, project Ubiquitous-Language lexicon) + 13 (FR-045-AC-1..6 + FR-046-AC-1..4 + NFR-020-AC-1..3, canonical Filament extraction) + 1 (FR-006-AC-7, frontmatter status, CR-011) + 14 (FR-047-AC-1..14, acceptance-criteria grammar incl. non-canonical-shape, supplement skip, module-data observable verbs, CR-017 quoted-keyword masking, CR-019 elided-copula predication) + 10 (FR-048-AC-1..10, per-check grammar severity incl. `off` + malformed CLI-entry rejection) + 8 (FR-049-AC-1..8, verification-reference integrity) + 12 (FR-050-AC-1..12, declarative coverage computation incl. CR-015) + 11 (FR-051-AC-1..11, source symbol extraction with relations incl. canonical markers + legacy class) + 1 (NFR-006-AC-5, sorted module discovery, CR-018) + 1 (FR-036-AC-7, `forbidden_section` lint rule, CR-020) + 1 (FR-039-AC-11, `-VC-` sub-id kind, CR-020) = **446**.

---

## Coverage Gaps

| Gap ID | Description | Risk Level | Mitigation |
|--------|-------------|------------|------------|
| GAP-001 | DSL evaluator parity test (TC-040) needs a curated fixture document per object_type across all 87+ types; some fixtures may not yet exist in the source repos. | Medium | Track per-type fixture availability in `tests/extract_parity/coverage.md`; missing fixtures are P1 follow-ups. |
| GAP-002 | Python Jinja2 reference renderer is not byte-stable across Jinja2 minor versions in all whitespace cases. | Low | StR-002-AC-2 documents known whitespace exceptions; pin reference's Jinja2 version. |
| GAP-003 | Cross-machine determinism (arm64 vs x86_64 byte parity) is implied but not explicitly benched. | Low | Add an arm64 + x86_64 CI matrix as a P2 enhancement. |
| GAP-006 | The 22 `StR-NNN-VC-N` stakeholder validation criteria introduced by the spec-artifacts-iso#11 table conversion are **not traced to any Test Case**. Giving StR criteria stable ids is precisely what makes them traceable (previously they were prose and unaddressable), so this gap is newly *expressible*, not newly created — but it is real and should not read as covered. The `446 / 446` figure above counts **Acceptance** Criteria only; Validation Criteria are a distinct kind and are outside that denominator. The single `StR-001-VC-2` occurrence in this file is TC-765's example prose, not a trace. | Medium | Allocate TCs for StR VC rows in the next matrix pass, or record explicitly that stakeholder validation is evidenced by Demonstration outside the TC matrix. Tracked on agent-ix/spec-artifacts-iso#11. |
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
| Unit | 78 | 0 | 0 | 78 | 0% |
| Integration | 40 | 0 | 0 | 40 | 0% |
| Static (hardening) | 11 | 0 | 0 | 11 | 0% |
| Process | 1 | 0 | 0 | 1 | 0% |
| Integration | 21 | 0 | 0 | 21 | 0% |
| Parity | 7 | 0 | 0 | 7 | 0% |
| Bench | 14 | 0 | 0 | 14 | 0% |
| Property | 14 | 0 | 0 | 14 | 0% |
| Static / Snapshot | 26 | 0 | 0 | 26 | 0% |
| Compile | 5 | 0 | 0 | 5 | 0% |
| Soak | 1 | 0 | 0 | 1 | 0% |
| **Total** | **194** | **0** | **0** | **194** | **0%** |
