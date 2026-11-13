//! Fenced-code-block-aware heading walk (FR-007).
//!
//! Pure function over `&[&str]` body lines. Emits a flat list of
//! `(level, heading_text, line_index)` tuples for every ATX heading
//! (`#`..`######`) that is NOT inside a fenced code block.
//!
//! Fences follow the CommonMark fence rules, measured from the enclosing
//! list item's content column (FR-007-AC-5/AC-6); this deliberately diverges
//! from the TS (`~/dev/quire/src/core/parser.ts`) and Py
//! (`~/dev/quire-py/quire/parser.py`) toggle-on-three-backticks algorithm
//! (FR-007 CR-009). The scanner here is the single fence model for the
//! heading walk, `extract_diagrams`, the `code_block` locator, list-item
//! minting and table detection.

/// A heading found by [`walk_headings`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    /// ATX level (1..=6, matching the count of `#` chars).
    pub level: usize,
    /// Heading text after the `#`+whitespace prefix, with leading and
    /// trailing whitespace stripped (including CRLF `\r`). If the
    /// heading line carries a trailing Pandoc attribute (`{#id}`),
    /// the attribute is NOT part of this string — it lands in
    /// [`block_id`](Self::block_id) instead.
    pub text: String,
    /// 0-based index of the heading line in the input slice.
    pub line: usize,
    /// Stable block ID, parsed from a trailing Pandoc heading
    /// attribute `## Heading {#blk-id}` (FR-019). `None` when the
    /// heading has no attribute. IDs are arbitrary author-supplied
    /// strings; the parser does not constrain their shape beyond
    /// "no whitespace and no closing brace".
    pub block_id: Option<String>,
}

/// Walk `lines` and collect every ATX heading outside fenced code blocks.
pub fn walk_headings(lines: &[&str]) -> Vec<Heading> {
    let mut out: Vec<Heading> = Vec::new();
    let mut tracker = FenceTracker::default();
    let mut in_comment = false;
    for (i, line) in lines.iter().enumerate() {
        // A line inside a multi-line `<!-- ... -->` block is comment text, not
        // a heading: a skeleton that quotes `## Heading` in its instructions
        // must not satisfy a contract that requires the real one.
        if in_comment {
            in_comment = !line.contains("-->");
            continue;
        }
        if tracker.feed(line) != LineKind::Outside {
            continue;
        }
        if let Some((level, mut text)) = parse_heading(line) {
            let block_id = strip_trailing_block_id(&mut text);
            out.push(Heading {
                level,
                text,
                line: i,
                block_id,
            });
        }
        in_comment = opens_comment(line);
    }
    out
}

/// Whether `line` opens an HTML comment that stays open past the line's end:
/// its last `<!--` has no `-->` after it.
fn opens_comment(line: &str) -> bool {
    line.rfind("<!--")
        .is_some_and(|open| !line[open + 4..].contains("-->"))
}

/// Strip a trailing Pandoc-style block attribute `{#id}` from
/// `text` if present. Returns `Some(id)` and mutates `text` to
/// remove the attribute + any whitespace immediately preceding it.
/// Returns `None` and leaves `text` untouched otherwise.
///
/// Recognized shape: `... {#<id>}` at end-of-text. `<id>` is one or
/// more non-whitespace, non-`}` characters. Permissive author syntax;
/// the engine doesn't restrict ID character set.
pub(crate) fn strip_trailing_block_id(text: &mut String) -> Option<String> {
    let trimmed_end = text.trim_end_matches(|c: char| c.is_whitespace());
    if !trimmed_end.ends_with('}') {
        return None;
    }
    // Walk backward from the `}` to find the matching `{#`.
    let close_byte = trimmed_end.len() - 1;
    let prefix = &trimmed_end[..close_byte];
    // Look for the latest `{#` that doesn't contain whitespace or `}`
    // between it and the close brace.
    let open_rel = prefix.rfind("{#")?;
    let inside = &prefix[open_rel + 2..];
    if inside.is_empty() || inside.chars().any(|c| c.is_whitespace() || c == '}') {
        return None;
    }
    // Found valid `{#id}` at the tail. Strip it + preceding whitespace.
    let id = inside.to_string();
    let pre = &trimmed_end[..open_rel];
    let pre_trimmed = pre.trim_end_matches(|c: char| c.is_whitespace());
    *text = pre_trimmed.to_string();
    Some(id)
}

