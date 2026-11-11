//! Restrict criterion backing to the source rows declared by a matrix.

use std::collections::{BTreeMap, BTreeSet};

use crate::corpus::declared_tables::ScannedRow;
use crate::symbols::trace::{SymbolGraph, VerifiesRelation};
use crate::traceability::TraceabilityModel;

use super::CoverageDiagnostic;

#[derive(Debug, Default)]
pub(super) struct RowBindings {
    constrained_ids: BTreeSet<String>,
    rows_by_id: BTreeMap<String, BTreeSet<String>>,
    declaration_by_id: BTreeMap<String, String>,
    matrix_row_ids: BTreeSet<String>,
}

impl RowBindings {
    pub(super) fn from_scanned_rows(
        model: &TraceabilityModel,
        scanned: &[Option<Vec<ScannedRow>>],
        minted_by_target: &BTreeMap<&str, BTreeSet<String>>,
    ) -> Self {
        let mut bindings = Self::default();
        let obligation_targets: BTreeSet<&str> = model
            .obligations
            .iter()
            .filter_map(|source| source.target.as_deref())
            .collect();

        for (declaration, rows) in model.document_references.iter().zip(scanned) {
            let Some(row_id_column) = declaration.row_id_column.as_deref() else {
                continue;
            };
            let row_targets: Vec<_> = model
                .trace_targets
                .iter()
                .filter(|target| {
                    target.archetype == declaration.archetype
                        && target.section == declaration.section
                        && target
                            .id_column()
                            .is_some_and(|column| column.eq_ignore_ascii_case(row_id_column))
                })
                .collect();
            if row_targets.is_empty() {
                continue;
            }

            let row_target_names: BTreeSet<&str> = row_targets
                .iter()
                .map(|target| target.name.as_str())
                .collect();
            let target_names: BTreeSet<&str> = declaration
                .targets
                .iter()
                .map(String::as_str)
                .filter(|name| {
                    !row_target_names.contains(name) && obligation_targets.contains(name)
                })
                .collect();
            if target_names.is_empty() {
                continue;
            }
            // A repo that declares no rows for this archetype has no matrix to
            // bind against, so its targets stay unconstrained.
            let Some(rows) = rows.as_ref().filter(|rows| !rows.is_empty()) else {
                continue;
            };

            for target_name in &target_names {
                if let Some(ids) = minted_by_target.get(target_name) {
                    bindings.constrained_ids.extend(ids.iter().cloned());
                    for id in ids {
                        bindings
                            .declaration_by_id
                            .entry(id.clone())
                            .or_insert_with(|| declaration.name.clone());
                    }
                }
            }

            let Ok(pattern) = regex::Regex::new(&declaration.pattern) else {
                continue; // invalid declarations are already rejected at load time
            };
            for row in rows {
                let Some(row_id) = row.cell(row_id_column) else {
                    continue;
                };
                bindings.matrix_row_ids.insert(row_id.to_string());
                let Some(cell) = row.cell(&declaration.column) else {
                    continue;
                };
                let normalized = crate::corpus::declared_tables::normalize_reference_cell(
                    cell,
                    declaration.strip_annotations,
                    declaration.expand_ranges,
                );
                for id in pattern
                    .captures_iter(&normalized)
                    .filter_map(|captures| captures.get(1))
                    .map(|capture| capture.as_str().trim())
                    .filter(|id| !id.is_empty() && bindings.constrained_ids.contains(*id))
                {
                    bindings
                        .rows_by_id
                        .entry(id.to_string())
                        .or_default()
                        .insert(row_id.to_string());
                }
            }
        }

        bindings
    }

    pub(super) fn eligible_relations<'a>(
        &self,
        graph: &'a SymbolGraph,
    ) -> (Vec<&'a VerifiesRelation>, Vec<CoverageDiagnostic>) {
        let mut ids_by_symbol: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
        for relation in &graph.verifies {
            ids_by_symbol
                .entry(relation.symbol_id.as_str())
                .or_default()
                .insert(relation.trace_id.as_str());
        }

        let mut eligible = Vec::with_capacity(graph.verifies.len());
        let mut diagnostics = Vec::new();
        for relation in &graph.verifies {
            let symbol_ids = ids_by_symbol
                .get(relation.symbol_id.as_str())
                .cloned()
                .unwrap_or_default();
            let has_matrix_row = symbol_ids
                .iter()
                .any(|id| self.matrix_row_ids.contains(*id));
            if !self.constrained_ids.contains(&relation.trace_id) {
                eligible.push(relation);
                continue;
            }
            let Some(rows) = self.rows_by_id.get(&relation.trace_id) else {
                diagnostics.push(self.out_of_row_diagnostic(relation, &BTreeSet::new()));
                continue;
            };
            if has_matrix_row && rows.iter().any(|row| symbol_ids.contains(row.as_str())) {
                eligible.push(relation);
            } else {
                diagnostics.push(self.out_of_row_diagnostic(relation, rows));
            }
        }

        (eligible, diagnostics)
    }

    fn out_of_row_diagnostic(
        &self,
        relation: &VerifiesRelation,
        rows: &BTreeSet<String>,
    ) -> CoverageDiagnostic {
        let expected_rows = if rows.is_empty() {
            "no declared matrix row".to_string()
        } else {
            rows.iter()
                .map(|row| format!("`{row}`"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let locus = format!("{}:{}", relation.path, relation.line);
        CoverageDiagnostic {
            declaration: self
                .declaration_by_id
                .get(&relation.trace_id)
                .cloned()
                .unwrap_or_else(|| "traceability.document_references".to_string()),
            reason: "criterion-tagged-outside-row".to_string(),
            message: format!(
                "criterion `{}` is tagged on `{}` at {} but no matrix row cited by that test \
                 references it; expected a same-test tag for one of {expected_rows}. This tag \
                 does not provide backing",
                relation.trace_id, relation.symbol, locus
            ),
            path: Some(relation.path.clone()),
            line: Some(relation.line),
            value: Some(relation.trace_id.clone()),
            guidance: Some(crate::finding::FindingGuidance::remedy(
                format!("criterion `{}`", relation.trace_id),
                locus,
                "correct the matrix reference or tag a test whose declared row references this criterion",
            )),
        }
    }
}
