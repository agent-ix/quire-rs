# Task 035: Locator Assert Facet + Field Interpolation

Status: complete

## Scope

Add the optional `assert` facet to `body_extraction` locators (FR-033) and
`{field}` interpolation for assert patterns (FR-034). The facet is evaluated by
`validate_document` (Task 036) and ignored by extraction. Load-time validation
rejects malformed/nonsensical asserts.

## Subtasks

- [ ] **Assert shape (FR-033).** `src/extract/locator.rs`: optional `LocatorAssert { level, columns, min_rows, min_items, id_column, id_pattern }` (all optional) on a locator. `serde(deny_unknown_fields)`.
- [ ] **Load-time validation (FR-033).** Extend the DSL load-time check (alongside `validate_dsl`, `src/extract/dsl.rs`): reject unknown keys and asserts nonsensical for the locator kind (e.g. `columns` on `section_body`) as `ArchetypeLoadFailure`. Extractor ignores `assert`.
- [ ] **Field interpolation (FR-034).** Resolve `{field}` tokens in `id_pattern` (and assert patterns) from document frontmatter at validate time; regex-escape substituted values; unresolved field → diagnostic. Pure function reused by Task 036.

## Owns

FR-033 (AC-1..6), FR-034 (AC-1..4).

## Dependencies

FR-011 DSL (Task 015), FR-016 fallback locators (Task 018) — complete.

## Unblocks

Task 036 (validate_document evaluates asserts).

## Deliverables

- `src/extract/locator.rs`, `src/extract/dsl.rs`, an interpolation helper (e.g. `src/extract/interpolate.rs`).

## Primary Tests

TC-534, TC-535, TC-536, TC-537, TC-538, TC-539 (asserts); TC-540, TC-541, TC-542, TC-543 (interpolation).

## Notes

Determinism: no `HashMap` in `src/extract` (use `IndexMap`/`BTreeMap`). The
extractor MUST produce byte-identical output with and without `assert` present.
