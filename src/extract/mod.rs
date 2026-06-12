//! DSL extraction surface (FR-011 + FR-016).
//!
//! `extract(doc, dsl) -> ExtractionResult` evaluates a parsed body-
//! extraction DSL against a `QuireDocument`:
//!
//! - Single-yield (`match:`): 0 or 1 records.
//! - Multi-yield (`iterate_over:` + `per_match:`): 1 record per
//!   iteration unit.
//! - Fallback Locator chains: first non-empty primitive wins,
//!   `FallbackLocatorUsed` diagnostic on non-canonical hits.

pub mod assert_eval;
pub mod dsl;
pub mod interpolate;
pub mod locator;

use indexmap::IndexMap;
use serde_json::{Map, Value};

use crate::ast::{QuireDocument, QuireSection};
use crate::diagnostic::Diagnostic;
use crate::error::QuireError;
use crate::extract::dsl::{EdgeTarget, EmitEdge, ExtractionDsl, IterateKind, IterateOver};
use crate::extract::locator::{eval_locator, is_whole_value_mustache, Locator};

/// FR-011-AC-17: drop resolved values that are a whole-value `{{…}}`
/// placeholder marker. Used on the extract path so a placeholder-only
/// locator contributes nothing (a required miss if nothing remains).
fn drop_whole_value_placeholders(values: Vec<Value>) -> Vec<Value> {
    values
        .into_iter()
        .filter(|v| match v {
            Value::String(s) => !is_whole_value_mustache(s),
            _ => true,
        })
        .collect()
}

/// Outcome of an `extract` call.
#[derive(Debug, Clone, PartialEq)]
pub struct ExtractionResult {
    /// Single-yield: 0 or 1 records. Multi-yield: 1 per iteration unit.
    pub records: Vec<Map<String, Value>>,
    /// Edges emitted by the extraction DSL, indexed to `records`.
    pub edges: Vec<ExtractedEdge>,
    /// Advisory notes (e.g. iterate-root missing, fallback used).
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExtractedEdge {
    pub record_index: usize,
    pub edge_type: String,
    pub target: String,
}

/// Evaluate `dsl` against `doc`.
pub fn extract(doc: &QuireDocument, dsl: &ExtractionDsl) -> Result<ExtractionResult, QuireError> {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    if let Some(match_map) = &dsl.yield_pattern.r#match {
        let record = eval_match(doc, match_map, &mut diagnostics)?;
        let records = record.map(|r| vec![r]).unwrap_or_default();
        let edges = emit_edges_for_records(doc, &records, dsl.emit_edges.as_deref());
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
        return eval_multi(doc, iter, per, dsl.emit_edges.as_deref(), diagnostics);
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
        // FR-011-AC-17: drop whole-value `{{…}}` placeholders on the
        // extract path (a required miss if nothing substantive remains).
        let values = drop_whole_value_placeholders(values);
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
        record.insert(key.clone(), collapse_values(loc, fallback_pos, values));
    }
    if record.is_empty() {
        Ok(None)
    } else {
        Ok(Some(record))
    }
}

/// Collapse a locator's resolved values into the record value:
/// first-wins for the default single-value contract, or the full
/// ordered list as a JSON array when the used primitive declares
/// `multiple: true` (FR-011-AC-21). `values` is non-empty (callers
/// handle the miss path before collapsing).
fn collapse_values(loc: &Locator, position_used: usize, values: Vec<Value>) -> Value {
    if loc.multiple_at(position_used) {
        Value::Array(values)
    } else {
        values
            .into_iter()
            .next()
            .expect("collapse_values requires non-empty values")
    }
}

