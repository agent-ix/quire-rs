// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Owner-neutral retained backing projection (FR-078).
//!
//! These values are reconciliation inputs, not qualification certificates. The
//! supported producer qualifier lives in Quoin; this module neither authenticates
//! observations nor implements its audit or per-act qualification rules.

use serde::{Deserialize, Serialize};

use crate::coverage::CoverageReport;
use crate::obligation::inventory::{
    CriterionIdentity, CriterionOrigin, InventoryLookupError, InventoryTerminal, SelectedInventory,
};

/// Original owner read-boundary locators, without a source-symbol identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct RecordLocator<'a> {
    /// Original retained record locator.
    pub record: &'a str,
    /// Original retained run locator.
    pub run: &'a str,
    /// Original entry locator within the run.
    pub entry: &'a str,
}

/// An exact non-criterion declaration, issued by the existing declaration scanner.
/// It is not an obligation, inventory entry or qualification certificate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeclaredTarget<'a> {
    selection: &'a crate::obligation::inventory::SelectionIdentity,
    declaration: &'a crate::traceability::TraceTarget,
    document: &'a str,
    id: &'a str,
    line: Option<usize>,
    raw: &'a str,
}

impl<'a> DeclaredTarget<'a> {
    /// Original native selected snapshot; this does not assert qualification.
    pub fn selection(&self) -> &'a crate::obligation::inventory::SelectionIdentity {
        self.selection
    }
    /// Original model-owned declaration.
    pub fn declaration(&self) -> &'a crate::traceability::TraceTarget {
        self.declaration
    }
    /// Original scope-relative document locator.
    pub fn document(&self) -> &'a str {
        self.document
    }
    /// Exact declared ID, without case or hierarchy inference.
    pub fn id(&self) -> &'a str {
        self.id
    }
    /// Original row, list-item or heading line, absent for a frontmatter-declared document.
    pub fn line(&self) -> Option<usize> {
        self.line
    }
}

/// A native criterion or a validated non-criterion declared target.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EvidenceTarget<'a> {
    /// The inventory owner's original criterion identity.
    Criterion(CriterionIdentity<'a>),
    /// An exact declaration resolved by the engine's existing scanner.
    Declared(DeclaredTarget<'a>),
}

impl<'a> From<CriterionIdentity<'a>> for EvidenceTarget<'a> {
    fn from(value: CriterionIdentity<'a>) -> Self {
        Self::Criterion(value)
    }
}

impl<'a> From<DeclaredTarget<'a>> for EvidenceTarget<'a> {
    fn from(value: DeclaredTarget<'a>) -> Self {
        Self::Declared(value)
    }
}

/// Resolve a non-criterion target using the existing trace-target declarations.
/// Duplicate rows refuse lookup; no first-match or shared-TC inference is used.
/// The document path is relative to the inventory's original scope root.
pub fn declared_target<'a>(
    inventory: &'a SelectedInventory,
    spec: &'a crate::Spec,
    declaration_name: &str,
    document: &'a str,
    id: &'a str,
) -> Result<DeclaredTarget<'a>, InventoryLookupError> {
    use crate::corpus::declared_tables::{self, ExcludeSet, ScanContext};
    let selection = inventory
        .selection()
        .ok_or(InventoryLookupError::SelectionMismatch)?;
    let model = inventory.request().model();
    let root = inventory.request().root();
    if model
        .obligations
        .iter()
        .any(|source| source.target.as_deref() == Some(declaration_name))
    {
        return Err(InventoryLookupError::UnknownIdentity);
    }
    let declaration = model
        .trace_targets
        .iter()
        .find(|target| target.name == declaration_name)
        .ok_or(InventoryLookupError::UnknownIdentity)?;
    let model_excluded = ExcludeSet::compile_validated(&model.exclude);
    let mut context = ScanContext::default();
    let minted = declared_tables::mint(spec, root, declaration, &model_excluded, &mut context);
    let mut matching = minted.into_iter().filter(|minted| {
        crate::coverage::relative(root, &minted.path) == document && minted.id == id
    });
    let row = matching.next();
    let frontmatter =
        crate::corpus::trace_refs::target_documents(spec, root, declaration, &model_excluded).any(
            |loaded| crate::coverage::relative(root, &loaded.path) == document && loaded.id == id,
        );
    if matching.next().is_some() || (row.is_some() && frontmatter) {
        return Err(InventoryLookupError::AmbiguousIdentity);
    }
    if row.is_none() && !frontmatter {
        return Err(InventoryLookupError::UnknownIdentity);
    }
    let loaded = spec
        .documents()
        .iter()
        .find(|loaded| crate::coverage::relative(root, &loaded.path) == document)
        .ok_or(InventoryLookupError::UnknownIdentity)?;
    Ok(DeclaredTarget {
        selection,
        declaration,
        document,
        id,
        line: row.map(|row| row.line),
        raw: loaded.raw(),
    })
}

