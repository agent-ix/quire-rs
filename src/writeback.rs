//! Section + block writeback (FR-022).
//!
//! Two primitives that take a parsed `QuireDocument` and an edit and
//! return the **full updated markdown** with the rest of the file
//! byte-identical:
//!
//! - [`update_section`]: replace one section's *content* (everything
//!   after the heading line, up to the next heading or EOF).
//!   Heading line + frontmatter + other sections stay byte-identical.
//!   Port of TS `agent-ix/quire`'s `updateSection` from
//!   `~/dev/quire/src/core/writeback.ts`.
//! - [`update_block`]: replace one *block's bytes entirely* — the
//!   heading line and its content together. Found by block_id (the
//!   stable `{#blk-id}` attribute). This is a pure byte-splice edit;
//!   the render-dependent block edit API has been removed.
//!
//! Both functions operate on byte ranges in `doc.raw`, never on the
//! parsed tree. The on-disk markdown stays canonical.

use crate::ast::{QuireDocument, QuireSection};
use crate::error::QuireError;
use crate::query::section as q_section;

/// Replace the content of the section whose heading matches
/// `heading` (case-insensitive, section-number-prefix normalized
/// per [`crate::query::section`]) with `new_content`. Returns the
/// updated full-file markdown.
///
/// Errors:
/// - `MissingField` when no section matches.
pub fn update_section(
    doc: &QuireDocument,
    heading: &str,
    new_content: &str,
) -> Result<String, QuireError> {
    let target = q_section(doc, heading).ok_or_else(|| QuireError::MissingField {
        key: heading.to_string(),
        locator: format!("section({heading})"),
    })?;
    splice_section_content(doc, target, new_content)
}

/// Replace the bytes of the block identified by `block_id` (the
/// stable Pandoc heading attribute `{#blk-id}`) with `new_bytes`.
/// `new_bytes` should be a complete block rendering — heading line
/// (with its `{#blk-id}` attribute preserved) followed by the
/// content. Returns the updated full-file markdown.
///
/// Errors:
/// - `MissingField` when no block carries the given id.
pub fn update_block(
    doc: &QuireDocument,
    block_id: &str,
    new_bytes: &str,
) -> Result<String, QuireError> {
    let target =
        find_block_by_id(&doc.sections, block_id).ok_or_else(|| QuireError::MissingField {
            key: block_id.to_string(),
            locator: format!("block({block_id})"),
        })?;
    splice_block(doc, target, new_bytes)
}

/// Recursive find by stable block ID. Returns the first section
/// whose `block_id == Some(block_id)`.
fn find_block_by_id<'d>(sections: &'d [QuireSection], block_id: &str) -> Option<&'d QuireSection> {
    for s in sections {
        if s.block_id.as_deref() == Some(block_id) {
            return Some(s);
        }
        if let Some(found) = find_block_by_id(&s.children, block_id) {
            return Some(found);
        }
    }
    None
}

/// Compute the byte offset where the post-frontmatter body begins
/// inside `doc.raw`. Mirrors the boundary that `extract_frontmatter`
/// uses, so line indices stay aligned.
fn body_start_offset(raw: &str) -> usize {
    let stripped_offset = if raw.starts_with('\u{FEFF}') { 3 } else { 0 };
    let rest = &raw[stripped_offset..];
    let opening: usize = if let Some(_r) = rest.strip_prefix("---\n") {
        4
    } else if let Some(_r) = rest.strip_prefix("---\r\n") {
        5
    } else {
        return stripped_offset;
    };
    let after_open = &rest[opening..];
    let close_rel = match after_open.find("\n---") {
        Some(p) => p,
        None => return stripped_offset,
    };
    let body_tail = &after_open[close_rel + 4..];
    let consumed_after = if body_tail.starts_with("\r\n") {
        2
    } else if body_tail.starts_with('\n') {
        1
    } else {
        0
    };
    stripped_offset + opening + close_rel + 4 + consumed_after
}

