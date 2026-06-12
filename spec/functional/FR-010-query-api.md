---
id: FR-010
title: "Query API Surface (section, sections, tables, lists, diagrams, search)"
artifact_type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-002"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-003"
    type: "implements"
    cardinality: "1:1"
---

## Behavior

The crate SHALL expose the following query functions in module `quire_rs::query`:

```rust
pub fn section<'d>(doc: &'d QuireDocument, heading: &str) -> Option<&'d QuireSection>;
pub fn sections<'d>(doc: &'d QuireDocument, level: Option<u8>) -> Vec<&'d QuireSection>;
pub fn parse_table(content: &str) -> Option<TableResult>;
pub fn parse_tables(content: &str) -> Vec<TableResult>;
pub fn table_from_section<'d>(doc: &'d QuireDocument, heading: &str) -> Option<TableResult>;
pub fn parse_bullet_list(content: &str, pattern: Option<ListPattern>) -> Vec<ListItem>;
pub fn extract_diagrams<'d>(doc: &'d QuireDocument, language: Option<&str>) -> Vec<DiagramBlock>;
pub fn search<'d>(doc: &'d QuireDocument, query: &str) -> Vec<SearchResult<'d>>;
```

Semantics match `agent-ix/quire/src/core/query.ts` exactly:

- `section()` returns the first section by exact heading match (case-sensitive).
- `sections()` flattens the tree; optional `level` filters to that heading level.
- `parse_table()` accepts pipe-delimited markdown table syntax; returns `None` if no table found at the top of `content`.

> **CR-007 (escaped pipes in table cells, 2026-06-12):** table rows split on
> **unescaped** pipes only — GFM's `\|` escape is a literal pipe in the cell
> text, with the backslash consumed (`<service\|alias>` is ONE cell reading
> `<service|alias>`); all other backslashes are kept verbatim. The original
> port used a naive `split('|')`, so an escaped pipe split the cell, shifted
> every later cell left-to-right, and the header-count truncation then
> silently dropped the final column — plausible-looking corrupted rows with
> no diagnostic. Surfaced by the 2026-06-11/12 configuration-table migration
> (helm-charts, sync-github-service had in-cell `\|` enums). This is an
> intentional GFM-conformance extension of the TS reference (recorded in
> `tests/parser_parity/divergences.md` §10); the production Python surface
> consumes this crate via the `quire` wheel, so the fix covers both.
> See FR-010-AC-4.
- `parse_bullet_list()` recognizes `- `, `* `, `+ ` bullets; the optional `pattern` parameter (`BoldDescription`, `BoldColon`, `Plain`) refines how the bullet text is split into `title` and `description`.
- `extract_diagrams()` returns every fenced code block; optional `language` filters by language tag (case-sensitive).
- `search()` returns substring matches across heading and content, with section ID and byte offsets.

## Acceptance

- **FR-010-AC-1**: Each function signature compiles and is `pub` from `quire_rs::query`.
- **FR-010-AC-2**: For every test fixture in `~/dev/quire/tests/` that exercises these functions, the Rust counterpart returns equivalent results.
- **FR-010-AC-3**: `section()` is `O(n)` where `n` is the total section count; `extract_diagrams()` is `O(lines)`; no quadratic behavior in any function.
- **FR-010-AC-4** (CR-007): `parse_table`/`parse_tables` treat `\|` as a literal pipe in cell content (escape consumed) in header and body cells, including a cell-final `\|` (not eaten by border trim); non-pipe backslashes (`\d`, `C:\tmp`) pass through verbatim; borderless rows split identically. A GFM alignment separator (`|:---|:---:|---:|`) is recognized, and all three bullet markers (`-`, `*`, `+`) parse — previously untested characterization now pinned.