/// A bridge's explicit backing projection. Constructing it does not qualify an act.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RecordClaim<'a> {
    /// Borrowed native owner identity, including its selected snapshot.
    pub target: EvidenceTarget<'a>,
    /// Original retained owner locators.
    pub locator: RecordLocator<'a>,
    /// Original declared method; no inferred Test conversion.
    pub method: &'a str,
    /// Actual performer or accountable attester supplied by the bridge.
    pub actor: &'a str,
    /// Original owner qualification/authority/observation references.
    pub provenance: &'a [&'a str],
}

/// Stable non-qualified cause categories, retaining owner-specific detail separately.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CauseCode {
    /// A required act failed.
    Failed,
    /// A required act errored.
    Errored,
    /// A required act was skipped.
    Skipped,
    /// A required retained record or act is absent.
    Absent,
    /// A required premise was not evaluated.
    Unevaluated,
    /// The performed source revision differs.
    WrongRevision,
    /// The selection differs from the supplied owner inventory.
    SelectionMismatch,
    /// Owner freshness invalidates the evidence.
    Stale,
    /// The exact target is unknown.
    InvalidTarget,
    /// The owner identity identifies multiple rows.
    AmbiguousTarget,
    /// Contradictory projections share one record identity.
    RecordConflict,
    /// An evaluated owner premise rejected the act.
    QualificationRejected,
    /// The native selected inventory did not complete.
    InventoryIncomplete,
}

impl CauseCode {
    /// Every stable retained-input refusal category.
    pub const fn all() -> &'static [Self] {
        &[
            Self::Failed,
            Self::Errored,
            Self::Skipped,
            Self::Absent,
            Self::Unevaluated,
            Self::WrongRevision,
            Self::SelectionMismatch,
            Self::Stale,
            Self::InvalidTarget,
            Self::AmbiguousTarget,
            Self::RecordConflict,
            Self::QualificationRejected,
            Self::InventoryIncomplete,
        ]
    }
    /// Stable machine code to print with the original target and locator.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Failed => "failed",
            Self::Errored => "errored",
            Self::Skipped => "skipped",
            Self::Absent => "absent",
            Self::Unevaluated => "unevaluated",
            Self::WrongRevision => "wrong-revision",
            Self::SelectionMismatch => "selection-mismatch",
            Self::Stale => "stale",
            Self::InvalidTarget => "invalid-target",
            Self::AmbiguousTarget => "ambiguous-target",
            Self::RecordConflict => "record-conflict",
            Self::QualificationRejected => "qualification-rejected",
            Self::InventoryIncomplete => "inventory-incomplete",
        }
    }
    /// Resolve an exact stable machine code without inspecting diagnostic prose.
    pub fn from_code(code: &str) -> Option<Self> {
        Self::all()
            .iter()
            .copied()
            .find(|value| value.as_str() == code)
    }
}

/// A bridge-provided non-qualified result; an absent record has no locator.
#[derive(Debug, Clone, Copy)]
pub struct ActResult<'a> {
    /// Exact native target, when the owner could resolve it.
    pub target: Option<EvidenceTarget<'a>>,
    /// Original requested target text, including unresolved requests.
    pub requested_target: &'a str,
    /// Available original owner locators.
    pub locator: Option<RecordLocator<'a>>,
    /// Stable consumer cause category.
    pub cause: CauseCode,
    /// Original owner subtype, premise and diagnostic references.
    pub detail: &'a str,
}

/// Explicit evidence mode, including a genuinely empty request/projection.
///
/// Only the owner bridge's private qualified view establishes qualification.
/// This public core projection grants no producer authority by construction.
#[derive(Debug)]
pub struct BackingInput<'a> {
    /// Native population and completion witness; never a copied census.
    pub inventory: &'a SelectedInventory,
    /// Owner-approved accepted-claim projection.
    pub claims: &'a [RecordClaim<'a>],
    /// Every non-qualified requested act and every applicable owner cause.
    pub results: &'a [ActResult<'a>],
}

