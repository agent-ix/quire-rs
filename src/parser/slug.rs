//! Heading slugification (FR-009).
//!
//! ASCII-only slug normalization matching the TS/Py reference regex
//! `[^a-z0-9]+` → `-`. Non-ASCII characters (including diacritics like
//! `é`) collapse to `-` rather than being decomposed. Full Unicode
//! normalization (NFD + strip-diacritics) is deferred to v1.1 per the
//! spec note in FR-009 — adding it now would diverge from TS/Py.

/// Normalize `text` into an ASCII-alphanumeric slug.
///
/// - Lowercase via Unicode-default lowercasing.
/// - Every run of one-or-more characters not in `[a-z0-9]` collapses
///   to a single `-`.
/// - Leading and trailing `-` are stripped.
///
/// Returns the empty string when no `[a-z0-9]` characters remain
/// (e.g. `slug("!!!") == ""`). Callers that need a stable ID for such
/// headings should use [`slug_line_id`] instead.
pub fn slug(text: &str) -> String {
    // Pre-allocate assuming most chars survive; tighten later if needed.
    let mut out: String = String::with_capacity(text.len());
    let mut last_was_dash: bool = false;
    for c in text.chars().flat_map(char::to_lowercase) {
        if c.is_ascii_alphanumeric() {
            out.push(c);
            last_was_dash = false;
        } else if !last_was_dash {
            out.push('-');
            last_was_dash = true;
        }
    }
    // Strip leading/trailing dashes without re-allocating.
    let trimmed_len: usize = out.trim_end_matches('-').len();
    out.truncate(trimmed_len);
    let leading: usize = out.bytes().take_while(|&b| b == b'-').count();
    if leading > 0 {
        out.drain(..leading);
    }
    out
}

/// Build the stable `<slug>-L<line>` ID used for [`QuireSection`].
///
/// `line` is the 0-based line index of the heading within the body
/// (post-frontmatter-strip), per FR-009-AC-4.
///
/// [`QuireSection`]: crate::ast
pub fn slug_line_id(text: &str, line: usize) -> String {
    format!("{}-L{}", slug(text), line)
}

#[cfg(test)]
mod tests {
    use super::*;

    // FR-009-AC-1
    #[test]
    fn lowercase_alphanum_dash_basic() {
        assert_eq!(slug_line_id("Hello, World!", 0), "hello-world-L0");
    }

    // FR-009-AC-2
    #[test]
    fn numeric_and_punctuation_collapse_to_dashes() {
        assert_eq!(slug_line_id("2.1 In Scope", 6), "2-1-in-scope-L6");
    }

    // FR-009-AC-3
    #[test]
    fn leading_and_trailing_whitespace_stripped() {
        assert_eq!(slug_line_id("   leading spaces   ", 3), "leading-spaces-L3");
    }

    // FR-009-AC-6 / TC-190: non-ASCII collapses to '-' (matches TS/Py).
    #[test]
    fn non_ascii_collapses_to_dash() {
        assert_eq!(slug_line_id("Café Menu", 4), "caf-menu-L4");
    }

    // FR-009-AC-7 / TC-191: degenerate heading yields "-L<line>".
    #[test]
    fn degenerate_heading_keeps_line_index() {
        assert_eq!(slug_line_id("!!!", 7), "-L7");
    }

    #[test]
    fn empty_heading_yields_empty_slug_with_line() {
        assert_eq!(slug_line_id("", 0), "-L0");
    }

    #[test]
    fn runs_of_punctuation_collapse_to_single_dash() {
        assert_eq!(slug("a---b!!!c"), "a-b-c");
    }

    #[test]
    fn unicode_lowercase_then_ascii_filter() {
        // 'Ü' lowercases to 'ü' which is not ASCII alphanum → '-'.
        assert_eq!(slug("Über"), "ber");
    }

    #[test]
    fn emoji_collapses_to_dash_and_strips() {
        // Single emoji-only heading produces empty slug.
        assert_eq!(slug("❤️"), "");
        assert_eq!(slug_line_id("❤️", 2), "-L2");
    }

    #[test]
    fn embedded_digits_preserved() {
        assert_eq!(slug("v1.2.3-alpha"), "v1-2-3-alpha");
    }
}
