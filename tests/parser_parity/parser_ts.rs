//! Port of `~/dev/quire/tests/core/parser.test.ts` (FR-001 + FR-027).
//!
//! Each `#[test]` corresponds to one `it(...)` block in the TS suite.
//!
//! Divergences (see `tests/parser_parity/divergences.md`):
//! - **Section content is byte-exact (FR-008).** The TS reference calls
//!   `.strip()` on `section.content`; Rust does not. Where the TS test
//!   asserts a stripped value, the Rust port asserts the byte-exact
//!   value (which typically includes a trailing `\n` that precedes the
//!   next heading line, or the bytes of trailing content lines).
//! - **null/undefined input (FR-027-AC-5).** Rust takes `&str`; the
//!   null-input case is unrepresentable. Skipped.

use quire_rs::parse_document;
use regex::Regex;

#[test]
fn fr_001_ac_1_splits_by_headings_into_named_sections() {
    let md = "## Purpose\nThis is the purpose\n## Scope\nThis is the scope";
    let doc = parse_document(md);

    assert_eq!(doc.sections.len(), 2);
    assert_eq!(doc.sections[0].heading, "Purpose");
    // Divergence: TS strips → "This is the purpose"; Rust is byte-exact
    // and includes the trailing newline that precedes the next heading.
    assert_eq!(doc.sections[0].content, "This is the purpose\n");
    assert_eq!(doc.sections[1].heading, "Scope");
    // Last section: no trailing newline since there is no next heading
    // and the body does not end with '\n'.
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
    // Byte-exact: last section, no trailing newline.
    assert_eq!(doc.sections[0].children[0].content, "child content");
}

#[test]
fn fr_001_ac_3_captures_preamble_before_first_heading() {
    let md = "This is preamble\n\n## First Section\ncontent";
    let doc = parse_document(md);

    assert_eq!(doc.preamble.as_deref(), Some("This is preamble"));
    assert_eq!(doc.sections.len(), 1);
    assert_eq!(doc.sections[0].heading, "First Section");
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
    let id_re = Regex::new(r"^purpose-L\d+$").unwrap();
    assert!(
        id_re.is_match(&doc.sections[0].id),
        "id {:?} should match purpose-L<digits>",
        doc.sections[0].id
    );
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
    assert_eq!(doc.sections[0].children.len(), 2); // L3, L3b
    assert_eq!(doc.sections[0].children[0].children.len(), 1); // L4
}

#[test]
fn does_not_parse_headings_inside_fenced_code_blocks() {
    let md = "## Real\ncontent\n```\n## Not a heading\n```\n## Also Real\nmore";
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
    // Adjacent headings: zero-length slice between them.
    assert_eq!(doc.sections[0].content, "");
    // Last section: byte-exact, no trailing newline (body does not end in '\n').
    assert_eq!(doc.sections[1].content, "content");
}

// Skipped: FR-027-AC-5 null-input TypeError. Rust signature takes `&str`;
// null is unrepresentable. See divergences.md.

#[test]
fn fr_027_ac_3_unclosed_fenced_block_is_handled() {
    let md = "## Section\n```mermaid\ngraph TD\n  A --> B";
    let doc = parse_document(md);

    // Must not crash; section is parsed.
    assert_eq!(doc.sections.len(), 1);
}
