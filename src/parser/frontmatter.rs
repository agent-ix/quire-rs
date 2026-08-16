//! YAML frontmatter detection + extraction (FR-006).
//!
//! Ports the TS (`~/dev/quire/src/core/frontmatter.ts`) and Py
//! (`~/dev/quire-py/quire/frontmatter.py`) reference semantics:
//!
//! - No leading `---` fence → no frontmatter, body = input.
//! - Missing closing `---` line → no frontmatter, body = input.
//! - Malformed YAML between fences → no frontmatter, body = input
//!   (the entire input is treated as content; this is NOT an error).
//! - Valid YAML object → `Some(map)`, body = text after the closing fence.
//!
//! Extensions over TS/Py reference:
//!
//! - UTF-8 BOM (`\u{FEFF}`) at the start of input is stripped before
//!   frontmatter detection (FR-006-AC-5..6) and is not present in the
//!   returned body in any branch.
//! - CRLF (`\r\n`) line endings are accepted around the `---` fences.

use serde_json::{Map, Value};

/// Why [`extract_frontmatter`] produced (or did not produce) a frontmatter
/// mapping. This is the single authority for distinguishing an *absent*
/// frontmatter block from a *present-but-malformed* one, so boundary
/// consumers (e.g. the Filament extraction engine, FR-045) do not have to
/// re-derive that fact from the raw markdown.
///
/// It is an out-of-band discriminant only: the parity-observable outputs
/// (`frontmatter`/`body`) are byte-identical across all branches, so this
/// carries no TS/Py wire-shape impact (cf. the BOM/CRLF extensions above).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrontmatterStatus {
    /// No opening `---` fence, an opening fence with no closing `\n---`, or a
    /// complete fence block whose content is empty / whitespace / comments
    /// (parses to YAML null). Treated as a document with no usable frontmatter.
    Absent,
    /// A complete `---` … `---` block that parsed into a YAML mapping.
    Present,
    /// A complete `---` … `---` block was present, but its contents were not
    /// a parseable YAML mapping: invalid YAML, or a valid non-null, non-object
    /// value (array / scalar / bool / number). An empty / null block is
    /// [`Absent`](Self::Absent), not `Malformed`.
    Malformed,
}

/// Result of [`extract_frontmatter`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontmatterResult {
    /// Parsed YAML object, or `None` if absent / malformed / not an object.
    pub frontmatter: Option<Map<String, Value>>,
    /// Markdown body after the closing fence (post-BOM-strip). On the
    /// "no frontmatter" branches this is the entire post-BOM-strip input.
    pub body: String,
    /// Why `frontmatter` is `None`, or that it parsed. See [`FrontmatterStatus`].
    pub status: FrontmatterStatus,
}

/// Borrowed-body result of [`extract_frontmatter_ref`] — identical semantics
/// to [`FrontmatterResult`], with `body` as a slice of the input instead of a
/// copy. This is what makes the header tier ([`super::parse_header`], CR-046)
/// genuinely cheap: membership and identity are decided without copying the
/// file. Internal only; the public FR-006 surface is unchanged.
pub(crate) struct FrontmatterRef<'a> {
    pub frontmatter: Option<Map<String, Value>>,
    pub body: &'a str,
    pub status: FrontmatterStatus,
}

/// Extract YAML frontmatter from a markdown string per FR-006.
///
/// See module docs for the four branches.
pub fn extract_frontmatter(markdown: &str) -> FrontmatterResult {
    let r = extract_frontmatter_ref(markdown);
    FrontmatterResult {
        frontmatter: r.frontmatter,
        body: r.body.to_string(),
        status: r.status,
    }
}

