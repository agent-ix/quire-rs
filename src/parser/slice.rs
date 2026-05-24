//! Byte-exact section content slicing (FR-008).
//!
//! Given a body and a precomputed table of line byte offsets,
//! [`slice_section_content`] returns the exact bytes between the line
//! after the heading and the line of the next heading (or end of
//! body). Whitespace, blank lines, and CRLF `\r` bytes are preserved.
//!
//! This intentionally diverges from the TS/Py reference, which applies
//! `.strip()` (lossy). The Rust engine MUST be byte-exact so that
//! round-trip writeback (Task 012+) is possible — see FR-008-AC-3.
//!
//! Slicing the joined-lines representation (`lines[a..b].join("\n")`)
//! drops the trailing `\n` byte between content and the next heading,
//! so this module operates on raw byte offsets instead.

/// Compute the byte offset where each `\n`-delimited line begins in
/// `body`. The returned vec has `lines.len()` entries when `body` ends
/// with `\n` and `lines.len() - 1` of them are line starts followed by
/// one "past-end" sentinel; in all cases callers can index in
/// `[0, lines.len())` for line starts.
///
/// Specifically: `offsets[i]` is the byte index where `lines[i]` (per
/// `body.split('\n')`) begins. `offsets` always contains
/// `body.split('\n').count()` entries.
pub fn line_offsets(body: &str) -> Vec<usize> {
    let line_count: usize = body.bytes().filter(|&b| b == b'\n').count() + 1;
    let mut offsets: Vec<usize> = Vec::with_capacity(line_count);
    offsets.push(0);
    for (i, b) in body.bytes().enumerate() {
        if b == b'\n' {
            offsets.push(i + 1);
        }
    }
    offsets
}

