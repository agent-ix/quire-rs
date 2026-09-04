//! Line-level scanning shared by the Properties, Invariants, and Operations
//! extractors: section bounds, fenced blocks with their lines, tables, and
//! bullet runs. Fence recognition follows the parser (`parser::walk`): a run
//! of three or more backticks or tildes at the start of the line opens a
//! block; a run of the same character at least as long closes it.

/// One fenced block inside a line range, with 1-based document lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fence {
    pub language: String,
    /// Line of the opening fence.
    pub open_line: usize,
    /// Line of the closing fence; `None` when unterminated.
    pub close_line: Option<usize>,
    /// Byte length of the closing fence line (for `endColumn`).
    pub close_len: usize,
    /// The body bytes between the fences, final line terminator excluded,
    /// CR bytes preserved.
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FenceChar {
    Backtick,
    Tilde,
}

/// `(kind, run length, language tag)` when `line` opens a fence.
pub fn fence_open(line: &str) -> Option<(FenceChar, usize, String)> {
    let trimmed = line.trim_end_matches('\r');
    // Up to three leading spaces, as the parser's `fence_kind` allows.
    let indent = trimmed.len() - trimmed.trim_start_matches(' ').len();
    if indent > 3 {
        return None;
    }
    let trimmed = &trimmed[indent..];
    let first = trimmed.chars().next()?;
    let kind = match first {
        '`' => FenceChar::Backtick,
        '~' => FenceChar::Tilde,
        _ => return None,
    };
    let run = trimmed.chars().take_while(|c| *c == first).count();
    if run < 3 {
        return None;
    }
    let tag = trimmed[run..].split_whitespace().next().unwrap_or("");
    Some((kind, run, tag.to_string()))
}

fn fence_close(line: &str, kind: &FenceChar, open_len: usize) -> bool {
    let trimmed = line.trim_end_matches('\r').trim_end();
    let indent = trimmed.len() - trimmed.trim_start_matches(' ').len();
    if indent > 3 {
        return false;
    }
    let trimmed = &trimmed[indent..];
    let ch = match kind {
        FenceChar::Backtick => '`',
        FenceChar::Tilde => '~',
    };
    let run = trimmed.chars().take_while(|c| *c == ch).count();
    run >= open_len && trimmed.chars().all(|c| c == ch)
}

/// Split `raw` into lines keeping CR bytes; `lines[i]` is document line i+1.
pub fn lines(raw: &str) -> Vec<&str> {
    raw.split('\n').collect()
}

/// Find every `## <heading>` (exact, trimmed) at level 2, returning the
/// 1-based heading line and the exclusive end line (next `##`/`#` heading
/// outside a fence, or end of document).
pub fn level2_sections(lines: &[&str], heading: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut open: Option<(FenceChar, usize)> = None;
    let mut current: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        let text = line.trim_end_matches('\r');
        if let Some((kind, len)) = &open {
            if fence_close(text, kind, *len) {
                open = None;
            }
            continue;
        }
        if let Some((kind, len, _)) = fence_open(text) {
            open = Some((kind, len));
            continue;
        }
        let is_level_le2 =
            (text.starts_with("# ") || text.starts_with("## ")) && !text.starts_with("###");
        if is_level_le2 {
            if let Some(start) = current.take() {
                out.push((start, i + 1));
            }
            if let Some(rest) = text.strip_prefix("## ") {
                if rest.trim() == heading {
                    current = Some(i + 1);
                }
            }
        }
    }
    if let Some(start) = current {
        out.push((start, lines.len() + 1));
    }
    out
}

/// Every fenced block whose opening fence lies in `[from, to)` (1-based).
pub fn fences_in(lines: &[&str], from: usize, to: usize) -> Vec<Fence> {
    let mut out = Vec::new();
    let mut i = from;
    while i < to && i <= lines.len() {
        let line = lines[i - 1];
        if let Some((kind, len, language)) = fence_open(line) {
            let open_line = i;
            let mut body: Vec<&str> = Vec::new();
            let mut close = None;
            let mut j = i + 1;
            while j <= lines.len() {
                if fence_close(lines[j - 1], &kind, len) {
                    close = Some(j);
                    break;
                }
                body.push(lines[j - 1]);
                j += 1;
            }
            let close_len = close
                .map(|c| lines[c - 1].trim_end_matches('\r').len())
                .unwrap_or(0);
            out.push(Fence {
                language,
                open_line,
                close_line: close,
                close_len,
                body: body.join("\n"),
            });
            i = close.map(|c| c + 1).unwrap_or(lines.len() + 1);
        } else {
            i += 1;
        }
    }
    out
}

