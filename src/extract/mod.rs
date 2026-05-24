//! DSL extraction surface (FR-011 + FR-016).
//!
//! `extract(doc, dsl) -> ExtractionResult` evaluates a parsed body-
//! extraction DSL against a `QuireDocument`:
//!
//! - Single-yield (`match:`): 0 or 1 records. (Task 015)
//! - Multi-yield (`iterate_over:` + `per_match:`): 1 record per
//!   iteration unit. (Task 016)
//! - Fallback Locator chains: first non-empty primitive wins,
//!   `FallbackLocatorUsed` diagnostic on non-canonical hits. (Task 018)
//! - `emit_edges:` declarative edges, one entry per record per
//!   emission. (Task 016)
//!
//! The hand-off into `crate::edges::harvest_edges` then merges these
//! emit_edges outputs with frontmatter sugar fields (Task 017).

pub mod dsl;
pub mod locator;

use indexmap::IndexMap;
use serde_json::{Map, Value};

use crate::ast::{QuireDocument, QuireSection};
use crate::diagnostic::Diagnostic;
use crate::error::QuireError;
use crate::extract::dsl::{EdgeEmission, EdgeTarget, ExtractionDsl, IterateKind, IterateOver};
use crate::extract::locator::{eval_locator, Locator};

/// Outcome of an `extract` call.
#[derive(Debug, Clone, PartialEq)]
pub struct ExtractionResult {
    /// Single-yield: 0 or 1 records. Multi-yield: 1 per iteration unit.
    pub records: Vec<Map<String, Value>>,
    /// Edges emitted from `emit_edges` entries. `Task 017` merges
    /// these with frontmatter sugar fields + structured relationships.
    pub edges: Vec<HarvestedEdge>,
    /// Advisory notes (e.g. iterate-root missing, fallback used).
    pub diagnostics: Vec<Diagnostic>,
}

/// One edge produced by an `emit_edges` entry or harvest pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarvestedEdge {
    pub r#type: String,
    pub target: String,
    pub metadata: IndexMap<String, Value>,
}

/// Evaluate `dsl` against `doc`.
pub fn extract(doc: &QuireDocument, dsl: &ExtractionDsl) -> Result<ExtractionResult, QuireError> {
    #[cfg(feature = "tracing")]
    let _span = tracing::debug_span!("quire_rs::extract").entered();
    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    if let Some(match_map) = &dsl.yield_pattern.r#match {
        let record = eval_match(doc, match_map, &mut diagnostics)?;
        let mut edges: Vec<HarvestedEdge> = Vec::new();
        if let Some(rec) = &record {
            collect_edges(doc, rec, &dsl.emit_edges, &mut edges, &mut diagnostics);
        }
        let records = record.map(|r| vec![r]).unwrap_or_default();
        return Ok(ExtractionResult {
            records,
            edges,
            diagnostics,
        });
    }

    if let (Some(iter), Some(per)) = (
        dsl.yield_pattern.iterate_over.as_ref(),
        dsl.yield_pattern.per_match.as_ref(),
    ) {
        return Ok(eval_multi(doc, iter, per, &dsl.emit_edges, diagnostics));
    }

    Err(QuireError::DslValidationError {
        archetype: "<unknown>".to_string(),
        reason: "yield_pattern has neither match nor iterate_over".to_string(),
    })
}

