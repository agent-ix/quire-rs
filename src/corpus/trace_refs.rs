//! Verification-reference integrity (FR-049).
//!
//! Table cells reference trace ids — the ISO AC `Verification` cell's
//! `Test (TC-035)` annotation is the motivating case — and those references
//! deserve the same dangling check `ix://` edges already get. Nothing about
//! the pattern, the column, or the target archetype is engine knowledge: every
//! part comes from the active module's declared traceability model (FR-050).
//!
//! The resolution set for a target kind is the union of bundle documents whose
//! archetype matches a declared target and the trace ids minted by declared
//! **auxiliary sources** — files the corpus walk excludes as non-artifacts
//! (`tests.md`), harvested here with a targeted scan following the FR-044
//! glossary-harvester pattern.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use regex::Regex;

use super::declared_tables;
use super::spec::Spec;
use super::validate::{pack, posture_tier, BundleFinding, BundlePosture, BundleReport};
use crate::grammar::GrammarSeverity;
use crate::registry::Registry;
use crate::traceability::{DocumentReference, TraceTarget, TraceTargetEvidence};

/// One referencing row found in a declared reference column.
struct ReferencingRow {
    /// Path of the document the reference was authored in.
    path: PathBuf,
    /// The row's own id, when the declaration names a `row_id_column`.
    row_id: Option<String>,
    /// The raw cell text.
    cell: String,
    /// The row's kind cell, when the declaration is kind-typed and the row
    /// carries the column.
    kind: Option<String>,
    /// The row's 1-based document line, frontmatter counted (FR-049-AC-19).
    line: usize,
}

/// Check every declared document reference against the bundle's resolution
/// set, appending posture-degradable `dangling-trace-reference` findings.
///
/// A no-op when no active module declares a traceability model (FR-049-AC-6).
pub(crate) fn validate_trace_references(
    spec: &Spec,
    registry: &Registry,
    posture: BundlePosture,
    root: &Path,
    report: &mut BundleReport,
) {
    let Some(model) = registry.traceability() else {
        return;
    };
    if model.document_references.is_empty() {
        return;
    }

    // CR-060: compiled once — every declaration below is scoped by it.
    let model_exclude = declared_tables::ExcludeSet::compile_validated(&model.exclude);

    // Resolution sets, one per declared target kind.
    let mut ctx = declared_tables::ScanContext::default();
    let mut resolution: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    for target in &model.trace_targets {
        let mut reference_only_ctx = declared_tables::ScanContext::default();
        let target_ctx = if target.evidence == TraceTargetEvidence::ReferenceOnly {
            &mut reference_only_ctx
        } else {
            &mut ctx
        };
        resolution.insert(
            target.name.as_str(),
            minted_ids(spec, root, target, &model_exclude, target_ctx),
        );
    }

    // Collect findings before pushing so their order is a property of the
    // bundle, not of the walk (FR-049-AC-8).
    // Ordered by (path, id, reason, message, line) (FR-049-AC-17, AC-21).
    let mut findings: Vec<(PathBuf, String, &'static str, String, usize)> = Vec::new();
    for declaration in &model.document_references {
        let Ok(pattern) = Regex::new(&declaration.pattern) else {
            continue; // patterns are validated at module load
        };
        let mut resolvable: BTreeSet<&String> = BTreeSet::new();
        for target in &declaration.targets {
            if let Some(ids) = resolution.get(target.as_str()) {
                resolvable.extend(ids.iter());
            }
        }
        let scanned = referencing_rows(spec, root, declaration, &model_exclude, &mut ctx);
        for row in scanned {
            let kind_subset = row
                .kind
                .as_deref()
                .and_then(|kind| declaration.targets_for_kind(kind));
            for caps in pattern.captures_iter(&row.cell) {
                let Some(id) = caps.get(1).map(|m| m.as_str().trim()) else {
                    continue;
                };
                if id.is_empty() {
                    continue;
                }
                let owner = row
                    .row_id
                    .as_deref()
                    .map(|r| format!(" (row '{r}')"))
                    .unwrap_or_default();
                if resolvable.iter().any(|known| known.as_str() == id) {
                    if let (Some(subset), Some(kind)) = (kind_subset, row.kind.as_deref()) {
                        let in_subset = subset.iter().any(|name| {
                            resolution
                                .get(name.as_str())
                                .is_some_and(|ids| ids.contains(id))
                        });
                        if !in_subset {
                            findings.push((
                                row.path.clone(),
                                id.to_string(),
                                "reference-kind-mismatch",
                                format!(
                                    "trace reference '{id}'{owner} declared by '{}' is on a row of kind \
                                     '{}' which may reference only {} targets",
                                    declaration.name,
                                    kind.trim(),
                                    subset.join("/")
                                ),
                                row.line,
                            ));
                        }
                    }
                    continue;
                }
                findings.push((
                    row.path.clone(),
                    id.to_string(),
                    "dangling-trace-reference",
                    format!(
                        "trace reference '{id}'{owner} declared by '{}' resolves to no {} target",
                        declaration.name,
                        declaration.targets.join("/")
                    ),
                    row.line,
                ));
            }
        }
    }

    // CR-054: a declaration that scanned nothing is reported as a warning
    // naming the declaration, in both postures. Before this, an `archetype:`
    // typo or an unreadable declared document produced an empty resolution
    // set in silence — and since CR-049 made selection load-bearing, the same
    // silence also stopped the engine parsing the bodies it would have read.
    //
    // The `minted_anything` argument is gone with the model-wide gate it fed
    // (CR-135, #304): one declaration succeeding is not a reason to withhold a
    // different declaration's finding.
    let severity = registry.grammar_severity();
    for (declaration, diagnostic) in ctx.into_diagnostics() {
        let (path, message) = declared_tables::scan_finding(&declaration, &diagnostic, root);
        // Warn tier in both postures (CR-054), now tunable (FR-057).
        report.route(
            severity,
            GrammarSeverity::Warning,
            BundleFinding::in_pack(
                pack::TRACE,
                declared_tables::scan_reason(&diagnostic),
                path,
                message,
            ),
        );
    }

    findings.sort();
    findings.dedup();
    for (path, _, reason, message, line) in findings {
        report.route(
            severity,
            posture_tier(posture),
            BundleFinding::in_pack(pack::TRACE, reason, path, message).at_line(Some(line)),
        );
    }
}

/// The ids a declared target mints: from bundle documents of the declared
/// archetype, or from a declared auxiliary document harvested off-corpus.
fn minted_ids(
    spec: &Spec,
    root: &Path,
    target: &TraceTarget,
    model_exclude: &declared_tables::ExcludeSet,
    ctx: &mut declared_tables::ScanContext,
) -> BTreeSet<String> {
    // A document of the target archetype mints its own id, too — an authored
    // `TC-900.md` is as much a target as a matrix row. An excluded document
    // mints neither (CR-038): a fixture is test data whether its ids come from
    // its frontmatter or from its rows.
    //
    // This is an axis `mint()` does not cover: it reads the declared
    // sections, this reads frontmatter `id`. The CR-117 minting diagnostics
    // `mint()` notes are reported under the same machine tokens `coverage`
    // uses — one vocabulary, two surfaces (CR-054).
    let mut ids: BTreeSet<String> = target_documents(spec, root, target, model_exclude)
        .map(|doc| doc.id.clone())
        .collect();
    ids.extend(
        declared_tables::mint(spec, root, target, model_exclude, ctx)
            .into_iter()
            .map(|minted| minted.id),
    );
    ids
}

/// Frontmatter-minted targets shared by reference resolution and retained lookup.
/// Row declarations remain the existing `declared_tables::scan` path.
pub(crate) fn target_documents<'a>(
    spec: &'a Spec,
    root: &'a Path,
    target: &'a TraceTarget,
    model_exclude: &'a declared_tables::ExcludeSet,
) -> impl Iterator<Item = &'a super::walk::LoadedDocument> {
    let exclude = declared_tables::ExcludeSet::compile_validated(&target.exclude);
    spec.documents().iter().filter(move |document| {
        document.concept_type() == Some(target.archetype.as_str())
            && !document.id.is_empty()
            && !model_exclude.excludes(root, &document.path)
            && !exclude.excludes(root, &document.path)
    })
}

