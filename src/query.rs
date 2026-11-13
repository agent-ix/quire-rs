//! Query API (FR-010, Layer 2).
//!
//! Read-only views over a parsed [`QuireDocument`]: section lookup,
//! pipe-delimited table parsing, bullet-list extraction, fenced code
//! block enumeration, and substring search. Mirrors
//! `~/dev/quire/src/core/query.ts` semantics so the Task 020 parity
//! sweep can compare structurally.
//!
//! All functions are pure and re-entrant. Regexes are compiled once
//! per call site via [`std::sync::OnceLock`].

use std::sync::OnceLock;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::ast::{QuireDocument, QuireSection};
use crate::parser::walk::Fence;

/// The document's concept **type** — the discriminator that selects which
/// archetype validates it. Reads frontmatter `type` exclusively. `None`
/// when the document carries no `type`.
///
/// This is the **one** canonical discriminator read. All routing — CLI
/// `validate`/`extract`/`lint`, corpus indexing, the Python bindings —
/// goes through here, so the field name can never drift apart across call
/// sites again.
pub fn concept_type(doc: &QuireDocument) -> Option<&str> {
    let fm = doc.frontmatter.as_ref()?;
    fm.get("type").and_then(|v| v.as_str())
}

// ─── Shared regexes (compile-once) ──────────────────────────────────────

fn re_section_number() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"^\d+(\.\d+)*\.?\s*").expect("section-number regex"))
}

fn re_bullet() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"^[-*+]\s+(.+)$").expect("bullet regex"))
}

fn re_bold_dash() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"^\*\*(.+?)\*\*\s*[\x{2014}\x{2013}\-]\s*(.+)$").expect("bold-dash regex")
    })
}

fn re_bold_colon() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"^\*\*(.+?)\*\*:\s*(.+)$").expect("bold-colon regex"))
}

fn re_table_separator() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"^\|?\s*[-:]+(\s*\|\s*[-:]+)*\s*\|?\s*$").expect("table-separator regex")
    })
}

fn re_heading() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"^(#{1,6})\s+(.+)$").expect("heading regex"))
}

fn re_info_word() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    // The language word: the leading word characters of the info string.
    R.get_or_init(|| Regex::new(r"^\s*(\w*)").expect("info-word regex"))
}

fn re_type_annotation() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"(?im)^%%\s*@type:\s*(\S+)").expect("@type regex"))
}

// ─── Public value types ─────────────────────────────────────────────────

/// Result of [`parse_table`] / [`parse_tables`] / [`table_from_section`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableResult {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

/// One item produced by [`parse_bullet_list`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListItem {
    /// Original bullet text minus the bullet marker.
    pub raw: String,
    /// Parsed title (for `BoldDescription` / `BoldColon`) or `raw` for
    /// `Plain`.
    pub title: String,
    /// Parsed description (for matched patterns) or `""` for `Plain`.
    pub description: String,
}

/// How [`parse_bullet_list`] should split each bullet into
/// `title` / `description`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ListPattern {
    /// `**Title** — description` (em/en/ascii dash). Default.
    #[default]
    BoldDescription,
    /// `**Title**: description`.
    BoldColon,
    /// No structure — `title == raw`, `description == ""`.
    Plain,
}

/// One fenced code block found by [`extract_diagrams`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagramBlock {
    /// 0-based ordinal of this block among the returned blocks: with no
    /// language filter it counts every fenced block; with a language filter
    /// it counts only the blocks of that language.
    pub index: usize,
    /// 1-based line of the opening fence. For [`extract_diagrams`] it is the
    /// line in the full file, frontmatter counted; for
    /// [`diagrams_from_content`] it is the line within the passed slice.
    pub line: usize,
    /// Info word after the whole opening fence (`""` if absent).
    pub language: String,
    /// The bytes between the opening and closing fences (joined by
    /// `\n`, no trailing newline).
    pub source: String,
    /// Heading text of the most-recent section above this block, or
    /// `None` if the block appears before any heading.
    pub section: Option<String>,
    /// `%% @type: <value>` annotation extracted from the source, if
    /// present.
    pub tag: Option<String>,
}

/// One match within a section, returned by [`search`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchMatch {
    /// 1-based line index within the document body where the hit
    /// occurred (mirrors the TS reference `section.startLine + 1 + i`).
    pub line: usize,
    pub text: String,
}

/// One section's hits, returned by [`search`].
#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult<'d> {
    pub section: &'d QuireSection,
    pub matches: Vec<SearchMatch>,
}

// ─── Section lookup (mirrors TS findSection / collectSections) ──────────

/// Strip a leading numeric prefix like `2.1`, `1.`, or `3.2.1 ` from a
/// heading and trim surrounding whitespace.
///
/// Used both by section-name matching (`section`/`matches_heading`) and by the
/// `from: heading` locator (`extract::locator::eval_heading`) so that ISO-style
/// section numbering is treated as decorative consistently: `## 2. Scope`
/// resolves the same as `## Scope`.
pub(crate) fn normalize_heading(heading: &str) -> String {
    let stripped = re_section_number().replace(heading, "");
    stripped.trim().to_string()
}

