//! Assert-facet evaluation (FR-033 + FR-034) in the *asserting posture*.
//!
//! `validate_document` (FR-032) runs a locator's optional `assert:`
//! facet against a parsed document. Each check that fails produces an
//! [`AssertFailure`] carrying a `reason` (`assert` for structural
//! mismatches, or a missing-`{field}` interpolation failure) plus a
//! human message and the 0-based body line of the located section (when
//! known) so the caller can build a line-numbered diagnostic.
//!
//! This module is pure over a [`QuireDocument`] and the document's
//! frontmatter; it does no IO and never panics. The extractor
//! (`crate::extract::extract`) never calls it — `assert` is ignored on
//! the extract path (FR-033-AC-6).

use regex::Regex;
use serde_json::{Map, Value};

use crate::ast::QuireDocument;
use crate::extract::interpolate::interpolate;
use crate::extract::locator::{ColumnRef, LocatorAssert, LocatorKind, LocatorPrimitive};
use crate::query::{parse_bullet_list, parse_table, section as q_section, sections as q_sections};

/// Why an assert check failed. Maps onto `validate_document`'s reason
/// vocabulary: `assert` for a structural mismatch, `unresolved-field`
/// for a missing `{field}` interpolation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssertReason {
    /// A structural assertion (`level`, `columns`, `min_rows`,
    /// `min_items`, `id_pattern`, `matches`) did not hold.
    Assert,
    /// A `{field}` token referenced a frontmatter key that is absent
    /// (FR-034-AC-2).
    UnresolvedField,
}

/// One failed assert check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssertFailure {
    pub reason: AssertReason,
    pub message: String,
    /// 0-based body line of the located section, if known.
    pub line: Option<usize>,
}

/// Evaluate `assert` for `primitive` against `doc`. Returns every failed
/// check (empty = all asserts hold). `frontmatter` supplies `{field}`
/// interpolation values (FR-034).
pub fn evaluate_assert(
    doc: &QuireDocument,
    primitive: &LocatorPrimitive,
    assert: &LocatorAssert,
    frontmatter: Option<&Map<String, Value>>,
) -> Vec<AssertFailure> {
    if assert.is_empty() {
        return Vec::new();
    }
    let mut failures = Vec::new();

    // `level` — required heading level of the located section.
    if let Some(level) = assert.level {
        check_level(doc, primitive, level, &mut failures);
    }

    // Table asserts — `columns`, `min_rows`, `id_column` + `id_pattern`.
    if matches!(primitive.kind(), LocatorKind::TableRow) {
        check_table(doc, primitive, assert, frontmatter, &mut failures);
    }

    // `min_items` — list length.
    if let Some(min_items) = assert.min_items {
        check_min_items(doc, primitive, min_items, &mut failures);
    }

    // `id_pattern` without `id_column` — apply to located scalar values
    // (heading text / section / frontmatter / list items).
    if assert.id_pattern.is_some()
        && assert.id_column.is_none()
        && !matches!(primitive.kind(), LocatorKind::TableRow)
    {
        check_value_pattern(doc, primitive, assert, frontmatter, &mut failures);
    }

    // `matches` — the located content must match a regex. Unlike the
    // required/content_status checks, a locator that resolves to NO values
    // (missing/empty section) does NOT fire here — absence is caught
    // elsewhere (FR-033).
    if assert.matches.is_some() {
        check_content_matches(doc, primitive, assert, frontmatter, &mut failures);
    }

    // `choices` — the located scalar value must be one of a fixed set
    // (CR-010). Like `matches`, a locator that resolves to no values does
    // NOT fire. Table columns use `column_choices` (handled in `check_table`).
    if assert.choices.is_some() && !matches!(primitive.kind(), LocatorKind::TableRow) {
        check_value_choices(doc, primitive, assert, &mut failures);
    }

    failures
}

fn located_section_name(primitive: &LocatorPrimitive) -> Option<String> {
    match primitive {
        LocatorPrimitive::SectionBody { after_heading, .. } => Some(after_heading.clone()),
        LocatorPrimitive::TableRow { under_section, .. }
        | LocatorPrimitive::ListItem { under_section, .. } => under_section.clone(),
        _ => None,
    }
}