/// Splice `new_content` into the section's content range, returning
/// the updated full-file markdown. The heading line stays
/// byte-identical; only the content between this heading and the
/// next heading (or EOF) is replaced.
fn splice_section_content(
    doc: &QuireDocument,
    target: &QuireSection,
    new_content: &str,
) -> Result<String, QuireError> {
    let body_offset = body_start_offset(&doc.raw);
    let body = &doc.raw[body_offset..];

    // Walk the body line offsets to find the byte range we're
    // replacing: from "after the heading line" to "start of next
    // heading line" (or end of body).
    let line_offsets = crate::parser::line_offsets(body);
    let content_start_body = line_offsets
        .get(target.start_line + 1)
        .copied()
        .unwrap_or(body.len());
    let next_line = target.end_line + 1; // line *after* the last content line
    let content_end_body = line_offsets.get(next_line).copied().unwrap_or(body.len());

    let abs_start = body_offset + content_start_body;
    let abs_end = body_offset + content_end_body;
    let trailing_newline = doc.raw[..abs_end].ends_with('\n');

    let mut out = String::with_capacity(doc.raw.len() + new_content.len());
    out.push_str(&doc.raw[..abs_start]);
    out.push_str(new_content);
    // Keep the original line boundary at content_end. If the
    // original content ended with `\n` (next-heading boundary or EOF
    // after `\n`), preserve it; otherwise don't add one.
    if trailing_newline && !new_content.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&doc.raw[abs_end..]);
    Ok(out)
}