/// Case-insensitive match (with section-number prefix normalization).
/// TS `query.ts:matchesHeading` parity — note the FR-010 doc text says
/// "case-sensitive" but FR-010-AC-2 mandates TS parity; we follow TS.
fn matches_heading(section: &QuireSection, query: &str) -> bool {
    normalize_heading(&section.heading).to_lowercase() == normalize_heading(query).to_lowercase()
}

/// First section whose heading matches `heading` (depth-first).
pub fn section<'d>(doc: &'d QuireDocument, heading: &str) -> Option<&'d QuireSection> {
    find_section(&doc.sections, heading)
}

fn find_section<'d>(sections: &'d [QuireSection], heading: &str) -> Option<&'d QuireSection> {
    for s in sections {
        if matches_heading(s, heading) {
            return Some(s);
        }
        if let Some(found) = find_section(&s.children, heading) {
            return Some(found);
        }
    }
    None
}

/// Flatten the tree; optional `level` filters to a single heading
/// level. Iteration is depth-first / document order.
pub fn sections<'d>(doc: &'d QuireDocument, level: Option<u8>) -> Vec<&'d QuireSection> {
    let mut out: Vec<&'d QuireSection> = Vec::new();
    collect_sections(&doc.sections, level, &mut out);
    out
}

fn collect_sections<'d>(
    sections: &'d [QuireSection],
    level: Option<u8>,
    out: &mut Vec<&'d QuireSection>,
) {
    for s in sections {
        if level.is_none_or(|l| s.level == l) {
            out.push(s);
        }
        collect_sections(&s.children, level, out);
    }
}

// ─── Table parsing ──────────────────────────────────────────────────────

fn is_separator_row(line: &str) -> bool {
    re_table_separator().is_match(line)
}

/// Split one table row into cells on UNESCAPED pipes only (GFM:
/// `\|` inside a cell is a literal pipe, not a delimiter — CR-007).
/// Leading/trailing delimiter pipes are handled inside the scan so a
/// row ending in `\|` is not mis-trimmed. The escape is consumed:
/// `\|` yields `|` in the cell text; every other backslash is kept
/// verbatim.
pub(crate) fn split_row_unescaped(line: &str) -> Vec<String> {
    let mut cells: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut chars = line.trim().chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' if chars.peek() == Some(&'|') => {
                cur.push('|');
                chars.next();
            }
            '|' => {
                cells.push(cur.trim().to_string());
                cur.clear();
            }
            other => cur.push(other),
        }
    }
    cells.push(cur.trim().to_string());
    // A leading/trailing `|` delimiter produces an empty first/last
    // fragment — drop those (same semantics as the old prefix/suffix
    // strip, but escape-aware).
    if cells.first().is_some_and(|c| c.is_empty()) {
        cells.remove(0);
    }
    if cells.last().is_some_and(|c| c.is_empty()) {
        cells.pop();
    }
    cells
}

fn parse_cells(line: &str, header_count: Option<usize>) -> Vec<String> {
    let mut cells = split_row_unescaped(line);
    if let Some(n) = header_count {
        if cells.len() < n {
            cells.resize(n, String::new());
        } else if cells.len() > n {
            cells.truncate(n);
        }
    }
    cells
}

/// Parse the first pipe-delimited markdown table at or near the top of
/// `content`. Returns `None` if no header+separator pair is found.
pub fn parse_table(content: &str) -> Option<TableResult> {
    parse_table_with_lines(content).map(|(table, _, _)| table)
}

/// Like [`parse_table`], but also returns each row's and the header's 0-based
/// line index within `content` (#210, #341).
///
/// A separate seam rather than a field on [`TableResult`]: the public shape is
/// mirrored by downstream consumers, and the one caller that needs positions —
/// the declared-table scan, so a coverage record can say which authored line it
/// is about — is in-crate. The index is against the same `content.split('\n')`
/// numbering `ears::locate_line` uses, so both convert to a document line by
/// the same arithmetic.
pub(crate) fn parse_table_with_lines(content: &str) -> Option<(TableResult, Vec<usize>, usize)> {
    parse_tables_with_lines(content).into_iter().next()
}

/// Every pipe-delimited table in `content`, each with the same line data
/// [`parse_table_with_lines`] returns for the first one.
pub(crate) fn parse_tables_with_lines(content: &str) -> Vec<(TableResult, Vec<usize>, usize)> {
    let lines: Vec<&str> = content.split('\n').collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i + 1 < lines.len() {
        if !(lines[i].contains('|') && is_separator_row(lines[i + 1])) {
            i += 1;
            continue;
        }
        let headers = parse_cells(lines[i], None);
        let header_count = headers.len();
        let mut rows: Vec<Vec<String>> = Vec::new();
        let mut row_lines: Vec<usize> = Vec::new();
        let mut j = i + 2;
        while j < lines.len() {
            let t = lines[j].trim();
            if t.is_empty() || !t.contains('|') {
                break;
            }
            rows.push(parse_cells(lines[j], Some(header_count)));
            row_lines.push(j);
            j += 1;
        }
        out.push((TableResult { headers, rows }, row_lines, i));
        i = j;
    }
    out
}

