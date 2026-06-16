//! Declarative lint rules (FR-036).
//!
//! Modules declare advisory rules in `manifest.yaml` under a top-level
//! `lint_rules:` list. Lint is a separate posture from structural
//! validation (FR-032): findings NEVER block extraction or document
//! sync — they surface authoring-convention drift (e.g. an Acceptance
//! Criteria `Verification` cell outside the ISO 29148 method
//! vocabulary, or a `Configuration` table `Scope` cell outside
//! `creation`/`runtime`/`session`).
//!
//! One rule type ships in v1: `table_column_values` — every data cell
//! in a named column of the table under a named section must be one of
//! an allowed set, optionally followed by an annotation matching a
//! regex (e.g. `Test (TC-035)`).
//!
//! A rule may scope itself to specific archetypes via `archetypes:`;
//! an empty/absent list applies the rule to every document linted
//! against the module.

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::ast::QuireDocument;
use crate::query::table_from_section;

/// Severity of a lint finding. Lint is advisory either way — severity
/// is a reporting/exit-code distinction for CLI consumers, not a
/// validation gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LintSeverity {
    #[default]
    Warning,
    Error,
}

impl LintSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

/// One declarative lint rule, discriminated by `type:`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LintRule {
    /// Every data cell in `column` of the table under `section` must be
    /// one of `allowed`, optionally followed by whitespace plus an
    /// annotation matching `annotation_pattern` (anchored match).
    TableColumnValues {
        /// Stable identifier reported with each finding.
        id: String,
        /// Archetype names this rule applies to (empty = all).
        #[serde(default)]
        archetypes: Vec<String>,
        /// Heading of the section owning the table.
        section: String,
        /// Header of the column to check (case-sensitive).
        column: String,
        /// Allowed leading values.
        allowed: Vec<String>,
        /// Optional regex the trailing annotation must match (evaluated
        /// against the remainder after an allowed value + whitespace).
        #[serde(default)]
        annotation_pattern: Option<String>,
        #[serde(default)]
        severity: LintSeverity,
    },
}

impl LintRule {
    pub fn id(&self) -> &str {
        match self {
            Self::TableColumnValues { id, .. } => id,
        }
    }

    pub fn severity(&self) -> LintSeverity {
        match self {
            Self::TableColumnValues { severity, .. } => *severity,
        }
    }

    /// Whether the rule applies to a document of `archetype`. An empty
    /// `archetypes` filter applies everywhere; `None` (archetype could
    /// not be resolved) only matches unfiltered rules.
    pub fn applies_to(&self, archetype: Option<&str>) -> bool {
        let filter = match self {
            Self::TableColumnValues { archetypes, .. } => archetypes,
        };
        if filter.is_empty() {
            return true;
        }
        match archetype {
            Some(name) => filter.iter().any(|a| a == name),
            None => false,
        }
    }
}

/// One lint finding. `severity` mirrors the owning rule.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LintFinding {
    pub rule: String,
    pub severity: LintSeverity,
    pub message: String,
}

/// Evaluate `rules` against `doc`. `archetype` is the resolved
/// archetype name used for rule scoping (lint never fails on an
/// unresolvable archetype — scoped rules simply don't run).
pub fn lint_document(
    rules: &[LintRule],
    archetype: Option<&str>,
    doc: &QuireDocument,
) -> Vec<LintFinding> {
    let mut findings = Vec::new();
    for rule in rules {
        if !rule.applies_to(archetype) {
            continue;
        }
        match rule {
            LintRule::TableColumnValues {
                id,
                section,
                column,
                allowed,
                annotation_pattern,
                severity,
                ..
            } => eval_table_column_values(
                doc,
                id,
                section,
                column,
                allowed,
                annotation_pattern.as_deref(),
                *severity,
                &mut findings,
            ),
        }
    }
    findings
}

#[allow(clippy::too_many_arguments)]
fn eval_table_column_values(
    doc: &QuireDocument,
    rule_id: &str,
    section: &str,
    column: &str,
    allowed: &[String],
    annotation_pattern: Option<&str>,
    severity: LintSeverity,
    findings: &mut Vec<LintFinding>,
) {
    // Missing section/table/column is not a lint finding — structural
    // requirements are validation's job (FR-032), not lint's.
    let Some(table) = table_from_section(doc, section) else {
        return;
    };
    let Some(col_idx) = table.headers.iter().position(|h| h == column) else {
        return;
    };
    let annotation_re = annotation_pattern.and_then(|p| Regex::new(&anchored(p)).ok());

    for (row_n, row) in table.rows.iter().enumerate() {
        let Some(cell) = row.get(col_idx) else {
            continue;
        };
        let cell = cell.trim();
        if cell_is_valid(cell, allowed, annotation_re.as_ref()) {
            continue;
        }
        findings.push(LintFinding {
            rule: rule_id.to_string(),
            severity,
            message: format!(
                "section '{section}' column '{column}' row {}: value '{cell}' \
                 is not one of [{}]",
                row_n + 1,
                allowed.join(", "),
            ),
        });
    }
}

