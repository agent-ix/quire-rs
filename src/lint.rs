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
//! Three rule types ship: `table_column_values` — every data cell in a
//! named column of the table under a named section must be one of an
//! allowed set, optionally followed by an annotation matching a regex
//! (e.g. `Test (TC-035)`); `section_body_pattern`, which warns when
//! a named section's body does not match a regex (e.g. a requirement
//! `Statement` lacking the `shall` keyword); and `forbidden_section`
//! (CR-020), which warns when a section the archetype never declared is
//! present at all (e.g. `## Acceptance Criteria` on a US, whose criteria
//! are non-binding by design).
//!
//! A rule may scope itself to specific archetypes via `archetypes:`;
//! an empty/absent list applies the rule to every document linted
//! against the module.

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::ast::{QuireDocument, QuireSection};
use crate::query::{section, table_from_section};

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
    /// The body of the section under `section` must contain a match for
    /// `pattern` (an `is_match`, not an anchored whole-body match). A
    /// missing section produces no finding — structural presence is
    /// validation's job, not lint's.
    SectionBodyPattern {
        /// Stable identifier reported with each finding.
        id: String,
        /// Archetype names this rule applies to (empty = all).
        #[serde(default)]
        archetypes: Vec<String>,
        /// Heading of the section whose body is checked.
        section: String,
        /// Regex the body must contain (`is_match`).
        pattern: String,
        /// Optional custom finding message (overrides the default).
        #[serde(default)]
        message: Option<String>,
        #[serde(default)]
        severity: LintSeverity,
    },
    /// The document must **not** carry a section headed `section`. The
    /// inverse of the other two rules, which only ever check a section that
    /// is present: this one fires precisely because it is there.
    ///
    /// Added for the case a `section_body_pattern` cannot express — a US
    /// document carrying an `## Acceptance Criteria` heading its archetype
    /// never declared. Structural *requirements* stay validation's job; a
    /// heading that is merely not part of the contract is drift, which is
    /// advisory (CR-020, spec-artifacts-iso#9).
    ForbiddenSection {
        /// Stable identifier reported with each finding.
        id: String,
        /// Archetype names this rule applies to (empty = all).
        #[serde(default)]
        archetypes: Vec<String>,
        /// Heading that must be absent.
        section: String,
        /// Optional custom finding message (overrides the default). Use it to
        /// point the author at the section they should have used instead.
        #[serde(default)]
        message: Option<String>,
        #[serde(default)]
        severity: LintSeverity,
    },
}

impl LintRule {
    pub fn id(&self) -> &str {
        match self {
            Self::TableColumnValues { id, .. }
            | Self::SectionBodyPattern { id, .. }
            | Self::ForbiddenSection { id, .. } => id,
        }
    }

    pub fn severity(&self) -> LintSeverity {
        match self {
            Self::TableColumnValues { severity, .. }
            | Self::SectionBodyPattern { severity, .. }
            | Self::ForbiddenSection { severity, .. } => *severity,
        }
    }

    /// Whether the rule applies to a document of `archetype`. An empty
    /// `archetypes` filter applies everywhere; `None` (archetype could
    /// not be resolved) only matches unfiltered rules.
    pub fn applies_to(&self, archetype: Option<&str>) -> bool {
        let filter = match self {
            Self::TableColumnValues { archetypes, .. }
            | Self::SectionBodyPattern { archetypes, .. }
            | Self::ForbiddenSection { archetypes, .. } => archetypes,
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
            LintRule::SectionBodyPattern {
                id,
                section,
                pattern,
                message,
                severity,
                ..
            } => eval_section_body_pattern(
                doc,
                id,
                section,
                pattern,
                message.as_deref(),
                *severity,
                &mut findings,
            ),
            LintRule::ForbiddenSection {
                id,
                section,
                message,
                severity,
                ..
            } => eval_forbidden_section(
                doc,
                id,
                section,
                message.as_deref(),
                *severity,
                &mut findings,
            ),
        }
    }
    findings
}