/// Parse every pipe-delimited table in `content`.
pub fn parse_tables(content: &str) -> Vec<TableResult> {
    let lines: Vec<&str> = content.split('\n').collect();
    let mut out: Vec<TableResult> = Vec::new();
    let mut i: usize = 0;
    while i + 1 < lines.len() {
        if lines[i].contains('|') && is_separator_row(lines[i + 1]) {
            let headers = parse_cells(lines[i], None);
            let header_count = headers.len();
            let mut rows: Vec<Vec<String>> = Vec::new();
            let mut j = i + 2;
            while j < lines.len() {
                let t = lines[j].trim();
                if t.is_empty() || !t.contains('|') {
                    break;
                }
                rows.push(parse_cells(lines[j], Some(header_count)));
                j += 1;
            }
            out.push(TableResult { headers, rows });
            i = j;
        } else {
            i += 1;
        }
    }
    out
}

/// Convenience: parse the first table inside a named section.
pub fn table_from_section(doc: &QuireDocument, heading: &str) -> Option<TableResult> {
    let s = section(doc, heading)?;
    parse_table(&s.content)
}

// ─── Bullet list parsing ────────────────────────────────────────────────

/// Parse bullet-marker lines (`-`, `*`, `+`) from `content`.
///
/// The `pattern` controls how each bullet's text is split into
/// `title`/`description`. Falls back to the other pattern, then to
/// plain, when the requested pattern doesn't match.
pub fn parse_bullet_list(content: &str, pattern: Option<ListPattern>) -> Vec<ListItem> {
    parse_bullet_list_with_lines(content, pattern)
        .into_iter()
        .map(|(_, item)| item)
        .collect()
}

/// [`parse_bullet_list`] with each item's 0-based line index within `content`
/// (FR-033-AC-18). A separate seam for the same reason as
/// [`parse_table_with_lines`]: [`ListItem`]'s public shape is mirrored
/// downstream.
pub(crate) fn parse_bullet_list_with_lines(
    content: &str,
    pattern: Option<ListPattern>,
) -> Vec<(usize, ListItem)> {
    let pattern = pattern.unwrap_or_default();
    content
        .split('\n')
        .enumerate()
        .filter_map(|(index, line)| {
            let bullet_cap = re_bullet().captures(line.trim())?;
            let raw = bullet_cap.get(1).expect("bullet group").as_str();
            Some((index, classify_bullet(raw, pattern)))
        })
        .collect()
}

fn re_list_item() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"^(?:[-*+]|\d+[.)])\s+(.+)$").expect("list-item regex"))
}

/// The text of every bullet or ordered list item in `content`, at any
/// nesting depth, with its 0-based line index. Lines inside fenced code
/// blocks are not list items. A fence closes only on a run of its own
/// character at least as long as the opening run, with nothing after it
/// (CommonMark), so a shorter fence inside a longer one stays content.
pub(crate) fn list_item_lines(content: &str) -> Vec<(usize, &str)> {
    let mut fence: Option<(char, usize)> = None;
    let mut out = Vec::new();
    for (index, line) in content.split('\n').enumerate() {
        let trimmed = line.trim();
        match fence {
            Some((open, len)) => {
                if fence_run(trimmed)
                    .is_some_and(|(ch, run, rest)| ch == open && run >= len && rest.is_empty())
                {
                    fence = None;
                }
            }
            None => {
                if let Some((ch, run, _)) = fence_run(trimmed) {
                    fence = Some((ch, run));
                } else if let Some(item) = re_list_item().captures(trimmed).and_then(|c| c.get(1)) {
                    out.push((index, item.as_str()));
                }
            }
        }
    }
    out
}

/// A fence line's character, run length (at least 3) and trimmed remainder.
fn fence_run(line: &str) -> Option<(char, usize, &str)> {
    let ch = line.chars().next().filter(|c| matches!(c, '`' | '~'))?;
    let run = line.len() - line.trim_start_matches(ch).len();
    (run >= 3).then(|| (ch, run, line[run..].trim()))
}

