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
/// CR-027), summarizing what [`crate::grammar::property`] classified.
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
/// The two CR-027 counts are an **all-or-nothing pair**: both are `Some` for a
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
    /// Binding criteria across the corpus (CR-027), or `None` when the corpus
    /// binds none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub criteria: Option<usize>,
    /// Criteria a generator can extract a property from (CR-027), or `None`
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
    pub untracked_symbols: Vec<UntrackedSymbol>,
    pub groups: Vec<GroupCounts>,
    /// Per-document property-shape counts (CR-027). Empty for a corpus whose
    /// documents bind no criteria, so such a report serializes exactly as it
    /// did before the field existed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub criteria: Vec<CriteriaCounts>,
    pub totals: CoverageTotals,
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
    // CR-027: the criteria counts are computed here rather than inside
    // `reconcile`, which reconciles against the declared model alone and takes
    // no `Registry`. The vocabularies the classifier reads hang off the same
    // `registry` this function already holds.
    report.criteria = criteria_counts(spec, registry, root);
    // Set as a pair, so a consumer never sees a criteria count without the
    // property-shaped count it is the denominator of.
    if !report.criteria.is_empty() {
        report.totals.criteria = Some(report.criteria.iter().map(|c| c.criteria).sum());
        report.totals.property_shaped =
            Some(report.criteria.iter().map(|c| c.property_shaped).sum());
    }
    Ok(report)
}

/// Per-document property-shape counts over the corpus (CR-027).
///
/// The rollup carries **no acceptance-criteria knowledge**: it walks the
/// already path-sorted corpus, resolves each document's frontmatter type to an
/// archetype, and asks the grammar layer what that archetype's binding
/// criteria classify as. A document yielding none — anything the `ac` binding
/// does not cover, and every document under a module declaring no grammar —
/// contributes no entry, so a corpus of non-requirement documents produces an
/// empty list.
///
/// This is a pure function of statement text and the merged module
/// vocabularies: no network, no service, no execution (FR-050-CON-2).
fn criteria_counts(spec: &Spec, registry: &Registry, root: &Path) -> Vec<CriteriaCounts> {
    let vocab = GrammarVocabularies {
        lexicon: registry.lexicon_matcher(),
        observable: registry.observable_verbs_matcher(),
        vacuous: registry.vacuous_predicates_matcher(),
        idioms: registry.property_idioms_matcher(),
    };

    let mut out: Vec<CriteriaCounts> = Vec::new();
    for entry in &spec.inner.documents {
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
        let records = crate::grammar::classify_document_properties(
            grammar_ref,
            &archetype.name,
            &entry.doc,
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

    // ── Minted targets, grouped by their minting document ──
    let mut minted: Vec<MintedTarget> = Vec::new();
    for target in &model.trace_targets {
        for row in declared_tables::scan(
            spec,
            root,
            target.archetype.as_deref(),
            target.document.as_deref(),
            &target.section,
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
    let mut referenced_ids: BTreeSet<String> = BTreeSet::new();
    let mut row_ids: BTreeSet<String> = BTreeSet::new();

    for declaration in &model.document_references {
        let Ok(pattern) = regex::Regex::new(&declaration.pattern) else {
            continue; // patterns are validated at module load
        };
        for row in declared_tables::scan(
            spec,
            root,
            declaration.archetype.as_deref(),
            declaration.document.as_deref(),
            &declaration.section,
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
    untracked_symbols
        .sort_by(|a, b| (&a.path, &a.symbol, &a.trace_id).cmp(&(&b.path, &b.symbol, &b.trace_id)));
    untracked_symbols.dedup();

    CoverageReport {
        unbacked_rows,
        status_lies,
        untracked_symbols,
        groups,
        // CR-027: filled by `compute`, which holds the `Registry` this
        // reconciliation deliberately does not take.
        criteria: Vec::new(),
        totals,
    }
}

/// A path relative to the scope root, `/`-separated so reports are stable
/// across platforms. Paths outside the root are emitted as-is.
fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("/")
}
