//! Declarative coverage computation (FR-050).
//!
//! The FR→AC→TC→test reconciliation the `gap-analysis` workflow performs by
//! grepping, computed generically: the engine knows nothing of "FR", "AC", or
//! "TC". Everything it reconciles — which documents mint trace ids, which rows
//! reference them, what a status value means — comes from the module-declared
//! traceability model ([`crate::traceability`]), so a module with an entirely
//! different vocabulary gets the same rollup with no engine change.
//!
//! **Data, not judgment.** The report states what is backed, what is not, and
//! what does not reconcile. Verdict policy (PASS / CONDITIONAL / FAIL), review
//! gating, and SpecReview authoring stay in the consuming workflow
//! (FR-050-CON-1).
//!
//! Inputs are the corpus, the registry, and local source trees: no network or
//! service I/O, and no execution of the code the symbols came from
//! (FR-050-CON-2, FR-051-CON-1).

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::corpus::declared_tables;
use crate::corpus::spec::Spec;
use crate::grammar::{AcPropertyCounts, GrammarVocabularies};
use crate::registry::Registry;
use crate::symbols::trace::SymbolGraph;
use crate::traceability::{StatusClass, TraceabilityModel};

/// Why a coverage run produced no report.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CoverageError {
    /// No active module declares a `traceability:` model, so there is nothing
    /// to reconcile against — a distinct diagnostic, never an empty report
    /// (FR-050-AC-9).
    #[error(
        "no active module declares a `traceability:` model; coverage needs one \
         (declare trace targets, document references, and a status vocabulary \
         in the module manifest)"
    )]
    ModelUndeclared,
}

/// A declared reference row whose trace target has no backing `verifies`
/// relation from any source symbol (FR-050-AC-3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnbackedRow {
    /// The declaration that produced the row.
    pub reference: String,
    /// Document the row lives in, relative to the scope root.
    pub document: String,
    /// The row's own id, when the declaration names a `row_id_column`.
    pub row_id: Option<String>,
    /// Trace ids the row is answerable for — its own id plus the ids it
    /// references — none of which any symbol backs.
    pub target_ids: Vec<String>,
}

/// A row whose status classes as `complete` while nothing backs it
/// (FR-050-AC-4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusLie {
    pub reference: String,
    pub document: String,
    pub row_id: Option<String>,
    /// The authored status value.
    pub status: String,
    pub target_ids: Vec<String>,
}

/// An unbacked row whose declared verification method mints no source symbol
/// (FR-050-AC-16, CR-041) — an eval, an inspection, a demonstration.
///
/// Reported separately from a status lie, because a lie is a row claiming
/// evidence it does not have, and this is a row whose own declared method makes
/// a source tag impossible. Which methods those are is module-declared; the
/// engine has no opinion about which verification produces code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoSymbolRow {
    pub reference: String,
    pub document: String,
    pub row_id: Option<String>,
    /// The declared test-type value that exempts the row.
    pub test_type: String,
    pub target_ids: Vec<String>,
}

/// A source symbol whose trace tag resolves to no declared target and no
/// declared reference row (FR-050-AC-5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UntrackedSymbol {
    pub path: String,
    pub symbol: String,
    pub trace_id: String,
}

/// Backed/total trace-target counts for one minting document (FR-050-AC-6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupCounts {
    /// The minting document, relative to the scope root.
    pub document: String,
    /// Declared target kind the ids belong to.
    pub target: String,
    pub backed: usize,
    pub total: usize,
}

/// Property-shape counts for one document's binding criteria (FR-050-AC-13,
/// CR-028), summarizing what [`crate::grammar::property`] classified.
///
/// A count is data, not a verdict (FR-050-CON-1): a low `property_shaped`
/// share is a description of a corpus, never a failing one — CR-020 already
/// recorded that criteria validated by demonstration legitimately score low.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CriteriaCounts {
    /// The document the criteria live in, relative to the scope root.
    pub document: String,
    /// The archetype the document resolved to.
    pub archetype: String,
    /// Binding criteria seen in the document.
    pub criteria: usize,
    /// Criteria a downstream generator can extract a property from.
    pub property_shaped: usize,
    /// Criteria per property-shape label. `BTreeMap` keeps the histogram
    /// deterministic (NFR-006).
    pub by_property: BTreeMap<String, usize>,
}

