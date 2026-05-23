# Test Matrix

## Overview

This matrix maps every Acceptance Criterion in `~/dev/quire-rs/spec/` to one or more Test Cases. Coverage status reflects intent (DRAFT) — implementation tasks are tracked separately via `/spec-to-plan`.

## Test Matrix Rules

1. **Coverage Rule**: Every acceptance criterion (AC) has at least one test case.
2. **Option Permutation Rule**: Each archetype's render path is exercised independently.
3. **Constraint Boundary Rule**: Schema constraints (pattern, min-length, enum) are tested at boundary values.
4. **Error Path Rule**: Every `QuireError` variant has at least one negative test.
5. **State Transition Rule**: Not applicable — `quire-rs` is stateless beyond the `Renderer` lifecycle.
6. **Edge Case Rule**: Parser edge cases (unclosed fence, malformed YAML, level skips, empty input) each have dedicated TCs.

---

## Requirements Traceability

### Stakeholder Requirement Coverage

| Stakeholder Req | Trace to US/FR | Test/Validation | Coverage Status |
|-----------------|----------------|-----------------|-----------------|
| StR-001 Single Rust engine | US-001, US-002, US-003, US-004, FR-001, FR-005, all FRs | TC-001, TC-002, TC-040 | ✅ Complete |
| StR-002 Render parity | US-005, FR-012, NFR-006 | TC-030 thru TC-039 (one per archetype) | ✅ Complete |
| StR-003 Parse parity | US-002, FR-005..010, NFR-006 | TC-020 thru TC-029 | ✅ Complete |
| StR-004 Safety scaffolding | NFR-003, NFR-004 | TC-050, TC-051 | ✅ Complete |

### User Story Coverage

| User Story | Acceptance Criteria | Test Cases | Coverage Status |
|------------|---------------------|------------|-----------------|
| US-001 LLM emits validated patch | AC-1, AC-2, AC-3 | TC-003, TC-031, TC-004 | ✅ Complete |
| US-002 Developer parses spec doc | AC-1, AC-2, AC-3 | TC-001, TC-029, TC-002 | ✅ Complete |
| US-003 Extractor evaluates DSL | AC-1, AC-2, AC-3 | TC-018, TC-019, TC-040 | ✅ Complete |
| US-004 Editor patch + render | AC-1, AC-2, AC-3 | TC-005, TC-007, TC-042 | ✅ Complete |
| US-005 CI detects regression | AC-1, AC-2, AC-3, AC-4 | TC-030, TC-031, TC-031, TC-041 | ✅ Complete |

### Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Coverage Status |
|----------------|---------------------|------------|-----------------|
| FR-001 Render dispatch | AC-1..5 | TC-003, TC-004 (unknown type), TC-006 (schema viol), TC-005, TC-008 (concurrency) | ✅ Complete |
| FR-002 Merge-then-validate | AC-1, AC-2, AC-3 | TC-007 | ✅ Complete |
| FR-003 schemars derivation | AC-1..4 | TC-009 | ✅ Complete |
| FR-004 Strict MiniJinja env | AC-1, AC-2, AC-3 | TC-010, TC-008, TC-011 (env bench) | ✅ Complete |
| FR-005 parse_document API | AC-1..4 | TC-001, TC-029 | ✅ Complete |
| FR-006 Frontmatter fallback | AC-1..4 | TC-012, TC-013 (malformed), TC-014 (unterminated) | ✅ Complete |
| FR-007 Fenced-block walk | AC-1..4 | TC-015 (backtick), TC-016 (unclosed), TC-017 (tilde) | ✅ Complete |
| FR-008 Byte-exact slicing | AC-1, AC-2, AC-3 | TC-022, TC-023, TC-024 (proptest roundtrip) | ✅ Complete |
| FR-009 Slug-line ID | AC-1..5 | TC-025, TC-026 | ✅ Complete |
| FR-010 Query API | AC-1, AC-2, AC-3 | TC-027 (compile), TC-028 (parity sweep), TC-029 (complexity) | ✅ Complete |
| FR-011 Body-extraction DSL | AC-1..4 | TC-018, TC-019, TC-040, TC-041 | ✅ Complete |
| FR-012 Archetype parity suite | AC-1..4 | TC-030 thru TC-039 (one per archetype), TC-041 (regression) | ✅ Complete |

### Non-Functional Requirement Coverage

| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
|--------------------|---------------------|---------------------|--------|
| NFR-001 Render <1ms | criterion bench (median) | TC-042 thru TC-051 (one bench per archetype), regression gate | ✅ Complete |
| NFR-002 Parse 5MB <500ms | criterion bench (median) | TC-052 (bench), TC-053 (roundtrip correctness) | ✅ Complete |
| NFR-003 Zero unsafe | static check (audit-unsafe) | TC-050 | ✅ Complete |
| NFR-004 License hygiene | cargo deny check licenses | TC-051 | ✅ Complete |
| NFR-005 Actionable errors | unit + snapshot | TC-006, TC-054 (display shape), TC-055 (snapshot) | ✅ Complete |
| NFR-006 Determinism | proptest (render + parse 100x) | TC-056, TC-057, TC-058 (clippy HashMap audit) | ✅ Complete |

---

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---------|-------|------|----------|-----------|--------|
| TC-001 | parse_document handles empty + preamble-only + nested headings | Unit | P0 | FR-005-AC-1..3, US-002-AC-1..2 | 🚧 |
| TC-002 | parse_document does not panic on 10k random inputs | Property | P0 | FR-005-AC-4, US-002-AC-3 | 🚧 |
| TC-003 | render("fr", valid) byte-equals Python reference | Integration | P0 | FR-001-AC-1, US-001-AC-2 | 🚧 |
| TC-004 | render("unknown-type") returns UnknownBlockType | Unit | P0 | FR-001-AC-2 | 🚧 |
| TC-005 | render rejects non-strict template-field references | Unit | P1 | FR-001-AC-4 | 🚧 |
| TC-006 | render returns field-keyed SchemaViolation on missing required | Unit | P0 | FR-001-AC-3, NFR-005-AC-1 | 🚧 |
| TC-007 | apply_patch merges then validates merged result | Unit | P0 | FR-002-AC-1..3, US-004-AC-1..2 | 🚧 |
| TC-008 | render is thread-safe under 64-thread concurrency | Integration | P1 | FR-001-AC-5, FR-004-AC-2 | 🚧 |
| TC-009 | schemars-derived schema matches reference for each archetype | Snapshot | P0 | FR-003-AC-1..4, US-001-AC-1 | 🚧 |
| TC-010 | Strict mode reports missing template field as TemplateError | Unit | P0 | FR-004-AC-1 | 🚧 |
| TC-011 | Renderer::new() one-time cost measured | Bench | P2 | FR-004-AC-3 | 🚧 |
| TC-012 | extract_frontmatter happy path | Unit | P0 | FR-006-AC-2 | 🚧 |
| TC-013 | extract_frontmatter malformed YAML returns body fallback | Unit | P0 | FR-006-AC-3, US-002 edge case | 🚧 |
| TC-014 | extract_frontmatter unterminated fence returns body fallback | Unit | P1 | FR-006-AC-4 | 🚧 |
| TC-015 | Backtick fence blocks heading parsing inside | Unit | P0 | FR-007-AC-1 | 🚧 |
| TC-016 | Unclosed fence: trailing lines are not parsed as headings | Unit | P1 | FR-007-AC-2 | 🚧 |
| TC-017 | Tilde fence behaves identically to backtick fence | Unit | P1 | FR-007-AC-3 | 🚧 |
| TC-018 | extract evaluates api_endpoint DSL on real fixture | Integration | P0 | FR-011-AC-1, US-003-AC-1 | 🚧 |
| TC-019 | extract code_block (language: json) byte-equals fenced content | Integration | P0 | FR-011-AC-2, US-003-AC-2 | 🚧 |
| TC-020 | TS reference test suite transliterated; all pass | Parity | P0 | StR-003-AC-2 | 🚧 |
| TC-021 | quire-py vs quire-rs structural equivalence on real corpus | Parity | P1 | StR-003-AC-3 | 🚧 |
| TC-022 | Section content preserves leading/trailing whitespace | Unit | P0 | FR-008-AC-1 | 🚧 |
| TC-023 | CRLF and LF endings preserved in section content | Unit | P1 | FR-008-AC-2 | 🚧 |
| TC-024 | Roundtrip: reconstructing body from sections equals input | Property | P0 | FR-008-AC-3, NFR-006 | 🚧 |
| TC-025 | Slug normalization (lowercase, alphanum-dash, trim) | Unit | P0 | FR-009-AC-1..3 | 🚧 |
| TC-026 | Line index ignores frontmatter offset | Unit | P0 | FR-009-AC-4..5 | 🚧 |
| TC-027 | Query API module-level signatures compile and re-export | Compile | P0 | FR-010-AC-1 | 🚧 |
| TC-028 | Query API parity sweep against TS fixtures | Parity | P0 | FR-010-AC-2 | 🚧 |
| TC-029 | Query API complexity: no quadratic walks | Property | P1 | FR-010-AC-3, US-002-AC-2 | 🚧 |
| TC-030 | Render parity — archetype: FR | Parity | P0 | FR-012-AC-1..2, US-005-AC-1..3 | 🚧 |
| TC-031 | Render parity — archetype: NFR | Parity | P0 | FR-012, US-001-AC-3 | 🚧 |
| TC-032 | Render parity — archetype: StR | Parity | P0 | FR-012 | 🚧 |
| TC-033 | Render parity — archetype: US | Parity | P0 | FR-012 | 🚧 |
| TC-034 | Render parity — archetype: IT | Parity | P0 | FR-012 | 🚧 |
| TC-035 | Render parity — archetype: TC | Parity | P0 | FR-012 | 🚧 |
| TC-036 | Render parity — archetype: AC | Parity | P0 | FR-012 | 🚧 |
| TC-037 | Render parity — archetype: CON | Parity | P0 | FR-012 | 🚧 |
| TC-038 | Render parity — archetype: ApplicationSpec | Parity | P0 | FR-012 | 🚧 |
| TC-039 | Render parity — archetype: MasterRequirements | Parity | P0 | FR-012 | 🚧 |
| TC-040 | extract sweep across all 38+ object types | Integration | P1 | FR-011-AC-4, US-003-AC-3 | 🚧 |
| TC-041 | Parity suite catches a deliberate template mutation | Regression | P0 | FR-012-AC-3, US-005-AC-4 | 🚧 |
| TC-042 | Bench: render FR median <1 ms | Bench | P0 | NFR-001-AC-1 | 🚧 |
| TC-043 | Bench: render NFR median <1 ms | Bench | P0 | NFR-001-AC-2 | 🚧 |
| TC-044 | Bench: render StR median <1 ms | Bench | P0 | NFR-001-AC-2 | 🚧 |
| TC-045 | Bench: render US median <1 ms | Bench | P0 | NFR-001-AC-2 | 🚧 |
| TC-046 | Bench: render IT median <1 ms | Bench | P0 | NFR-001-AC-2 | 🚧 |
| TC-047 | Bench: render TC median <1 ms | Bench | P0 | NFR-001-AC-2 | 🚧 |
| TC-048 | Bench: render AC median <1 ms | Bench | P0 | NFR-001-AC-2 | 🚧 |
| TC-049 | Bench: render CON median <1 ms | Bench | P0 | NFR-001-AC-2 | 🚧 |
| TC-050 | check_unsafe_comments.sh exits 0; baseline empty | Static | P0 | NFR-003-AC-1..3, StR-004-AC-2 | 🚧 |
| TC-051 | cargo deny check licenses exits 0; GPL test PR fails | Static | P0 | NFR-004-AC-1..3, StR-004-AC-2 | 🚧 |
| TC-052 | Bench: parse_document 5 MB median <500 ms | Bench | P0 | NFR-002-AC-1 | 🚧 |
| TC-053 | Bench: 5 MB document round-trips byte-for-byte | Property | P0 | NFR-002-AC-3 | 🚧 |
| TC-054 | QuireError::Display contains all four required tuple elements | Unit | P0 | NFR-005-AC-1, US-001-AC-3 | 🚧 |
| TC-055 | QuireError snapshot pins canonical error per archetype | Snapshot | P1 | NFR-005-AC-3 | 🚧 |
| TC-056 | Determinism: render 100x across threads → byte-identical | Property | P0 | NFR-006-AC-1 | 🚧 |
| TC-057 | Determinism: parse 100x → Eq | Property | P0 | NFR-006-AC-2 | 🚧 |
| TC-058 | Static audit: no `HashMap` in render/parse code paths | Static | P1 | NFR-006-AC-3 | 🚧 |

