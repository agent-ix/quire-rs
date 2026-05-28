//! Body-extraction DSL shape + structural validation (FR-011).
//!
//! Authors ship a `body_extraction:` block in their object-type
//! manifest. Each block has a `yield_pattern` that's either
//! `match: {key: Locator}` (single-yield, zero-or-one record) or
//! `iterate_over: { ... }` + `per_match: { ... }` (multi-yield, one
//! record per iteration unit) — but NEVER both (FR-011-AC-6).
//!
//! Load-time validation rejects DSLs that set both, set neither, or
//! contain unknown keys (FR-011-AC-7).

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::error::QuireError;
use crate::extract::locator::Locator;

/// One body-extraction DSL — the parsed form of `body_extraction:` in
/// an object-type manifest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtractionDsl {
    pub yield_pattern: YieldPattern,
    #[serde(default)]
    pub emit_edges: Option<Vec<EmitEdge>>,
}

/// `yield_pattern:` — single (`match`) XOR multi (`iterate_over` +
/// `per_match`). Authors set one or the other.
///
/// We deliberately do NOT use `#[serde(untagged)]` here; the
/// validator below picks one branch by which key is present, surfacing
/// the load error message itself.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct YieldPattern {
    #[serde(default, rename = "match")]
    pub r#match: Option<IndexMap<String, Locator>>,
    #[serde(default)]
    pub iterate_over: Option<IterateOver>,
    #[serde(default)]
    pub per_match: Option<IndexMap<String, Locator>>,
}

/// `iterate_over:` — describes how the multi-yield evaluator picks
/// iteration units (one record per unit).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IterateOver {
    /// Heading path to the iteration root (e.g. `["Algorithms"]`).
    pub section_path: Vec<String>,
    /// What unit the evaluator iterates: child headings, list items,
    /// or table rows.
    pub kind: IterateKind,
    /// Heading depth (relative under `section_path`). Defaults to 1.
    #[serde(default)]
    pub depth: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IterateKind {
    Heading,
    ListItem,
    TableRow,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmitEdge {
    #[serde(rename = "type")]
    pub edge_type: String,
    pub target: EdgeTarget,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EdgeTarget {
    Static(String),
    Locator(Locator),
}

/// Structural validation: `match` XOR `iterate_over` + `per_match`
/// (FR-011-AC-6). The `archetype` argument is the owning archetype's
/// name — included in the error so DSL violations point at the right
/// declaration.
pub fn validate_dsl(archetype: &str, dsl: &ExtractionDsl) -> Result<(), QuireError> {
    let has_match = dsl.yield_pattern.r#match.is_some();
    let has_iter = dsl.yield_pattern.iterate_over.is_some();
    let has_per = dsl.yield_pattern.per_match.is_some();
    if has_match && (has_iter || has_per) {
        return Err(QuireError::DslValidationError {
            archetype: archetype.to_string(),
            reason: "match and iterate_over are mutually exclusive".to_string(),
        });
    }
    if !has_match && !has_iter {
        return Err(QuireError::DslValidationError {
            archetype: archetype.to_string(),
            reason: "yield_pattern must declare either match or iterate_over".to_string(),
        });
    }
    if has_iter && !has_per {
        return Err(QuireError::DslValidationError {
            archetype: archetype.to_string(),
            reason: "iterate_over requires per_match".to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(yaml: &str) -> ExtractionDsl {
        serde_yaml::from_str(yaml).expect("parse")
    }

    #[test]
    fn match_pattern_round_trips_through_yaml() {
        let dsl = parse(
            r#"
yield_pattern:
  match:
    id:
      from: frontmatter_field
      path: [id]
      required: true
    purpose:
      from: section_body
      after_heading: Purpose
"#,
        );
        validate_dsl("ot", &dsl).expect("ok");
        let m = dsl.yield_pattern.r#match.as_ref().unwrap();
        assert!(m.contains_key("id"));
        assert!(m.contains_key("purpose"));
    }

    #[test]
    fn iterate_over_pattern_validates() {
        let dsl = parse(
            r#"
yield_pattern:
  iterate_over:
    section_path: [Algorithms]
    kind: heading
    depth: 1
  per_match:
    name:
      from: heading
"#,
        );
        validate_dsl("ot", &dsl).expect("ok");
    }

    // FR-011-AC-6
    #[test]
    fn both_match_and_iterate_over_is_dsl_error() {
        let dsl = parse(
            r#"
yield_pattern:
  match:
    a:
      from: heading
  iterate_over:
    section_path: [x]
    kind: heading
  per_match:
    b:
      from: heading
"#,
        );
        let err = validate_dsl("ot", &dsl).expect_err("collision");
        assert!(matches!(err, QuireError::DslValidationError { .. }));
    }

    #[test]
    fn neither_match_nor_iterate_over_is_dsl_error() {
        let dsl: ExtractionDsl = serde_yaml::from_str("yield_pattern: {}\n").unwrap();
        let err = validate_dsl("ot", &dsl).expect_err("empty");
        assert!(matches!(err, QuireError::DslValidationError { .. }));
    }

    // FR-011-AC-7
    #[test]
    fn unknown_key_in_yield_pattern_fails_yaml_parse() {
        let r: Result<ExtractionDsl, _> = serde_yaml::from_str(
            r#"
yield_pattern:
  unknown_key: x
"#,
        );
        assert!(r.is_err(), "deny_unknown_fields should reject unknown keys");
    }
}