/// Bundle-wide totals; the backed/total pair equals the sum over
/// [`CoverageReport::groups`], the criteria pair the sum over
/// [`CoverageReport::criteria`].
///
/// The two CR-028 counts are an **all-or-nothing pair**: both are `Some` for a
/// corpus binding any criteria and both are `None` for a corpus binding none.
/// `None` is what keeps FR-050-AC-13's byte-identity — the keys are absent, so
/// a no-criteria report is byte-for-byte what an engine predating the fields
/// would have written. They are `Option` rather than a skipped zero because a
/// corpus with criteria and no property-shaped ones must still emit
/// `property_shaped: 0`: the primary consumer reads this payload as JSON, and
/// an absent key there makes the extraction ratio `NaN` in exactly the case
/// most worth reporting on.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverageTotals {
    pub backed: usize,
    pub total: usize,
    /// Binding criteria across the corpus (CR-028), or `None` when the corpus
    /// binds none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub criteria: Option<usize>,
    /// Criteria a generator can extract a property from (CR-028), or `None`
    /// when the corpus binds no criteria at all. Zero is a real value here.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub property_shaped: Option<usize>,
}

/// The machine-readable coverage report. Every collection is deterministically
/// ordered, so repeated runs over identical inputs serialize byte-identically
/// (FR-050-AC-7).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverageReport {
    pub unbacked_rows: Vec<UnbackedRow>,
    pub status_lies: Vec<StatusLie>,
    /// Unbacked rows exempted from `status_lies` by their declared method
    /// (CR-041). Empty — and so absent from the JSON — for a module declaring
    /// no `no_source_symbol` vocabulary, which keeps FR-050-AC-7 byte-identity.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub no_symbol_rows: Vec<NoSymbolRow>,
    pub untracked_symbols: Vec<UntrackedSymbol>,
    pub groups: Vec<GroupCounts>,
    /// Per-document property-shape counts (CR-028). Empty for a corpus whose
    /// documents bind no criteria, so such a report serializes exactly as it
    /// did before the field existed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub criteria: Vec<CriteriaCounts>,
    /// Declarations that scanned nothing, and why (CR-054). Empty — and so
    /// absent from the JSON — for a model whose every declaration selected
    /// something, which keeps FR-050-AC-7 byte-identity.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<CoverageDiagnostic>,
    pub totals: CoverageTotals,
}

/// A declaration that produced no rows for a reason the operator can act on
/// (CR-054): no scope at all, an archetype no document has, or a declared
/// auxiliary document that could not be read.
///
/// Non-fatal and never an exit code on its own — but since CR-049 made body
/// selection load-bearing on the declaration, a silent one of these is an
/// engine that parsed nothing and reported full coverage of nothing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverageDiagnostic {
    /// The declaration's name, as the model authored it.
    pub declaration: String,
    /// Stable machine token, shared with `quire validate`'s bundle warnings.
    pub reason: String,
    pub message: String,
    /// The unreadable document, when the diagnostic is about one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

impl CoverageReport {
    /// The report as JSON — the `quire coverage` stdout payload.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("coverage report serializes")
    }
}

/// One minted trace id and where it came from.
struct MintedTarget {
    id: String,
    target: String,
    document: String,
}

/// Reconcile declared targets, declared reference rows, and scanned source
/// trace tags into a coverage report.
///
/// `root` is the scope root every emitted path is relative to; the symbol
/// `graph` comes from [`crate::symbols::trace::bind`].
pub fn compute(
    spec: &Spec,
    registry: &Registry,
    graph: &SymbolGraph,
    root: &Path,
) -> Result<CoverageReport, CoverageError> {
    let model = registry
        .traceability()
        .ok_or(CoverageError::ModelUndeclared)?;
    let mut report = reconcile(spec, model, graph, root);
    // CR-028: the criteria counts are computed here rather than inside
    // `reconcile`, which reconciles against the declared model alone and takes
    // no `Registry`. The vocabularies the classifier reads hang off the same
    // `registry` this function already holds.
    report.criteria = criteria_counts(spec, registry, model, root);
    // Set as a pair, so a consumer never sees a criteria count without the
    // property-shaped count it is the denominator of.
    if !report.criteria.is_empty() {
        report.totals.criteria = Some(report.criteria.iter().map(|c| c.criteria).sum());
        report.totals.property_shaped =
            Some(report.criteria.iter().map(|c| c.property_shaped).sum());
    }
    Ok(report)
}