/// Rendered locus for reports. Matching always uses the native borrowed identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetLocus {
    /// Native source declaration name.
    pub source: String,
    /// Native selected document path.
    pub document: String,
    /// Native authored or rendered ID.
    pub id: String,
    /// Native origin label; this report is not a reconstructible owner identity.
    pub origin: String,
    /// Native rendered-row ordinal, absent for other origins.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row: Option<usize>,
    /// Original document line when the owner supplied it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
}

impl From<CriterionIdentity<'_>> for TargetLocus {
    fn from(identity: CriterionIdentity<'_>) -> Self {
        let (id, origin, row) = match identity.origin() {
            CriterionOrigin::AuthoredRow { authored_id } => (authored_id, "authored-row", None),
            CriterionOrigin::RenderedRow { rendered_id, row } => {
                (rendered_id, "rendered-row", Some(row))
            }
            CriterionOrigin::CombinatorialTable { rendered_id } => {
                (rendered_id, "combinatorial-table", None)
            }
        };
        Self {
            source: identity.source_name().into(),
            document: identity.document().into(),
            id: id.into(),
            origin: origin.into(),
            row,
            line: None,
        }
    }
}

impl From<EvidenceTarget<'_>> for TargetLocus {
    fn from(target: EvidenceTarget<'_>) -> Self {
        match target {
            EvidenceTarget::Criterion(identity) => identity.into(),
            EvidenceTarget::Declared(target) => Self {
                source: target.declaration.name.clone(),
                document: target.document.into(),
                id: target.id.into(),
                origin: "declared-target".into(),
                row: None,
                line: target.line,
            },
        }
    }
}

/// Original retained locators rendered in an owned report.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetainedLocator {
    /// Retained record locator.
    pub record: String,
    /// Retained run locator.
    pub run: String,
    /// Retained run entry locator.
    pub entry: String,
}
impl From<RecordLocator<'_>> for RetainedLocator {
    fn from(value: RecordLocator<'_>) -> Self {
        Self {
            record: value.record.into(),
            run: value.run.into(),
            entry: value.entry.into(),
        }
    }
}

/// A separate record binder; never a source declaration-site binder.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordBinder {
    /// Rendered exact native target locus.
    pub target: TargetLocus,
    /// Original owner record/run/entry locators.
    pub locator: RetainedLocator,
    /// Preserved declared method.
    pub method: String,
    /// Original performer/attester.
    pub actor: String,
    /// Owner qualification and authority references.
    pub provenance: Vec<String>,
}

/// A non-qualified result, including all original owner causes.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceResult {
    /// Resolved native target locus when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<TargetLocus>,
    /// Original request, including unresolved requests.
    pub requested_target: String,
    /// Original locator, absent for missing records.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locator: Option<RetainedLocator>,
    /// Stable consumer cause.
    pub cause: CauseCode,
    /// Original owner cause detail.
    pub detail: String,
}

/// Original native non-completion causes rendered without inventing a new census.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InventoryCauseReport {
    /// Original native typed cause category.
    pub code: crate::obligation::inventory::InventoryCauseCode,
    /// Original selected source when available.
    pub source: Option<String>,
    /// Original document locus when available.
    pub document: Option<String>,
    /// Original row ordinal when available.
    pub row: Option<usize>,
    /// Original line when available.
    pub line: Option<usize>,
    /// Original typed cause's diagnostic representation.
    pub detail: String,
}

pub(crate) struct ReconciledEvidence<'a> {
    pub input: &'a BackingInput<'a>,
    pub accepted: Vec<RecordClaim<'a>>,
    pub results: Vec<EvidenceResult>,
}

fn lookup_cause(error: InventoryLookupError) -> CauseCode {
    match error {
        InventoryLookupError::SelectionMismatch => CauseCode::SelectionMismatch,
        InventoryLookupError::UnknownIdentity => CauseCode::InvalidTarget,
        InventoryLookupError::AmbiguousIdentity => CauseCode::AmbiguousTarget,
    }
}