fn classify_bullet(raw: &str, pattern: ListPattern) -> ListItem {
    let plain = |raw: &str| ListItem {
        raw: raw.to_string(),
        title: raw.to_string(),
        description: String::new(),
    };
    let from_cap = |cap: regex::Captures<'_>, raw: &str| ListItem {
        raw: raw.to_string(),
        title: cap.get(1).unwrap().as_str().to_string(),
        description: cap.get(2).unwrap().as_str().to_string(),
    };
    match pattern {
        ListPattern::Plain => plain(raw),
        ListPattern::BoldDescription => {
            if let Some(cap) = re_bold_dash().captures(raw) {
                return from_cap(cap, raw);
            }
            if let Some(cap) = re_bold_colon().captures(raw) {
                return from_cap(cap, raw);
            }
            plain(raw)
        }
        ListPattern::BoldColon => {
            if let Some(cap) = re_bold_colon().captures(raw) {
                return from_cap(cap, raw);
            }
            if let Some(cap) = re_bold_dash().captures(raw) {
                return from_cap(cap, raw);
            }
            plain(raw)
        }
    }
}

// ─── Diagram extraction ─────────────────────────────────────────────────

/// Enumerate fenced code blocks across the whole document, optionally
/// filtered by language tag, annotating each block with its nearest
/// heading (`DiagramBlock.section`).
///
/// This is the **document-wide harvest** query (US-010 / RAG): it scans
/// `doc.raw` (frontmatter stripped) and tracks the most-recent heading.
/// It is *not* the substrate for the `code_block` body-extraction
/// locator — that resolves from a section content slice via
/// [`diagrams_from_content`] so it stays section-owned and multi-yield
/// safe (FR-011).
pub fn extract_diagrams(doc: &QuireDocument, language: Option<&str>) -> Vec<DiagramBlock> {
    let body: &str = strip_frontmatter_view(&doc.raw);
    // The body begins after the frontmatter; its first line is the rest of the
    // closing `---` line, so body line `i` is full-file line `base + i + 1`.
    let skipped: &str = &doc.raw[..doc.raw.len() - body.len()];
    let base: usize = skipped.matches('\n').count();
    scan_fenced_blocks_with_lines(body, language, true)
        .into_iter()
        .map(|(_, block)| DiagramBlock {
            line: block.line + base,
            ..block
        })
        .collect()
}

/// Parse fenced code blocks from a **content slice**, optionally filtered
/// by language tag. The slice is already the scope, so blocks carry
/// `section: None`; only fence/language/`@type` logic is applied (no
/// heading tracking).
///
/// This is the section-owned substrate for the `code_block` locator: the
/// caller passes the target section's (or iteration unit's) `.content`,
/// guaranteeing containment and determinism under multi-yield.
pub fn diagrams_from_content(content: &str, language: Option<&str>) -> Vec<DiagramBlock> {
    diagrams_from_content_with_lines(content, language)
        .into_iter()
        .map(|(_, block)| block)
        .collect()
}

/// [`diagrams_from_content`] with the 0-based line index, within `content`, of
/// each block's opening fence (FR-033-AC-18).
pub(crate) fn diagrams_from_content_with_lines(
    content: &str,
    language: Option<&str>,
) -> Vec<(usize, DiagramBlock)> {
    scan_fenced_blocks_with_lines(content, language, false)
}

/// Core fenced-block scanner shared by [`extract_diagrams`] (whole-doc,
/// heading-tracking) and [`diagrams_from_content`] (slice, no headings).
/// When `track_headings` is false, every block's `section` is `None`.
fn scan_fenced_blocks_with_lines(
    body: &str,
    language: Option<&str>,
    track_headings: bool,
) -> Vec<(usize, DiagramBlock)> {
    let mut out: Vec<(usize, DiagramBlock)> = Vec::new();
    let mut fence_line: usize = 0;
    let mut current_section: Option<String> = None;
    // `Some(fence)` while inside a block opened by `fence`; `None` otherwise.
    let mut open_fence: Option<Fence<'_>> = None;
    let mut block_lang: String = String::new();
    let mut block_lines: Vec<String> = Vec::new();
    let mut block_index: usize = 0;

    for (line_index, line) in body.split('\n').enumerate() {
        if open_fence.is_none() && track_headings {
            if let Some(cap) = re_heading().captures(line) {
                current_section = Some(cap.get(2).unwrap().as_str().trim().to_string());
                continue;
            }
        }
        match open_fence {
            None => {
                if let Some(fence) = Fence::opening(line) {
                    block_lang = re_info_word()
                        .captures(fence.info)
                        .map(|c| c[1].to_string())
                        .unwrap_or_default();
                    open_fence = Some(fence);
                    block_lines.clear();
                    fence_line = line_index;
                }
            }
            // Close only on a fence of the same character and at least the
            // opening length; any other line is content (FR-010-AC-6).
            Some(open) if open.closes(line) => {
                push_block(
                    &mut out,
                    fence_line,
                    &mut block_index,
                    &block_lang,
                    &block_lines,
                    &current_section,
                    language,
                );
                open_fence = None;
                block_lang.clear();
                block_lines.clear();
            }
            Some(_) => {
                block_lines.push(line.to_string());
            }
        }
    }
    if open_fence.is_some() && !block_lines.is_empty() {
        push_block(
            &mut out,
            fence_line,
            &mut block_index,
            &block_lang,
            &block_lines,
            &current_section,
            language,
        );
    }
    out
}