---

## Option Permutation Matrix

The render dispatch admits one main option dimension: `block_type` (10 valid values + invalid). The parity sweep TC-030..TC-039 covers all valid options; TC-004 covers the invalid case.

| Test Case | block_type | data validity | Expected |
|-----------|-----------|----------------|----------|
| TC-030..039 | fr, nfr, str, us, it, tc, ac, con, applicationspec, masterrequirements | valid | byte-equal markdown |
| TC-004 | unknown | (any) | UnknownBlockType |
| TC-006 | fr | missing required field | SchemaViolation(field path) |

---

## Constraint Boundary Tests

Schema constraints derive from the existing iso schemas. Boundary tests follow `garde` field constraint shapes.

| Constraint | Boundary | Test Case | Expected |
|------------|----------|-----------|----------|
| `id pattern ^[A-Z]{2,4}-[0-9]+$` | min length valid (`AB-1`) | TC-009-pattern-min | pass |
| `id pattern` | exact boundary (`ABCD-99999`) | TC-009-pattern-max | pass |
| `id pattern` | lowercase invalid (`ab-1`) | TC-006-pattern-lower | SchemaViolation |
| `id pattern` | no number (`AB-`) | TC-006-pattern-no-num | SchemaViolation |
| `title min length 1` | empty string `""` | TC-007 (merge case) | SchemaViolation |
| `relationships[].target ^ix://` | empty string | TC-006-rel-empty | SchemaViolation |
| `relationships[].target ^ix://` | http://... | TC-006-rel-wrong-scheme | SchemaViolation |
| `artifact_type const "FR"` | wrong const value | TC-006-artifact-type-wrong | SchemaViolation |