impl<'a> ReconciledEvidence<'a> {
    pub fn new(input: &'a BackingInput<'a>) -> Self {
        let mut this = Self {
            input,
            accepted: Vec::new(),
            results: input
                .results
                .iter()
                .map(|r| EvidenceResult {
                    target: r.target.map(TargetLocus::from),
                    requested_target: r.requested_target.into(),
                    locator: r.locator.map(RetainedLocator::from),
                    cause: r.cause,
                    detail: r.detail.into(),
                })
                .collect(),
        };
        if input.inventory.terminal() != InventoryTerminal::Complete {
            for cause in input.inventory.causes() {
                this.results.push(EvidenceResult {
                    target: None,
                    requested_target: String::new(),
                    locator: None,
                    cause: CauseCode::InventoryIncomplete,
                    detail: format!("{:?}: {:?}", cause.code(), cause.detail()),
                });
            }
        }
        for claim in input.claims {
            let lookup = match claim.target {
                EvidenceTarget::Criterion(identity) => input
                    .inventory
                    .lookup(&identity)
                    .map(|entry| entry.obligation().method.as_deref()),
                EvidenceTarget::Declared(target) => match input.inventory.selection() {
                    Some(selection) if selection == target.selection => Ok(Some(claim.method)),
                    _ => Err(InventoryLookupError::SelectionMismatch),
                },
            };
            let refusal = match lookup {
                Err(error) => Some((lookup_cause(error), error.to_string())),
                Ok(method) if method != Some(claim.method) => Some((
                    CauseCode::QualificationRejected,
                    "projected method differs from selected declaration".into(),
                )),
                Ok(_)
                    if input.claims.iter().any(|other| {
                        other.target == claim.target
                            && other.locator == claim.locator
                            && other != claim
                    }) =>
                {
                    Some((
                        CauseCode::RecordConflict,
                        "contradictory projections for retained record/run/entry".into(),
                    ))
                }
                Ok(_)
                    if input.results.iter().any(|result| {
                        result.target == Some(claim.target) && result.locator == Some(claim.locator)
                    }) =>
                {
                    Some((
                        CauseCode::RecordConflict,
                        "accepted projection also has non-qualified owner results".into(),
                    ))
                }
                Ok(_) => None,
            };
            if let Some((cause, detail)) = refusal {
                let target = TargetLocus::from(claim.target);
                this.results.push(EvidenceResult {
                    requested_target: target.id.clone(),
                    target: Some(target),
                    locator: Some(claim.locator.into()),
                    cause,
                    detail,
                });
            } else if !this.accepted.contains(claim) {
                this.accepted.push(*claim);
            }
        }
        this.results.sort_by_key(|result| {
            let index = input.inventory.entries().position(|entry| {
                result.target.as_ref() == Some(&TargetLocus::from(entry.identity()))
            });
            (index.unwrap_or(usize::MAX), result.clone())
        });
        this.results.dedup();
        this.accepted.sort_by_key(|claim| {
            (
                input
                    .inventory
                    .entries()
                    .position(|entry| EvidenceTarget::Criterion(entry.identity()) == claim.target)
                    .unwrap_or(usize::MAX),
                TargetLocus::from(claim.target),
                claim.locator,
            )
        });
        this
    }

    pub fn check_context(
        &mut self,
        spec: &crate::Spec,
        model: &crate::traceability::TraceabilityModel,
        root: &std::path::Path,
    ) {
        let context = self.input.inventory.matches_spec(spec, root, model);
        let declarations_match = self.accepted.iter().all(|claim| match claim.target {
            EvidenceTarget::Criterion(_) => true,
            EvidenceTarget::Declared(target) => spec.documents().iter().any(|document| {
                crate::coverage::relative(root, &document.path) == target.document
                    && document.raw() == target.raw
            }),
        });
        if context.is_err() || !declarations_match {
            self.accepted.clear();
            self.results.push(EvidenceResult {
                target: None,
                requested_target: String::new(),
                locator: None,
                cause: CauseCode::SelectionMismatch,
                detail: context.err().map_or_else(
                    || "declared target document snapshot differs".into(),
                    |error| error.to_string(),
                ),
            });
        }
    }

    pub fn all_binders(&self) -> Vec<RecordBinder> {
        self.accepted
            .iter()
            .map(|claim| RecordBinder {
                target: claim.target.into(),
                locator: claim.locator.into(),
                method: claim.method.into(),
                actor: claim.actor.into(),
                provenance: claim
                    .provenance
                    .iter()
                    .map(|value| (*value).into())
                    .collect(),
            })
            .collect()
    }

    pub fn binders(&self, identity: EvidenceTarget<'_>) -> Vec<RecordBinder> {
        self.accepted
            .iter()
            .zip(self.all_binders())
            .filter(|(claim, _)| claim.target == identity)
            .map(|(_, binder)| binder)
            .collect()
    }

