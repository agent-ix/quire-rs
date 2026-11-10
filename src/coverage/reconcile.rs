//! Reconcile declared targets and references with the bound symbol graph.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::Path;

use crate::corpus::declared_tables;
use crate::corpus::spec::Spec;
use crate::registry::Registry;
use crate::symbols::trace::{IdKinds, OutsideMarkerId, SymbolGraph, VerifiesRelation};
use crate::traceability::{StatusClass, TraceTargetEvidence, TraceabilityModel};

use super::binding_diagnostics::{
    binding_diagnostics, legacy_implements_diagnostics, minted_child_diagnostics,
    near_miss_diagnostics, outside_marker_diagnostics, shell_finding_diagnostics,
};
use super::{
    relative, CoverageDiagnostic, CoverageReport, CoverageTotals, GroupCounts, ImplementsRecord,
    MintedTargetRecord, NoSymbolRow, RetiredRow, SharedTraceId, SharedTraceSymbol, StatusLie,
    UnbackedRow, UndeclaredStatus, UntrackedSymbol, COVERAGE_DIAGNOSTIC_REASONS,
};

/// Rows with their own status cell that reference one id.
struct StatusRefs {
    rows: usize,
    all_exempt: bool,
}

impl Default for StatusRefs {
    fn default() -> Self {
        Self {
            rows: 0,
            all_exempt: true,
        }
    }
}

/// Whether a status cell is `planned`, `blocked` or `parked` (FR-050-AC-64):
/// ASCII case-insensitive, leading non-ASCII-alphanumeric characters ignored,
/// whole word only. The words are fixed by the engine and never consult the
/// status vocabulary.
fn is_strict_exempt_status(status: &str) -> bool {
    const WORDS: [&str; 3] = ["planned", "blocked", "parked"];
    let bytes = status.trim().as_bytes();
    let start = bytes
        .iter()
        .position(u8::is_ascii_alphanumeric)
        .unwrap_or(bytes.len());
    let rest = &bytes[start..];
    WORDS.iter().any(|word| {
        let n = word.len();
        rest.len() >= n
            && rest[..n].eq_ignore_ascii_case(word.as_bytes())
            && rest.get(n).is_none_or(|b| !b.is_ascii_alphanumeric())
    })
}