fn push_block(
    out: &mut Vec<(usize, DiagramBlock)>,
    fence_line: usize,
    index: &mut usize,
    lang: &str,
    lines: &[String],
    section: &Option<String>,
    language_filter: Option<&str>,
) {
    if let Some(filter) = language_filter {
        if filter != lang {
            return;
        }
    }
    let source: String = lines.join("\n");
    let tag: Option<String> = re_type_annotation()
        .captures(&source)
        .map(|c| c.get(1).unwrap().as_str().to_string());
    out.push((
        fence_line,
        DiagramBlock {
            index: *index,
            line: fence_line + 1,
            language: lang.to_string(),
            source,
            section: section.clone(),
            tag,
        },
    ));
    *index += 1;
}

/// View into `raw` with any leading frontmatter block stripped. Mirrors
/// TS `doc.raw.slice(doc.raw.indexOf('\n---', 3) + 4)` when frontmatter
/// is present.
fn strip_frontmatter_view(raw: &str) -> &str {
    let stripped: &str = raw.strip_prefix('\u{FEFF}').unwrap_or(raw);
    if !(stripped.starts_with("---\n") || stripped.starts_with("---\r\n")) {
        return stripped;
    }
    let after_open: &str = if let Some(r) = stripped.strip_prefix("---\n") {
        r
    } else {
        stripped.strip_prefix("---\r\n").unwrap_or(stripped)
    };
    match after_open.find("\n---") {
        Some(end) => &after_open[end + 4..],
        None => stripped,
    }
}

// ─── Search ─────────────────────────────────────────────────────────────