    pub fn backs_minted(&self, document: &str, id: &str, line: usize, target_name: &str) -> bool {
        self.accepted.iter().any(|claim| match claim.target {
            EvidenceTarget::Declared(target) => {
                target.document == document
                    && target.id == id
                    && target.line == Some(line)
                    && target.declaration.name == target_name
            }
            EvidenceTarget::Criterion(identity) => {
                self.input.inventory.lookup(&identity).is_ok_and(|entry| {
                    identity.document() == document
                        && entry.obligation().id == id
                        && entry.line() == Some(line)
                        && self.input.inventory.selection().is_some_and(|selection| {
                            selection.sources().iter().any(|source| {
                                source.declaration().name == identity.source_name()
                                    && source.declaration().target.as_deref() == Some(target_name)
                            })
                        })
                })
            }
        })
    }

    pub fn backs_row(&self, document: &str, id: &str, line: usize) -> bool {
        self.accepted.iter().any(|claim| {
            matches!(claim.target, EvidenceTarget::Declared(target)
            if target.document == document && target.id == id && target.line == Some(line))
        }) || self.input.inventory.entries().any(|entry| {
            entry.identity().document() == document
                && entry.obligation().id == id
                && entry.line() == Some(line)
                && self
                    .accepted
                    .iter()
                    .any(|claim| claim.target == EvidenceTarget::Criterion(entry.identity()))
        })
    }
}

/// Query retained records by exact native target or original owner locator.
#[derive(Debug, Clone, Copy)]
pub enum TraceQuery<'a> {
    /// Forward lookup, without display-ID or sibling inference.
    Target(EvidenceTarget<'a>),
    /// Inverse lookup, without manufacturing a file/symbol identity.
    Record(RecordLocator<'a>),
    /// Every entry belonging to the original run locator.
    Run(&'a str),
}

/// Query the same projection used by coverage; source search remains separate.
pub fn trace(input: &BackingInput<'_>, query: TraceQuery<'_>) -> Vec<RecordBinder> {
    let evidence = ReconciledEvidence::new(input);
    evidence
        .accepted
        .iter()
        .zip(evidence.all_binders())
        .filter(|(claim, _)| match query {
            TraceQuery::Target(identity) => claim.target == identity,
            TraceQuery::Record(locator) => claim.locator == locator,
            TraceQuery::Run(run) => claim.locator.run == run,
        })
        .map(|(_, binder)| binder)
        .collect()
}

/// A declared input that the host actually attempted but could not evaluate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeclaredInputError {
    /// Original declared input locator.
    pub locator: String,
    /// Original evaluation error.
    pub detail: String,
}

/// Inherited static coverage rejection causes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum StrictCause {
    /// No live or retired target exists.
    ZeroTargets,
    /// At least one live reference row is unbacked.
    UnbackedRows,
    /// At least one complete status contradicts its backing.
    ContradictedStatus,
}

/// Complete ordered reasons to print before mapping rejection to exit 1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, thiserror::Error)]
#[error("strict coverage rejected; inspect all structured reasons")]
pub struct StrictRejection {
    /// Every non-qualified act and core evidence refusal.
    pub evidence: Vec<EvidenceResult>,
    /// Original native inventory cause categories and loci.
    pub inventory_causes: Vec<InventoryCauseReport>,
    /// Inherited failures in stable order.
    pub inherited: Vec<StrictCause>,
    /// Original declared-input evaluation failures, never a healthy flag.
    pub declared_inputs: Vec<DeclaredInputError>,
}

/// Decide strict coverage, retaining evidence and inherited failures together.
///
/// The host supplies actual declared-input evaluation errors and prints the full
/// coverage report and reasons before mapping `Ok` to exit 0 or `Err` to exit 1.
pub fn strict_decision(
    report: &CoverageReport,
    declared_inputs: &[DeclaredInputError],
) -> Result<(), StrictRejection> {
    let mut inherited = Vec::new();
    if report.totals.total == 0 && report.totals.retired == 0 {
        inherited.push(StrictCause::ZeroTargets);
    }
    if !report.unbacked_rows.is_empty() {
        inherited.push(StrictCause::UnbackedRows);
    }
    if !report.status_lies.is_empty() {
        inherited.push(StrictCause::ContradictedStatus);
    }
    let evidence = report.evidence_results.clone().unwrap_or_default();
    if inherited.is_empty() && evidence.is_empty() && declared_inputs.is_empty() {
        Ok(())
    } else {
        Err(StrictRejection {
            evidence,
            inventory_causes: report.inventory_causes.clone().unwrap_or_default(),
            inherited,
            declared_inputs: declared_inputs.to_vec(),
        })
    }
}
