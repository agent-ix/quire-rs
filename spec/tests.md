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
| FR-005 parse_document API | AC-1..7 | TC-001, TC-029, TC-812, TC-813 (CR-046 header/body tiers), TC-819 (CR-050 parse_body totality) | ✅ Complete |
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
| FR-019 Stable block identifiers | AC-1..3; CON-1 | TC-400 (block_id parsed; no attribute → None), TC-402 (attribute stripped from heading text), TC-443 (id survives write-back + reparse), TC-403 (negative) | ✅ Complete (document authored, CR-042) |
| FR-020 Block addressing | AC-1..2; CON-1 | TC-410 (nested block_id lookup), TC-411 (block_type → archetype 1:1 alias) | ✅ Complete (CR-042 — the absent `Block` struct is the design, stated as CON-1) |
| ~~FR-021 Block edit API~~ | — | — | ⛔ RETIRED (CR-042: `apply_block_patch`/`replace_block` were render-dependent and removed with render; US-006/US-007 ACs already retired) |
| FR-022 Write-back primitives | AC-1..7; CON-1 | TC-430 (update_section replaces content), TC-431 (update_block replaces heading+content), TC-432 (other blocks byte-identical), TC-433 (frontmatter preserved), TC-434/435 (missing heading/id → MissingField), TC-896 (CR-069 self-write identity + end-of-file separator) | ✅ Complete |
| FR-023 PyO3 binding surface | AC-1..7 | TC-460 (feature-gate), TC-461 (parse parity), TC-462 (validate parity), TC-463 (load_repo via binding), TC-464 (GIL release), TC-465 (abi3 cross-version), TC-466 (no subprocess) | ✅ Complete |
| FR-028 Expanded Python surface | AC-1..8 | TC-510 (⛔ RETIRED — render removed), TC-511 (validate happy+sad), TC-512 (validate_manifest), TC-513 (extract envelope), TC-514 (extract_frontmatter), TC-515 (harvest_edges dict+str), TC-516 (exception hierarchy), TC-517 (GIL release multi-thread) | ✅ Complete |
| FR-029 Archetype input contract (recast, ADR 0004) | AC-1..6 | TC-548 (FR/NFR contract), TC-549 (NFR sections), TC-550 (iso required_sections order), TC-551 (byte-stable JSON), TC-552 (unknown→err), TC-553 (unresolved-mapping diag) | 🚧 Pending implementation |
| FR-030 Required-section validation (superseded by FR-032/FR-033, ADR 0004) | AC-1..6 | TC-529, TC-530, TC-536, TC-528, TC-533 (covered by FR-032/FR-033 TCs) | 🚧 Superseded — covered by FR-032/FR-033 |
| FR-031 Unified archetype shape | AC-1..6 | TC-522 (validatable+extractable, no renderability), TC-523 (no body_extraction → extraction None), TC-524 (defaults retained), TC-525 (two validators), TC-526 (required_sections ignored+diag), TC-527 (resolve parity) | 🚧 Pending implementation |
| FR-032 validate_document (markdown) | AC-1..13 | TC-528..533, TC-561 + TC-573 (placeholder set), TC-574 (none/n-a substantive), TC-575 (empty table/list reason), TC-576 (assert on resolved), TC-610 (composed object error), TC-611 (unknown object → warning), TC-612 (no object key), TC-613 (composed conformant) | ✅ |
| FR-033 Locator assert facet | AC-1..16 | TC-991 (CR-097 row-scoped failures carry the row and its own line), TC-1005 (#254 that line is the row, not the separator), TC-534..539, TC-561/562 + TC-570 (legality matrix), TC-571 (id-column precedence), TC-572 (id_pattern non-table), TC-608 (CR-008 `matches` content assert), TC-633 (CR-010 `choices` scalar enum), TC-634 (`column_choices`), TC-635 (`column_patterns`) | ✅ |
| FR-034 Assert field interpolation | AC-1..4 | TC-540 (id prefix), TC-541 (missing field diag), TC-542 (regex-escape), TC-543 (no-token static regex) | 🚧 Pending implementation |
| FR-035 Per-level heading uniqueness | AC-1..4 | TC-544 (dup L2), TC-545 (cross-level ok), TC-546 (iterate_over children), TC-547 (line number) | 🚧 Pending implementation |
| FR-036 Declarative lint rules | AC-1..6 | TC-584 (manifest→Registry::lint_rules + malformed rule fails load), TC-585 (vocab finding + annotation pass), TC-586 (archetype scoping), TC-587 (missing section/column → none), TC-588 (lint never affects extract/validate), TC-609 (CR-009 `section_body_pattern`) | ✅ |
| FR-037 Base concept frontmatter schema (OKF) | AC-1..6 | TC-590 (minimal typed), TC-591 (optional desc/tags), TC-592 (missing type), TC-593 (empty type), TC-594/595/596 (mistyped desc/tags/non-string item), TC-528 (shape wired into validate_document) | ✅ |
| FR-038 OKF bundle validation (Strict vs Okf + index) | AC-1..8 | TC-600 (strict untyped→error), TC-601 (okf untyped→error), TC-602 (okf tolerates unknown type+broken link, strict rejects), TC-603 (strict conformant+complete index valid), TC-604 (index incompleteness error/warning), TC-605 (root missing okf_version), TC-606 (subdir no okf_version), TC-607 (strict mistyped description) | ✅ |
| FR-024 Parallel repo walk (load_repo) | AC-1..11 | TC-470 (N files→N docs), TC-471 (malformed→diagnostic), TC-472 (gitignore), TC-473 (path-sorted determinism), TC-474 (symlink loop), TC-475 (id derivation), TC-476 (bad root), TC-455 (bench), TC-502 (no shared mutable state in the walk fan-out; named-exemption audit widened to OnceLock/OnceCell — CR-047 — and wired into ci.yml with path-scoped, exact-line, stale-checked exemptions — CR-053), TC-807 (type-driven membership), TC-808 (glossary scanner agrees) | ✅ Complete |
| FR-025 Spec corpus model | AC-1..8 | TC-480 (len), TC-481 (id index), TC-482 (dup id), TC-483 (Send+Sync), TC-484 (scope-guard surface), TC-485 (no-IO queries, incl. lazy body — CR-047), TC-817 (zero-body-parse queries, CR-047), TC-815/TC-816 (concurrent first-touch once + agree, CR-047) | ✅ Complete |
| FR-026 Intra-spec reference resolution | AC-1..14; CON-1 | TC-486 (frontmatter edge), TC-487 (ix:// edge), TC-488 (dangling), TC-489 (cross-spec dangling), TC-490 (bidirectional), TC-491 (target-id extraction), TC-492 (O(edges) proptest), TC-501 (dedup), TC-620 (rel-path edge/dangling), TC-621 (index/log excluded), TC-622 (dedup parity across sources), TC-880 (grammar accepts every authored shape), TC-881 (grammar rejects the bare protocol + placeholders), TC-882 (single-doc surface reads the same grammar) , TC-897 (CR-071 destination exclusions) | ✅ Complete |
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
| NFR-017 Concurrency permutation (loom) | loom exhaustive interleaving (scheduled lane); AC-4 lazy-body first-touch (CR-047) | TC-502, TC-503, TC-815 | ✅ Complete |
| NFR-018 FFI sanitizer lanes (TSAN+ASAN) | scheduled sanitizer lanes on the extension | TC-504, TC-505 | ✅ Complete |
| US-016 Consume canonical Filament extraction | Illustrative examples | TC-681..TC-690 | ✅ Complete |
| US-017 Agent verifies coverage deterministically | Illustrative examples | TC-724..TC-750, TC-753, TC-756 (via FR-049/050/051) | 🚧 Pending implementation |
| FR-045 Canonical Filament core extraction engine | AC-1..6; CON-1..4 | TC-691..TC-704, TC-690, TC-705, TC-768 (downstream compatibility reference) | ✅ Complete |
| FR-046 Filament extraction bindings | AC-1..4; CON-1..3 | TC-686, TC-687, TC-688, TC-689, TC-767, TC-769 (downstream consumer reference) | ✅ Complete |
| FR-047 Acceptance-criteria grammar | AC-1..14; CON-1..2 | TC-707 (shape classification, assertion canonical), TC-708 (every non-empty cell segmented), TC-709 (non-singular + pair idiom), TC-710 (vague-response via lexicon), TC-711 (vacuous-outcome), TC-712 (binding), TC-713 (finding fields + routing), TC-714 (generic --summary prefix), TC-715 (PyO3 parity), TC-751 (non-canonical-shape steers obligation/GWT → assertion), TC-754 (fenced/blockquote skip in supplements), TC-757 (module-data observable/vacuity vocabularies), TC-761 (quoted keywords are mentions, not uses — CR-017), TC-763 (elided-copula predication is a predicate — CR-019) | ✅ Implemented (CLI-surface AC-8 awaits EXT-3 `quire-cli`) |
| FR-048 Per-check grammar severity | AC-1..11 | TC-794 (own `spec/` dogfooded against the shipped promotion), TC-716 (manifest registry + accessor), TC-717 (first-wins + DuplicateGrammarSeverity), TC-718 (per-check error routing), TC-719 (absent key → warning), TC-720 (--severity override + repeatable), TC-721 (--strict unchanged), TC-722 (type-only all-default), TC-723 (malformed entry fails load), TC-752 (`off` suppresses a check entirely), TC-755 (malformed --severity CLI entry rejected) | ✅ Implemented (CLI-surface AC-5/6/10 await EXT-3 `quire-cli`) |
| FR-049 Verification-reference integrity | AC-1..9 | TC-814 (CR-045 two-root reference resolution), TC-724 (resolved reference clean), TC-725 (dangling finding), TC-726 (posture degradation), TC-727 (model-driven pattern/column), TC-728 (auxiliary trace-source harvest), TC-729 (no model → no findings), TC-730 (multi-annotation cells), TC-731 (deterministic findings) | ✅ Implemented |
| FR-050 Declarative coverage computation | AC-1..37; CON-1..2 | TC-1050, TC-1051 (CR-136 an id that binds to nothing and a row backed by nothing, joined), TC-1048, TC-1049 (CR-135 a declared target that matched no document is reported), TC-1033, TC-1034, TC-1035 (CR-117 the archetype matched and the declared table did not), `scripts/tests/test_check_engine.py` (CR-105 the pin/tree/binary agreement gate and its capability abort), `scripts/tests/test_overfit_check.py` (CR-101 cross-corpus generalization statistics),  `scripts/tests/test_bench.py` (CR-099 the corpus benchmark's ratchet and metric semantics), TC-992..TC-996 (CR-098 the declarative regression corpus), TC-989 (CR-095 the catch-all is split out of the headline), TC-983, TC-984 (CR-093 the binding census and its two diagnostics), TC-955, TC-956, TC-957 (CR-089 records carry the matrix-row line), TC-952, TC-953, TC-954 (CR-088 source_exclude observability + loud invalid globs), TC-950, TC-951 (CR-087 shared trace ids are a reported defect), TC-944, TC-945, TC-949 (CR-085 source_exclude scopes the symbol walk), TC-941, TC-942 (CR-083 undeclared status values are a reported defect), TC-946 (CR-086 duplicate undeclared-status rows dedup), TC-829, TC-830 (CR-062 archetype is the only origin), TC-826 (CR-060 model-level exclusion scopes the criteria walk), TC-824 (CR-057 byte-identity baseline gate), TC-822 (CR-054 declarations that select nothing), TC-818 (CR-049 declaration-driven body selection), TC-809, TC-810, TC-811 (CR-045 two roots from one scope), TC-805 (CR-041 no-source-symbol methods), TC-801 (CR-038 declared path scoping), TC-797 (CR-035 zero matched rows ≠ 100%), TC-788 (CR-028 criteria counts + totals), TC-732 (traceability model load), TC-733 (malformed/absent model), TC-734 (unbacked rows), TC-735 (status lies), TC-736 (untracked symbols), TC-737 (per-group counts), TC-738 (byte-identical output), TC-739 (non-ISO model), TC-740 (no model → diagnostic exit), TC-758 (status marker + note, retired class), TC-759 (declared column vocabularies), TC-760 (range expansion + annotation stripping), TC-756 (CON-2 static boundary audit) | ✅ Implemented (AC-9 CLI exit awaits EXT-3 `quire-cli`) |
| FR-051 Source symbol extraction with relations | AC-1..22; CON-1..3 | TC-1044, TC-1045, TC-1046, TC-1047 (CR-134 a tag that reaches no channel is reported), TC-1039, TC-1040 (CR-119 a suite header is a container that binds nothing), TC-1029, TC-1030, TC-1031 (CR-115 python triple-quote state tracked at any column), TC-982 (CR-093 the binder says what it looked at), TC-958, TC-960, TC-961 (CR-090 widened registration grammar pinned through `extract_tree`), TC-943, TC-948 (CR-084 curried and wrapped registrations), TC-827, TC-828 (CR-061 benchmarks and fuzz targets are leaf evidence), TC-806 (CR-043 legacy comma lists bind every id), TC-804 (CR-040 Rust raw strings + lifetimes), TC-803 (CR-039 one lexer pass), TC-800 (CR-037 wrapped signature span), TC-798 (CR-036 string-aware comment stripping), TC-799 (CR-036 template state across lines), TC-741 (adapter symbol extraction), TC-742 (identity stability), TC-743 (test-symbol classification), TC-744 (canonical markers bind statically), TC-745 (marker/tag forms are module data), TC-746 (duplicate-id dedup), TC-747 (FR-045 record shapes), TC-748 (defined_in/contains edges), TC-749 (unparseable-file degradation), TC-750 (byte-identical repeat), TC-753 (legacy textual forms + rewrite suggestions), TC-756 (CON-1 static boundary audit) | ✅ Implemented |
| FR-052 Acceptance-criteria property classification | AC-1..19; CON-1..4 | TC-990 (CR-096 decomposition keys on quantification, not on the label), TC-989 (CR-095 specific-shape split + per-shape span grounding), TC-779 (universal decomposition, three spans), TC-780 (metamorphic idioms + fixed precedence), TC-781 (CR-017 mask parity), TC-782 (weak-only boundary → no spans), TC-783 (`Example`, not extractable, no finding), TC-784 (span/offset invariants), TC-785 (CON-1 `ac` finding stream unchanged), TC-786 (`property_idioms` first-wins merge), TC-787 (binding parity with `ac`), TC-788 (FR-050 rollup counts), TC-789 (PyO3 parity), TC-790 (CON-4 `extractable` registry-independent), TC-791 (closed structural signals without a registry), TC-792 (determiner read at a bounded subject position, CR-030), TC-793 (byte-identity of everything the widening does not claim), TC-795 (three-valued `extraction` outcome, CR-033), TC-796 (`extraction` derived; CON-4 unchanged) | ✅ Implemented |
| FR-053 Obligation record | AC-1..14; CON-1..4 | TC-831 (target-bound source, ids the rollup already mints), TC-832 (archetype-bound source, rendered ids over the NFR measurement table), TC-833 (both-or-neither origin rejected at parse), TC-834 (hash is whitespace-insensitive and word-sensitive, incl. inside code spans), TC-835 (one cell, two readings: method vs FR-049 reference), TC-836 (absent parameters omitted), TC-837 (criticality optional), TC-838 (empty statement skipped and reported), TC-839 (deterministic, ordered), TC-840 (classification record carries it), TC-841 (coverage report carries it; absent key preserves FR-050-AC-7), TC-842 (hash follows the statement, not its position), TC-843 (nested form does not repeat the record), TC-870 (the skipped-row diagnostic reaches the report), TC-871 (NFC), TC-872 (declaration order, not source name), TC-873 (`exclude` binds both surfaces) | ✅ Implemented |
| FR-054 Verification-method catalog | AC-1..13; CON-1..6 | TC-844 (entries exposed intact), TC-845 (first-wins + DuplicateVerificationMethod), TC-846 (undeclared is None, not empty), TC-847 (unknown key fails load), TC-848 (empty required field fails load), TC-849 (derived vocabularies track the merge), TC-850 (test_type unchanged; unknown name empty), TC-851 (CON-2 applicability opaque), TC-852 (CON-3 no finding), TC-853 (CON-4 derived, never authored twice), TC-874 (an uncatalogued method is reported), TC-875 (no catalog asks no question), TC-967 (CR-091 the diagnostic carries the method as a structured `value`), TC-968, TC-969 (CR-092 `cost` exposed intact + additive serialization) | ✅ Implemented |
| FR-055 Published JSON output contract | AC-1..8; CON-1..3 | TC-854 (schemas valid + versioned), TC-855 (baseline conforms), TC-856 (every optional key exercised), TC-857 (emitted criteria conform), TC-858 (additionalProperties closed at depth), TC-859 (optional/required split matches the engine), TC-947 (CR-086 `implements` optional-key record), TC-860 (CON-1/CON-2 no version key, no schemars), TC-1010 (CR-104 instrument provenance is optional and closed) | ✅ Implemented (the `properties` envelope conformance test lives in `quire-cli`, which assembles it) |
| FR-065 Controlled-corpus contract | AC-1..47; CON-1..4 | TC-1011 (read in place; mutating copies), TC-1012 (required fields + control pairing), TC-1013 (optional expect fields), TC-1014 (the bounds vocabulary), TC-1015 (gap_count is a count), TC-1016 (the L1/L2/L3 ladder), TC-1017 (every failure case has its control), TC-1018 (the real module binds by default), TC-1019 (determinism), TC-1020 (reproducible by hand, with a module), TC-1021 (one declaration of each vocabulary, read from corpus.yaml), TC-1022 (a variant varies expectations, not identity), TC-1023 (the live block and the forward block), TC-1024 (unbacked_rows, groups and L3 lists are exact), TC-1025 (the loader refuses a block asserting the wrong thing), TC-1026 (the vocabulary is checked against the engine), TC-1027 (a control binds its partner's declaration), TC-1028 (a failure case discriminates from its control), TC-1032 (a regression case pins a landed fix; an ecosystem-bound one credits its cell, a variant-bound one credits none), TC-1043 (the Rust reader requires what `case_schema` declares required, and models every field it declares) | ✅ |
| FR-058 Upward-trace completeness | AC-1..11; CON-1..2 | TC-898 (orphan reported, linked one not), TC-899 (any declared verb satisfies), TC-900 (incoming direction), TC-901 (dangling cannot satisfy), TC-902 (cycle once, ordered), TC-903 (per-relation severity + advisory), TC-904 (undeclared module unchanged), TC-905 (CON-1 every field survives the merge), TC-906 (unexecutable declaration rejected at load), TC-907 (duplicate relation name rejected), TC-908 (dead relation vocabulary reports itself), TC-909 (a use-case-backed FR is not an orphan), TC-910 (the finding reads as a sentence) | ✅ Implemented |
| FR-059 Declared-vocabulary coverage | AC-1..10; CON-1..4 | TC-911 (unowned value reported), TC-912 (vocabulary read from the schema), TC-913 (justified absence covers), TC-914 (justification on any document), TC-915 (independently tunable), TC-916 (no enum reports itself), TC-917 (undeclared module unchanged), TC-918 (empty projection is one finding), TC-962, TC-963, TC-964, TC-965 (CR-091 owned/excused/unowned payload records + absent-key byte-identity), TC-966 (CR-091 dead declaration is a coverage diagnostic) | ✅ Implemented |
| FR-060 Vocabulary references | AC-1..6; CON-1..3 | TC-919 (column reference resolves), TC-920 (scalar reference resolves), TC-921 (unknown name is empty not absent), TC-922 (literal wins over reference), TC-923 (untouched archetype unchanged), TC-924 (a reference obeys its literal's kind rules) | ✅ Implemented |

| FR-061 Combinatorial obligations | AC-1..10; CON-1..3 | TC-925 (tuple count), TC-926 (strength beyond dimensions is 0), TC-927 (forbidden combination excluded), TC-928 (exclusion bites at higher strength), TC-929 (statement covers the space), TC-930 (cells parse as authored), TC-931 (one obligation minted end to end), TC-932 (no interaction mints nothing), TC-933 (strength 0 rejected at load), TC-934 (the CORPUS path mints the same one obligation — the branch shipped only in the single-document path, so `quire coverage` minted one obligation per dimension row and quoin FR-035 could never see a combinatorial obligation) | ✅ Implemented |
| FR-057 Per-check corpus severity | AC-1..10; CON-1..2 | TC-883 (promote/demote/off), TC-884 (unconfigured tier per check), TC-885 (`--severity`-shaped layering reaches corpus checks), TC-886 (severity carried, reason stable, key well-formed), TC-887 (order unperturbed), TC-888 (CON-1 bridged results not registrable), TC-889 (sibling packs independent) | ✅ Implemented |
| FR-056 Requirement-quality lints | AC-1..13; CON-1..5 | TC-861 (built-in term fires), TC-862 (longest term names the finding), TC-863 (CON-2 module terms layer over built-ins), TC-864 (allocation, not voice), TC-865 (two modals), TC-866 (CR-017 mention parity), TC-867 (CON-1 advisory + per-check `off`), TC-868 (CON-4 ears/ac streams unchanged), TC-869 (checks independent), TC-876 (row-level line attribution), TC-877 (all four modals collected), TC-878 (a deadline or a sort key is not an agent), TC-879 (unknown ambiguity_terms key fails load) | ✅ Implemented |
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
| TC-024 | Roundtrip: reconstructing body from `(preamble, [(heading_line, content)])` byte-equals the input, over 10 000 generated bodies (`src/parser/slice.rs`; carried no TC id until CR-069) | Property | P0 | FR-008-AC-3, NFR-006 | ✅ |
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
| TC-400 | Heading `## Behavior {#blk-7af2}` parses to `block_id = "blk-7af2"` with heading text `Behavior`; a heading with no attribute yields `None` | Unit | P0 | FR-019-AC-1 | ✅ |
| TC-401 | Round-trip through `apply_block_patch` ((RETIRED)) | Integration | P0 | FR-019-AC-2 | ⛔ RETIRED — the API was render-dependent and removed; the surviving round-trip is TC-443 (CR-042) |
| TC-402 | Pandoc attribute stripped from heading text on parse (no `{#…}` trailing in `QuireSection.heading`) | Unit | P0 | FR-019-AC-3 | ✅ |
| TC-403 | Heading without `{#…}` → block_id = None; heading text byte-identical to input ((negative)) | Unit | P0 | FR-019-AC-1 | ✅ |
| TC-410 | Lookup by block id walks the nested section tree, and resolves nothing for an id no heading declares | Unit | P0 | FR-020-AC-1 | ✅ |
| TC-411 | Registry::block_type(name) returns the same CompiledArchetype as archetype(name) | Unit | P1 | FR-020-AC-2 | ✅ |
| TC-420 | `apply_block_patch` merge → validate → render → splice ((RETIRED)) | Unit | P0 | FR-021-AC-1 | ⛔ RETIRED — render removal (CR-042) |
| TC-421 | `replace_block` full-replaces data, renders and splices ((RETIRED)) | Unit | P0 | FR-021-AC-2 | ⛔ RETIRED — render removal (CR-042) |
| TC-422 | `apply_block_patch` merged data violating schema → SchemaViolation ((RETIRED)) | Unit | P0 | FR-021-AC-3 | ⛔ RETIRED — render removal (CR-042) |
| TC-423 | `apply_block_patch` unknown block_type → UnknownArchetype ((RETIRED)) | Unit | P0 | FR-021-AC-4 | ⛔ RETIRED — render removal (CR-042) |
| TC-424 | `apply_block_patch` unknown block_id → MissingField ((RETIRED)) | Unit | P0 | FR-021-AC-5 | ⛔ RETIRED — render removal (CR-042) |
| TC-425 | LLM-flow spliced bytes equal a direct template render ((RETIRED)) | Integration | P0 | FR-021-AC-6 | ⛔ RETIRED — render removal (CR-042) |
| TC-430 | update_section replaces heading's content range; heading line + frontmatter + other sections byte-identical | Unit | P0 | FR-022-AC-1 | ✅ |
| TC-431 | update_block replaces heading + content range together; addresses by block_id, finds nested blocks | Unit | P0 | FR-022-AC-2 | ✅ |
| TC-432 | After update_block, untouched blocks byte-identical (incl. trailing whitespace + nested bullets) | Unit | P0 | FR-022-AC-3 | ✅ |
| TC-433 | Frontmatter (`---\nid: …\n---\n`) byte-identical through update_section + update_block | Unit | P0 | FR-022-AC-4 | ✅ |
| TC-434 | update_section unknown heading → MissingField ((negative)) | Unit | P0 | FR-022-AC-5 | ✅ |
| TC-435 | update_block unknown block_id → MissingField ((negative)) | Unit | P0 | FR-022-AC-5 | ✅ |
| TC-440 | End-to-end `apply_block_patch` over an FR-like artifact ((RETIRED)) | Integration | P0 | FR-021-AC-1 | ⛔ RETIRED — render removal (CR-042) |
| TC-441 | End-to-end `replace_block` renders fresh data ((RETIRED)) | Integration | P0 | FR-021-AC-2 | ⛔ RETIRED — render removal (CR-042) |
| TC-442 | End-to-end empty patch is idempotent ((RETIRED)) | Integration | P1 | FR-021-AC-1 | ⛔ RETIRED — render removal (CR-042) |
| TC-443 | A block id survives a write-back addressed by that id and a reparse of the result | Integration | P0 | FR-019-AC-3 | ✅ |
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
| TC-807 | Membership is type-driven: typed `tests.md` + untyped `tests.md` + unregistered-type `notes.md` load; a frontmatter-less draft and a malformed-frontmatter file under the root are absent from `documents` and each emits exactly one non-fatal `DocumentWithoutFrontmatter` warning naming its path (flavors distinguished); repo-root `README.md`/`CHANGELOG.md` sit outside the walked root and are never visited (CR-044, inverted by CR-048) | Unit | P0 | FR-024-AC-10 | ✅ |
| TC-808 | `glossary_terms_from_path` applies the same membership rule: identical `## Ubiquitous Language` content harvests nothing from a frontmatter-less `README.md` and harvests the term from a document, and both harvesters agree over the tree (CR-044) | Unit | P1 | FR-024-AC-11 | ✅ |
| TC-809 | `extract_tree_excluding` never enters an excluded subtree — the document root passed by coverage yields no symbols from `spec/` — an empty exclusion list extracts identically to `extract_tree`, and the exclusion holds wherever the caller's `is_dir()` held: a case-insensitively matched `Spec/` and a symlinked `spec/` are both excluded by what they resolve to (CR-045, identity comparison CR-056) | Unit | P0 | FR-050-AC-17 | ✅ |
| TC-810 | `quire coverage --scope <repo>` derives the document root `<scope>/spec`: repo-root `README.md`/`CHANGELOG.md`/`plan/*.md` are never read as documents, and the minted-id set over a compliant repo is byte-identical to a pre-split run (CR-045) | Integration | P0 | FR-050-AC-17 | 🚧 awaiting EXT-3 `quire-cli` (test lives in `quire-cli` `tests/cli_coverage.rs`; not verifiable from this repo — CR-058) |
| TC-811 | A `--scope` with no `spec/` directory exits non-zero with a diagnostic naming the missing document root — no silent fallback to walking the scope (CR-045) | Integration | P0 | FR-050-AC-17 | 🚧 awaiting EXT-3 `quire-cli` (test lives in `quire-cli` `tests/cli_coverage.rs`; not verifiable from this repo — CR-058) |
| TC-812 | `parse_header` returns `None` for frontmatter-less/unterminated/non-mapping input without entering the body pipeline, and reads `id` (empty when absent), `type`, `uuid` and the full frontmatter map for a document (CR-046) | Unit | P0 | FR-005-AC-5 | ✅ |
| TC-813 | `parse_body` under a `parse_header` header equals `parse_document` on BOM/CRLF/empty-body/no-heading fixtures and on arbitrary UTF-8 by proptest — the tiers *compose*; both sides share one body pipeline, so this is not a pre-refactor reference (that is TC-821) (CR-046, rescoped by CR-052) | Property | P0 | FR-005-AC-6 | ✅ |
| TC-814 | With the corpus at `<scope>/spec`, a `document: spec/tests.md` target mints when `validate_bundle` gets document root and reference root separately, and un-mints (reference dangles) when the roots are conflated; the `exclude:` half likewise — a `spec/fixtures/**` glob excludes the fixture under the split roots and lapses under conflated ones, the fragile direction since a lapsed exclusion *adds* ids silently (CR-045, exclude half CR-056) | Unit | P0 | FR-049-AC-9 | ✅ |
| TC-815 | loom: two threads first-touch one document's lazy body via a modeled once-cell — the init runs exactly once and both racers observe the identical value under every interleaving (std `OnceLock` modeled by contract; loom cannot instrument std — raced for real by TC-816) (CR-047) | Property | P0 | FR-025-AC-8, NFR-017-AC-4 | ✅ |
| TC-816 | The real std once-cell raced for real (TSAN lane target): two OS threads first-touch the same document — equal parsed bodies, repeated access pointer-identical, zero parsed before the touch and only the touched one after — plus 8 threads × 16 documents at staggered offsets agreeing on every body, and the rayon-forcing shape `python::load_repo` runs landing on the same bodies as a sequential force (CR-047, widened CR-053) | Integration | P0 | FR-025-AC-8 | ✅ |
| TC-817 | `len`/`by_id`/`by_type`/`diagnostics` plus `edges`/`outgoing`/`referencing`/`dangling`/`orphans` over a resolved corpus leave every document's body unparsed; touching one body parses exactly that document (CR-047) | Unit | P0 | FR-025-AC-7 | ✅ |
| TC-818 | Coverage over a corpus holding declared-archetype and undeclared-archetype documents parses the declared bodies and leaves the undeclared unmaterialised, with selection decided on the header tier and the report minting as before — including an undeclared type in a file *named* like a declared one, which is what makes the fixture able to falsify filename-driven selection (CR-049, fixture corrected CR-054) | Integration | P0 | FR-050-AC-18 | ✅ |
| TC-819 | `parse_body` is total in its header: a header parsed from one string, applied to another that is shorter than the body offset or whose multi-byte character straddles it, returns a document whose `raw` is the string it was given — named fixtures plus arbitrary UTF-8 pairs by proptest (CR-050) | Property | P0 | FR-005-AC-7 | ✅ |
| TC-820 | `validate_bundle` bridges a frontmatter-less file and a malformed-frontmatter file under the document root into exactly one `BundleReport` warning each, naming the path, in both postures — never an error, `is_valid()` unmoved — with distinct machine reasons `no-frontmatter` and `malformed-frontmatter` (CR-051) | Unit | P0 | FR-024-AC-12 | ✅ |
| TC-822 | A declared model that selects nothing reports why: a failed declared `document:` named per affected declaration with path and OS error, a declared `archetype:` no document has when the model minted nothing, and a model with no `trace_targets`; a healthy model reports none and omits the key; excluding every match of an archetype is not reported as a missing one; and the auxiliary document is read once however many declarations name it (CR-054, amended CR-059) | Integration | P0 | FR-050-AC-19 | ✅ |
| TC-824 | The coverage report over the checked-in baseline corpus is byte-identical to `tests/fixtures/coverage_baseline/expected.json`, regenerated only by `make coverage-baseline-update`; a companion case fails if that corpus stops exercising unbacked rows, status lies, the no-symbol exemption, untracked symbols, groups, criteria, the `exclude:` glob or lazy-body selection (CR-057) | Integration | P0 | FR-050-AC-7, FR-050-AC-20 | ✅ |
| TC-827 | The Rust adapter classifies benchmarks and fuzz targets: a `criterion_group!` registration promotes the top-level functions it names (short and `targets =` forms, wrapped or not) while a nested namesake and an unregistered function stay ordinary, `#[bench]` classifies directly, and a `fuzz_target!` invocation — which declares no `fn`, so nothing was extracted at all before — mints one symbol spanning its whole file from line 1 (CR-061) | Unit | P0 | FR-051-AC-17 | ✅ |
| TC-828 | A benchmark and a fuzz target bind trace ids while a container and a plain production function bind none — prose in production code citing an AC is not evidence — and this repository's own `bench_validate_document` and `fuzz_validate_extract_query` tags bind as authored, which catches a reverted `/`-separated id pair and a tag moved into a `//!` header (CR-061) | Integration | P0 | FR-051-AC-17 | ✅ |
| TC-826 | A model-level `traceability.exclude` scopes the CR-028 criteria walk as well as every declaration: an excluded document contributes no `criteria` entry, moves neither total, mints no ids, has its references never read and its body never parsed, while the same corpus under a model declaring no exclusion counts all of it; the patterns are compile-checked, merge across modules as a union, and declaring only `exclude:` leaves the model undeclared (CR-060) | Integration | P0 | FR-050-AC-13, FR-050-AC-15 | ✅ |
| TC-821 | `parse_document` over the checked-in golden corpus (frontmatter shapes, nesting, fences, BOM/CRLF, block ids, unicode, whitespace edges, no-frontmatter) serializes byte-identically to `tests/fixtures/parser_golden/expected.json`, captured from the engine at `7b1db82` — before the CR-046 tier split — and the two-tier path serializes to the same bytes (CR-052) | Integration | P0 | FR-005-AC-8 | ✅ |
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
| TC-880 | The `ix://` grammar accepts every shape the ecosystem authors — `org/repo/ID` (5,080 uses), `org/repo/spec/class/ID` (540), `org/repo` (225), `org/repo/spec/class/subdir/ID` (107), an object-slug target (`master-requirements`, `spire-partition-object-header`), `aggregate_root/User`, `ix://npm/…`, a `#fragment`, and both `[t](ix://…)` and `<ix://…>` wrappers. Occurrence counts are carried in the test so a future tightening argues with real usage | Unit | P0 | FR-026-AC-12, FR-026-CON-1 | ✅ |
| TC-881 | The grammar rejects the bare protocol `` `ix://` `` (158 uses — the #89 defect, whose target was the closing backtick), a single-segment URI, the `<org>`/`<ID>` and `{code}` doc templates, an `ix://([^)]+)` regex in prose, and a URI truncated by an elided segment (`ix://org/repo/...` mints nothing, not an edge to `repo`) | Unit | P0 | FR-026-AC-13 | ✅ |
| TC-882 | `harvest_edges` (the single-document surface behind the Python binding) reads the same grammar as `resolve`: the bare protocol is dropped and a backticked well-formed URI is kept | Unit | P0 | FR-026-CON-1 | ✅ |
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
| TC-641b | When `object:` names an archetype the registry cannot resolve, the edge vocabulary falls back to the artifact axis alone and Tier-1 validation still runs (`validate_document::tests::tc641_unknown_object_*`) | Unit | P0 | FR-040-AC-8 | ✅ |
| TC-642 | A corpus edge whose target document's `object:` archetype/roles fail the verb's target list yields a warning `DisallowedEdgeTarget`; same verb to a target carrying the required role passes (cross-module); skipped for `"*"`, no-`object:` targets, and dangling/cross-repo targets | Integration | P0 | FR-040-AC-9 | 🚧 |
| TC-643 | Tier-1/Tier-2 findings are warnings only — they do not block extraction or FR-032 structural validation, and a corpus with disallowed edges still loads | Integration | P0 | FR-040-AC-10, FR-032 | 🚧 |
| TC-644 | Tier-1/Tier-2 diagnostics sorted by `(source, target, edge_type)`; identical across repeated runs and thread counts | Property | P0 | FR-040-AC-10, NFR-006 | 🚧 |
| TC-645 | `input_skeleton` with an optional `object` arg renders a Relationships block listing each resolved verb with category/description/targets; without `object`, only the artifact vocabulary is listed | Unit | P0 | FR-040-AC-11, FR-029 | 🚧 |
| TC-652 | The merged `Registry` exposes an inverse index mapping each declared `inverse:` label to its forward verb; a registry with no declared inverses exposes an empty index | Unit | P0 | FR-041-AC-1 | 🚧 |
| TC-652b | An inverse label used as an `allowed_links` key in a manifest is a valid verb and does not raise `UnknownEdgeType` at load (`inverse_edges::tc652_inverse_label_*`) | Unit | P1 | FR-041-AC-1 | ✅ |
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
| TC-823 | The glossary heading pre-filter matches what the section lookup matches — ISO-numbered headings and block-id-carrying headings harvest, prose mentioning the words does not — and a glossary-bearing file with no frontmatter block is reported as `DocumentWithoutFrontmatter` instead of silently skipped, while an ordinary frontmatter-less README stays quiet (CR-055) | Unit | P1 | FR-044-AC-8 | ✅ |
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
| TC-767 | Python and WASM bindings return equivalent JSON over every canonical graph fixture | Integration | P0 | FR-046-AC-1, NFR-020-AC-3 | 🚧 external (evidence lives in the binding consumers; apparent matches in `filament-ide-rs` are that repo's own TC id space — CR-058) |
| TC-768 | parser-lib shim returns core-data-valid payloads matching canonical graph fixture expectations (filament-parser-lib FR-118 compatibility reference) | Integration | P0 | FR-045 (downstream compatibility reference) | 🚧 external (`filament-parser-lib` FR-118; not verifiable from this repo — CR-058) |
| TC-769 | Filament IDE worker merges a real quire-wasm canonical graph fixture into `CoreSyncFilePayload` (Filament IDE FR-046 reference) | Integration | P0 | FR-046 (downstream consumer reference) | 🚧 external (Filament IDE FR-046; not verifiable from this repo — CR-058) |
| TC-502 | `scripts/audits/check_no_shared_mutable.sh` — the enforcement identity of FR-024-AC-9's Inspection: no Mutex/RwLock/Atomic/OnceLock/OnceCell/LazyLock/Cell/RefCell/`thread_local!`/`static mut`/`unsafe impl Sync` in `src/corpus` or `src/python` outside a named exemption carrying a repo-relative path, the exact source line and a printed `why`; an exemption matching nothing fails as stale. Runs in `make audit-static`, `make ci` and the ci.yml `audit-static` job (CR-053) | Static | P0 | FR-024-AC-9 | 🚧 enforced but unbackable: `language_of` reads `.rs`/`.py`/`.ts` only, so a `.sh` audit is never opened by the extractor and mints no source symbol. CR-061 widened `trace::bind` to benchmarks and fuzz targets, which does **not** reach this — the blocker is the absent shell language, filed separately (CR-058, amended CR-061) |
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
| TC-526b | `variants` in a manifest is a hard `ArchetypeLoadFailure` — the no-compat rule, no tolerated legacy field (`loader::tests::tc526_variants_*`) | Unit | P1 | FR-031-AC-5 | ✅ |
| TC-526c | `template_ref` in a manifest is a hard `ArchetypeLoadFailure`; render was removed with no backward-compatibility layer (`loader::tests::tc526_template_ref_*`) | Unit | P1 | FR-031-AC-5 | ✅ |
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
| TC-577 | Bench: `bench_validate_document` on a typical FR-sized artifact median <1ms (warm registry); >10% vs baseline fails CI. Backed since CR-061: a `criterion_group!`-registered function classifies as a benchmark symbol and binds its tag | Benchmark | P0 | NFR-002-AC-4 | ✅ |
| TC-578 | Determinism: `validate_document` + `extract` on the same input 100× across threads → equal `ValidationResult` (ordered diagnostics) + `ExtractionResult` (records+edges+diagnostics) | Property | P0 | NFR-006-AC-4 | ✅ |
| TC-579 | Fuzz: arbitrary byte slices (lossy `&str`) into `parse_document`/`validate_document`/`extract` run clean (no panic/UB) for the scheduled duration; crashes committed as regression reproducers. Backed since CR-061: a `fuzz_target!` invocation mints a symbol whose span is its whole file. Its `Type` was `Integration`, which was wrong for the row and is corrected here | Fuzz | P0 | NFR-019-AC-1 | ✅ |
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
| TC-775 | The positive/negative pair idiom is recognized by the second obligation, not the separator: `and SHALL NOT`, `but … SHALL be None`, `. No … SHALL occur.` and `; otherwise … SHALL be omitted` each yield no `non-singular`, while two positive obligations, a `not` in the condition, and a three-obligation criterion each still yield one | Unit | P0 | FR-047-AC-15 | ✅ |
| TC-776 | A precedence chain using `then` with no modal yields no `non-singular`; a Given/When/Then cell with two `Then` consequents still yields one | Unit | P0 | FR-047-AC-16 | ✅ |
| TC-777 | `functions` as a noun (`the code functions that implement it`) yields no `vacuous-outcome`; the qualified predicate (`functions independently of exporters`) still yields one | Unit | P0 | FR-047-AC-17 | ✅ |
| TC-778 | A modal quoted inside a double-backtick span is masked, so the criterion classifies `assertion` and yields no `non-singular` or `non-canonical-shape`; an unbalanced run opens no span and a following unquoted modal still counts | Unit | P0 | FR-047-AC-18 | ✅ |
| TC-794 | Dogfood gate: every document in this repo's own `spec/` is free of findings for the checks `spec-artifacts-iso` promotes to `error`, judged on the engine's default vocabularies; a reachable module checkout must agree with the mirrored promotion | Integration | P0 | FR-048-AC-11 | ✅ |
| TC-779 | The issue's own cell (`A finding whose key is absent from the merged map defaults to warning`) classifies `Universal` with all three of `domain`, `precondition` and `oracle` populated | Unit | P0 | FR-052-AC-1 | ✅ |
| TC-780 | Round-trip, idempotence, ordering and invariant idioms each label to their own shape, and a universally quantified round-trip labels `RoundTrip` rather than `Universal` (fixed precedence) | Unit | P0 | FR-052-AC-2 | ✅ |
| TC-781 | CR-017 parity for the property classifier: an idiom phrase inside an inline code span is a mention and fires no signal, while the same phrase unquoted fires it | Unit | P0 | FR-052-AC-3 | ✅ |
| TC-782 | A criterion whose oracle boundary is supported only by the weak inflected-verb marker classifies `Universal` with `spans: None` rather than emitting a guessed decomposition | Unit | P1 | FR-052-AC-4 | ✅ |
| TC-783 | A specific-scenario criterion classifies `Example` with `extractable == false` and contributes zero findings to any `ac` check | Unit | P0 | FR-052-AC-5 | ✅ |
| TC-784 | For every emitted span: `statement[start..end] == text`; spans of one record are in bounds, non-overlapping and ascending by start offset | Property | P0 | FR-052-AC-6 | ✅ |
| TC-785 | Negative space (CON-1): `ac::check()` output over a fixture corpus is unchanged finding-for-finding, field-for-field and order-for-order by the classifier's presence | Unit | P0 | FR-052-AC-7 | ✅ |
| TC-786 | A module `property_idioms` registry merges first-wins over the engine built-ins; an absent declaration uses the built-ins alone (mirrors TC-757) | Unit | P1 | FR-052-AC-8 | ✅ |
| TC-787 | Binding parity: property classification sees exactly the cells `ac::check` sees — FR/NFR `Acceptance Criteria`, StR `Validation Criteria`, and supplements — while US and IT documents yield no records and no finding (CR-020) | Unit | P0 | FR-052-AC-9 | ✅ |
| TC-788 | `CoverageReport.criteria` entries and the two new `CoverageTotals` counts appear for a criteria-bearing corpus, a no-criteria corpus yields an empty list, and two runs serialize byte-identically (FR-050-AC-7) | Integration | P0 | FR-052-AC-10, FR-050-AC-13 | ✅ |
| TC-789 | The PyO3 `classify_properties` binding returns the same records, field for field, as the in-process Rust call over the same fixture (mirrors TC-715) | Unit | P0 | FR-052-AC-11 | ✅ |
| TC-790 | CON-4: `extractable` is identical over a fixture corpus with the idiom registry present and absent; only `property` labels may differ, so a missed idiom degrades a label and never coverage | Property | P0 | FR-052-AC-12 | ✅ |
| TC-791 | Closed structural signals fire with no registry declared: composition plus an identity back-reference → `RoundTrip`; a repetition adverb plus an equality verb → `Idempotence`; a bare `deterministic`, `before` or `order` → **not** `Ordering` | Unit | P0 | FR-052-AC-13 | ✅ |
| TC-792 | The universal determiner is read at a bounded subject position as well as at offset 0: a fronted subordinate clause's subject and a determiner-headed main subject after the comma that closes fronted material both classify `Universal` and extractable under their own signals, while a definite determiner, an unbounded fronted phrase and a determiner standing in the outcome are each refused (CR-030) | Unit | P0 | FR-052-AC-14 | ✅ |
| TC-793 | Byte-identity of everything the widened positions do not claim: `ac::check`'s finding stream over a fixture corpus is unchanged in all three archetypes, and every declined criterion keeps its exact prior `property`, `extractable` and `signals` fingerprint (CR-030) | Unit | P0 | FR-052-AC-15 | ✅ |
| TC-795 | The three-valued `extraction` outcome: `extractable: true` → `extractable`; `extractable: false` with a metamorphic `property` → `candidate`; an `Example` or `Unclassified` criterion → `not-extractable` and never `candidate` (CR-033) | Unit | P0 | FR-052-AC-16 | ✅ |
| TC-796 | `extraction` is derived, not fed back: over a fixture corpus every criterion's `extractable` is identical with a `property_idioms` registry declared and with none (CON-4 unchanged), while a criterion whose metamorphic label comes only from a declared idiom reports `candidate` with the registry and `not-extractable` without it (CR-033) | Unit | P0 | FR-052-AC-17 | ✅ |
| TC-798 | A `/*` inside a template literal (a git refspec) is content, not a comment opener: the file still parses, its test symbol's span still reaches back over its JSDoc, and a real `//` comment still strips (CR-036) | Unit | P0 | FR-051-AC-12 | ✅ |
| TC-806 | A legacy form whose match carries a comma-separated list binds every id, not only the first — irregular spacing and a trailing comma yield the ids and never an empty one, no dedup diagnostic fires, one authored line yields one rewrite suggestion naming all its ids, and an `id_format` form still renders a single id (CR-043) | Unit | P0 | FR-051-AC-16 | ✅ |
| TC-804 | A brace inside a raw string (`r#"…"#`), a lifetime (`&'a str`), a character literal or a nested block comment never moves the depth and never rejects the file — the shape that made 33 of this repo's own source files yield zero symbols (CR-040) | Unit | P0 | FR-051-AC-15 | ✅ |
| TC-803 | One lexer pass per file serves every consumer: a brace inside a block comment, inside a carried template literal, and after an unterminated quote each count as zero, the file balances on the real block alone, and the span cut from those deltas is the declaration's own (CR-039) | Unit | P0 | FR-051-AC-14 | ✅ |
| TC-800 | A `def` whose signature a formatter wrapped across lines still spans its docstring: two tests differing only in wrapping bind identically, and a paren inside a string default or a trailing comment is not counted (CR-037) | Unit | P0 | FR-051-AC-13 | ✅ |
| TC-799 | The same `/*` on a **continuation** line of a multi-line template literal is still content: the file parses, its test symbol's span reaches back over its JSDoc, and the template state opens, carries and closes across the three lines (CR-036) | Unit | P0 | FR-051-AC-12 | ✅ |
| TC-805 | A row whose declared `Type` names a method that mints no source symbol is a no-symbol row and not a status lie, stays listed as unbacked, and — with the vocabulary undeclared — is an ordinary lie again with the key absent from the JSON (CR-041) | Unit | P0 | FR-050-AC-16 | ✅ |
| TC-801 | A declaration excluding `fixtures/**` mints no ids and contributes no reference rows from a matching document; the identical corpus with the exclusion removed admits the fixture matrix and reads its reused id as backed (CR-038) | Unit | P0 | FR-050-AC-15 | ✅ |
| TC-829 | `archetype:` is the single required origin: a target declaring it alone is valid, a `document:` key is rejected by `deny_unknown_fields` so a stale module fails load rather than minting nothing, an empty `archetype` is a validation error, and a target with no origin fails to deserialize (CR-062) | Unit | P0 | FR-050-AC-15 | ✅ |
| TC-830 | One archetype-bound entry mints from **every** matrix in the corpus whatever each is called — two differently-named matrices reconcile against the same target kind from a single declaration, where the retired `document:` form needed one entry per filename and still reached nothing nested (CR-062) | Integration | P0 | FR-050-AC-15 | ✅ |
| TC-831 | A `target:`-bound obligation source mints one record per row of the named trace target's table, each keyed on the id the coverage rollup already mints, carrying statement, method and criticality (FR-053) | Integration | P0 | FR-053-AC-1 | ✅ |
| TC-832 | An `archetype:`+`section:`+`id_format:` source covers rows with no id column — the NFR `Measurement and Evaluation` table — rendering `{document}` and the 1-based `{row}`, and carrying the metric's target and threshold as parameters (FR-053) | Integration | P0 | FR-053-AC-2 | ✅ |
| TC-833 | A source declaring both `target:` and `archetype:`, or neither, is rejected at manifest parse: the module contributes nothing, the failure names the offending source, and no obligation reaches a consumer (FR-053) | Integration | P0 | FR-053-AC-3 | ✅ |
| TC-834 | Whitespace and line-wrapping do not churn the statement hash; any word change does — **including a word inside an inline code span**, which the CR-017 mask would have collapsed (FR-053) | Unit | P0 | FR-053-AC-4 | ✅ |
| TC-835 | One `Verification` cell, two readings: the obligation's method drops the trailing annotation (`Test (TC-707)` → `Test`) while FR-049's integrity check still reads `TC-707` out of the same cell (FR-053) | Integration | P0 | FR-053-AC-5 | ✅ |
| TC-836 | A declared parameter whose cell is empty is **omitted** from the record rather than carried as an empty string — a threshold nobody wrote is not a threshold of zero (FR-053) | Integration | P1 | FR-053-AC-6 | ✅ |
| TC-837 | A source declaring no `criticality_column`, and one declaring an empty column, both yield an absent criticality and are otherwise identical — criticality is genuinely optional, since the ISO AC contract carries no priority column (FR-053) | Integration | P1 | FR-053-AC-7 | ✅ |
| TC-838 | A row whose statement cell is empty is skipped and reported with its document and row ordinal, never emitted as a record stating nothing (FR-053) | Integration | P1 | FR-053-AC-8 | ✅ |
| TC-839 | Two derivations over an identical corpus are equal and serialize identically, ordered by source, then document, then authored row order (FR-053) | Integration | P0 | FR-053-AC-9 | ✅ |
| TC-840 | A criterion carries its obligation on the property-classification record, matched by row id; with no source declared it carries `None` and every other field is unchanged (FR-053) | Integration | P0 | FR-053-AC-10 | ✅ |
| TC-841 | The coverage report carries one obligation per minting row, and a model declaring none serializes the key **away entirely** rather than as an empty list, so FR-050-AC-7 byte-identity holds for every module that has not adopted them (FR-053) | Integration | P0 | FR-053-AC-11 | ✅ |
| TC-842 | The hash follows the statement, not its position: the same statement in a different file, under a different id, hashes the same, while a one-word rewording does not (FR-053) | Integration | P0 | FR-053-AC-12 | ✅ |
| TC-843 | The obligation nested on a classification record carries no `id`, `statement` or `document` — the record and its enclosing object already carry all three (FR-053) | Integration | P1 | FR-053-AC-13 | ✅ |
| TC-870 | The skipped-row diagnostic reaches the **coverage report** — declaration, document, row ordinal, and the `obligation-row-states-nothing` reason in the JSON — not only `derive`'s return value (FR-053, CR-063) | Integration | P1 | FR-053-AC-8 | ✅ |
| TC-871 | A composed and a decomposed accent hash identically: normalization is NFC before trim-and-collapse, so an editor rewriting NFD is not a suspect link (FR-053, CR-063) | Unit | P0 | FR-053-AC-4 | ✅ |
| TC-872 | Record order follows source **declaration** order, proven with a module whose two sources are declared in reverse alphabetical order — the only arrangement where declaration order and name order disagree (FR-053, CR-063) | Integration | P0 | FR-053-AC-9 | ✅ |
| TC-873 | An `exclude`d document states no obligation on **either** surface — absent from the coverage rollup and `None` on its classification record — so a generator cannot emit a trace tag for an id nothing mints (FR-053, CR-063) | Integration | P0 | FR-053-AC-14 | ✅ |
| TC-844 | Every declared catalog field — name, class, definition, evidence kind, applicability rules, tooling — survives to `Registry::verification_catalog()` intact (FR-054) | Integration | P0 | FR-054-AC-1 | ✅ |
| TC-845 | Two modules declaring the same method id merge first-wins, the later body is skipped, and one `DuplicateVerificationMethod` names both modules — two modules disagreeing about what a method means is not something to absorb in silence (FR-054) | Integration | P0 | FR-054-AC-2 | ✅ |
| TC-846 | A module declaring no catalog yields `None`, so a consumer reports it **undeclared** rather than as containing no methods — the FR-050-AC-2 distinction applied to this registry (FR-054) | Integration | P0 | FR-054-AC-3 | ✅ |
| TC-847 | An entry carrying an unknown key fails module load naming the key, so a typo cannot be silently discarded — the `deny_unknown_fields` rule that makes engine-before-module ordering load-bearing (FR-054) | Integration | P0 | FR-054-AC-4 | ✅ |
| TC-848 | An entry whose name, class or definition is empty or whitespace-only fails module load naming the method id: an entry that cannot say what it is advises nothing (FR-054) | Integration | P1 | FR-054-AC-5 | ✅ |
| TC-849 | `verification_method` returns exactly the merged catalog keys sorted, and `verification_class` each distinct class once — both tracking the merge, so a body that lost first-wins contributes no class (FR-054) | Integration | P0 | FR-054-AC-6 | ✅ |
| TC-850 | `test_type` is unchanged, an unknown vocabulary name returns empty rather than a default, and a module with a catalog but no traceability vocabulary answers the derived names only (FR-054) | Integration | P1 | FR-054-AC-7 | ✅ |
| TC-851 | An applicability rule name the engine has never heard of survives verbatim, and an entry declaring none carries an empty map rather than a default — the engine stores and surfaces, never interprets (FR-054, CON-2) | Integration | P0 | FR-054-AC-8 | ✅ |
| TC-852 | Declaring a catalog contributes no error and leaks into no finding message: the catalog is data, never a verdict (FR-054, CON-3) | Integration | P0 | FR-054-AC-9 | ✅ |
| TC-853 | The derived vocabularies need no separate declaration and move when the catalog moves — a second authored copy is the duplication this FR removes (FR-054, CON-4) | Integration | P0 | FR-054-AC-10 | ✅ |
| TC-874 | A `Verification` cell naming neither a catalog method id nor a catalog class is reported as `uncatalogued-verification-method` — once per distinct (source, method) pair, with the row count and an example document, so 14 identical rows are one decision rather than 14 diagnostics (FR-054) | Integration | P0 | FR-054-AC-11 | ✅ |
| TC-875 | A corpus whose modules declare no catalog is reported nothing: an absent catalog cannot answer the question, and silence here is what keeps FR-050-AC-7 byte-identity for every module that has not adopted one (FR-054) | Integration | P1 | FR-054-AC-11 | ✅ |
| TC-854 | Both published schemas are valid draft 2020-12 documents and self-identify by their versioned filename in `$id` (FR-055) | Integration | P0 | FR-055-AC-1 | ✅ |
| TC-855 | The CR-057 byte-golden baseline validates against `coverage-v1.schema.json` with zero errors — one corpus, two gates, reviewed in one diff (FR-055) | Integration | P0 | FR-055-AC-2 | ✅ |
| TC-856 | A payload carrying **every** optional key — no_symbol_rows, criteria, diagnostics (with `value`), obligations, vocabulary_coverage, both totals pairs — validates, so the optional keys are covered by a payload that has them rather than only by one that omits them (FR-055) | Integration | P0 | FR-055-AC-3 | ✅ |
| TC-857 | Every criterion record the engine emits for a fixture document validates against the `Criterion` definition, obligation included (FR-055) | Integration | P0 | FR-055-AC-4 | ✅ |
| TC-858 | An added field is rejected at the root, inside a nested object, and inside an array item — `additionalProperties: false` holds at depth, the usual place a hand-authored schema is accidentally open (FR-055) | Integration | P0 | FR-055-AC-5 | ✅ |
| TC-859 | Removing an optional key leaves a payload valid and removing a required one does not, in both directions, so the split matches the engine's skip-when-empty behaviour rather than being asserted (FR-055) | Integration | P0 | FR-055-AC-6 | ✅ |
| TC-860 | No payload carries a `version`, `$schema` or `schema_version` key, and `schemars` is absent from the lockfile — the contract is carried by the published artifact alone (FR-055, CON-1/CON-2) | Integration | P0 | FR-055-AC-7 | ✅ |
| TC-1010 | Both payloads accept an optional `engine` provenance object and reject one missing `cli`, `engine` or `capabilities` or carrying an undeclared member; a `-<n>-g<sha>` engine string survives verbatim and an unrecognised capability token is accepted, so the envelope is closed while its vocabulary stays open (CR-104, agent-ix/quire-cli#68) | Integration | P0 | FR-055-AC-8 | ✅ |
| TC-1011 | A non-mutating case is read from disk in place with nothing generated or copied, and a mutating case operates on a copy leaving its checked-in `input/` byte-unchanged — the property that distinguishes a corpus from a generator (FR-065) | Integration | P0 | FR-065-AC-1, FR-065-AC-2 | ✅ |
| TC-1012 | A case omitting any required `case.yaml` field is rejected naming the case and the field, and a `control` declaring no `control_for` is rejected — a fixture whose origin or pairing is unrecorded becomes one nobody dares change (FR-065) | Unit | P0 | FR-065-AC-3, FR-065-AC-4 | ✅ |
| TC-1013 | An omitted `expect` field is asserted on by nothing rather than defaulted, so a case pins only what it is about and one unrelated change cannot fail forty cases (FR-065) | Unit | P0 | FR-065-AC-5 | 🚧 awaiting #268 (the loader rejection paths these assert) |
| TC-1014 | Every declared cell reads as exactly one of `covered`/`out-of-scope`/`GAP`; an `out-of-scope` cell with an empty reason is rejected, and a cell in no state is rejected naming the case and language (FR-065) | Unit | P0 | FR-065-AC-6, FR-065-AC-7, FR-065-AC-8 | 🚧 awaiting #268 (the loader rejection paths these assert) |
| TC-1015 | `bounds.gap_count` renders as an integer count on every payload this crate emits and as a ratio on none — FR-063-AC-6 applied to this metric, since a ratio falls as easy cases are added and hides the hard missing one (FR-065) | Integration | P0 | FR-065-AC-9, FR-065-AC-10 | 🚧 awaiting #268 (the loader rejection paths these assert) |
| TC-1016 | A failing case reports the highest detection level reached and the first level lost, distinguishing L1/L2/L3 — `the case failed` and `the message stopped naming the row` are different repairs (FR-065) | Integration | P0 | FR-065-AC-11, FR-065-AC-12 | ✅ |
| TC-1017 | A failure case whose `control_for` partner is absent is rejected naming the missing control, and a present control produces no finding for the mode its partner asserts — #250 scored perfect recall firing 549 times on 551 candidates (FR-065) | Integration | P0 | FR-065-AC-13, FR-065-AC-14 | ✅ |
| TC-1018 | A variant-bound case declares exactly one classification: a temporary `relaxation_ticket` or a reason-bearing `declaration_under_test`; an ecosystem-bound case declares neither (#330) | Unit | P0 | FR-065-AC-15, FR-065-AC-16 | ✅ |
| TC-1019 | Two runs of one case over unchanged input produce a byte-identical engine report — `CoverageReport::to_json()` compared to itself across two `run()` calls, which is what the test has always asserted (CR-124 narrowed the criterion to it) (FR-065) | Integration | P0 | FR-065-AC-17 | ✅ |
| TC-1020 | Each case carries the invocation that reproduces it and that invocation names a module — without `--module` no model loads, the run reports 0/0, and the case cannot exhibit the declaration defect it exists for (FR-065) | Integration | P0 | FR-065-AC-18 | ✅ |
| TC-1021 | The runner reads the bounds enum and the mode families from `corpus.yaml` rather than compiled-in lists, and a case naming an undeclared family is rejected. The grading LADDER is held to a different claim: `Level::ALL` rendered through `Level::token` must equal `grading_levels` in name and in ORDER, because the ladder is the grading rule rather than a vocabulary and no runner ever accepted a fourth level without a code edit. This row said it did. The assertion was also a literal `["L1","L2","L3"]` written in the test — a third copy of the ladder, compared to the declaration while the enum that grades was compared to neither (CR-129, #337). Mutation-verified both ways: renaming `L3` to `L4` and reordering `L1`/`L2` each fail it by name | Integration | P0 | FR-065-AC-19, FR-065-AC-20, FR-065-AC-21 | ✅ |
| TC-1022 | A language set's variant declares only what varies: DECLARING `case`, `mode`, `module`, `kind` or `pending` is rejected naming the field whether or not the shared file declares it too, and every reader derives one id `<shared id>-<language>` — measured, a one-line override moved a covered cell to a different inventory row while `gap_count` stayed put, and the two readers disagreed about a variant's identity (CR-109) | Integration | P0 | FR-065-AC-22, FR-065-AC-23 | ✅ |
| TC-1023 | A case's `expect.yaml` is graded and must hold whether or not it is pending, its `expect-pending.yaml` must not hold yet, and either file without the other is rejected — measured, `pending:` excusing the whole block let both minting fixtures regress to minting nothing in three languages and stay green (CR-110) | Integration | P0 | FR-065-AC-25, FR-065-AC-26, FR-065-AC-27 | ✅ |
| TC-1024 | `unbacked_rows` and `groups` are exact in both directions — an empty list is an assertion — and every substring in an L3 list is asserted, not just the first (CR-110) | Integration | P0 | FR-065-AC-28, FR-065-AC-29 | ✅ |
| TC-1025 | The LOADER refuses a block that asserts the wrong thing, proven by mutating a copy of the corpus: an undeclared token, a live block requiring what no engine emits, a failure case asserting its own pending token absent, a forward block that asserts nothing, one that is merely FALSE rather than about its ticket, and one requiring an already-emitted token (CR-111, CR-112) | Integration | P0 | FR-065-AC-30, FR-065-AC-31, FR-065-AC-32, FR-065-AC-33, FR-065-AC-36, FR-065-AC-39 | ✅ |
| TC-1026 | `corpus.yaml`'s `emitted` and `forward` lists are checked against the engine, and every emitted reason is required by a failure case and forbidden by a healthy control (#300, #359) | Integration | P0 | FR-065-AC-34, FR-065-AC-35, FR-065-AC-48 | ✅ |
| TC-1027 | A control binds its partner's `mode` and `module`, every failure case is named by some control, and `known_gaps` is enforced in both directions — the block listed three uncontrolled cases where eleven were true, and was read by no code (CR-112) | Integration | P0 | FR-065-AC-37, FR-065-AC-38, FR-065-AC-40, FR-065-AC-41 | ✅ |
| TC-1028 | A failure case's assertions SEPARATE its own input from its control's: graded against the control's payload, `expect.yaml` must produce at least one mismatch — a floor, not a proof: ten of eleven controlled fixtures are satisfiable by one incidental scalar. **That denominator is as of corpus `776a6b3` and has not been re-audited since** — controlled failure cases were 11 there, 33 at `db55b05` and **34** at `3ff72c0`, so "ten of" describes eleven fixtures out of thirty-four (CR-123); re-running the audit is #301's. This row read **35** and cited `bounds.py --json`, which emits no such count: 35 was the Rust harness's number, inflated by `marker-mismatch` — a DECLARED uncontrolled gap it reached through another case's `case:` alias — and cited to the other reader's tool (CR-128). Both readers now return 34 cases and grade **35 (case, control) pairs**, two controls naming `marker-form-mismatch`. AC-42 also existed in ONE reader until #337: `verify.py` accepted a fixture blinded to `total: 4` at `mismatches: 0` while this test rejected the same corpus. Every other rule in FR-065 is a predicate on the shape of a declaration, and five review rounds each removed one shape that was non-empty and meaningless before the next found another; this is a predicate on meaning (CR-114). **AND ON THE RIGHT CHANNEL, since CR-130**: the block is graded a second time RESTRICTED to the `witness_channels` its mode declares, and the restriction must still mismatch — `total` is a witness for `minting` and for nothing else, so the incidental scalar **14 of the 35 (case, control) pairs** differ in — equivalently 14 of the 34 controlled cases, measured by running every case and every control and comparing `totals.total` over the whole controlled population — no longer satisfies the rule anywhere else. A mode declaring a channel `CaseExpect` cannot restrict on is rejected rather than silently dropped — **in both readers since CR-132**, which found AC-47 implemented in this one alone and `qa-corpus`'s whole `make ci` green on a corpus where a one-character typo had weakened AC-46 for the `disposition` mode. The cross-reader assertion at the end of this test compares the sorted **(case, language, control) list** rather than the pair COUNT, also CR-132: both the pre-#337 and the post-#337 resolutions yield 35 pairs while the SETS differ by one each way, so the count could not see the regression it was written for. Mutation-verified in both branches and in both readers: a block asserting only `backed` names no `disposition` channel; the same block plus an `absent_diagnostic_reasons` true of the control names one that does not discriminate | Integration | P0 | FR-065-AC-42, FR-065-AC-46, FR-065-AC-47 | ✅ |
| TC-1032 | A `regression` case is accepted without a control; ecosystem-bound pins credit their cell, relaxation variants remain ticketed GAPs, and declaration-under-test variants are explicit out-of-scope cells carrying the authored reason (#330) | Integration | P0 | FR-065-AC-43, FR-065-AC-44, FR-065-AC-45 | ✅ |
| TC-1029 | A Python triple-quoted string is tracked wherever it opens: an assigned opener (`FIXTURE = """`), the `'''` kind, the `r`/`f`/`rb` prefixes and a same-line open-and-close each leave no phantom declaration from the literal's body, and the real `def` after all of them is still seen — the closer closed rather than re-opened (CR-115, #274) | Unit | P0 | FR-051-AC-20 | ✅ |
| TC-1030 | A triple delimiter inside a single-quoted string, escaped, or after a `#` toggles nothing, and a `#` inside a triple-quoted literal does not end it; asserted both on the minted symbols and on the state machine directly, so a pass cannot be two errors cancelling (CR-115, #274) | Unit | P0 | FR-051-AC-20 | ✅ |
| TC-1031 | A declaration after an embedded string keeps its TRUE container: the class following the literal is seen at all, and its method is attributed to it rather than to the class that was open when the desync began — the dominant mode by count, and the half a fix that only stopped the swallow would miss (CR-115, #274) | Unit | P0 | FR-051-AC-20 | ✅ |
| TC-1033 | A declared `section:` the archetype-matching document does not have is reported per document under `section-matches-nothing`, naming the file in `path` and, in its message, the heading FOUND, the heading DECLARED and the `id_column` it could NOT check; the three ways a scan produces no rows are three distinct answers rather than one empty `Vec`; the same tree with the declared heading fires neither minting token and backs the row it stranded (CR-117, #270) | Unit | P0 | FR-050-AC-33 | ✅ |
| TC-1034 | A declared `id_column` absent from the found table is its OWN token, `id-column-matches-nothing`, naming the column FOUND and the column DECLARED — and the section token does not fire alongside it; the same tree with the heading wrong instead reports only the section token, while both report `backed: 0`, which is the measured argument for two tokens rather than one (CR-117, #270) | Integration | P0 | FR-050-AC-33 | ✅ |
| TC-1035 | Neither minting token is gated on whether another declaration minted: a bundle whose FR criteria mint normally still reports its stranded matrix, the section message names the id column it could not check, and fixing only the heading exposes the second fault that was there all along — the loop that sentence exists to shorten (CR-117, #270, #304) | Integration | P0 | FR-050-AC-33 | ✅ |
| TC-1036 | `minting.section_hit_rate` is a RATIO over the documents a trace target's archetype selected: both sections found is 2/2 and not hollow, one found is 1/2 and still not hollow, none found is hollow and reported by name under `hollow-denominator`, and a model declaring no `trace_targets` reports it not computed rather than zero (CR-117, #270) | Integration | P0 | FR-063-AC-7 | ✅ |
| TC-1037 | A declaration naming SEVERAL sections mints from every one of them, in document order, and its reference declaration reads the same rows — a status lie under a qualified heading is reported. The control is the SAME tree read by a declaration naming ONE section: it mints that one row and no other, and a row under a heading neither names stays in `untracked_symbols` (CR-118, #272) | Integration | P0 | FR-050-AC-34 | ✅ |
| TC-1038 | `section:` reads a scalar or a sequence on the ONE key, a one-name declaration round-trips back out as a scalar, an empty list or a blank entry fails module load, and a name carrying no `*` matches exactly what `query::section` has always matched — asserted over generated headings against the engine's own lookup, so prose punctuation (`[`, `{`, `?`) stays literal (CR-118, #272) | Unit | P0 | FR-050-AC-34 | ✅ |
| TC-1039 | A `describe(…)` / `suite(…)` registration mints a CONTAINER named by its title, spanning its block and the leading comment the tag sits in, and parents the registrations inside it WITHOUT renaming them — a nested suite, a sibling outside every suite and a class inside one all keep their true container. `context` is outside the grammar, and the lookalikes it would admit (`context.setTransform(`, `await context.client.putSettings(`) register nothing (CR-119, #273) | Unit | P0 | FR-051-AC-21 | ✅ |
| TC-1040 | A trace tag on a suite HEADER mints no `verifies` relation and the suite is not a `binding_census` candidate — it is counted as an `implements` candidate instead, which is the double-count #312 must settle. The tag on the `it` one line lower binds normally, so the pair separates "the suite is not evidence" from "the binder read nothing" (CR-119, #273, #312) | Unit | P0 | FR-051-AC-21 | ✅ |
| TC-1042 | A registration whose name chain names a suite anywhere along it is a suite: `test.describe(…)` classifies exactly as `describe(…)` does, and neither header tag binds. It used to matter — `TEST_NAMES` was tried first and the AC-18 modifier window swallowed `.describe`, so the Playwright spelling minted a test, bound its header tag and entered the census. 120 such headers across two corpus repos, 79 carrying an id in the title, would have bound as evidence the moment sap#68 declared a TypeScript test-name form (CR-121, #322) | Unit | P0 | FR-051-AC-21 | ✅ |
| TC-1041 | A document whose every matched section holds no table reports `section-holds-no-table`, and a document where only SOME matched section is table-less reports nothing — the first mints nothing while both sibling diagnostics stand down, the second is a parent heading with sub-headings and is ordinary. Landed because #272's widening made three repositories match a table-less heading and LOSE their `section-matches-nothing` while still minting zero, one of them reporting a perfect 33/33 `minting.section_hit_rate` (CR-120) | Integration | P0 | FR-050-AC-35 | ✅ |
| TC-1050 | Zero-padding, case and separator are one class and normalise onto one key: `TC-1`, `tc_001`, `tc001` and `Tc-0001` all equal `TC-001`, while `TC-001` and `TC-010` do not collide and `TC-000` is not `TC-`. Collapsing the three together is deliberate — each is a human writing the same id twice and getting a different string, and splitting them would report three names for one mistake (CR-136, #307) | Unit | P0 | FR-050-AC-37 | ✅ |
| TC-1051 | A near miss is reported naming BOTH spellings — "an id did not match" is useless when the whole defect is that `TC-1` and `TC-001` look identical until you count zeros — while an EXACT match is not reported, because that is a different defect, and an id matching no row is left to `untracked_symbols`. Mutation-verified: removing the exact-match guard fails it by name; removing the join or the padding normalisation takes the corpus fixture from L1 to nothing (CR-136, #307) | Unit | P0 | FR-050-AC-37 | ✅ |
| TC-1048 | A declared archetype no document has survives `into_diagnostics`: there is no model-wide gate left to drop it. It used to take `minted_anything: bool` and, when anything in the model had minted, discard every one — so a repository with no TestMatrix reported `groups` holding only `acceptance-criterion`, the whole `test-case` target missing from the payload rather than present and zero, and said nothing, because the criteria declaration had minted. A missing denominator is not a low score, it is NO score, and the two were indistinguishable on every surface (CR-135, #304) | Unit | P0 | FR-050-AC-36 | ✅ |
| TC-1049 | Only a TRACE TARGET reports a missing archetype; a reference declaration, whose section is legitimately optional, does not. Without the narrowing the rule fires on every repository for every archetype it lacks — measured over 245, ungated and unnarrowed: 741 findings, of which `inspection` and `suite` are 484, firing on 242 repositories each because almost nobody writes `suites.md` or `inspections.md`. That is a declaration-side fact (`agent-ix/spec-artifacts-process#75`), and the narrowing reuses the same `mints` distinction that keeps `section-matches-nothing` off healthy repositories rather than inventing a second rule (CR-135, #304) | Unit | P0 | FR-050-AC-36 | ✅ |
| TC-1055 | A legacy textual form inside a string literal is masked in EVERY language, not Rust alone: a trace id a file carries as DATA is not a tag, and binding it invents coverage nobody authored — measured on the corpus fixture pre-fix at `backed 2/3` with nothing unbacked (#323) | Unit | P0 | FR-051-AC-24 | ✅ |
| TC-1056 | Each language keeps its own declared tag channel: a Python DOCSTRING and a TypeScript registration TITLE survive the mask, because `python-docstring-id` and `typescript-test-name-id` read an id out of a string by design — and both exemptions are POSITIONAL, so the same text assigned mid-line is masked. Rust needs no exemption; its `rust-test-name-id` reads an identifier (#323) | Unit | P0 | FR-051-AC-24 | ✅ |
| TC-1057 | Comments survive the mask — that is where legacy tags live — and byte length and line count are preserved, because the rewrite-suggestion pass matches this span and reports positions into the original (#323) | Unit | P0 | FR-051-AC-24 | ✅ |
| TC-1058 | The corpus harness runs the representative `clean-control` case twice to byte-identical coverage JSON, grades it successfully, and renders the exact L3 outcome text; this characterizes the execution/grading/rendering boundary before #360 moves it into separate modules | Integration | P0 | FR-050-AC-7 | ✅ |
| TC-1052 | The symbol table reports the QUALIFIED NAME the engine built, with its container — the field three ports of `symbols/python.rs` disagreed on, giving 386, 490 and 5,263 lost declarations over one tree. A defect in the scanner cannot be sized by a reimplementation of the scanner (#309) | Unit | P0 | FR-051-AC-23 | ✅ |
| TC-1053 | Each record carries whether its KIND can bind a trace id and whether it can carry `implements`, and the two are complements for every symbol — the first thing to check when a row will not bind, previously only inferable from a coverage rollup two layers away (#309, #312, CR-061) | Unit | P0 | FR-051-AC-23 | ✅ |
| TC-1054 | With no module the report says binding was NOT ASKED rather than reporting zero: an unbound run and a repository nobody tagged produce the same empty `trace_ids`, and the per-language census keeps `binding_kinds` separate from `bound` so a rate is never drawn over the wrong denominator (#309) | Unit | P0 | FR-051-AC-23 | ✅ |
| TC-1047 | A CANONICAL marker on a non-binding symbol is not reported, and a legacy form in the same position is: the narrowing is about what each form guarantees, not about lowering a count. A canonical marker is syntax attached to the declaration that follows it, so one that bound nothing decorates either a symbol that bound (already filtered) or no declaration at all — data, not code. Measured: `cases/parser/triple-quote-scope-desync` carries `@pytest.mark.trace("TC-999")` inside a `"""…"""` literal as fixture data for the defect it pins, and the detector reported it — 1 false positive of 6 findings, tier-1 precision 83%. String masking covers Rust legacy forms only; Python and TypeScript are #323, and this does not wait on it. Mutation-verified: reporting canonical markers again fails it by name (CR-134, #312) | Unit | P0 | FR-051-AC-22 | ✅ |
| TC-1044 | A trace id a declared verifies form attaches to a symbol whose kind cannot bind it is reported, naming the id, the symbol, the kind and the form that matched — and the payload is otherwise unmoved: the tag still binds nothing and the symbol still does not enter `binding_census.candidates`. Before #312 the tag reached NO channel and nothing said so: `verifies` refuses it by kind, `implements` wants the literal keyword the comment does not carry, and the census omits it from the DENOMINATOR rather than counting it unbound — so `no-symbol-bound` and `low-symbol-binding` both read a denominator the defect has been removed from. Measured on the corpus fixture: census `python 1 candidate / 1 bound`, a flawless 100%, while the row the tag names is unbacked (CR-134, #312) | Unit | P0 | FR-051-AC-22 | ✅ |
| TC-1045 | An id that bound somewhere else is NOT reported, and neither is a symbol whose `implements` marker bound. Both rules are what keep the check off healthy input: a container's span runs to end of file, so the module symbol of any tagged test file carries every id in it, and the corpus's `tag-at-module-scope` control is exactly that tree. The second is the harder control the ticket predicted — the id-shaped annotation stays on production code and only the RELATION changes to `Implements:`, so a detector written as “any declared trace-id form inside a symbol that does not bind trace ids” fires and is wrong. Mutation-verified: removing the bound-elsewhere filter takes five controls red (CR-134, #312) | Unit | P0 | FR-051-AC-22 | ✅ |
| TC-1046 | Where several symbols span one tag the INNERMOST is named, because the fix a reader needs is the symbol the tag sits on and not the module that contains it. At equal `leading_line` a container loses, which is the tie a tag on line 1 of a file creates. Mutation-verified: preferring the outermost takes `tag-on-non-test-function` python and typescript from L3 to L2 — the actionable level is the one that goes (CR-134, #312) | Unit | P0 | FR-051-AC-22 | ✅ |
| TC-1043 | Deleting each field `corpus.yaml`'s `case_schema.required` names, from a real single-layout declaration, makes `CaseMeta` refuse to deserialize it naming the field — and every field the schema declares is one `CaseMeta` models, probed with a value ill-typed for every field so `unknown field` separates unmodelled from ill-typed. Behaviour, not two lists: the Python reader had no schema at all, so removing `issue_ref` left `bounds.py` exiting 0 over 77 fixtures while serde refused the same tree, and a second hand-written list in Python would be that defect one level up. Found `findable`/`reproduce` defaulted where the declaration said required, and `tags` required by TC-1021 where no declaration said so. Mutation-verified: restoring `#[serde(default)]` on `findable` fails it by name (CR-126, #336) | Unit | P0 | FR-065-AC-3 | ✅ |
| TC-861 | A built-in ambiguity term fires `quality:ambiguous-term` naming the term, and a quantified statement fires nothing (FR-056) | Integration | P0 | FR-056-AC-1 | ✅ |
| TC-862 | `as appropriate` is reported as itself, not as the `appropriate` inside it — the report names what the author wrote (FR-056) | Integration | P1 | FR-056-AC-2 | ✅ |
| TC-863 | A module's declared terms fire **and** every built-in still fires — the registry layers over the built-ins rather than replacing them (FR-056, CON-2) | Integration | P0 | FR-056-AC-3 | ✅ |
| TC-864 | `shall be validated` fires but `shall be validated by the parser` does not — the check is about missing allocation, not the passive voice (FR-056) | Integration | P0 | FR-056-AC-4 | ✅ |
| TC-865 | A statement mixing `shall` and `should` fires naming both; one modal alone fires nothing, because a lone `should` is a legitimate recommendation (FR-056) | Integration | P0 | FR-056-AC-5 | ✅ |
| TC-866 | An ambiguity term inside an inline code span is a mention and fires nothing; unquoted it fires — CR-017 parity (FR-056) | Integration | P0 | FR-056-AC-6 | ✅ |
| TC-867 | Every quality finding routes to warnings and never errors, and an FR-048 `quality:<check>=off` removes that check entirely (FR-056, CON-1) | Integration | P0 | FR-056-AC-7 | ✅ |
| TC-868 | Silencing the whole pack leaves the `ears` and `ac` finding streams identical field-for-field — the pack adds a grammar, it does not reinterpret the two that exist (FR-056, CON-4) | Integration | P0 | FR-056-AC-8 | ✅ |
| TC-869 | A statement violating three checks reports three findings — the checks are independent, not first-match (FR-056) | Integration | P1 | FR-056-AC-9 | ✅ |
| TC-876 | Two flawed rows of a `Constraints` table report two document lines, not two copies of the section heading's line — a mechanical lint that cannot point at the row makes the reader re-find it by hand (FR-056) | Unit | P1 | FR-056-AC-10 | ✅ |
| TC-877 | A `must`-only and a `may`-only prose requirement are judged by the pack, and a line carrying no modal at all is judged by none — the collection gate admits exactly the four modals `mixed-modal` reads (FR-056) | Unit | P0 | FR-056-AC-11 | ✅ |
| TC-878 | `by 12:00`, `by name` and `by priority` do not suppress `agentless-passive`, while `by the archiver` does, and an agent wrapped in emphasis or a code span still counts as named (FR-056) | Unit | P0 | FR-056-AC-12 | ✅ |
| TC-879 | A typo'd key inside an `ambiguity_terms` entry fails module load naming the key, as `verification_catalog` already did (FR-056) | Unit | P1 | FR-056-AC-13 | ✅ |
| TC-883 | `refs:dangling-reference` mapped `error` promotes an Okf warning and fails the bundle; mapped `warning` demotes a Strict hard error and the bundle becomes valid; mapped `off` records nothing in either vector, in either posture | Integration | P0 | FR-057-AC-1, FR-057-AC-2, FR-057-AC-3 | ✅ |
| TC-884 | With no entry, each check keeps its exact pre-FR-057 tier, asserted per check: `dangling-reference`/`index-incomplete`/`index-okf-version` posture-routed, `no-frontmatter`/`malformed-frontmatter` warning in both postures | Integration | P0 | FR-057-AC-4 | ✅ |
| TC-885 | `merge_severity_overrides` + `with_grammar_severity` — quire-cli's `apply_severity_overrides` verbatim — reaches corpus checks, and a CLI entry beats a module entry for the same key while leaving the source registry untouched | Integration | P0 | FR-057-AC-5 | ✅ |
| TC-886 | Every finding carries the severity applied; the `reason` tokens are byte-identical to their pre-FR values; every pack finding yields a `<pack>:<check>` key `is_severity_key` accepts, asserted over what the engine emits rather than a hardcoded list | Integration | P0 | FR-057-AC-7, FR-057-AC-8, FR-057-AC-9 | ✅ |
| TC-887 | Findings appear in the same order with and without a severity map, and repeated runs over one bundle agree (NFR-006) | Integration | P1 | FR-057-AC-10 | ✅ |
| TC-888 | `unknown-type` and the missing-`type` error stand even with `bundle:unknown-type` and `bundle:frontmatter` mapped `off`, and carry no pack, so no key can ever address them | Integration | P0 | FR-057-CON-1 | ✅ |
| TC-889 | With `refs:dangling-reference` off, a bundle that also has a dangling trace reference still reports `dangling-trace-reference` at its unchanged count | Integration | P0 | FR-057-AC-6 | ✅ |
| TC-890 | `normalize_statement` is idempotent, and `statement_hash` is invariant under it — an obligation's identity cannot depend on how many times its text was round-tripped | Property | P0 | FR-053-AC-4 | ✅ |
| TC-891 | `ears::normalize` is idempotent and never lengthens its input. Before CR-069 stripping `**` could synthesize a link the link pass had already run past (`[]**()` → `[]()` → `""`); the generator draws from the tokens the function rewrites, because a plain `.*` strategy passed 2 000 cases against the broken form | Property | P0 | FR-042-AC-1 | ✅ |
| TC-892 | `normalize_reference_cell` is idempotent under every combination of the two opt-in flags, and is the identity with both off. Before CR-069 a chained range `FR-001..FR-003..FR-005` expanded only its first range and left a `..` the pattern rejects | Property | P0 | FR-049-AC-1 | ✅ |
| TC-893 | `slug` is idempotent, its output alphabet is closed over `[a-z0-9-]` for any input including non-ASCII, and `slug_line_id` ignores surrounding whitespace on the heading — formatting cannot move a block id | Property | P0 | FR-009-AC-1, FR-009-AC-3, FR-009-AC-6 | ✅ |
| TC-894 | `mask_code_spans` is idempotent, preserves byte length (so an offset found in the mask indexes the original), and never *introduces* a backtick. Equality of backtick positions would be wrong — a backtick inside a span is span content | Property | P0 | FR-047-AC-18 | ✅ |
| TC-895 | `allowed_links` normalization reaches a fixpoint: re-serializing a normalized map and normalizing again yields the same map, from both the array and the map authoring form | Property | P1 | FR-040-AC-4 | ✅ |
| TC-896 | `update_section` with a section's **own** content is the byte-identity on the whole document, including an empty section; and a write into a section whose heading is the document's last line inserts the separating line break instead of concatenating onto the heading text | Property | P0 | FR-022-AC-6, FR-022-AC-7 | ✅ |
| TC-898 | A document of the declared `from` archetype with no accepted edge is reported naming the document and the declaration; one holding the edge is not | Integration | P0 | FR-058-AC-1 | ✅ |
| TC-899 | Any one of the declared `edges` satisfies the relation; a verb the declaration does not list does not | Integration | P0 | FR-058-AC-2 | ✅ |
| TC-900 | `direction: incoming` reports a document nothing points at over the accepted verbs — the same declaration read the other way | Integration | P0 | FR-058-AC-3 | ✅ |
| TC-901 | A dangling edge does not satisfy a relation whose `to` is constrained, so a typo'd target cannot satisfy the requirement it broke | Integration | P0 | FR-058-AC-4 | ✅ |
| TC-902 | A three-node `refines` cycle is reported exactly once, naming the path, keyed on the cycle's smallest member so rotations collapse | Integration | P0 | FR-058-AC-5, FR-058-AC-9 | ✅ |
| TC-903 | Each relation carries its own `trace:<check>` key: one mapped `off` leaves its sibling reporting, one mapped `error` promotes only it, and unconfigured findings are advisory | Integration | P0 | FR-058-AC-6, FR-058-AC-7 | ✅ |
| TC-904 | A module declaring neither `required_relations` nor `acyclic_edges` sees no FR-058 finding at all | Integration | P0 | FR-058-AC-8 | ✅ |
| TC-905 | Every field of the declared model survives `is_empty` and the cross-module merge — the guard for the two hand-maintained per-field functions a new field silently breaks | Integration | P0 | FR-058-CON-1 | ✅ |
| TC-906 | A declaration that cannot be executed is rejected at load: no accepted `edges` (which would report every `from` document), and a `check` token that cannot form a `trace:<check>` severity key (which would leave the relation untunable) | Integration | P0 | FR-058-AC-10 | ✅ |
| TC-907 | Two required relations cannot share a name — a finding must be traceable to the declaration that produced it | Integration | P1 | FR-058-AC-10 | ✅ |
| TC-908 | A relation whose `from` names a kind nothing declares and no document is reports itself; the same typo silently disables the orphan check, which is the damage the guard prevents | Integration | P0 | FR-058-AC-11 | ✅ |
| TC-909 | An FR satisfying a `US` rather than a `StR` is not an orphan — the `to` list accepts several upstream kinds (the 29148 stakeholder→system→software chain) | Integration | P1 | FR-058-AC-2 | ✅ |
| TC-910 | The finding reads as a sentence for both shapes of `to` — the `to: []` case rendered "from any any document" in the first end-to-end run against spec-objects-safety | Integration | P2 | FR-058-AC-1 | ✅ |
| TC-911 | A declared vocabulary value no document claims is reported and a claimed one is not — the all-functional-requirements failure mode, invisible to every per-document check | Integration | P0 | FR-059-AC-1 | ✅ |
| TC-912 | The vocabulary is READ from the projected archetype's frontmatter-schema `enum`; the fixture manifest restates no value, so a manifest-list implementation could not pass | Integration | P0 | FR-059-AC-2 | ✅ |
| TC-913 | A value named in the declared justified-absence field counts as covered — a check that cannot accept an answer forces a false finding or a fabricated requirement | Integration | P0 | FR-059-AC-3 | ✅ |
| TC-914 | The justification may live on any document, not only the projected archetype — otherwise an NFR must be authored to say an NFR is unnecessary | Integration | P1 | FR-059-AC-4 | ✅ |
| TC-915 | The finding carries its own `trace:<check>` key: advisory by default, dropped by `off`, promoted by `error` | Integration | P0 | FR-059-AC-5 | ✅ |
| TC-916 | A declaration whose field yields no `enum` reports itself rather than silently reporting nothing unowned | Integration | P0 | FR-059-AC-7 | ✅ |
| TC-917 | A module declaring no `vocabulary_coverage` sees byte-identical output | Integration | P0 | FR-059-AC-6 | ✅ |
| TC-918 | An empty projection is ONE finding naming how many values are unowned, not one per value — 90 of 243 corpus bundles have no NFR at all, which was 1080 of 2792 findings | Integration | P0 | FR-059-AC-8 | ✅ |
| TC-919 | `column_vocabularies: {Header: name}` resolves to the declared values and the reference is consumed, so nothing downstream must understand it | Integration | P0 | FR-060-AC-1 | ✅ |
| TC-920 | `from_vocabulary: name` resolves the scalar counterpart the same way | Integration | P0 | FR-060-AC-2 | ✅ |
| TC-921 | A vocabulary name no module declares resolves to an EMPTY choice set, never an absent constraint — dropping it would let a typo silently widen the contract | Integration | P0 | FR-060-AC-3 | ✅ |
| TC-922 | A literal `choices` beside a reference wins and the reference is dropped rather than merged | Integration | P1 | FR-060-AC-4 | ✅ |
| TC-923 | An archetype naming no vocabulary is byte-identical and not cloned | Integration | P0 | FR-060-AC-5 | ✅ |
| TC-924 | A reference is legal exactly where its literal is: `from_vocabulary` rejected on `table_row`, `column_vocabularies` rejected off it — found by a failing test, not by design | Integration | P0 | FR-060-AC-6 | ✅ |
| TC-925 | The t-way tuple count is the sum over every set of t dimensions of the product of their value counts — hand-computable on a small case | Unit | P0 | FR-061-AC-1 | ✅ |
| TC-926 | A strength of 0, or above the dimension count, yields 0 tuples rather than an error or the full product | Unit | P0 | FR-061-AC-2 | ✅ |
| TC-927 | A forbidden combination is excluded — counting one that cannot exist makes the target permanently unreachable | Unit | P0 | FR-061-AC-3 | ✅ |
| TC-928 | An exclusion forbids every wider tuple containing it, so a two-value constraint bites at strength 3 | Unit | P0 | FR-061-AC-4 | ✅ |
| TC-929 | The hashed statement carries every value and the strength, so any change to the space suspects every binding over it | Unit | P0 | FR-061-AC-5 | ✅ |
| TC-930 | Cells parse as authored; a repeated value counts once and a single-assignment exclusion is rejected | Unit | P1 | FR-061-AC-6 | ✅ |
| TC-931 | `obligation::for_document` mints ONE obligation for the whole table with strength/dimensions/tuples in parameters | Integration | P0 | FR-061-AC-7 | ✅ |
| TC-932 | A space with fewer than two real dimensions mints nothing — a permanently-satisfied row reads exactly like a real one | Integration | P0 | FR-061-AC-8 | ✅ |
| TC-933 | A combinatorial source declaring strength 0 fails at module load | Unit | P0 | FR-061-AC-9 | ✅ |
| TC-934 | The corpus path mints the same one obligation as the single-document path — same id, parameters and statement hash, so a binding made against one matches the other | Unit | P0 | FR-061-AC-10 | ✅ |
| TC-935 | The obligation record carries the test-case ids its method cell names — several, none, and an unclosed parenthetical that must read nothing rather than invent an id | Unit | P0 | FR-053-AC-11 | ✅ |
| TC-936 | A production symbol carrying an `implements` marker mints the relation and backs NO trace id — scope is not evidence | Unit | P0 | FR-062-AC-1 | ✅ |
| TC-937 | An `implements` marker on a test, and a `trace` marker on production code, each bind nothing: the symbol kinds are complements, so getting it wrong yields no relation rather than the wrong one | Unit | P0 | FR-062-AC-2 | ✅ |
| TC-938 | A requirement named by several markers yields one relation, deterministically ordered | Unit | P1 | FR-062-AC-3 | ✅ |
| TC-939 | The `implements` relation reaches `coverage --json` and moves no coverage number — FR-061 shipped a branch the report never carried, so minting is asserted separately from exposing | Unit | P0 | FR-062-AC-4 | ✅ |
| TC-940 | Marker forms declared in a **module manifest** reach `bind`, loaded through `Registry::load_module` rather than pushed onto the struct — the path every consumer uses, and the one the other four tests bypass | Unit | P0 | FR-062-AC-5 | ✅ |
| TC-941 | A status value the declared vocabulary classes as nothing is reported in `undeclared_statuses` with the authored string verbatim; an all-declared corpus omits the key entirely (CR-083) | Unit | P0 | FR-050-AC-21 | ✅ |
| TC-942 | Vocabulary drift is reported on a **backed** row too — the classification sits above the backed early-continue, so the backstop sees every row and not the unbacked subset (CR-083) | Unit | P0 | FR-050-AC-21 | ✅ |
| TC-943 | A curried `it.skipIf(cond)(…)` / `it.each([…])(…)` registration, and one whose title wraps onto a later line, each register a test symbol (CR-084) | Unit | P0 | FR-051-AC-18 | ✅ |
| TC-944 | A declared `source_exclude` glob removes the matching file's symbols; a non-matching glob leaves the extraction byte-identical; a start-anchored glob cannot reach a nested directory of the same name (CR-085) | Unit | P0 | FR-050-AC-22 | ✅ |
| TC-945 | `source_exclude` patterns are compile-checked at module load and name the key **as the noun of the message** — "invalid/empty `source_exclude` pattern", not the location prefix alone (CR-088) — when they fail; a model declaring only non-source paths still reads as undeclared (CR-085) | Unit | P1 | FR-050-AC-22, FR-050-AC-25 | ✅ |
| TC-946 | Two byte-identical undeclared-status rows collapse to one record — `undeclared_statuses` is deduplicated after its sort, mirroring `untracked_symbols` — while two distinct drifted values on duplicate row ids both survive (CR-086) | Unit | P1 | FR-050-AC-21 | ✅ |
| TC-947 | An empty `implements` serializes with no key at all, a populated one carries its records, and the published contract accepts both payloads — the discrete record of the `implements` optional-key correction #203 folded into tc859 with no matrix presence (CR-086) | Unit | P1 | FR-055-AC-6 | ✅ |
| TC-948 | The forward scan refuses what is not a title: a variable title, an `iterate(` lookalike and a literal past the window register nothing — a wrong symbol name is worse than none (CR-084; re-idded from a duplicate TC-943 by CR-087) | Unit | P0 | FR-051-AC-18 | ✅ |
| TC-949 | The key can only subtract — `spec/**` as a source glob cannot un-exclude the document root, whose exclusion is the caller's non-configurable argument (CR-085; re-idded from a duplicate TC-944 by CR-087) | Unit | P0 | FR-050-AC-22, FR-050-AC-17 | ✅ |
| TC-950 | A trace id bound by two distinct source symbols is reported in `shared_trace_ids` with the id and both binders, deterministically ordered, while the row stays backed and `totals` are untouched (CR-087) | Unit | P0 | FR-050-AC-23 | ✅ |
| TC-951 | A corpus whose every id is uniquely bound reports an empty `shared_trace_ids` and the key is absent from the JSON — byte-identity for repositories already conformant (CR-087) | Unit | P1 | FR-050-AC-23 | ✅ |
| TC-952 | The source walk counts what a declared glob subtracts: a matching glob's removals land in `SymbolExtraction.excluded_source_files`, a non-matching or empty glob list counts zero (CR-088) | Unit | P0 | FR-050-AC-24 | ✅ |
| TC-953 | The excluded-file count travels extraction → symbol graph → `CoverageReport.excluded_source_files` and the JSON key; a report excluding nothing omits the key entirely — never `0` — keeping byte-identity (CR-088) | Unit | P0 | FR-050-AC-24 | ✅ |
| TC-954 | An invalid glob in the list refuses the **whole** list: no glob applies — never a partial subset — and a diagnostic names the pattern that does not compile; red-verified against the shipped silent-partial-filter behaviour (CR-088) | Unit | P0 | FR-050-AC-25 | ✅ |
| TC-955 | Every row-shaped record carries the 1-based frontmatter-included document line of its matrix row, hand-counted fixture positions as ground truth; two unbacked rows in one document report different lines (CR-089) | Unit | P0 | FR-050-AC-26 | ✅ |
| TC-956 | `untracked_symbols` carries the tagged test's declaration line — `Symbol.line` always had it, the `VerifiesRelation` in between discarded it (CR-089) | Unit | P0 | FR-050-AC-26 | ✅ |
| TC-957 | The contract's `line` keys are optional in both directions: an unrecovered line is omitted — never `null` — a recovered one is carried, and the published schema accepts both payloads (CR-089) | Unit | P1 | FR-050-AC-26 | ✅ |
| TC-958 | Every widened registration form — curried, parametrised, multi-modifier, wrapped title, whitespace-separated, awaited — reaches `extract_tree` as a test symbol named by its title, in declaration order, with span and container attributes; CR-084 verified the scanner only through the crate-private `parse()` (CR-090; TC-959 skipped, see the CR-090 note) | Integration | P0 | FR-051-AC-18 | ✅ |
| TC-960 | The fixture's negative shapes register nothing through `extract_tree`: a variable title, a title past the lookahead window, an `iterate(` lookalike and a whitespace-split modifier chain — absence, never a fallback name (CR-090) | Integration | P1 | FR-051-AC-18 | ✅ |
| TC-961 | The widened grammar's edges are pinned one by one: whitespace before `(`, whitespace between curried groups and an unbounded `.modifier` chain are admitted; a whitespace-split chain, an empty modifier and an identifier continuing past `test`/`it` stay outside (CR-090) | Unit | P0 | FR-051-AC-18 | ✅ |
| TC-962 | A claimed vocabulary value is an `owned` record in the coverage payload, naming the claiming document and carrying `{vocabulary, archetype, field, check}` — a fact the warning stream never states, because a covered value produces no warning at all (CR-091) | Integration | P0 | FR-059-AC-9 | ✅ |
| TC-963 | An excused value is an `excused` record naming the excusing document — a Spec, not the projected archetype — so "who excused this, and where" is answerable without a second frontmatter reader (CR-091) | Integration | P0 | FR-059-AC-9 | ✅ |
| TC-964 | A value nothing claims and nothing excuses is an `unowned` record with no documents, and the record set covers every declared value exactly once in the schema enum's order (CR-091) | Integration | P0 | FR-059-AC-9 | ✅ |
| TC-965 | A module declaring no `vocabulary_coverage` serializes with no `vocabulary_coverage` key at all — never `[]` — keeping FR-050-AC-7 byte-identity for every module that has not adopted the declaration (CR-091) | Integration | P0 | FR-059-AC-9 | ✅ |
| TC-966 | A declaration whose field yields no `enum` is an `undeclared-coverage-vocabulary` diagnostic on the coverage surface too — without it a dead declaration and an undeclared module read identically in the payload (CR-091) | Integration | P0 | FR-059-AC-10 | ✅ |
| TC-967 | The `uncatalogued-verification-method` diagnostic carries the authored method in a structured `value` field, byte-equal to the obligation records' `method`, and a diagnostic not about one value omits the key — never `null` (CR-091) | Integration | P0 | FR-054-AC-12 | ✅ |
| TC-968 | A declared catalog `cost` survives to the accessor verbatim, and an entry declaring none reads `None` — absence is "the module said nothing", never a default (CR-092) | Integration | P0 | FR-054-AC-13 | ✅ |
| TC-969 | The serialized catalog entry omits an undeclared `cost` entirely — never `null` — and carries a declared one, so a consumer written before the field existed reads an unchanged entry from every catalog that has not adopted it (CR-092) | Integration | P1 | FR-054-AC-13 | ✅ |
| TC-982 | The binding census counts evidence-symbol candidates and bound symbols per language, ordered by language label, naming the forms consulted; a container and a production function are never candidates; `bound` counts symbols not relations; and the same tree bound against a grammar matching nothing reports identical candidates with zero bound (CR-093) | Unit | P0 | FR-051-AC-19 | ✅ |
| TC-983 | A language whose evidence symbols all fail to bind is a `no-symbol-bound` diagnostic naming the language, the candidate count and every consulted form; the same tree with the declared spelling reports the census, no diagnostic, and carries `binding_census` in the JSON either way (CR-093) | Integration | P0 | FR-050-AC-27 | ✅ |
| TC-984 | 1 of 21 candidates bound is a `low-symbol-binding` diagnostic carrying both counts rather than a verdict, and is not the zero case; 2 of 21 is over the floor and reports nothing (CR-093) | Integration | P0 | FR-050-AC-27 | ✅ |
| TC-985 | A hollow RATIO is a non-zero population with input offered and none read; a zero `examined` (nothing to read), a zero population, and a low-but-non-zero `matched` are each not hollow — the `examined` half was added because without it the check fired on a fixture whose source tree is one comment line (CR-094). A COUNT with the identical numbers is never hollow, so the shape decides and not the arithmetic (CR-102) | Unit | P0 | FR-063-AC-1, FR-063-AC-6 | ✅ |
| TC-986 | `not computed` and `computed zero` are unequal, serialize under different states, and the uncomputed one carries no `value`/`population`/`examined`/`matched` at all — there is no zero to be read as an answer; both round-trip (CR-094, #226) | Unit | P0 | FR-063-AC-2 | ✅ |
| TC-987 | A metric cannot be constructed without a unit and a method, and reports its numerator (CR-094) | Unit | P1 | FR-063-AC-1 | ✅ |
| TC-997 | Every assertion behind a narrowing guard is a `vacuous-under-guard` suspicion carrying guarded/total; one unguarded assertion yields none (CR-100) | Unit | P0 | FR-064-AC-1 | ✅ |
| TC-998 | Absence of an assertion MACRO is not a finding: a `never panics` proptest and a compile-time `assert_send_sync::<T>()` both yield nothing — measured, that rule was wrong 12 of 12 sampled — and production code is never judged (CR-100) | Unit | P0 | FR-064-AC-1 | ✅ |
| TC-999 | An oracle that is a character-for-character copy of the implementation scores 1.00 and is reported; one that judges the same subject independently is not (CR-100) | Unit | P0 | FR-064-AC-2 | ✅ |
| TC-1000 | Similarity reads identifier tokens: reformatting scores identical, shared keywords alone stay under the floor, an empty side scores 0 (CR-100) | Unit | P0 | FR-064-AC-3 | ✅ |
| TC-1001 | Suspicions reach the report ordered by `(path, line, symbol)` each carrying non-empty evidence; removing them changes no total and no diagnostic count, and the key is absent when there are none (CR-100) | Integration | P0 | FR-064-AC-4, FR-064-CON-1 | ✅ |
| TC-1002 | A narrowing guard that opens AND closes on one line guards the assertion on that line; a one-line `for` body, which is not a guard, does not (CR-102) | Unit | P0 | FR-064-AC-1 | ✅ |
| TC-1003 | A TypeScript `vitest` suite of ordinary `it(…)` arrow-function bodies yields no suspicion — an arrow function is not a `match` arm, the misread that produced 549 of 551 on `agent-ix/quoin` (CR-102) | Unit | P0 | FR-064-AC-5 | ✅ |
| TC-1004 | A guard and an assertion quoted inside a COMMENT are neither; the same two tokens as real code do report, so the control measures comment-stripping rather than an absent match (CR-102) | Unit | P0 | FR-064-AC-5 | ✅ |
| TC-992 | Marker-form mismatch and its control: an undeclared marker spelling yields candidates with zero bound, `no-symbol-bound` and `hollow-denominator`; the same tree with the declared spelling fires none of them and reports `matched` equal to `examined` (CR-098) | Integration | P0 | FR-050-AC-29 | ✅ |
| TC-993 | A stale test NAME over a correct marker binds correctly and is reported as no defect on either side (CR-098) | Integration | P0 | FR-050-AC-29 | ✅ |
| TC-994 | A corpus with no evidence symbols reports 0% honestly: `examined` 0, not hollow, no diagnostic — the case `examined` exists for (CR-098) | Integration | P0 | FR-050-AC-29 | ✅ |
| TC-995 | A module declaring no `implements` forms reports `coverage.implements` as `not_computed` with no numbers, so an empty list and an unasked question are distinguishable (CR-098) | Integration | P0 | FR-050-AC-29 | ✅ |
| TC-996 | A corpus whose every extractable criterion is the catch-all reports `specific_shaped` 0 alongside a non-zero `property_shaped` (CR-098) | Integration | P0 | FR-050-AC-29 | ✅ |
| TC-991 | A row-scoped assert failure carries the offending row's `id_column` cell and its own document line, so two rows failing one check are two distinguishable findings rather than two byte-identical strings at one locus; a table-scoped failure (`min_rows`) keeps the section line and names no row; and an assert declaring no `id_column` yields a row line with no guessed id (CR-097) | Unit | P0 | FR-033-AC-16 | ✅ |
| TC-1005 | The whole-document line a row-scoped assert reports is the offending row's own, not the dashed separator line above it, checked through `validate_document` with frontmatter present so the frontmatter offset, section start, heading line and 1-based conversion are verified together; its control fires nothing (#254) | Unit | P0 | FR-033-AC-16 | ✅ |
| TC-1006 | The binding census names ONE unbound candidate — the lowest `(path, line)` so it is deterministic, at the ANNOTATION line rather than the `fn` line because that is what a reader edits — and `no-symbol-bound` carries it as `path` and in its message; the control, the same tree with the declared marker, carries no example and fires nothing (#256) | Unit | P0 | FR-051-AC-19 | ✅ |
| TC-1007 | A classified criterion reports its own ABSOLUTE document line with frontmatter present, not the separator row above it — asserted as exact values because a relative assertion is satisfied by every off-by-N, which is how #254 shipped (#257) | Unit | P0 | FR-052-AC-18 | ✅ |
| TC-1008 | The `no_source_symbol` vocabulary is consulted for BOTH the declared test-type column and each reference declaration's own column, with parentheticals stripped so `Inspection (TC-002)` names the method it annotates; its control, the same tree verified by `Test`, exempts nothing (#259) | Unit | P0 | FR-050-AC-16 | ✅ |
| TC-1009 | `catch-all-universal` is a LOCATED diagnostic, not only a moving metric: one finding per corpus carrying the all-universal document count and naming a criterion at `path:line`; its control, one specifically-shaped criterion, silences it (#261) | Unit | P0 | FR-050-AC-28 | ✅ |
| TC-990 | Decomposition keys on quantification, not on the winning label: a quantified `invariant` statement carries the same spans the `universal` path produces, an unquantified `idempotence` statement carries none, and both hold identically with and without a declared idiom registry (CON-4 unchanged) (CR-096) | Unit | P0 | FR-052-AC-19 | ✅ |
| TC-989 | The catch-all is split out of the headline: a corpus whose every extractable criterion is `universal` reports `specific_shaped` 0 while `property_shaped` stays non-zero, both figures reach the FR-063 envelope under their own names over the same denominator, and per-shape `grounding` counts every classified criterion exactly once with `all_three` never exceeding any of its parts (CR-095) | Integration | P0 | FR-050-AC-28, FR-052-AC-18 | ✅ |
| TC-988 | Every coverage headline number is enveloped with unit, method, population, `examined` and `matched`; three tests carrying an undeclared marker spelling make `coverage.backed` hollow and mint a `hollow-denominator` diagnostic naming it; `coverage.implements` reports `not_computed` with the condition named; and the same tree read cleanly, and a tree with no symbols at all, each report nothing (CR-094) | Integration | P0 | FR-063-AC-3, FR-063-AC-4, FR-063-AC-5 | ✅ |
| TC-897 | Every exclusion in the relative-destination filter is load-bearing, one at a time — empty, `scheme://`, `#anchor`, `mailto:`, `tel:`, non-`.md` — including forms carrying a `.md` tail; and end to end, a document whose only links are excluded destinations mints no edge. Found by the quoin#48 mutation pilot: each `&&` flipped to `\|\|` with no test failing | Unit | P0 | FR-026-AC-14 | ✅ |
| TC-797 | A declared model matching zero rows: `quire coverage` renders `0/0` distinctly and never as `100%`, and `--strict` exits non-zero on it — the state that made a wired gate pass vacuously (CR-035) | Integration | P0 | FR-050-AC-14 | 🚧 awaiting EXT-3 `quire-cli` (CLI behaviour; `tests/cli_coverage.rs` — CR-058) |
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

**Performance criteria (`US-nnn-PC-n`) are outside this audit** — one treatment for all of them (CR-058). They are verified by benches (TC-450..454, TC-455..459, TC-469, TC-492, TC-498) carrying their own 🚧 status, not by acceptance-criterion coverage, and a bench cannot back a matrix row in any case: `trace::bind` binds trace ids on test functions only. Before CR-058 the US-011..013 PCs were audited here while the 22 US-006..010 PCs were not, so the denominator was inconsistent with itself; 9 of the 22 name no bench at all, and auditing them would have reported a coverage gap for criteria this matrix never claimed to cover that way.

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
| US-012-AC-1 | TC-493 |
| US-012-AC-2 | TC-495 |
| US-012-AC-3 | TC-494 |
| US-012-AC-4 | TC-496 |
| US-012-AC-5 | TC-485 |
| US-013-AC-1 | TC-486 |
| US-013-AC-2 | TC-487 |
| US-013-AC-3 | TC-488 |
| US-013-AC-4 | TC-489 |
| US-013-AC-5 | TC-490 |
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
| FR-005-AC-5 | TC-812 |
| FR-005-AC-6 | TC-813 |
| FR-005-AC-7 | TC-819 |
| FR-005-AC-8 | TC-821 |
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
| FR-009-AC-1 | TC-025, TC-893 |
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
| FR-019-AC-2 | TC-402 |
| FR-019-AC-3 | TC-443 |
| FR-020-AC-1 | TC-410 |
| FR-020-AC-2 | TC-411 |
| FR-022-AC-1 | TC-430 |
| FR-022-AC-2 | TC-431 |
| FR-022-AC-3 | TC-432 |
| FR-022-AC-4 | TC-433 |
| FR-022-AC-5 | TC-434, TC-435 |
| FR-022-AC-6 | TC-896 |
| FR-022-AC-7 | TC-896 |
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
| FR-024-AC-10 | TC-807 |
| FR-024-AC-11 | TC-808 |
| FR-024-AC-12 | TC-820 |
| FR-025-AC-1 | TC-480 |
| FR-025-AC-2 | TC-481 |
| FR-025-AC-3 | TC-482 |
| FR-025-AC-4 | TC-483 |
| FR-025-AC-5 | TC-484 |
| FR-025-AC-6 | TC-485 |
| FR-025-AC-7 | TC-817 |
| FR-025-AC-8 | TC-815, TC-816 |
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
| FR-026-AC-12 | TC-880 |
| FR-026-AC-13 | TC-881 |
| FR-026-AC-14 | TC-897 |
| FR-058-AC-1 | TC-898 |
| FR-058-AC-1 | TC-910 |
| FR-059-AC-1 | TC-911 |
| FR-059-AC-2 | TC-912 |
| FR-059-AC-3 | TC-913 |
| FR-059-AC-4 | TC-914 |
| FR-059-AC-5 | TC-915 |
| FR-059-AC-6 | TC-917 |
| FR-059-AC-7 | TC-916 |
| FR-059-AC-8 | TC-918 |
| FR-059-AC-9 | TC-962, TC-963, TC-964, TC-965 |
| FR-059-AC-10 | TC-966 |
| FR-060-AC-1 | TC-919 |
| FR-060-AC-2 | TC-920 |
| FR-060-AC-3 | TC-921 |
| FR-060-AC-4 | TC-922 |
| FR-060-AC-5 | TC-923 |
| FR-060-AC-6 | TC-924 |

| FR-061-AC-1 | TC-925 |
| FR-061-AC-2 | TC-926 |
| FR-061-AC-3 | TC-927 |
| FR-061-AC-4 | TC-928 |
| FR-061-AC-5 | TC-929 |
| FR-061-AC-6 | TC-930 |
| FR-061-AC-7 | TC-931 |
| FR-061-AC-8 | TC-932 |
| FR-061-AC-9 | TC-933 |
| FR-033-AC-16 | TC-991, TC-1005 |
| FR-050-AC-28 | TC-989, TC-1009 |
| FR-050-AC-29 | TC-992, TC-993, TC-994, TC-995, TC-996 |
| FR-050-AC-33 | TC-1033, TC-1034, TC-1035 |
| FR-050-AC-34 | TC-1037, TC-1038 |
| FR-050-AC-35 | TC-1041 |
| FR-050-AC-36 | TC-1048, TC-1049 |
| FR-050-AC-37 | TC-1050, TC-1051 |
| FR-064-AC-1 | TC-997, TC-998, TC-1002 |
| FR-064-AC-2 | TC-999 |
| FR-064-AC-3 | TC-1000 |
| FR-064-AC-4 | TC-1001 |
| FR-064-AC-5 | TC-1003, TC-1004 |
| FR-050-AC-31 | `scripts/tests/test_overfit_check.py` (Inspection — python-side sweep, mints no Rust symbol) |
| FR-050-AC-32 | `scripts/tests/test_bench.py` (Inspection — python-side gate, mints no Rust symbol) |
| FR-050-AC-30 | `scripts/tests/test_bench.py` (Inspection — python-side gate, mints no Rust symbol) |
| FR-052-AC-18 | TC-989, TC-1007 |
| FR-052-AC-19 | TC-780, TC-990 |
| FR-063-AC-1 | TC-985, TC-987 |
| FR-063-AC-6 | TC-985 |
| FR-063-AC-2 | TC-986 |
| FR-063-AC-3 | TC-988 |
| FR-063-AC-4 | TC-988 |
| FR-063-AC-5 | TC-988 |
| FR-063-AC-7 | TC-1036 |
| FR-058-AC-2 | TC-899 |
| FR-058-AC-2 | TC-909 |
| FR-058-AC-3 | TC-900 |
| FR-058-AC-4 | TC-901 |
| FR-058-AC-5 | TC-902 |
| FR-058-AC-6 | TC-903 |
| FR-058-AC-7 | TC-903 |
| FR-058-AC-8 | TC-904 |
| FR-058-AC-9 | TC-902 |
| FR-058-AC-10 | TC-906 |
| FR-058-AC-10 | TC-907 |
| FR-058-AC-11 | TC-908 |
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
| FR-040-AC-4 | TC-638, TC-895 |
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
| FR-042-AC-1 | TC-657, TC-891 |
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
| FR-044-AC-8 | TC-823 |
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
| FR-047-AC-15 | TC-775 |
| FR-047-AC-16 | TC-776 |
| FR-047-AC-17 | TC-777 |
| FR-047-AC-18 | TC-778, TC-894 |
| FR-048-AC-5 | TC-720, TC-766 |
| FR-048-AC-6 | TC-721 |
| FR-048-AC-7 | TC-722 |
| FR-048-AC-8 | TC-723 |
| FR-048-AC-9 | TC-752 |
| FR-048-AC-10 | TC-755 |
| FR-048-AC-11 | TC-794 |
| FR-049-AC-1 | TC-724, TC-892 |
| FR-049-AC-2 | TC-725 |
| FR-049-AC-3 | TC-726 |
| FR-049-AC-4 | TC-727 |
| FR-049-AC-5 | TC-728 |
| FR-049-AC-6 | TC-729 |
| FR-049-AC-7 | TC-730 |
| FR-049-AC-8 | TC-731 |
| FR-049-AC-9 | TC-814 |
| FR-050-AC-1 | TC-732 |
| FR-050-AC-2 | TC-733 |
| FR-050-AC-3 | TC-734 |
| FR-050-AC-4 | TC-735 |
| FR-050-AC-5 | TC-736 |
| FR-050-AC-6 | TC-737 |
| FR-050-AC-7 | TC-738, TC-824, TC-1058 |
| FR-050-AC-8 | TC-739 |
| FR-050-AC-9 | TC-740 |
| FR-050-AC-10 | TC-758 |
| FR-050-AC-11 | TC-759 |
| FR-050-AC-12 | TC-760 |
| FR-050-AC-13 | TC-788, TC-826 |
| FR-050-AC-14 | TC-797 |
| FR-050-AC-15 | TC-801, TC-826, TC-829, TC-830 |
| FR-050-AC-16 | TC-805, TC-1008 |
| FR-050-AC-17 | TC-809, TC-810, TC-811, TC-949 |
| FR-050-AC-18 | TC-818, TC-738 |
| FR-050-AC-19 | TC-822 |
| FR-050-AC-20 | TC-824 |
| FR-050-AC-21 | TC-941, TC-942, TC-946 |
| FR-050-AC-22 | TC-944, TC-945, TC-949 |
| FR-050-AC-23 | TC-950, TC-951 |
| FR-050-AC-24 | TC-952, TC-953 |
| FR-050-AC-25 | TC-954, TC-945 |
| FR-050-AC-26 | TC-955, TC-956, TC-957 |
| FR-050-AC-27 | TC-983, TC-984 |
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
| FR-051-AC-12 | TC-798, TC-799 |
| FR-051-AC-13 | TC-800 |
| FR-051-AC-14 | TC-803 |
| FR-051-AC-15 | TC-804 |
| FR-051-AC-16 | TC-806 |
| FR-051-AC-17 | TC-827, TC-828 |
| FR-051-AC-18 | TC-943, TC-948, TC-958, TC-960, TC-961 |
| FR-051-AC-19 | TC-982, TC-1006 |
| FR-051-AC-20 | TC-1029, TC-1030, TC-1031 |
| FR-051-AC-21 | TC-1039, TC-1040 |
| FR-051-AC-22 | TC-1044, TC-1045, TC-1046, TC-1047 |
| FR-052-AC-1 | TC-779 |
| FR-052-AC-2 | TC-780 |
| FR-052-AC-3 | TC-781 |
| FR-052-AC-4 | TC-782 |
| FR-052-AC-5 | TC-783 |
| FR-052-AC-6 | TC-784 |
| FR-052-AC-7 | TC-785 |
| FR-052-AC-8 | TC-786 |
| FR-052-AC-9 | TC-787 |
| FR-052-AC-10 | TC-788 |
| FR-052-AC-11 | TC-789 |
| FR-052-AC-12 | TC-790 |
| FR-052-AC-13 | TC-791 |
| FR-052-AC-14 | TC-792 |
| FR-052-AC-15 | TC-793 |
| FR-052-AC-16 | TC-795 |
| FR-052-AC-17 | TC-796 |
| FR-053-AC-1 | TC-831 |
| FR-053-AC-2 | TC-832 |
| FR-053-AC-3 | TC-833 |
| FR-053-AC-4 | TC-834, TC-871, TC-890 |
| FR-053-AC-5 | TC-835 |
| FR-053-AC-6 | TC-836 |
| FR-053-AC-7 | TC-837 |
| FR-053-AC-8 | TC-838, TC-870 |
| FR-053-AC-9 | TC-839, TC-872 |
| FR-053-AC-10 | TC-840 |
| FR-053-AC-11 | TC-841 |
| FR-053-AC-12 | TC-842 |
| FR-053-AC-13 | TC-843 |
| FR-053-AC-14 | TC-873 |
| FR-054-AC-1 | TC-844 |
| FR-054-AC-2 | TC-845 |
| FR-054-AC-3 | TC-846 |
| FR-054-AC-4 | TC-847 |
| FR-054-AC-5 | TC-848 |
| FR-054-AC-6 | TC-849 |
| FR-054-AC-7 | TC-850 |
| FR-054-AC-8 | TC-851 |
| FR-054-AC-9 | TC-852 |
| FR-054-AC-10 | TC-853 |
| FR-054-AC-11 | TC-874, TC-875 |
| FR-054-AC-12 | TC-967 |
| FR-054-AC-13 | TC-968, TC-969 |
| FR-055-AC-1 | TC-854 |
| FR-055-AC-2 | TC-855 |
| FR-055-AC-3 | TC-856 |
| FR-055-AC-4 | TC-857 |
| FR-055-AC-5 | TC-858 |
| FR-055-AC-6 | TC-859, TC-947 |
| FR-055-AC-7 | TC-860 |
| FR-055-AC-8 | TC-1010 |
| FR-065-AC-1 | TC-1011 |
| FR-065-AC-2 | TC-1011 |
| FR-065-AC-3 | TC-1012, TC-1043 |
| FR-065-AC-4 | TC-1012 |
| FR-065-AC-5 | TC-1013 |
| FR-065-AC-6 | TC-1014 |
| FR-065-AC-7 | TC-1014 |
| FR-065-AC-8 | TC-1014 |
| FR-065-AC-9 | TC-1015 |
| FR-065-AC-10 | TC-1015 |
| FR-065-AC-11 | TC-1016 |
| FR-065-AC-12 | TC-1016 |
| FR-065-AC-13 | TC-1017 |
| FR-065-AC-14 | TC-1017 |
| FR-065-AC-15 | TC-1018 |
| FR-065-AC-16 | TC-1018 |
| FR-065-AC-17 | TC-1019 |
| FR-065-AC-18 | TC-1020 |
| FR-065-AC-19 | TC-1021 |
| FR-065-AC-20 | TC-1021 |
| FR-065-AC-21 | TC-1021 |
| FR-065-AC-22 | TC-1022 |
| FR-065-AC-23 | TC-1022 |
| FR-065-AC-24 | TC-1017 |
| FR-065-AC-25 | TC-1023 |
| FR-065-AC-26 | TC-1023 |
| FR-065-AC-27 | TC-1023 |
| FR-065-AC-28 | TC-1024 |
| FR-065-AC-29 | TC-1024 |
| FR-065-AC-30 | TC-1025 |
| FR-065-AC-31 | TC-1025 |
| FR-065-AC-32 | TC-1025 |
| FR-065-AC-33 | TC-1025 |
| FR-065-AC-34 | TC-1026 |
| FR-065-AC-35 | TC-1026 |
| FR-065-AC-36 | TC-1025 |
| FR-065-AC-37 | TC-1027 |
| FR-065-AC-38 | TC-1027 |
| FR-065-AC-39 | TC-1025 |
| FR-065-AC-40 | TC-1027 |
| FR-065-AC-41 | TC-1027 |
| FR-065-AC-42 | TC-1028 |
| FR-065-AC-43 | TC-1032 |
| FR-065-AC-44 | TC-1032 |
| FR-065-AC-45 | TC-1032 |
| FR-065-AC-46 | TC-1028 |
| FR-065-AC-47 | TC-1028 |
| FR-065-AC-48 | TC-1026 |
| FR-056-AC-1 | TC-861 |
| FR-056-AC-2 | TC-862 |
| FR-056-AC-3 | TC-863 |
| FR-056-AC-4 | TC-864 |
| FR-056-AC-5 | TC-865 |
| FR-056-AC-6 | TC-866 |
| FR-056-AC-7 | TC-867 |
| FR-056-AC-8 | TC-868 |
| FR-056-AC-9 | TC-869 |
| FR-056-AC-10 | TC-876 |
| FR-056-AC-11 | TC-877 |
| FR-056-AC-12 | TC-878 |
| FR-056-AC-13 | TC-879 |
| FR-057-AC-1 | TC-883 |
| FR-057-AC-2 | TC-883 |
| FR-057-AC-3 | TC-883 |
| FR-057-AC-4 | TC-884 |
| FR-057-AC-5 | TC-885 |
| FR-057-AC-6 | TC-889 |
| FR-057-AC-7 | TC-886 |
| FR-057-AC-8 | TC-886 |
| FR-057-AC-9 | TC-886 |
| FR-057-AC-10 | TC-887 |

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
| NFR-017-AC-4 | TC-815 |
| NFR-018-AC-1 | TC-504 |
| NFR-018-AC-2 | TC-505 |
| NFR-018-AC-3 | TC-504, TC-505 |
| NFR-018-AC-4 | (process AC; covered by P0-reproducer policy, parity with NFR-011-AC-4) |
| NFR-019-AC-1 | TC-579 |
**Mapping completeness (re-derived 2026-08-18, CR-074): every acceptance criterion stated in a requirement document is mapped to at least one TC except three, and all three are RETIRED** — `FR-012-AC-4` (parity-fixture regeneration), `NFR-001-AC-3` (render latency) and `NFR-012-AC-2` (Miri caching), retired with the render removal and ADR-0006 respectively. This measures **mapping**, not verification, and CR-058 rewords it because the bare "100%" was read as the second: of **573** TC rows, **276 are ✅, 272 are 🚧 and 25 are retired** (the ADR-0011 P2 wave-A slice adds 15 ACs and 18 TC rows, all ✅ on landing — CR-067 FR-026-AC-12/13 with TC-880..882, CR-068 FR-057-AC-1..10 with TC-883..889, CR-069 FR-022-AC-6/7 with TC-890..896, CR-071 FR-026-AC-14 with TC-897 — and CR-069 moves TC-024 from 🚧 to ✅, which was never a gap: the property had run on every `make ci` since the parser landed and simply carried no id) (the wave-B slice adds FR-058-AC-1..10 with TC-898..907, all ✅ on landing — CR-073 the check itself, CR-074 the load-time rejection of a declaration that cannot be executed, whose absence let `edges: []` report every document of a kind as an orphan) (the P1 slice adds 39: FR-053 13, FR-054 10, FR-055 7, FR-056 9 — all ✅ on landing) (CR-062 retired TC-825 with the `document:` form it pinned, and replaced TC-802 with TC-829 + TC-830) — every AC has a test case *written down*, and most of those test cases are not implemented. Performance criteria are outside this audit (see below). The figure to quote for verification is the `quire coverage` rollup over this repo, which counts backed trace ids rather than matrix rows. The byte-identity-gate slice (CR-057, 2026-08-15) adds FR-050-AC-20 (CR-045, CR-047 and CR-049 each rest their correctness argument on "the coverage JSON is byte-identical to the pre-program baseline" and **nothing backed it**: no checked-in baseline, no script, no make target, no CI step, and no gate in this repo has ever referenced the `spec-artifacts-process` corpus the claim was verified against by hand, once. TC-738 is not that gate — it runs the same engine twice over a synthetic fixture, which is determinism, and passes unchanged the moment the engine reconciles something different. A fixture corpus chosen to exercise the whole reconciliation surface at once now has its report checked in and byte-diffed on every run, with regeneration a deliberate reviewable act rather than a silent one, and a companion case that fails if the corpus stops exercising that surface — the failure mode of every baseline nobody re-reads. Building it surfaced agent-ix/quire-rs#124: `exclude:` scopes the declarations but not the CR-028 criteria walk, so fixture data still inflates the criteria denominator; the baseline pins that as it is, deliberately, so closing #124 fails the gate and the diff is read — TC-824) — 1 AC, closing agent-ix/quire-rs#114 (umbrella #106). The two-root identity slice (CR-056, 2026-08-15) adds no AC and closes the engine half of the two-root hardening: `extract_tree_excluding` compared the excluded subtree by **exact path**, so on a case-insensitive filesystem — macOS/APFS, the canonical perf runner — `<scope>/Spec` satisfied the caller's `scope.join("spec").is_dir()` check while the walk's `==` never matched it, the exclusion lapsed in silence, and every spec document was ingested a second time as source; a symlinked `spec/` failed the same way, resolving on one side only. Both sides now canonicalize, and only directories pay the syscall so the NFR-015 walk is untouched. `validate_bundle` also re-conflated the two roots one line after #99 split them (`let root = document_root;`), leaving the ambiguous name for future code to pick up by default — deleted. TC-814 covered only the `document:` half of FR-049-AC-9; the `exclude:` half is the more fragile one, since a lapsed exclusion *adds* ids while every reference still resolves, and it is now covered — TC-809 and TC-814 extended) — 0 ACs, the engine half of agent-ix/quire-rs#113 (umbrella #106). The glossary-pre-filter slice (CR-055, 2026-08-15) adds FR-044-AC-8 (a perf pre-filter that is stricter than the lookup it gates does not save work, it silently changes the answer: `has_glossary_heading` compared the raw heading title verbatim while `query::section` matches through `normalize_heading`, which treats ISO section numbering as decorative, so `## 3.2 Ubiquitous Language` and `### 4. Terms` — the standard ISO heading form — stopped contributing terms with no CR note, no AC and no test, shrinking the composed EARS lexicon until `vague-response` could fire on a repo's own domain nouns one indirection from the cause; the filter now normalizes exactly as the parsed heading does, block id included. CR-048 inverted the walk's silence on frontmatter-less markdown but missed the **second** consumer of the same membership rule: `glossary_terms_from_path` still skipped in silence, so a `spec/glossary.md` that lost its front block said nothing — it now reports the same `DocumentWithoutFrontmatter` diagnostic, and only for a file carrying a glossary heading, since every README is legitimately frontmatter-less. Two CR-046 leftovers go with it: `is_document` copied the whole body to answer a yes/no question, and `parse_one` ran a second copying extraction purely to recover the malformed flag the first one had — TC-823) — 1 AC, closing agent-ix/quire-rs#112 (umbrella #106). The fail-open-selection slice (CR-054, 2026-08-15) adds FR-050-AC-19 (CR-049 made body selection load-bearing on the declaration, so a declaration that selects nothing stopped being merely a quiet report and became an engine that parsed nothing: an `archetype:` typo, or a declared `document:` whose read failed — `.ok()?` swallowed every IO error, the exact class CR-045 cost 123 findings to — produced `total: 0`, exit 0 and no diagnostic, caught only by the off-by-default `--strict` empty-denominator guard. `CoverageReport.diagnostics` and matching `quire validate` warnings name the declaration and the cause, from one shared reason vocabulary so the two commands cannot disagree; an unreadable document is reported against every declaration naming it, while a missing archetype is reported only when the model minted nothing at all, since a model legitimately declares archetypes an individual repo has no instance of and reporting each would be noise on every healthy repo. `harvest` also re-read and re-parsed its document once per declaration — `spec/tests.md`, typically the largest file in the repo, read once per trace target *and* once per document reference — so the scan now caches per path. TC-818's fixture could not falsify the filename claim it was pinning, its undeclared documents being named unlike the declared ones; it gains an undeclared type in a file named `FR-002.md` — TC-822, TC-818 corrected) — 1 AC, closing agent-ix/quire-rs#111 (umbrella #106). The mitigation-enforcement slice (CR-053, 2026-08-15) adds no AC and makes FR-024-AC-9's three compensating controls real: `check_no_shared_mutable.sh` was missing from the ci.yml `audit-static` job (6 of the Makefile's 7 scripts were listed) and `make sanitize` was missing from `make hardening`, so the TSAN lane backing TC-816 was in no default set at all; the audit itself matched exemptions by **basename** (any future `**/body_cache.rs` inherited one) and by **substring** (every present and future `OnceLock` in `declared_tables.rs` was exempt, a genuinely shared static included), never noticed a stale entry, never printed the `why` it parsed, and missed `LazyLock`/`once_cell::sync::Lazy`/`Cell`/`RefCell`/`thread_local!`/`static mut`/`unsafe impl Sync`; its scope excluded `src/python`, the one module that opens a rayon region over corpus state. TC-502 was a phantom — a comment in the script header, listed in this matrix as 🚧 — and now names the script as FR-024-AC-9's enforcement identity. TC-816 widens past 2 threads × 1 document to 8 × 16 and covers the rayon-forcing shape `python::load_repo` runs, since TC-815 models the once-cell contract with loom primitives and can say nothing about std's `OnceLock`. The `body_cache.rs` invariant comment claimed the cell is never touched from a parallel region; it is, by that binding, and the comment now says so — 0 ACs, closing agent-ix/quire-rs#109 (umbrella #106). The parser byte-identity slice (CR-052, 2026-08-15) adds FR-005-AC-8 (CR-046's "outputs are unchanged" was pinned by nothing: AC-6's proptest compares the two tiers *after* the split funnelled both into one `parse_body_at` with `body_offset` computed by the identical expression, so it exercises `Some(map)` vs `Some(map.clone())` and compares by `PartialEq`, not bytes — a refactor cannot be its own reference; a checked-in golden corpus is snapshotted from the engine at `7b1db82`, the commit before CR-046, and the current engine reproduces it byte-for-byte, so the claim now has evidence; the snapshot is a fixed reference, never regenerated to make a test pass, and AC-6/TC-813 are rescoped to the composition statement they actually make — TC-821) — 1 AC, closing agent-ix/quire-rs#108 (umbrella #106). The frontmatter-warning machine-surface slice (CR-051, 2026-08-15) adds FR-024-AC-12 (the CR-048 walk→bundle bridge shipped with no owning criterion, so nothing pinned it: `validate_bundle` turns each `DocumentWithoutFrontmatter` diagnostic into exactly one `BundleReport` warning naming the path, in both postures, never an error and never moving `is_valid()`; the two flavors now carry **distinct** machine reasons — `no-frontmatter` for an absent block, `malformed-frontmatter` for a block that is not a YAML mapping — where the bridge previously dropped the diagnostic's `malformed: bool` and tagged both identically, making "someone wrote a front block and it does not parse" indistinguishable from "this file was never meant to be a document" — TC-820) — 1 AC, the engine half of agent-ix/quire-rs#110 (umbrella #106). The `parse_body` totality slice (CR-050, 2026-08-15) adds FR-005-AC-7 (CR-046 stated FR-005's purity clause against `parse_document` alone, but `Header` carries a private byte offset into the input it was parsed from and cannot borrow from it — it is stored beside an owned text on `LoadedDocument` — so `parse_body(other, &header)` is constructible from safe public API and sliced that offset unchecked, panicking out-of-bounds on a shorter string and on a char boundary inside a multi-byte character; the offset is re-derived from the string actually given, one `is_char_boundary` on the correct path, and an in-bounds on-boundary mismatch stays undetectable and so stays the caller's contract — TC-819) — 1 AC, closing agent-ix/quire-rs#107 (umbrella #106). The declaration-driven-selection slice (CR-049, 2026-08-15) adds FR-050-AC-18 (the `traceability:` model bounds what is *parsed*, not just what is reported: during coverage a corpus document whose archetype no trace target, document reference, or grammar binding names keeps its body unmaterialised — selection decided on the header tier, never by filename, `exclude:` globs applying after selection, a declared archetype without its declared section legally minting nothing — while the report stays byte-identical to a full-parse engine, AC-7 being the whole gate; depth is emergent from CR-047's first-touch semantics, no new API and no mode flag, and FR-025 states the caller-declared-depth reading — TC-818, with TC-738 pinning byte-identity) — 1 AC, closing agent-ix/quire-rs#94 (umbrella #90, its final engine child). The silence-inversion slice (CR-048, 2026-08-15) rewrites FR-024-AC-10 in place — no AC added or removed (CR-044's "produce no diagnostic" assertion inverts: a frontmatter-less `.md` under the walked root emits exactly one non-fatal `DocumentWithoutFrontmatter` warning naming its path, the malformed-block flavor distinguished via the FR-006 status; the silence was justified only by tolerating a repo-root walk, which CR-045 removed, so what remains inside `spec/` is almost certainly an authoring mistake and silence made it a real error nobody ever saw; never re-suppressed by filename — a name list is exactly what CR-044 deleted; `validate_bundle` bridges the walk diagnostic into `BundleReport` warnings, reason `no-frontmatter`, in both postures; TC-807's `README.md`/`CHANGELOG.md` fixtures move outside the document root, never visited rather than tolerated — TC-807 updated) — 0 ACs, closing agent-ix/quire-rs#95 (umbrella #90). The lazy-body slice (CR-047, 2026-08-15) adds FR-025-AC-7..8 and NFR-017-AC-4 (the corpus gains its two-tier document model: headers — path/id/uuid/full frontmatter map/verbatim text — eager at construction, bodies materialised on first `body()` access exactly once behind a per-document once-cell in `Arc<SpecInner>`, concurrent first accessors receiving the identical value and materialisation reading no filesystem — extended TC-485; `len`/`by_id`/`by_type`/`diagnostics` and the FR-026/027 edge queries complete with zero body parses since the walk parses headers only per CR-046 and resolution reads frontmatter + raw text alone — TC-817; FR-024-AC-9 is **narrowed to the walk fan-out and stated that way** rather than deleted, with `check_no_shared_mutable.sh`'s pattern *widened* to also catch `OnceLock`/`OnceCell` and every hit either failing or carried on a named `file|match-substring|why` exemption list — the pre-existing `declared_tables.rs` compile-once regexes become visible instead of silent; the concurrency risk the blanket ban stood in for moves to explicit coverage, a loom first-touch permutation modeling the std `OnceLock` contract loom cannot instrument — TC-815 — plus the real primitive raced under the NFR-018 TSAN lane — TC-816 — and NFR-017's shuttle-reconsideration clause is resolved: one cell × two threads is loom's sweet spot, shuttle stays not adopted; `LoadedDocument.doc` is replaced by the `raw()`/`frontmatter()`/`concept_type()`/`body()`/`from_parsed()` accessor surface) — 3 ACs, closing agent-ix/quire-rs#93 (umbrella #90). The validate two-root slice (CR-045 follow-through, 2026-08-15) adds FR-049-AC-9 (`validate_bundle` receives the document root and the reference root separately — model-declared `document:`/`exclude:` paths are authored against the repository scope, so a corpus walked from `<scope>/spec` with a single conflated root silently un-minted every path-bound trace target, measured as 123 new `dangling-trace-reference` findings on this repo's own spec during the #91 CLI derivation; `validate_bundle_at` keeps single-root semantics for self-contained bundles — TC-814) — 1 AC. The header/body-tier slice (CR-046, 2026-08-15) adds FR-005-AC-5..6 (`parse_document` splits into a cheap header tier and an expensive body tier: `parse_header` decides membership and identity — `id`/`type`/`uuid` plus the **full** frontmatter map, so resolution and validation can read frontmatter without a body parse — in one extraction with no body work and no input copy, `parse_body` runs the body pipeline under that header, and `parse_document` composes them with unchanged signature and semantics; `walk::parse_one` now membership-checks via `parse_header`, retiring `read_identity` and the CR-044 duplicate `extract_frontmatter` that ran after the full parse, so a non-document costs one read and one failed fence check instead of a full parse discarded — TC-812, TC-813 with tier composition pinned by proptest over arbitrary UTF-8) — 2 ACs, closing agent-ix/quire-rs#92 (umbrella #90), the enabler for the FR-025 lazy body tier (#93). The two-roots slice (CR-045, 2026-08-15) adds FR-050-AC-17 (`quire coverage` derives a document root `<scope>/spec` and a code root `<scope>` from one `--scope` and never interchanges them: the corpus walk is bounded by the document root — FR-024 gains the bounding clause — the code walk excludes the document root via `extract_tree_excluding`, a scope with no `spec/` exits with a diagnostic naming the missing root, and the minted-id set over a compliant repo is byte-identical because `--scope` stays the relativization base; this is the traversal bug CR-044's membership rule was silently tolerating — the 9,172 `required 'type' is missing` errors across 223 repos are gone because repo-root files are never visited, not because they were classified away — TC-809 ✅, TC-810/TC-811 with the `quire-cli` two-root derivation) — 1 AC, the engine half of agent-ix/quire-rs#91 (umbrella #90). The type-driven-membership slice (CR-044, 2026-08-15) adds FR-024-AC-10 (corpus membership decided by the presence of a frontmatter block, never by filename: `DEFAULT_SKIP` and `WalkOptions::skip_names` are deleted, and a markdown file with no frontmatter is silently not a document — the rule that retires the `README.md` entry and generalizes to every stray `.md`. The constant was a **graph-ingestion** filter in `filament_parser/loader.py`, where it meant "not a graph node"; copied into this validation loader it became "not a document", so the engine could not load the canonical instance of `TestMatrix`, a type its own module registers. **[RAN]** `scripts/classify_matrices.py` over `~/dev`: of 184 matrices at a bound path, **0 lack a frontmatter block**, 170 stay, 14 are mis-typed (10 declaring `type: index`) of which 6 mint rows today; 20 real matrices in 9 repos become visible for the first time, 12 of them minting. `NON_ARTIFACT_FILES` drops to `{index.md, log.md}` in the same slice, so an index omitting its matrix now reports `index-incomplete` — 172 of 180 repos do. FR-024-AC-11 pins the second consumer of the rule: `glossary_terms_from_path` scans raw text rather than building a `Spec`, and inherited the skip through `discover_files`, so its scope would have widened in silence to every stray `.md` — TC-807, TC-808) — 2 ACs, closing agent-ix/quire-rs#63, #73, #76 and #77. The legacy-list slice (CR-043, 2026-08-14) adds FR-051-AC-16 (a legacy textual form mints one relation per id its match carries, so a comma-separated list binds every id instead of only the first — **[RAN]** 98 such lines across `~/dev` were dropping **205 ids in 17 repos**, spanning every declared legacy shape and all three languages, and all 15 of quoin's status lies had this one cause; the engine half alone is insufficient, contrary to the filing, since capture group 1 is already a single id, so the declared patterns widen their id group in `spec-artifacts-process` and the engine splits it where `marker_ids` already does; `id_format` forms render one id and are not split — TC-806) — 1 AC, closing agent-ix/quire-rs#68. The block-model backfill (CR-042, 2026-08-14) authors [FR-019](./functional/FR-019-stable-block-ids.md), [FR-020](./functional/FR-020-block-addressing.md) and [FR-022](./functional/FR-022-writeback-primitives.md) — 10 ACs for behaviour that shipped in v0.2 and was never written up, so this matrix carried 16 rows against requirements with no document. The criteria are read off working code (`strip_trailing_block_id`, `find_block_by_id`, `Registry::block_type`, the eight `writeback.rs` cases), not proposed. **FR-021 is retired**: `apply_block_patch`/`replace_block` were render-dependent and went with render, which is why 10 of those rows claimed an API the crate does not export — US-006/US-007's ACs were already retired for the same reason. Closes the group-(b) half of agent-ix/quire-rs#60. The no-source-symbol slice (CR-041, 2026-08-14) adds FR-050-AC-16 (a module declares which test-type values mint no source symbol, so an unbacked row carrying one is reported as a no-symbol row rather than a status lie — 40 of quoin's 55 lies are agent-behaviour evals whose method can never produce a symbol to tag, against exactly one such row here, which is why the gap was invisible from this repo; the exemption changes the verdict and never the facts, and an undeclared vocabulary serializes byte-identically — TC-805) — 1 AC, answering agent-ix/quoin#65. The Rust-lexer slice (CR-040, 2026-08-13) adds FR-051-AC-15 (raw strings, lifetimes, character literals and nested block comments recognized, so a brace inside any of them neither moves the depth nor rejects the file — **[RAN]** 33 of this repo's own source files were being rejected and yielding zero symbols, which alone accounted for 78 of the 140 status lies in agent-ix/quire-rs#60: 144/907 → 306/907 backed with no matrix edit — TC-804) — 1 AC. The single-lexer-pass slice (CR-039, 2026-08-13) adds FR-051-AC-14 (comment/string/template state derived once per file and read by every consumer, instead of three functions each re-deriving it — the structural cause behind both CR-036 and CR-037, each of which cost a whole file's symbols in silence; the `block_end` restart was measured **unreachable** through `parse`, since every declaration matcher is `^`-anchored, so this removes the hazard rather than a live defect — TC-803) — 1 AC, closing agent-ix/quire-rs#62. The declared-path-scoping slice (CR-038, 2026-08-13) adds FR-050-AC-15 (a trace target or document reference may declare `exclude:` globs and may name `archetype` and `document` together — scanning `spec-artifacts-process` by archetype minted 67 test-case ids from deliberately malformed fixtures and read 50 of them as backed, because a fixture reusing `TC-017` collides with the real one; which paths hold test data stays module-declared, and `exclude` is absent-by-default so FR-050-AC-7 byte-identity is untouched — TC-801, TC-802) — 1 AC, closing agent-ix/quire-rs#61; the remaining `DEFAULT_SKIP` question it exposes is split to agent-ix/quire-rs#63. The wrapped-signature slice (CR-037, 2026-08-13) adds FR-051-AC-13 (a `def` a formatter wrapped across lines still spans its docstring — the closing `) -> None:` dedents to the declaration's own column, which the indent rule read as the end of the suite, cutting the span one line before the tag; found by running `gap-analysis` over `spec-artifacts-process`, where two tests differing only in wrapping bound differently — TC-800) — 1 AC. The string-aware-stripping slice (CR-036, 2026-08-13) adds FR-051-AC-12 (a `//` or `/*` inside a string or template literal is content, not a comment opener — one git refspec in a template literal made `check_balanced` reject a valid file, which under CON-2 means zero symbols and every trace tag in it silently bound to nothing; carried across lines because the corpus writes the refspec on a *continuation* line of a multi-line template, where the first, per-line fix still re-opened the comment — TC-798, TC-799) — 1 AC, found while fixing agent-ix/quoin#61 and completed by reviewing that fix. The empty-denominator slice (CR-035, 2026-08-13) adds FR-050-AC-14 (a declared model matching zero rows is reported distinctly from full coverage and fails `--strict`, instead of rendering as `100%` and exiting 0 — the vacuous pass that hid the ecosystem-wide `trace_tags` gap for nine days; fixed in `quire-cli`, no `CoverageReport` field and so no FR-050-AC-7 impact — TC-797) — 1 AC, closing agent-ix/quire-rs#58. The candidate-extraction slice (CR-033, 2026-08-08) adds FR-052-AC-16..17 (a closed three-valued `extraction` outcome — `extractable` | `candidate` | `not-extractable` — answering what `{property: <metamorphic>, extractable: false}` means to a consumer, with `extractable` and CON-4 untouched and `candidate` review-gated by construction so a module-declared idiom can raise a criterion for attention without ever entering an unattended generation set — TC-795, TC-796) — 2 ACs, closing agent-ix/quire-rs#46. The promotion-dogfood slice (CR-031, 2026-08-08) adds FR-048-AC-11 (this repo's own `spec/` is judged under the severity promotion its published module ships, with the mirrored promotion verified against a real module checkout when one is reachable — TC-794) — 1 AC, added after two `ac:non-singular` errors reported by a **stale** `quire` CLI (quire-rs v0.16.0, pre-CR-024/026) were mistaken for a live checker defect; the engine on `main` emits none, and nothing in CI could have contradicted either reading. The property-recall outcome slice (CR-030, 2026-08-07) adds FR-052-AC-14..15 (the universal determiner read at two further bounded subject positions — a fronted subordinate clause's subject and a determiner-headed main subject after the comma that closes fronted material — with refusal wherever the subject cannot be bounded, and byte-identity of every classification the widening does not claim together with the whole `ac` finding stream — TC-792, TC-793), the one of three candidate widenings that cleared the ≥85% precision gate fixed in advance; the other two were deleted rather than narrowed. The acceptance-criteria property-classification slice (CR-028, 2026-08-07) adds FR-052-AC-1..13 (a second, orthogonal shape axis over the same `ac` binding: a closed property-shape enum under one fixed precedence, `{domain, precondition, oracle}` spans that are statement-relative and carry both byte offsets and their own text, `row_id` and a `signals` audit trail on each record, `extractable` derived in one place, and a module `property_idioms` registry demoted to a booster so CON-4 keeps extraction coverage independent of it — TC-779..791) and FR-050-AC-13 (`CoverageReport.criteria` plus two `CoverageTotals` counts, empty and byte-identical on a corpus carrying no criteria — TC-788) — 14 ACs, specified ahead of the engine work on agent-ix/quire-rs#20 and implemented in the same Phase B stack; TC-779..791 are ✅, the Rust cases in `src/grammar/property.rs` and `tests/coverage_rollup.rs` and the PyO3 parity case in `tests/python/test_bindings.py`. The `ac` promotion-readiness slice (CR-024, CR-025, CR-026, 2026-08-07) adds FR-047-AC-15..18 (the positive/negative pair idiom recognized by its second obligation rather than by a separator; `Then` counted only in a modal-free Given/When/Then criterion; a vacuous predicate that is also a common noun qualified so it does not fire on the noun; CommonMark backtick-run matching so a double-tick span masks what it quotes — TC-775..778) — 4 ACs, found by the FR-047-CON-1 corpus baseline sweep, which measured `non-singular` firing on 23 singular criteria out of 48. The AC-grammar/traceability-coverage slice (FR-047..FR-051, US-017, 2026-08-04) adds FR-047-AC-1..14 (acceptance-criteria grammar `ac`: assertion-canonical shape classification with obligation/GWT recognized-but-steered via `non-canonical-shape` (CR-013; EARS was the original canon), every-cell segmentation, and the five shipped checks — `unclassifiable` (structural: no predicate at all), `non-singular`, lexicon-backed `vague-response`, `vacuous-outcome` (a closed, module-extensible `vacuous_predicates` set suppressed by any concrete signal, lexicon term, or declared observable verb) and `non-canonical-shape` (CR-014; `observable_verbs` keeps its ADR-0009 module-data role, demoted from a membership test to a suppressor); binding, fenced/blockquote skip in supplements, mention-vs-use masking (CR-017), elided-copula predication (CR-019), generic `[<grammar>:<check>]` --summary; CON-1 gates error-promotion behind a corpus baseline sweep + user sign-off — TC-707..715, TC-751, TC-754, TC-757, TC-761, TC-763), FR-048-AC-1..10 (per-check `grammar_severity` registry over `off`|`warning`|`error` + `--severity` CLI override incl. repeatable form and malformed-entry rejection, first-wins merge, type-only all-default, `off` full suppression — TC-716..723, TC-752, TC-755, TC-766), FR-049-AC-1..8 (model-driven verification-reference integrity, `dangling-trace-reference`, posture degradation, auxiliary trace-source harvest — TC-724..731), FR-050-AC-1..12 (declarative `traceability:` model + generic `quire coverage` rollup: unbacked rows, status lies, untracked symbols, per-group counts, byte-identical output; CR-015 adds the leading-marker status class, declared column vocabularies, and default-off range/annotation normalization — TC-732..740, TC-758..760), and FR-051-AC-1..11 (source-symbol extraction with stable identities; framework-native markers — pytest marker / Rust `#[trace]` attribute / TS `trace()` helper — as the canonical statically-parsed trace form with the textual forms as a sunset-gated legacy class (CON-3); `verifies`/`defined_in`/`contains` relations, FR-045-shaped records — TC-741..750, TC-753; the FR-050-CON-2/FR-051-CON-1 purity constraints are backed by the TC-756 static boundary audit, TC-690 pattern) — 55 ACs. Implementation landed 2026-08-04/05 (Plan-001 Tracks A and B, gates G1/G2 passed, amended by CR-013/CR-014/CR-015): every TC is ✅ except the five stated at the `quire validate` / `quire coverage` **command** level (TC-714, TC-720, TC-721, TC-740, TC-755, awaiting EXT-3 `quire-cli`). The canonical Filament extraction slice (FR-045/FR-046/NFR-020, US-016) adds FR-045-AC-1..6, FR-046-AC-1..4, NFR-020-AC-1..3 (14 ACs incl. FR-006-AC-7 frontmatter status, CR-011), covered by TC-681..706 + TC-767..003. The project-glossary slice (FR-044, 2026-06-23) adds FR-044-AC-1..7 (a repo's authored Ubiquitous-Language terms — a `Glossary` `## Terms` table + `## Ubiquitous Language` bullets — are harvested and composed with the module lexicon into an ad-hoc `GrammarLexicon` injected via `validate_document_in_registry_with_lexicon`; the corpus `validate_bundle` applies it per doc; advisory and a no-op when no glossary exists — TC-674..680) — 7 ACs. The module-lexicon slice (FR-043, ADR 0009, 2026-06-23) adds FR-043-AC-1..7 (modules ship a mergeable `lexicon:` registry the EARS object-aware vague-response check consumes; the engine drops its hardcoded concrete-noun list; the type-only path degrades to an empty lexicon; PyO3 `check_grammar` gains `module_root` — TC-667..673) — 7 ACs. The requirement-grammar slice (FR-042, EARS, 2026-06-22) adds FR-042-AC-1..10 (grammar-check framework with EARS as the first grammar: six-pattern classification, the non-singular/missing-subject/vague-response/non-canonical-trigger clause checks with per-archetype dialects, warning→error severity routing into `ValidationResult`, fenced/quote/reference skip, and PyO3 parity — TC-657..666) — 10 ACs. The authorable-inverse-edges slice (FR-041, ADR 0008, 2026-06-21) adds FR-041-AC-1..5 (declared `inverse:` labels become authorable as derived views of their forward edge: inverse index, Tier-1 recognition, precedence/`DuplicateInverseEdge`, Tier-2 forward normalization, warn-only determinism, TC-652..656) — 5 ACs. The object-edge-vocabulary slice (FR-040, 2026-06-20) adds FR-040-AC-1..11 (object-axis typed edge vocabulary + cross-domain role-typed targets: mergeable `edge_types`/`roles` registries with first-wins+diagnostic merge, object `roles` parsed onto the archetype, array|map `allowed_links`, union resolution, warn-tier Tier-1/Tier-2 validation, composed skeleton, TC-636..645 + TC-650/651) and US-015-AC-1..4 (author declares an object's relationship vocabulary, TC-646..649) — 15 ACs. The per-value assert slice (CR-010, 2026-06-20) adds FR-033-AC-11..13 (`choices` scalar enum + `column_choices`/`column_patterns` per-column table validation, TC-633..635) — 3 ACs. The internal-links slice (ADR 0007, 2026-06-17) adds FR-026-AC-9..11 (relative-path link edge source + index/log exclusion + dedup parity, TC-620..622) and FR-039-AC-1..10 (unlinked-reference detection & autofix suggestions, incl. AC-10 multi-token code-span skip, TC-623..632) — 13 ACs. The composed type+object validation slice (2026-06-16) adds FR-032-AC-11..13 (`validate_document_in_registry` composes the `type` archetype with the frontmatter `object:` archetype; resolved-object failures are errors, unknown-object is a warning, `ValidationResult` carries typed `warnings`) — TC-610..613, 3 ACs. The assert/lint extension slice (2026-06-16) adds FR-033-AC-10 (CR-008 `matches` content assert, TC-608) and FR-036-AC-6 (CR-009 `section_body_pattern` lint rule, TC-609) — 2 ACs. The binding-contract slice (CR-020, 2026-08-06) adds FR-036-AC-7 (`forbidden_section` lint rule — TC-764) and FR-039-AC-11 (`-VC-` sub-id kind in `parent_id` and the token regex — TC-765) — 2 ACs. The OKF slice (2026-06-16) adds FR-037-AC-1..6 (base concept frontmatter schema, TC-590..596 + TC-528) and FR-038-AC-1..8 (OKF bundle validation, TC-600..607) — 14 ACs. v0.4 adds FR-011-AC-21 (CR-006 `multiple: true`, TC-583) and FR-036-AC-1..5 (declarative lint rules, TC-584..588). v0.2 block model added 16 ACs (FR-019..022, TC-400..443). v0.3 adds 81 ACs — StR-005/006, US-011..013, FR-023..027 (incl. review-added FR-026-AC-8, FR-027-AC-9), NFR-015/016, plus the hardening re-review (NFR-003-AC-4, FR-024-AC-9, NFR-017, NFR-018) — covered by TC-455..507 (plus reused TC-456..459). The Miri ACs (NFR-012-AC-1..5) were **retired** (ADR 0006) and the compile-time **NFR-003-AC-5** (`forbid(unsafe_code)`, TC-582) added. PC (performance criteria) for US-011..013 are tracked as benches (TC-455..459, TC-469) and marked 🚧 pending implementation, consistent with the US-006..010 perf-bench convention. The v0.3 hardening re-review (loom NFR-017, TSAN/ASAN NFR-018) is recorded in spec.md §19.

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

**Integrity check (grep-verified):** all **471 distinct file-defined ACs** (definition-anchored: a bold `**<ID>-AC-N**` bullet declaration **or** an `| <ID>-AC-N |` leading cell in a `## Acceptance Criteria` table — both are definitions; the table form became the majority when the NFR sections were converted to the required table shape for spec-artifacts-iso#11) across `stakeholder/ usecase/ functional/ non-functional/` appear in the AC→TC audit table — **0 uncovered**. Note: `FR-900-AC-1/2` appearing inside FR-034-AC-1's example prose are NOT defined ACs and are excluded from the denominator (match `**…**:` definitions, not inline mentions). Retired ACs (marked `(RETIRED)`, un-bolded) are excluded by construction. Count: 316 (pre-removal) − 41 (retired) + 16 (back-fill) + 1 (FR-011-AC-20, CR-005 heading normalization) − 5 (NFR-012-AC-1..5 retired, ADR 0006) + 1 (NFR-003-AC-5, forbid(unsafe_code)) + 1 (FR-011-AC-21, CR-006 multiple:true) + 5 (FR-036-AC-1..5, declarative lint rules) + 1 (FR-010-AC-4, CR-007 escaped pipes) + 6 (FR-037-AC-1..6, OKF base concept schema) + 8 (FR-038-AC-1..8, OKF bundle validation) + 1 (FR-033-AC-10, CR-008 `matches` content assert) + 1 (FR-036-AC-6, CR-009 `section_body_pattern`) + 3 (FR-032-AC-11..13, composed type+object validation) + 3 (FR-026-AC-9..11, relative-path link edge source) + 10 (FR-039-AC-1..10, unlinked-reference detection incl. multi-token code-span skip) + 3 (FR-033-AC-11..13, CR-010 per-value enum/regex asserts) + 11 (FR-040-AC-1..11, object-axis typed edge vocabulary + cross-domain targets) + 4 (US-015-AC-1..4, author declares object relationship vocabulary) + 5 (FR-041-AC-1..5, authorable inverse edge verbs) + 10 (FR-042-AC-1..10, requirement-grammar check (EARS)) + 7 (FR-043-AC-1..7, module-supplied concrete lexicon) + 7 (FR-044-AC-1..7, project Ubiquitous-Language lexicon) + 13 (FR-045-AC-1..6 + FR-046-AC-1..4 + NFR-020-AC-1..3, canonical Filament extraction) + 1 (FR-006-AC-7, frontmatter status, CR-011) + 14 (FR-047-AC-1..14, acceptance-criteria grammar incl. non-canonical-shape, supplement skip, module-data observable verbs, CR-017 quoted-keyword masking, CR-019 elided-copula predication) + 10 (FR-048-AC-1..10, per-check grammar severity incl. `off` + malformed CLI-entry rejection) + 8 (FR-049-AC-1..8, verification-reference integrity) + 12 (FR-050-AC-1..12, declarative coverage computation incl. CR-015) + 11 (FR-051-AC-1..11, source symbol extraction with relations incl. canonical markers + legacy class) + 1 (NFR-006-AC-5, sorted module discovery, CR-018) + 1 (FR-036-AC-7, `forbidden_section` lint rule, CR-020) + 1 (FR-039-AC-11, `-VC-` sub-id kind, CR-020) + 1 (FR-048-AC-11, own `spec/` dogfooded against the shipped severity promotion, CR-031) = **447**. **This addition chain is retired (CR-032, 2026-08-08).** It was kept by hand, so every slice that landed had to be added to it by hand and four never were; it read 447 where the anchor rule above counts **471** at the same commit. The figure of record is the mechanical count, not the chain: match `**<ID>-AC-N**` bold declarations and leading `| <ID>-AC-N |` table cells across `spec/{stakeholder,usecase,functional,non-functional}`, discard any whose own criterion text is marked RETIRED, and count distinct ids. Recount that way rather than extending the chain. (The `FR-019..FR-022` rows once had no source document — the v0.2 block-model FRs were never written as artifacts. CR-042 authored FR-019, FR-020 and FR-022 against the shipped behaviour and retired FR-021 with the render removal, so their ACs are now defined and counted like any other.)

---

## Coverage Gaps

| Gap ID | Description | Risk Level | Mitigation |
|--------|-------------|------------|------------|
| GAP-001 | DSL evaluator parity test (TC-040) needs a curated fixture document per object_type across all 87+ types; some fixtures may not yet exist in the source repos. | Medium | Track per-type fixture availability in `tests/extract_parity/coverage.md`; missing fixtures are P1 follow-ups. |
| GAP-002 | Python Jinja2 reference renderer is not byte-stable across Jinja2 minor versions in all whitespace cases. | Low | StR-002-AC-2 documents known whitespace exceptions; pin reference's Jinja2 version. |
| GAP-003 | Cross-machine determinism (arm64 vs x86_64 byte parity) is implied but not explicitly benched. | Low | Add an arm64 + x86_64 CI matrix as a P2 enhancement. |
| GAP-006 | The 22 `StR-NNN-VC-N` stakeholder validation criteria introduced by the spec-artifacts-iso#11 table conversion are **not traced to any Test Case**. Giving StR criteria stable ids is precisely what makes them traceable (previously they were prose and unaddressable), so this gap is newly *expressible*, not newly created — but it is real and should not read as covered. The `471 / 471` figure above counts **Acceptance** Criteria only; Validation Criteria are a distinct kind and are outside that denominator. The single `StR-001-VC-2` occurrence in this file is TC-765's example prose, not a trace. | Medium | Allocate TCs for StR VC rows in the next matrix pass, or record explicitly that stakeholder validation is evidenced by Demonstration outside the TC matrix. Tracked on agent-ix/spec-artifacts-iso#11. |
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
