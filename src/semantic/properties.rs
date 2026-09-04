//! Typed `## Properties` extraction to `FieldDecl[]` (FR-070).
//!
//! Form recognition and the cell grammars follow the `agent-ix/quoin`
//! FR-071 mapping; a row error makes the whole kind `unavailable`
//! (`row-errors`) so a consumer never receives a partial type.

use jsonschema::JSONSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::context::SemanticContext;
use super::contract::SemanticSeverity;
use super::decl::{
    is_identifier, Constraint, DecimalPolicy, FieldDecl, Multiplicity, TypeRef, KERNEL_SCALARS,
    UNIT_TARGETS,
};
use super::resolver::compile_module_schema;
use super::scan::{blocks_in, level2_sections, lines, Block, Fence, Table};
use super::{KindAvailability, SemanticDiagnostic};

pub const TYPED_HEADER: [&str; 4] = ["Field", "Type", "Multiplicity", "Constraints"];
pub const PARAM_HEADER: [&str; 4] = ["Param", "Type", "Multiplicity", "Constraints"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FieldsForm {
    Table,
    Fence,
}

/// Result of FR-070 over one document.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldsOutcome {
    pub availability: KindAvailability,
    pub fields: Option<Vec<FieldDecl>>,
    pub form: Option<FieldsForm>,
    /// 1-based line of the first Properties block (table header, fence
    /// opening, or list start), for legacy-form reports.
    pub block_line: Option<usize>,
    pub diagnostics: Vec<SemanticDiagnostic>,
}

impl FieldsOutcome {
    fn unavailable(reason: &str, diagnostics: Vec<SemanticDiagnostic>) -> Self {
        Self {
            availability: KindAvailability::unavailable(reason),
            fields: None,
            form: None,
            block_line: None,
            diagnostics,
        }
    }
}

fn err(code: &str, line: usize, message: impl Into<String>) -> SemanticDiagnostic {
    SemanticDiagnostic::new(code, SemanticSeverity::Error, line, message)
}

fn strip_ticks(cell: &str) -> &str {
    let t = cell.trim();
    t.strip_prefix('`')
        .and_then(|s| s.strip_suffix('`'))
        .unwrap_or(t)
        .trim()
}

