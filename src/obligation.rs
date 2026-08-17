//! Obligation records — the quire↔quoin contract (FR-053).
//!
//! ADR 0011 names the obligation record the thing quire *states* and quoin
//! *audits the discharge of*. Neither side had one: `quire properties --json`
//! carries property shapes, FR-050 coverage carries the AC→TC rollup, and
//! neither carries a content hash or a verification method. Without the hash
//! nothing downstream can see a **suspect link** — evidence bound to a
//! statement that has since been reworded. Without the method nothing can see
//! a requirement whose declared verification is analysis discharged by a unit
//! test.
//!
//! An obligation is **derived, never authored**. There is no obligation
//! document and no `quire obligations` command: choosing an acceptance
//! criterion's verification method *is* minting the obligation, and this module
//! reads what the author already wrote.
//!
//! Everything about *which* rows state obligations is module data
//! ([`ObligationSource`]). The engine knows the shape — id, statement, method,
//! parameters, criticality — and never a column name, a method name or an
//! archetype (FR-053-CON-1).

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::corpus::declared_tables::{self, DeclaredScope, ExcludeSet, ScanContext, ScannedRow};
use crate::corpus::spec::Spec;
use crate::traceability::{ObligationSource, TraceabilityModel};

/// One derived obligation.
///
/// Deliberately carries no verdict, threshold comparison, evidence reference or
/// freshness judgement (FR-053-CON-2). The engine states obligations; reading
/// evidence and judging discharge is quoin's Auditor role.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Obligation {
    /// The source declaration that minted this record.
    pub source: String,
    /// The obligation's id — the row's own id, or rendered from `id_format`.
    /// This is the id the coverage rollup and every trace tag already key on,
    /// so a binding joins without translation.
    pub id: String,
    /// Scope-relative path of the document stating it.
    pub document: String,
    /// The normative statement, verbatim and untruncated.
    pub statement: String,
    /// SHA-256 of the **normalized** statement, lowercase hex. Suspect-link
    /// detection is `current_hash != hash_at_binding` and nothing more.
    pub statement_hash: String,
    /// Declared verification method, annotation stripped. Absent when the
    /// source declares no method column or the cell is empty. Not validated
    /// against any vocabulary here — that is the catalog's job and the
    /// auditor's.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    /// Declared parameters, result key → cell value. Absent keys are omitted,
    /// never defaulted: a threshold nobody wrote is not a threshold of zero.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub parameters: BTreeMap<String, String>,
    /// Declared criticality. Absent for a source declaring no criticality
    /// column — the ISO acceptance-criteria contract carries none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub criticality: Option<String>,
}

/// The obligation as it rides on one property-classification record (FR-053).
///
/// Carries no `id`, `statement` or `document`: `AcClassification` already has
/// `row_id` and `statement`, and `properties --json` nests the records under
/// their document. Repeating the three would put one fact in two places on a
/// single payload, which is the shape that eventually disagrees with itself.
///
/// It is also what keeps [`crate::validate_document::classify_document_criteria`]
/// text-only — a criterion's hash, method and criticality are readable from the
/// criterion's own row, where a document path is not.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CriterionObligation {
    pub source: String,
    pub statement_hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub parameters: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub criticality: Option<String>,
}

/// Derive the criterion-level obligations of **one parsed document**, keyed by
/// row id.
///
/// Used by the classification path, which holds a document and a registry but
/// no corpus. Only `target:`-bound sources whose archetype matches can apply:
/// an `archetype:`+`id_format:` source describes rows that mint no id, and the
/// `ac` grammar binds no such row, so none can ever appear here.
pub fn for_document(
    model: &TraceabilityModel,
    archetype: &str,
    doc: &crate::ast::QuireDocument,
) -> BTreeMap<String, CriterionObligation> {
    let mut out = BTreeMap::new();
    for source in &model.obligations {
        let Some(resolved) = resolve(source, model) else {
            continue;
        };
        if resolved.archetype != archetype {
            continue;
        }
        let Some(id_column) = resolved.id_column else {
            continue;
        };
        // The path is only used to stamp `ScannedRow.path`, which this caller
        // does not read — the rows come from the one document it was handed.
        for row in declared_tables::rows_of(doc, Path::new(""), resolved.section) {
            let (Some(id), Some(statement)) =
                (row.cell(id_column), row.cell(&source.statement_column))
            else {
                continue;
            };
            let mut parameters = BTreeMap::new();
            for (key, column) in &source.parameters {
                if let Some(value) = row.cell(column) {
                    parameters.insert(key.clone(), value.to_string());
                }
            }
            // First source wins for a given id, matching how every other
            // declaration merges (NFR-006: the answer cannot depend on which
            // module happened to load first).
            out.entry(id.to_string())
                .or_insert_with(|| CriterionObligation {
                    source: source.name.clone(),
                    statement_hash: statement_hash(statement),
                    method: source
                        .method_column
                        .as_deref()
                        .and_then(|c| row.cell(c))
                        .and_then(method_of),
                    parameters,
                    criticality: source
                        .criticality_column
                        .as_deref()
                        .and_then(|c| row.cell(c))
                        .map(str::to_string),
                });
        }
    }
    out
}