/// Per-document property-shape counts over the corpus (CR-028).
///
/// The rollup carries **no acceptance-criteria knowledge**: it walks the
/// already path-sorted corpus, resolves each document's frontmatter type to an
/// archetype, and asks the grammar layer what that archetype's binding
/// criteria classify as. A document yielding none — anything the `ac` binding
/// does not cover, and every document under a module declaring no grammar —
/// contributes no entry, so a corpus of non-requirement documents produces an
/// empty list.
///
/// The walk is declaration-free by necessity — criteria classification is not a
/// declared target — but it is not *scope*-free: the model-level `exclude:`
/// says which paths hold no corpus data at all, and this walk reads it (CR-060).
/// Before that, a deliberately malformed fixture under an excluded path
/// contributed to `criteria` and to both totals, inflating the denominator, and
/// its body was parsed despite the declaration saying it is not corpus data.
///
/// This is a pure function of statement text and the merged module
/// vocabularies: no network, no service, no execution (FR-050-CON-2).
fn criteria_counts(
    spec: &Spec,
    registry: &Registry,
    model: &TraceabilityModel,
    root: &Path,
) -> Vec<CriteriaCounts> {
    let vocab = GrammarVocabularies {
        lexicon: registry.lexicon_matcher(),
        observable: registry.observable_verbs_matcher(),
        vacuous: registry.vacuous_predicates_matcher(),
        idioms: registry.property_idioms_matcher(),
    };
    let excluded = declared_tables::ExcludeSet::compile(&model.exclude);

    let mut out: Vec<CriteriaCounts> = Vec::new();
    for entry in &spec.inner.documents {
        // Path-only, and ahead of every other gate: an excluded document must
        // not be classified *or* body-parsed (CR-060). It matches on the same
        // `relative_path` derivation a report path uses, so a glob and a
        // reported path compare as the same string (CR-038).
        if excluded.excludes(root, &entry.path) {
            continue;
        }
        let Some(archetype) =
            crate::corpus::spec::artifact_type(entry).and_then(|ty| registry.archetype(&ty))
        else {
            continue;
        };
        let Some(grammar_ref) = archetype.grammar_ref() else {
            continue;
        };
        // Only the tallies are wanted here, and no tally reads a record's
        // line, so the line offset is immaterial to this surface.
        //
        // The body touch happens only past the archetype/grammar gates
        // above (CR-047): a document under a module declaring no grammar
        // stays unparsed.
        let records = crate::grammar::classify_document_properties(
            grammar_ref,
            &archetype.name,
            entry.body(),
            0,
            vocab,
        );
        if records.is_empty() {
            continue;
        }
        let counts = AcPropertyCounts::tally(records.iter());
        out.push(CriteriaCounts {
            document: relative(root, &entry.path),
            archetype: archetype.name.clone(),
            criteria: counts.criteria,
            property_shaped: counts.property_shaped,
            by_property: counts.by_property,
        });
    }
    out.sort_by(|a, b| (&a.document, &a.archetype).cmp(&(&b.document, &b.archetype)));
    out
}

