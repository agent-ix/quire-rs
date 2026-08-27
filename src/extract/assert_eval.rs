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
use crate::query::{
    parse_bullet_list, parse_table_with_lines, section as q_section, sections as q_sections,
};

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
    /// 0-based body line. For a **row-scoped** failure this is the offending
    /// row's own line (FR-033-AC-16, CR-097); for a table-scoped one it is the
    /// located section's start line, as before.
    pub line: Option<usize>,
    /// The offending row's id, when the assert declares an `id_column` and the
    /// failure is about one row (FR-033-AC-16, CR-097).
    ///
    /// Without it, a per-cell failure named neither the row it came from nor a
    /// line that distinguished it: measured on `agent-ix/filament-ide-rs`, 496
    /// `[assert]` findings shared **one distinct line per document** and 15
    /// carried a row id anywhere, so two byte-identical strings at one locus
    /// were the whole report for two different rows.
    pub row_id: Option<String>,
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
                    row_id: None,
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
            row_id: None,
        }),
        None => failures.push(AssertFailure {
            reason: AssertReason::Assert,
            message: format!("section not found for level assert (required level {level})"),
            line: None,
            row_id: None,
        }),
    }
}

/// The located table, the section's start line, and each data row's own
/// **document** line (FR-033-AC-16, CR-097).
///
/// `parse_table_with_lines` returns row lines relative to the section's
/// **content**, which begins on the line *after* the heading. So a content
/// index `rel` sits at body line `start + rel + 1`, and `to_doc_line` adds the
/// frontmatter offset and the 1-based conversion on top.
///
/// The `+ 1` is the heading line, and omitting it (#254) put every row-scoped
/// assert failure on the `|---|---|` separator — a locus carrying no data. The
/// coverage path does the same conversion at `corpus::declared_tables::rows_of`
/// and lands on the data row; two paths disagreeing about one row's line is the
/// defect, so this arithmetic must keep matching that one.
fn locate_table(
    doc: &QuireDocument,
    primitive: &LocatorPrimitive,
) -> Option<(crate::query::TableResult, Option<usize>, Vec<usize>)> {
    let with_lines = |content: &str, start: usize| {
        parse_table_with_lines(content).map(|(t, rows, _)| {
            let rows = rows.into_iter().map(|r| r + start + 1).collect::<Vec<_>>();
            (t, Some(start), rows)
        })
    };
    match located_section_name(primitive) {
        Some(name) => {
            let s = q_section(doc, &name)?;
            with_lines(&s.content, s.start_line)
        }
        None => doc
            .sections
            .iter()
            .find_map(|s| with_lines(&s.content, s.start_line)),
    }
}

/// The offending row's identity, as the declared `id_column` cell.
///
/// `None` when the assert declares no `id_column` or the table has no such
/// header — the engine names the row by what the contract said identifies it,
/// and guesses nothing when the contract said nothing.
fn row_identity(
    table: &crate::query::TableResult,
    assert: &LocatorAssert,
    row: &[String],
) -> Option<String> {
    let col = assert.id_column.as_deref()?;
    let idx = table.headers.iter().position(|h| h == col)?;
    let cell = row.get(idx)?.trim();
    (!cell.is_empty()).then(|| cell.to_string())
}

