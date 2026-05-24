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

/// Result of [`extract_frontmatter`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontmatterResult {
    /// Parsed YAML object, or `None` if absent / malformed / not an object.
    pub frontmatter: Option<Map<String, Value>>,
    /// Markdown body after the closing fence (post-BOM-strip). On the
    /// "no frontmatter" branches this is the entire post-BOM-strip input.
    pub body: String,
}

/// Extract YAML frontmatter from a markdown string per FR-006.
///
/// See module docs for the four branches.
pub fn extract_frontmatter(markdown: &str) -> FrontmatterResult {
    let stripped: &str = markdown.strip_prefix('\u{FEFF}').unwrap_or(markdown);

    // Detect opening fence: "---\n" or "---\r\n" at the very start.
    let yaml_start: usize = if stripped.starts_with("---\n") {
        4
    } else if stripped.starts_with("---\r\n") {
        5
    } else {
        return no_frontmatter(stripped);
    };

    // First "\n---" closes the frontmatter (TS/Py parity:
    // `markdown.indexOf('\n---', 3)`). Match that exactly — being
    // stricter (requiring "---" to be on its own line) would diverge
    // from the reference and risk breaking the Task 020 parity suite.
    let after_open: &str = &stripped[yaml_start..];
    let close_rel: usize = match after_open.find("\n---") {
        Some(p) => p,
        None => return no_frontmatter(stripped),
    };
    let body_tail: &str = &after_open[close_rel + 4..];

    // The YAML payload sits between the opening fence and the '\n' that
    // begins the closing fence. Drop a trailing '\r' for CRLF inputs.
    let yaml_raw: &str = &after_open[..close_rel];
    let yaml_str: &str = yaml_raw.strip_suffix('\r').unwrap_or(yaml_raw);

    let parsed: Value = match serde_yaml::from_str::<Value>(yaml_str) {
        Ok(v) => v,
        Err(_) => return no_frontmatter(stripped),
    };

    let map: Map<String, Value> = match parsed {
        Value::Object(m) => m,
        // Valid YAML but not an object (null / array / scalar) → no FM.
        // The spec types frontmatter as Map<String, Value>; non-objects
        // cannot be represented and are treated as the malformed branch.
        _ => return no_frontmatter(stripped),
    };

    // Strip a single leading EOL from the body — it belongs to the
    // closing-fence line, not the body (TS: `.replace(/^\n/, '')`).
    let body: &str = body_tail
        .strip_prefix("\r\n")
        .or_else(|| body_tail.strip_prefix('\n'))
        .unwrap_or(body_tail);

    FrontmatterResult {
        frontmatter: Some(map),
        body: body.to_string(),
    }
}

fn no_frontmatter(stripped: &str) -> FrontmatterResult {
    FrontmatterResult {
        frontmatter: None,
        body: stripped.to_string(),
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
    }

    // FR-006-AC-2 / TC-012 (happy path)
    #[test]
    fn happy_path_parses_yaml_and_strips_fences() {
        let r = extract_frontmatter("---\nid: FR-001\n---\nbody");
        let map = r.frontmatter.expect("expected frontmatter");
        assert_eq!(map.get("id"), Some(&fm_value("FR-001")));
        assert_eq!(r.body, "body");
    }

    // FR-006-AC-3 / TC-013
    #[test]
    fn malformed_yaml_returns_body_fallback() {
        let input = "---\nid: : malformed\n---\nbody";
        let r = extract_frontmatter(input);
        assert!(r.frontmatter.is_none());
        assert_eq!(r.body, input);
    }

    // FR-006-AC-4 / TC-014
    #[test]
    fn unterminated_fence_returns_body_fallback() {
        let input = "---\nid: FR-001\nno closing fence\nbody";
        let r = extract_frontmatter(input);
        assert!(r.frontmatter.is_none());
        assert_eq!(r.body, input);
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