fn eval_multi(
    doc: &QuireDocument,
    iter: &IterateOver,
    per: &IndexMap<String, Locator>,
    emit_edges: Option<&[EmitEdge]>,
    mut diagnostics: Vec<Diagnostic>,
) -> Result<ExtractionResult, QuireError> {
    if find_section_by_path(&doc.sections, &iter.section_path).is_none() {
        diagnostics.push(Diagnostic::IterateRootMissing {
            path: iter.section_path.clone(),
        });
        return Ok(ExtractionResult {
            records: Vec::new(),
            edges: Vec::new(),
            diagnostics,
        });
    }

    // Each unit carries pre-populated fields plus a "local scope"
    // QuireDocument view used by per_match Locators. For Heading
    // iteration the scope is just the unit subtree so e.g.
    // `section_body, after_heading: X` finds X *inside* the unit,
    // not a same-named section elsewhere (FR-011 "evaluated against
    // each iteration unit's local scope").
    let units = iteration_units(doc, iter);

    let mut records: Vec<Map<String, Value>> = Vec::new();
    let mut edges: Vec<ExtractedEdge> = Vec::new();
    for unit in &units {
        let mut record: Map<String, Value> = unit.fields.clone();
        for (key, loc) in per {
            let (values, fallback_pos) = eval_locator(&unit.scope, loc);
            // FR-011-AC-17: drop whole-value `{{…}}` placeholders.
            let values = drop_whole_value_placeholders(values);
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
            record.insert(key.clone(), collapse_values(loc, fallback_pos, values));
        }
        if !record.is_empty() {
            let record_index = records.len();
            edges.extend(emit_edges_for_scope(&unit.scope, record_index, emit_edges));
            records.push(record);
        }
    }

    Ok(ExtractionResult {
        records,
        edges,
        diagnostics,
    })
}

fn emit_edges_for_records(
    doc: &QuireDocument,
    records: &[Map<String, Value>],
    emit_edges: Option<&[EmitEdge]>,
) -> Vec<ExtractedEdge> {
    records
        .iter()
        .enumerate()
        .flat_map(|(idx, _)| emit_edges_for_scope(doc, idx, emit_edges))
        .collect()
}

fn emit_edges_for_scope(
    scope: &QuireDocument,
    record_index: usize,
    emit_edges: Option<&[EmitEdge]>,
) -> Vec<ExtractedEdge> {
    let Some(specs) = emit_edges else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for spec in specs {
        let targets: Vec<String> = match &spec.target {
            EdgeTarget::Static(target) => vec![target.clone()],
            EdgeTarget::Locator(locator) => eval_locator(scope, locator)
                .0
                .into_iter()
                .filter_map(|v| v.as_str().map(ToString::to_string))
                .collect(),
        };
        for target in targets {
            if !target.is_empty() {
                out.push(ExtractedEdge {
                    record_index,
                    edge_type: spec.edge_type.clone(),
                    target,
                });
            }
        }
    }
    out
}

/// One iteration unit + the local-scope `QuireDocument` view that
/// `per_match` Locators evaluate against (FR-011 "iteration unit's
/// local scope").
pub struct UnitContext {
    pub fields: Map<String, Value>,
    pub scope: QuireDocument,
}

/// Resolve the multi-yield iteration units (one local-scope view per
/// unit) for an `iterate_over`. Shared by the extractor and by
/// `validate_document` (FR-032 step 3), so both postures resolve the
/// **same** units against the **same** local scopes. Returns an empty
/// vec when the iteration root is absent (the extract path additionally
/// emits an `IterateRootMissing` diagnostic; validation surfaces the
/// absence via the required-locator check on the units it produces).
pub fn iteration_units(doc: &QuireDocument, iter: &IterateOver) -> Vec<UnitContext> {
    let Some(root) = find_section_by_path(&doc.sections, &iter.section_path) else {
        return Vec::new();
    };
    match iter.kind {
        IterateKind::Heading => iterate_heading_units(doc, root, iter.depth),
        IterateKind::ListItem => iterate_list_units(doc, root),
        IterateKind::TableRow => iterate_table_row_units(doc, root),
    }
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
    depth: Option<u8>,
) -> Vec<UnitContext> {
    let mut out: Vec<UnitContext> = Vec::new();
    match depth {
        Some(depth) => {
            let level = if depth > root.level {
                depth
            } else {
                root.level.saturating_add(depth)
            };
            walk_children_at_level(parent_doc, root, level, &mut out);
        }
        None => {
            for child in &root.children {
                push_heading_unit(parent_doc, child, &mut out);
            }
        }
    }
    out
}

fn walk_children_at_level(
    parent_doc: &QuireDocument,
    parent: &QuireSection,
    target_level: u8,
    out: &mut Vec<UnitContext>,
) {
    for child in &parent.children {
        if child.level == target_level {
            push_heading_unit(parent_doc, child, out);
        }
        walk_children_at_level(parent_doc, child, target_level, out);
    }
}

