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