fn check_table(
    doc: &QuireDocument,
    primitive: &LocatorPrimitive,
    assert: &LocatorAssert,
    frontmatter: Option<&Map<String, Value>>,
    failures: &mut Vec<AssertFailure>,
) {
    let Some((table, line, row_lines)) = locate_table(doc, primitive) else {
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
                row_id: None,
            });
        }
        return;
    };

    if let Some(columns) = &assert.columns {
        let optional = assert.optional_columns.as_deref().unwrap_or(&[]);
        if optional.is_empty() {
            if &table.headers != columns {
                failures.push(AssertFailure {
                    reason: AssertReason::Assert,
                    message: format!(
                        "table columns {:?} do not match asserted columns {:?}",
                        table.headers, columns
                    ),
                    line,
                    row_id: None,
                });
            }
        } else {
            // CR-023: headers must be an ordered subsequence of `columns` that
            // still contains every non-optional column. Order is preserved, so
            // a reordered or unknown header fails exactly as before; only a
            // declared-optional column may be absent.
            let mut it = table.headers.iter();
            let mut missing: Vec<&String> = Vec::new();
            let mut consumed = 0usize;
            for want in columns {
                match it.clone().next() {
                    Some(h) if h == want => {
                        it.next();
                        consumed += 1;
                    }
                    _ if optional.iter().any(|o| o == want) => missing.push(want),
                    _ => missing.push(want),
                }
            }
            let unknown = consumed != table.headers.len();
            let required_missing: Vec<&&String> = missing
                .iter()
                .filter(|m| !optional.iter().any(|o| o == **m))
                .collect();
            if unknown || !required_missing.is_empty() {
                failures.push(AssertFailure {
                    reason: AssertReason::Assert,
                    message: format!(
                        "table columns {:?} do not match asserted columns {:?} (optional: {:?})",
                        table.headers, columns, optional
                    ),
                    line,
                    row_id: None,
                });
            }
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
                row_id: None,
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
                row_id: None,
            });
            return;
        };
        for (n, row) in table.rows.iter().enumerate() {
            let cell = row.get(idx).map(String::as_str).unwrap_or("");
            if !re.is_match(cell) {
                failures.push(AssertFailure {
                    reason: AssertReason::Assert,
                    message: format!("id cell '{cell}' does not match pattern /{}/", re.as_str()),
                    line: row_lines.get(n).copied().or(line),
                    // The id itself is the offending cell, so naming it twice
                    // would read as two facts. The line is what this failure
                    // was missing.
                    row_id: None,
                });
            }
        }
    }

    // A column declared optional (CR-023) and genuinely absent is not a
    // per-value failure: `column_choices`/`column_patterns` describe what the
    // cells must look like *if authored*, and reporting "column not found" for
    // one the contract just said may be omitted would make `optional_columns`
    // useless on exactly the contracts that need it. An optional column that
    // IS present is checked like any other.
    let declared_optional = assert.optional_columns.as_deref().unwrap_or(&[]);
    let omitted_optional = |col: &str| {
        declared_optional.iter().any(|o| o == col) && !table.headers.iter().any(|h| h == col)
    };

    // Per-column enum constraints (CR-010): every data cell in the named
    // column must be one of the listed values (trimmed, exact match).
    if let Some(col_choices) = &assert.column_choices {
        for (col, allowed) in col_choices {
            if omitted_optional(col) {
                continue;
            }
            check_table_column_choices(&table, assert, col, allowed, line, &row_lines, failures);
        }
    }

    // Per-column regex constraints (CR-010): every data cell in the named
    // column must match the (interpolated) pattern.
    if let Some(col_patterns) = &assert.column_patterns {
        for (col, pattern) in col_patterns {
            if omitted_optional(col) {
                continue;
            }
            let re = match resolve_regex(pattern, frontmatter) {
                Ok(re) => re,
                Err(failure) => {
                    failures.push(failure);
                    continue;
                }
            };
            check_table_column_pattern(&table, assert, col, &re, line, &row_lines, failures);
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
                row_id: None,
            });
            None
        }
    }
}

fn check_table_column_choices(
    table: &crate::query::TableResult,
    assert: &LocatorAssert,
    col: &str,
    allowed: &[String],
    line: Option<usize>,
    row_lines: &[usize],
    failures: &mut Vec<AssertFailure>,
) {
    let Some(idx) = table_column_index(table, col, line, failures) else {
        return;
    };
    for (n, row) in table.rows.iter().enumerate() {
        let cell = row.get(idx).map(|s| s.trim()).unwrap_or("");
        if !allowed.iter().any(|a| a == cell) {
            failures.push(AssertFailure {
                reason: AssertReason::Assert,
                message: format!("column '{col}' cell '{cell}' is not one of {allowed:?}"),
                line: row_lines.get(n).copied().or(line),
                row_id: row_identity(table, assert, row),
            });
        }
    }
}