fn push_heading_unit(
    parent_doc: &QuireDocument,
    section: &QuireSection,
    out: &mut Vec<UnitContext>,
) {
    let mut m = Map::new();
    m.insert(
        "heading".to_string(),
        Value::String(section.heading.clone()),
    );
    m.insert(
        "content".to_string(),
        Value::String(section.content.trim().to_string()),
    );
    out.push(UnitContext {
        fields: m,
        scope: scope_to_section(parent_doc, section),
    });
}

fn iterate_list_units(parent_doc: &QuireDocument, root: &QuireSection) -> Vec<UnitContext> {
    crate::query::parse_bullet_list(&root.content, None)
        .into_iter()
        .map(|item| {
            let mut m = Map::new();
            m.insert("raw".to_string(), Value::String(item.raw));
            m.insert("title".to_string(), Value::String(item.title));
            m.insert("description".to_string(), Value::String(item.description));
            let heading = m
                .get("raw")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let scope = QuireDocument {
                preamble: None,
                sections: vec![QuireSection {
                    id: format!("{}-item", root.id),
                    block_id: None,
                    heading,
                    level: root.level.saturating_add(1),
                    content: String::new(),
                    children: Vec::new(),
                    start_line: root.start_line,
                    end_line: root.end_line,
                }],
                raw: parent_doc.raw.clone(),
                frontmatter: parent_doc.frontmatter.clone(),
            };
            UnitContext { fields: m, scope }
        })
        .collect()
}

