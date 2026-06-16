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

/// The document's concept **type** — the discriminator that selects which
/// archetype validates it. Reads frontmatter `type`, falling back to the
/// legacy `artifact_type` during the rename migration (the fallback is
/// removed once every corpus is on `type`). `None` when the document
/// carries neither.
///
/// This is the **one** canonical discriminator read. All routing — CLI
/// `validate`/`extract`/`lint`, corpus indexing, the Python bindings —
/// goes through here, so the field name and fallback order can never
/// drift apart across call sites again.
pub fn concept_type(doc: &QuireDocument) -> Option<&str> {
    let fm = doc.frontmatter.as_ref()?;
    fm.get("type")
        .or_else(|| fm.get("artifact_type"))
        .and_then(|v| v.as_str())
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

fn re_fence_open() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    // Either fence character (``` or ~~~), capturing the info-string word.
    // Mirrors the parser's FR-007 fence model so the scanner and the
    // heading walk agree on what opens a block.
    R.get_or_init(|| Regex::new(r"^(?:```|~~~)(\w*)").expect("fence-open regex"))
}

/// The fence character that opened a code block. A block opened with one
/// kind is closed ONLY by a fence line of the same kind; a mismatched
/// fence line is content (mirrors `walk.rs::FenceKind` + FR-007-AC-4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FenceKind {
    Backtick,
    Tilde,
}

impl FenceKind {
    /// The fence kind a `trim_start`ed line opens or closes, if any.
    fn of_line(trimmed_start: &str) -> Option<Self> {
        if trimmed_start.starts_with("```") {
            Some(Self::Backtick)
        } else if trimmed_start.starts_with("~~~") {
            Some(Self::Tilde)
        } else {
            None
        }
    }
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
    /// 0-based index in the document (counts all fenced blocks).
    pub index: usize,
    /// Language tag after the opening backticks (`""` if absent).
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
        if level.map_or(true, |l| s.level == l) {
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
fn split_row_unescaped(line: &str) -> Vec<String> {
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
    let lines: Vec<&str> = content.split('\n').collect();
    let header_line: usize = lines
        .windows(2)
        .position(|w| w[0].contains('|') && is_separator_row(w[1]))?;
    let headers = parse_cells(lines[header_line], None);
    let header_count = headers.len();
    let mut rows: Vec<Vec<String>> = Vec::new();
    for line in lines.iter().skip(header_line + 2) {
        let t = line.trim();
        if t.is_empty() || !t.contains('|') {
            break;
        }
        rows.push(parse_cells(line, Some(header_count)));
    }
    Some(TableResult { headers, rows })
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
    let pattern = pattern.unwrap_or_default();
    let mut items: Vec<ListItem> = Vec::new();

    for line in content.split('\n') {
        let bullet_cap = match re_bullet().captures(line.trim()) {
            Some(c) => c,
            None => continue,
        };
        let raw: String = bullet_cap
            .get(1)
            .expect("bullet group")
            .as_str()
            .to_string();
        let item = classify_bullet(&raw, pattern);
        items.push(item);
    }
    items
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
    scan_fenced_blocks(body, language, true)
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
    scan_fenced_blocks(content, language, false)
}

/// Core fenced-block scanner shared by [`extract_diagrams`] (whole-doc,
/// heading-tracking) and [`diagrams_from_content`] (slice, no headings).
/// When `track_headings` is false, every block's `section` is `None`.
fn scan_fenced_blocks(
    body: &str,
    language: Option<&str>,
    track_headings: bool,
) -> Vec<DiagramBlock> {
    let mut out: Vec<DiagramBlock> = Vec::new();
    let mut current_section: Option<String> = None;
    // `Some(kind)` while inside a block opened by `kind`; `None` otherwise.
    let mut open_fence: Option<FenceKind> = None;
    let mut block_lang: String = String::new();
    let mut block_lines: Vec<String> = Vec::new();
    let mut block_index: usize = 0;

    for line in body.split('\n') {
        if open_fence.is_none() && track_headings {
            if let Some(cap) = re_heading().captures(line) {
                current_section = Some(cap.get(2).unwrap().as_str().trim().to_string());
                continue;
            }
        }
        let trimmed_start = line.trim_start();
        match open_fence {
            None => {
                if let Some(cap) = re_fence_open().captures(trimmed_start) {
                    // The capture only matches after a valid fence open, so
                    // the line's fence kind is necessarily `Some`.
                    open_fence = FenceKind::of_line(trimmed_start);
                    block_lang = cap
                        .get(1)
                        .map(|m| m.as_str().to_string())
                        .unwrap_or_default();
                    block_lines.clear();
                }
            }
            // Close ONLY on a matching-character fence line; a mismatched
            // fence (e.g. `~~~` inside a ``` block) is content (FR-007-AC-4).
            Some(open) if FenceKind::of_line(trimmed_start) == Some(open) => {
                push_block(
                    &mut out,
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
    out: &mut Vec<DiagramBlock>,
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
    out.push(DiagramBlock {
        index: *index,
        language: lang.to_string(),
        source,
        section: section.clone(),
        tag,
    });
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