fn check_table_column_pattern(
    table: &crate::query::TableResult,
    assert: &LocatorAssert,
    col: &str,
    re: &Regex,
    line: Option<usize>,
    row_lines: &[usize],
    failures: &mut Vec<AssertFailure>,
) {
    let Some(idx) = table_column_index(table, col, line, failures) else {
        return;
    };
    for (n, row) in table.rows.iter().enumerate() {
        let cell = row.get(idx).map(String::as_str).unwrap_or("");
        if !re.is_match(cell) {
            failures.push(AssertFailure {
                reason: AssertReason::Assert,
                message: format!(
                    "column '{col}' cell '{cell}' does not match pattern /{}/",
                    re.as_str()
                ),
                line: row_lines.get(n).copied().or(line),
                row_id: row_identity(table, assert, row),
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
                    row_id: None,
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
                    row_id: None,
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
            row_id: None,
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
                    row_id: None,
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
                    row_id: None,
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
        row_id: None,
    })?;
    Regex::new(&resolved).map_err(|e| AssertFailure {
        reason: AssertReason::Assert,
        message: format!("assert pattern is not a valid regex: {e}"),
        line: None,
        row_id: None,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::parser::parse_document;
    use ix_trace_rs::trace;

    fn prim(yaml: &str) -> LocatorPrimitive {
        serde_yaml::from_str(yaml).expect("parse locator")
    }

    #[trace("TC-991", "FR-033-AC-16")]
    // a row-scoped failure names the row and its (CR-097)
    // own line; a table-scoped one keeps the section line and names no row.
    #[test]
    fn tc991_row_scoped_failures_carry_the_row_and_its_line() {
        // Two rows failing the same check. Before CR-097 these were two
        // byte-identical findings at one line — the whole report for two
        // different rows.
        let doc = parse_document(concat!(
            "## T\n",
            "| Test ID | Type |\n",
            "| - | - |\n",
            "| TC-001 | Unit |\n",
            "| TC-002 | Inspection |\n",
            "| TC-003 | Analysis |\n",
        ));
        let choices = prim(concat!(
            "from: table_row\n",
            "under_section: T\n",
            "assert:\n",
            "  id_column: Test ID\n",
            "  column_choices:\n",
            "    Type: [Unit]\n",
        ));
        let a = choices.assert().unwrap().clone();
        let fails = evaluate_assert(&doc, &choices, &a, None);
        assert_eq!(fails.len(), 2, "{fails:?}");

        let ids: Vec<Option<&str>> = fails.iter().map(|f| f.row_id.as_deref()).collect();
        assert_eq!(ids, vec![Some("TC-002"), Some("TC-003")]);

        // The ABSOLUTE body line of each offending row, not merely distinct
        // ones in order (#254). The original assertion here was `lines[0] <
        // lines[1]`, which every off-by-N satisfies — and one shipped: these
        // pointed at the row above, which on a markdown table is the
        // `|---|---|` separator. A relative assertion cannot catch a
        // translation, and FR-033-AC-16 asks for the row's OWN line.
        //
        // Body lines, 0-based: 0 `## T`, 1 header, 2 separator, 3 TC-001,
        // 4 TC-002, 5 TC-003. The two failing rows are TC-002 and TC-003.
        let lines: Vec<Option<usize>> = fails.iter().map(|f| f.line).collect();
        assert_eq!(
            lines,
            vec![Some(4), Some(5)],
            "each row reports its own body line, not the one above it"
        );
        // …and the two findings are no longer equal field for field.
        assert_ne!(fails[0], fails[1]);

        // `column_patterns` is the same shape.
        let patterns = prim(concat!(
            "from: table_row\n",
            "under_section: T\n",
            "assert:\n",
            "  id_column: Test ID\n",
            "  column_patterns:\n",
            "    Type: '^Unit$'\n",
        ));
        let a = patterns.assert().unwrap().clone();
        let fails = evaluate_assert(&doc, &patterns, &a, None);
        assert_eq!(
            fails
                .iter()
                .map(|f| f.row_id.as_deref())
                .collect::<Vec<_>>(),
            vec![Some("TC-002"), Some("TC-003")]
        );

        // A table-scoped failure keeps the section line and names no row:
        // `min_rows` is about the table, not about any row in it.
        let empty = parse_document(concat!("## T\n", "| Test ID | Type |\n", "| - | - |\n",));
        let rows = prim(concat!(
            "from: table_row\n",
            "under_section: T\n",
            "assert:\n",
            "  id_column: Test ID\n",
            "  min_rows: 1\n",
        ));
        let a = rows.assert().unwrap().clone();
        let fails = evaluate_assert(&empty, &rows, &a, None);
        assert_eq!(fails.len(), 1);
        assert_eq!(fails[0].row_id, None);

        // No `id_column` declared: the row line still lands, and no id is
        // guessed from the first cell.
        let undeclared = prim(concat!(
            "from: table_row\n",
            "under_section: T\n",
            "assert:\n",
            "  column_choices:\n",
            "    Type: [Unit]\n",
        ));
        let a = undeclared.assert().unwrap().clone();
        let fails = evaluate_assert(&doc, &undeclared, &a, None);
        assert_eq!(fails.len(), 2);
        assert!(fails.iter().all(|f| f.row_id.is_none()));
        assert_eq!(
            fails.iter().map(|f| f.line).collect::<Vec<_>>(),
            vec![Some(4), Some(5)]
        );
    }

    #[trace("TC-1005", "FR-033-AC-16")]
    // #254: the whole-document line a row-scoped assert reports, through the
    // real `validate_document` path with frontmatter present, and it agrees
    // with what the coverage path reports for the same row.
    #[test]
    fn tc1005_row_assert_line_is_the_offending_row_in_the_whole_document() {
        // The shape that exposed the defect: a TestMatrix with frontmatter, so
        // the finding's line is the sum of the frontmatter offset, the section
        // start, the heading line and the 1-based conversion. Every one of
        // those has to be right, and only a whole-document assertion checks
        // them together — the in-crate `evaluate_assert` tests above see body
        // lines and cannot see the frontmatter term at all.
        //
        //  1  ---
        //  2  id: TM-001
        //  3  type: TestMatrix
        //  4  ---
        //  5
        //  6  ## Test Cases
        //  7
        //  8  | ID | Traces To | Type | Status |
        //  9  |----|-----------|------|--------|
        // 10  | TC-001 | FR-001-AC-1 | Demonstration | ✅ |
        //
        // Line 9 is the separator and carries no data; line 10 is the cell the
        // check is about. Reporting 9 is what shipped.
        const MANIFEST: &str = concat!(
            "name: row-line-mod\n",
            "artifact_types:\n",
            "- name: TestMatrix\n",
            "  body_extraction:\n",
            "    yield_pattern:\n",
            "      match:\n",
            "        test_cases:\n",
            "          from: table_row\n",
            "          under_section: Test Cases\n",
            "          required: true\n",
            "          multiple: true\n",
            "          assert:\n",
            "            id_column: ID\n",
            "            column_choices:\n",
            "              Type: [Unit, Integration, Inspection]\n",
        );
        let registry = crate::Registry::from_inline_parts(MANIFEST.as_bytes(), &BTreeMap::new())
            .expect("inline module loads");
        let archetype = registry.archetype("TestMatrix").expect("TestMatrix");

        let doc = concat!(
            "---\nid: TM-001\ntype: TestMatrix\n---\n\n",
            "## Test Cases\n\n",
            "| ID | Traces To | Type | Status |\n",
            "|----|-----------|------|--------|\n",
            "| TC-001 | FR-001-AC-1 | Demonstration | ✅ |\n",
        )
        .to_string();

        let result = crate::validate_document(archetype, &doc);
        let asserts: Vec<_> = result
            .errors
            .iter()
            .filter(|e| e.reason == crate::ValidationReason::Assert)
            .collect();
        assert_eq!(asserts.len(), 1, "{:?}", result.errors);
        assert_eq!(
            asserts[0].line,
            Some(10),
            "the finding must name the row carrying `Demonstration`, not the \
             separator above it: {:?}",
            asserts[0]
        );

        // The control: the same tree with a declared value fires nothing, so
        // the assertion above is about the defect and not about the check
        // being loud.
        let clean = doc.replace("Demonstration", "Inspection");
        assert!(
            crate::validate_document(archetype, &clean)
                .errors
                .iter()
                .all(|e| e.reason != crate::ValidationReason::Assert),
            "a declared Type value must fire no assert"
        );
    }

    #[trace("TC-534", "FR-033-AC-1")]
    // section_body assert level.
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

    #[trace("TC-535", "FR-033-AC-2")]
    // table_row columns exact text + order.
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

    #[trace("TC-536", "FR-033-AC-3")]
    // min_rows / min_items.
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

    #[trace("TC-537", "FR-033-AC-4")]
    // id_column + id_pattern.
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

    #[trace("TC-540", "FR-034-AC-1")]
    // id_pattern interpolation with this doc's id.
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

    #[trace("TC-541", "FR-034-AC-2")]
    // unresolved field → reason unresolved-field.
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

    #[trace("TC-542", "FR-034-AC-3")]
    // regex metacharacters in value are escaped.
    #[test]
    fn tc542_regex_escape() {
        let doc = parse_document("---\nid: A.B+\n---\n## AC\n| ID |\n| - |\n| A.B+ |\n");
        let p = prim(
            "from: table_row\nunder_section: AC\nassert:\n  id_column: ID\n  id_pattern: '^{id}$'",
        );
        let a = p.assert().unwrap().clone();
        assert!(evaluate_assert(&doc, &p, &a, doc.frontmatter.as_ref()).is_empty());
    }

    #[trace("TC-543", "FR-034-AC-4")]
    // static pattern with no token.
    #[test]
    fn tc543_static_pattern() {
        let doc = parse_document("## AC\n| ID |\n| - |\n| AC-1 |\n");
        let p = prim(
            "from: table_row\nunder_section: AC\nassert:\n  id_column: ID\n  id_pattern: '^AC-\\d+$'",
        );
        let a = p.assert().unwrap().clone();
        assert!(evaluate_assert(&doc, &p, &a, None).is_empty());
    }

    #[trace("TC-571", "FR-033-AC-8")]
    // `id_column` resolution precedence on a
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

    #[trace("TC-572", "FR-033-AC-9")]
    // `id_pattern` on non-table locators applies to
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

    #[trace("TC-608", "FR-033-AC-10")]
    // a `section_body` `matches` regex asserts the
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

    #[trace("TC-633", "FR-033-AC-11")]
    // `choices` scalar enum on a `section_body`
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

        // A `frontmatter_field` value (not pre-trimmed by `eval`) proves the
        // "exact match after trim" semantic actually lives in the choices check.
        let fp = prim(
            "from: frontmatter_field\npath: [severity]\nassert:\n  choices: [low, medium, high]",
        );
        let fa = fp.assert().unwrap().clone();
        let fdoc = parse_document("---\nseverity: '  high  '\n---\n# H\n");
        assert!(
            evaluate_assert(&fdoc, &fp, &fa, fdoc.frontmatter.as_ref()).is_empty(),
            "surrounding whitespace must be trimmed before membership"
        );
        let fbad = parse_document("---\nseverity: critical\n---\n# H\n");
        assert_eq!(
            evaluate_assert(&fbad, &fp, &fa, fbad.frontmatter.as_ref()).len(),
            1
        );
    }

    #[trace("TC-634", "FR-033-AC-12")]
    // `column_choices` constrains every cell in a
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

    #[trace("TC-635", "FR-033-AC-13")]
    // `column_patterns` regex-validates every cell in
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

        // `{field}` interpolation in a column pattern (FR-034 parity).
        let interp = prim(
            "from: table_row\nunder_section: Findings\nassert:\n  column_patterns:\n    ID: '^{prefix}-\\d+$'",
        );
        let ainterp = interp.assert().unwrap().clone();
        let idoc = parse_document(
            "---\nprefix: FND\n---\n## Findings\n| ID | Severity |\n| - | - |\n| FND-1 | low |\n",
        );
        assert!(evaluate_assert(&idoc, &interp, &ainterp, idoc.frontmatter.as_ref()).is_empty());
        // A missing `{field}` surfaces reason `unresolved-field`, not `assert`.
        let nofield =
            parse_document("## Findings\n| ID | Severity |\n| - | - |\n| FND-1 | low |\n");
        let ufails = evaluate_assert(&nofield, &interp, &ainterp, nofield.frontmatter.as_ref());
        assert_eq!(ufails.len(), 1);
        assert_eq!(ufails[0].reason, AssertReason::UnresolvedField);
    }
}
