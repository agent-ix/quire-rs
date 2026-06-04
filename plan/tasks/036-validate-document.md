# Task 036: validate_document + Per-Level Heading Uniqueness

Status: complete

## Scope

Implement `validate_document(archetype, doc_text) -> ValidationResult` (FR-032) —
the default markdown validation path — plus the per-level heading-uniqueness rule
(FR-035). Runs parse + frontmatter-schema + body_extraction asserts (asserting
posture) + heading uniqueness; line-numbered diagnostics.

## Subtasks

- [ ] **Entry point (FR-032).** `src/validate_document.rs`: `validate_document(&CompiledArchetype, doc_text) -> ValidationResult { is_valid, errors: Vec<ValidationError{ message, line, reason }> }`. Reasons: `missing`,`empty`,`placeholder`,`assert`,`frontmatter`,`duplicate-heading`.
- [ ] **Pipeline.** parse via `parse_document` (FR-005); validate frontmatter against frontmatter schema (FR-002); run each `required` locator over the Query API (`src/query.rs`) and assert non-empty/non-placeholder; evaluate `assert` facets (Task 035) incl. `{field}` interpolation (Task 035). Placeholder set: `TODO`,`TBD`,`{{...}}`,`placeholder`(ci),`none specified`, empty tables/lists.
- [ ] **Heading uniqueness (FR-035).** Within a doc, no two headings share text at the same level → `duplicate-heading` (line-numbered). Same text at different levels allowed; compatible with `iterate_over`.
- [ ] **Distinct from context validation.** The existing `validate()` (FR-002, JSON object vs schema) stays a separate function; markdown is the default path.

## Owns

FR-032 (AC-1..6), FR-035 (AC-1..4).

## Dependencies

Task 037 (unified archetype: body_extraction + schema), Task 035 (asserts + interpolation). FR-005 parse + FR-010 query (complete).

## Unblocks

Task 038 (PyO3 binding), all downstream consumers (quire-cli, filament-parser, /specify).

## Deliverables

- `src/validate_document.rs` (+ `lib.rs` re-export), `src/diagnostic.rs`/`error.rs` additions for the new reasons.

## Primary Tests

TC-528, TC-529, TC-530, TC-531, TC-532, TC-533 (validate_document); TC-544, TC-545, TC-546, TC-547 (heading uniqueness).

## Notes

`ValidationResult` mirrors existing diagnostic shapes; reuse `thiserror` patterns,
no panics. No `HashMap` in the validation path (determinism, NFR-006).
