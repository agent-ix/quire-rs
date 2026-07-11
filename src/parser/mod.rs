//! Parser primitives (Task 001).
//!
//! Four pure, dependency-free primitives that Task 002's `parse_document`
//! orchestrates:
//!
//! - [`frontmatter::extract_frontmatter`] (FR-006)
//! - [`walk::walk_headings`] (FR-007)
//! - [`slice::slice_section_content`] (FR-008)
//! - [`slug::slug`] / [`slug::slug_line_id`] (FR-009)

pub mod frontmatter;
pub mod slice;
pub mod slug;
pub mod walk;

pub use frontmatter::{extract_frontmatter, FrontmatterResult, FrontmatterStatus};
pub use slice::{line_offsets, slice_section_content};
pub use slug::{slug, slug_line_id};
pub use walk::{walk_headings, Heading};

use crate::ast::{QuireDocument, QuireSection};

/// Parse a markdown string into a [`QuireDocument`] per FR-005.
///
/// Pipeline:
/// 1. [`extract_frontmatter`] strips frontmatter + BOM.
/// 2. [`walk_headings`] collects heading positions (fence-aware).
/// 3. [`slice_section_content`] yields byte-exact content per section.
/// 4. A level-aware stack assembles the heading hierarchy.
///
/// Pure: no IO, no panics, no global mutable state. Re-entrant from
/// any thread (FR-005-AC-4 + NFR-006 determinism).
pub fn parse_document(markdown: &str) -> QuireDocument {
    if markdown.is_empty() {
        return QuireDocument::empty();
    }

    let fm = extract_frontmatter(markdown);
    let body: &str = &fm.body;
    let raw: String = markdown.to_string();

    if body.is_empty() {
        return QuireDocument {
            preamble: None,
            sections: Vec::new(),
            raw,
            frontmatter: fm.frontmatter,
        };
    }

    let lines: Vec<&str> = body.split('\n').collect();
    let offsets: Vec<usize> = line_offsets(body);
    let headings: Vec<Heading> = walk_headings(&lines);

    if headings.is_empty() {
        return QuireDocument {
            preamble: trimmed_to_option(body),
            sections: Vec::new(),
            raw,
            frontmatter: fm.frontmatter,
        };
    }

    // Preamble: lines before the first heading, trimmed (TS/Py parity).
    // Section content stays byte-exact (FR-008); only the preamble trims.
    let first_line: usize = headings[0].line;
    let preamble: Option<String> = if first_line > 0 {
        trimmed_to_option(&lines[..first_line].join("\n"))
    } else {
        None
    };

    // Build flat sections in document order.
    let total_lines: usize = lines.len();
    let mut flat: Vec<QuireSection> = Vec::with_capacity(headings.len());
    for (idx, h) in headings.iter().enumerate() {
        let next_line: usize = if idx + 1 < headings.len() {
            headings[idx + 1].line
        } else {
            offsets.len()
        };
        let content: String = slice_section_content(body, &offsets, h.line, next_line).to_string();
        let end_line: usize = if idx + 1 < headings.len() {
            headings[idx + 1].line.saturating_sub(1)
        } else {
            total_lines.saturating_sub(1)
        };
        flat.push(QuireSection {
            id: slug_line_id(&h.text, h.line),
            block_id: h.block_id.clone(),
            heading: h.text.clone(),
            level: h.level as u8,
            content,
            children: Vec::new(),
            start_line: h.line,
            end_line,
        });
    }

    QuireDocument {
        preamble,
        sections: assemble_tree(flat),
        raw,
        frontmatter: fm.frontmatter,
    }
}