fn check_level(
    doc: &QuireDocument,
    primitive: &LocatorPrimitive,
    level: u8,
    failures: &mut Vec<AssertFailure>,
) {
    // The section the locator addresses by name. For `heading` with a
    // `level` already set, this is a no-op (the locator selects by level
    // and the assert just confirms it).
    let target = match primitive {
        LocatorPrimitive::SectionBody { after_heading, .. } => q_section(doc, after_heading),
        LocatorPrimitive::Heading { path: Some(p), .. } => {
            p.last().and_then(|name| q_section(doc, name))
        }
        LocatorPrimitive::Heading { level: Some(_), .. } => {
            // Heading-by-level: confirm at least one section exists at
            // the asserted level.
            let any = q_sections(doc, Some(level)).into_iter().next();
            if any.is_none() {
                failures.push(AssertFailure {
                    reason: AssertReason::Assert,
                    message: format!("no heading found at asserted level {level}"),
                    line: None,
                });
            }
            return;
        }
        _ => None,
    };
    match target {
        Some(s) if s.level == level => {}
        Some(s) => failures.push(AssertFailure {
            reason: AssertReason::Assert,
            message: format!(
                "section '{}' is level {} but assert requires level {level}",
                s.heading, s.level
            ),
            line: Some(s.start_line),
        }),
        None => failures.push(AssertFailure {
            reason: AssertReason::Assert,
            message: format!("section not found for level assert (required level {level})"),
            line: None,
        }),
    }
}

fn locate_table(
    doc: &QuireDocument,
    primitive: &LocatorPrimitive,
) -> Option<(crate::query::TableResult, Option<usize>)> {
    let section_name = located_section_name(primitive);
    match section_name {
        Some(name) => {
            let s = q_section(doc, &name)?;
            parse_table(&s.content).map(|t| (t, Some(s.start_line)))
        }
        None => doc
            .sections
            .iter()
            .find_map(|s| parse_table(&s.content).map(|t| (t, Some(s.start_line)))),
    }
}

fn check_table(
    doc: &QuireDocument,
    primitive: &LocatorPrimitive,
    assert: &LocatorAssert,
    frontmatter: Option<&Map<String, Value>>,
    failures: &mut Vec<AssertFailure>,
) {
    let Some((table, line)) = locate_table(doc, primitive) else {
        // Surface a single failure per declared table assert so the
        // missing table is reported (not silently passing).
        if assert.columns.is_some()
            || assert.min_rows.is_some()
            || assert.id_pattern.is_some()
            || assert.column_choices.is_some()
            || assert.column_patterns.is_some()
        {
            failures.push(AssertFailure {
                reason: AssertReason::Assert,
                message: "table not found for table_row assert".to_string(),
                line: None,
            });
        }
        return;
    };

    if let Some(columns) = &assert.columns {
        if &table.headers != columns {
            failures.push(AssertFailure {
                reason: AssertReason::Assert,
                message: format!(
                    "table columns {:?} do not match asserted columns {:?}",
                    table.headers, columns
                ),
                line,
            });
        }
    }

    if let Some(min_rows) = assert.min_rows {
        if table.rows.len() < min_rows {
            failures.push(AssertFailure {
                reason: AssertReason::Assert,
                message: format!(
                    "table has {} data row(s) but assert requires at least {min_rows}",
                    table.rows.len()
                ),
                line,
            });
        }
    }

    if let Some(id_pattern) = &assert.id_pattern {
        let re = match resolve_regex(id_pattern, frontmatter) {
            Ok(re) => re,
            Err(failure) => {
                failures.push(failure);
                return;
            }
        };
        // Resolve the id column index.
        let col_idx = match &assert.id_column {
            Some(name) => table.headers.iter().position(|h| h == name),
            // Fall back to a `column` projection on the locator, else
            // column 0.
            None => match primitive {
                LocatorPrimitive::TableRow {
                    column: Some(ColumnRef::Name(name)),
                    ..
                } => table.headers.iter().position(|h| h == name),
                LocatorPrimitive::TableRow {
                    column: Some(ColumnRef::Index(n)),
                    ..
                } => n.checked_sub(1),
                _ => Some(0),
            },
        };
        let Some(idx) = col_idx else {
            failures.push(AssertFailure {
                reason: AssertReason::Assert,
                message: format!(
                    "id_column {:?} not found in table headers {:?}",
                    assert.id_column, table.headers
                ),
                line,
            });
            return;
        };
        for row in &table.rows {
            let cell = row.get(idx).map(String::as_str).unwrap_or("");
            if !re.is_match(cell) {
                failures.push(AssertFailure {
                    reason: AssertReason::Assert,
                    message: format!("id cell '{cell}' does not match pattern /{}/", re.as_str()),
                    line,
                });
            }
        }
    }

    // Per-column enum constraints (CR-010): every data cell in the named
    // column must be one of the listed values (trimmed, exact match).
    if let Some(col_choices) = &assert.column_choices {
        for (col, allowed) in col_choices {
            check_table_column_choices(&table, col, allowed, line, failures);
        }
    }

    // Per-column regex constraints (CR-010): every data cell in the named
    // column must match the (interpolated) pattern.
    if let Some(col_patterns) = &assert.column_patterns {
        for (col, pattern) in col_patterns {
            let re = match resolve_regex(pattern, frontmatter) {
                Ok(re) => re,
                Err(failure) => {
                    failures.push(failure);
                    continue;
                }
            };
            check_table_column_pattern(&table, col, &re, line, failures);
        }
    }
}