fn eval_match(
    doc: &QuireDocument,
    match_map: &IndexMap<String, Locator>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<Option<Map<String, Value>>, QuireError> {
    let mut record: Map<String, Value> = Map::new();
    for (key, loc) in match_map {
        let (values, fallback_pos) = eval_locator(doc, loc);
        if values.is_empty() {
            if loc.required() {
                return Err(QuireError::MissingField {
                    key: key.clone(),
                    locator: loc.canonical().describe(),
                });
            }
            continue;
        }
        if fallback_pos > 0 {
            diagnostics.push(Diagnostic::FallbackLocatorUsed {
                key: key.clone(),
                position: fallback_pos,
                locator: loc.canonical().describe(),
            });
        }
        record.insert(key.clone(), values.into_iter().next().unwrap());
    }
    if record.is_empty() {
        Ok(None)
    } else {
        Ok(Some(record))
    }
}

fn eval_multi(
    doc: &QuireDocument,
    iter: &IterateOver,
    per: &IndexMap<String, Locator>,
    emit: &[EdgeEmission],
    mut diagnostics: Vec<Diagnostic>,
) -> ExtractionResult {
    let root = find_section_by_path(&doc.sections, &iter.section_path);
    let root = match root {
        Some(s) => s,
        None => {
            diagnostics.push(Diagnostic::IterateRootMissing {
                path: iter.section_path.clone(),
            });
            return ExtractionResult {
                records: Vec::new(),
                edges: Vec::new(),
                diagnostics,
            };
        }
    };

    // Each unit carries:
    //   (a) a pre-populated `Map` of fields derived from the unit
    //       itself (heading text, list raw/title/desc, row header
    //       → cell mapping);
    //   (b) an optional "local scope" `QuireDocument` view — for
    //       `IterateKind::Heading` it's a synthetic doc whose
    //       `sections` is just the unit subtree, so a `per_match`
    //       locator that says `section_body, after_heading: X`
    //       resolves to a child of the iteration unit, NOT to a
    //       same-named section elsewhere in the document
    //       (FR-011 "evaluated against each iteration unit's local
    //       scope"). List/table units don't have a natural subtree
    //       so the local scope falls back to the iteration root's
    //       section subtree.
    let units: Vec<UnitContext> = match iter.kind {
        IterateKind::Heading => iterate_heading_units(doc, root, iter.depth.unwrap_or(1)),
        IterateKind::ListItem => iterate_list_units(doc, root),
        IterateKind::TableRow => iterate_table_row_units(doc, root),
    };

    let mut records: Vec<Map<String, Value>> = Vec::new();
    let mut edges: Vec<HarvestedEdge> = Vec::new();
    for unit in &units {
        let mut record: Map<String, Value> = unit.fields.clone();
        let mut record_ok = true;
        for (key, loc) in per {
            let (values, fallback_pos) = eval_locator(&unit.scope, loc);
            if values.is_empty() {
                if loc.required() {
                    record_ok = false;
                    break;
                }
                continue;
            }
            if fallback_pos > 0 {
                diagnostics.push(Diagnostic::FallbackLocatorUsed {
                    key: key.clone(),
                    position: fallback_pos,
                    locator: loc.canonical().describe(),
                });
            }
            record.insert(key.clone(), values.into_iter().next().unwrap());
        }
        if record_ok && !record.is_empty() {
            // emit_edges evaluates against the unit's scope too, so
            // `target: { from: section_body, after_heading: ... }`
            // resolves to the unit's own section.
            collect_edges(&unit.scope, &record, emit, &mut edges, &mut diagnostics);
            records.push(record);
        }
    }

    ExtractionResult {
        records,
        edges,
        diagnostics,
    }
}

/// One iteration unit + the local-scope `QuireDocument` view that
/// `per_match` / `emit_edges` Locators evaluate against (FR-011
/// "iteration unit's local scope").
struct UnitContext {
    fields: Map<String, Value>,
    scope: QuireDocument,
}

/// Build a `QuireDocument` view scoped to `section` (and its
/// children). Inherits `frontmatter` and `raw` from `parent_doc` so
/// `from: frontmatter_field` still works inside a per_match block.
fn scope_to_section(parent_doc: &QuireDocument, section: &QuireSection) -> QuireDocument {
    QuireDocument {
        preamble: None,
        sections: vec![section.clone()],
        raw: parent_doc.raw.clone(),
        frontmatter: parent_doc.frontmatter.clone(),
    }
}

fn iterate_heading_units(
    parent_doc: &QuireDocument,
    root: &QuireSection,
    depth: u8,
) -> Vec<UnitContext> {
    let mut out: Vec<UnitContext> = Vec::new();
    walk_children_at_depth(parent_doc, root, depth, 1, &mut out);
    out
}

fn walk_children_at_depth(
    parent_doc: &QuireDocument,
    parent: &QuireSection,
    target_depth: u8,
    current_depth: u8,
    out: &mut Vec<UnitContext>,
) {
    for child in &parent.children {
        if current_depth == target_depth {
            let mut m = Map::new();
            m.insert("heading".to_string(), Value::String(child.heading.clone()));
            m.insert(
                "content".to_string(),
                Value::String(child.content.trim().to_string()),
            );
            out.push(UnitContext {
                fields: m,
                scope: scope_to_section(parent_doc, child),
            });
        } else {
            walk_children_at_depth(parent_doc, child, target_depth, current_depth + 1, out);
        }
    }
}

fn iterate_list_units(parent_doc: &QuireDocument, root: &QuireSection) -> Vec<UnitContext> {
    let scope = scope_to_section(parent_doc, root);
    crate::query::parse_bullet_list(&root.content, None)
        .into_iter()
        .map(|item| {
            let mut m = Map::new();
            m.insert("raw".to_string(), Value::String(item.raw));
            m.insert("title".to_string(), Value::String(item.title));
            m.insert("description".to_string(), Value::String(item.description));
            UnitContext {
                fields: m,
                scope: scope.clone(),
            }
        })
        .collect()
}

fn iterate_table_row_units(parent_doc: &QuireDocument, root: &QuireSection) -> Vec<UnitContext> {
    let scope = scope_to_section(parent_doc, root);
    match crate::query::parse_table(&root.content) {
        Some(t) => t
            .rows
            .into_iter()
            .map(|row| {
                let mut m = Map::new();
                for (i, cell) in row.into_iter().enumerate() {
                    let key = t.headers.get(i).cloned().unwrap_or_else(|| i.to_string());
                    m.insert(key, Value::String(cell));
                }
                UnitContext {
                    fields: m,
                    scope: scope.clone(),
                }
            })
            .collect(),
        None => Vec::new(),
    }
}

fn find_section_by_path<'a>(
    sections: &'a [QuireSection],
    path: &[String],
) -> Option<&'a QuireSection> {
    let mut current: Option<&'a QuireSection> = None;
    for (i, name) in path.iter().enumerate() {
        current = if i == 0 {
            sections
                .iter()
                .find(|s| s.heading.eq_ignore_ascii_case(name))
        } else {
            current.and_then(|s| {
                s.children
                    .iter()
                    .find(|c| c.heading.eq_ignore_ascii_case(name))
            })
        };
        current?;
    }
    current
}