pub(super) fn reconcile(
    spec: &Spec,
    model: &TraceabilityModel,
    graph: &SymbolGraph,
    root: &Path,
    registry: &Registry,
    evidence: Option<&crate::evidence::ReconciledEvidence<'_>>,
) -> (CoverageReport, declared_tables::MintingCensus) {
    // FR-053: derived here, ahead of the untracked-symbols reconciliation
    // below, because CR-187 R2 makes an obligation's own id count as
    // declared — a tag naming one must not land in `untracked_symbols`, and
    // that filter needs the id set before it runs.
    // Evidence mode consumes only the native selected entries and cause report.
    // Legacy derivation would scan unselected sources and leak their skipped rows.
    let (obligations, skipped) = match evidence {
        Some(evidence) => (
            evidence
                .input
                .inventory
                .entries()
                .map(|entry| entry.obligation().clone())
                .collect(),
            Vec::new(),
        ),
        None => crate::obligation::derive(spec, root, model),
    };
    let obligation_ids: BTreeSet<&str> = obligations.iter().map(|o| o.id.as_str()).collect();
    // CR-060: compiled once for the whole reconciliation — every declaration
    // is scoped by it.
    let model_exclude = declared_tables::ExcludeSet::compile_validated(&model.exclude);

    // ── Minted targets, grouped by their minting document ──
    let mut ctx = declared_tables::ScanContext::default();
    let mut minted: Vec<MintedTargetRecord> = Vec::new();
    let mut minted_by_target: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    // Reference integrity and source-evidence coverage deliberately have
    // different populations. Every declared target id resolves references,
    // while only source targets enter the coverage denominator (#363).
    let mut declared_ids: BTreeSet<String> = BTreeSet::new();
    for target in &model.trace_targets {
        let mut reference_only_ctx = declared_tables::ScanContext::default();
        let target_ctx = if target.evidence == TraceTargetEvidence::ReferenceOnly {
            &mut reference_only_ctx
        } else {
            &mut ctx
        };
        for minted_id in declared_tables::mint(spec, root, target, &model_exclude, target_ctx) {
            declared_ids.insert(minted_id.id.clone());
            minted_by_target
                .entry(target.name.as_str())
                .or_default()
                .insert(minted_id.id.clone());
            if target.evidence == TraceTargetEvidence::ReferenceOnly {
                continue;
            }
            minted.push(MintedTargetRecord {
                document: relative(root, &minted_id.path),
                id: minted_id.id,
                target: target.name.clone(),
                line: minted_id.line,
                backed: false,
                retired: false,
            });
        }
    }
    // Cache reference rows once so row-aware backing and the ordinary
    // reconciliation below use the same parsed rows.
    let reference_rows = scan_reference_rows(spec, root, model, &model_exclude, &mut ctx);
    let row_bindings = super::row_binding::RowBindings::from_scanned_rows(
        model,
        &reference_rows,
        &minted_by_target,
    );
    let (verifies, mut row_diagnostics) = row_bindings.eligible_relations(graph);
    let backed: BTreeSet<&str> = verifies
        .iter()
        .map(|relation| relation.trace_id.as_str())
        .collect();
    for record in &mut minted {
        record.backed = backed.contains(record.id.as_str())
            || evidence.is_some_and(|evidence| {
                evidence.backs_minted(&record.document, &record.id, record.line, &record.target)
            });
    }
    minted.sort_by(|a, b| {
        (&a.target, &a.document, &a.id, a.line).cmp(&(&b.target, &b.document, &b.id, b.line))
    });

    // ── Reference rows: unbacked rows and status lies ──
    let mut unbacked_rows: Vec<UnbackedRow> = Vec::new();
    let mut status_lies: Vec<StatusLie> = Vec::new();
    let mut retired_rows: Vec<RetiredRow> = Vec::new();
    let mut retired_loci: BTreeSet<(String, usize, Option<String>)> = BTreeSet::new();
    let mut no_symbol_rows: Vec<NoSymbolRow> = Vec::new();
    let mut undeclared_statuses: Vec<UndeclaredStatus> = Vec::new();
    let mut referenced_ids: BTreeSet<String> = BTreeSet::new();
    let mut row_ids: BTreeSet<String> = BTreeSet::new();
    // Row ids of rows that carry a status cell (FR-050-AC-23, CR-087). The
    // one-id-one-symbol policy is scoped to these: a status-classed row backed
    // by any one of N binders stays green while the other N−1 rot, which is
    // the defect. An id whose rows carry no status (an acceptance criterion
    // verified by several tests) is legitimately N:1 and never reported.
    let mut status_row_ids: BTreeSet<String> = BTreeSet::new();
    // Strict exemption (FR-050-AC-64): per referenced id, how many rows with
    // their own status cell reference it and whether all of them are exempt.
    let mut status_refs: BTreeMap<String, StatusRefs> = BTreeMap::new();
    // Unbacked rows with no status cell of their own, resolved from
    // `status_refs` once every row has been read.
    let mut declared_status_pending: Vec<usize> = Vec::new();

    for (declaration, scanned_rows) in model.document_references.iter().zip(&reference_rows) {
        let Some(scanned_rows) = scanned_rows else {
            continue;
        };
        let selected_status = declaration.selected_status_column(model.status.as_ref());
        let Ok(pattern) = regex::Regex::new(&declaration.pattern) else {
            continue; // patterns are validated at module load
        };
        for row in scanned_rows {
            let Some(raw_cell) = row.cell(&declaration.column) else {
                continue;
            };
            // CR-015: same normalization as FR-049, from the shared helper.
            let cell = &declared_tables::normalize_reference_cell(
                raw_cell,
                declaration.strip_annotations,
                declaration.expand_ranges,
            );
            let row_id = declaration
                .row_id_column
                .as_deref()
                .and_then(|c| row.cell(c))
                .map(str::to_string);
            if let Some(id) = &row_id {
                row_ids.insert(id.clone());
            }
            let mut ids: Vec<String> = pattern
                .captures_iter(cell)
                .filter_map(|c| c.get(1))
                .map(|m| m.as_str().trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            referenced_ids.extend(ids.iter().cloned());

            // Keep all referenced ids in the diagnostic, even when an
            // obligation requires evidence for its own identity.
            let mut answerable = ids.clone();
            if let Some(id) = &row_id {
                answerable.push(id.clone());
            }
            answerable.sort();
            answerable.dedup();
            ids.sort();
            ids.dedup();
            let obligation_references: Vec<_> = answerable
                .iter()
                .filter(|id| obligation_ids.contains(id.as_str()))
                .collect();
            let source_backed = match row_id.as_deref() {
                Some(id) if obligation_ids.contains(id) => backed.contains(id),
                _ if !obligation_references.is_empty() => obligation_references
                    .iter()
                    .all(|id| backed.contains(id.as_str())),
                _ => answerable.iter().any(|id| backed.contains(id.as_str())),
            };
            let is_backed = source_backed
                || evidence.is_some_and(|evidence| {
                    row_id.as_deref().is_some_and(|id| {
                        evidence.backs_row(&relative(root, &row.path), id, row.line)
                    })
                });
            let document = relative(root, &row.path);

            // CR-083: an undeclared status is classified **above** the backed
            // early-continue, and that placement is the whole point. Vocabulary
            // drift is a property of the declaration, not of the row's evidence:
            // a backstop that only ever sees unbacked rows is a by-product, not
            // a backstop. `class_of` has always returned `Unknown` here; until
            // now nothing asked.
            if let (Some(status), Some(selected)) = (&model.status, selected_status) {
                if let Some(value) = row.cell(selected.column()) {
                    if let Some(id) = &row_id {
                        status_row_ids.insert(id.clone());
                    }
                    if status.class_of(value) == StatusClass::Unknown {
                        undeclared_statuses.push(UndeclaredStatus {
                            reference: declaration.name.clone(),
                            document: document.clone(),
                            row_id: row_id.clone(),
                            status: value.to_string(),
                            line: Some(row.line),
                        });
                    }
                }
            }

            let own_exempt = selected_status
                .and_then(|selected| row.cell(selected.column()))
                .map(is_strict_exempt_status);
            if let Some(exempt) = own_exempt {
                for id in &answerable {
                    let refs = status_refs.entry(id.clone()).or_default();
                    refs.rows += 1;
                    refs.all_exempt &= exempt;
                }
            }

            let retirement = row
                .cell(&declaration.column)
                .and_then(|value| {
                    model
                        .vocabularies
                        .is_retired(cell)
                        .then_some((declaration.column.as_str(), value))
                })
                .or_else(|| {
                    let status = model.status.as_ref()?;
                    let column = selected_status?.column();
                    let value = row.cell(column)?;
                    (status.class_of(value) == StatusClass::Retired).then_some((column, value))
                });
            if let Some((column, value)) = retirement {
                retired_loci.insert((document.clone(), row.line, row_id.clone()));
                retired_rows.push(RetiredRow {
                    reference: declaration.name.clone(),
                    document,
                    row_id,
                    target_ids: answerable,
                    line: row.line,
                    column: column.to_string(),
                    value: value.to_string(),
                    cells: row.cells.clone(),
                });
                continue;
            }

            if is_backed {
                continue;
            }

            if own_exempt.is_none() && row_id.is_some() {
                declared_status_pending.push(unbacked_rows.len());
            }
            unbacked_rows.push(UnbackedRow {
                reference: declaration.name.clone(),
                document: document.clone(),
                row_id: row_id.clone(),
                target_ids: answerable.clone(),
                line: Some(row.line),
                strict_exempt: own_exempt.unwrap_or(false),
            });

            // A method that mints no symbol leaves a factually unbacked row but
            // cannot be a status lie. Check both the configured type column and
            // the declaration's own method column after stripping annotations
            // (FR-050-AC-16; CR-041 and #259).
            let mints_no_symbol = |value: &str| {
                let bare = declared_tables::normalize_reference_cell(value, true, false);
                model.vocabularies.mints_no_symbol(bare.trim())
            };
            let exempting_type = model
                .vocabularies
                .test_type_column
                .as_deref()
                .and_then(|column| row.cell(column))
                .filter(|value| mints_no_symbol(value))
                .or_else(|| {
                    row.cell(&declaration.column)
                        .filter(|value| mints_no_symbol(value))
                });
            if let Some(test_type) = exempting_type {
                no_symbol_rows.push(NoSymbolRow {
                    reference: declaration.name.clone(),
                    document,
                    row_id,
                    test_type: test_type.to_string(),
                    target_ids: answerable,
                    line: Some(row.line),
                });
                continue;
            }

            // A status that classes `complete` over an unbacked row is a lie.
            if let (Some(status), Some(selected)) = (&model.status, selected_status) {
                if let Some(value) = row.cell(selected.column()) {
                    if status.class_of(value) == StatusClass::Complete {
                        status_lies.push(StatusLie {
                            reference: declaration.name.clone(),
                            document,
                            row_id,
                            status: value.to_string(),
                            target_ids: answerable,
                            line: Some(row.line),
                        });
                    }
                }
            }
        }
    }

    for index in declared_status_pending {
        let exempt = unbacked_rows[index]
            .row_id
            .as_ref()
            .and_then(|id| status_refs.get(id))
            .is_some_and(|refs| refs.rows > 0 && refs.all_exempt);
        unbacked_rows[index].strict_exempt = exempt;
    }
    let strict_exempt_unbacked = unbacked_rows.iter().filter(|row| row.strict_exempt).count();

    // ── Untracked symbols: a trace tag pointing at nothing declared ──
    let mut untracked_symbols: Vec<UntrackedSymbol> = graph
        .verifies
        .iter()
        .filter(|relation| {
            !declared_ids.contains(&relation.trace_id)
                && !referenced_ids.contains(&relation.trace_id)
                && !row_ids.contains(&relation.trace_id)
                // CR-187 R2: an obligation's own id — including one minted
                // only as an obligation (an NFR-metric id, a combinatorial
                // source's rendered id) — counts as declared exactly like a
                // minted trace target's id already does.
                && !obligation_ids.contains(relation.trace_id.as_str())
        })
        .map(|relation| UntrackedSymbol {
            path: relation.path.clone(),
            symbol: relation.symbol.clone(),
            trace_id: relation.trace_id.clone(),
            line: Some(relation.line),
        })
        .collect();

    let mut counts: BTreeMap<(String, String), (usize, usize, usize)> = BTreeMap::new();
    for entry in &mut minted {
        entry.retired = retired_at(&retired_loci, &entry.document, entry.line, &entry.id);
        let count = counts
            .entry((entry.document.clone(), entry.target.clone()))
            .or_default();
        if entry.retired {
            count.2 += 1;
        } else {
            count.1 += 1;
            count.0 += usize::from(entry.backed);
        }
    }
    retired_rows.sort_by(|a, b| {
        (&a.reference, &a.document, &a.row_id, a.line).cmp(&(
            &b.reference,
            &b.document,
            &b.row_id,
            b.line,
        ))
    });
    retired_rows.dedup();

    let groups: Vec<GroupCounts> = counts
        .into_iter()
        .map(
            |((document, target), (backed, total, retired))| GroupCounts {
                document,
                target,
                backed,
                total,
                retired,
            },
        )
        .collect();
    let totals = CoverageTotals {
        strict_exempt_unbacked,
        backed: groups.iter().map(|g| g.backed).sum(),
        total: groups.iter().map(|g| g.total).sum(),
        retired: groups.iter().map(|g| g.retired).sum(),
        ..CoverageTotals::default()
    };

    unbacked_rows.sort_by(|a, b| {
        (&a.reference, &a.document, &a.row_id, &a.target_ids).cmp(&(
            &b.reference,
            &b.document,
            &b.row_id,
            &b.target_ids,
        ))
    });
    status_lies.sort_by(|a, b| {
        (&a.reference, &a.document, &a.row_id).cmp(&(&b.reference, &b.document, &b.row_id))
    });
    no_symbol_rows.sort_by(|a, b| {
        (&a.reference, &a.document, &a.row_id).cmp(&(&b.reference, &b.document, &b.row_id))
    });
    undeclared_statuses.sort_by(|a, b| {
        (&a.reference, &a.document, &a.row_id, &a.status).cmp(&(
            &b.reference,
            &b.document,
            &b.row_id,
            &b.status,
        ))
    });
    // Two identical matching rows in one document are one defect, not two
    // records — mirrors `untracked_symbols` below (#213). Compared without
    // `line` (#210): the duplicates sit on different lines by definition, and
    // letting the line distinguish them would quietly reopen CR-086. The
    // sort above is stable, so the surviving record is the first — lowest —
    // line.
    undeclared_statuses.dedup_by(|a, b| {
        a.reference == b.reference
            && a.document == b.document
            && a.row_id == b.row_id
            && a.status == b.status
    });
    untracked_symbols
        .sort_by(|a, b| (&a.path, &a.symbol, &a.trace_id).cmp(&(&b.path, &b.symbol, &b.trace_id)));
    untracked_symbols.dedup();

    // FR-063-AC-7: read before `into_diagnostics` consumes the context. The
    // metric is built in `compute`, where every other one is.
    let minting_census = ctx.census();

    // CR-054: declarations that selected nothing, rendered from the one
    // shared vocabulary `quire validate` also reports them under. Already
    // sorted by `into_diagnostics`, so the order is a property of the model.
    let mut diagnostics: Vec<CoverageDiagnostic> = ctx
        .into_diagnostics()
        .into_iter()
        .map(|(declaration, diagnostic)| {
            let (_, message) = declared_tables::scan_finding(&declaration, &diagnostic, root);
            // CR-117 made good on the note this field carried since CR-062:
            // `archetype-matches-nothing` is declaration-level and still has no
            // document to point at, while the two minting faults name the one
            // file whose heading or header row is wrong.
            let origin = diagnostic
                .document()
                .is_none()
                .then(|| registry.trace_target_origin(&declaration))
                .flatten();
            let path = diagnostic
                .document()
                .map(str::to_string)
                .or_else(|| origin.map(|origin| origin.path.display().to_string()));
            CoverageDiagnostic {
                declaration,
                reason: declared_tables::scan_reason(&diagnostic).to_string(),
                message,
                path,
                line: diagnostic
                    .line()
                    .or_else(|| origin.map(|origin| origin.line)),
                value: None,
                guidance: None,
            }
        })
        .collect();

    // A model declared without a single trace target mints nothing at all, so
    // every ratio it reports is over an empty denominator. `is_empty()` reads
    // it as *declared* (status or trace-tag entries alone are enough), which
    // is why `ModelUndeclared` never fires for it (CR-054).
    if model.trace_targets.is_empty() {
        let origin = registry.traceability_origin();
        diagnostics.insert(
            0,
            CoverageDiagnostic {
                declaration: "traceability".to_string(),
                reason: "model-mints-nothing".to_string(),
                message: "the declared traceability model has no trace_targets, so it \
                          mints no ids and every count is over an empty denominator"
                    .to_string(),
                path: origin.map(|origin| origin.path.display().to_string()),
                line: origin.map(|origin| origin.line),
                value: None,
                guidance: None,
            },
        );
    }

    // CR-093: the binder could not read a language it was pointed at. Ordered
    // ahead of the obligation diagnostics and after the declaration-scan ones
    // because it invalidates more than either: a language that bound nothing
    // makes every unbacked row in it unreadable, not just the rows one
    // declaration selects.
    diagnostics.extend(binding_diagnostics(&graph.binding_census));

    // CR-215: an id written in a comment or docstring, which never binds.
    // Ordered immediately after the census diagnostics: they read a
    // denominator this is the explanation for, since a repository still on
    // textual tags reports its tests as unbound.
    // Only a kind the declared targets mint is a trace id (CR-215): `CR-`,
    // `PLAT-` and a bare `AC-` in a comment are not.
    let id_kinds = IdKinds::from_minted(&declared_ids);
    let outside_marker: Vec<OutsideMarkerId> = graph
        .outside_marker
        .iter()
        .filter(|found| id_kinds.admits(&found.trace_id))
        .cloned()
        .collect();
    diagnostics.extend(outside_marker_diagnostics(&outside_marker, model));

    // FR-051-AC-39, AC-44: a shell marker that matched and bound nothing.
    diagnostics.extend(shell_finding_diagnostics(&graph.shell_findings));

    // PLAT-2: a criterion tag on a test whose declared matrix row does not
    // reference that criterion is visible, but is not evidence for it.
    diagnostics.append(&mut row_diagnostics);

    // #307: the two halves of one mistake, already in the payload, joined at
    // last. Ordered after the census diagnostics for the same reason as #312 —
    // this is the case they cannot see, because from their vantage point the
    // binding worked.
    diagnostics.extend(near_miss_diagnostics(&untracked_symbols, &unbacked_rows));

    // #328: a bound coarse id and the exact child ids the model really mints
    // were already in this reconciliation. Join them into an actionable
    // finding without treating the parent as evidence for every child.
    let source_target_ids: BTreeSet<String> = minted.iter().map(|row| row.id.clone()).collect();
    diagnostics.extend(minted_child_diagnostics(
        &untracked_symbols,
        &source_target_ids,
    ));

    // FR-053-AC-8: a row whose statement cell is empty contributes no record,
    // and the AC says it is skipped **with a diagnostic**. It was not — the
    // second half of `derive`'s return was dropped here, so the only surface
    // that ever named a skipped row was a unit test calling `derive` directly
    // (CR-063). An authoring error nobody is told about is an authoring error
    // nobody fixes.
    diagnostics.extend(skipped.iter().map(|row| CoverageDiagnostic {
        declaration: row.source.clone(),
        reason: "obligation-row-states-nothing".to_string(),
        message: format!(
            "row {} of '{}' states no obligation: the source's statement column \
             is empty, so the row mints no record",
            row.row, row.document
        ),
        path: Some(row.document.clone()),
        line: None,
        value: None,
        guidance: None,
    }));

    // CR-187, FR-050-AC-50: a range written inside a source marker or a
    // textual `implements` use binds no id and is reported once per
    // occurrence, advisory (FR-057 `warning`), naming the file, the line, the
    // qualified symbol and the literal range text.
    diagnostics.extend(
        graph
            .range_diagnostics
            .iter()
            .map(|range| CoverageDiagnostic {
                declaration: range.symbol.clone(),
                reason: "range-in-trace-tag".to_string(),
                message: format!(
                    "`{}` in {}:{} names a range, which binds no id — including its own \
             endpoints; write one trace tag naming each id explicitly",
                    range.range_text, range.path, range.line
                ),
                path: Some(range.path.clone()),
                line: Some(range.line),
                value: Some(range.range_text.clone()),
                guidance: None,
            }),
    );

    diagnostics.extend(graph.partial_attribute_matches.iter().map(|finding| CoverageDiagnostic {
        declaration: finding.form.clone(),
        reason: "partial-implements-attribute-match".to_string(),
        message: format!("form '{}' matched only part of attribute {} on {} at {}:{}; match the complete attribute", finding.form, finding.attribute, finding.symbol, finding.path, finding.line),
        path: Some(finding.path.clone()), line: Some(finding.line),
        value: Some(finding.attribute.clone()), guidance: None,
    }));

    diagnostics.extend(graph.partial_syntax_matches.iter().map(|finding| CoverageDiagnostic {
        declaration: finding.form.clone(),
        reason: "partial-implements-syntax-match".to_string(),
        message: format!("form '{}' matched only part of {} on {} at {}:{}; match the complete candidate", finding.form, finding.attribute, finding.symbol, finding.path, finding.line),
        path: Some(finding.path.clone()), line: Some(finding.line),
        value: Some(finding.attribute.clone()), guidance: None,
    }));

    diagnostics.extend(graph.unattached_syntax.iter().map(|finding| CoverageDiagnostic {
        declaration: finding.form.clone(),
        reason: "implements-syntax-unattached".to_string(),
        message: format!("form '{}' names {} at {}:{}, but no wrapper chain attaches it to a const, let or var declarator, so it binds nothing", finding.form, finding.candidate, finding.path, finding.line),
        path: Some(finding.path.clone()), line: Some(finding.line),
        value: Some(finding.candidate.clone()), guidance: None,
    }));

    diagnostics.extend(legacy_implements_diagnostics(
        &graph.legacy_implements,
        model,
    ));

    // ── Shared trace ids: one id bound by several distinct symbols ──
    // (FR-050-AC-23, CR-087). Scoped to ids that are row ids of
    // status-carrying rows — the population where N binders let a green row
    // rot N−1 tests deep. Grouped through BTree collections so the record
    // order — and inside each record the symbol order — is a property of the
    // data, not of the walk (NFR-006). Distinctness is `(path, symbol)`: the
    // same symbol binding one id through two declared forms (or a duplicated
    // in-symbol id, FR-051's dedup) is one binder, not a shared id.
    let mut binders: BTreeMap<&str, BTreeSet<(&str, &str)>> = BTreeMap::new();
    for relation in &verifies {
        if !status_row_ids.contains(&relation.trace_id) {
            continue;
        }
        binders
            .entry(relation.trace_id.as_str())
            .or_default()
            .insert((relation.path.as_str(), relation.symbol.as_str()));
    }
    let shared_trace_ids: Vec<SharedTraceId> = binders
        .into_iter()
        .filter(|(_, symbols)| symbols.len() > 1)
        .map(|(trace_id, symbols)| SharedTraceId {
            trace_id: trace_id.to_string(),
            symbols: symbols
                .into_iter()
                .map(|(path, symbol)| SharedTraceSymbol {
                    path: path.to_string(),
                    symbol: symbol.to_string(),
                })
                .collect(),
        })
        .collect();

    // CR-187: the computed coverage_matrix, built from the obligation
    // population alone (FR-050-AC-47..49) — never the trace-target set.
    let retired_obligations =
        retired_obligation_flags(spec, root, model, &obligations, &retired_loci);
    let matrix = coverage_matrix(
        &obligations,
        &verifies,
        model,
        &retired_obligations,
        evidence,
    );
    diagnostics.extend(super::diagnostics::uncatalogued_methods(
        &obligations,
        registry,
        &retired_obligations,
    ));

    let report = CoverageReport {
        unbacked_rows,
        status_lies,
        retired_rows,
        no_symbol_rows,
        undeclared_statuses,
        untracked_symbols,
        shared_trace_ids,
        groups,
        minted_targets: minted,
        // CR-028: filled by `compute`, which holds the `Registry` this
        // reconciliation deliberately does not take.
        criteria: Vec::new(),
        // FR-059-AC-9 (CR-091): likewise filled by `compute` — the vocabulary
        // lives in an archetype's frontmatter schema, which only the
        // `Registry` can read.
        vocabulary_coverage: Vec::new(),
        diagnostics,
        diagnostic_reason_registry: COVERAGE_DIAGNOSTIC_REASONS
            .iter()
            .map(|reason| (*reason).to_string())
            .collect(),
        obligations,
        coverage_matrix: matrix,
        evidence_results: evidence.map(|evidence| evidence.results.clone()),
        record_binders: evidence.map(|evidence| evidence.all_binders()),
        inventory_causes: evidence.map(|evidence| {
            evidence
                .input
                .inventory
                .causes()
                .iter()
                .map(|cause| crate::evidence::InventoryCauseReport {
                    code: cause.code(),
                    source: cause.source_name().map(str::to_string),
                    document: cause
                        .document()
                        .map(|path| path.to_string_lossy().into_owned()),
                    row: cause.row(),
                    line: cause.line(),
                    detail: format!("{:?}", cause.detail()),
                })
                .collect()
        }),
        // FR-062. Carried through to the JSON so a consumer can scope work by
        // requirement; deliberately NOT folded into `totals`, `backed` or
        // `untracked_symbols` — scope is not evidence.
        implements: graph
            .implements
            .iter()
            .map(|relation| ImplementsRecord {
                path: relation.path.clone(),
                symbol: relation.symbol.clone(),
                trace_id: relation.trace_id.clone(),
                form: relation.form.clone(),
            })
            .collect(),
        // FR-050-AC-24 (#215): what the declared `source_exclude` subtracted,
        // carried by the graph because this reconciliation never sees the
        // extraction itself.
        excluded_source_files: graph.excluded_source_files,
        // FR-050-AC-27 (CR-093): the premise, carried whether or not it holds.
        binding_census: graph.binding_census.clone(),
        // FR-050-AC-39 (#362): engine-owned annotation parsing, exposed so a
        // census can join an unread authored id to its matrix row without
        // reimplementing the language adapters.
        unmatched_tags: graph
            .unmatched_tags
            .iter()
            .filter(|tag| id_kinds.admits(&tag.trace_id))
            .cloned()
            .collect(),
        // FR-063: filled by `compute`, which is where the criteria totals one
        // of these metrics describes are set.
        metrics: Vec::new(),
        // FR-064: carried by the graph, because the binder is where the
        // extraction and the symbol kinds are both in hand.
        suspicions: graph.suspicions.clone(),
        totals,
    };
    (report, minting_census)
}

fn scan_reference_rows(
    spec: &Spec,
    root: &Path,
    model: &TraceabilityModel,
    model_exclude: &declared_tables::ExcludeSet,
    ctx: &mut declared_tables::ScanContext,
) -> Vec<Option<Vec<declared_tables::ScannedRow>>> {
    model
        .document_references
        .iter()
        .map(|declaration| {
            regex::Regex::new(&declaration.pattern).ok()?;
            let exclude = declared_tables::ExcludeSet::compile_validated(&declaration.exclude);
            let selected_status = declaration.selected_status_column(model.status.as_ref());
            Some(declared_tables::scan(
                spec,
                root,
                declared_tables::DeclaredScope {
                    name: &declaration.name,
                    archetype: &declaration.archetype,
                    exclude: &exclude,
                    model_exclude,
                    mints: None,
                    status_column: selected_status,
                    target_required: false,
                },
                &declaration.section,
                ctx,
            ))
        })
        .collect()
}

/// Build `coverage_matrix` (CR-187, FR-050-AC-47..49): the derived obligation
/// population, grouped by its own `document` field, each criterion carrying
/// its binding test symbols and a computed static status.
///
/// Population is the obligation set alone — never a minted trace target by
/// itself (FR-050-AC-47): `test-case`, `suite` and `inspection` targets never
/// become a criterion, only a module's declared `obligations:` sources decide
/// what counts.
fn coverage_matrix(
    obligations: &[crate::obligation::Obligation],
    verifies: &[&VerifiesRelation],
    model: &TraceabilityModel,
    retired: &[bool],
    evidence: Option<&crate::evidence::ReconciledEvidence<'_>>,
) -> Vec<super::CoverageMatrixRequirement> {
    use super::{
        CoverageMatrixBinder, CoverageMatrixCriterion, CoverageMatrixRequirement,
        CoverageMatrixStatus,
    };

    // Binders per obligation id, deduped by (path, line, column) — the
    // binding symbol's own declaration site, never its qualified name alone:
    // FR-051-AC-21 makes a TypeScript registration's qualified name ignorant
    // of its enclosing suite, so two identically-titled `it(...)` calls in
    // different `describe` blocks are two distinct sites that must not
    // collapse into one binder.
    let mut binders_by_id: BTreeMap<&str, BTreeMap<(&str, usize, usize), CoverageMatrixBinder>> =
        BTreeMap::new();
    for relation in verifies {
        binders_by_id
            .entry(relation.trace_id.as_str())
            .or_default()
            .entry((relation.path.as_str(), relation.line, relation.column))
            .or_insert_with(|| CoverageMatrixBinder {
                path: relation.path.clone(),
                line: relation.line,
                column: relation.column,
                qualified_name: relation.symbol.clone(),
                kind: relation.kind.as_str().to_string(),
                ignored: relation.ignored,
            });
    }

    // Grouped by document while preserving `obligations`' own order: that
    // order is already (declaration order, document path, row ordinal), so
    // extending each document's group in encounter order is exactly
    // "restricted to that document, in declaration order" (FR-050-AC-48). The
    // final `BTreeMap` sorts the groups themselves by `document`, without
    // disturbing the order criteria were pushed within each one.
    let mut by_document: BTreeMap<&str, Vec<CoverageMatrixCriterion>> = BTreeMap::new();
    for (index, (obligation, retired)) in obligations.iter().zip(retired).enumerate() {
        let record_binders = evidence.map(|evidence| {
            evidence
                .input
                .inventory
                .entries()
                .nth(index)
                .map(|entry| evidence.binders(entry.identity().into()))
                .unwrap_or_default()
        });
        let mut binders: Vec<CoverageMatrixBinder> = binders_by_id
            .get(obligation.id.as_str())
            .map(|m| m.values().cloned().collect())
            .unwrap_or_default();
        binders.sort_by(|a, b| {
            (&a.path, a.line, a.column, &a.qualified_name, &a.kind).cmp(&(
                &b.path,
                b.line,
                b.column,
                &b.qualified_name,
                &b.kind,
            ))
        });

        // A live criterion's method exemption takes precedence over binder count.
        let status = if *retired {
            CoverageMatrixStatus::Retired
        } else if record_binders
            .as_ref()
            .is_some_and(|binders| !binders.is_empty())
        {
            CoverageMatrixStatus::QualifiedRecord
        } else if obligation
            .method
            .as_deref()
            .is_some_and(|m| model.vocabularies.mints_no_symbol(m))
        {
            CoverageMatrixStatus::MethodWithoutSymbol
        } else if binders.is_empty() {
            CoverageMatrixStatus::Untagged
        } else if binders.iter().all(|b| b.ignored) {
            CoverageMatrixStatus::TaggedByIgnoredTest
        } else {
            CoverageMatrixStatus::Tagged
        };

        let criterion = CoverageMatrixCriterion {
            id: obligation.id.clone(),
            statement: obligation.statement.clone(),
            method: obligation.method.clone(),
            binders,
            record_binders,
            identity: evidence.and_then(|evidence| {
                evidence.input.inventory.entries().nth(index).map(|entry| {
                    let mut locus = crate::evidence::TargetLocus::from(entry.identity());
                    locus.line = entry.line();
                    locus
                })
            }),
            status,
        };
        by_document
            .entry(obligation.document.as_str())
            .or_default()
            .push(criterion);
    }

    // FR-050-AC-48: requirements ordered by `document` — the same path every
    // other list on this report already sorts by. The `BTreeMap` sorts the
    // groups themselves; each group's own `Vec` keeps the push order above,
    // which is `obligations`' own (declaration order, row ordinal) restricted
    // to that document.
    by_document
        .into_iter()
        .map(|(document, criteria)| CoverageMatrixRequirement {
            document: document.to_string(),
            criteria,
        })
        .collect()
}

/// Whether the declaring row is retired, matching an own id when one is stated.
fn retired_at(
    retired: &BTreeSet<(String, usize, Option<String>)>,
    document: &str,
    line: usize,
    id: &str,
) -> bool {
    retired.contains(&(document.to_string(), line, Some(id.to_string())))
        || retired.contains(&(document.to_string(), line, None))
}

/// Join retirement to selected obligation rows without changing their population.
fn retired_obligation_flags(
    spec: &Spec,
    root: &Path,
    model: &TraceabilityModel,
    obligations: &[crate::obligation::Obligation],
    retired: &BTreeSet<(String, usize, Option<String>)>,
) -> Vec<bool> {
    if retired.is_empty() {
        return vec![false; obligations.len()];
    }
    type RowIdentity = (String, String, String);
    let mut rows: BTreeMap<RowIdentity, VecDeque<bool>> = BTreeMap::new();
    let retired_rows: BTreeSet<_> = retired
        .iter()
        .map(|(document, line, _)| (document.as_str(), *line))
        .collect();
    let model_exclude = declared_tables::ExcludeSet::compile_validated(&model.exclude);
    let mut ctx = declared_tables::ScanContext::default();
    for source in &model.obligations {
        // An aggregate obligation has no single declaring row to retire.
        if source.combinatorial.is_some() {
            continue;
        }
        let target = source.target.as_ref().and_then(|name| {
            model
                .trace_targets
                .iter()
                .find(|target| &target.name == name)
        });
        let (archetype, section, id_column) = match target {
            // Load validation admits only table-row targets here.
            Some(target) => match target.id_column() {
                Some(id_column) => (target.archetype.as_str(), &target.section, Some(id_column)),
                None => continue,
            },
            None => {
                let (Some(archetype), Some(section)) =
                    (source.archetype.as_deref(), source.section.as_ref())
                else {
                    continue;
                };
                (archetype, section, None)
            }
        };
        let exclude = declared_tables::ExcludeSet::compile_validated(&source.exclude);
        for row in declared_tables::scan(
            spec,
            root,
            declared_tables::DeclaredScope {
                name: &source.name,
                archetype,
                exclude: &exclude,
                model_exclude: &model_exclude,
                mints: None,
                status_column: None,
                target_required: false,
            },
            section,
            &mut ctx,
        ) {
            let Some(statement) = row.cell(&source.statement_column) else {
                continue;
            };
            if id_column.is_some_and(|column| row.cell(column).is_none()) {
                continue;
            }
            // Derivation preserves relative row order for each source/document.
            // Keep repeated statements queued rather than collapsing duplicate ids.
            let document = relative(root, &row.path);
            let is_retired = match id_column.and_then(|column| row.cell(column)) {
                Some(id) => retired_at(retired, &document, row.line, id),
                // A direct source renders its obligation id. The retirement
                // fact belongs to this declaring row, not that rendered id.
                None => retired_rows.contains(&(document.as_str(), row.line)),
            };
            rows.entry((source.name.clone(), document, statement.to_string()))
                .or_default()
                .push_back(is_retired);
        }
    }
    obligations
        .iter()
        .map(|obligation| {
            rows.get_mut(&(
                obligation.source.clone(),
                obligation.document.clone(),
                obligation.statement.clone(),
            ))
            .and_then(VecDeque::pop_front)
            .unwrap_or(false)
        })
        .collect()
}