/// A cell is valid when it equals an allowed value, or starts with one
/// and the remainder (after whitespace) matches the annotation regex.
fn cell_is_valid(cell: &str, allowed: &[String], annotation_re: Option<&Regex>) -> bool {
    for value in allowed {
        if cell == value {
            return true;
        }
        if let Some(rest) = cell.strip_prefix(value.as_str()) {
            let rest = rest.trim_start();
            if rest.is_empty() {
                return true; // trailing whitespace only
            }
            if let Some(re) = annotation_re {
                if re.is_match(rest) {
                    return true;
                }
            }
        }
    }
    false
}

/// Anchor a user pattern so the WHOLE annotation must match.
fn anchored(pattern: &str) -> String {
    format!(
        "^(?:{})$",
        pattern.trim_start_matches('^').trim_end_matches('$')
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_document;

    const AC_DOC: &str = "\
---\nid: FR-001\ntype: FR\n---\n\
## Acceptance Criteria\n\
| ID | Criteria | Verification |\n\
| - | - | - |\n\
| FR-001-AC-1 | does x | Test (TC-035) |\n\
| FR-001-AC-2 | does y | Inspection |\n\
| FR-001-AC-3 | does z | Docs audit |\n";

    fn verification_rule(severity: LintSeverity) -> LintRule {
        LintRule::TableColumnValues {
            id: "ac-verification-method".to_string(),
            archetypes: vec!["FR".to_string()],
            section: "Acceptance Criteria".to_string(),
            column: "Verification".to_string(),
            allowed: ["Inspection", "Analysis", "Demonstration", "Test"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            annotation_pattern: Some(r"\(TC-\d+(,\s*TC-\d+)*\)".to_string()),
            severity,
        }
    }

    #[test]
    fn allowed_values_and_annotations_pass_others_flagged() {
        let doc = parse_document(AC_DOC);
        let findings = lint_document(
            &[verification_rule(LintSeverity::Warning)],
            Some("FR"),
            &doc,
        );
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].rule, "ac-verification-method");
        assert_eq!(findings[0].severity, LintSeverity::Warning);
        assert!(findings[0].message.contains("Docs audit"));
        assert!(findings[0].message.contains("row 3"));
    }

    #[test]
    fn archetype_scoping_skips_non_matching_documents() {
        let doc = parse_document(AC_DOC);
        let rule = verification_rule(LintSeverity::Warning);
        assert!(lint_document(std::slice::from_ref(&rule), Some("NFR"), &doc).is_empty());
        // Unresolvable archetype only runs unfiltered rules.
        assert!(lint_document(&[rule], None, &doc).is_empty());
    }

    #[test]
    fn missing_section_or_column_yields_no_findings() {
        let doc = parse_document("## Other\nprose\n");
        let findings = lint_document(&[verification_rule(LintSeverity::Error)], Some("FR"), &doc);
        assert!(findings.is_empty());
    }

    #[test]
    fn severity_error_is_carried_on_findings() {
        let doc = parse_document(AC_DOC);
        let findings = lint_document(&[verification_rule(LintSeverity::Error)], Some("FR"), &doc);
        assert_eq!(findings[0].severity, LintSeverity::Error);
    }

    #[test]
    fn scope_column_rule_for_configuration_tables() {
        let md = "\
---\nid: FR-071\ntype: FR\nobject: configuration\n---\n\
## Configuration\n\
| Name | Scope | Type | Default | Description |\n\
| - | - | - | - | - |\n\
| m | creation | int | 16 | graph degree |\n\
| ef_search | runtime | int | 64 | beam width |\n\
| mood | vibes | str | - | not a scope |\n";
        let doc = parse_document(md);
        let rule = LintRule::TableColumnValues {
            id: "configuration-scope".to_string(),
            archetypes: Vec::new(),
            section: "Configuration".to_string(),
            column: "Scope".to_string(),
            allowed: ["creation", "runtime", "session"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            annotation_pattern: None,
            severity: LintSeverity::Warning,
        };
        let findings = lint_document(&[rule], Some("FR"), &doc);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("vibes"));
    }

    #[test]
    fn rule_yaml_round_trip() {
        let yaml = r#"
- type: table_column_values
  id: ac-verification-method
  archetypes: [FR]
  section: Acceptance Criteria
  column: Verification
  allowed: [Inspection, Analysis, Demonstration, Test]
  annotation_pattern: '\(TC-\d+\)'
  severity: warning
"#;
        let rules: Vec<LintRule> = serde_yaml::from_str(yaml).expect("parse");
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].id(), "ac-verification-method");
        assert_eq!(rules[0].severity(), LintSeverity::Warning);
    }
}
