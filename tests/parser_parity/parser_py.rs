//! Port of `~/dev/quire-py/tests/test_parser.py` (FR-005/006/007/008/009).
//!
//! Divergences (see `tests/parser_parity/divergences.md`):
//! - Section content is byte-exact (FR-008) in Rust; the Py reference
//!   strips. The Rust assertions reflect the byte-exact bytes.
//! - `_slug` is a private helper in Py; Rust exposes it via
//!   `parser::slug`. We exercise it through the public re-export.
//! - `test_fr_027_ac_5_non_string_input_raises_typeerror`: the Rust
//!   API takes `&str`; null is unrepresentable. Skipped.

use quire_rs::parse_document;

#[test]
fn fr_001_ac_1_splits_by_headings_into_named_sections() {
    let md = "## Purpose\nThis is the purpose\n## Scope\nThis is the scope";
    let doc = parse_document(md);
    assert_eq!(doc.sections.len(), 2);
    assert_eq!(doc.sections[0].heading, "Purpose");
    assert_eq!(doc.sections[0].content, "This is the purpose\n");
    assert_eq!(doc.sections[1].heading, "Scope");
    assert_eq!(doc.sections[1].content, "This is the scope");
}

#[test]
fn fr_001_ac_2_nested_headings_form_hierarchy() {
    let md = "## Parent\nparent content\n### Child\nchild content";
    let doc = parse_document(md);
    assert_eq!(doc.sections.len(), 1);
    assert_eq!(doc.sections[0].heading, "Parent");
    assert_eq!(doc.sections[0].children.len(), 1);
    assert_eq!(doc.sections[0].children[0].heading, "Child");
    assert_eq!(doc.sections[0].children[0].content, "child content");
}

#[test]
fn fr_001_ac_3_captures_preamble_before_first_heading() {
    let md = "This is preamble\n\n## First Section\ncontent";
    let doc = parse_document(md);
    assert_eq!(doc.preamble.as_deref(), Some("This is preamble"));
    assert_eq!(doc.sections.len(), 1);
}

#[test]
fn fr_001_ac_4_preserves_numbered_headings() {
    let md = "## 2.1 In Scope\ncontent";
    let doc = parse_document(md);
    assert_eq!(doc.sections[0].heading, "2.1 In Scope");
}

#[test]
fn generates_stable_section_ids() {
    let md = "## Purpose\ncontent";
    let doc = parse_document(md);
    assert!(doc.sections[0].id.starts_with("purpose-L"));
}

#[test]
fn handles_multi_level_nesting() {
    let md = [
        "## L2",
        "l2 content",
        "### L3",
        "l3 content",
        "#### L4",
        "l4 content",
        "### L3b",
        "l3b content",
    ]
    .join("\n");
    let doc = parse_document(&md);
    assert_eq!(doc.sections.len(), 1);
    assert_eq!(doc.sections[0].children.len(), 2);
    assert_eq!(doc.sections[0].children[0].children.len(), 1);
}

#[test]
fn does_not_parse_headings_inside_backtick_fenced_blocks() {
    let md = "## Real\ncontent\n```\n## Not a heading\n```\n## Also Real\nmore";
    let doc = parse_document(md);
    assert_eq!(doc.sections.len(), 2);
    assert_eq!(doc.sections[0].heading, "Real");
    assert_eq!(doc.sections[1].heading, "Also Real");
}

#[test]
fn does_not_parse_headings_inside_tilde_fenced_blocks() {
    let md = "## Real\ncontent\n~~~\n## Not a heading\n~~~\n## Also Real\nmore";
    let doc = parse_document(md);
    assert_eq!(doc.sections.len(), 2);
    assert_eq!(doc.sections[0].heading, "Real");
    assert_eq!(doc.sections[1].heading, "Also Real");
}

#[test]
fn preserves_raw_markdown() {
    let md = "## A\ncontent";
    let doc = parse_document(md);
    assert_eq!(doc.raw, md);
}

#[test]
fn handles_document_with_no_headings() {
    let md = "Just some text\nwith no headings";
    let doc = parse_document(md);
    assert_eq!(doc.sections.len(), 0);
    assert_eq!(
        doc.preamble.as_deref(),
        Some("Just some text\nwith no headings")
    );
}

