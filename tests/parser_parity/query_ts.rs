//! Port of `~/dev/quire/tests/core/query.test.ts` (FR-010).
//!
//! Each `#[test]` corresponds to one `it(...)` block in the TS suite.
//!
//! Divergences (see `tests/parser_parity/divergences.md`):
//! - Section content is byte-exact (FR-008). Where the TS asserts a
//!   stripped value, the Rust port asserts the byte-exact slice.
//! - `parseTable` returns `{headers: [], rows: []}` in TS for no-table
//!   inputs; Rust returns `Option::None`. Asserts use `.is_none()`.
//! - `tableFromSection` returns empty in TS for missing section; Rust
//!   returns `None`.
//! - `parseDelegations` / `findDiagramByTag` are not yet exposed by
//!   `quire-rs`. These tests are skipped; see divergences.md.

use quire_rs::{
    extract_diagrams, parse_bullet_list, parse_document, parse_table, parse_tables, section,
    sections, table_from_section, ListPattern,
};

// ─── section() / sections() ───────────────────────────────────────────────

fn sample_doc() -> quire_rs::QuireDocument {
    let md = [
        "## Purpose",
        "purpose content",
        "## 2.1 In Scope",
        "scope content",
        "## notes",
        "notes content",
    ]
    .join("\n");
    parse_document(&md)
}

#[test]
fn fr_002_ac_1_matches_by_heading_text() {
    let doc = sample_doc();
    let s = section(&doc, "Purpose").expect("Purpose");
    // Byte-exact: includes trailing newline before next heading.
    assert_eq!(s.content, "purpose content\n");
}

#[test]
fn fr_002_ac_1_matches_numbered_headings_stripped_of_number() {
    let doc = sample_doc();
    let s = section(&doc, "In Scope").expect("In Scope");
    assert_eq!(s.content, "scope content\n");
}

#[test]
fn fr_002_ac_2_case_insensitive_match() {
    let doc = sample_doc();
    assert!(section(&doc, "in scope").is_some());
}

#[test]
fn fr_002_ac_3_returns_none_for_non_existent_heading() {
    let doc = sample_doc();
    assert!(section(&doc, "NonExistent").is_none());
}

#[test]
fn fr_002_ac_5_exact_match_after_number_stripping() {
    // 'Scope' does NOT match '2.1 In Scope'.
    let doc = sample_doc();
    assert!(section(&doc, "Scope").is_none());
}

#[test]
fn fr_002_ac_4_returns_only_sections_at_specified_level() {
    let md = "# H1\nh1\n## H2a\nh2a\n### H3\nh3\n## H2b\nh2b";
    let doc = parse_document(md);
    let level2 = sections(&doc, Some(2));
    assert_eq!(level2.len(), 2);
    assert_eq!(level2[0].heading, "H2a");
    assert_eq!(level2[1].heading, "H2b");
}

#[test]
fn fr_002_returns_all_sections_when_no_filter() {
    let md = "# H1\nh1\n## H2a\nh2a\n### H3\nh3\n## H2b\nh2b";
    let doc = parse_document(md);
    let all = sections(&doc, None);
    assert!(all.len() >= 4);
}

// ─── parse_table / parse_tables / table_from_section ─────────────────────

#[test]
fn fr_003_ac_1_extracts_headers_and_rows() {
    let content = "| A | B |\n|---|---|\n| 1 | 2 |\n| 3 | 4 |";
    let r = parse_table(content).expect("table");
    assert_eq!(r.headers, vec!["A", "B"]);
    assert_eq!(r.rows, vec![vec!["1", "2"], vec!["3", "4"]]);
}

#[test]
fn fr_003_ac_2_excludes_separator_rows() {
    let r = parse_table("| H |\n|---|\n| d |").expect("table");
    assert_eq!(r.rows.len(), 1);
}