fn collect_edges(
    doc: &QuireDocument,
    _record: &Map<String, Value>,
    emit: &[EdgeEmission],
    out: &mut Vec<HarvestedEdge>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for entry in emit {
        let metadata = eval_metadata(doc, &entry.metadata, diagnostics);
        match &entry.target {
            EdgeTarget::Static(s) => out.push(HarvestedEdge {
                r#type: entry.r#type.clone(),
                target: s.clone(),
                metadata,
            }),
            EdgeTarget::Located(loc) => {
                let (values, _) = eval_locator(doc, loc);
                for v in values {
                    if let Some(target) = value_to_string(&v) {
                        out.push(HarvestedEdge {
                            r#type: entry.r#type.clone(),
                            target,
                            metadata: metadata.clone(),
                        });
                    } else if let Value::Array(arr) = v {
                        for item in arr {
                            if let Some(t) = value_to_string(&item) {
                                out.push(HarvestedEdge {
                                    r#type: entry.r#type.clone(),
                                    target: t,
                                    metadata: metadata.clone(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}

fn eval_metadata(
    doc: &QuireDocument,
    md: &IndexMap<String, Locator>,
    diagnostics: &mut Vec<Diagnostic>,
) -> IndexMap<String, Value> {
    let mut out: IndexMap<String, Value> = IndexMap::new();
    for (key, loc) in md {
        let (values, fb) = eval_locator(doc, loc);
        if let Some(v) = values.into_iter().next() {
            if fb > 0 {
                diagnostics.push(Diagnostic::FallbackLocatorUsed {
                    key: key.clone(),
                    position: fb,
                    locator: loc.canonical().describe(),
                });
            }
            out.insert(key.clone(), v);
        }
    }
    out
}

fn value_to_string(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        Value::Null => None,
        Value::Array(_) | Value::Object(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_document;

    fn doc() -> QuireDocument {
        parse_document(
            "---\nid: FR-001\ntype: doc\ndepends_on:\n- FR-002\n- FR-003\n---\n\
             ## Purpose\nthe purpose\n\
             ## API\n| Method | Path |\n| - | - |\n| GET | /a |\n",
        )
    }

    fn dsl_from(yaml: &str) -> ExtractionDsl {
        serde_yaml::from_str(yaml).unwrap()
    }

    // ── Task 015 carry-over (still passes) ──────────────────────────

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
        assert_eq!(r.records[0]["id"], serde_json::json!("FR-001"));
    }

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

    // ── Task 016: multi-yield + emit_edges ──────────────────────────

    // FR-011 "iteration unit's local scope": per_match locators
    // must resolve against the unit's section subtree, not the whole
    // document. Build a doc where the same heading name appears
    // outside the iteration root — the per_match value must come
    // from INSIDE the unit, not the same-named sibling.
    #[test]
    fn per_match_locators_evaluate_against_unit_local_scope() {
        let md = "\
## Algorithms\nintro\n\
### A\n\
#### Detail\nfrom-A\n\
### B\n\
#### Detail\nfrom-B\n\
## Detail\nfrom-outside\n";
        let d = parse_document(md);
        let dsl: ExtractionDsl = serde_yaml::from_str(
            r#"
yield_pattern:
  iterate_over:
    section_path: [Algorithms]
    kind: heading
    depth: 1
  per_match:
    detail:
      from: section_body
      after_heading: Detail
"#,
        )
        .unwrap();
        let r = extract(&d, &dsl).expect("ok");
        assert_eq!(r.records.len(), 2);
        // Per unit, `Detail` resolves to the unit's own child, NOT
        // the top-level "## Detail" sibling.
        assert_eq!(
            r.records[0]["detail"],
            serde_json::json!("from-A"),
            "unit A must read its own Detail subsection, not the outside one"
        );
        assert_eq!(
            r.records[1]["detail"],
            serde_json::json!("from-B"),
            "unit B must read its own Detail subsection, not the outside one"
        );
    }

    // FR-011-AC-2
    #[test]
    fn multi_yield_iterate_headings_emits_one_record_per_unit() {
        let d =
            parse_document("## Algorithms\nintro\n### A\nbody a\n### B\nbody b\n### C\nbody c\n");
        let dsl = dsl_from(
            r#"
yield_pattern:
  iterate_over:
    section_path: [Algorithms]
    kind: heading
    depth: 1
  per_match: {}
"#,
        );
        let r = extract(&d, &dsl).expect("ok");
        assert_eq!(r.records.len(), 3);
        assert_eq!(r.records[0]["heading"], serde_json::json!("A"));
    }

    // FR-011-AC-3
    #[test]
    fn emit_edges_one_per_frontmatter_list_item() {
        let d = doc();
        let dsl = dsl_from(
            r#"
yield_pattern:
  match:
    id:
      from: frontmatter_field
      path: [id]
      required: true
emit_edges:
- type: depends_on
  target:
    from: frontmatter_field
    path: [depends_on]
"#,
        );
        let r = extract(&d, &dsl).expect("ok");
        assert_eq!(r.edges.len(), 2);
        assert_eq!(r.edges[0].target, "FR-002");
        assert_eq!(r.edges[1].target, "FR-003");
    }

    // FR-011-AC-8
    #[test]
    fn iterate_root_missing_emits_diagnostic_and_zero_records() {
        let d = doc();
        let dsl = dsl_from(
            r#"
yield_pattern:
  iterate_over:
    section_path: [Nonexistent]
    kind: heading
  per_match: {}
"#,
        );
        let r = extract(&d, &dsl).expect("ok");
        assert!(r.records.is_empty());
        assert!(r
            .diagnostics
            .iter()
            .any(|d| matches!(d, Diagnostic::IterateRootMissing { .. })));
    }

    // ── Task 018: fallback Locator chain ─────────────────────────────

    #[test]
    fn fallback_canonical_position_does_not_emit_diagnostic() {
        let d = doc();
        let dsl = dsl_from(
            r#"
yield_pattern:
  match:
    id:
    - from: frontmatter_field
      path: [id]
    - from: heading
      level: 2
"#,
        );
        let r = extract(&d, &dsl).expect("ok");
        assert!(!r
            .diagnostics
            .iter()
            .any(|d| matches!(d, Diagnostic::FallbackLocatorUsed { .. })));
    }

    #[test]
    fn fallback_secondary_position_emits_diagnostic() {
        let d = doc();
        let dsl = dsl_from(
            r#"
yield_pattern:
  match:
    name:
    - from: frontmatter_field
      path: [nonexistent]
    - from: heading
      level: 2
"#,
        );
        let r = extract(&d, &dsl).expect("ok");
        assert!(r
            .diagnostics
            .iter()
            .any(|d| matches!(d, Diagnostic::FallbackLocatorUsed { position: 1, .. })));
    }

    #[test]
    fn locator_primitive_only_still_parses() {
        // Backward-compat: a bare object (not a list) still works as
        // a Locator::Primitive.
        let _: Locator = serde_yaml::from_str(
            r#"
from: heading
level: 2
"#,
        )
        .unwrap();
    }
}
