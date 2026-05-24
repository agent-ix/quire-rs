# Task 015: DSL Locators + Single-Yield (FR-011 part 1)

Status: blocked on Gate G2 (Task 012) + Task 003 (Query API)

## Scope

Implement the 6 Locator primitives and the single-yield (`match`) DSL pattern. This is the foundation of the extract side.

## Subtasks

- [ ] **Locator deserialization.** Define `Locator` (enum: `Single(LocatorPrimitive)` | `Fallback(Vec<LocatorPrimitive>)`) and `LocatorPrimitive` (with `from:` discriminator). YAML untagged variant detection.
- [ ] **6 primitives.** `frontmatter_field`, `section_body`, `code_block`, `table_row`, `list_item`, `heading`. Each backed by the Query API (Task 003) or direct frontmatter JSONPath.
- [ ] **DSL load-time validation.** `match` XOR `iterate_over` (rejected if both); unknown keys rejected at load.
- [ ] **Single-yield evaluator.** `extract(doc, dsl)` returns `ExtractionResult { records: [<=1 entry>], edges: [], diagnostics }`. Required-field misses → `MissingField`.
- [ ] **Purity.** No IO; deterministic.

## Owns

FR-011 ACs 1, 4, 6, 7 (locators + load validation + single-yield required behavior).

## Dependencies

Gate G2 (so we're confident the engine works end-to-end), Task 003 (Query API).

## Unblocks

Task 016 (multi-yield + emit_edges), Task 018 (fallback locators).

## Deliverables

- `src/extract/{mod,dsl,locator}.rs`

## Primary Tests

TC-072 (each locator), TC-073, TC-018 (api_endpoint DSL), TC-019 (code_block DSL), TC-150 (XOR), TC-151 (unknown key).

## Notes

- Reference: `~/dev/filament-parser-lib/filament_parser/tier2/engine.py` for canonical semantics.
- Insertion-ordered HashMap for `match` keys (NFR-006) — use `indexmap::IndexMap` if needed.