/// Extract `## Properties` for a document under `ctx` (FR-070 Behavior).
pub fn extract_fields(raw: &str, ctx: &SemanticContext) -> FieldsOutcome {
    let lines = lines(raw);
    let sections = level2_sections(&lines, "Properties");
    let Some(&(start, end)) = sections.first() else {
        return FieldsOutcome {
            availability: KindAvailability::not_applicable(),
            fields: None,
            form: None,
            block_line: None,
            diagnostics: Vec::new(),
        };
    };
    if sections.len() > 1 {
        let second = sections[1].0;
        return FieldsOutcome::unavailable(
            "duplicate-section",
            vec![err(
                "semantic.duplicate-section",
                second,
                "a second `## Properties` heading; one section per artifact",
            )],
        );
    }
    let blocks = blocks_in(&lines, start + 1, end);
    let mut diagnostics = Vec::new();
    // Form recognition: the first declaration block decides; a second
    // declaration block, or a legacy block after a typed form, is both-forms.
    let mut first: Option<(&Block, bool)> = None; // (block, is typed form)
    for block in &blocks {
        let (typed, relevant, line) = match block {
            Block::Table(t) => (is_typed_header(&t.headers), true, t.line),
            Block::Fence(f) if f.language == "sysml" => (true, true, f.open_line),
            Block::Fence(_) => (false, false, 0),
            Block::List { line } => (false, true, *line),
        };
        if !relevant {
            continue;
        }
        match first {
            None => first = Some((block, typed)),
            Some((_, first_typed)) if first_typed || typed => {
                diagnostics.push(err(
                    "semantic.properties-both-forms",
                    line,
                    "a second Properties form; an artifact carries one typed table or one `sysml` fence",
                ));
                return FieldsOutcome::unavailable("both-forms", diagnostics);
            }
            Some(_) => {} // legacy after legacy: still legacy (first block's form)
        }
    }
    let Some((block, typed)) = first else {
        // A heading with prose only: no declaration block to read.
        diagnostics.push(SemanticDiagnostic::new(
            "semantic.properties-no-block",
            SemanticSeverity::Warning,
            start,
            "`## Properties` holds no typed table, `sysml` fence, or legacy block",
        ));
        return FieldsOutcome::unavailable("no-block", diagnostics);
    };
    let block_line = Some(match block {
        Block::Table(t) => t.line,
        Block::Fence(f) => f.open_line,
        Block::List { line } => *line,
    });
    if !typed {
        let (form, line) = match block {
            Block::Table(t) => ("free-column-table", t.line),
            Block::List { line } => ("bullet-list", *line),
            Block::Fence(_) => unreachable!("non-sysml fences are not relevant"),
        };
        let severity = if ctx.module.legacy_forms == "error" {
            SemanticSeverity::Error
        } else {
            SemanticSeverity::Warning
        };
        let mut d = SemanticDiagnostic::new(
            "semantic.legacy-properties-form",
            severity,
            line,
            format!("Properties authored as {form}; migrate to the typed table (migration: typed-table)"),
        );
        d.reason = Some(form.to_string());
        diagnostics.push(d);
        let mut out = FieldsOutcome::unavailable("legacy-form", diagnostics);
        out.block_line = block_line;
        return out;
    }
    let (rows, form): (Vec<RowInput>, FieldsForm) = match block {
        Block::Table(t) => (table_rows(t), FieldsForm::Table),
        Block::Fence(f) => (fence_rows(f, &mut diagnostics), FieldsForm::Fence),
        Block::List { .. } => unreachable!(),
    };
    let mut fields: Vec<FieldDecl> = Vec::new();
    let mut field_lines: Vec<usize> = Vec::new();
    let mut lossy = false;
    for row in rows {
        if let Some((field, row_lossy)) = map_row(&row, ctx, &mut diagnostics) {
            lossy |= row_lossy;
            fields.push(field);
            field_lines.push(row.line);
        }
    }
    reader_rules(&fields, &field_lines, &mut diagnostics);
    validate_decls(&fields, ctx, &mut diagnostics);
    if diagnostics.iter().any(SemanticDiagnostic::is_error) {
        let loci: Vec<String> = diagnostics
            .iter()
            .filter(|d| d.is_error())
            .filter_map(|d| d.line.map(|l| l.to_string()))
            .collect();
        return FieldsOutcome {
            availability: KindAvailability::unavailable(format!(
                "row-errors: lines {}",
                loci.join(", ")
            )),
            fields: None,
            form: Some(form),
            block_line,
            diagnostics,
        };
    }
    FieldsOutcome {
        availability: KindAvailability::available(lossy),
        fields: Some(fields),
        form: Some(form),
        block_line,
        diagnostics,
    }
}

fn is_typed_header(headers: &[String]) -> bool {
    headers.len() == 4
        && headers
            .iter()
            .zip(TYPED_HEADER.iter())
            .all(|(h, t)| strip_ticks(h) == *t)
}

pub fn is_param_header(headers: &[String]) -> bool {
    headers.len() == 4
        && headers
            .iter()
            .zip(PARAM_HEADER.iter())
            .all(|(h, t)| strip_ticks(h) == *t)
}

/// One field row, from a table row or a fence line.
#[derive(Debug, Clone)]
pub struct RowInput {
    pub line: usize,
    pub name: String,
    pub type_cell: String,
    pub mult_cell: String,
    pub constraints_cell: String,
    /// `ref item` in the fence form: the target must be an object or import.
    pub reference_only: bool,
}

pub fn table_rows(table: &Table) -> Vec<RowInput> {
    table
        .rows
        .iter()
        .map(|(line, cells)| RowInput {
            line: *line,
            name: strip_ticks(cells.first().map(String::as_str).unwrap_or("")).to_string(),
            type_cell: strip_ticks(cells.get(1).map(String::as_str).unwrap_or("")).to_string(),
            mult_cell: strip_ticks(cells.get(2).map(String::as_str).unwrap_or("")).to_string(),
            constraints_cell: strip_ticks(cells.get(3).map(String::as_str).unwrap_or(""))
                .to_string(),
            reference_only: false,
        })
        .collect()
}

