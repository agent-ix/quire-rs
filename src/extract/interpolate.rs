//! `{field}` interpolation for assert patterns (FR-034).
//!
//! Assert pattern fields (e.g. `id_pattern`) MAY contain `{field}`
//! tokens that are resolved at **validate time** from the document's own
//! frontmatter. `{id}` resolves to `frontmatter.id`, `{title}` to
//! `frontmatter.title`, etc. Interpolation runs **before** the pattern is
//! compiled as a regex, and substituted values are **regex-escaped** so a
//! value containing regex metacharacters does not alter the pattern.
//!
//! Only single-brace `{ident}` tokens whose name is a valid identifier
//! (`[A-Za-z_][A-Za-z0-9_]*`) are treated as fields; regex repetition
//! quantifiers like `{2,4}` start with a digit and are left untouched.

use std::sync::OnceLock;

use regex::Regex;
use serde_json::{Map, Value};

/// A `{field}` token that could not be resolved (FR-034-AC-2). Carries
/// the offending field name so the caller can build a line-numbered
/// diagnostic naming the archetype + locator + missing field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnresolvedField {
    pub field: String,
}

fn re_field_token() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"\{([A-Za-z_][A-Za-z0-9_]*)\}").expect("field-token regex"))
}

/// Resolve `{field}` tokens in `pattern` from `frontmatter`.
///
/// Each token's frontmatter value is rendered as a scalar string and
/// **regex-escaped** before substitution (FR-034-AC-3). A token whose
/// field is absent (or whose frontmatter is `None`) returns the first
/// such field as an [`UnresolvedField`] error (FR-034-AC-2); the pattern
/// is never silently passed through.
///
/// A pattern with no token is returned unchanged (FR-034-AC-4).
pub fn interpolate(
    pattern: &str,
    frontmatter: Option<&Map<String, Value>>,
) -> Result<String, UnresolvedField> {
    let re = re_field_token();
    if !re.is_match(pattern) {
        return Ok(pattern.to_string());
    }

    let mut out = String::with_capacity(pattern.len());
    let mut last = 0usize;
    for caps in re.captures_iter(pattern) {
        let whole = caps.get(0).expect("group 0");
        let field = caps.get(1).expect("group 1").as_str();
        out.push_str(&pattern[last..whole.start()]);
        let value = frontmatter
            .and_then(|fm| fm.get(field))
            .and_then(scalar_to_string)
            .ok_or_else(|| UnresolvedField {
                field: field.to_string(),
            })?;
        out.push_str(&regex::escape(&value));
        last = whole.end();
    }
    out.push_str(&pattern[last..]);
    Ok(out)
}

/// Render a frontmatter scalar as a string. Non-scalar values (arrays,
/// objects, null) are **not** interpolatable (FR-034: scalars only) and
/// resolve to `None` so the caller raises an unresolved-field
/// diagnostic.
fn scalar_to_string(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        Value::Null | Value::Array(_) | Value::Object(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fm(pairs: &[(&str, Value)]) -> Map<String, Value> {
        let mut m = Map::new();
        for (k, v) in pairs {
            m.insert((*k).to_string(), v.clone());
        }
        m
    }

    // FR-034-AC-1: `{id}` resolves to frontmatter.id.
    #[test]
    fn resolves_id_token() {
        let m = fm(&[("id", json!("FR-900"))]);
        let out = interpolate(r"^{id}-AC-\d+$", Some(&m)).unwrap();
        assert_eq!(out, r"^FR\-900-AC-\d+$");
    }

    // FR-034-AC-2: missing field → UnresolvedField, not silent pass.
    #[test]
    fn missing_field_is_error() {
        let m = fm(&[("id", json!("FR-900"))]);
        let err = interpolate(r"^{title}-x$", Some(&m)).unwrap_err();
        assert_eq!(err.field, "title");
    }

    #[test]
    fn missing_frontmatter_is_error() {
        let err = interpolate(r"^{id}$", None).unwrap_err();
        assert_eq!(err.field, "id");
    }

    // FR-034-AC-3: regex metacharacters in the value are escaped.
    #[test]
    fn value_with_metacharacters_is_escaped() {
        let m = fm(&[("id", json!("A.B+"))]);
        let out = interpolate(r"^{id}$", Some(&m)).unwrap();
        // The literal `A.B+` must be matched literally.
        let re = Regex::new(&out).unwrap();
        assert!(re.is_match("A.B+"));
        assert!(!re.is_match("AxBB"));
    }

    // FR-034-AC-4: no token → unchanged (quantifiers untouched).
    #[test]
    fn no_token_returns_unchanged() {
        let m = fm(&[("id", json!("FR-900"))]);
        let pattern = r"^[A-Z]{2,4}-\d+-AC-\d+$";
        assert_eq!(interpolate(pattern, Some(&m)).unwrap(), pattern);
        // Also fine with no frontmatter at all.
        assert_eq!(interpolate(pattern, None).unwrap(), pattern);
    }
}