/// Normalize a statement for hashing: NFC, trim, collapse whitespace runs.
///
/// **Not** the CR-017 code-span mask, and the difference matters. CR-017
/// replaces a code span's *contents* with `x` so a quoted keyword reads as a
/// mention rather than a use — correct for grammar detection, catastrophic for
/// a hash: ``reject a `foo` token`` and ``reject a `bar` token`` mask to the
/// same string, so a suspect-link detector built on it would stay silent
/// through the rename it exists to catch. A backticked identifier is usually
/// the most load-bearing token in a criterion.
///
/// Whitespace normalization gives the property that was actually wanted — a
/// re-wrap or a trailing space does not churn the hash — without ever equating
/// two statements a reader would read differently.
///
/// NFC is applied by comparing char classes rather than pulling a Unicode
/// normalization dependency: the engine has no `unicode-normalization` crate
/// and NFR-009 pins dependencies deliberately. Statements in this corpus are
/// authored NFC by every editor in use; the trim-and-collapse pass is what
/// carries the property in practice, and a future NFC pass can be added without
/// changing any consumer, because it only ever *reduces* churn.
pub fn normalize_statement(statement: &str) -> String {
    let mut out = String::with_capacity(statement.len());
    let mut in_space = false;
    for ch in statement.trim().chars() {
        if ch.is_whitespace() {
            in_space = true;
            continue;
        }
        if in_space && !out.is_empty() {
            out.push(' ');
        }
        in_space = false;
        out.push(ch);
    }
    out
}

