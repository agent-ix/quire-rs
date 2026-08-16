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

use serde_json::{Map, Value};
use uuid::Uuid;

use crate::ast::{QuireDocument, QuireSection};

/// The cheap header tier of a document parse (FR-005 two tiers, CR-046).
///
/// Membership and identity are decided entirely by frontmatter, by exactly
/// the keys carried here plus the full mapping for resolution and validation
/// reads. The body contributes nothing to either — it is the expensive tier,
/// materialised by [`parse_body`] only for documents something actually
/// queries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    /// Human artifact id from frontmatter `id`; empty string when absent
    /// (FR-024-AC-6 — read, never derived).
    pub id: String,
    /// Frontmatter `type` — the corpus membership/selection discriminator.
    pub type_: Option<String>,
    /// Durable catalog id from frontmatter `uuid`; `None` when absent or
    /// unparseable.
    pub uuid: Option<Uuid>,
    /// The full parsed frontmatter mapping — everything identity, edge
    /// resolution and frontmatter validation read without a body parse.
    pub frontmatter: Map<String, Value>,
    /// Byte offset of the body within the input `parse_header` was given,
    /// so [`parse_body`] slices instead of re-extracting. Nothing in the type
    /// system binds it to that input — see [`body_offset_in`].
    body_offset: usize,
}

/// Parse only the frontmatter block of `markdown` (FR-005 header tier).
///
/// `None` means *not a document*: no frontmatter block, an unterminated
/// fence, or a block that is not a YAML mapping — the same membership rule
/// as `extract_frontmatter().frontmatter.is_some()` (CR-044). One
/// frontmatter extraction, no body work, no copy of the input.
pub fn parse_header(markdown: &str) -> Option<Header> {
    let fm = frontmatter::extract_frontmatter_ref(markdown);
    let map = fm.frontmatter?;
    let id = map
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let type_ = map.get("type").and_then(|v| v.as_str()).map(str::to_string);
    let uuid = map
        .get("uuid")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    Some(Header {
        id,
        type_,
        uuid,
        frontmatter: map,
        body_offset: markdown.len() - fm.body.len(),
    })
}

/// Parse the body of `markdown` under an already-parsed [`Header`]
/// (FR-005 body tier). `markdown` should be the same input the header came
/// from; a mismatched pair is defined rather than a panic (FR-005-AC-7, see
/// [`body_offset_in`]). Composing the two tiers is exactly [`parse_document`].
pub fn parse_body(markdown: &str, header: &Header) -> QuireDocument {
    parse_body_at(
        markdown,
        body_offset_in(markdown, header),
        Some(header.frontmatter.clone()),
    )
}

/// Where `header`'s body starts inside `markdown` (FR-005-AC-7).
///
/// [`Header::body_offset`] is an offset into the input [`parse_header`] was
/// given, and the type system cannot bind the two: a `Header` is owned and
/// stored *beside* an owned text (`corpus::LoadedDocument`), so it cannot
/// borrow from its input. `parse_body(other, &header)` is therefore
/// constructible, and FR-005's purity clause forbids it panicking on a public,
/// PyO3/wasm-reachable entry point. When the offset does not land on a char
/// boundary of `markdown` the pair cannot be the one the header came from, so
/// the offset is re-derived from `markdown` itself and the parse describes the
/// input it was actually given. Correct usage costs one `is_char_boundary`.
fn body_offset_in(markdown: &str, header: &Header) -> usize {
    if markdown.is_char_boundary(header.body_offset) {
        header.body_offset
    } else {
        let fm = frontmatter::extract_frontmatter_ref(markdown);
        markdown.len() - fm.body.len()
    }
}

/// Parse a markdown string into a [`QuireDocument`] per FR-005.
///
/// Composes the two tiers (CR-046): one frontmatter extraction decides the
/// header, then the body pipeline runs —
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

    let fm = frontmatter::extract_frontmatter_ref(markdown);
    parse_body_at(markdown, markdown.len() - fm.body.len(), fm.frontmatter)
}