fn iterate_table_row_units(parent_doc: &QuireDocument, root: &QuireSection) -> Vec<UnitContext> {
    match crate::query::parse_table(&root.content) {
        Some(t) => t
            .rows
            .into_iter()
            .map(|row| {
                let heading = row.join("\t");
                let mut m = Map::new();
                for (i, cell) in row.into_iter().enumerate() {
                    let key = t.headers.get(i).cloned().unwrap_or_else(|| i.to_string());
                    m.insert(key, Value::String(cell));
                }
                let scope = QuireDocument {
                    preamble: None,
                    sections: vec![QuireSection {
                        id: format!("{}-row", root.id),
                        block_id: None,
                        heading,
                        level: root.level.saturating_add(1),
                        content: String::new(),
                        children: Vec::new(),
                        start_line: root.start_line,
                        end_line: root.end_line,
                    }],
                    raw: parent_doc.raw.clone(),
                    frontmatter: parent_doc.frontmatter.clone(),
                };
                UnitContext { fields: m, scope }
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

    // FR-011-AC-21: `multiple: true` keeps every located value as a
    // JSON array; the default contract still collapses to the first.
    #[test]
    fn multiple_true_keeps_all_values_as_array() {
        let md = "## Workflow\n\
                  ```mermaid\nflowchart LR\n```\n\
                  ```mermaid\nstateDiagram-v2\n```\n";
        let d = parse_document(md);
        let dsl = dsl_from(
            r#"
yield_pattern:
  match:
    diagram:
      from: code_block
      language: mermaid
      under_section: Workflow
      multiple: true
"#,
        );
        let r = extract(&d, &dsl).expect("ok");
        assert_eq!(r.records.len(), 1);
        let diagrams = r.records[0]["diagram"].as_array().expect("array");
        assert_eq!(diagrams.len(), 2);
        assert!(diagrams[0].as_str().unwrap().contains("flowchart"));
        assert!(diagrams[1].as_str().unwrap().contains("stateDiagram"));
    }

    #[test]
    fn without_multiple_first_value_wins_unchanged() {
        let md = "## Workflow\n\
                  ```mermaid\nflowchart LR\n```\n\
                  ```mermaid\nstateDiagram-v2\n```\n";
        let d = parse_document(md);
        let dsl = dsl_from(
            r#"
yield_pattern:
  match:
    diagram:
      from: code_block
      language: mermaid
      under_section: Workflow
"#,
        );
        let r = extract(&d, &dsl).expect("ok");
        let v = r.records[0]["diagram"].as_str().expect("scalar string");
        assert!(v.contains("flowchart"));
    }

    // FR-011-AC-21: in a fallback chain the `multiple` flag is read
    // from the primitive that actually produced the values.
    #[test]
    fn multiple_in_fallback_chain_uses_hit_primitive_flag() {
        let md = "## Steps\n- one\n- two\n";
        let d = parse_document(md);
        let dsl = dsl_from(
            r#"
yield_pattern:
  match:
    steps:
      - from: section_body
        after_heading: Nonexistent
      - from: list_item
        under_section: Steps
        multiple: true
"#,
        );
        let r = extract(&d, &dsl).expect("ok");
        let steps = r.records[0]["steps"].as_array().expect("array");
        assert_eq!(steps.len(), 2);
    }

    // FR-011-AC-21: multiple:true under iterate_over/per_match keeps
    // each unit's full value list.
    #[test]
    fn multiple_true_applies_per_iteration_unit() {
        let md = "\
## Algorithms\nintro\n\
### A\n\
```mermaid\na1\n```\n\
```mermaid\na2\n```\n\
### B\n\
```mermaid\nb1\n```\n";
        let d = parse_document(md);
        let dsl = dsl_from(
            r#"
yield_pattern:
  iterate_over:
    section_path: [Algorithms]
    kind: heading
    depth: 1
  per_match:
    diagrams:
      from: code_block
      language: mermaid
      multiple: true
"#,
        );
        let r = extract(&d, &dsl).expect("ok");
        assert_eq!(r.records.len(), 2);
        assert_eq!(r.records[0]["diagrams"].as_array().unwrap().len(), 2);
        assert_eq!(r.records[1]["diagrams"].as_array().unwrap().len(), 1);
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

    // ── Multi-yield ─────────────────────────────────────────────────

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

    // FR-011-AC-13: `code_block` is section-owned, so under iteration
    // each yielded record gets ITS OWN unit's fenced block — not unit
    // #1's block for every record (the latent multi-yield gap closed by
    // routing the locator through the unit's content slice rather than
    // the document-wide harvest).
    #[test]
    fn code_block_per_match_isolates_per_unit_under_iterate_over() {
        let md = "\
## Steps\nintro\n\
### One\n```mermaid\ngraph TD; ONE-->X\n```\n\
### Two\n```mermaid\ngraph TD; TWO-->Y\n```\n";
        let d = parse_document(md);
        let dsl: ExtractionDsl = serde_yaml::from_str(
            r#"
yield_pattern:
  iterate_over:
    section_path: [Steps]
    kind: heading
    depth: 1
  per_match:
    diagram:
      from: code_block
      language: mermaid
"#,
        )
        .unwrap();
        let r = extract(&d, &dsl).expect("ok");
        assert_eq!(r.records.len(), 2);
        assert!(r.records[0]["diagram"]
            .as_str()
            .unwrap()
            .contains("ONE-->X"));
        assert!(r.records[1]["diagram"]
            .as_str()
            .unwrap()
            .contains("TWO-->Y"));
        // Crucially, unit #2 must NOT get unit #1's block.
        assert!(!r.records[1]["diagram"].as_str().unwrap().contains("ONE"));
    }

    // FR-011-AC-13: a required `code_block` per_match fails for the
    // specific unit that lacks its own block (containment, not a
    // document-wide fallback that would silently borrow a sibling's).
    #[test]
    fn code_block_per_match_required_fails_for_unit_missing_its_block() {
        let md = "\
## Steps\nintro\n\
### One\n```mermaid\ngraph TD; ONE-->X\n```\n\
### Two\nno diagram here\n";
        let d = parse_document(md);
        let dsl: ExtractionDsl = serde_yaml::from_str(
            r#"
yield_pattern:
  iterate_over:
    section_path: [Steps]
    kind: heading
    depth: 1
  per_match:
    diagram:
      from: code_block
      language: mermaid
      required: true
"#,
        )
        .unwrap();
        let err = extract(&d, &dsl).expect_err("unit Two has no block");
        assert!(matches!(
            err,
            QuireError::MissingField { key, .. } if key == "diagram"
        ));
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

    #[test]
    fn multi_yield_required_miss_returns_missing_field() {
        let d = parse_document("## Algorithms\nintro\n### A\nbody a\n### B\nbody b\n");
        let dsl = dsl_from(
            r#"
yield_pattern:
  iterate_over:
    section_path: [Algorithms]
    kind: heading
    depth: 1
  per_match:
    example:
      from: code_block
      after_heading: Example
      required: true
"#,
        );
        let err = extract(&d, &dsl).expect_err("missing required per_match field");
        assert!(matches!(
            err,
            QuireError::MissingField { key, .. } if key == "example"
        ));
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

    // TC-539 (FR-033-AC-6): extraction ignores the `assert` facet — the
    // extracted value is byte-identical with and without `assert`.
    #[test]
    fn extraction_ignores_assert_facet() {
        let d = parse_document(
            "---\nid: FR-1\n---\n## AC\n| ID | Criteria |\n| - | - |\n| AC-1 | a |\n",
        );
        let without = dsl_from(
            r#"
yield_pattern:
  match:
    ids:
      from: table_row
      under_section: AC
      column: ID
"#,
        );
        let with = dsl_from(
            r#"
yield_pattern:
  match:
    ids:
      from: table_row
      under_section: AC
      column: ID
      assert:
        columns: [ID, Criteria]
        min_rows: 1
        id_column: ID
        id_pattern: '^AC-\d+$'
"#,
        );
        let r_without = extract(&d, &without).expect("ok");
        let r_with = extract(&d, &with).expect("ok");
        assert_eq!(
            r_without, r_with,
            "extraction output must not change when an assert is present"
        );
    }

    // ── FR-011 back-fills (TC-565..569) ─────────────────────────────────

    // TC-565 (FR-011-AC-15): per-locator `regex:` projection. `(\d+)`
    // yields capture group 1; `\d+` (no group) yields group 0; a non-match
    // drops the key (required:false) or returns MissingField (required:true);
    // an invalid (uncompilable) regex yields an empty projected value.
    #[test]
    fn tc565_regex_projection() {
        let d = parse_document("---\nid: FR-001\n---\n## V\nrelease 42 build\n");

        // Capture group 1.
        let g1 = dsl_from(
            "yield_pattern:\n  match:\n    n:\n      from: section_body\n      after_heading: V\n      regex: '(\\d+)'\n",
        );
        let r = extract(&d, &g1).expect("ok");
        assert_eq!(r.records[0]["n"], serde_json::json!("42"));

        // No group → group 0 (whole match).
        let g0 = dsl_from(
            "yield_pattern:\n  match:\n    n:\n      from: section_body\n      after_heading: V\n      regex: 'release \\d+'\n",
        );
        let r0 = extract(&d, &g0).expect("ok");
        assert_eq!(r0.records[0]["n"], serde_json::json!("release 42"));

        // Non-match, required:false → key dropped (a companion required
        // key keeps the single record alive).
        let opt = dsl_from(
            "yield_pattern:\n  match:\n    id:\n      from: frontmatter_field\n      path: [id]\n      required: true\n    n:\n      from: section_body\n      after_heading: V\n      required: false\n      regex: 'ZZZ(\\d+)'\n",
        );
        let ro = extract(&d, &opt).expect("ok");
        assert_eq!(ro.records.len(), 1);
        assert!(!ro.records[0].contains_key("n"));

        // Non-match, required:true → MissingField.
        let req = dsl_from(
            "yield_pattern:\n  match:\n    n:\n      from: section_body\n      after_heading: V\n      required: true\n      regex: 'ZZZ(\\d+)'\n",
        );
        assert!(matches!(
            extract(&d, &req).expect_err("miss"),
            QuireError::MissingField { .. }
        ));

        // Invalid regex (unterminated group) → empty projected value, no
        // panic. required:false so the key is simply absent; a companion
        // required key keeps the record alive.
        let bad = dsl_from(
            "yield_pattern:\n  match:\n    id:\n      from: frontmatter_field\n      path: [id]\n      required: true\n    n:\n      from: section_body\n      after_heading: V\n      required: false\n      regex: '(unterminated'\n",
        );
        let rb = extract(&d, &bad).expect("no panic");
        assert_eq!(rb.records.len(), 1);
        assert!(!rb.records[0].contains_key("n"));
    }

    // TC-566 (FR-011-AC-16): `under_section: None` substrate. `table_row`
    // resolves against the joined body using the first table; `list_item`
    // and `code_block` read the joined-body substrate.
    #[test]
    fn tc566_under_section_none_substrate() {
        let d = parse_document(
            "## A\n| K | V |\n| - | - |\n| a | 1 |\n\
             ## B\n- item-one\n- item-two\n\
             ## C\n```json\n{\"x\":1}\n```\n",
        );

        // table_row, under_section omitted → first table in joined body.
        let t = dsl_from("yield_pattern:\n  match:\n    rows:\n      from: table_row\n");
        let rt = extract(&d, &t).expect("ok");
        assert!(rt.records[0]["rows"].as_str().unwrap().contains("a"));

        // list_item, under_section omitted → joined-body bullets.
        let l = dsl_from("yield_pattern:\n  match:\n    items:\n      from: list_item\n");
        let rl = extract(&d, &l).expect("ok");
        let joined = rl.records[0]["items"].as_str().unwrap();
        assert!(joined.contains("item-one") || joined.contains("item-two"));

        // code_block, under_section omitted → joined-body fenced block.
        let c = dsl_from(
            "yield_pattern:\n  match:\n    code:\n      from: code_block\n      language: json\n",
        );
        let rc = extract(&d, &c).expect("ok");
        assert!(rc.records[0]["code"].as_str().unwrap().contains("\"x\""));
    }

    // TC-567 (FR-011-AC-17): a whole-value `{{ id }}` resolved value
    // contributes no extracted value (placeholder); an embedded `{{x}}`
    // mid-prose does not trigger the rule and the surrounding content
    // extracts normally.
    #[test]
    fn tc567_whole_value_mustache_is_placeholder() {
        // A companion required key keeps the single record alive while the
        // whole-value `{{ id }}` `s` key is dropped.
        let whole = parse_document("---\nid: FR-1\n---\n## S\n{{ id }}\n");
        let dsl = dsl_from(
            "yield_pattern:\n  match:\n    keep:\n      from: frontmatter_field\n      path: [id]\n      required: true\n    s:\n      from: section_body\n      after_heading: S\n      required: false\n",
        );
        let r = extract(&whole, &dsl).expect("ok");
        assert_eq!(r.records.len(), 1);
        assert!(
            !r.records[0].contains_key("s"),
            "whole-value {{{{…}}}} should contribute nothing, got {:?}",
            r.records[0]
        );

        // A required whole-value marker → MissingField.
        let req = dsl_from(
            "yield_pattern:\n  match:\n    s:\n      from: section_body\n      after_heading: S\n      required: true\n",
        );
        assert!(matches!(
            extract(&whole, &req).expect_err("placeholder is a required miss"),
            QuireError::MissingField { .. }
        ));

        // Embedded token mid-prose → surrounding content extracted.
        let embedded = parse_document("## S\nThe value {{x}} appears mid-sentence here.\n");
        let only_s = dsl_from(
            "yield_pattern:\n  match:\n    s:\n      from: section_body\n      after_heading: S\n      required: true\n",
        );
        let r2 = extract(&embedded, &only_s).expect("ok");
        assert!(r2.records[0]["s"]
            .as_str()
            .unwrap()
            .contains("mid-sentence"));
    }

    // TC-568 (FR-011-AC-18): an unclosed fenced block (both ``` and ~~~)
    // is flushed as the final block — trailing content is part of the
    // block, not a phantom following block.
    #[test]
    fn tc568_unclosed_fence_flushed_as_final_block() {
        for fence in ["```", "~~~"] {
            let md = format!("## Code\n{fence}rust\nfn main() {{}}\nstill in block\n");
            let d = parse_document(&md);
            let dsl = dsl_from(
                "yield_pattern:\n  match:\n    code:\n      from: code_block\n      under_section: Code\n      language: rust\n",
            );
            let r = extract(&d, &dsl).expect("ok");
            let code = r.records[0]["code"].as_str().unwrap();
            assert!(
                code.contains("still in block"),
                "unclosed {fence} block should include trailing content, got {code:?}"
            );
        }
    }

    // TC-569 (FR-011-AC-19): `emit_edges` projects one edge per record
    // whose field resolves; records lacking the field emit no edge.
    #[test]
    fn tc569_emit_edges_per_record() {
        // Multi-yield over ### children of ## Items; each unit may carry a
        // `target:` frontmatter-free section_body that resolves to an id.
        let md = "## Items\n\
                  ### One\n#### Ref\nFR-100\n\
                  ### Two\n(no ref child)\n";
        let d = parse_document(md);
        let dsl = dsl_from(
            "yield_pattern:\n  iterate_over:\n    section_path: [Items]\n    kind: heading\n    depth: 1\n  per_match:\n    name:\n      from: heading\n      required: true\nemit_edges:\n  - type: references\n    target:\n      from: section_body\n      after_heading: Ref\n",
        );
        let r = extract(&d, &dsl).expect("ok");
        // Two records (One, Two); only the first has a Ref child → one edge.
        assert_eq!(r.records.len(), 2);
        assert_eq!(r.edges.len(), 1, "{:?}", r.edges);
        assert_eq!(r.edges[0].record_index, 0);
        assert_eq!(r.edges[0].edge_type, "references");
        assert_eq!(r.edges[0].target, "FR-100");
    }
}