#[test]
fn fr_003_ac_3_strips_leading_trailing_pipes() {
    let r = parse_table("| A |\n|---|\n| val |").expect("table");
    assert_eq!(r.headers, vec!["A"]);
    assert_eq!(r.rows[0], vec!["val"]);
}

#[test]
fn fr_003_ac_4_trims_cell_whitespace() {
    let r = parse_table("|  A  |  B  |\n|---|---|\n|  1  |  2  |").expect("table");
    assert_eq!(r.rows[0], vec!["1", "2"]);
}

#[test]
fn fr_003_ac_5_no_table_returns_none() {
    // Divergence: TS returns empty result; Rust returns Option::None.
    assert!(parse_table("just some text\nno table here").is_none());
}

#[test]
fn fr_003_ac_7_pads_short_rows_and_truncates_long_rows() {
    let content = "| A | B | C |\n|---|---|---|\n| 1 |\n| 1 | 2 | 3 | 4 |";
    let r = parse_table(content).expect("table");
    assert_eq!(r.rows[0], vec!["1", "", ""]);
    assert_eq!(r.rows[1], vec!["1", "2", "3"]);
}

#[test]
fn fr_003_ac_6_parse_tables_returns_all_tables() {
    let content = [
        "| A | B |",
        "|---|---|",
        "| 1 | 2 |",
        "",
        "Some text",
        "",
        "| X | Y |",
        "|---|---|",
        "| 3 | 4 |",
    ]
    .join("\n");
    let results = parse_tables(&content);
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].headers, vec!["A", "B"]);
    assert_eq!(results[1].headers, vec!["X", "Y"]);
}

#[test]
fn table_from_section_extracts_named() {
    let md = "## API\n| Method | Path |\n|---|---|\n| GET | /api |";
    let doc = parse_document(md);
    let r = table_from_section(&doc, "API").expect("API table");
    assert_eq!(r.headers, vec!["Method", "Path"]);
    assert_eq!(r.rows[0], vec!["GET", "/api"]);
}

#[test]
fn table_from_section_returns_none_for_missing() {
    // Divergence: TS returns empty; Rust returns None.
    let doc = parse_document("## Other\ncontent");
    assert!(table_from_section(&doc, "Missing").is_none());
}

// ─── parse_bullet_list ───────────────────────────────────────────────────

#[test]
fn fr_004_ac_1_bold_description_pattern() {
    let content = "- **Auth** — Token-based authentication\n- **Cache** — Redis-backed caching";
    let items = parse_bullet_list(content, None);
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].title, "Auth");
    assert_eq!(items[0].description, "Token-based authentication");
}

#[test]
fn fr_004_ac_2_bold_colon_pattern() {
    let items = parse_bullet_list("- **Auth**: Token-based", Some(ListPattern::BoldColon));
    assert_eq!(items[0].title, "Auth");
    assert_eq!(items[0].description, "Token-based");
}

#[test]
fn fr_004_ac_3_plain_items_have_full_text_as_title() {
    let items = parse_bullet_list("- Simple item\n- Another item", Some(ListPattern::Plain));
    assert_eq!(items[0].title, "Simple item");
    assert_eq!(items[0].description, "");
}

#[test]
fn fr_004_ac_4_excludes_non_bullet_lines() {
    let items = parse_bullet_list("Normal text\n- Bullet\nMore text", None);
    assert_eq!(items.len(), 1);
}

#[test]
fn handles_asterisk_bullets() {
    let items = parse_bullet_list("* Item one\n* Item two", Some(ListPattern::Plain));
    assert_eq!(items.len(), 2);
}

// ─── extract_diagrams ────────────────────────────────────────────────────

#[test]
fn fr_006_ac_1_extracts_mermaid_blocks() {
    let md = "## Arch\n```mermaid\ngraph TD\n  A --> B\n```";
    let doc = parse_document(md);
    let diagrams = extract_diagrams(&doc, None);
    assert_eq!(diagrams.len(), 1);
    assert_eq!(diagrams[0].language, "mermaid");
    assert!(diagrams[0].source.contains("A --> B"));
}