/// `attribute <name> : <Type>[<mult>] { … }` / `ref item <name> : <Type>[<mult>] { … }`.
pub fn fence_rows(fence: &Fence, diagnostics: &mut Vec<SemanticDiagnostic>) -> Vec<RowInput> {
    let mut rows = Vec::new();
    if fence.close_line.is_none() {
        diagnostics.push(err(
            "semantic.clause-fence-unterminated",
            fence.open_line,
            "the `sysml` fence is never closed",
        ));
        return rows;
    }
    for (offset, raw_line) in fence.body.split('\n').enumerate() {
        let line = fence.open_line + 1 + offset;
        let text = raw_line.trim_end_matches('\r').trim();
        if text.is_empty() {
            continue;
        }
        let (reference_only, rest) = if let Some(r) = text.strip_prefix("attribute ") {
            (false, r)
        } else if let Some(r) = text.strip_prefix("ref item ") {
            (true, r)
        } else {
            diagnostics.push(err(
                "semantic.sysml-outside-subset",
                line,
                format!("fence line outside the attribute / ref item subset: {text}"),
            ));
            continue;
        };
        let rest = rest.trim_end_matches(';').trim();
        // name : rest
        let Some((name, after)) = rest.split_once(':') else {
            diagnostics.push(err(
                "semantic.sysml-outside-subset",
                line,
                format!("fence line lacks `:`: {text}"),
            ));
            continue;
        };
        let after = after.trim();
        // <Type>( [unit])?[<mult>], then an optional `{ … }` whose content is
        // opaque constraint text (nested braces included).
        let Some(mult_open) = after.find('[').and_then(|first| {
            // the multiplicity bracket is the last `[` before any `{`
            let limit = after.find('{').unwrap_or(after.len());
            after[..limit].rfind('[').or(Some(first))
        }) else {
            diagnostics.push(err(
                "semantic.sysml-outside-subset",
                line,
                format!("fence line lacks `[multiplicity]`: {text}"),
            ));
            continue;
        };
        let Some(mult_close) = after[mult_open..].find(']') else {
            diagnostics.push(err(
                "semantic.sysml-outside-subset",
                line,
                format!("fence line lacks `]`: {text}"),
            ));
            continue;
        };
        let mult_cell = after[mult_open + 1..mult_open + mult_close]
            .trim()
            .to_string();
        let tail = after[mult_open + mult_close + 1..].trim();
        let braces = if tail.is_empty() {
            String::new()
        } else if let Some(inner) = tail.strip_prefix('{').and_then(|t| t.strip_suffix('}')) {
            inner.trim().to_string()
        } else {
            diagnostics.push(err(
                "semantic.sysml-outside-subset",
                line,
                format!("trailing text after the multiplicity: {text}"),
            ));
            continue;
        };
        let typed = after;
        let type_cell = typed[..mult_open].trim().to_string();
        rows.push(RowInput {
            line,
            name: name.trim().to_string(),
            type_cell,
            mult_cell,
            constraints_cell: braces,
            reference_only,
        });
    }
    rows
}