---

## Edge Cases

| ID | Description | Related Req | Test Case | Risk if Untested |
|----|-------------|-------------|-----------|------------------|
| EC-001 | Empty markdown input | FR-005 | TC-001 | parse_document panics or returns malformed doc |
| EC-002 | Markdown with no headings (preamble only) | FR-005 | TC-001 | preamble lost or merged into phantom section |
| EC-003 | Heading inside fenced code block | FR-007 | TC-015 | False positive heading splits content |
| EC-004 | Unclosed fenced code block | FR-007 | TC-016 | Trailing content split into phantom sections |
| EC-005 | Frontmatter with invalid YAML | FR-006 | TC-013 | parse_document returns Err instead of body-fallback |
| EC-006 | Frontmatter without closing fence | FR-006 | TC-014 | Body lost or partial body returned |
| EC-007 | Heading level skip (`## A` then `#### B`) | FR-007 (algorithm parity) | TC-020 | Section tree mis-nested |
| EC-008 | CRLF line endings | FR-008 | TC-023 | Content slice loses CR or normalizes endings |
| EC-009 | Title with leading/trailing whitespace | FR-009 | TC-025 | Slug ID contains stray dashes |
| EC-010 | 5 MB document | NFR-002 | TC-052, TC-053 | Quadratic walk surfaces here; OOM on copy |
| EC-011 | LLM-emitted patch with `additionalProperties` violation | FR-002 | TC-007 | Silently accepted vs rejected |
| EC-012 | Concurrent renders from many threads | FR-004, NFR-006 | TC-008, TC-056 | Data race, non-determinism |

---

## Coverage Gaps

| Gap ID | Description | Risk Level | Mitigation |
|--------|-------------|------------|------------|
| GAP-001 | DSL evaluator coverage relies on real fixture availability; some 38+ ix-spec-objects types lack a curated fixture in this repo. | Medium | TC-040 lists each type explicitly; missing fixtures are tracked as P1 follow-up tasks. |
| GAP-002 | The Python Jinja2 reference renderer is not byte-stable across Jinja2 minor versions in all whitespace cases. | Low | StR-002-AC-2 documents known whitespace exceptions; pin Python reference's Jinja2 version. |
| GAP-003 | Storybook / browser integration not applicable (Rust library). | None | No integration test matrix entries. |
| GAP-004 | Cross-machine determinism: NFR-006 is verified within a CI runner; cross-machine byte parity (e.g. arm64 vs x86_64) is implied but not explicitly benched. | Low | Add an arm64 + x86_64 CI matrix as a P2 enhancement. |

---

## Test Execution Summary

All tests are DRAFT — pending implementation via `/spec-to-plan` → `/implement-plan`.

| Category | Total | Passed | Failed | Blocked | Coverage |
|----------|-------|--------|--------|---------|----------|
| Unit | 22 | 0 | 0 | 22 | 0% |
| Integration | 6 | 0 | 0 | 6 | 0% |
| Parity | 13 | 0 | 0 | 13 | 0% |
| Bench | 11 | 0 | 0 | 11 | 0% |
| Property | 6 | 0 | 0 | 6 | 0% |
| Static / Snapshot | 4 | 0 | 0 | 4 | 0% |
| **Total** | **62** | **0** | **0** | **62** | **0%** |