fn reconcile(
    spec: &Spec,
    model: &TraceabilityModel,
    graph: &SymbolGraph,
    root: &Path,
) -> CoverageReport {
    let backed: BTreeSet<&str> = graph.backed_trace_ids();
    // CR-060: compiled once for the whole reconciliation — every declaration
    // is scoped by it.
    let model_exclude = declared_tables::ExcludeSet::compile(&model.exclude);

    // ── Minted targets, grouped by their minting document ──
    let mut ctx = declared_tables::ScanContext::default();
    let mut minted: Vec<MintedTarget> = Vec::new();
    for target in &model.trace_targets {
        let exclude = declared_tables::ExcludeSet::compile(&target.exclude);
        for row in declared_tables::scan(
            spec,
            root,
            declared_tables::DeclaredScope {
                name: &target.name,
                archetype: &target.archetype,
                exclude: &exclude,
                model_exclude: &model_exclude,
            },
            &target.section,
            &mut ctx,
        ) {
            let Some(id) = row.cell(&target.id_column) else {
                continue;
            };
            minted.push(MintedTarget {
                id: id.to_string(),
                target: target.name.clone(),
                document: relative(root, &row.path),
            });
        }
    }

    let mut counts: BTreeMap<(String, String), (usize, usize)> = BTreeMap::new();
    let mut declared_ids: BTreeSet<String> = BTreeSet::new();
    for entry in &minted {
        declared_ids.insert(entry.id.clone());
        let slot = counts
            .entry((entry.document.clone(), entry.target.clone()))
            .or_insert((0, 0));
        slot.1 += 1;
        if backed.contains(entry.id.as_str()) {
            slot.0 += 1;
        }
    }

    // ── Reference rows: unbacked rows and status lies ──
    let mut unbacked_rows: Vec<UnbackedRow> = Vec::new();
    let mut status_lies: Vec<StatusLie> = Vec::new();
    let mut no_symbol_rows: Vec<NoSymbolRow> = Vec::new();
    let mut referenced_ids: BTreeSet<String> = BTreeSet::new();
    let mut row_ids: BTreeSet<String> = BTreeSet::new();

    for declaration in &model.document_references {
        let Ok(pattern) = regex::Regex::new(&declaration.pattern) else {
            continue; // patterns are validated at module load
        };
        let exclude = declared_tables::ExcludeSet::compile(&declaration.exclude);
        for row in declared_tables::scan(
            spec,
            root,
            declared_tables::DeclaredScope {
                name: &declaration.name,
                archetype: &declaration.archetype,
                exclude: &exclude,
                model_exclude: &model_exclude,
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
            if is_backed {
                continue;
            }

            let document = relative(root, &row.path);
            unbacked_rows.push(UnbackedRow {
                reference: declaration.name.clone(),
                document: document.clone(),
                row_id: row_id.clone(),
                target_ids: answerable.clone(),
            });

            // CR-041: a row verified by a method that mints no source symbol
            // cannot carry a trace tag, so calling it a lie asserts something
            // its own declared method makes impossible. It stays in
            // `unbacked_rows` — that is a fact — and is explained here.
            let exempting_type = model
                .vocabularies
                .test_type_column
                .as_deref()
                .and_then(|column| row.cell(column))
                .filter(|value| model.vocabularies.mints_no_symbol(value));
            if let Some(test_type) = exempting_type {
                no_symbol_rows.push(NoSymbolRow {
                    reference: declaration.name.clone(),
                    document,
                    row_id,
                    test_type: test_type.to_string(),
                    target_ids: answerable,
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
    untracked_symbols
        .sort_by(|a, b| (&a.path, &a.symbol, &a.trace_id).cmp(&(&b.path, &b.symbol, &b.trace_id)));
    untracked_symbols.dedup();

    // CR-054: declarations that selected nothing, rendered from the one
    // shared vocabulary `quire validate` also reports them under. Already
    // sorted by `into_diagnostics`, so the order is a property of the model.
    let mut diagnostics: Vec<CoverageDiagnostic> = ctx
        .into_diagnostics(totals.total > 0)
        .into_iter()
        .map(|(declaration, diagnostic)| {
            let (_, message) = declared_tables::scan_finding(&declaration, &diagnostic, root);
            CoverageDiagnostic {
                declaration,
                reason: declared_tables::scan_reason(&diagnostic).to_string(),
                message,
                // Since CR-062 the only scan diagnostic is declaration-level,
                // and a declaration-level fault has no document to point at.
                // The field stays on the report shape: a future diagnostic that
                // does name a document must not require a payload change.
                path: None,
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
            },
        );
    }

    CoverageReport {
        unbacked_rows,
        status_lies,
        no_symbol_rows,
        untracked_symbols,
        groups,
        // CR-028: filled by `compute`, which holds the `Registry` this
        // reconciliation deliberately does not take.
        criteria: Vec::new(),
        diagnostics,
        totals,
    }
}

/// A path relative to the scope root, `/`-separated so reports are stable
/// across platforms. Shared with the scan layer so a report path and an
/// `exclude` glob are matched against the same string (CR-038).
fn relative(root: &Path, path: &Path) -> String {
    declared_tables::relative_path(root, path)
}
