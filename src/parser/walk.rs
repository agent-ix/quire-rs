//! Fenced-code-block-aware heading walk (FR-007).
//!
//! Pure function over `&[&str]` body lines. Emits a flat list of
//! `(level, heading_text, line_index)` tuples for every ATX heading
//! (`#`..`######`) that is NOT inside a fenced code block.
//!
//! Ports the TS (`~/dev/quire/src/core/parser.ts`) and Py
//! (`~/dev/quire-py/quire/parser.py`) fence-tracking algorithm with
//! one extension per FR-007-AC-4: fence types are tracked
//! independently, so a mismatched fence character does not close an
//! open fence (e.g. `~~~` inside a `` ``` `` block is content, not a
//! close).

/// A heading found by [`walk_headings`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    /// ATX level (1..=6, matching the count of `#` chars).
    pub level: usize,
    /// Heading text after the `#`+whitespace prefix, with leading and
    /// trailing whitespace stripped (including CRLF `\r`).
    pub text: String,
    /// 0-based index of the heading line in the input slice.
    pub line: usize,
}

/// Walk `lines` and collect every ATX heading outside fenced code blocks.
pub fn walk_headings(lines: &[&str]) -> Vec<Heading> {
    let mut out: Vec<Heading> = Vec::new();
    let mut fence: Option<FenceKind> = None;
    for (i, line) in lines.iter().enumerate() {
        if let Some(kind) = fence_kind(line) {
            match fence {
                None => fence = Some(kind),
                Some(open) if open == kind => fence = None,
                Some(_) => {
                    // Mismatched fence character (e.g. tilde line inside
                    // a backtick block) — treat as content per FR-007-AC-4.
                }
            }
            continue;
        }
        if fence.is_some() {
            continue;
        }
        if let Some((level, text)) = parse_heading(line) {
            out.push(Heading {
                level,
                text,
                line: i,
            });
        }
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FenceKind {
    Backtick,
    Tilde,
}

/// Return the fence kind if `line` (post `trim_start`) opens or closes a
/// fenced code block. `None` otherwise.
fn fence_kind(line: &str) -> Option<FenceKind> {
    let trimmed: &str = line.trim_start();
    if trimmed.starts_with("```") {
        Some(FenceKind::Backtick)
    } else if trimmed.starts_with("~~~") {
        Some(FenceKind::Tilde)
    } else {
        None
    }
}

/// Parse an ATX heading line per the TS/Py regex `^(#{1,6})\s+(.+)$`,
/// returning `(level, stripped_text)` or `None` if `line` is not a
/// heading. Leading whitespace before `#` is NOT permitted.
fn parse_heading(line: &str) -> Option<(usize, String)> {
    let bytes: &[u8] = line.as_bytes();
    let mut hashes: usize = 0;
    while hashes < bytes.len() && bytes[hashes] == b'#' {
        hashes += 1;
    }
    if hashes == 0 || hashes > 6 {
        return None;
    }
    let rest: &str = &line[hashes..];
    // The `\s+` in the regex requires at least one whitespace char.
    let text_start: usize = rest.find(|c: char| !c.is_whitespace())?;
    if text_start == 0 {
        // `#heading` (no whitespace separator) is not a heading.
        return None;
    }
    let text: &str = rest[text_start..].trim_end();
    if text.is_empty() {
        return None;
    }
    Some((hashes, text.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(s: &str) -> Vec<&str> {
        s.split('\n').collect()
    }

    // FR-007-AC-1 / TC-015
    #[test]
    fn backtick_fence_blocks_inner_headings() {
        let input = "## Real\n```\n# fake\n```\n## Real2";
        let hs = walk_headings(&lines(input));
        assert_eq!(hs.len(), 2);
        assert_eq!(hs[0].text, "Real");
        assert_eq!(hs[0].level, 2);
        assert_eq!(hs[1].text, "Real2");
        assert_eq!(hs[1].level, 2);
    }

    // FR-007-AC-2 / TC-016
    #[test]
    fn unclosed_fence_swallows_trailing_lines() {
        let input = "## Real\n```\n## still-inside\n";
        let hs = walk_headings(&lines(input));
        assert_eq!(hs.len(), 1);
        assert_eq!(hs[0].text, "Real");
    }

    // FR-007-AC-3 / TC-017
    #[test]
    fn tilde_fence_behaves_like_backtick() {
        let input = "~~~\n# fake\n~~~\n## Real";
        let hs = walk_headings(&lines(input));
        assert_eq!(hs.len(), 1);
        assert_eq!(hs[0].text, "Real");
    }

    // FR-007-AC-4: mismatched fence character does not close an open fence.
    #[test]
    fn mismatched_fence_does_not_close_backtick_block() {
        let input = "```\n~~~\n# still-inside\n```\n## Real";
        let hs = walk_headings(&lines(input));
        assert_eq!(hs.len(), 1);
        assert_eq!(hs[0].text, "Real");
    }

    #[test]
    fn headings_at_levels_one_through_six() {
        let input = "# h1\n## h2\n### h3\n#### h4\n##### h5\n###### h6";
        let hs = walk_headings(&lines(input));
        let levels: Vec<usize> = hs.iter().map(|h| h.level).collect();
        assert_eq!(levels, vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn seven_hashes_is_not_a_heading() {
        let hs = walk_headings(&lines("####### too many"));
        assert!(hs.is_empty());
    }

    #[test]
    fn missing_space_after_hashes_is_not_a_heading() {
        let hs = walk_headings(&lines("##no-space"));
        assert!(hs.is_empty());
    }

    #[test]
    fn leading_whitespace_before_hashes_is_not_a_heading() {
        let hs = walk_headings(&lines("  ## indented"));
        assert!(hs.is_empty());
    }

    #[test]
    fn heading_text_is_trimmed_including_crlf() {
        let hs = walk_headings(&lines("##   spaced  \r"));
        assert_eq!(hs.len(), 1);
        assert_eq!(hs[0].text, "spaced");
    }

    #[test]
    fn fence_line_with_info_string_still_toggles() {
        // ``` followed by an info string like "rust" is still a fence open.
        let input = "## Real\n```rust\n## inside\n```\n## After";
        let hs = walk_headings(&lines(input));
        let texts: Vec<&str> = hs.iter().map(|h| h.text.as_str()).collect();
        assert_eq!(texts, vec!["Real", "After"]);
    }

    #[test]
    fn line_indices_match_input_positions() {
        let input = "preamble\n\n## A\nbody\n### B";
        let hs = walk_headings(&lines(input));
        assert_eq!(hs[0].line, 2);
        assert_eq!(hs[1].line, 4);
    }
}