#[test]
fn fr_006_ac_2_parses_type_annotation_into_tag_field() {
    let md = [
        "## Arch",
        "```mermaid",
        "%% @type: logical",
        "graph TD",
        "  A --> B",
        "```",
        "```mermaid",
        "%% @type: deployment",
        "graph TD",
        "  A --> B",
        "```",
    ]
    .join("\n");
    let doc = parse_document(&md);
    let diagrams = extract_diagrams(&doc, None);
    assert_eq!(diagrams[0].tag.as_deref(), Some("logical"));
    assert_eq!(diagrams[1].tag.as_deref(), Some("deployment"));
}

#[test]
fn fr_006_ac_3_tag_is_none_when_no_annotation_present() {
    let md = "## A\n```mermaid\ngraph TD\n  A --> B\n```";
    let doc = parse_document(md);
    let diagrams = extract_diagrams(&doc, None);
    assert!(diagrams[0].tag.is_none());
}

#[test]
fn fr_006_ac_4_tracks_section_association() {
    let md = "## Architecture\n```mermaid\ngraph TD\n  A\n```";
    let doc = parse_document(md);
    let diagrams = extract_diagrams(&doc, None);
    assert_eq!(diagrams[0].section.as_deref(), Some("Architecture"));
}

#[test]
fn extract_diagrams_filters_by_language() {
    let md = "```mermaid\nA\n```\n```plantuml\nB\n```";
    let doc = parse_document(md);
    let mermaid = extract_diagrams(&doc, Some("mermaid"));
    assert_eq!(mermaid.len(), 1);
    assert_eq!(mermaid[0].language, "mermaid");
}

#[test]
fn extract_diagrams_assigns_sequential_indices() {
    let md = "```mermaid\nA\n```\n```mermaid\nB\n```";
    let doc = parse_document(md);
    let diagrams = extract_diagrams(&doc, None);
    assert_eq!(diagrams[0].index, 0);
    assert_eq!(diagrams[1].index, 1);
}

// Divergence #9 (see divergences.md): the TS reference's `extractDiagrams`
// recognizes ``` fences ONLY (`/^```(\w*)/`). The Rust scanner additionally
// recognizes `~~~` fences with matching-character close, matching the
// parser's own FR-007-AC-4 tilde handling. This is a deliberate extension,
// not a parity break — the TS suite has no `~~~` extractDiagrams fixture to
// contradict. The Rust-extension behavior is asserted here.
#[test]
fn divergence_9_tilde_fences_are_a_rust_extension() {
    let md = "## Arch\n~~~mermaid\ngraph TD\n  A --> B\n~~~";
    let doc = parse_document(md);
    let diagrams = extract_diagrams(&doc, None);
    assert_eq!(diagrams.len(), 1);
    assert_eq!(diagrams[0].language, "mermaid");
    assert!(diagrams[0].source.contains("A --> B"));
}

// Skipped: `findDiagramByTag` / `parseDelegations` are not part of the
// quire-rs v1 query surface (FR-010). See divergences.md.

// ─── search ──────────────────────────────────────────────────────────────

fn search_doc() -> quire_rs::QuireDocument {
    parse_document("## Auth\ntoken authentication here\n## Cache\nredis caching layer")
}

#[test]
fn fr_007_ac_1_search_finds_sections_containing_query() {
    let doc = search_doc();
    let r = quire_rs::search(&doc, "authentication");
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].section.heading, "Auth");
}

#[test]
fn fr_007_ac_2_search_case_insensitive() {
    let doc = search_doc();
    let r = quire_rs::search(&doc, "AUTHENTICATION");
    assert_eq!(r.len(), 1);
}

#[test]
fn fr_007_ac_3_search_includes_line_numbers_and_text() {
    let doc = search_doc();
    let r = quire_rs::search(&doc, "token");
    assert!(r[0].matches[0].line > 0);
    assert!(!r[0].matches[0].text.is_empty());
}
