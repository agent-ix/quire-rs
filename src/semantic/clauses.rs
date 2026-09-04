//! `## Invariants` → `ClauseRef[]` + verbatim `clauseText`, and
//! `## Operations` → `OperationDecl[]` (FR-071). Fence bodies are copied,
//! never parsed; spans come from the shared line-level scanner.

use std::collections::BTreeMap;

use jsonschema::JSONSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::context::SemanticContext;
use super::contract::SemanticSeverity;
use super::decl::{is_identifier, FieldDecl, TypeRef};
use super::properties::{
    is_param_header, map_multiplicity, map_row, map_type, table_rows, RowInput,
};
use super::resolver::compile_module_schema;
use super::scan::{blocks_in, level2_sections, lines, Block, Fence};
use super::{KindAvailability, SemanticDiagnostic};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceLocus {
    pub source_identity: String,
    pub path: String,
    pub start_line: usize,
    pub start_column: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_column: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClauseRef {
    pub language: String,
    pub clause_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_span: Option<SourceLocus>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperationDecl {
    pub name: String,
    pub params: Vec<FieldDecl>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub returns: Option<TypeRef>,
    pub pre: Vec<ClauseRef>,
    pub post: Vec<ClauseRef>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClausesOutcome {
    pub availability: KindAvailability,
    pub clauses: Option<Vec<ClauseRef>>,
    pub clause_text: BTreeMap<String, String>,
    pub diagnostics: Vec<SemanticDiagnostic>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OperationsOutcome {
    pub availability: KindAvailability,
    pub operations: Option<Vec<OperationDecl>>,
    pub diagnostics: Vec<SemanticDiagnostic>,
}

fn err(code: &str, line: usize, message: impl Into<String>) -> SemanticDiagnostic {
    SemanticDiagnostic::new(code, SemanticSeverity::Error, line, message)
}

fn advisory(code: &str, line: usize, message: impl Into<String>) -> SemanticDiagnostic {
    SemanticDiagnostic::new(code, SemanticSeverity::Advisory, line, message)
}

/// Is `tag` a semantic-core `ClauseLanguage`?
pub fn clause_language_class(tag: &str) -> ClauseLanguageClass {
    if tag.is_empty() {
        return ClauseLanguageClass::Missing;
    }
    if tag == "ocl" {
        return ClauseLanguageClass::Checked;
    }
    if tag == "sysml" || tag == "fretish" {
        return ClauseLanguageClass::Unchecked;
    }
    // `<ns>:<name>` with ns `[a-z0-9][a-z0-9.-]*`, name `[A-Za-z0-9][A-Za-z0-9._-]*`
    if let Some((ns, name)) = tag.split_once(':') {
        let ns_ok = ns
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
            && ns
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '-');
        let name_ok = name
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphanumeric())
            && name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-');
        if ns_ok && name_ok {
            return ClauseLanguageClass::Unchecked;
        }
    }
    ClauseLanguageClass::Invalid
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClauseLanguageClass {
    Missing,
    Invalid,
    Checked,
    Unchecked,
}

/// One `### <id>` clause heading and what it owns.
struct ClauseSection {
    heading_line: usize,
    id: String,
    end: usize,
}

fn level3_headings(lines: &[&str], from: usize, to: usize) -> Vec<ClauseSection> {
    let fences = super::scan::fences_in(lines, from, to);
    let inside_fence = |l: usize| {
        fences
            .iter()
            .any(|f| l > f.open_line && f.close_line.map_or(true, |c| l < c))
    };
    let mut out: Vec<ClauseSection> = Vec::new();
    for i in from..to.min(lines.len() + 1) {
        let text = lines[i - 1].trim_end_matches('\r');
        if inside_fence(i) {
            continue;
        }
        if let Some(rest) = text.strip_prefix("### ") {
            if let Some(prev) = out.last_mut() {
                prev.end = i;
            }
            out.push(ClauseSection {
                heading_line: i,
                id: rest.trim().to_string(),
                end: to,
            });
        }
    }
    out
}

fn span(ctx: &SemanticContext, identity: &str, fence: &Fence) -> SourceLocus {
    SourceLocus {
        source_identity: identity.to_string(),
        path: ctx.path.clone(),
        start_line: fence.open_line,
        start_column: 1,
        end_line: fence.close_line,
        end_column: fence.close_line.map(|_| fence.close_len + 1),
    }
}

/// FR-071 Invariants.
pub fn extract_clauses(raw: &str, ctx: &SemanticContext) -> ClausesOutcome {
    let lines = lines(raw);
    let sections = level2_sections(&lines, "Invariants");
    let Some(&(start, end)) = sections.first() else {
        return ClausesOutcome {
            availability: KindAvailability::not_applicable(),
            clauses: None,
            clause_text: BTreeMap::new(),
            diagnostics: Vec::new(),
        };
    };
    let mut diagnostics = Vec::new();
    if sections.len() > 1 {
        diagnostics.push(err(
            "semantic.duplicate-section",
            sections[1].0,
            "a second `## Invariants` heading",
        ));
        return ClausesOutcome {
            availability: KindAvailability::unavailable("duplicate-section"),
            clauses: None,
            clause_text: BTreeMap::new(),
            diagnostics,
        };
    }
    let (identity, defaulted) = ctx.resolved_source_identity();
    let headings = level3_headings(&lines, start + 1, end);
    // Fences before the first heading own no clause.
    let first_heading = headings.first().map(|h| h.heading_line).unwrap_or(end);
    for fence in super::scan::fences_in(&lines, start + 1, first_heading) {
        diagnostics.push(err(
            "semantic.clause-without-id",
            fence.open_line,
            "a fence under `## Invariants` outside any `### <clauseId>` heading",
        ));
    }
    let mut clauses: Vec<ClauseRef> = Vec::new();
    let mut clause_text: BTreeMap<String, String> = BTreeMap::new();
    let mut seen: Vec<String> = Vec::new();
    let mut lossy = false;
    let mut any_span = false;
    for section in &headings {
        let id = &section.id;
        if !is_identifier(id) {
            diagnostics.push(err(
                "semantic.clause-id-not-identifier",
                section.heading_line,
                format!("clause heading {id:?} is not an Identifier"),
            ));
            continue;
        }
        let fences = super::scan::fences_in(&lines, section.heading_line + 1, section.end);
        let external: Vec<usize> = (section.heading_line + 1..section.end.min(lines.len() + 1))
            .filter(|&l| {
                lines[l - 1]
                    .trim_end_matches('\r')
                    .trim_start()
                    .starts_with("Clause:")
            })
            .collect();
        if seen.contains(id) {
            diagnostics.push(err(
                "semantic.duplicate-clause-id",
                section.heading_line,
                format!("clause {id} is declared twice"),
            ));
            continue;
        }
        seen.push(id.clone());
        if let (Some(fence), Some(&ext)) = (fences.first(), external.first()) {
            let second = fence.open_line.max(ext);
            diagnostics.push(err(
                "semantic.duplicate-clause-authority",
                second,
                format!("clause {id} is declared by a fence and by a `Clause:` reference"),
            ));
            continue;
        }
        if fences.is_empty() {
            match external.first() {
                Some(&line) => {
                    diagnostics.push(advisory("semantic.clause-external-unsupported", line, format!("clause {id} is declared by an external `Clause:` reference; the file is not read")));
                }
                None => diagnostics.push(err(
                    "semantic.clause-missing-body",
                    section.heading_line,
                    format!("clause {id} has no fence"),
                )),
            }
            continue;
        }
        if fences.len() > 1 {
            diagnostics.push(err(
                "semantic.clause-multiple-bodies",
                fences[1].open_line,
                format!("clause {id} owns more than one fence"),
            ));
            continue;
        }
        let fence = &fences[0];
        if fence.close_line.is_none() {
            diagnostics.push(err(
                "semantic.clause-fence-unterminated",
                fence.open_line,
                format!("clause {id}: fence is never closed"),
            ));
            continue;
        }
        match clause_language_class(&fence.language) {
            ClauseLanguageClass::Missing => {
                diagnostics.push(err(
                    "semantic.clause-language-missing",
                    fence.open_line,
                    format!("clause {id}: fence has no language tag"),
                ));
                continue;
            }
            ClauseLanguageClass::Invalid => {
                diagnostics.push(err(
                    "semantic.clause-language-invalid",
                    fence.open_line,
                    format!(
                        "clause {id}: language {:?} is outside the ClauseLanguage pattern",
                        fence.language
                    ),
                ));
                continue;
            }
            ClauseLanguageClass::Unchecked => {
                diagnostics.push(advisory(
                    "semantic.clause-language-unchecked",
                    fence.open_line,
                    format!(
                        "clause {id}: language {} is carried unchecked",
                        fence.language
                    ),
                ));
                lossy = true;
            }
            ClauseLanguageClass::Checked => {}
        }
        any_span = true;
        clauses.push(ClauseRef {
            language: fence.language.clone(),
            clause_id: id.clone(),
            source_span: Some(span(ctx, &identity, fence)),
        });
        clause_text.insert(id.clone(), fence.body.clone());
    }
    if defaulted && any_span {
        diagnostics.push(advisory(
            "semantic.source-identity-defaulted",
            start,
            format!("no source identity supplied; spans carry {identity}"),
        ));
    }
    validate_decls(
        &clauses,
        "ClauseRef",
        &ctx.module.semantic_core,
        &mut diagnostics,
    );
    if diagnostics.iter().any(SemanticDiagnostic::is_error) {
        let loci: Vec<String> = diagnostics
            .iter()
            .filter(|d| d.is_error())
            .filter_map(|d| d.line.map(|l| l.to_string()))
            .collect();
        return ClausesOutcome {
            availability: KindAvailability::unavailable(format!(
                "entry-errors: lines {}",
                loci.join(", ")
            )),
            clauses: None,
            clause_text: BTreeMap::new(),
            diagnostics,
        };
    }
    ClausesOutcome {
        availability: KindAvailability::available(lossy),
        clauses: Some(clauses),
        clause_text,
        diagnostics,
    }
}

/// FR-071 Operations. `clauses` are the artifact's extracted invariants.
pub fn extract_operations(
    raw: &str,
    ctx: &SemanticContext,
    clauses: &[ClauseRef],
) -> OperationsOutcome {
    let lines = lines(raw);
    let sections = level2_sections(&lines, "Operations");
    let Some(&(start, end)) = sections.first() else {
        return OperationsOutcome {
            availability: KindAvailability::not_applicable(),
            operations: None,
            diagnostics: Vec::new(),
        };
    };
    let mut diagnostics = Vec::new();
    if sections.len() > 1 {
        diagnostics.push(err(
            "semantic.duplicate-section",
            sections[1].0,
            "a second `## Operations` heading",
        ));
        return OperationsOutcome {
            availability: KindAvailability::unavailable("duplicate-section"),
            operations: None,
            diagnostics,
        };
    }
    let mut operations: Vec<OperationDecl> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    let mut lossy = false;
    for section in level3_headings(&lines, start + 1, end) {
        let name = section.id.clone();
        if !is_identifier(&name) {
            diagnostics.push(err(
                "semantic.operation-name-not-identifier",
                section.heading_line,
                format!("operation heading {name:?} is not an Identifier"),
            ));
            continue;
        }
        if seen.contains(&name) {
            diagnostics.push(err(
                "semantic.duplicate-operation",
                section.heading_line,
                format!("operation {name} is declared twice"),
            ));
            continue;
        }
        seen.push(name.clone());
        let before = diagnostics.len();
        // Parameter table.
        let mut params: Vec<FieldDecl> = Vec::new();
        for block in blocks_in(&lines, section.heading_line + 1, section.end) {
            if let Block::Table(table) = block {
                if !is_param_header(&table.headers) {
                    diagnostics.push(err("semantic.invalid-operation-table", table.line, format!("operation {name}: a table under an operation carries the header `Param | Type | Multiplicity | Constraints`")));
                    continue;
                }
                for row in table_rows(&table) {
                    if let Some((field, row_lossy)) = map_row(&row, ctx, &mut diagnostics) {
                        lossy |= row_lossy;
                        params.push(field);
                    }
                }
            }
        }
        // Returns / Pre / Post lines.
        let mut returns = None;
        let mut pre = Vec::new();
        let mut post = Vec::new();
        for l in section.heading_line + 1..section.end.min(lines.len() + 1) {
            let text = lines[l - 1].trim_end_matches('\r').trim();
            if let Some(rest) = text.strip_prefix("Returns:") {
                returns = parse_returns(rest.trim(), l, &name, ctx, &mut diagnostics);
            } else if let Some(rest) = text.strip_prefix("Pre:") {
                pre = resolve_refs(rest, l, clauses, &mut diagnostics);
            } else if let Some(rest) = text.strip_prefix("Post:") {
                post = resolve_refs(rest, l, clauses, &mut diagnostics);
            }
        }
        if diagnostics[before..]
            .iter()
            .any(SemanticDiagnostic::is_error)
        {
            continue;
        }
        operations.push(OperationDecl {
            name,
            params,
            returns,
            pre,
            post,
        });
    }
    validate_decls(
        &operations,
        "OperationDecl",
        &ctx.module.semantic_core,
        &mut diagnostics,
    );
    if diagnostics.iter().any(SemanticDiagnostic::is_error) {
        let loci: Vec<String> = diagnostics
            .iter()
            .filter(|d| d.is_error())
            .filter_map(|d| d.line.map(|l| l.to_string()))
            .collect();
        return OperationsOutcome {
            availability: KindAvailability::unavailable(format!(
                "entry-errors: lines {}",
                loci.join(", ")
            )),
            operations: None,
            diagnostics,
        };
    }
    OperationsOutcome {
        availability: KindAvailability::available(lossy),
        operations: Some(operations),
        diagnostics,
    }
}

fn parse_returns(
    text: &str,
    line: usize,
    op: &str,
    ctx: &SemanticContext,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> Option<TypeRef> {
    let Some(open) = text.rfind('[') else {
        diagnostics.push(err(
            "semantic.invalid-returns",
            line,
            format!("operation {op}: Returns needs `<Type>[<multiplicity>]`"),
        ));
        return None;
    };
    let Some(close) = text[open..].find(']') else {
        diagnostics.push(err(
            "semantic.invalid-returns",
            line,
            format!("operation {op}: Returns lacks `]`"),
        ));
        return None;
    };
    let type_cell = text[..open].trim();
    let mult_cell = text[open + 1..open + close].trim();
    if type_cell.contains('[') {
        diagnostics.push(err(
            "agent-ix.semantic-core.UNIT_ON_RETURNS",
            line,
            format!("operation {op}: a unit is not allowed on Returns"),
        ));
        return None;
    }
    let row = RowInput {
        line,
        name: "returns".into(),
        type_cell: type_cell.into(),
        mult_cell: mult_cell.into(),
        constraints_cell: String::new(),
        reference_only: false,
    };
    let (type_ref, _) = map_type(type_cell, &row, ctx, diagnostics);
    let mult = map_multiplicity(mult_cell, line, diagnostics)?;
    let mut t = type_ref?;
    t.multiplicity = Some(mult);
    Some(t)
}

fn resolve_refs(
    text: &str,
    line: usize,
    clauses: &[ClauseRef],
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> Vec<ClauseRef> {
    let mut out = Vec::new();
    for id in text.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        match clauses.iter().find(|c| c.clause_id == id) {
            Some(c) => out.push(ClauseRef {
                language: c.language.clone(),
                clause_id: c.clause_id.clone(),
                source_span: None,
            }),
            None => diagnostics.push(err(
                "semantic.dangling-clause-ref",
                line,
                format!("clause {id} is declared by no invariant of this artifact"),
            )),
        }
    }
    out
}

fn validate_decls<T: Serialize>(
    items: &[T],
    model: &str,
    semantic_core: &str,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) {
    if items.is_empty() {
        return;
    }
    let Some(validator) = model_validator(model, semantic_core) else {
        return;
    };
    for item in items {
        let value = serde_json::to_value(item).unwrap_or(Value::Null);
        if !validator.is_valid(&value) {
            diagnostics.push(err(
                "semantic.internal-invalid-decl",
                0,
                format!("produced {model} does not validate against {model}.json (engine defect)"),
            ));
        }
    }
}

fn model_validator(model: &str, semantic_core: &str) -> Option<JSONSchema> {
    let schema = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": format!("https://schemas.agent-ix.org/agent-ix/quire-rs/{semantic_core}/{model}Gate.json"),
        "$ref": format!("https://schemas.agent-ix.org/semantic-core/{semantic_core}/{model}.json")
    });
    compile_module_schema(
        &schema,
        &|_| None,
        semantic_core,
        "https://schemas.agent-ix.org/agent-ix/quire-rs/",
    )
    .ok()
}
