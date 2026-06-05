# Task 040: Section-Owned `code_block` Extraction

Status: complete

## Scope

Make the `code_block` body-extraction locator **deterministic and section-owned**
(FR-011, CR-003). The locator previously resolved against the document-wide diagram
catalog (`extract_diagrams(doc)`) filtered by nearest-heading, which defeated per-unit
scoping under multi-yield (`scope_to_section` carries the whole document's `raw`). The
locator now resolves from the section content slice (and thus the iteration unit's
local scope), mirroring `table_row` / `list_item`. The document-wide `extract_diagrams`
remains a separate harvest query (US-010 / RAG), no longer the locator substrate.

Latent gap (not a live bug): the only `code_block` users (`process`, `state_machine`,
`event` in `spec-objects-business`) are single-yield with a unique `after_heading`, so
the global harvest resolved uniquely. No manifest/schema change.

## Subtasks

- [x] **Slice substrate (`src/query.rs`).** Add `pub fn diagrams_from_content(content: &str, language: Option<&str>) -> Vec<DiagramBlock>` (fence/`@type`/language logic identical to `extract_diagrams`'s inner loop; `section: None`). Re-implement `extract_diagrams` to delegate to a shared scanner while still annotating nearest-heading `section` — harvest output byte-for-byte unchanged.
- [x] **Locator rewrite (`src/extract/locator.rs`).** `eval_code_block` → slice pattern mirroring `eval_list_item`: `under_section: Some(h)` → `q_section(doc,h).content`; `None` → joined section bodies (under multi-yield = the unit's scope) → `diagrams_from_content(&content, language)`. Drop the `extract_diagrams` import.
- [x] **Export (`src/lib.rs`).** Keep the `extract_diagrams` re-export; also export `diagrams_from_content`.

## Owns

FR-011-AC-13 (CR-003).

## Dependencies

FR-011 DSL (Task 015), Task 036 (multi-yield local scope) — complete.

## Deliverables

- `src/query.rs`, `src/extract/locator.rs`, `src/lib.rs`.

## Primary Tests

TC-563 (single-yield `under:X` excludes other sections; multi-yield `per_match`
isolates each unit's block; required-miss → `MissingField` for the unit lacking one).
Plus `query.rs` unit tests for `diagrams_from_content` (slice parse, language filter,
`@type`) and the regression guard that `extract_diagrams` harvest output is unchanged.

## Notes

Determinism: no `HashMap` in `src/query` / `src/extract`. `extract_diagrams` harvest
output MUST stay byte-identical (regression guarded by the existing `query.rs` tests).