/// Map one row to a `FieldDecl`; `None` when the row carries an error.
/// The bool reports lossiness (an unresolved placeholder).
pub fn map_row(
    row: &RowInput,
    ctx: &SemanticContext,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> Option<(FieldDecl, bool)> {
    let before = diagnostics.len();
    if !is_identifier(&row.name) {
        diagnostics.push(err(
            "semantic.invalid-field-name",
            row.line,
            format!("field name {:?} is not an Identifier", row.name),
        ));
    }
    let (type_ref, lossy) = map_type(&row.type_cell, row, ctx, diagnostics);
    let multiplicity = map_multiplicity(&row.mult_cell, row.line, diagnostics);
    let (constraints, identity, nullable) =
        map_constraints(&row.constraints_cell, row.line, diagnostics);
    if diagnostics.len() > before
        && diagnostics[before..]
            .iter()
            .any(SemanticDiagnostic::is_error)
    {
        return None;
    }
    let mut type_ref = type_ref?;
    type_ref.multiplicity = Some(multiplicity?);
    Some((
        FieldDecl {
            name: row.name.clone(),
            type_ref,
            identity: identity.then_some(true),
            nullable: nullable.then_some(true),
            doc: None,
            constraints: if constraints.is_empty() {
                None
            } else {
                Some(constraints)
            },
        },
        lossy,
    ))
}

/// Type cell → `TypeRef` (without multiplicity). Precedence per FR-070.
pub(crate) fn map_type(
    cell: &str,
    row: &RowInput,
    ctx: &SemanticContext,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> (Option<TypeRef>, bool) {
    let line = row.line;
    let mut text = cell.trim().to_string();
    // ` [unit]` suffix
    let mut unit = None;
    if text.ends_with(']') {
        if let Some(open) = text.rfind('[') {
            let candidate = text[open + 1..text.len() - 1].trim().to_string();
            let head = text[..open].trim().to_string();
            let head_ok = !head.is_empty() && (!head.contains('(') || head.ends_with(')'));
            if candidate.is_empty() {
                diagnostics.push(err(
                    "semantic.invalid-type-token",
                    line,
                    format!("type {cell:?}: empty unit brackets"),
                ));
                return (None, false);
            }
            if head_ok {
                unit = Some(candidate);
                text = head;
            }
        }
    }
    // Decimal(p,s)
    let mut decimal = None;
    let mut token = text.clone();
    if let Some(open) = text.find('(') {
        if text.ends_with(')') {
            let inner = &text[open + 1..text.len() - 1];
            token = text[..open].trim().to_string();
            let parts: Vec<&str> = inner.split(',').map(str::trim).collect();
            match (
                parts.first().and_then(|p| p.parse::<u64>().ok()),
                parts.get(1).and_then(|s| s.parse::<u64>().ok()),
            ) {
                (Some(precision), Some(scale))
                    if parts.len() == 2 && token == "Decimal" && precision >= 1 =>
                {
                    decimal = Some(DecimalPolicy { precision, scale });
                }
                _ => {
                    diagnostics.push(err(
                        "semantic.invalid-type-token",
                        line,
                        format!("type {cell:?}: only Decimal(precision,scale) takes parameters"),
                    ));
                    return (None, false);
                }
            }
        }
    }
    if !is_identifier(&token) {
        diagnostics.push(err(
            "semantic.invalid-type-token",
            line,
            format!("type token {token:?} is not an Identifier"),
        ));
        return (None, false);
    }
    if KERNEL_SCALARS.contains(&token.as_str()) {
        if token == "Decimal" && decimal.is_none() {
            diagnostics.push(err(
                "agent-ix.semantic-core.MISSING_DECIMAL_POLICY",
                line,
                "bare `Decimal` needs a policy: Decimal(precision,scale)",
            ));
            return (None, false);
        }
        if unit.is_some() && !UNIT_TARGETS.contains(&token.as_str()) {
            diagnostics.push(err(
                "agent-ix.semantic-core.UNIT_NOT_ALLOWED",
                line,
                format!("a unit applies to Integer, Decimal, or Duration, not {token}"),
            ));
            return (None, false);
        }
        if row.reference_only {
            diagnostics.push(err(
                "semantic.sysml-outside-subset",
                line,
                format!("`ref item` targets an object or import, not the kernel scalar {token}"),
            ));
            return (None, false);
        }
        return (
            Some(TypeRef {
                target: token,
                multiplicity: None,
                unit,
                decimal,
            }),
            false,
        );
    }
    if decimal.is_some() {
        diagnostics.push(err(
            "agent-ix.semantic-core.DECIMAL_ON_NON_DECIMAL",
            line,
            format!("Decimal(p,s) on the non-Decimal target {token}"),
        ));
        return (None, false);
    }
    if unit.is_some() {
        diagnostics.push(err(
            "agent-ix.semantic-core.UNIT_NOT_ALLOWED",
            line,
            format!("a unit applies to Integer, Decimal, or Duration, not {token}"),
        ));
        return (None, false);
    }
    let package = ctx.identity_package();
    let (org, repo) = package.split_once('/').unwrap_or((package, ""));
    let type_identity = |name: &str| format!("ix://{org}/{repo}/type/{name}");
    let bundle = &ctx.bundle;
    // 2. object by id
    if bundle.objects.iter().any(|o| o.id == token) {
        return (
            Some(TypeRef {
                target: type_identity(&token),
                multiplicity: None,
                unit: None,
                decimal: None,
            }),
            false,
        );
    }
    // 3. object by name; two matches are ambiguous
    let by_name: Vec<&str> = bundle
        .objects
        .iter()
        .filter(|o| o.names.iter().any(|n| n == &token))
        .map(|o| o.id.as_str())
        .collect();
    if by_name.len() > 1 {
        diagnostics.push(err(
            "semantic.ambiguous-type",
            line,
            format!("type {token:?} names {} and {}", by_name[0], by_name[1]),
        ));
        return (None, false);
    }
    if by_name.len() == 1 {
        return (
            Some(TypeRef {
                target: type_identity(&token),
                multiplicity: None,
                unit: None,
                decimal: None,
            }),
            false,
        );
    }
    // 4. enumeration by id or name
    if bundle
        .enumerations
        .iter()
        .any(|e| e.id == token || e.names.iter().any(|n| n == &token))
    {
        return (
            Some(TypeRef {
                target: type_identity(&token),
                multiplicity: None,
                unit: None,
                decimal: None,
            }),
            false,
        );
    }
    // 5. import
    for (package, exports) in &bundle.imports {
        if exports.iter().any(|e| e == &token) {
            let (o, r) = package.split_once('/').unwrap_or((package, ""));
            return (
                Some(TypeRef {
                    target: format!("ix://{o}/{r}/type/{token}"),
                    multiplicity: None,
                    unit: None,
                    decimal: None,
                }),
                false,
            );
        }
    }
    // Unresolved: placeholder + advisory with the reason.
    let reason = if bundle.is_empty() {
        "no-bundle-index"
    } else if ctx
        .module
        .imports
        .keys()
        .any(|p| !bundle.imports.contains_key(p))
    {
        "import-unresolved"
    } else {
        "unknown-token"
    };
    diagnostics.push(
        SemanticDiagnostic::new(
            "semantic.unresolved-type",
            SemanticSeverity::Advisory,
            line,
            format!("type {token:?} resolves to no kernel scalar, bundle declaration, or import ({reason})"),
        )
        .with_reason(reason),
    );
    (
        Some(TypeRef {
            target: format!("ix://{org}/{repo}/unresolved/{token}"),
            multiplicity: None,
            unit: None,
            decimal: None,
        }),
        true,
    )
}

pub(crate) fn map_multiplicity(
    cell: &str,
    line: usize,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> Option<Multiplicity> {
    let text = cell.trim();
    if text.is_empty() {
        return Some(Multiplicity::one());
    }
    let mut parts = text.split_whitespace();
    let bounds = parts.next().unwrap_or("");
    let flags: Vec<&str> = parts.collect();
    let invalid = |diagnostics: &mut Vec<SemanticDiagnostic>, why: &str| {
        diagnostics.push(err(
            "semantic.invalid-multiplicity",
            line,
            format!("multiplicity {text:?}: {why}"),
        ));
        None
    };
    let (lower, upper) = if bounds == "*" {
        (0, None)
    } else if let Some((lo, hi)) = bounds.split_once("..") {
        let Ok(lo) = lo.parse::<u64>() else {
            return invalid(diagnostics, "lower bound is not an integer");
        };
        if hi == "*" {
            (lo, None)
        } else {
            let Ok(hi) = hi.parse::<u64>() else {
                return invalid(diagnostics, "upper bound is not an integer or *");
            };
            if lo > hi {
                return invalid(diagnostics, "lower bound exceeds upper bound");
            }
            (lo, Some(hi))
        }
    } else {
        let Ok(n) = bounds.parse::<u64>() else {
            return invalid(diagnostics, "not an integer, n..m, n..*, or *");
        };
        (n, Some(n))
    };
    let mut m = Multiplicity {
        lower,
        upper,
        ordered: None,
        unique: None,
    };
    for flag in flags {
        match flag {
            "ordered" => m.ordered = Some(true),
            "unique" => m.unique = Some(true),
            other => return invalid(diagnostics, &format!("unknown flag {other:?}")),
        }
    }
    if (m.ordered.is_some() || m.unique.is_some()) && !m.is_collection() {
        return invalid(
            diagnostics,
            "ordered/unique apply only when the upper bound is absent or greater than 1",
        );
    }
    Some(m)
}

/// Split a constraints cell on commas outside a `/…/` pattern.
fn split_constraints(cell: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut current = String::new();
    let mut in_pattern = false;
    let mut chars = cell.chars().peekable();
    while let Some(c) = chars.next() {
        if in_pattern {
            current.push(c);
            if c == '/' {
                // the pattern runs to the last `/` of the item: stay in pattern
                // while another `/` follows before a comma
                let rest: String = chars.clone().collect();
                let next_comma = rest.find(',').unwrap_or(rest.len());
                if !rest[..next_comma].contains('/') {
                    in_pattern = false;
                }
            }
            continue;
        }
        if c == ',' {
            items.push(current.trim().to_string());
            current.clear();
            continue;
        }
        current.push(c);
        if c == '/' && current.trim_start().starts_with("pattern") {
            in_pattern = true;
        }
    }
    if !current.trim().is_empty() {
        items.push(current.trim().to_string());
    }
    items
}

/// A numeric literal or a string; `NaN`/`inf` are neither (they cannot be
/// carried as JSON numbers) and empty text has no value.
fn number_or_string(text: &str) -> Option<Value> {
    if let Ok(i) = text.parse::<i64>() {
        return Some(json!(i));
    }
    if let Ok(f) = text.parse::<f64>() {
        return if f.is_finite() { Some(json!(f)) } else { None };
    }
    if text.is_empty() {
        return None;
    }
    Some(json!(text))
}

fn map_constraints(
    cell: &str,
    line: usize,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> (Vec<Constraint>, bool, bool) {
    let mut out = Vec::new();
    let mut identity = false;
    let mut nullable = false;
    for item in split_constraints(cell) {
        let (keyword, value) = match item.split_once(':') {
            Some((k, v)) if !k.trim().is_empty() => (k.trim(), Some(v.trim())),
            _ => (item.trim(), None),
        };
        let unknown = |diagnostics: &mut Vec<SemanticDiagnostic>| {
            diagnostics.push(err(
                "semantic.unknown-constraint-keyword",
                line,
                format!("constraint {item:?} uses a keyword outside the closed set"),
            ));
        };
        let bad_value = |diagnostics: &mut Vec<SemanticDiagnostic>| {
            diagnostics.push(err(
                "semantic.invalid-constraint-value",
                line,
                format!("constraint {item:?} has no usable value"),
            ));
        };
        match (keyword, value) {
            ("identity", None) => identity = true,
            ("nullable", None) => nullable = true,
            ("nonEmpty", None) => out.push(Constraint::NonEmpty),
            ("unique", None) => out.push(Constraint::Unique),
            ("min" | "max" | "exclusiveMin" | "exclusiveMax", Some(v)) => {
                match number_or_string(v) {
                    Some(value) => out.push(match keyword {
                        "min" => Constraint::Min { value },
                        "max" => Constraint::Max { value },
                        "exclusiveMin" => Constraint::ExclusiveMin { value },
                        _ => Constraint::ExclusiveMax { value },
                    }),
                    None => bad_value(diagnostics),
                }
            }
            ("minLength", Some(v)) => match v.parse::<u64>() {
                Ok(n) => out.push(Constraint::MinLength { value: n }),
                Err(_) => unknown(diagnostics),
            },
            ("maxLength", Some(v)) => match v.parse::<u64>() {
                Ok(n) => out.push(Constraint::MaxLength { value: n }),
                Err(_) => unknown(diagnostics),
            },
            ("pattern", Some(v)) => {
                let v = v.trim();
                match (v.strip_prefix('/'), v.rfind('/')) {
                    (Some(_), Some(last)) if last > 0 => out.push(Constraint::Pattern {
                        regex: v[1..last].to_string(),
                        dialect: "ecma-262".to_string(),
                    }),
                    _ => unknown(diagnostics),
                }
            }
            ("enumValues", Some(v)) => {
                let values: Vec<Option<Value>> =
                    v.split('|').map(|s| number_or_string(s.trim())).collect();
                if v.trim().is_empty() || v.contains(' ') || values.iter().any(Option::is_none) {
                    bad_value(diagnostics);
                } else {
                    out.push(Constraint::EnumValues {
                        values: values.into_iter().flatten().collect(),
                    });
                }
            }
            ("format", Some(v)) => {
                if v.contains(':') && !v.contains(' ') {
                    out.push(Constraint::Format {
                        name: v.to_string(),
                    });
                } else {
                    unknown(diagnostics);
                }
            }
            _ => unknown(diagnostics),
        }
    }
    (out, identity, nullable)
}

/// Semantic-core reader rules carried into extraction (FR-070 Behavior).
fn reader_rules(fields: &[FieldDecl], lines: &[usize], diagnostics: &mut Vec<SemanticDiagnostic>) {
    // `lines[i]` is the source line of `fields[i]`.
    let line_of = |i: usize| lines.get(i).copied().unwrap_or(0);
    let mut seen: Vec<&str> = Vec::new();
    for (i, field) in fields.iter().enumerate() {
        if seen.contains(&field.name.as_str()) {
            diagnostics.push(err(
                "agent-ix.semantic-core.DUPLICATE_NAME",
                line_of(i),
                format!("field {} is declared twice", field.name),
            ));
        }
        seen.push(&field.name);
        let m = field.type_ref.multiplicity.clone().unwrap_or_default();
        if field.identity == Some(true) {
            if !m.is_single() {
                diagnostics.push(err(
                    "agent-ix.semantic-core.IDENTITY_NOT_SINGLE",
                    line_of(i),
                    format!("identity field {} must be 1..1", field.name),
                ));
            }
            if field.type_ref.target == "JsonObject" {
                diagnostics.push(err(
                    "agent-ix.semantic-core.IDENTITY_ON_JSON_OBJECT",
                    line_of(i),
                    format!("identity field {} cannot be a JsonObject", field.name),
                ));
            }
        }
    }
}

/// Every produced entry validates against the vendored `FieldDecl.json`; a
/// failure is an engine defect, reported and never dropped.
fn validate_decls(
    fields: &[FieldDecl],
    ctx: &SemanticContext,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) {
    if fields.is_empty() {
        return;
    }
    let Some(validator) = field_decl_validator(&ctx.module.semantic_core) else {
        diagnostics.push(err(
            "semantic.unsupported-semantic-core",
            0,
            format!(
                "no vendored semantic-core {} bundle to validate FieldDecl against",
                ctx.module.semantic_core
            ),
        ));
        return;
    };
    for field in fields {
        let value = serde_json::to_value(field).unwrap_or(Value::Null);
        if !validator.is_valid(&value) {
            diagnostics.push(err(
                "semantic.internal-invalid-decl",
                0,
                format!("produced FieldDecl for {} does not validate against FieldDecl.json (engine defect)", field.name),
            ));
        }
    }
}

pub(crate) fn field_decl_validator(semantic_core: &str) -> Option<JSONSchema> {
    let schema = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": format!("https://schemas.agent-ix.org/agent-ix/quire-rs/{semantic_core}/FieldDeclGate.json"),
        "$ref": format!("https://schemas.agent-ix.org/semantic-core/{semantic_core}/FieldDecl.json")
    });
    compile_module_schema(
        &schema,
        &|_| None,
        semantic_core,
        "https://schemas.agent-ix.org/agent-ix/quire-rs/",
    )
    .ok()
}