/// The character of a fence run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FenceKind {
    Backtick,
    Tilde,
}

/// An opening code fence recognised under the CommonMark fence rules: a run
/// of three or more backticks or tildes indented at most three columns past
/// the enclosing list item's content column, and for a backtick fence an info
/// string with no backtick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Fence<'a> {
    kind: FenceKind,
    /// Length of the opening run; a closing fence must be at least this long.
    len: usize,
    /// Content column of the enclosing list item (0 at top level).
    container: usize,
    /// Everything after the opening run, untrimmed.
    pub(crate) info: &'a str,
}

impl<'a> Fence<'a> {
    fn opening(line: &'a str, container: usize) -> Option<Self> {
        let (kind, len, rest) = fence_run(line, container)?;
        if kind == FenceKind::Backtick && rest.contains('`') {
            return None;
        }
        Some(Self {
            kind,
            len,
            container,
            info: rest,
        })
    }

    /// Whether `line` closes this fence: same character, a run at least as
    /// long as the opener, indented at most three columns past the
    /// container, and nothing but whitespace after the run.
    fn closes(&self, line: &str) -> bool {
        fence_run(line, self.container).is_some_and(|(kind, len, rest)| {
            kind == self.kind && len >= self.len && rest.trim().is_empty()
        })
    }
}

/// Width in columns of the leading whitespace of `line` (a tab advances to
/// the next multiple of four) and its byte length.
fn leading_columns(line: &str) -> (usize, usize) {
    let mut col: usize = 0;
    let mut bytes: usize = 0;
    for b in line.bytes() {
        match b {
            b' ' => col += 1,
            b'\t' => col += 4 - col % 4,
            _ => break,
        }
        bytes += 1;
    }
    (col, bytes)
}

/// Split a fence line into its character, run length and remainder.
fn fence_run(line: &str, container: usize) -> Option<(FenceKind, usize, &str)> {
    let (col, bytes) = leading_columns(line);
    if col > container + 3 {
        return None;
    }
    let text: &str = &line[bytes..];
    let (kind, ch) = match text.as_bytes().first()? {
        b'`' => (FenceKind::Backtick, '`'),
        b'~' => (FenceKind::Tilde, '~'),
        _ => return None,
    };
    let len: usize = text.chars().take_while(|&c| c == ch).count();
    if len < 3 {
        return None;
    }
    Some((kind, len, &text[len..]))
}

/// The content column of a list item whose marker starts `line` (a bullet or
/// an ordered marker of up to nine digits, followed by whitespace or the end
/// of the line), or `None` when `line` is not a list item.
fn item_content_column(line: &str) -> Option<usize> {
    let (col, bytes) = leading_columns(line);
    let text: &str = &line[bytes..];
    let marker: usize = match text.as_bytes().first()? {
        b'-' | b'*' | b'+' => 1,
        b'0'..=b'9' => {
            let digits: usize = text.bytes().take_while(u8::is_ascii_digit).count();
            if digits > 9 || !matches!(text.as_bytes().get(digits), Some(b'.' | b')')) {
                return None;
            }
            digits + 1
        }
        _ => return None,
    };
    let after: &str = &text[marker..];
    let spaces: usize = after.bytes().take_while(|&b| b == b' ').count();
    if after.is_empty() || after.trim().is_empty() {
        return Some(col + marker + 1);
    }
    if spaces == 0 && !after.starts_with('\t') {
        return None;
    }
    // Five or more spaces after the marker: the content is an indented code
    // block, so the item content column is one space past the marker.
    let gap: usize = if (1..=4).contains(&spaces) { spaces } else { 1 };
    Some(col + marker + gap)
}