/// The borrowed-body core of [`extract_frontmatter`]. In every branch `body`
/// is a suffix of the (BOM-stripped) input, so it borrows rather than copies.
pub(crate) fn extract_frontmatter_ref(markdown: &str) -> FrontmatterRef<'_> {
    let stripped: &str = markdown.strip_prefix('\u{FEFF}').unwrap_or(markdown);

    // Detect opening fence: "---\n" or "---\r\n" at the very start.
    let yaml_start: usize = if stripped.starts_with("---\n") {
        4
    } else if stripped.starts_with("---\r\n") {
        5
    } else {
        return no_frontmatter(stripped, FrontmatterStatus::Absent);
    };

    // First "\n---" closes the frontmatter (TS/Py parity:
    // `markdown.indexOf('\n---', 3)`). Match that exactly — being
    // stricter (requiring "---" to be on its own line) would diverge
    // from the reference and risk breaking the Task 020 parity suite.
    let after_open: &str = &stripped[yaml_start..];
    let close_rel: usize = match after_open.find("\n---") {
        Some(p) => p,
        // Opening fence but no closing fence: an unterminated block is not a
        // frontmatter block (conservative — could be a `---` thematic break).
        None => return no_frontmatter(stripped, FrontmatterStatus::Absent),
    };
    let body_tail: &str = &after_open[close_rel + 4..];

    // The YAML payload sits between the opening fence and the '\n' that
    // begins the closing fence. Drop a trailing '\r' for CRLF inputs.
    let yaml_raw: &str = &after_open[..close_rel];
    let yaml_str: &str = yaml_raw.strip_suffix('\r').unwrap_or(yaml_raw);

    let parsed: Value = match serde_yaml::from_str::<Value>(yaml_str) {
        Ok(v) => v,
        // A complete fence block whose contents are not parseable YAML.
        Err(_) => return no_frontmatter(stripped, FrontmatterStatus::Malformed),
    };

    let map: Map<String, Value> = match parsed {
        Value::Object(m) => m,
        // An empty / whitespace-only / comment-only fence block parses to
        // `Null`: it carries no metadata and is indistinguishable in intent
        // from having no frontmatter (and `---`\n\n`---` is a common pair of
        // thematic breaks). Treat it as *absent*, not a parse failure, so it
        // does not pollute Filament's index_errors (CR-011).
        Value::Null => return no_frontmatter(stripped, FrontmatterStatus::Absent),
        // Valid YAML but a non-mapping value (array / scalar / bool / number):
        // the spec types frontmatter as Map<String, Value>, so a wrong-typed
        // block is genuinely malformed. Still `frontmatter: None` for FR-006
        // parity; the status lets boundary consumers surface it.
        _ => return no_frontmatter(stripped, FrontmatterStatus::Malformed),
    };

    // Strip a single leading EOL from the body — it belongs to the
    // closing-fence line, not the body (TS: `.replace(/^\n/, '')`).
    let body: &str = body_tail
        .strip_prefix("\r\n")
        .or_else(|| body_tail.strip_prefix('\n'))
        .unwrap_or(body_tail);

    FrontmatterRef {
        frontmatter: Some(map),
        body,
        status: FrontmatterStatus::Present,
    }
}