/// Resolve a named table column to its index, pushing a "column not found"
/// failure when absent (CR-010).
fn table_column_index(
    table: &crate::query::TableResult,
    col: &str,
    line: Option<usize>,
    failures: &mut Vec<AssertFailure>,
) -> Option<usize> {
    match table.headers.iter().position(|h| h == col) {
        Some(idx) => Some(idx),
        None => {
            failures.push(AssertFailure {
                reason: AssertReason::Assert,
                message: format!(
                    "column '{col}' not found in table headers {:?}",
                    table.headers
                ),
                line,
            });
            None
        }
    }
}

fn check_table_column_choices(
    table: &crate::query::TableResult,
    col: &str,
    allowed: &[String],
    line: Option<usize>,
    failures: &mut Vec<AssertFailure>,
) {
    let Some(idx) = table_column_index(table, col, line, failures) else {
        return;
    };
    for row in &table.rows {
        let cell = row.get(idx).map(|s| s.trim()).unwrap_or("");
        if !allowed.iter().any(|a| a == cell) {
            failures.push(AssertFailure {
                reason: AssertReason::Assert,
                message: format!("column '{col}' cell '{cell}' is not one of {allowed:?}"),
                line,
            });
        }
    }
}

fn check_table_column_pattern(
    table: &crate::query::TableResult,
    col: &str,
    re: &Regex,
    line: Option<usize>,
    failures: &mut Vec<AssertFailure>,
) {
    let Some(idx) = table_column_index(table, col, line, failures) else {
        return;
    };
    for row in &table.rows {
        let cell = row.get(idx).map(String::as_str).unwrap_or("");
        if !re.is_match(cell) {
            failures.push(AssertFailure {
                reason: AssertReason::Assert,
                message: format!(
                    "column '{col}' cell '{cell}' does not match pattern /{}/",
                    re.as_str()
                ),
                line,
            });
        }
    }
}

/// `choices` — the located scalar value(s) must each be one of a fixed set
/// (CR-010). A locator that resolves to no values does not fire.
fn check_value_choices(
    doc: &QuireDocument,
    primitive: &LocatorPrimitive,
    assert: &LocatorAssert,
    failures: &mut Vec<AssertFailure>,
) {
    let Some(choices) = &assert.choices else {
        return;
    };
    let line = located_section_name(primitive)
        .and_then(|name| q_section(doc, &name).map(|s| s.start_line));
    for v in &crate::extract::locator::eval(doc, primitive) {
        if let Some(s) = v.as_str() {
            let value = s.trim();
            if !choices.iter().any(|c| c == value) {
                failures.push(AssertFailure {
                    reason: AssertReason::Assert,
                    message: format!("value '{value}' is not one of {choices:?}"),
                    line,
                });
            }
        }
    }
}