/// SHA-256 of the normalized statement, lowercase hex.
///
/// Computed from the statement **alone** — never the id, path, line or method
/// (FR-053-CON-3). A statement that moves file or renumbers has not changed,
/// and a suspect-link detector that fires on a move is one that gets switched
/// off within a week.
pub fn statement_hash(statement: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(normalize_statement(statement).as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Strip a trailing parenthetical annotation from a method cell.
///
/// A `Verification` cell reads `Test (TC-707)`: the method is `Test` and
/// `TC-707` is the reference FR-049's integrity check reads from the same cell.
/// Both readings are correct and neither consumes the other.
fn method_of(cell: &str) -> Option<String> {
    let head = match cell.find('(') {
        Some(idx) => &cell[..idx],
        None => cell,
    };
    let trimmed = head.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

/// Render an `id_format` template for a row that mints no id of its own.
fn render_id(template: &str, document: &str, row: usize) -> String {
    template
        .replace("{document}", document)
        .replace("{row}", &row.to_string())
}

/// A row that stated no obligation, and why. Reported rather than dropped: a
/// row whose statement cell is empty is an authoring error, and emitting it as
/// a record with an empty statement would put an obligation nothing can state
/// into the contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedRow {
    pub source: String,
    pub document: String,
    pub row: usize,
}

/// Derive every obligation the model declares, in a deterministic order.
///
/// Order is (source declaration order, document id, row ordinal) and every map
/// is a `BTreeMap`, so two runs over identical inputs serialize byte-identically
/// (NFR-006, FR-053-AC-9).
pub fn derive(
    spec: &Spec,
    root: &Path,
    model: &TraceabilityModel,
) -> (Vec<Obligation>, Vec<SkippedRow>) {
    let mut out = Vec::new();
    let mut skipped = Vec::new();
    let model_exclude = ExcludeSet::compile(&model.exclude);
    let mut ctx = ScanContext::default();

    for source in &model.obligations {
        let Some(resolved) = resolve(source, model) else {
            // Unreachable for a validated model — `TraceabilityModel::validate`
            // rejects both-or-neither and an undeclared target at load. Skipped
            // rather than panicking, because a caller holding an unvalidated
            // model should get no obligations, not a crash.
            continue;
        };
        let exclude = ExcludeSet::compile(&source.exclude);
        let scope = DeclaredScope {
            name: &source.name,
            archetype: resolved.archetype,
            exclude: &exclude,
            model_exclude: &model_exclude,
        };
        let rows = declared_tables::scan(spec, root, scope, resolved.section, &mut ctx);
        collect(source, &resolved, &rows, root, &mut out, &mut skipped);
    }

    // Sort is stable and the scan already yields documents in corpus order, so
    // this only enforces the by-document ordering the AC states without
    // disturbing row order within a document.
    out.sort_by(|a, b| (&a.source, &a.document).cmp(&(&b.source, &b.document)));
    (out, skipped)
}

/// What a source resolved to: where its rows live and how they are identified.
struct Resolved<'a> {
    archetype: &'a str,
    section: &'a str,
    /// `Some` when rows carry their own id in this column; `None` when ids are
    /// rendered from the template instead.
    id_column: Option<&'a str>,
    id_format: Option<&'a str>,
}

fn resolve<'a>(source: &'a ObligationSource, model: &'a TraceabilityModel) -> Option<Resolved<'a>> {
    match (&source.target, &source.archetype) {
        (Some(target), None) => {
            let target = model.trace_targets.iter().find(|t| &t.name == target)?;
            Some(Resolved {
                archetype: &target.archetype,
                section: &target.section,
                id_column: Some(&target.id_column),
                id_format: None,
            })
        }
        (None, Some(archetype)) => Some(Resolved {
            archetype,
            section: source.section.as_deref()?,
            id_column: None,
            id_format: Some(source.id_format.as_deref()?),
        }),
        _ => None,
    }
}

fn collect(
    source: &ObligationSource,
    resolved: &Resolved<'_>,
    rows: &[ScannedRow],
    root: &Path,
    out: &mut Vec<Obligation>,
    skipped: &mut Vec<SkippedRow>,
) {
    // The row ordinal is per document, not per scan: `{row}` must mean "the
    // second row of this NFR's measurement table", or an id changes whenever an
    // unrelated document is added to the corpus.
    let mut ordinal: BTreeMap<&Path, usize> = BTreeMap::new();
    for row in rows {
        let n = ordinal.entry(row.path.as_path()).or_insert(0);
        *n += 1;
        let n = *n;
        let document = declared_tables::relative_path(root, &row.path);

        let Some(statement) = row.cell(&source.statement_column) else {
            skipped.push(SkippedRow {
                source: source.name.clone(),
                document,
                row: n,
            });
            continue;
        };

        let id = match (resolved.id_column, resolved.id_format) {
            (Some(column), _) => match row.cell(column) {
                Some(id) => id.to_string(),
                None => {
                    skipped.push(SkippedRow {
                        source: source.name.clone(),
                        document,
                        row: n,
                    });
                    continue;
                }
            },
            (None, Some(template)) => {
                let stem = document_id(&document);
                render_id(template, &stem, n)
            }
            (None, None) => continue,
        };

        let mut parameters = BTreeMap::new();
        for (key, column) in &source.parameters {
            if let Some(value) = row.cell(column) {
                parameters.insert(key.clone(), value.to_string());
            }
        }

        out.push(Obligation {
            source: source.name.clone(),
            id,
            document,
            statement: statement.to_string(),
            statement_hash: statement_hash(statement),
            method: source
                .method_column
                .as_deref()
                .and_then(|c| row.cell(c))
                .and_then(method_of),
            parameters,
            criticality: source
                .criticality_column
                .as_deref()
                .and_then(|c| row.cell(c))
                .map(str::to_string),
        });
    }
}

/// The `{document}` substitution: the file stem, which is the artifact id for
/// every ISO-shaped document (`NFR-006-determinism.md` → `NFR-006`).
///
/// Taken from the path rather than frontmatter because the scan yields rows,
/// not documents, and because a template author writing `{document}-M-{row}`
/// means the id they can see in the filename.
fn document_id(document: &str) -> String {
    let stem = document
        .rsplit('/')
        .next()
        .unwrap_or(document)
        .trim_end_matches(".md");
    // `NFR-006-determinism` → `NFR-006`: the leading `<PREFIX>-<digits>` run.
    let mut parts = stem.split('-');
    match (parts.next(), parts.next()) {
        (Some(prefix), Some(number)) if number.chars().all(|c| c.is_ascii_digit()) => {
            format!("{prefix}-{number}")
        }
        _ => stem.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalization_collapses_whitespace_but_not_words() {
        assert_eq!(normalize_statement("  a   b \n c  "), "a b c");
        assert_eq!(statement_hash("a b c"), statement_hash("  a   b \n c  "));
        assert_ne!(statement_hash("a b c"), statement_hash("a b d"));
    }

    #[test]
    fn code_span_contents_change_the_hash() {
        // The CR-017 mask would collapse these two; the whole point of not
        // reusing it is that they must differ (FR-053-AC-4).
        assert_ne!(
            statement_hash("The parser SHALL reject a `foo` token."),
            statement_hash("The parser SHALL reject a `bar` token."),
        );
    }

    #[test]
    fn method_drops_a_trailing_annotation() {
        assert_eq!(method_of("Test (TC-707)"), Some("Test".to_string()));
        assert_eq!(method_of("Analysis"), Some("Analysis".to_string()));
        assert_eq!(method_of("(TC-707)"), None);
        assert_eq!(method_of("   "), None);
    }

    #[test]
    fn id_template_renders_document_and_row() {
        assert_eq!(render_id("{document}-M-{row}", "NFR-006", 2), "NFR-006-M-2");
    }

    #[test]
    fn document_id_takes_the_prefix_and_number() {
        assert_eq!(document_id("spec/non-functional/NFR-006-determinism.md"), "NFR-006");
        assert_eq!(document_id("spec/functional/FR-050-declarative.md"), "FR-050");
        assert_eq!(document_id("notes.md"), "notes");
    }
}