/// A pipe table: header cells, then `(line, cells)` per data row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    pub line: usize,
    pub headers: Vec<String>,
    pub rows: Vec<(usize, Vec<String>)>,
}

/// Split a pipe row into trimmed cells; `\|` is a literal pipe.
pub fn split_row(line: &str) -> Vec<String> {
    let text = line.trim_end_matches('\r').trim();
    let text = text.strip_prefix('|').unwrap_or(text);
    let text = text.strip_suffix('|').unwrap_or(text);
    let mut cells = Vec::new();
    let mut cell = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' && chars.peek() == Some(&'|') {
            cell.push('|');
            chars.next();
        } else if c == '|' {
            cells.push(cell.trim().to_string());
            cell.clear();
        } else {
            cell.push(c);
        }
    }
    cells.push(cell.trim().to_string());
    cells
}

fn is_separator(line: &str) -> bool {
    let t = line.trim_end_matches('\r').trim();
    let t = t.strip_prefix('|').unwrap_or(t);
    !t.is_empty()
        && t.split('|').all(|c| {
            let c = c.trim();
            c.is_empty() || (c.chars().all(|ch| ch == '-' || ch == ':') && c.contains("---"))
        })
}

/// Blocks in a section, in order: what the Properties classifier reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Fence(Fence),
    Table(Table),
    /// First line of a bullet-list run.
    List {
        line: usize,
    },
}

/// Scan `[from, to)` for fences, tables, and bullet runs (quoin FR-074's
/// classifier, line for line).
pub fn blocks_in(lines: &[&str], from: usize, to: usize) -> Vec<Block> {
    let mut out = Vec::new();
    let mut i = from;
    let mut in_list = false;
    while i < to && i <= lines.len() {
        let line = lines[i - 1];
        let text = line.trim_end_matches('\r');
        if let Some((kind, len, language)) = fence_open(text) {
            let mut body: Vec<&str> = Vec::new();
            let mut close = None;
            let mut j = i + 1;
            while j <= lines.len() {
                if fence_close(lines[j - 1], &kind, len) {
                    close = Some(j);
                    break;
                }
                body.push(lines[j - 1]);
                j += 1;
            }
            let close_len = close
                .map(|c| lines[c - 1].trim_end_matches('\r').len())
                .unwrap_or(0);
            out.push(Block::Fence(Fence {
                language,
                open_line: i,
                close_line: close,
                close_len,
                body: body.join("\n"),
            }));
            in_list = false;
            i = close.map(|c| c + 1).unwrap_or(lines.len() + 1);
            continue;
        }
        if text.trim_start().starts_with('|') {
            let next = lines.get(i).map(|l| l.trim_end_matches('\r')).unwrap_or("");
            if is_separator(next) {
                let headers = split_row(text);
                let mut rows = Vec::new();
                let mut j = i + 2;
                while j <= lines.len() && j < to && lines[j - 1].trim_start().starts_with('|') {
                    rows.push((j, split_row(lines[j - 1])));
                    j += 1;
                }
                out.push(Block::Table(Table {
                    line: i,
                    headers,
                    rows,
                }));
                in_list = false;
                i = j;
                continue;
            }
            i += 1;
            continue;
        }
        let t = text.trim_start();
        if (t.starts_with("- ") || t.starts_with("* ")) && t.len() > 2 {
            if !in_list {
                out.push(Block::List { line: i });
                in_list = true;
            }
        } else if !t.is_empty() {
            in_list = false;
        }
        i += 1;
    }
    out
}

/// Line numbers in `[from, to)` that are not inside a fenced block (the
/// fence lines themselves excluded): where `Clause:`, `Returns:`, `Pre:`,
/// and `Post:` lines may be read (FR-071-CON-1: fence interiors are opaque).
pub fn lines_outside_fences(lines: &[&str], from: usize, to: usize) -> Vec<usize> {
    let fences = fences_in(lines, from, to);
    (from..to.min(lines.len() + 1))
        .filter(|&l| {
            !fences
                .iter()
                .any(|f| l >= f.open_line && f.close_line.map_or(true, |c| l <= c))
        })
        .collect()
}
