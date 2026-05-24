# Task 003: Query API (Layer 2)

Status: blocked on Task 002

## Scope

Implement the public `quire_rs::query` module surface mirroring `~/dev/quire/src/core/query.ts`.

## Subtasks

- [ ] **section / sections.** Linear walk by heading match / level filter.
- [ ] **Tables.** `parse_table`, `parse_tables`, `table_from_section`. Pipe-delimited markdown table parser.
- [ ] **Bullet lists.** `parse_bullet_list(content, pattern)` with three `ListPattern` variants.
- [ ] **Diagrams.** `extract_diagrams(doc, lang?)` — fenced code block enumeration.
- [ ] **Search.** `search(doc, query)` — substring across heading + content with section ID and byte offsets.

## Owns

FR-010 (3 ACs).

## Dependencies

Task 002 (`QuireDocument`).

## Unblocks

Task 015 (DSL locators uses query.section + extract_diagrams).

## Deliverables

- `src/query.rs` (or `src/query/{tables,lists,diagrams,search}.rs` if size warrants)

## Primary Tests

TC-027 (compile), TC-028 (TS parity sweep), TC-029 (complexity proptest).

## Notes

- Reference: `~/dev/quire/src/core/query.ts`.
- All functions take `&QuireDocument` and return owned or borrowed results; nothing mutates the doc.
- No quadratic walks (FR-010-AC-3).
