//! Select trace relations that provide evidence for declared matrix rows.

use std::collections::{BTreeMap, BTreeSet};

use crate::corpus::declared_tables::ScannedRow;
use crate::symbols::trace::{SymbolGraph, VerifiesRelation};
use crate::traceability::{DocumentReference, TraceabilityModel};

#[derive(Debug, Default)]
pub(super) struct RowBindings {
    constrained_ids: BTreeSet<String>,
    rows_by_id: BTreeMap<String, BTreeSet<String>>,
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

        let mut declared: Vec<(&DocumentReference, &str, &[ScannedRow])> = Vec::new();
        for (declaration, rows) in model.document_references.iter().zip(scanned) {
            let Some(row_id_column) = declaration.row_id_column.as_deref() else {
                continue;
            };
            // A repo that declares no rows for this archetype has no matrix to
            // bind against, so its targets stay unconstrained.
            let Some(rows) = rows.as_deref().filter(|rows| !rows.is_empty()) else {
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
            let target_names: Vec<&str> = declaration
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
            // Only test-matrix rows constrain: a row id that another declaration
            // cites (an FR's Verification cell naming a test case) is an id a
            // test tags. Rows nothing cites, such as inspections, only widen.
            let is_test_matrix = row_targets.iter().any(|row_target| {
                model.document_references.iter().any(|other| {
                    other.name != declaration.name && other.targets.contains(&row_target.name)
                })
            });
            if !is_test_matrix {
                continue;
            }
            declared.push((declaration, row_id_column, rows));
            for target_name in target_names {
                if let Some(ids) = minted_by_target.get(target_name) {
                    bindings.constrained_ids.extend(ids.iter().cloned());
                }
            }
        }

        for (declaration, row_id_column, rows) in declared {
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

    pub(super) fn row_evidence_relations<'a>(
        &self,
        graph: &'a SymbolGraph,
    ) -> Vec<&'a VerifiesRelation> {
        let mut ids_by_symbol: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
        for relation in &graph.verifies {
            ids_by_symbol
                .entry(relation.symbol_id.as_str())
                .or_default()
                .insert(relation.trace_id.as_str());
        }

        let mut eligible = Vec::with_capacity(graph.verifies.len());
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
                continue;
            };
            if has_matrix_row && rows.iter().any(|row| symbol_ids.contains(row.as_str())) {
                eligible.push(relation);
            }
        }

        eligible
    }
}
