# Task 016: DSL Multi-Yield + emit_edges (FR-011 part 2)

Status: blocked on Task 015

## Scope

Add `iterate_over + per_match` multi-yield and declarative `emit_edges` evaluation on top of the single-yield foundation from Task 015.

## Subtasks

- [ ] **iterate_over deserialization.** `{ section_path: Vec<String>, kind: heading|list_item|table_row, depth: Option<u8> }`.
- [ ] **Iteration unit resolution.** Walk the section tree to the root section, then iterate per `kind` at depth.
- [ ] **per_match evaluation.** For each iteration unit, evaluate the `Locator` map in unit-local scope (where `section_body` references resolve relative to the unit).
- [ ] **emit_edges.** Declarative list of `{ type, target: Locator | String, metadata }`. For each iteration record, emit an edge per entry where target resolves.
- [ ] **IterateRootMissing diagnostic.** Empty records + diagnostic when `section_path` doesn't resolve.

## Owns

FR-011 ACs 2, 3, 8.

## Dependencies

Task 015.

## Unblocks

Task 017 (edge harvesting consumes `emit_edges` output).

## Deliverables

- Extensions to `src/extract/dsl.rs`

## Primary Tests

TC-070, TC-071, TC-152.

## Notes

- Multi-yield bound: `records.len() ≤ max(headings, list_items, table_rows)` in the doc. No engine-side cap.
- Edge metadata can itself be `Locator`-valued; evaluate per iteration unit before emitting.