/// Case-insensitive substring search across each section's content.
pub fn search<'d>(doc: &'d QuireDocument, query: &str) -> Vec<SearchResult<'d>> {
    let needle = query.to_lowercase();
    let mut out: Vec<SearchResult<'d>> = Vec::new();
    let all = sections(doc, None);
    for s in all {
        let mut matches: Vec<SearchMatch> = Vec::new();
        for (i, line) in s.content.split('\n').enumerate() {
            if line.to_lowercase().contains(&needle) {
                matches.push(SearchMatch {
                    line: s.start_line + 1 + i,
                    text: line.to_string(),
                });
            }
        }
        if !matches.is_empty() {
            out.push(SearchResult {
                section: s,
                matches,
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_document;
    use ix_trace_rs::trace;

    // ─── section / sections ─────────────────────────────────────────────

    #[test]
    fn section_matches_case_insensitively_and_strips_numbers() {
        let d = parse_document("## 2.1 In Scope\nbody\n## Out\nbody2");
        assert_eq!(section(&d, "in scope").unwrap().heading, "2.1 In Scope");
        assert_eq!(section(&d, "2.1 In Scope").unwrap().heading, "2.1 In Scope");
        assert_eq!(section(&d, "OUT").unwrap().heading, "Out");
        assert!(section(&d, "missing").is_none());
    }

    #[test]
    fn sections_filtered_by_level() {
        let d = parse_document("# A\n## B\n### C\n## D");
        assert_eq!(sections(&d, None).len(), 4);
        assert_eq!(sections(&d, Some(2)).len(), 2);
        assert_eq!(sections(&d, Some(3)).len(), 1);
        assert_eq!(sections(&d, Some(5)).len(), 0);
    }

    #[test]
    fn section_recurses_into_children() {
        let d = parse_document("## Parent\nx\n### Child\ny");
        assert_eq!(section(&d, "child").unwrap().heading, "Child");
    }

    // ─── parse_table / parse_tables / table_from_section ────────────────

    #[test]
    fn parse_table_handles_pipe_delimited_with_borders() {
        let content = "| a | b | c |\n| - | - | - |\n| 1 | 2 | 3 |\n| 4 | 5 | 6 |\n";
        let t = parse_table(content).unwrap();
        assert_eq!(t.headers, vec!["a", "b", "c"]);
        assert_eq!(t.rows, vec![vec!["1", "2", "3"], vec!["4", "5", "6"]]);
    }

    #[test]
    fn parse_table_returns_none_when_absent() {
        assert!(parse_table("no table here").is_none());
    }

    #[test]
    fn parse_table_normalizes_short_and_long_rows() {
        let content = "| a | b |\n| - | - |\n| 1 |\n| 1 | 2 | 3 |\n";
        let t = parse_table(content).unwrap();
        assert_eq!(t.rows, vec![vec!["1", ""], vec!["1", "2"]]);
    }

    #[test]
    fn parse_tables_finds_multiple() {
        let content = "| a | b |\n| - | - |\n| 1 | 2 |\n\nprose\n\n| c |\n| - |\n| 3 |\n";
        let tables = parse_tables(content);
        assert_eq!(tables.len(), 2);
        assert_eq!(tables[0].headers, vec!["a", "b"]);
        assert_eq!(tables[1].headers, vec!["c"]);
    }

    #[test]
    fn table_from_section_pulls_first_in_named_section() {
        let d = parse_document("## Stats\n| a |\n| - |\n| 1 |\n## Other\nx");
        let t = table_from_section(&d, "Stats").unwrap();
        assert_eq!(t.headers, vec!["a"]);
    }

    // ─── CR-007: escaped pipes are literal cell content ─────────────────

    #[trace("FR-010-AC-4")]
    // an escaped pipe is cell content, not a delimiter. (CR-007)
    #[test]
    fn cr007_escaped_pipe_in_body_cell_is_literal_not_delimiter() {
        // The cell that bit in the wild: `<service\|alias>` was split
        // into two cells, shifting the row and silently truncating the
        // last column.
        let content = "| Name | Type | Description |\n| - | - | - |\n\
                       | host | <service\\|alias> | upstream target |\n";
        let t = parse_table(content).unwrap();
        assert_eq!(
            t.rows,
            vec![vec!["host", "<service|alias>", "upstream target"]]
        );
    }

    #[test]
    fn cr007_escaped_pipe_in_header_cell() {
        let content = "| Value | A\\|B union |\n| - | - |\n| x | y |\n";
        let t = parse_table(content).unwrap();
        assert_eq!(t.headers, vec!["Value", "A|B union"]);
        assert_eq!(t.rows, vec![vec!["x", "y"]]);
    }

    #[test]
    fn cr007_cell_ending_in_escaped_pipe_keeps_trailing_pipe() {
        // A trailing `\|` must not be eaten by the border-trim.
        let content = "| a | b |\n| - | - |\n| ends-with\\| | z |\n";
        let t = parse_table(content).unwrap();
        assert_eq!(t.rows, vec![vec!["ends-with|", "z"]]);
    }

    #[test]
    fn cr007_multiple_escapes_and_enum_cells() {
        let content = "| Name | Values |\n| - | - |\n\
                       | mode | static\\|identity\\|hybrid |\n";
        let t = parse_table(content).unwrap();
        assert_eq!(t.rows, vec![vec!["mode", "static|identity|hybrid"]]);
    }

    #[test]
    fn cr007_non_pipe_backslashes_kept_verbatim() {
        // Only `\|` is an escape; regex-ish content like `\d+` or a
        // windows path keeps its backslashes.
        let content = "| Pattern | Path |\n| - | - |\n| ^TC-\\d+$ | C:\\tmp |\n";
        let t = parse_table(content).unwrap();
        assert_eq!(t.rows, vec![vec!["^TC-\\d+$", "C:\\tmp"]]);
    }

    #[test]
    fn cr007_borderless_rows_still_split_correctly() {
        let content = "a | b\n- | -\nleft\\|inside | right\n";
        let t = parse_table(content).unwrap();
        assert_eq!(t.headers, vec!["a", "b"]);
        assert_eq!(t.rows, vec![vec!["left|inside", "right"]]);
    }

    // ─── separator-row characterization (previously untested) ───────────

    #[test]
    fn separator_row_accepts_gfm_alignment_colons() {
        let content = "| left | center | right |\n|:--- |:---:| ---:|\n| 1 | 2 | 3 |\n";
        let t = parse_table(content).unwrap();
        assert_eq!(t.headers, vec!["left", "center", "right"]);
        assert_eq!(t.rows, vec![vec!["1", "2", "3"]]);
    }

    // ─── parse_bullet_list ──────────────────────────────────────────────

    #[test]
    fn bullets_default_to_bold_description() {
        let items = parse_bullet_list("- **Title** — desc here\n- **T2**: also desc", None);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "Title");
        assert_eq!(items[0].description, "desc here");
        // Fallback to bold-colon when bold-dash doesn't match.
        assert_eq!(items[1].title, "T2");
        assert_eq!(items[1].description, "also desc");
    }

    #[test]
    fn bullets_plain_pattern_keeps_raw_text() {
        let items = parse_bullet_list("* one\n+ two", Some(ListPattern::Plain));
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "one");
        assert_eq!(items[0].description, "");
        assert_eq!(items[1].title, "two");
    }

    #[test]
    fn bullets_unmatched_pattern_falls_back_to_raw() {
        let items = parse_bullet_list("- not a bold thing", Some(ListPattern::BoldDescription));
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "not a bold thing");
        assert_eq!(items[0].description, "");
    }

    // Previously untested: the bullet regex accepts all three GFM
    // markers (`-`, `*`, `+`), not just dashes.
    #[test]
    fn bullets_accept_star_and_plus_markers() {
        let items = parse_bullet_list("* starred\n+ plussed\n- dashed", None);
        let titles: Vec<&str> = items.iter().map(|i| i.title.as_str()).collect();
        assert_eq!(titles, vec!["starred", "plussed", "dashed"]);
    }

    // ─── extract_diagrams ───────────────────────────────────────────────

    #[test]
    fn diagrams_collected_with_section_context_and_language() {
        let input =
            "## Diagrams\n```mermaid\ngraph TD;\n  a-->b\n```\n## More\n```rust\nfn x() {}\n```";
        let d = parse_document(input);
        let all = extract_diagrams(&d, None);
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].language, "mermaid");
        assert_eq!(all[0].source, "graph TD;\n  a-->b");
        assert_eq!(all[0].section.as_deref(), Some("Diagrams"));
        assert_eq!(all[1].language, "rust");
        assert_eq!(all[1].section.as_deref(), Some("More"));
        // Filter by language.
        let m = extract_diagrams(&d, Some("mermaid"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn diagram_type_annotation_is_extracted() {
        let input = "## D\n```mermaid\n%% @type: sequence\nfoo\n```";
        let d = parse_document(input);
        let blocks = extract_diagrams(&d, None);
        assert_eq!(blocks[0].tag.as_deref(), Some("sequence"));
    }

    #[test]
    fn unclosed_diagram_block_is_emitted_as_final_block() {
        let input = "## D\n```mermaid\nfoo\nbar";
        let d = parse_document(input);
        let blocks = extract_diagrams(&d, None);
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].source.contains("foo"));
        assert!(blocks[0].source.contains("bar"));
    }

    #[test]
    fn diagrams_in_frontmatter_doc_skip_the_fence_block() {
        let input = "---\ntitle: x\n---\n## A\n```mermaid\ng\n```";
        let d = parse_document(input);
        let blocks = extract_diagrams(&d, None);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].section.as_deref(), Some("A"));
    }

    // ─── diagrams_from_content (slice substrate) ────────────────────────

    #[test]
    fn diagrams_from_content_parses_slice_blocks_with_section_none() {
        let content = "prose\n```mermaid\ngraph TD;\n  a-->b\n```\nmore\n```rust\nfn x() {}\n```";
        let all = diagrams_from_content(content, None);
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].language, "mermaid");
        assert_eq!(all[0].source, "graph TD;\n  a-->b");
        // The slice IS the scope — no whole-doc heading tracking.
        assert_eq!(all[0].section, None);
        assert_eq!(all[1].language, "rust");
        assert_eq!(all[1].section, None);
    }

    #[test]
    fn diagrams_from_content_filters_by_language() {
        let content = "```mermaid\ng\n```\n```rust\nfn x() {}\n```";
        let m = diagrams_from_content(content, Some("mermaid"));
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].language, "mermaid");
    }

    #[test]
    fn diagrams_from_content_extracts_type_annotation() {
        let content = "```mermaid\n%% @type: sequence\nfoo\n```";
        let blocks = diagrams_from_content(content, None);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].tag.as_deref(), Some("sequence"));
    }

    // ─── FR-011-AC-14: tilde (`~~~`) fence support, matching-char close ──

    #[test]
    fn tilde_fence_block_is_extracted() {
        let input = "## Diagrams\n~~~mermaid\ngraph TD;\n  a-->b\n~~~";
        let d = parse_document(input);
        let all = extract_diagrams(&d, None);
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].language, "mermaid");
        assert_eq!(all[0].source, "graph TD;\n  a-->b");
        assert_eq!(all[0].section.as_deref(), Some("Diagrams"));
    }

    #[test]
    fn tilde_fence_in_content_slice_is_extracted() {
        let content = "prose\n~~~mermaid\ngraph TD;\n  a-->b\n~~~\nmore";
        let all = diagrams_from_content(content, None);
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].language, "mermaid");
        assert_eq!(all[0].source, "graph TD;\n  a-->b");
        assert_eq!(all[0].section, None);
    }

    #[test]
    fn backtick_block_is_not_closed_by_tilde_line() {
        // A `~~~` line inside a ``` block is CONTENT, not a close.
        let input = "## D\n```mermaid\nfoo\n~~~\nbar\n```";
        let d = parse_document(input);
        let all = extract_diagrams(&d, None);
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].language, "mermaid");
        assert_eq!(all[0].source, "foo\n~~~\nbar");
    }

    #[test]
    fn tilde_block_is_not_closed_by_backtick_line() {
        // A ``` line inside a `~~~` block is CONTENT, not a close.
        let content = "~~~mermaid\nfoo\n```\nbar\n~~~";
        let all = diagrams_from_content(content, None);
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].language, "mermaid");
        assert_eq!(all[0].source, "foo\n```\nbar");
    }

    #[test]
    fn unclosed_tilde_block_is_emitted_as_final_block() {
        let content = "~~~mermaid\nfoo\nbar";
        let blocks = diagrams_from_content(content, None);
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].source.contains("foo"));
        assert!(blocks[0].source.contains("bar"));
    }

    // ─── FR-010 CR-008: fence line and CommonMark fence rules ───────────

    #[trace("FR-010-AC-5")]
    #[test]
    fn fence_line_is_one_based_in_the_full_file() {
        let bare = "## A\n```mermaid\ng\n```\n```mermaid\nh\n```";
        let lines: Vec<usize> = extract_diagrams(&parse_document(bare), None)
            .iter()
            .map(|b| b.line)
            .collect();
        assert_eq!(lines, vec![2, 5]);
        // Three frontmatter lines (`---`, `title: x`, `---`) shift every line.
        let fm = format!("---\ntitle: x\n---\n{bare}");
        let blocks = extract_diagrams(&parse_document(&fm), None);
        let lines: Vec<usize> = blocks.iter().map(|b| b.line).collect();
        assert_eq!(lines, vec![5, 8]);
        for b in &blocks {
            assert!(fm.split('\n').nth(b.line - 1).unwrap().starts_with("```"));
        }
        let slice = diagrams_from_content("p\n```mermaid\ng\n```", None);
        assert_eq!(slice[0].line, 2);
    }

    #[trace("FR-010-AC-6")]
    #[test]
    fn closing_fence_must_match_character_and_length() {
        let input =
            "## A\n````mermaid\nx\n```\n~~~~\n```text\n## inner\n````\n## B\n```rust\ny\n```";
        let blocks = extract_diagrams(&parse_document(input), None);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].source, "x\n```\n~~~~\n```text\n## inner");
        assert_eq!(blocks[0].section.as_deref(), Some("A"));
        assert_eq!(blocks[1].language, "rust");
        assert_eq!(blocks[1].section.as_deref(), Some("B"));
        // A longer closing run closes.
        let longer = diagrams_from_content("```a\nx\n`````\nafter", None);
        assert_eq!(longer.len(), 1);
        assert_eq!(longer[0].source, "x");
    }

    #[trace("FR-010-AC-7")]
    #[test]
    fn fence_indent_up_to_three_spaces() {
        let input = "   ```mermaid\na\n   ```\n    ```mermaid\nb\n    ```\n~~~mermaid\nc\n~~~\n``` x`y\nz\n```";
        let blocks = diagrams_from_content(input, None);
        let sources: Vec<&str> = blocks.iter().map(|b| b.source.as_str()).collect();
        assert_eq!(sources, vec!["a", "c"]);
        assert_eq!(blocks[1].line, 7);
    }

    #[trace("FR-010-AC-8")]
    #[test]
    fn language_is_read_after_the_whole_fence_and_index_counts_filtered() {
        let input = "```rust\nr\n```\n````mermaid\nm1\n````\n~~~~mermaid\nm2\n~~~~";
        let d = parse_document(input);
        let m = extract_diagrams(&d, Some("mermaid"));
        assert_eq!(m.len(), 2);
        assert_eq!((m[0].index, m[0].line), (0, 4));
        assert_eq!((m[1].index, m[1].line), (1, 7));
        assert_eq!(extract_diagrams(&d, None)[2].index, 2);
    }

    // ─── search ─────────────────────────────────────────────────────────

    #[test]
    fn search_returns_section_aware_hits() {
        let input = "## A\nfoo bar\nbaz\n## B\nFOO again";
        let d = parse_document(input);
        let r = search(&d, "foo");
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].section.heading, "A");
        assert_eq!(r[0].matches.len(), 1);
        assert_eq!(r[0].matches[0].text, "foo bar");
        assert_eq!(r[1].section.heading, "B");
    }

    #[test]
    fn search_returns_empty_when_no_match() {
        let d = parse_document("## A\nhello");
        assert!(search(&d, "missing").is_empty());
    }

    // ─── FR-010-AC-3: no quadratic walks ─────────────────────────────────
    //
    // Compare wall-clock at n vs 4n. Linear walks land at ~4×; a
    // quadratic walk lands at ~16×. We allow up to 12× as headroom
    // for jitter and constant-factor noise.

    #[test]
    fn sections_walk_scales_subquadratically() {
        fn measure(n: usize) -> std::time::Duration {
            let body: String = (0..n)
                .map(|i| format!("## H{}\n", i))
                .collect::<Vec<_>>()
                .join("");
            let d = parse_document(&body);
            // Best of three to dampen noise.
            let mut best = std::time::Duration::MAX;
            for _ in 0..3 {
                let t0 = std::time::Instant::now();
                let all = sections(&d, None);
                let dt = t0.elapsed();
                assert_eq!(all.len(), n);
                best = best.min(dt);
            }
            best
        }
        let small = measure(500);
        let big = measure(2000); // 4× n
                                 // Floor `small` so a sub-microsecond reading doesn't divide
                                 // the ratio into infinity.
        let small = small.max(std::time::Duration::from_nanos(1_000));
        let ratio = big.as_secs_f64() / small.as_secs_f64();
        assert!(
            ratio < 12.0,
            "sections walk looks super-linear: n=500 took {small:?}, n=2000 took {big:?} (ratio {ratio:.2})"
        );
    }
}