#[test]
fn fr_027_ac_1_empty_string_returns_empty_document() {
    let doc = parse_document("");
    assert!(doc.preamble.is_none());
    assert_eq!(doc.sections.len(), 0);
    assert_eq!(doc.raw, "");
}

#[test]
fn fr_027_ac_4_heading_with_empty_content_has_empty_string() {
    let md = "## Empty\n## Next\ncontent";
    let doc = parse_document(md);
    assert_eq!(doc.sections.len(), 2);
    assert_eq!(doc.sections[0].content, "");
    assert_eq!(doc.sections[1].content, "content");
}

// Skipped: fr_027_ac_5_non_string_input_raises_typeerror — Rust takes &str.

#[test]
fn fr_027_ac_3_unclosed_fenced_block_does_not_crash() {
    let md = "## Section\n```mermaid\ngraph TD\n  A --> B";
    let doc = parse_document(md);
    assert_eq!(doc.sections.len(), 1);
}

#[test]
fn fr_109_con_2_content_is_byte_exact_source_slice() {
    let md = "## A\n| col | val |\n| --- | --- |\n| 1   | x   |\n## B\nz";
    let doc = parse_document(md);
    // Byte-exact slice includes trailing newline before the next heading.
    assert_eq!(
        doc.sections[0].content,
        "| col | val |\n| --- | --- |\n| 1   | x   |\n"
    );
}

#[test]
fn fr_109_con_3_fenced_code_blocks_respected_with_indent() {
    let md = "## A\n  ```\n  ## Not a heading\n  ```\n## B";
    let doc = parse_document(md);
    assert_eq!(doc.sections.len(), 2);
    assert_eq!(doc.sections[0].heading, "A");
    assert_eq!(doc.sections[1].heading, "B");
}

#[test]
fn fr_109_con_9_empty_markdown_yields_empty_ast() {
    let doc = parse_document("");
    assert!(doc.preamble.is_none());
    assert!(doc.sections.is_empty());
    assert!(doc.frontmatter.is_none());
}

#[test]
fn fr_109_con_9_no_headings_yields_empty_sections_with_preamble() {
    let md = "just a paragraph\nstill just text";
    let doc = parse_document(md);
    assert!(doc.sections.is_empty());
    assert_eq!(
        doc.preamble.as_deref(),
        Some("just a paragraph\nstill just text")
    );
}

#[test]
fn whitespace_only_input_yields_no_preamble() {
    let doc = parse_document("   \n  \n");
    assert!(doc.preamble.is_none());
    assert!(doc.sections.is_empty());
}

#[test]
fn heading_at_line_zero_no_preamble() {
    let doc = parse_document("## First\ncontent");
    assert!(doc.preamble.is_none());
    assert_eq!(doc.sections[0].heading, "First");
}

#[test]
fn slug_strips_leading_and_trailing_dashes() {
    use quire_rs::parser::slug;
    assert_eq!(slug("!!Hello, World!!"), "hello-world");
    assert_eq!(slug("---"), "");
}

#[test]
fn h1_through_h6_levels() {
    let md = "# H1\n## H2\n### H3\n#### H4\n##### H5\n###### H6";
    let doc = parse_document(md);
    assert_eq!(doc.sections[0].level, 1);
    assert_eq!(doc.sections[0].children[0].level, 2);
    assert_eq!(doc.sections[0].children[0].children[0].level, 3);
    assert_eq!(doc.sections[0].children[0].children[0].children[0].level, 4);
    assert_eq!(
        doc.sections[0].children[0].children[0].children[0].children[0].level,
        5
    );
}

#[test]
fn sibling_headings_at_same_level_are_siblings_not_parent_child() {
    let md = "## A\n## B\n## C";
    let doc = parse_document(md);
    assert_eq!(doc.sections.len(), 3);
    for s in &doc.sections {
        assert!(s.children.is_empty());
    }
}

#[test]
fn start_line_and_end_line_populated() {
    let md = "## A\nx\n## B\ny\nz";
    let doc = parse_document(md);
    assert_eq!(doc.sections[0].start_line, 0);
    assert_eq!(doc.sections[0].end_line, 1);
    assert_eq!(doc.sections[1].start_line, 2);
    assert_eq!(doc.sections[1].end_line, 4);
}