fn eval_forbidden_section(
    doc: &QuireDocument,
    rule_id: &str,
    section_heading: &str,
    message: Option<&str>,
    severity: LintSeverity,
    findings: &mut Vec<LintFinding>,
) {
    if section(doc, section_heading).is_none() {
        return;
    }
    findings.push(LintFinding {
        rule: rule_id.to_string(),
        severity,
        message: message.map(str::to_string).unwrap_or_else(|| {
            format!("section '{section_heading}' is not part of this archetype's contract")
        }),
    });
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

fn eval_section_body_pattern(
    doc: &QuireDocument,
    rule_id: &str,
    section_heading: &str,
    pattern: &str,
    message: Option<&str>,
    severity: LintSeverity,
    findings: &mut Vec<LintFinding>,
) {
    // Missing section is not a lint finding — structural presence is
    // validation's job (FR-032), not lint's.
    let Some(s) = section(doc, section_heading) else {
        return;
    };
    // An invalid regex is skipped (no panic), mirroring how the
    // table-column rule treats an unparseable annotation pattern.
    let Ok(re) = Regex::new(pattern) else {
        return;
    };
    // Match against the section's full subtree, not just its direct body:
    // a section's `content` stops at the next heading of ANY level, so a
    // token authored inside a subsection (e.g. a success-criterion id under
    // a `### Step` heading) would be missed by a direct-body check. This is
    // a "does this section contain X anywhere" advisory, so subsections count.
    if re.is_match(&section_subtree_text(s)) {
        return;
    }
    findings.push(LintFinding {
        rule: rule_id.to_string(),
        severity,
        message: message.map(str::to_string).unwrap_or_else(|| {
            format!("section '{section_heading}' body does not match expected pattern /{pattern}/")
        }),
    });
}

/// A section's own content plus the heading + content of every descendant,
/// joined with newlines — the text a `section_body_pattern` rule matches
/// against so a token anywhere under the heading (including subsections)
/// satisfies the rule.
fn section_subtree_text(s: &QuireSection) -> String {
    let mut out = String::from(&s.content);
    for child in &s.children {
        out.push('\n');
        out.push_str(&child.heading);
        out.push('\n');
        out.push_str(&section_subtree_text(child));
    }
    out
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

    // TC-585 (FR-036-AC-2): the Verification vocabulary rule, both directions.
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

    // TC-586 (FR-036-AC-3): a scoped rule ignores documents it does not cover.
    #[test]
    fn archetype_scoping_skips_non_matching_documents() {
        let doc = parse_document(AC_DOC);
        let rule = verification_rule(LintSeverity::Warning);
        assert!(lint_document(std::slice::from_ref(&rule), Some("NFR"), &doc).is_empty());
        // Unresolvable archetype only runs unfiltered rules.
        assert!(lint_document(&[rule], None, &doc).is_empty());
    }

    // TC-587 (FR-036-AC-4): structure is FR-032's job, not a lint finding.
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

    // TC-584 (FR-036-AC-1): rules parse typed and survive a round trip.
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

    // TC-609 (FR-036-AC-6): `section_body_pattern` advisory rule.
    fn shall_rule(severity: LintSeverity) -> LintRule {
        LintRule::SectionBodyPattern {
            id: "statement-shall".to_string(),
            archetypes: vec!["FR".to_string()],
            section: "Statement".to_string(),
            pattern: r"\bshall\b".to_string(),
            message: None,
            severity,
        }
    }

    // (a) a body containing the pattern produces NO finding.
    #[test]
    fn tc609_section_body_pattern_match_yields_no_finding() {
        let doc = parse_document("## Statement\nThe system shall do the thing.\n");
        let findings = lint_document(&[shall_rule(LintSeverity::Warning)], Some("FR"), &doc);
        assert!(findings.is_empty(), "{findings:?}");
    }

    // (b) a body lacking the pattern produces exactly one finding with the
    // rule's severity.
    #[test]
    fn tc609_section_body_pattern_mismatch_yields_one_finding() {
        let doc = parse_document("## Statement\nThe system does the thing.\n");
        let findings = lint_document(&[shall_rule(LintSeverity::Error)], Some("FR"), &doc);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].rule, "statement-shall");
        assert_eq!(findings[0].severity, LintSeverity::Error);
        assert!(findings[0].message.contains("Statement"));
    }

    // (c) archetype scoping skips non-matching archetypes.
    #[test]
    fn tc609_section_body_pattern_archetype_scoping() {
        let doc = parse_document("## Statement\nThe system does the thing.\n");
        let rule = shall_rule(LintSeverity::Warning);
        assert!(lint_document(std::slice::from_ref(&rule), Some("NFR"), &doc).is_empty());
        assert!(lint_document(&[rule], None, &doc).is_empty());
    }

    // (d) a missing section yields no finding.
    #[test]
    fn tc609_section_body_pattern_missing_section_yields_no_finding() {
        let doc = parse_document("## Other\nprose\n");
        let findings = lint_document(&[shall_rule(LintSeverity::Warning)], Some("FR"), &doc);
        assert!(findings.is_empty());
    }

    // (e) YAML round-trip for the new `type: section_body_pattern` rule,
    // including a custom `message`.
    #[test]
    fn tc609_section_body_pattern_yaml_round_trip() {
        let yaml = r#"
- type: section_body_pattern
  id: statement-shall
  archetypes: [FR]
  section: Statement
  pattern: '\bshall\b'
  message: 'requirement statements should use the keyword "shall"'
  severity: warning
"#;
        let rules: Vec<LintRule> = serde_yaml::from_str(yaml).expect("parse");
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].id(), "statement-shall");
        assert_eq!(rules[0].severity(), LintSeverity::Warning);

        // The custom message is emitted verbatim on a mismatch.
        let doc = parse_document("## Statement\nno keyword here\n");
        let findings = lint_document(&rules, Some("FR"), &doc);
        assert_eq!(findings.len(), 1);
        assert_eq!(
            findings[0].message,
            "requirement statements should use the keyword \"shall\""
        );
    }

    // (f) a token authored inside a subsection counts — the rule matches the
    // section's full subtree, not just its direct body (e.g. an `IT-XXX-SC-NN`
    // id under a `### Step` heading satisfies an IT success-criteria rule).
    #[test]
    fn tc609_section_body_pattern_matches_in_subsection() {
        let rule = LintRule::SectionBodyPattern {
            id: "it-sc".to_string(),
            archetypes: vec!["IT".to_string()],
            section: "Test Procedure".to_string(),
            pattern: r"\bIT-\d+-SC-\d+".to_string(),
            message: None,
            severity: LintSeverity::Warning,
        };
        // The SC id lives in a `### Step` subsection, not the H2's direct body.
        let doc = parse_document(
            "## Test Procedure\nRun the steps below.\n### Step 1\nDo a thing. IT-001-SC-01: ok.\n",
        );
        assert!(
            lint_document(std::slice::from_ref(&rule), Some("IT"), &doc).is_empty(),
            "SC id in a subsection should satisfy the rule",
        );
        // No SC id anywhere in the subtree → one finding.
        let doc2 = parse_document("## Test Procedure\nRun the steps.\n### Step 1\nDo a thing.\n");
        assert_eq!(lint_document(&[rule], Some("IT"), &doc2).len(), 1);
    }

    // TC-764 (FR-036-AC-7, CR-020): `forbidden_section` fires *because* a
    // section is present — the inverse of the other two rule types, which only
    // ever check a section that exists. The motivating case is a US document
    // carrying an `## Acceptance Criteria` heading its archetype never declared
    // (spec-artifacts-iso#9); `section_body_pattern` cannot express it, since a
    // missing section is defined to produce no finding.
    #[test]
    fn tc764_forbidden_section() {
        let yaml = r#"
- type: forbidden_section
  id: us-acceptance-criteria-drift
  archetypes: [US]
  section: Acceptance Criteria
  message: "a US carries `## Acceptance Examples (Illustrative)`; nothing on a US is binding"
  severity: warning
"#;
        let rules: Vec<LintRule> = serde_yaml::from_str(yaml).expect("parse");
        assert_eq!(rules[0].id(), "us-acceptance-criteria-drift");
        assert_eq!(rules[0].severity(), LintSeverity::Warning);

        // Present → exactly one finding, carrying the custom message.
        let drifted = parse_document(
            "## Acceptance Examples (Illustrative)\nGiven X, when Y, then Z.\n\
             ## Acceptance Criteria\n| ID | Criteria | Verification |\n",
        );
        let findings = lint_document(&rules, Some("US"), &drifted);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("Acceptance Examples"));

        // Absent → no finding. This is the conformant US.
        let clean =
            parse_document("## Acceptance Examples (Illustrative)\nGiven X, when Y, then Z.\n");
        assert!(lint_document(&rules, Some("US"), &clean).is_empty());

        // `archetypes:` scoping applies as it does to the other rule types: the
        // same heading on an FR is that archetype's contract, not drift.
        assert!(lint_document(&rules, Some("FR"), &drifted).is_empty());
        assert!(lint_document(&rules, None, &drifted).is_empty());

        // Default message names the section when none is supplied.
        let bare = LintRule::ForbiddenSection {
            id: "x".to_string(),
            archetypes: vec![],
            section: "Acceptance Criteria".to_string(),
            message: None,
            severity: LintSeverity::Error,
        };
        let f = lint_document(&[bare], Some("US"), &drifted);
        assert_eq!(f.len(), 1);
        assert!(f[0].message.contains("Acceptance Criteria"));
        assert_eq!(f[0].severity, LintSeverity::Error);
    }
}
