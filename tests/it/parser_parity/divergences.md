# Parser Parity Divergences (Task 004 / Gate G1)

Intentional differences between the Rust `quire-rs` parser+query surface
and the TS `agent-ix/quire` + Py `agent-ix/quire-py` references. Each
entry lists the reference test name, the divergence, the FR/AC that
authorizes it, and where the Rust port asserts the deliberate behavior.

## 1. Section content is byte-exact

- **TS/Py:** `parser.test.ts` / `test_parser.py` assert
  `section.content == "stripped value"` because both ports call
  `.strip()` on each section slice.
- **Rust:** `parse_document` preserves the byte-exact slice between
  consecutive heading positions per FR-008. The slice typically includes
  the trailing `\n` separator that precedes the next heading line, and
  preserves any leading/trailing whitespace and `\r` bytes inside the
  content.
- **Authority:** FR-008-AC-1..3 (byte-exact; CRLF preserved; round-trip
  stitch).
- **Rust ports:** `parser_ts.rs`, `parser_py.rs`, `query_ts.rs`,
  `ast_py.rs` assert the byte-exact form (`"this content\n"` rather than
  `"this content"`).

## 2. `parse_table` / `table_from_section` return `Option`, not empty result

- **TS/Py:** `parse_table("no table here")` returns
  `{headers: [], rows: []}`.
- **Rust:** Returns `Option<TableResult>` — `None` when no table is
  found. Authority: FR-010 spec's Rust API signature for `parse_table`
  uses `Option<TableResult>`.
- **Rust ports:** `query_ts.rs::fr_003_ac_5_no_table_returns_none` and
  `table_from_section_returns_none_for_missing` assert `Option::None`.

## 3. Heading-match contradiction in spec text: TS parity wins

- **Spec text (FR-010):** "section() returns the first section by
  exact heading match (case-sensitive)".
- **TS reference (`query.ts:matchesHeading`):** case-INsensitive, with
  leading section-number normalization.
- **FR-010-AC-2:** mandates TS-fixture parity.
- **Rust:** Implements the TS (case-insensitive + number-normalized)
  behavior since AC-2 wins over the prose. Note logged in
  `src/query.rs:matches_heading`.

## 4. Skipped: null/undefined input TypeError tests

- **TS/Py:** `parseDocument(null)` / `extract_frontmatter(None)` raise.
- **Rust:** Signatures take `&str`; the null case is unrepresentable.
  Skipped in the parity suite. No FR is invalidated.

## 5. Skipped: `findDiagramByTag`, `parseDelegations`

- **TS:** `query.ts` exposes `findDiagramByTag(diagrams, tag)` and
  `parseDelegations(content)`. Neither is part of the FR-010 Rust API
  surface in v1. Skipped; revisit when a new FR adds them.

## 6. Skipped: `QuireAst` wrapper tests

- **Py:** `test_ast.py` exercises a `QuireAst.parse(...)` class wrapper
  that hides `frontmatter` / `raw`.
- **Rust:** No `QuireAst` analog — `QuireDocument` exposes the same
  fields directly. The structurally-relevant cases (heading count,
  byte-exact content, nesting) are ported in `ast_py.rs`.

## 7. Empty-frontmatter behavior matches TS observably

- **TS:** Empty `---\n---\n` frontmatter parses to an empty object then
  is treated as `null` (no useful FM).
- **Rust:** `serde_yaml` decodes empty YAML between fences as
  `Value::Null`, which fails the "must be Map<String, Value>" guard in
  `extract_frontmatter` and returns `None`. Observable behavior matches
  TS; the internal path differs.

## 8. CRLF / BOM behavior is a Rust extension

- **TS/Py:** No CRLF or BOM handling around the frontmatter fences.
- **Rust:** Strips a leading UTF-8 BOM and accepts CRLF around the
  `---` fences per FR-006-AC-5..6. The TS/Py parity tests never exercise
  these cases, so there's nothing to skip — but a Rust extension test
  in `src/parser/frontmatter.rs` covers them.

## 9. `~~~` (tilde) code fences in `extractDiagrams` are a Rust extension

- **TS reference (`query.ts:extractDiagrams`):** the fence scanner uses
  `/^```(\w*)/` and closes on `line.trimStart().startsWith('```')` — it
  recognizes **backtick fences only**. A `~~~` fenced block is invisible
  to the TS diagram/code-block extractor.
- **Rust (`src/query.rs:scan_fenced_blocks`):** the scanner recognizes
  **both** ``` and `~~~` fences, captures the info-string after either,
  and closes a block ONLY on a fence line of the **same** character (a
  `~~~` line inside a ``` block — and vice versa — is content). This
  mirrors the parser's own fence model (`src/parser/walk.rs::fence_kind`,
  FR-007-AC-3/AC-4), so the heading walk and the code-block scanner agree
  on what opens/closes a block.
- **Authority:** FR-011-AC-14 (scanner recognizes ``` and `~~~` with
  matching-character close, parity with the FR-007 parser fence model).
  This is an intentional extension of the TS surface, not a parity break:
  the TS suite has no `~~~` `extractDiagrams` fixture that the Rust
  behavior would contradict.
- **Rust ports:** `query_ts.rs::divergence_9_tilde_fences_are_a_rust_extension`
  asserts the extension; `src/query.rs` unit tests
  (`tilde_fence_block_is_extracted`, `backtick_block_is_not_closed_by_tilde_line`,
  `tilde_block_is_not_closed_by_backtick_line`,
  `unclosed_tilde_block_is_emitted_as_final_block`) and
  `src/extract/locator.rs::code_block_resolves_tilde_fenced_block_under_section`
  cover the scanner and the section-owned locator.

## 10. Escaped pipes (`\|`) in table cells are literal content (CR-007)

- **Behavior:** `parse_table`/`parse_tables` split rows on UNESCAPED pipes
  only; `\|` contributes a literal `|` to the cell text (escape consumed),
  matching GFM table semantics. Border trimming happens inside the
  escape-aware scan, so a cell-final `\|` is not eaten. All other
  backslashes pass through verbatim.
- **Authority:** FR-010-AC-4 (CR-007, 2026-06-12). The original port used a
  naive `split('|')` matching the TS reference: an escaped pipe split the
  cell, shifted later cells, and the header-count truncation silently
  dropped the last column — corrupted rows with no diagnostic, surfaced by
  the 2026-06 configuration-table migration.
- **Parity note:** intentional GFM-conformance extension of the TS surface,
  not a parity break — the TS/Python fixture suites carry no escaped-pipe
  table fixture that the Rust behavior would contradict, and the production
  Python surface consumes this crate via the `quire` wheel.
- **Rust ports:** `src/query.rs` unit tests
  (`cr007_escaped_pipe_in_body_cell_is_literal_not_delimiter`,
  `cr007_escaped_pipe_in_header_cell`,
  `cr007_cell_ending_in_escaped_pipe_keeps_trailing_pipe`,
  `cr007_multiple_escapes_and_enum_cells`,
  `cr007_non_pipe_backslashes_kept_verbatim`,
  `cr007_borderless_rows_still_split_correctly`).
