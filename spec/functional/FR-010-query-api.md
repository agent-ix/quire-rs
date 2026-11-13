---
id: FR-010
title: "Query API Surface (section, sections, tables, lists, diagrams, search)"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-002"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-003"
    type: "implements"
    cardinality: "1:1"
---

## Description

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

> **CR-008 (fence line and CommonMark fence rules, PLAT-1682):** each
> `DiagramBlock` carries `line`, the 1-based line of its opening fence. For
> `extract_diagrams` it is the line in the full file, frontmatter counted; for
> `diagrams_from_content` it is the line within the passed slice. `index` is the
> 0-based ordinal among the returned blocks, so with a `language` filter it counts
> only blocks of that language. Fences follow the CommonMark rules, shared with the
> heading walk ([FR-007-AC-5](./FR-007-fenced-block-heading-walk.md)): an opening
> fence is a run of three or more backticks or tildes indented at most three
> spaces (a backtick fence's info string holds no backtick); the language is the
> info word read after the whole opening run (a four-backtick `mermaid` fence is a
> `mermaid` block); the block closes only on a fence of the same character, at
> least as long as the opener, indented at most three spaces and followed by
> whitespace only. A shorter fence, a fence of the other character, or a closing
> line with an info string is content. Headings inside the block are not headings,
> and headings after the closing fence set `section` again. This extends
> [FR-011](./FR-011-body-extraction-dsl.md) CR-004, whose scanner also backs the
> `code_block` locator. Consumer: quire-cli FR-042 names blocks by `line`.
- `search()` returns substring matches across heading and content, with section ID and byte offsets.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-010-AC-1 | Each function signature compiles and is `pub` from `quire_rs::query`. | Test |
| FR-010-AC-2 | For every test fixture in `~/dev/quire/tests/` that exercises these functions, the Rust counterpart returns equivalent results. | Test |
| FR-010-AC-3 | `section()` is `O(n)` where `n` is the total section count; `extract_diagrams()` is `O(lines)`; no quadratic behavior in any function. | Analysis |
| FR-010-AC-4 | (CR-007) `parse_table`/`parse_tables` treat `\|` as a literal pipe in cell content (escape consumed) in header and body cells, including a cell-final `\|` (not eaten by border trim); non-pipe backslashes (`\d`, `C:\tmp`) pass through verbatim; borderless rows split identically. A GFM alignment separator (`\|:---\|:---:\|---:\|`) is recognized, and all three bullet markers (`-`, `*`, `+`) parse — previously untested characterization now pinned. | Test |
| FR-010-AC-5 | (CR-008) `extract_diagrams` returns each block's `line` as the 1-based line of its opening fence in the full file: with and without frontmatter, the same block reports the same line as its position in the file text. `diagrams_from_content` reports the line within the slice. | Test |
| FR-010-AC-6 | (CR-008) A fence closes only on a fence of the same character at least as long as the opener: a shorter backtick run, a tilde run inside a backtick block, and a closing line carrying an info string are content; a longer closing run closes. Lines after the closing fence, including headings, are attributed to the right `section`. | Test |
| FR-010-AC-7 | (CR-008) A fence indented up to three spaces opens a block; one indented four spaces does not. Backtick and tilde fences both open blocks; a backtick opening line whose info string contains a backtick does not. | Test |
| FR-010-AC-8 | (CR-008) The language is the info word after the whole opening run: ` ````mermaid ` and ` ~~~~mermaid ` are `mermaid` blocks, and a `mermaid` filter selects them. With a language filter, `index` counts only blocks of that language. | Test |

## Dependencies

- **Upstream**: [US-002](../usecase/US-002-developer-parses-spec-doc.md), [StR-003](../stakeholder/StR-003-parse-parity-with-quire.md)
- **Downstream**: none