/// The body pipeline shared by [`parse_body`] and [`parse_document`]:
/// `body_offset` locates the post-frontmatter body inside `markdown`. Total in
/// `body_offset` — both callers derive it from `markdown`, and the slice stays
/// checked so no offset can panic the parser (FR-005-AC-4/AC-7).
fn parse_body_at(
    markdown: &str,
    body_offset: usize,
    frontmatter: Option<Map<String, Value>>,
) -> QuireDocument {
    let body: &str = markdown.get(body_offset..).unwrap_or("");
    let raw: String = markdown.to_string();

    if body.is_empty() {
        return QuireDocument {
            preamble: None,
            sections: Vec::new(),
            raw,
            frontmatter,
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
            frontmatter,
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
        frontmatter,
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

    // TC-812 (FR-005-AC-5): the header tier alone decides membership — a
    // frontmatter-less, unterminated, or non-mapping input is None, with no
    // body pipeline reachable from parse_header at all.
    #[test]
    fn tc812_parse_header_none_for_non_documents() {
        for input in [
            "# readme\n\nno frontmatter here.\n", // no fence
            "---\nid: FR-001\nno closing fence",  // unterminated
            "---\n- a\n- b\n---\nbody",           // valid YAML, not a mapping
            "---\nid: : malformed\n---\nbody",    // unparseable YAML
            "",                                   // empty input
        ] {
            assert!(
                parse_header(input).is_none(),
                "{input:?} should not be a document"
            );
        }
    }

    // TC-812 (FR-005-AC-5): the header carries id/type/uuid and the full
    // frontmatter mapping — identity is read, never derived (FR-024-AC-6).
    #[test]
    fn tc812_parse_header_reads_identity_and_full_map() {
        let input = "---\nid: FR-001\ntype: FR\nuuid: 01890a5d-ac96-774b-bcce-b302099a8057\nextra: kept\n---\n## A\nbody";
        let h = parse_header(input).expect("document");
        assert_eq!(h.id, "FR-001");
        assert_eq!(h.type_.as_deref(), Some("FR"));
        assert_eq!(
            h.uuid,
            Some(uuid::Uuid::parse_str("01890a5d-ac96-774b-bcce-b302099a8057").unwrap())
        );
        assert_eq!(
            h.frontmatter.get("extra").and_then(|v| v.as_str()),
            Some("kept")
        );

        let no_id = parse_header("---\ntype: FR\n---\nbody").expect("document");
        assert_eq!(
            no_id.id, "",
            "id is empty string when absent, never derived"
        );
        assert_eq!(no_id.uuid, None);
    }

    // TC-813 (FR-005-AC-6): parse_document is exactly the composition of the
    // two tiers — parse_body under a parse_header header is byte-identical.
    #[test]
    fn tc813_parse_document_composes_the_tiers() {
        for input in [
            "---\nid: FR-001\ntype: FR\n---\n## A\n\n  body  \n\n### B\nchild\n## C\n",
            "---\nid: x\n---\n",
            "---\nid: x\n---\npreamble only, no headings",
            "\u{FEFF}---\nid: bom\n---\n## H\ncontent",
            "---\r\nid: crlf\r\n---\r\nbody\r\n",
        ] {
            let header = parse_header(input).expect("document");
            assert_eq!(
                parse_body(input, &header),
                parse_document(input),
                "tiers must compose to parse_document for {input:?}"
            );
        }
    }

    // TC-819 (FR-005-AC-7): parse_body is total in its header. `body_offset`
    // is an offset into the input the header came from, so a caller can pair a
    // header with a different string; that must describe the string actually
    // given, never panic (public API, PyO3/wasm-reachable).
    #[test]
    fn tc819_parse_body_with_a_foreign_header_is_total() {
        // Frontmatter block is 28 bytes, so `body_offset` is 28.
        let header =
            parse_header("---\nid: FR-001\ntype: FR\n---\n## A\nbody\n").expect("document");

        // Offset past the end of the other input.
        let short = parse_body("hi", &header);
        assert_eq!(short.raw, "hi");
        assert_eq!(short.preamble.as_deref(), Some("hi"));
        assert_eq!(
            short
                .frontmatter
                .as_ref()
                .and_then(|m| m.get("id"))
                .and_then(|v| v.as_str()),
            Some("FR-001"),
            "the header's own frontmatter is still what the document carries"
        );

        // Offset inside a multi-byte char of the other input (36 bytes, char
        // boundaries every 3 — byte 28 is mid-char).
        let straddling = "日".repeat(12);
        assert_eq!(parse_body(&straddling, &header).raw, straddling);

        // Offset in bounds and on a boundary: no way to detect the mismatch,
        // but the result is still a parse of the input it was given.
        let in_bounds = "x".repeat(40);
        assert_eq!(parse_body(&in_bounds, &header).raw, in_bounds);
    }

    proptest! {
        // FR-005-AC-4 / TC-002: parse_document does not panic on any input.
        #![proptest_config(ProptestConfig::with_cases(10_000))]
        #[test]
        fn never_panics_on_arbitrary_utf8(s in "\\PC*") {
            let _ = parse_document(&s);
        }

        // TC-819 (FR-005-AC-7): no (header input, body input) pair panics
        // parse_body, and the document always describes the body input.
        #[test]
        fn tc819_parse_body_never_panics_on_a_foreign_header(a in "\\PC*", b in "\\PC*") {
            if let Some(h) = parse_header(&a) {
                let doc = parse_body(&b, &h);
                prop_assert_eq!(doc.raw.as_str(), b.as_str());
            }
        }

        // TC-813 (FR-005-AC-6): tier composition equals parse_document on
        // arbitrary input — whenever the input is a document at all.
        #[test]
        fn tiers_compose_on_arbitrary_utf8(s in "\\PC*") {
            if let Some(h) = parse_header(&s) {
                prop_assert_eq!(parse_body(&s, &h), parse_document(&s));
            }
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