fn no_frontmatter(stripped: &str, status: FrontmatterStatus) -> FrontmatterRef<'_> {
    FrontmatterRef {
        frontmatter: None,
        body: stripped,
        status,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fm_value(s: &str) -> Value {
        Value::String(s.to_string())
    }

    // FR-006-AC-1 / TC-012 (no-FM half)
    #[test]
    fn no_frontmatter_returns_input_as_body() {
        let r = extract_frontmatter("# heading");
        assert!(r.frontmatter.is_none());
        assert_eq!(r.body, "# heading");
        // No opening fence → the block is genuinely absent (not malformed).
        assert_eq!(r.status, FrontmatterStatus::Absent);
    }

    // FR-006-AC-2 / TC-012 (happy path)
    #[test]
    fn happy_path_parses_yaml_and_strips_fences() {
        let r = extract_frontmatter("---\nid: FR-001\n---\nbody");
        let map = r.frontmatter.expect("expected frontmatter");
        assert_eq!(map.get("id"), Some(&fm_value("FR-001")));
        assert_eq!(r.body, "body");
        assert_eq!(r.status, FrontmatterStatus::Present);
    }

    // FR-006-AC-3 / TC-013
    #[test]
    fn malformed_yaml_returns_body_fallback() {
        let input = "---\nid: : malformed\n---\nbody";
        let r = extract_frontmatter(input);
        assert!(r.frontmatter.is_none());
        assert_eq!(r.body, input);
        // A complete fence block with unparseable YAML is malformed, not absent
        // (CR-011): boundary consumers surface this as a parse failure.
        assert_eq!(r.status, FrontmatterStatus::Malformed);
    }

    // FR-006-AC-4 / TC-014
    #[test]
    fn unterminated_fence_returns_body_fallback() {
        let input = "---\nid: FR-001\nno closing fence\nbody";
        let r = extract_frontmatter(input);
        assert!(r.frontmatter.is_none());
        assert_eq!(r.body, input);
        // No closing fence → conservatively treated as absent (could be a
        // `---` thematic break), so it is NOT a parse failure.
        assert_eq!(r.status, FrontmatterStatus::Absent);
    }

    // FR-006-AC-5 / TC-180
    #[test]
    fn bom_prefix_with_frontmatter() {
        let r = extract_frontmatter("\u{FEFF}---\nid: FR-001\n---\nbody");
        let map = r.frontmatter.expect("expected frontmatter");
        assert_eq!(map.get("id"), Some(&fm_value("FR-001")));
        assert_eq!(r.body, "body");
    }

    // FR-006-AC-6 / TC-181
    #[test]
    fn bom_prefix_without_frontmatter() {
        let r = extract_frontmatter("\u{FEFF}# heading");
        assert!(r.frontmatter.is_none());
        assert_eq!(r.body, "# heading");
    }

    #[test]
    fn crlf_line_endings_around_fences() {
        let r = extract_frontmatter("---\r\nid: FR-001\r\n---\r\nbody\r\n");
        let map = r.frontmatter.expect("expected frontmatter");
        assert_eq!(map.get("id"), Some(&fm_value("FR-001")));
        assert_eq!(r.body, "body\r\n");
    }

    #[test]
    fn closing_fence_at_eof_no_trailing_newline() {
        let r = extract_frontmatter("---\nid: FR-001\n---");
        let map = r.frontmatter.expect("expected frontmatter");
        assert_eq!(map.get("id"), Some(&fm_value("FR-001")));
        assert_eq!(r.body, "");
    }

    #[test]
    fn body_preserves_leading_blank_lines_after_fence() {
        // After the closing "---\n", remaining bytes are returned verbatim:
        // no leading-newline trimming (the leading '\n' belongs to the
        // closing-fence line, not the body).
        let r = extract_frontmatter("---\nid: FR-001\n---\n\nbody");
        assert_eq!(r.body, "\nbody");
    }

    #[test]
    fn non_object_yaml_falls_back() {
        // A YAML array between fences is valid YAML but not an object;
        // we cannot represent it as Map<String, Value> so treat as no-FM.
        let input = "---\n- a\n- b\n---\nbody";
        let r = extract_frontmatter(input);
        assert!(r.frontmatter.is_none());
        assert_eq!(r.body, input);
        // Valid YAML but a non-object between complete fences → malformed.
        assert_eq!(r.status, FrontmatterStatus::Malformed);
    }

    // FR-006-AC-7 / TC-706 (empty/null → Absent half of the status classification)
    #[test]
    fn empty_or_comment_only_frontmatter_is_absent_not_malformed() {
        // A complete fence block whose content parses to YAML null (empty,
        // whitespace-only, or comment-only) carries no metadata and must NOT
        // be reported as a parse failure (CR-011) — it is treated as absent.
        for input in [
            "---\n\n---\nbody",
            "---\n   \n---\nbody",
            "---\n# only a comment\n---\nbody",
        ] {
            let r = extract_frontmatter(input);
            assert!(
                r.frontmatter.is_none(),
                "{input:?} should have no frontmatter"
            );
            assert_eq!(
                r.status,
                FrontmatterStatus::Absent,
                "{input:?} should be Absent, not Malformed"
            );
        }
    }

    #[test]
    fn first_close_fence_wins_ts_parity() {
        // TS/Py parity: `indexOf('\n---', 3)` takes the FIRST "\n---",
        // not the first standalone "---" line. The "---foo" prefix is
        // treated as the closing fence here, even though it's not a
        // true fence line. This matches the reference; the parity
        // suite (Task 020) depends on it.
        let r = extract_frontmatter("---\nid: FR-001\n---foo\n---\nbody");
        let map = r.frontmatter.expect("expected frontmatter");
        assert_eq!(map.get("id"), Some(&fm_value("FR-001")));
        assert_eq!(r.body, "foo\n---\nbody");
    }
}