fn check_min_items(
    doc: &QuireDocument,
    primitive: &LocatorPrimitive,
    min_items: usize,
    failures: &mut Vec<AssertFailure>,
) {
    let (content, line) = match located_section_name(primitive) {
        Some(name) => match q_section(doc, &name) {
            Some(s) => (s.content.clone(), Some(s.start_line)),
            None => {
                failures.push(AssertFailure {
                    reason: AssertReason::Assert,
                    message: format!("section '{name}' not found for min_items assert"),
                    line: None,
                });
                return;
            }
        },
        None => (
            doc.sections
                .iter()
                .map(|s| s.content.as_str())
                .collect::<Vec<_>>()
                .join("\n"),
            None,
        ),
    };
    let count = parse_bullet_list(&content, None).len();
    if count < min_items {
        failures.push(AssertFailure {
            reason: AssertReason::Assert,
            message: format!("list has {count} item(s) but assert requires at least {min_items}"),
            line,
        });
    }
}

fn check_value_pattern(
    doc: &QuireDocument,
    primitive: &LocatorPrimitive,
    assert: &LocatorAssert,
    frontmatter: Option<&Map<String, Value>>,
    failures: &mut Vec<AssertFailure>,
) {
    let Some(pattern) = &assert.id_pattern else {
        return;
    };
    let re = match resolve_regex(pattern, frontmatter) {
        Ok(re) => re,
        Err(failure) => {
            failures.push(failure);
            return;
        }
    };
    let values = crate::extract::locator::eval(doc, primitive);
    for v in &values {
        if let Some(s) = v.as_str() {
            if !re.is_match(s) {
                failures.push(AssertFailure {
                    reason: AssertReason::Assert,
                    message: format!("value '{s}' does not match pattern /{}/", re.as_str()),
                    line: None,
                });
            }
        }
    }
}

fn check_content_matches(
    doc: &QuireDocument,
    primitive: &LocatorPrimitive,
    assert: &LocatorAssert,
    frontmatter: Option<&Map<String, Value>>,
    failures: &mut Vec<AssertFailure>,
) {
    let Some(pattern) = &assert.matches else {
        return;
    };
    let re = match resolve_regex(pattern, frontmatter) {
        Ok(re) => re,
        Err(failure) => {
            failures.push(failure);
            return;
        }
    };
    // A locator that resolves to no values (missing/empty section) does not
    // fire `matches` — that absence is the required/content_status path's
    // job, not this one (FR-033).
    let values = crate::extract::locator::eval(doc, primitive);
    let line = located_section_name(primitive)
        .and_then(|name| q_section(doc, &name).map(|s| s.start_line));
    for v in &values {
        if let Some(s) = v.as_str() {
            if !re.is_match(s) {
                failures.push(AssertFailure {
                    reason: AssertReason::Assert,
                    message: format!(
                        "section content does not match required pattern /{}/",
                        re.as_str()
                    ),
                    line,
                });
            }
        }
    }
}

