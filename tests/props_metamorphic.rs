//! Metamorphic properties over the engine's **public** surface (CR-069,
//! agent-ix/quire-rs#84).
//!
//! The relations that need a normalizer called twice live next to the function
//! they cover — `mask_code_spans`, `ears::normalize` and
//! `normalize_reference_cell` are internal, and widening a published API purely
//! so a test could reach it would be changing the contract for the test's
//! convenience. What lands here is what a consumer can actually observe: the
//! parse ↔ writeback fixpoint.
//!
//! The other two relations the ticket names were already covered and are not
//! re-stated: the parse → reconstruct byte-fixpoint is `slice.rs`'s 10,000-case
//! `tc024_roundtrip_stitch_is_byte_exact` (FR-008-AC-3), and repeated extraction
//! over an identical tree is TC-750 (FR-051-AC-10).

use proptest::prelude::*;
use quire_rs::{parse_document, update_section};

/// A document-shaped string: frontmatter, then headings and content lines.
/// Biased toward the shapes that break byte-splicing — blank lines, adjacent
/// headings, mixed levels, trailing-whitespace lines.
fn any_document() -> impl Strategy<Value = String> {
    let line = prop_oneof![
        Just(String::new()),
        "[a-z]{1,8}".prop_map(|s| s),
        "## [A-Z][a-z]{1,6}".prop_map(|s| s),
        "### [A-Z][a-z]{1,6}".prop_map(|s| s),
        "# [A-Z][a-z]{1,6}".prop_map(|s| s),
        "  [a-z]{1,5}  ".prop_map(|s| s),
    ];
    proptest::collection::vec(line, 0..24)
        .prop_map(|v| format!("---\nid: X-1\ntype: note\n---\n{}", v.join("\n")))
}

/// The headings a parsed document exposes, in document order.
fn headings(text: &str) -> Vec<String> {
    parse_document(text)
        .sections
        .iter()
        .map(|s| s.heading.clone())
        .collect()
}

/// Every heading in the document, at **every** depth.
///
/// Resolution by name is first-match-wins and searches the whole tree, so
/// `# St` and `### St` are one name with two homes — a `# St` top-level section
/// and a `### St` nested under a different parent collide. That ambiguity is
/// what [`quire_rs::update_block`] and FR-020's stable block ids exist to
/// resolve; it is not something `update_section` can be asked to fix. Counting
/// only top-level headings made this property assert against the wrong section.
fn all_headings(sections: &[quire_rs::QuireSection], out: &mut Vec<String>) {
    for s in sections {
        out.push(s.heading.clone());
        all_headings(&s.children, out);
    }
}

/// True when `heading` names exactly one section anywhere in the document, so
/// rewriting it by name is well-defined.
fn is_unambiguous(doc: &quire_rs::QuireDocument, heading: &str) -> bool {
    let mut names = Vec::new();
    all_headings(&doc.sections, &mut names);
    names.iter().filter(|h| h.as_str() == heading).count() == 1
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2_000))]

    /// TC-896 (FR-022-AC-1/AC-4, CR-069, Property): writing a section's own
    /// content back is the **identity** on the whole document, byte for byte —
    /// frontmatter, heading lines and every other section included.
    ///
    /// This is the relation that makes writeback safe to run speculatively. The
    /// example-based TC-430/TC-433 assert it for one document each; stated over
    /// the input space it also covers the shapes that only a generator finds —
    /// an empty section, a section whose content is only blank lines, two
    /// adjacent headings with nothing between them.
    #[test]
    fn tc896_rewriting_a_section_with_its_own_content_is_the_identity(text in any_document()) {
        let doc = parse_document(&text);
        for section in &doc.sections {
            // A duplicate heading resolves to the first match, so rewriting by
            // name is only well-defined when the name is unique. That is
            // FR-035's concern, not this property's.
            if !is_unambiguous(&doc, &section.heading) {
                continue;
            }
            let rewritten = update_section(&doc, &section.heading, &section.content)
                .unwrap_or_else(|e| panic!("update_section({:?}) failed: {e}", section.heading));
            prop_assert_eq!(
                &rewritten,
                &text,
                "rewriting {:?} with its own content changed the document",
                section.heading
            );
        }
    }

    /// TC-896, FR-022-AC-1 (CR-069, Property): after replacing one section's
    /// content, re-parsing yields the same heading structure and the new content
    /// under that heading — the write is readable back through the parser, which
    /// is the fixpoint the byte-splice has to hit for a round-trip edit to be
    /// safe.
    #[test]
    fn tc896_a_section_write_reads_back_through_the_parser(text in any_document()) {
        let doc = parse_document(&text);
        let before = headings(&text);
        for section in &doc.sections {
            if !is_unambiguous(&doc, &section.heading) {
                continue;
            }
            // Content that cannot itself be read as a heading, so the structure
            // is unchanged by construction and any drift is the splice's.
            let replacement = "replaced body\n";
            let rewritten = update_section(&doc, &section.heading, replacement)
                .unwrap_or_else(|e| panic!("update_section({:?}) failed: {e}", section.heading));

            prop_assert_eq!(
                headings(&rewritten),
                before.clone(),
                "the heading structure moved when rewriting {:?}",
                section.heading
            );

            let reparsed = parse_document(&rewritten);
            let read_back = reparsed
                .sections
                .iter()
                .find(|s| s.heading == section.heading)
                .map(|s| s.content.clone());
            prop_assert_eq!(
                read_back.as_deref(),
                Some(replacement),
                "content did not read back under {:?}",
                section.heading
            );
        }
    }
}

/// TC-896 regressions (FR-022-AC-6/AC-7, CR-069): the exact documents the
/// properties minimized to, pinned as examples so the fixes hold under every
/// seed rather than under the one that happened to find them. A property that
/// only fails one run in N is a flaky gate; a property plus its witnesses is a
/// gate.
#[test]
fn tc896_regressions_from_the_generator() {
    const FM: &str = "---\nid: X-1\ntype: note\n---\n";

    // An empty section — heading immediately followed by the next heading.
    // Rewriting it with its own (empty) content inserted a blank line.
    let text = format!("{FM}## Aa\n## Ab");
    let doc = parse_document(&text);
    let empty = doc.sections.iter().find(|s| s.heading == "Aa").unwrap();
    assert_eq!(empty.content, "", "fixture must have an empty section");
    assert_eq!(
        update_section(&doc, "Aa", &empty.content).unwrap(),
        text,
        "a self-write must be the identity for an empty section"
    );

    // A heading on the document's last line has no newline of its own, so the
    // content range starts at EOF and the write was concatenated onto the
    // heading text: `## Aa` + `replaced body` parsed as a heading `Aareplaced
    // body`.
    let text = format!("{FM}## Aa");
    let doc = parse_document(&text);
    let written = update_section(&doc, "Aa", "replaced body\n").unwrap();
    assert_eq!(written, format!("{FM}## Aa\nreplaced body\n"));
    let reparsed = parse_document(&written);
    assert_eq!(
        reparsed
            .sections
            .iter()
            .map(|s| s.heading.as_str())
            .collect::<Vec<_>>(),
        vec!["Aa"],
        "the heading must survive the write"
    );
    assert_eq!(reparsed.sections[0].content, "replaced body\n");

    // Writing into an empty section still works — the separator logic must not
    // suppress a legitimate write.
    let text = format!("{FM}## Aa\n## Ab");
    let doc = parse_document(&text);
    assert_eq!(
        update_section(&doc, "Aa", "body\n").unwrap(),
        format!("{FM}## Aa\nbody\n## Ab")
    );
}