/// Splice `new_bytes` into the block's *full* byte range (heading
/// line + content) — returns the updated full-file markdown.
fn splice_block(
    doc: &QuireDocument,
    target: &QuireSection,
    new_bytes: &str,
) -> Result<String, QuireError> {
    let body_offset = body_start_offset(&doc.raw);
    let body = &doc.raw[body_offset..];
    let line_offsets = crate::parser::line_offsets(body);

    let block_start_body = line_offsets
        .get(target.start_line)
        .copied()
        .unwrap_or(body.len());
    let next_line = target.end_line + 1;
    let block_end_body = line_offsets.get(next_line).copied().unwrap_or(body.len());

    let abs_start = body_offset + block_start_body;
    let abs_end = body_offset + block_end_body;
    let trailing_newline = doc.raw[..abs_end].ends_with('\n');

    let mut out = String::with_capacity(doc.raw.len() + new_bytes.len());
    out.push_str(&doc.raw[..abs_start]);
    out.push_str(new_bytes);
    if trailing_newline && !new_bytes.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&doc.raw[abs_end..]);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_document;

    // ─── update_section ─────────────────────────────────────────────

    // Port of TS writeback.test.ts FR-014-AC-1: target section replaced.
    #[test]
    fn update_section_replaces_target_content() {
        let md = "## Purpose\nold purpose\n## Scope\nthe scope\n";
        let doc = parse_document(md);
        let out = update_section(&doc, "Purpose", "new purpose\n").expect("ok");
        assert_eq!(out, "## Purpose\nnew purpose\n## Scope\nthe scope\n");
    }

    // FR-014-AC-2: other sections byte-identical.
    #[test]
    fn update_section_preserves_other_sections_byte_identical() {
        let md = "## Purpose\nold\n## Scope\nthe scope   \n  with  spaces\n";
        let doc = parse_document(md);
        let out = update_section(&doc, "Purpose", "x\n").expect("ok");
        // The Scope section's bytes survive unchanged.
        assert!(out.contains("## Scope\nthe scope   \n  with  spaces\n"));
    }

    // FR-014-AC-3: frontmatter preserved.
    #[test]
    fn update_section_preserves_frontmatter_byte_identical() {
        let md = "---\nid: FR-001\ntitle: x\n---\n## Behavior\nold\n## Acceptance\n- AC-1\n";
        let doc = parse_document(md);
        let out = update_section(&doc, "Behavior", "new behavior\n").expect("ok");
        assert!(out.starts_with("---\nid: FR-001\ntitle: x\n---\n"));
        assert!(out.contains("## Behavior\nnew behavior\n"));
        assert!(out.contains("## Acceptance\n- AC-1\n"));
    }

    #[test]
    fn update_section_unknown_heading_returns_missing_field() {
        let md = "## A\nbody\n";
        let doc = parse_document(md);
        let err = update_section(&doc, "Nope", "x").expect_err("missing");
        assert!(matches!(err, QuireError::MissingField { .. }));
    }

    #[test]
    fn update_section_handles_no_frontmatter() {
        let md = "## A\nold\n## B\nb\n";
        let doc = parse_document(md);
        let out = update_section(&doc, "A", "new\n").unwrap();
        assert_eq!(out, "## A\nnew\n## B\nb\n");
    }

    // ─── update_block (block_id addressing) ──────────────────────────

    #[test]
    fn update_block_replaces_heading_and_content() {
        let md =
            "## Behavior {#blk-7af2}\nold body\nold line 2\n## Acceptance {#blk-9c14}\n- AC-1\n";
        let doc = parse_document(md);
        let new_bytes = "## Behavior {#blk-7af2}\nnew body\n";
        let out = update_block(&doc, "blk-7af2", new_bytes).expect("ok");
        assert_eq!(
            out,
            "## Behavior {#blk-7af2}\nnew body\n## Acceptance {#blk-9c14}\n- AC-1\n"
        );
    }

    #[test]
    fn update_block_other_blocks_byte_identical() {
        let md =
            "---\nid: FR-001\n---\n## Behavior {#blk-a}\nold\n## Acceptance {#blk-b}\n- AC-1\n  - nested  \n";
        let doc = parse_document(md);
        let out = update_block(&doc, "blk-a", "## Behavior {#blk-a}\nnew\n").unwrap();
        assert!(out.starts_with("---\nid: FR-001\n---\n"));
        assert!(out.contains("## Acceptance {#blk-b}\n- AC-1\n  - nested  \n"));
    }

    #[test]
    fn update_block_finds_nested_block_id() {
        let md = "## Parent\nparent body\n### Inner {#blk-deep}\ninner old\n## Other\nother\n";
        let doc = parse_document(md);
        let out = update_block(&doc, "blk-deep", "### Inner {#blk-deep}\ninner new\n").unwrap();
        assert!(out.contains("### Inner {#blk-deep}\ninner new\n"));
        assert!(out.contains("## Parent\nparent body\n"));
        assert!(out.contains("## Other\nother\n"));
    }

    #[test]
    fn update_block_unknown_id_returns_missing_field() {
        let md = "## A {#blk-x}\nbody\n";
        let doc = parse_document(md);
        let err = update_block(&doc, "nope", "## A {#blk-x}\nbody\n").expect_err("missing");
        assert!(matches!(err, QuireError::MissingField { .. }));
    }

    // Stable round-trip property: update_block followed by
    // parse_document of the result gives back a doc whose block_id is
    // preserved.
    #[test]
    fn update_block_preserves_block_id_through_roundtrip() {
        let md = "## Behavior {#blk-7af2}\nold\n## Acceptance {#blk-9c14}\n- AC\n";
        let doc1 = parse_document(md);
        let out = update_block(&doc1, "blk-7af2", "## Behavior {#blk-7af2}\nnew\n").unwrap();
        let doc2 = parse_document(&out);
        let id1: Vec<Option<&str>> = doc1
            .sections
            .iter()
            .map(|s| s.block_id.as_deref())
            .collect();
        let id2: Vec<Option<&str>> = doc2
            .sections
            .iter()
            .map(|s| s.block_id.as_deref())
            .collect();
        assert_eq!(id1, id2);
        assert_eq!(id1, vec![Some("blk-7af2"), Some("blk-9c14")]);
    }
}