/// Interpolate `{field}` tokens (FR-034), then compile the regex. An
/// unresolved field returns an `UnresolvedField` failure; an invalid
/// regex returns an `Assert` failure.
fn resolve_regex(
    pattern: &str,
    frontmatter: Option<&Map<String, Value>>,
) -> Result<Regex, AssertFailure> {
    let resolved = interpolate(pattern, frontmatter).map_err(|e| AssertFailure {
        reason: AssertReason::UnresolvedField,
        message: format!(
            "assert pattern references absent frontmatter field '{}'",
            e.field
        ),
        line: None,
    })?;
    Regex::new(&resolved).map_err(|e| AssertFailure {
        reason: AssertReason::Assert,
        message: format!("assert pattern is not a valid regex: {e}"),
        line: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_document;

    fn prim(yaml: &str) -> LocatorPrimitive {
        serde_yaml::from_str(yaml).expect("parse locator")
    }

    // TC-534 (FR-033-AC-1): section_body assert level.
    #[test]
    fn tc534_level_assert() {
        let doc = parse_document("## Purpose\nbody\n### Deep\nx\n");
        let p = prim("from: section_body\nafter_heading: Purpose\nassert: { level: 2 }");
        let a = p.assert().unwrap().clone();
        assert!(evaluate_assert(&doc, &p, &a, doc.frontmatter.as_ref()).is_empty());

        let p3 = prim("from: section_body\nafter_heading: Deep\nassert: { level: 2 }");
        let a3 = p3.assert().unwrap().clone();
        let fails = evaluate_assert(&doc, &p3, &a3, None);
        assert_eq!(fails.len(), 1);
        assert_eq!(fails[0].reason, AssertReason::Assert);
        assert_eq!(fails[0].line, Some(2));
    }

    // TC-535 (FR-033-AC-2): table_row columns exact text + order.
    #[test]
    fn tc535_columns_assert() {
        let doc = parse_document(
            "## AC\n| ID | Criteria | Verification |\n| - | - | - |\n| AC-1 | x | y |\n",
        );
        let ok = prim(
            "from: table_row\nunder_section: AC\nassert: { columns: [ID, Criteria, Verification] }",
        );
        let a = ok.assert().unwrap().clone();
        assert!(evaluate_assert(&doc, &ok, &a, None).is_empty());

        // Wrong order.
        let bad = prim(
            "from: table_row\nunder_section: AC\nassert: { columns: [Criteria, ID, Verification] }",
        );
        let ab = bad.assert().unwrap().clone();
        assert_eq!(evaluate_assert(&doc, &bad, &ab, None).len(), 1);
    }

    // TC-536 (FR-033-AC-3): min_rows / min_items.
    #[test]
    fn tc536_min_rows_and_min_items() {
        let header_only = parse_document("## T\n| ID |\n| - |\n");
        let p = prim("from: table_row\nunder_section: T\nassert: { min_rows: 1 }");
        let a = p.assert().unwrap().clone();
        assert_eq!(evaluate_assert(&header_only, &p, &a, None).len(), 1);

        let empty_list = parse_document("## L\nno items here\n");
        let li = prim("from: list_item\nunder_section: L\nassert: { min_items: 1 }");
        let al = li.assert().unwrap().clone();
        assert_eq!(evaluate_assert(&empty_list, &li, &al, None).len(), 1);

        let with_item = parse_document("## L\n- one\n");
        assert!(evaluate_assert(&with_item, &li, &al, None).is_empty());
    }

    // TC-537 (FR-033-AC-4): id_column + id_pattern.
    #[test]
    fn tc537_id_pattern_assert() {
        let doc = parse_document(
            "## AC\n| ID | Criteria |\n| - | - |\n| FR-9-AC-1 | a |\n| FR-9-AC-2 | b |\n",
        );
        let ok = prim(
            "from: table_row\nunder_section: AC\nassert:\n  id_column: ID\n  id_pattern: '^[A-Z]{2,4}-\\d+-AC-\\d+$'",
        );
        let a = ok.assert().unwrap().clone();
        assert!(evaluate_assert(&doc, &ok, &a, None).is_empty());

        let bad = parse_document("## AC\n| ID | Criteria |\n| - | - |\n| nope | a |\n");
        assert_eq!(evaluate_assert(&bad, &ok, &a, None).len(), 1);
    }

    // TC-540 (FR-034-AC-1): id_pattern interpolation with this doc's id.
    #[test]
    fn tc540_id_pattern_interpolation() {
        let doc = parse_document(
            "---\nid: FR-900\n---\n## AC\n| ID |\n| - |\n| FR-900-AC-1 |\n| FR-900-AC-2 |\n",
        );
        let p = prim(
            "from: table_row\nunder_section: AC\nassert:\n  id_column: ID\n  id_pattern: '^{id}-AC-\\d+$'",
        );
        let a = p.assert().unwrap().clone();
        assert!(evaluate_assert(&doc, &p, &a, doc.frontmatter.as_ref()).is_empty());

        // A different doc's id is rejected.
        let other = parse_document("---\nid: FR-900\n---\n## AC\n| ID |\n| - |\n| FR-901-AC-1 |\n");
        let fails = evaluate_assert(&other, &p, &a, other.frontmatter.as_ref());
        assert_eq!(fails.len(), 1);
    }

    // TC-541 (FR-034-AC-2): unresolved field → reason unresolved-field.
    #[test]
    fn tc541_unresolved_field() {
        let doc = parse_document("## AC\n| ID |\n| - |\n| X-1 |\n");
        let p = prim(
            "from: table_row\nunder_section: AC\nassert:\n  id_column: ID\n  id_pattern: '^{missing}-\\d+$'",
        );
        let a = p.assert().unwrap().clone();
        let fails = evaluate_assert(&doc, &p, &a, doc.frontmatter.as_ref());
        assert_eq!(fails.len(), 1);
        assert_eq!(fails[0].reason, AssertReason::UnresolvedField);
        assert!(fails[0].message.contains("missing"));
    }

    // TC-542 (FR-034-AC-3): regex metacharacters in value are escaped.
    #[test]
    fn tc542_regex_escape() {
        let doc = parse_document("---\nid: A.B+\n---\n## AC\n| ID |\n| - |\n| A.B+ |\n");
        let p = prim(
            "from: table_row\nunder_section: AC\nassert:\n  id_column: ID\n  id_pattern: '^{id}$'",
        );
        let a = p.assert().unwrap().clone();
        assert!(evaluate_assert(&doc, &p, &a, doc.frontmatter.as_ref()).is_empty());
    }

    // TC-543 (FR-034-AC-4): static pattern with no token.
    #[test]
    fn tc543_static_pattern() {
        let doc = parse_document("## AC\n| ID |\n| - |\n| AC-1 |\n");
        let p = prim(
            "from: table_row\nunder_section: AC\nassert:\n  id_column: ID\n  id_pattern: '^AC-\\d+$'",
        );
        let a = p.assert().unwrap().clone();
        assert!(evaluate_assert(&doc, &p, &a, None).is_empty());
    }

    // TC-571 (FR-033-AC-8): `id_column` resolution precedence on a
    // table_row locator is `assert.id_column` → locator `column` → col 0.
    #[test]
    fn tc571_id_column_resolution_precedence() {
        // Table: col0 = Name (no match), col1 = Key (matches ^K-).
        let doc =
            parse_document("## T\n| Name | Key |\n| - | - |\n| alpha | K-1 |\n| beta | K-2 |\n");

        // (1) all three present: explicit assert.id_column wins (→ Key).
        // `column: Name` + col-0 would both point elsewhere; only Key matches.
        let p1 = prim(
            "from: table_row\nunder_section: T\ncolumn: Name\nassert:\n  id_column: Key\n  id_pattern: '^K-\\d+$'",
        );
        let a1 = p1.assert().unwrap().clone();
        assert!(
            evaluate_assert(&doc, &p1, &a1, None).is_empty(),
            "assert.id_column should win and resolve to Key"
        );

        // (2) id_column absent → falls back to the locator `column` (Key).
        let p2 = prim(
            "from: table_row\nunder_section: T\ncolumn: Key\nassert:\n  id_pattern: '^K-\\d+$'",
        );
        let a2 = p2.assert().unwrap().clone();
        assert!(
            evaluate_assert(&doc, &p2, &a2, None).is_empty(),
            "locator column should resolve to Key"
        );
        // Sanity: pointing `column` at Name (col 0) fails the ^K- pattern.
        let p2b = prim(
            "from: table_row\nunder_section: T\ncolumn: Name\nassert:\n  id_pattern: '^K-\\d+$'",
        );
        let a2b = p2b.assert().unwrap().clone();
        assert_eq!(evaluate_assert(&doc, &p2b, &a2b, None).len(), 2);

        // (3) both absent → column index 0 (Name) is used.
        let p3 = prim("from: table_row\nunder_section: T\nassert:\n  id_pattern: '^(alpha|beta)$'");
        let a3 = p3.assert().unwrap().clone();
        assert!(
            evaluate_assert(&doc, &p3, &a3, None).is_empty(),
            "no id_column/column → column 0 (Name)"
        );
    }

    // TC-572 (FR-033-AC-9): `id_pattern` on non-table locators applies to
    // the located scalar value — heading text, section first-line/id
    // token, each list item, and the frontmatter scalar. A mismatch fails
    // with reason `assert`; a match passes.
    #[test]
    fn tc572_id_pattern_on_non_table_locators() {
        // heading: matches the heading text.
        let doc = parse_document("## Algo-1: first\n### sub\nx\n");
        let h_ok = prim("from: heading\nlevel: 2\nassert:\n  id_pattern: '^Algo-\\d+:'");
        let ah = h_ok.assert().unwrap().clone();
        assert!(evaluate_assert(&doc, &h_ok, &ah, None).is_empty());
        let h_bad = prim("from: heading\nlevel: 2\nassert:\n  id_pattern: '^ZZZ-'");
        let ahb = h_bad.assert().unwrap().clone();
        assert_eq!(evaluate_assert(&doc, &h_bad, &ahb, None).len(), 1);

        // section_body: matches the section's first line / id token.
        let sdoc = parse_document("## Body\nFR-1 is the lead token\n");
        let s_ok =
            prim("from: section_body\nafter_heading: Body\nassert:\n  id_pattern: 'FR-\\d+'");
        let asok = s_ok.assert().unwrap().clone();
        assert!(evaluate_assert(&sdoc, &s_ok, &asok, None).is_empty());
        let s_bad =
            prim("from: section_body\nafter_heading: Body\nassert:\n  id_pattern: '^NOPE-'");
        let asbad = s_bad.assert().unwrap().clone();
        assert_eq!(evaluate_assert(&sdoc, &s_bad, &asbad, None).len(), 1);

        // list_item: matches each item.
        let ldoc = parse_document("## L\n- LI-1\n- LI-2\n");
        let l_ok = prim("from: list_item\nunder_section: L\nassert:\n  id_pattern: '^LI-\\d+$'");
        let alok = l_ok.assert().unwrap().clone();
        assert!(evaluate_assert(&ldoc, &l_ok, &alok, None).is_empty());
        let lbad = parse_document("## L\n- LI-1\n- bogus\n");
        assert_eq!(evaluate_assert(&lbad, &l_ok, &alok, None).len(), 1);

        // frontmatter_field: matches the scalar value.
        let fdoc = parse_document("---\nid: FR-7\n---\n# H\n");
        let f_ok = prim("from: frontmatter_field\npath: [id]\nassert:\n  id_pattern: '^FR-\\d+$'");
        let afok = f_ok.assert().unwrap().clone();
        assert!(evaluate_assert(&fdoc, &f_ok, &afok, fdoc.frontmatter.as_ref()).is_empty());
        let f_bad = prim("from: frontmatter_field\npath: [id]\nassert:\n  id_pattern: '^XX-'");
        let afbad = f_bad.assert().unwrap().clone();
        assert_eq!(
            evaluate_assert(&fdoc, &f_bad, &afbad, fdoc.frontmatter.as_ref()).len(),
            1
        );
    }

    // TC-608 (FR-033-AC-10): a `section_body` `matches` regex asserts the
    // located content shape. A body carrying the `As a … / I want … / So
    // that …` shape passes; one lacking it fails with reason `assert`; a
    // missing section does NOT fire `matches` (absence is validation's job).
    #[test]
    fn tc608_section_body_matches_pattern() {
        let story = parse_document(
            "## Story\n**As a** developer\n**I want** a feature\n**So that** I ship it\n",
        );
        let ok = prim(
            "from: section_body\nafter_heading: Story\nassert:\n  matches: '(?is)as an?\\b.+i want\\b.+so that\\b'",
        );
        let a = ok.assert().unwrap().clone();
        assert!(evaluate_assert(&story, &ok, &a, None).is_empty());

        // A body lacking the shape fails with reason `assert`, line-numbered
        // at the section heading.
        let no_shape = parse_document("## Story\njust some prose, no story keywords\n");
        let fails = evaluate_assert(&no_shape, &ok, &a, None);
        assert_eq!(fails.len(), 1);
        assert_eq!(fails[0].reason, AssertReason::Assert);
        assert_eq!(fails[0].line, Some(0));
        assert!(fails[0].message.contains("does not match"));

        // A missing section resolves to no values → `matches` does NOT fire.
        let missing = parse_document("## Other\nunrelated\n");
        assert!(
            evaluate_assert(&missing, &ok, &a, None).is_empty(),
            "missing section must not produce a `matches` failure"
        );
    }

    // TC-633 (FR-033-AC-11): `choices` scalar enum on a `section_body`
    // locator passes on a member value, fails reason `assert` on a
    // non-member, and does NOT fire when the section is absent.
    #[test]
    fn tc633_choices_scalar_enum() {
        let p = prim(
            "from: section_body\nafter_heading: Severity\nassert:\n  choices: [low, medium, high]",
        );
        let a = p.assert().unwrap().clone();

        let ok = parse_document("## Severity\nmedium\n");
        assert!(evaluate_assert(&ok, &p, &a, None).is_empty());

        let bad = parse_document("## Severity\nhuge\n");
        let fails = evaluate_assert(&bad, &p, &a, None);
        assert_eq!(fails.len(), 1);
        assert_eq!(fails[0].reason, AssertReason::Assert);
        assert!(fails[0].message.contains("is not one of"));

        // Absent section → no values → `choices` does not fire.
        let missing = parse_document("## Other\nx\n");
        assert!(evaluate_assert(&missing, &p, &a, None).is_empty());
    }

    // TC-634 (FR-033-AC-12): `column_choices` constrains every cell in a
    // named table column; an absent column fails "column not found".
    #[test]
    fn tc634_column_choices() {
        let doc = parse_document(
            "## Findings\n| ID | Severity |\n| - | - |\n| FND-1 | medium |\n| FND-2 | low |\n",
        );
        let ok = prim(
            "from: table_row\nunder_section: Findings\nassert:\n  column_choices:\n    Severity: [low, medium, high]",
        );
        let aok = ok.assert().unwrap().clone();
        assert!(evaluate_assert(&doc, &ok, &aok, None).is_empty());

        let bad = parse_document(
            "## Findings\n| ID | Severity |\n| - | - |\n| FND-1 | medium |\n| FND-2 | huge |\n",
        );
        assert_eq!(evaluate_assert(&bad, &ok, &aok, None).len(), 1);

        // Named column absent → "column not found".
        let absent = prim(
            "from: table_row\nunder_section: Findings\nassert:\n  column_choices:\n    Nope: [a]",
        );
        let aabsent = absent.assert().unwrap().clone();
        let fails = evaluate_assert(&doc, &absent, &aabsent, None);
        assert_eq!(fails.len(), 1);
        assert!(fails[0].message.contains("not found"));
    }

    // TC-635 (FR-033-AC-13): `column_patterns` regex-validates every cell in
    // a named table column; supports `{field}` interpolation; absent column
    // fails "column not found".
    #[test]
    fn tc635_column_patterns() {
        let doc = parse_document(
            "## Findings\n| ID | Severity |\n| - | - |\n| FND-1 | medium |\n| FND-2 | low |\n",
        );
        let ok = prim(
            "from: table_row\nunder_section: Findings\nassert:\n  column_patterns:\n    ID: '^FND-\\d+$'",
        );
        let aok = ok.assert().unwrap().clone();
        assert!(evaluate_assert(&doc, &ok, &aok, None).is_empty());

        let bad = parse_document(
            "## Findings\n| ID | Severity |\n| - | - |\n| FND-1 | medium |\n| nope | low |\n",
        );
        assert_eq!(evaluate_assert(&bad, &ok, &aok, None).len(), 1);

        // Named column absent → "column not found".
        let absent = prim(
            "from: table_row\nunder_section: Findings\nassert:\n  column_patterns:\n    Missing: '^.'",
        );
        let aabsent = absent.assert().unwrap().clone();
        let fails = evaluate_assert(&doc, &absent, &aabsent, None);
        assert_eq!(fails.len(), 1);
        assert!(fails[0].message.contains("not found"));
    }
}
