//! QSL declaration-head adapter (FR-094).
//!
//! Reads a `.native` or `.qsl` file line by line and mints one `declaration`
//! symbol for every declaration head. It is deliberately not a second QSL grammar
//! (FR-094-CON-1): a head is a column-0 line at brace depth 0 whose first two
//! tokens are a lowercase keyword and an identifier, and the only keywords it
//! knows are the three file-header ones it must skip. QSL owns the rest.
//!
//! String and comment handling here is this adapter's own approximation of the
//! QSL lexer, used only to decide which braces count toward depth: a `//`
//! comment ends the scan of its line, a `"` opens a string that a `\` escapes
//! into and that never spans a line.

use std::sync::OnceLock;

use regex::Regex;

use super::{RawSymbol, SymbolKind};

/// Keywords that open a file header rather than a declaration.
const HEADER_KEYWORDS: [&str; 3] = ["language", "profile", "model"];

/// What one pass over a `.native` or `.qsl` file found.
#[derive(Debug, Default)]
pub(crate) struct QslScan {
    /// Declaration heads, in source order. Empty when braces are unbalanced.
    pub symbols: Vec<RawSymbol>,
    /// 1-based line of the stray `}` or, when braces remain open at end of file,
    /// of the outermost unclosed `{`.
    pub unbalanced_at: Option<usize>,
}

fn head_pattern() -> &'static Regex {
    static HEAD: OnceLock<Regex> = OnceLock::new();
    HEAD.get_or_init(|| {
        Regex::new(r"^([a-z][a-z0-9_]*)\s+([A-Za-z_][A-Za-z0-9_]*)").expect("qsl head pattern")
    })
}

/// Walk `source` once, tracking brace depth and collecting declaration heads.
pub(crate) fn scan(source: &str) -> QslScan {
    let head = head_pattern();
    let mut symbols: Vec<RawSymbol> = Vec::new();
    // Index into `symbols` of a declaration whose body has not closed yet.
    let mut open_decl: Option<usize> = None;
    let mut open_braces: Vec<usize> = Vec::new();
    let mut stray_close: Option<usize> = None;
    // First line of the run of `//` lines directly above the current line.
    let mut comment_run_start: Option<usize> = None;

    for (index, text) in source.lines().enumerate() {
        let line = index + 1;
        let at_depth_zero = open_braces.is_empty();
        let at_column_zero = text.chars().next().is_some_and(|c| !c.is_whitespace());

        if at_depth_zero && at_column_zero {
            if let Some(caps) = head.captures(text) {
                if !HEADER_KEYWORDS.contains(&&caps[1]) {
                    symbols.push(RawSymbol {
                        attributes: Vec::new(),
                        position: None,
                        qualified_name: caps[2].to_string(),
                        kind: SymbolKind::Declaration,
                        line,
                        column: 1,
                        leading_line: comment_run_start.unwrap_or(line),
                        end_line: line,
                        container: None,
                        tag_comments: Vec::new(),
                        ignored: false,
                    });
                    open_decl = Some(symbols.len() - 1);
                }
            }
        }

        scan_braces(text, line, &mut open_braces, &mut stray_close);

        if open_braces.is_empty() {
            if let Some(decl) = open_decl.take() {
                symbols[decl].end_line = line;
            }
        }

        comment_run_start = if text.trim_start().starts_with("//") {
            comment_run_start.or(Some(line))
        } else {
            None
        };
    }

    let unbalanced_at = stray_close.or_else(|| open_braces.first().copied());
    if unbalanced_at.is_some() {
        symbols.clear();
    }
    QslScan {
        symbols,
        unbalanced_at,
    }
}

/// Apply the braces of one line to the depth stack.
fn scan_braces(
    text: &str,
    line: usize,
    open_braces: &mut Vec<usize>,
    stray_close: &mut Option<usize>,
) {
    let mut in_string = false;
    let mut escaped = false;
    let mut previous_slash = false;
    for c in text.chars() {
        if in_string {
            match (escaped, c) {
                (true, _) => escaped = false,
                (false, '\\') => escaped = true,
                (false, '"') => in_string = false,
                _ => {}
            }
            continue;
        }
        match c {
            '"' => in_string = true,
            '/' if previous_slash => return,
            '{' => open_braces.push(line),
            '}' => match open_braces.pop() {
                Some(_) => {}
                None => {
                    stray_close.get_or_insert(line);
                }
            },
            _ => {}
        }
        previous_slash = c == '/';
    }
}

#[cfg(test)]
mod tests {
    use ix_trace_rs::trace;

    use super::*;

    fn names(scan: &QslScan) -> Vec<&str> {
        scan.symbols
            .iter()
            .map(|s| s.qualified_name.as_str())
            .collect()
    }

    #[trace("FR-094-AC-9")]
    #[test]
    fn heads_exclude_bodies_indented_lines_and_header_keywords() {
        let source = "language qsl\nprofile core\nmodel M\n\
                      pre P1 using v on M::S::op {\n  pre Inner using v\n}\n\
                      post P2 {\n}\n  indented Ignored\n";
        let out = scan(source);
        assert_eq!(names(&out), ["P1", "P2"]);
        assert_eq!(out.unbalanced_at, None);
    }

    #[trace("FR-094-AC-9")]
    #[test]
    fn braces_in_comments_and_strings_do_not_change_depth() {
        let source = "// {\npre A { \"}\" }\nrule B // }\npre C\n";
        let out = scan(source);
        assert_eq!(names(&out), ["A", "B", "C"]);
        assert_eq!(out.unbalanced_at, None);
    }

    #[trace("FR-094-AC-17")]
    #[test]
    fn escaped_quote_keeps_the_string_open_and_unterminated_strings_end_at_the_line() {
        let escaped = scan("pre A \"a\\\"{\"\npre B\n");
        assert_eq!(names(&escaped), ["A", "B"]);
        let unterminated = scan("pre A \"open {\npre B\n");
        assert_eq!(names(&unterminated), ["A", "B"]);
    }

    #[trace("FR-094-AC-16")]
    #[test]
    fn unbalanced_braces_name_the_outermost_open_or_the_stray_close() {
        let open = scan("pre A {\n  x {\n");
        assert_eq!(open.unbalanced_at, Some(1));
        assert!(open.symbols.is_empty());
        let stray = scan("pre A\n}\npre B\n");
        assert_eq!(stray.unbalanced_at, Some(2));
        assert!(stray.symbols.is_empty());
    }

    #[trace("FR-094-AC-1")]
    #[test]
    fn leading_line_is_the_comment_run_directly_above() {
        let out = scan("pre Z\n\n// a\n// b\npre P1 {\n}\n");
        let p1 = &out.symbols[1];
        assert_eq!((p1.leading_line, p1.line, p1.end_line), (3, 5, 6));
    }
}