/// Every referencing row a declaration covers, in bundle order.
fn referencing_rows(
    spec: &Spec,
    root: &Path,
    declaration: &DocumentReference,
    model_exclude: &declared_tables::ExcludeSet,
    ctx: &mut declared_tables::ScanContext,
) -> Vec<ReferencingRow> {
    let exclude = declared_tables::ExcludeSet::compile_validated(&declaration.exclude);
    let scanned = declared_tables::scan(
        spec,
        root,
        declared_tables::DeclaredScope {
            name: &declaration.name,
            archetype: &declaration.archetype,
            exclude: &exclude,
            model_exclude,
            mints: None,
            status_column: None,
            kind_column: declaration
                .kind_column
                .as_deref()
                .map(|kind| (declaration.column.as_str(), kind)),
            target_required: false,
        },
        &declaration.section,
        ctx,
    );
    let mut rows = Vec::new();
    for row in scanned {
        // CR-015: apply the declaration's opt-in normalizations before any id
        // is read out of the cell.
        let Some(raw) = row.cell(&declaration.column) else {
            continue;
        };
        let cell = declared_tables::normalize_reference_cell(
            raw,
            declaration.strip_annotations,
            declaration.expand_ranges,
        );
        let row_id = declaration
            .row_id_column
            .as_deref()
            .and_then(|c| row.cell(c))
            .map(str::to_string);
        let kind = declaration
            .kind_column
            .as_deref()
            .and_then(|c| row.cell(c))
            .map(str::to_string);
        rows.push(ReferencingRow {
            path: row.path,
            row_id,
            cell,
            kind,
            line: row.line,
        });
    }
    rows
}