/// Trim `s` (TS/Py `String.strip()` equivalent) and return `None`
/// when the result is empty.
fn trimmed_to_option(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

/// Assemble a flat document-order list of sections into a heading
/// hierarchy via the TS level-aware stack (parser.ts:109-130).
///
/// Two-pass index-based build: compute each section's parent index
/// using a stack of in-progress sections, then move children into
/// parents in reverse so each parent's `children` ends up in document
/// order without per-insert shifting.
fn assemble_tree(flat: Vec<QuireSection>) -> Vec<QuireSection> {
    let n: usize = flat.len();
    if n == 0 {
        return Vec::new();
    }
    let mut parents: Vec<Option<usize>> = vec![None; n];
    let mut stack: Vec<usize> = Vec::new();
    for (i, s) in flat.iter().enumerate() {
        while let Some(&top) = stack.last() {
            if flat[top].level >= s.level {
                stack.pop();
            } else {
                break;
            }
        }
        parents[i] = stack.last().copied();
        stack.push(i);
    }

    // Group children by parent (preserves document order).
    let mut children_of: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut roots: Vec<usize> = Vec::new();
    for (i, p) in parents.iter().enumerate() {
        match p {
            Some(idx) => children_of[*idx].push(i),
            None => roots.push(i),
        }
    }

    // Move sections out of `flat` into the tree via Option::take.
    let mut owned: Vec<Option<QuireSection>> = flat.into_iter().map(Some).collect();
    fn build(
        idx: usize,
        children_of: &[Vec<usize>],
        owned: &mut [Option<QuireSection>],
    ) -> QuireSection {
        let child_idxs: Vec<usize> = children_of[idx].clone();
        let mut node = owned[idx].take().expect("section taken twice");
        for ci in child_idxs {
            node.children.push(build(ci, children_of, owned));
        }
        node
    }

    roots
        .into_iter()
        .map(|r| build(r, &children_of, &mut owned))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // FR-005-AC-1
    #[test]
    fn empty_input_returns_empty_document() {
        let d = parse_document("");
        assert_eq!(d, QuireDocument::empty());
    }

    // FR-005-AC-2
    #[test]
    fn preamble_only_input() {
        let d = parse_document("preamble only");
        assert_eq!(d.preamble.as_deref(), Some("preamble only"));
        assert!(d.sections.is_empty());
        assert_eq!(d.raw, "preamble only");
        assert!(d.frontmatter.is_none());
    }

    // FR-005-AC-3
    #[test]
    fn canonical_ts_fixture_produces_expected_tree() {
        let input =
            "## Parent\nparent content\n### Child\nchild content\n## Sibling\nsibling content";
        let d = parse_document(input);
        assert_eq!(d.sections.len(), 2);
        let parent = &d.sections[0];
        let sibling = &d.sections[1];
        assert_eq!(parent.id, "parent-L0");
        assert_eq!(parent.heading, "Parent");
        assert_eq!(parent.level, 2);
        assert_eq!(parent.children.len(), 1);
        let child = &parent.children[0];
        assert_eq!(child.id, "child-L2");
        assert_eq!(child.heading, "Child");
        assert_eq!(child.level, 3);
        assert_eq!(sibling.id, "sibling-L4");
        assert_eq!(sibling.children.len(), 0);
    }

    #[test]
    fn frontmatter_is_attached_and_does_not_shift_line_indices() {
        let input = "---\nid: FR-001\n---\n## First\ncontent";
        let d = parse_document(input);
        let fm = d.frontmatter.as_ref().expect("frontmatter");
        assert_eq!(fm.get("id").and_then(|v| v.as_str()), Some("FR-001"));
        // Line 0 of the body (post-frontmatter-strip) is the heading.
        // FR-009-AC-4: frontmatter does NOT count toward line indices.
        assert_eq!(d.sections[0].start_line, 0);
        assert_eq!(d.sections[0].id, "first-L0");
        // raw still holds the verbatim original.
        assert_eq!(d.raw, input);
    }

    #[test]
    fn whitespace_only_body_yields_no_preamble() {
        let d = parse_document("   \n\n   ");
        assert!(d.preamble.is_none());
        assert!(d.sections.is_empty());
    }

    #[test]
    fn level_skip_still_nests_under_nearest_lower_level() {
        // "## A" then "#### B": stack-based assembly nests B under A
        // (A is the most recent lower-level node), matching TS.
        let d = parse_document("## A\n#### B\ncontent");
        assert_eq!(d.sections.len(), 1);
        assert_eq!(d.sections[0].heading, "A");
        assert_eq!(d.sections[0].children.len(), 1);
        assert_eq!(d.sections[0].children[0].heading, "B");
        assert_eq!(d.sections[0].children[0].level, 4);
    }

    #[test]
    fn sibling_at_same_level_does_not_nest() {
        let d = parse_document("## A\nfoo\n## B\nbar");
        assert_eq!(d.sections.len(), 2);
        assert!(d.sections[0].children.is_empty());
        assert!(d.sections[1].children.is_empty());
    }

    #[test]
    fn content_is_byte_exact_not_trimmed() {
        // Diverges from TS/Py (which strip). FR-008 mandates byte-exact.
        let d = parse_document("## A\n\n  body  \n\n## B");
        assert_eq!(d.sections[0].content, "\n  body  \n\n");
    }

    #[test]
    fn start_and_end_lines_match_python_reference() {
        let d = parse_document("## A\nfoo\nbar\n## B\nbaz");
        assert_eq!(d.sections[0].start_line, 0);
        assert_eq!(d.sections[0].end_line, 2);
        assert_eq!(d.sections[1].start_line, 3);
        assert_eq!(d.sections[1].end_line, 4);
    }

    proptest! {
        // FR-005-AC-4 / TC-002: parse_document does not panic on any input.
        #![proptest_config(ProptestConfig::with_cases(10_000))]
        #[test]
        fn never_panics_on_arbitrary_utf8(s in "\\PC*") {
            let _ = parse_document(&s);
        }
    }

    // NFR-006: deterministic across threads.
    #[test]
    fn cross_thread_determinism() {
        use std::sync::Arc;
        use std::thread;
        let input = Arc::new("---\nid: x\n---\n## A\n  body  \n### B\nchild\n## C\n".to_string());
        let baseline = parse_document(&input);
        let handles: Vec<_> = (0..16)
            .map(|_| {
                let input = Arc::clone(&input);
                let baseline = baseline.clone();
                thread::spawn(move || {
                    let got = parse_document(&input);
                    assert_eq!(got, baseline, "non-deterministic parse across threads");
                })
            })
            .collect();
        for h in handles {
            h.join().expect("thread");
        }
    }
}
