# Task 001: Parser Primitives

Status: not started

## Scope

Implement the four parser primitive modules that `parse_document` (Task 002) orchestrates: frontmatter extraction with malformed-fallback, fenced-code-block-aware heading walk, byte-exact section content slicing, and slug-line ID generation.

## Subtasks

- [ ] **Frontmatter (FR-006).** `extract_frontmatter(&str) -> FrontmatterResult` per FR-006-AC-1..6. BOM strip first. Malformed YAML → body fallback (NOT error). YAML library: `serde_yaml`. Tests: TC-012, 013, 014, 180, 181.
- [ ] **Fence walk (FR-007).** Pure function over `&[&str]` lines: emits `Vec<(level, heading_text, line_index)>`. Toggles on backtick + tilde fences. Tests: TC-015, 016, 017, 020 transliteration partial.
- [ ] **Byte-exact slicing (FR-008).** Given the heading positions, slice source byte-exactly between consecutive heading lines (inclusive of trailing newlines, no normalization). Tests: TC-022, 023, 024 (roundtrip).
- [ ] **Slug-line ID (FR-009).** `slug(text, line) -> String` using ASCII-only `[a-z0-9]+` post-lowercase, collapsing other runs to `-`, stripping edges, appending `-L<line>`. Empty-slug case yields `-L<n>`. Tests: TC-025, 026, 190, 191.

## Owns

FR-006, FR-007, FR-008, FR-009 (and their 6+4+3+7 = 20 ACs).

## Dependencies

None (foundational).

## Unblocks

Task 002 (parse_document) and Task 003 (Query API) and indirectly everything downstream.

## Deliverables

- `src/parser/frontmatter.rs`
- `src/parser/walk.rs`
- `src/parser/slice.rs`
- `src/parser/slug.rs`
- Per-module unit tests inline + a top-level `src/parser/mod.rs` re-exporting the surface.

## Primary Tests

TC-012, TC-013, TC-014, TC-180, TC-181, TC-015, TC-016, TC-017, TC-022, TC-023, TC-024, TC-025, TC-026, TC-190, TC-191.

## Notes

- The TS reference at `~/dev/quire/src/core/parser.ts` is the algorithm source of truth. Read it before starting; the Python port at `~/dev/quire-py/quire/parser.py` is the cleanest reference.
- No `unsafe` (NFR-003). No HashMap on any iteration-order-observable path (NFR-006).
- Per-fixture testing is cheap; cover edge cases liberally — these primitives are foundational.