/// What a line is, as seen by [`FenceTracker::feed`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LineKind<'a> {
    /// Not part of any fenced block.
    Outside,
    /// The opening fence line of a block.
    Opens(Fence<'a>),
    /// A content line of an open block.
    Inside,
    /// The closing fence line of a block.
    Closes,
}

/// Line-by-line fence state: the open fence, if any, and the content columns
/// of the list items enclosing the current line, which the three-column
/// indent limit is measured from.
#[derive(Debug, Default)]
pub(crate) struct FenceTracker<'a> {
    open: Option<Fence<'a>>,
    containers: Vec<usize>,
}

impl<'a> FenceTracker<'a> {
    /// Classify the next line of the input. An unclosed fence stays open to
    /// the end of the input.
    pub(crate) fn feed(&mut self, line: &'a str) -> LineKind<'a> {
        if let Some(open) = self.open {
            if open.closes(line) {
                self.open = None;
                return LineKind::Closes;
            }
            return LineKind::Inside;
        }
        if line.trim().is_empty() {
            return LineKind::Outside;
        }
        let (col, _) = leading_columns(line);
        while self.containers.last().is_some_and(|&c| col < c) {
            self.containers.pop();
        }
        let container: usize = self.containers.last().copied().unwrap_or(0);
        if let Some(fence) = Fence::opening(line, container) {
            self.open = Some(fence);
            return LineKind::Opens(fence);
        }
        if let Some(content) = item_content_column(line) {
            self.containers.push(content);
        }
        LineKind::Outside
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
    use ix_trace_rs::trace;

    fn lines(s: &str) -> Vec<&str> {
        s.split('\n').collect()
    }

    #[trace("FR-007-AC-1")]
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

    #[trace("FR-007-AC-2")]
    #[test]
    fn unclosed_fence_swallows_trailing_lines() {
        let input = "## Real\n```\n## still-inside\n";
        let hs = walk_headings(&lines(input));
        assert_eq!(hs.len(), 1);
        assert_eq!(hs[0].text, "Real");
    }

    #[trace("FR-007-AC-3")]
    #[test]
    fn tilde_fence_behaves_like_backtick() {
        let input = "~~~\n# fake\n~~~\n## Real";
        let hs = walk_headings(&lines(input));
        assert_eq!(hs.len(), 1);
        assert_eq!(hs[0].text, "Real");
    }

    #[trace("FR-007-AC-4")]
    // mismatched fence character does not close an open fence.
    #[test]
    fn mismatched_fence_does_not_close_backtick_block() {
        let input = "```\n~~~\n# still-inside\n```\n## Real";
        let hs = walk_headings(&lines(input));
        assert_eq!(hs.len(), 1);
        assert_eq!(hs[0].text, "Real");
    }

    #[trace("FR-007-AC-5")]
    #[test]
    fn commonmark_fence_length_and_indent_rules() {
        let longer = "## A\n````\n```\n# fake\n````\n## B";
        let texts: Vec<String> = walk_headings(&lines(longer))
            .into_iter()
            .map(|h| h.text)
            .collect();
        assert_eq!(texts, vec!["A", "B"]);
        // Four spaces of indent is not a fence, so the heading stays live.
        let indented = "    ```\n## Live";
        assert_eq!(walk_headings(&lines(indented)).len(), 1);
    }

    #[trace("FR-007-AC-6")]
    #[test]
    fn heading_walk_closing_rules() {
        // A closing line with an info string is content, so the fence stays
        // open and swallows the heading.
        let info = "```\n```text\n## inside\n```\n## Out";
        let texts: Vec<String> = walk_headings(&lines(info))
            .into_iter()
            .map(|h| h.text)
            .collect();
        assert_eq!(texts, vec!["Out"]);
        // A shorter tilde run does not close a longer tilde fence.
        let tilde = "~~~~\n~~~\n## inside\n~~~~\n## Out";
        assert_eq!(walk_headings(&lines(tilde)).len(), 1);
    }

    #[trace("FR-007-AC-7")]
    #[test]
    fn list_nested_fences_measure_from_the_item_content() {
        // Ordered item (content column 4), nested bullet (column 6).
        let input = "1.  item\n\n    ```\n    # fake\n    ```\n\n    - nested\n\n      ```\n      ## fake2\n      ```\n## Real";
        let texts: Vec<String> = walk_headings(&lines(input))
            .into_iter()
            .map(|h| h.text)
            .collect();
        assert_eq!(texts, vec!["Real"]);
        // After the list ends, indentation is measured from column 0 again.
        let after = "- item\ntext\n\n    ```\n## H";
        assert_eq!(walk_headings(&lines(after)).len(), 1);
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

    #[trace("FR-019-AC-2")]
    // the `{#…}` attribute is stripped from the heading text.
    #[trace("FR-019-AC-2")]
    // the attribute is parsed off the heading text.
    #[test]
    fn pandoc_block_id_attribute_is_stripped_from_text() {
        let hs = walk_headings(&lines("## Behavior {#blk-7af2}"));
        assert_eq!(hs.len(), 1);
        assert_eq!(hs[0].text, "Behavior");
        assert_eq!(hs[0].block_id.as_deref(), Some("blk-7af2"));
    }

    #[trace("FR-019-AC-1")]
    // no attribute means no id, never a synthesized one.
    #[test]
    fn heading_without_attribute_has_no_block_id() {
        let hs = walk_headings(&lines("## Behavior"));
        assert_eq!(hs[0].text, "Behavior");
        assert!(hs[0].block_id.is_none());
    }

    #[trace("FR-019-AC-1")]
    // the id characters an author may write.
    #[test]
    fn block_id_allows_alphanumeric_dash_underscore() {
        let hs = walk_headings(&lines("### Section foo {#abc_123-XY}"));
        assert_eq!(hs[0].text, "Section foo");
        assert_eq!(hs[0].block_id.as_deref(), Some("abc_123-XY"));
    }

    #[test]
    fn malformed_attribute_with_whitespace_is_not_an_id() {
        // `{# blk}` (space after `{#`) → not a valid attribute.
        let hs = walk_headings(&lines("## Title {# blk}"));
        assert!(hs[0].block_id.is_none());
        // Text retains the literal `{# blk}`.
        assert_eq!(hs[0].text, "## Title {# blk}".trim_start_matches("## "));
    }

    #[test]
    fn block_id_survives_trailing_whitespace_before_attribute() {
        let hs = walk_headings(&lines("## A   {#blk-x}   "));
        assert_eq!(hs[0].text, "A");
        assert_eq!(hs[0].block_id.as_deref(), Some("blk-x"));
    }

    #[test]
    fn headings_inside_a_multi_line_html_comment_are_not_headings() {
        let hs = walk_headings(&lines(
            "## A\n<!--\n## Quoted\n-->\n## B\n<!-- one line ## X -->\n## C\n<!-- open ## Y\n## Z\n-->\n## D",
        ));
        let texts: Vec<&str> = hs.iter().map(|h| h.text.as_str()).collect();
        assert_eq!(texts, ["A", "B", "C", "D"]);
    }

    #[trace("FR-019-AC-3")]
    // the id survives a reparse.
    #[test]
    fn block_id_round_trip_parse_reparse_is_stable() {
        // Document is parsed, the section's block_id captured, and a
        // re-render (just the line) reparses to the same block_id.
        let original = "## Behavior {#blk-stable-7af2}";
        let hs = walk_headings(&lines(original));
        let id1 = hs[0].block_id.clone();
        // Reconstructing the line the way writeback would (heading +
        // text + " {#id}") then reparsing yields the same ID.
        let reconstructed = format!("{} {} {{#{}}}", "##", hs[0].text, id1.as_deref().unwrap());
        let hs2 = walk_headings(&lines(&reconstructed));
        assert_eq!(hs2[0].block_id, id1);
        assert_eq!(hs2[0].text, hs[0].text);
    }
}
