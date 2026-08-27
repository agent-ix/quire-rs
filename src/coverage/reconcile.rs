//! Reconcile declared targets and references with the bound symbol graph.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::corpus::declared_tables;
use crate::corpus::spec::Spec;
use crate::symbols::trace::SymbolGraph;
use crate::traceability::{StatusClass, TraceabilityModel};

use super::binding_diagnostics::{
    binding_diagnostics, near_miss_diagnostics, non_binding_tag_diagnostics,
};
use super::{
    relative, CoverageDiagnostic, CoverageReport, CoverageTotals, GroupCounts, ImplementsRecord,
    MintedTargetRecord, NoSymbolRow, SharedTraceId, SharedTraceSymbol, StatusLie, UnbackedRow,
    UndeclaredStatus, UntrackedSymbol, COVERAGE_DIAGNOSTIC_REASONS,
};

pub(super) fn reconcile(
    spec: &Spec,
    model: &TraceabilityModel,
    graph: &SymbolGraph,
    root: &Path,
) -> (CoverageReport, declared_tables::MintingCensus) {
    let backed: BTreeSet<&str> = graph.backed_trace_ids();
    // CR-060: compiled once for the whole reconciliation — every declaration
    // is scoped by it.
    let model_exclude = declared_tables::ExcludeSet::compile_validated(&model.exclude);

    // ── Minted targets, grouped by their minting document ──
    let mut ctx = declared_tables::ScanContext::default();
    let mut minted: Vec<MintedTargetRecord> = Vec::new();
    for target in &model.trace_targets {
        let exclude = declared_tables::ExcludeSet::compile_validated(&target.exclude);
        for row in declared_tables::scan(
            spec,
            root,
            declared_tables::DeclaredScope {
                name: &target.name,
                archetype: &target.archetype,
                exclude: &exclude,
                model_exclude: &model_exclude,
                // The minting half of the model: CR-117's two diagnostics are
                // scoped to it, and the id column they name comes from here.
                mints: Some(&target.id_column),
            },
            &target.section,
            &mut ctx,
        ) {
            let Some(id) = row.cell(&target.id_column) else {
                continue;
            };
            minted.push(MintedTargetRecord {
                id: id.to_string(),
                target: target.name.clone(),
                document: relative(root, &row.path),
                line: row.line,
                backed: backed.contains(id),
            });
        }
    }
    minted.sort_by(|a, b| {
        (&a.target, &a.document, &a.id, a.line).cmp(&(&b.target, &b.document, &b.id, b.line))
    });

    let mut counts: BTreeMap<(String, String), (usize, usize)> = BTreeMap::new();
    let mut declared_ids: BTreeSet<String> = BTreeSet::new();
    for entry in &minted {
        declared_ids.insert(entry.id.clone());
        let slot = counts
            .entry((entry.document.clone(), entry.target.clone()))
            .or_insert((0, 0));
        slot.1 += 1;
        if entry.backed {
            slot.0 += 1;
        }
    }

    // ── Reference rows: unbacked rows and status lies ──
    let mut unbacked_rows: Vec<UnbackedRow> = Vec::new();
    let mut status_lies: Vec<StatusLie> = Vec::new();
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

    for declaration in &model.document_references {
        let Ok(pattern) = regex::Regex::new(&declaration.pattern) else {
            continue; // patterns are validated at module load
        };
        let exclude = declared_tables::ExcludeSet::compile_validated(&declaration.exclude);
        for row in declared_tables::scan(
            spec,
            root,
            declared_tables::DeclaredScope {
                name: &declaration.name,
                archetype: &declaration.archetype,
                exclude: &exclude,
                model_exclude: &model_exclude,
                // A reference declaration mints nothing, and its section is
                // legitimately optional — `functional-coverage` reads a heading
                // the matrix template emits only when it has content, so
                // diagnosing its absence would fire on every healthy matrix.
                mints: None,
            },
            &declaration.section,
            &mut ctx,
        ) {
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

            // A row is answerable for its own trace id *and* the ids it
            // references: a matrix row is backed when a test binds the row's
            // own id, an AC row when a test binds the TC it names.
            let mut answerable = ids.clone();
            if let Some(id) = &row_id {
                answerable.push(id.clone());
            }
            answerable.sort();
            answerable.dedup();
            ids.sort();
            ids.dedup();
            let is_backed = answerable.iter().any(|id| backed.contains(id.as_str()));
            let document = relative(root, &row.path);

            // CR-083: an undeclared status is classified **above** the backed
            // early-continue, and that placement is the whole point. Vocabulary
            // drift is a property of the declaration, not of the row's evidence:
            // a backstop that only ever sees unbacked rows is a by-product, not
            // a backstop. `class_of` has always returned `Unknown` here; until
            // now nothing asked.
            if let Some(status) = &model.status {
                if let Some(value) = row.cell(&status.column) {
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

            if is_backed {
                continue;
            }

            unbacked_rows.push(UnbackedRow {
                reference: declaration.name.clone(),
                document: document.clone(),
                row_id: row_id.clone(),
                target_ids: answerable.clone(),
                line: Some(row.line),
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
            if let Some(status) = &model.status {
                if let Some(value) = row.cell(&status.column) {
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

    // ── Untracked symbols: a trace tag pointing at nothing declared ──
    let mut untracked_symbols: Vec<UntrackedSymbol> = graph
        .verifies
        .iter()
        .filter(|relation| {
            !declared_ids.contains(&relation.trace_id)
                && !referenced_ids.contains(&relation.trace_id)
                && !row_ids.contains(&relation.trace_id)
        })
        .map(|relation| UntrackedSymbol {
            path: relation.path.clone(),
            symbol: relation.symbol.clone(),
            trace_id: relation.trace_id.clone(),
            line: Some(relation.line),
        })
        .collect();

    let groups: Vec<GroupCounts> = counts
        .into_iter()
        .map(|((document, target), (backed, total))| GroupCounts {
            document,
            target,
            backed,
            total,
        })
        .collect();
    let totals = CoverageTotals {
        backed: groups.iter().map(|g| g.backed).sum(),
        total: groups.iter().map(|g| g.total).sum(),
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
            let path = diagnostic.document().map(str::to_string);
            CoverageDiagnostic {
                declaration,
                reason: declared_tables::scan_reason(&diagnostic).to_string(),
                message,
                path,
                line: None,
                value: None,
            }
        })
        .collect();

    // A model declared without a single trace target mints nothing at all, so
    // every ratio it reports is over an empty denominator. `is_empty()` reads
    // it as *declared* (status or trace-tag entries alone are enough), which
    // is why `ModelUndeclared` never fires for it (CR-054).
    if model.trace_targets.is_empty() {
        diagnostics.insert(
            0,
            CoverageDiagnostic {
                declaration: "traceability".to_string(),
                reason: "model-mints-nothing".to_string(),
                message: "the declared traceability model has no trace_targets, so it \
                          mints no ids and every count is over an empty denominator"
                    .to_string(),
                path: None,
                line: None,
                value: None,
            },
        );
    }

    // CR-093: the binder could not read a language it was pointed at. Ordered
    // ahead of the obligation diagnostics and after the declaration-scan ones
    // because it invalidates more than either: a language that bound nothing
    // makes every unbacked row in it unreadable, not just the rows one
    // declaration selects.
    diagnostics.extend(binding_diagnostics(&graph.binding_census));

    // #312: a tag that was written and reached no channel at all. Ordered
    // immediately after the census diagnostics because it is the case they
    // cannot see: `no-symbol-bound` and `low-symbol-binding` both read a
    // denominator this defect is missing from, so a repository whose tags are
    // all on the wrong symbols reports a flawless 100% and says nothing.
    diagnostics.extend(non_binding_tag_diagnostics(&graph.non_binding_tags));

    // #307: the two halves of one mistake, already in the payload, joined at
    // last. Ordered after the census diagnostics for the same reason as #312 —
    // this is the case they cannot see, because from their vantage point the
    // binding worked.
    diagnostics.extend(near_miss_diagnostics(&untracked_symbols, &unbacked_rows));

    // FR-053: derived here rather than in `compute` because obligations read
    // the same declared tables this reconciliation already walks, and need no
    // `Registry`.
    let (obligations, skipped) = crate::obligation::derive(spec, root, model);

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
    }));

    // ── Shared trace ids: one id bound by several distinct symbols ──
    // (FR-050-AC-23, CR-087). Scoped to ids that are row ids of
    // status-carrying rows — the population where N binders let a green row
    // rot N−1 tests deep. Grouped through BTree collections so the record
    // order — and inside each record the symbol order — is a property of the
    // data, not of the walk (NFR-006). Distinctness is `(path, symbol)`: the
    // same symbol binding one id through two declared forms (or a duplicated
    // in-symbol id, FR-051's dedup) is one binder, not a shared id.
    let mut binders: BTreeMap<&str, BTreeSet<(&str, &str)>> = BTreeMap::new();
    for relation in &graph.verifies {
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

    let report = CoverageReport {
        unbacked_rows,
        status_lies,
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
        unmatched_tags: graph.unmatched_tags.clone(),
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
