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
    let mut findings: Vec<(PathBuf, String, String)> = Vec::new();
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
        for row in referencing_rows(spec, root, declaration, &model_exclude, &mut ctx) {
            for caps in pattern.captures_iter(&row.cell) {
                let Some(id) = caps.get(1).map(|m| m.as_str().trim()) else {
                    continue;
                };
                if id.is_empty() || resolvable.iter().any(|known| known.as_str() == id) {
                    continue;
                }
                let owner = row
                    .row_id
                    .as_deref()
                    .map(|r| format!(" (row '{r}')"))
                    .unwrap_or_default();
                findings.push((
                    row.path.clone(),
                    id.to_string(),
                    format!(
                        "trace reference '{id}'{owner} declared by '{}' resolves to no {} target",
                        declaration.name,
                        declaration.targets.join("/")
                    ),
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
    for (path, _, message) in findings {
        report.route(
            severity,
            posture_tier(posture),
            BundleFinding::in_pack(pack::TRACE, "dangling-trace-reference", path, message),
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
    let exclude = declared_tables::ExcludeSet::compile_validated(&target.exclude);
    let scope = declared_tables::DeclaredScope {
        name: &target.name,
        archetype: &target.archetype,
        exclude: &exclude,
        model_exclude,
        // A trace target: the CR-117 minting diagnostics apply, and `validate`
        // reports them under the same machine tokens `coverage` does — one
        // vocabulary, two surfaces (CR-054).
        mints: Some(&target.id_column),
        status_column: None,
        section_required: target.required,
    };
    let mut ids: BTreeSet<String> = BTreeSet::new();
    // A document of the target archetype mints its own id, too — an authored
    // `TC-900.md` is as much a target as a matrix row. An excluded document
    // mints neither (CR-038): a fixture is test data whether its ids come from
    // its frontmatter or from its rows.
    //
    // This is an axis `scan()` does not cover: it reads table rows, this reads
    // frontmatter `id`.
    for doc in &spec.inner.documents {
        if doc.concept_type() == Some(target.archetype.as_str())
            && !doc.id.is_empty()
            && !scope.excludes(root, &doc.path)
        {
            ids.insert(doc.id.clone());
        }
    }
    for row in declared_tables::scan(spec, root, scope, &target.section, ctx) {
        if let Some(id) = row.cell(&target.id_column) {
            ids.insert(id.to_string());
        }
    }
    ids
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
    declared_tables::scan(
        spec,
        root,
        declared_tables::DeclaredScope {
            name: &declaration.name,
            archetype: &declaration.archetype,
            exclude: &exclude,
            model_exclude,
            mints: None,
            status_column: None,
            section_required: false,
        },
        &declaration.section,
        ctx,
    )
    .into_iter()
    .filter_map(|row| {
        // CR-015: apply the declaration's opt-in normalizations before any id
        // is read out of the cell.
        let cell = declared_tables::normalize_reference_cell(
            row.cell(&declaration.column)?,
            declaration.strip_annotations,
            declaration.expand_ranges,
        );
        let row_id = declaration
            .row_id_column
            .as_deref()
            .and_then(|c| row.cell(c))
            .map(str::to_string);
        Some(ReferencingRow {
            path: row.path,
            row_id,
            cell,
        })
    })
    .collect()
}
