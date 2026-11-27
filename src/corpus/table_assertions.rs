//! Typed cross-row assertions declared by a traceability model (SWM-248).

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use super::declared_tables::{self, ExcludeSet, ScanContext, ScannedRow};
use super::spec::Spec;
use super::validate::{pack, posture_tier, BundleFinding, BundlePosture, BundleReport};
use crate::registry::Registry;
use crate::traceability::{TableAssertion, TraceTarget};
use ix_trace_rs::implements;

/// Evaluate declared table assertions and route violations through bundle
/// validation. No-op when the model declares none.
#[implements("FR-050-AC-73")]
pub(crate) fn validate_table_assertions(
    spec: &Spec,
    registry: &Registry,
    posture: BundlePosture,
    root: &Path,
    report: &mut BundleReport,
) {
    let Some(model) = registry.traceability() else {
        return;
    };
    if model.table_assertions.is_empty() {
        return;
    }

    let targets: BTreeMap<&str, &TraceTarget> = model
        .trace_targets
        .iter()
        .map(|target| (target.name.as_str(), target))
        .collect();
    let selected: BTreeSet<&str> = model
        .table_assertions
        .iter()
        .flat_map(|assertion| match assertion {
            TableAssertion::UniqueIds { target, .. } => vec![target.as_str()],
            TableAssertion::AllPairsUnequal {
                left_target,
                right_target,
                ..
            } => vec![left_target.as_str(), right_target.as_str()],
        })
        .collect();
    let model_exclude = ExcludeSet::compile_validated(&model.exclude);
    let mut context = ScanContext::default();
    let rows: BTreeMap<&str, Vec<ScannedRow>> = selected
        .iter()
        .filter_map(|name| {
            let target = targets.get(name).copied()?;
            let exclude = ExcludeSet::compile_validated(&target.exclude);
            Some((
                *name,
                declared_tables::scan(
                    spec,
                    root,
                    declared_tables::DeclaredScope {
                        name: &target.name,
                        archetype: &target.archetype,
                        exclude: &exclude,
                        model_exclude: &model_exclude,
                        mints: target.id_columns(),
                        status_column: None,
                        kind_column: None,
                        target_required: target.required,
                    },
                    target.section.as_ref()?,
                    &mut context,
                ),
            ))
        })
        .collect();

    let severity = registry.grammar_severity();
    for assertion in &model.table_assertions {
        match assertion {
            TableAssertion::UniqueIds { name, target } => {
                let Some(target_rows) = rows.get(target.as_str()) else {
                    continue;
                };
                let Some(declaration) = targets.get(target.as_str()).copied() else {
                    continue;
                };
                let Some(id_column) = declaration.id_column() else {
                    continue;
                };
                let mut seen: BTreeMap<(String, String), usize> = BTreeMap::new();
                for row in target_rows {
                    let Some(id) = row.cell(id_column) else {
                        continue;
                    };
                    let document = declared_tables::relative_path(root, &row.path);
                    if let Some(first_line) = seen.get(&(document.clone(), id.to_string())) {
                        report.route(
                            severity,
                            posture_tier(posture),
                            BundleFinding::in_pack(
                                pack::TRACE,
                                "table-assertion-failed",
                                row.path.clone(),
                                format!(
                                    "table assertion '{name}' found duplicate id '{id}' in target '{target}' (first at line {first_line})"
                                ),
                            )
                            .at_line(Some(row.line)),
                        );
                    } else {
                        seen.insert((document, id.to_string()), row.line);
                    }
                }
            }
            TableAssertion::AllPairsUnequal {
                name,
                left_target,
                left_column,
                right_target,
                right_column,
            } => {
                let (Some(left_rows), Some(right_rows)) = (
                    rows.get(left_target.as_str()),
                    rows.get(right_target.as_str()),
                ) else {
                    continue;
                };
                let left_id_column = targets
                    .get(left_target.as_str())
                    .and_then(|target| target.id_column());
                let right_id_column = targets
                    .get(right_target.as_str())
                    .and_then(|target| target.id_column());
                for left in left_rows {
                    let Some(left_value) = left.cell(left_column) else {
                        continue;
                    };
                    for right in right_rows {
                        let Some(right_value) = right.cell(right_column) else {
                            continue;
                        };
                        if left_value != right_value {
                            continue;
                        }
                        let left_id = left_id_column
                            .and_then(|column| left.cell(column))
                            .unwrap_or("(no row id)");
                        let right_id = right_id_column
                            .and_then(|column| right.cell(column))
                            .unwrap_or("(no row id)");
                        report.route(
                            severity,
                            posture_tier(posture),
                            BundleFinding::in_pack(
                                pack::TRACE,
                                "table-assertion-failed",
                                right.path.clone(),
                                format!(
                                    "table assertion '{name}' found equal values in {left_target} row '{left_id}' and {right_target} row '{right_id}'"
                                ),
                            )
                            .at_line(Some(right.line)),
                        );
                    }
                }
            }
        }
    }
}