/// Slice the byte-exact content of a section.
///
/// `heading_line` is the index of the heading line itself; the slice
/// starts at the line *after* it. `next_heading_line` is the index of
/// the next heading (any level), or `offsets.len()` for the final
/// section.
///
/// The returned slice borrows from `body` and is byte-identical to the
/// original bytes that sit between the two heading-line boundaries —
/// including any trailing `\n` (or `\r\n`) that precedes the next
/// heading line.
pub fn slice_section_content<'a>(
    body: &'a str,
    offsets: &[usize],
    heading_line: usize,
    next_heading_line: usize,
) -> &'a str {
    let len: usize = body.len();
    let start: usize = offsets
        .get(heading_line + 1)
        .copied()
        .unwrap_or(len)
        .min(len);
    let end: usize = if next_heading_line < offsets.len() {
        offsets[next_heading_line]
    } else {
        len
    };
    if start >= end {
        return "";
    }
    &body[start..end]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::walk::walk_headings;
    use proptest::prelude::*;

    fn split(body: &str) -> Vec<&str> {
        body.split('\n').collect()
    }

    #[test]
    fn line_offsets_matches_split_positions() {
        let body = "abc\nde\n\nfghi";
        let offsets = line_offsets(body);
        assert_eq!(offsets, vec![0, 4, 7, 8]);
        let lines: Vec<&str> = body.split('\n').collect();
        for (i, &off) in offsets.iter().enumerate() {
            let end = if i + 1 < offsets.len() {
                offsets[i + 1] - 1 // exclude '\n'
            } else {
                body.len()
            };
            assert_eq!(&body[off..end], lines[i]);
        }
    }

    #[test]
    fn line_offsets_for_empty_body() {
        assert_eq!(line_offsets(""), vec![0]);
    }

    #[test]
    fn line_offsets_for_trailing_newline() {
        let body = "abc\n";
        let offsets = line_offsets(body);
        assert_eq!(offsets, vec![0, 4]);
        // split produces ["abc", ""]; we have 2 line starts.
    }

    // FR-008-AC-1 / TC-022
    #[test]
    fn preserves_leading_and_trailing_whitespace() {
        let body = "## A\n  indented body  \n## B\n";
        let lines = split(body);
        let offsets = line_offsets(body);
        let hs = walk_headings(&lines);
        assert_eq!(hs.len(), 2);
        let a = slice_section_content(body, &offsets, hs[0].line, hs[1].line);
        // Slice includes the trailing '\n' that precedes the next heading.
        assert_eq!(a, "  indented body  \n");
    }

    // FR-008-AC-2 / TC-023
    #[test]
    fn preserves_crlf_byte_for_byte() {
        let body = "## A\r\nhello\r\nworld\r\n## B\r\n";
        let lines = split(body);
        let offsets = line_offsets(body);
        let hs = walk_headings(&lines);
        assert_eq!(hs.len(), 2);
        let a = slice_section_content(body, &offsets, hs[0].line, hs[1].line);
        assert_eq!(a, "hello\r\nworld\r\n");
    }

    #[test]
    fn empty_section_between_adjacent_headings() {
        let body = "## A\n## B";
        let lines = split(body);
        let offsets = line_offsets(body);
        let hs = walk_headings(&lines);
        let a = slice_section_content(body, &offsets, hs[0].line, hs[1].line);
        assert_eq!(a, "");
    }

    #[test]
    fn last_section_runs_to_end_of_body() {
        let body = "## A\nfirst\n## B\nsecond\nmore";
        let lines = split(body);
        let offsets = line_offsets(body);
        let hs = walk_headings(&lines);
        let b = slice_section_content(body, &offsets, hs[1].line, offsets.len());
        assert_eq!(b, "second\nmore");
    }

    #[test]
    fn last_section_with_trailing_newline_keeps_it() {
        let body = "## A\nfoo\n";
        let lines = split(body);
        let offsets = line_offsets(body);
        let hs = walk_headings(&lines);
        let a = slice_section_content(body, &offsets, hs[0].line, offsets.len());
        assert_eq!(a, "foo\n");
    }

    #[test]
    fn blank_lines_inside_content_are_preserved() {
        let body = "## A\n\n  para 1  \n\n\n  para 2  \n## B";
        let lines = split(body);
        let offsets = line_offsets(body);
        let hs = walk_headings(&lines);
        let a = slice_section_content(body, &offsets, hs[0].line, hs[1].line);
        assert_eq!(a, "\n  para 1  \n\n\n  para 2  \n");
    }

    /// FR-008-AC-3 / TC-024 invariant: reassembling preamble + (heading
    /// line + content) for each section yields `body` byte-for-byte.
    /// Because each `content_i` already includes the `\n` that follows
    /// its last content line (up to the next heading), the join is
    /// straightforward: preamble bytes + heading_line bytes + content
    /// bytes, repeated.
    fn reconstruct(body: &str) -> String {
        let lines: Vec<&str> = body.split('\n').collect();
        let offsets = line_offsets(body);
        let hs = walk_headings(&lines);
        if hs.is_empty() {
            return body.to_string();
        }
        let mut out = String::new();
        let first: usize = hs[0].line;
        // Preamble bytes: everything up to the start of the first heading line.
        out.push_str(&body[..offsets[first]]);
        for (i, h) in hs.iter().enumerate() {
            // Heading line bytes including trailing '\n' if present.
            let heading_end: usize = if h.line + 1 < offsets.len() {
                offsets[h.line + 1]
            } else {
                body.len()
            };
            out.push_str(&body[offsets[h.line]..heading_end]);
            let next: usize = if i + 1 < hs.len() {
                hs[i + 1].line
            } else {
                offsets.len()
            };
            let content = slice_section_content(body, &offsets, h.line, next);
            out.push_str(content);
        }
        out
    }

    #[test]
    fn reconstruct_handcrafted_cases() {
        for input in [
            "## A\nbody\n## B\nbody2",
            "## A\nbody\n## B\nbody2\n",
            "preamble\n## A\nbody",
            "## A",
            "## A\n",
            "no headings at all",
            "## A\r\nbody\r\n## B",
            "## A\n\n  spaced  \n## B",
            "",
        ] {
            assert_eq!(reconstruct(input), input, "input={input:?}");
        }
    }

    proptest! {
        // FR-008-AC-3: round-trip stitch is byte-exact for any body.
        // 10 000 cases matches the parse-document no-panic budget;
        // this property is load-bearing for writeback (Task 012+).
        #![proptest_config(ProptestConfig::with_cases(10_000))]
        #[test]
        fn roundtrip_stitch_is_byte_exact(body in any_body()) {
            prop_assert_eq!(reconstruct(&body), body);
        }
    }

    /// Generate a small "body-shaped" string of headings and content
    /// lines. Bias toward inputs that exercise the slicing edge cases:
    /// blank lines, adjacent headings, mixed levels.
    fn any_body() -> impl Strategy<Value = String> {
        let line = prop_oneof![
            Just(String::new()),
            "[a-z]{1,8}".prop_map(|s| s),
            "## [A-Z][a-z]{1,6}".prop_map(|s| s),
            "### [A-Z][a-z]{1,6}".prop_map(|s| s),
            "# [A-Z][a-z]{1,6}".prop_map(|s| s),
            "  [a-z]{1,5}  ".prop_map(|s| s),
        ];
        proptest::collection::vec(line, 0..30).prop_map(|v| v.join("\n"))
    }
}
