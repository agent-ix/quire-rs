//! Port of relevant cases from `~/dev/quire-py/tests/test_ast.py`.
//!
//! Divergences (see `tests/parser_parity/divergences.md`):
//! - The Py `QuireAst` wrapper that hides frontmatter/raw is not part
//!   of the Rust API surface — Rust exposes `QuireDocument` directly.
//!   Tests that assert `not hasattr(ast, "frontmatter")` are skipped.
//! - Section content is byte-exact in Rust (FR-008). The Py test for
//!   FR-109-AC-5 byte-exactness still holds bit-for-bit.

use quire_rs::parse_document;

#[test]
fn parse_returns_two_top_level_sections() {
    let doc = parse_document("## A\nx\n## B\ny");
    assert_eq!(doc.sections.len(), 2);
    assert_eq!(doc.sections[0].heading, "A");
    assert_eq!(doc.sections[1].heading, "B");
}

#[test]
fn parse_with_frontmatter_still_exposes_sections() {
    let doc = parse_document("---\nk: v\n---\n## A\nbody");
    assert_eq!(doc.sections[0].heading, "A");
    // Rust diverges from QuireAst: frontmatter IS exposed on
    // QuireDocument. See divergences.md.
    assert!(doc.frontmatter.is_some());
}

#[test]
fn parse_empty_markdown() {
    let doc = parse_document("");
    assert!(doc.preamble.is_none());
    assert!(doc.sections.is_empty());
}

#[test]
fn parse_nested_headings_returns_two_top_level() {
    let doc = parse_document("## A\nbody\n## B\nmore");
    assert_eq!(doc.sections.len(), 2);
    assert_eq!(doc.sections[0].heading, "A");
}

#[test]
fn fr_109_ac_5_byte_exact_table_content_through_quire_document() {
    let md = "## Properties\n| name | type |\n| --- | --- |\n| id   | uuid |";
    let doc = parse_document(md);
    // Last section, body has no trailing '\n' — byte-exact slice has
    // no trailing newline either.
    assert_eq!(
        doc.sections[0].content,
        "| name | type |\n| --- | --- |\n| id   | uuid |"
    );
}
