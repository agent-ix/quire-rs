//! DSL extraction surface (FR-011).
//!
//! `extract(doc, dsl) -> ExtractionResult` evaluates a parsed body-
//! extraction DSL against a `QuireDocument` and emits zero, one, or
//! many records. Task 015 covers the single-yield (`match:`) shape +
//! the six Locator primitives. Task 016 adds multi-yield
//! (`iterate_over` / `per_match`) and `emit_edges`.

pub mod dsl;
pub mod locator;

use indexmap::IndexMap;
use serde_json::{Map, Value};

use crate::ast::QuireDocument;
use crate::diagnostic::Diagnostic;
use crate::error::QuireError;
use crate::extract::dsl::ExtractionDsl;
use crate::extract::locator::{eval, LocatorPrimitive};

/// Outcome of an `extract` call.
#[derive(Debug, Clone, PartialEq)]
pub struct ExtractionResult {
    /// Single-yield: 0 or 1 records. Multi-yield: 1 per iteration unit.
    pub records: Vec<Map<String, Value>>,
    /// Edges harvested from `emit_edges` entries. Empty until Task 016
    /// implements multi-yield + emission semantics.
    pub edges: Vec<HarvestedEdge>,
    /// Advisory notes (e.g. iterate-root missing).
    pub diagnostics: Vec<Diagnostic>,
}

/// One edge produced by an `emit_edges` entry. Task 017 promotes
/// these to ix:// resolved form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarvestedEdge {
    pub r#type: String,
    pub target: String,
    pub metadata: IndexMap<String, Value>,
}

/// Evaluate `dsl` against `doc`.
pub fn extract(doc: &QuireDocument, dsl: &ExtractionDsl) -> Result<ExtractionResult, QuireError> {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    // Single-yield branch — Task 015 owns this path.
    if let Some(match_map) = &dsl.yield_pattern.r#match {
        let record = eval_match(doc, match_map)?;
        let records = match record {
            Some(r) => vec![r],
            None => Vec::new(),
        };
        return Ok(ExtractionResult {
            records,
            edges: Vec::new(),
            diagnostics,
        });
    }

    // Multi-yield (iterate_over + per_match) lands in Task 016. For
    // now, return an empty record set with a placeholder diagnostic
    // so callers know they need the multi-yield evaluator.
    if let Some(iter) = &dsl.yield_pattern.iterate_over {
        diagnostics.push(Diagnostic::SymlinkLoop {
            // Sentinel — Task 016 introduces a dedicated diagnostic
            // variant. Until then we surface SOMETHING so consumers
            // notice. The DSL was structurally valid.
            path: std::path::PathBuf::from(format!(
                "<multi-yield-not-yet-implemented: section_path={:?}>",
                iter.section_path
            )),
        });
        return Ok(ExtractionResult {
            records: Vec::new(),
            edges: Vec::new(),
            diagnostics,
        });
    }

    // Should be unreachable if dsl::validate_dsl ran first.
    Err(QuireError::DslValidationError {
        archetype: "<unknown>".to_string(),
        reason: "yield_pattern has neither match nor iterate_over".to_string(),
    })
}

/// Evaluate a `match: { key: Locator }` block. Returns `Some(record)`
/// when every required locator resolves (optional locators may be
/// missing and are simply omitted from the record). Returns `None`
/// when no value was produced at all (empty record).
fn eval_match(
    doc: &QuireDocument,
    match_map: &IndexMap<String, LocatorPrimitive>,
) -> Result<Option<Map<String, Value>>, QuireError> {
    let mut record: Map<String, Value> = Map::new();
    for (key, loc) in match_map {
        let values = eval(doc, loc);
        if values.is_empty() {
            if loc.required() {
                return Err(QuireError::MissingField {
                    key: key.clone(),
                    locator: loc.describe(),
                });
            }
            continue;
        }
        // Single-yield: take the first value. Locators that naturally
        // produce many (table_row, list_item, heading without path)
        // get collapsed to their head; multi-yield consumers (Task 016)
        // will iterate the full list.
        record.insert(key.clone(), values.into_iter().next().unwrap());
    }
    if record.is_empty() {
        Ok(None)
    } else {
        Ok(Some(record))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_document;

    fn doc() -> QuireDocument {
        parse_document(
            "---\nid: FR-001\ntype: doc\n---\n## Purpose\nthe purpose\n## API\n| Method | Path |\n| - | - |\n| GET | /a |\n",
        )
    }

    fn dsl_from(yaml: &str) -> ExtractionDsl {
        serde_yaml::from_str(yaml).unwrap()
    }

    #[test]
    fn single_yield_collects_all_resolved_keys() {
        let d = doc();
        let dsl = dsl_from(
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
        let r = extract(&d, &dsl).expect("ok");
        assert_eq!(r.records.len(), 1);
        let rec = &r.records[0];
        assert_eq!(rec["id"], serde_json::json!("FR-001"));
        assert_eq!(rec["purpose"], serde_json::json!("the purpose"));
    }

    // FR-011-AC-4: required-key miss → MissingField.
    #[test]
    fn required_miss_returns_missing_field() {
        let d = doc();
        let dsl = dsl_from(
            r#"
yield_pattern:
  match:
    nope:
      from: frontmatter_field
      path: [nonexistent]
      required: true
"#,
        );
        let err = extract(&d, &dsl).expect_err("missing");
        assert!(matches!(err, QuireError::MissingField { .. }));
    }

    #[test]
    fn optional_miss_is_silently_omitted() {
        let d = doc();
        let dsl = dsl_from(
            r#"
yield_pattern:
  match:
    id:
      from: frontmatter_field
      path: [id]
      required: true
    extra:
      from: frontmatter_field
      path: [doesnt_exist]
      required: false
"#,
        );
        let r = extract(&d, &dsl).expect("ok");
        assert_eq!(r.records.len(), 1);
        let rec = &r.records[0];
        assert!(rec.contains_key("id"));
        assert!(!rec.contains_key("extra"));
    }

    #[test]
    fn table_row_single_yield_picks_first_row() {
        let d = doc();
        let dsl = dsl_from(
            r#"
yield_pattern:
  match:
    first_endpoint:
      from: table_row
      under_section: API
"#,
        );
        let r = extract(&d, &dsl).expect("ok");
        assert_eq!(
            r.records[0]["first_endpoint"]["Method"],
            serde_json::json!("GET")
        );
        assert_eq!(
            r.records[0]["first_endpoint"]["Path"],
            serde_json::json!("/a")
        );
    }
}
