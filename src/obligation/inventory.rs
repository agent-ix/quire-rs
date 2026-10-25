//! Owner-issued selected obligation inventories (FR-053-AC-15..23).
//!
//! Selection and row identity are corpus facts, not verification evidence.
//! Only this producer can issue an exhausted inventory; a deserialized record
//! list or a caller-built `Spec` cannot establish that the filesystem was read.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};

use crate::corpus::declared_tables::{self, ExcludeSet, ScannedRow, ScannedTable};
use crate::corpus::walk::{self, LoadedDocument, Outcome, WalkOptions};
use crate::diagnostic::Diagnostic;
use crate::traceability::{ObligationSource, TraceTarget, TraceabilityModel};

use super::{collect_combinatorial, resolve, row_obligation, Obligation, Resolved};

/// Whether the owner observed all required ends without a blocking cause.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryTerminal {
    /// The real walk and each selected scan ended normally without a failure.
    Complete,
    /// At least one failure occurred or a required end was not observed.
    Incomplete,
}

/// Stable classifications of an inventory's non-completion causes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum InventoryCauseCode {
    /// The supplied traceability model failed its owner validation.
    InvalidModel,
    /// Empty, unknown or repeated source selectors cannot name a population.
    InvalidSelection,
    /// The requested document root does not exist.
    RootMissing,
    /// The requested document root is not a directory.
    RootNotDirectory,
    /// Root metadata or canonicalization could not be read.
    RootUnreadable,
    /// The real ignore-aware directory walk reported an error.
    WalkFailed,
    /// A non-excluded candidate could not be read as UTF-8 text.
    DocumentUnreadable,
    /// A non-excluded candidate has absent/malformed frontmatter or no usable type.
    UnclassifiableDocument,
    /// No declared section is present in a selected document.
    MissingSection,
    /// The matched sections contain no table.
    MissingTable,
    /// A required identity or statement column is absent.
    MissingColumn,
    /// An observed selected row cannot produce its owner obligation.
    UnusableRow,
    /// Multiple authored rows have the same source/document/authored-ID key.
    DuplicateIdentity,
    /// The emitted-entry stop boundary was reached before observing an end.
    LimitReached,
}

/// Why a list of requested source names cannot denote a selected population.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionFailure {
    /// No source was selected.
    Empty,
    /// A requested source name is not declared.
    UnknownSource,
    /// The requested source-name list repeats a name.
    DuplicateSource,
}

/// Original error or structured observation retained beneath a typed cause.
#[derive(Debug)]
#[non_exhaustive]
pub enum InventoryCauseDetail {
    /// The original fallible filesystem operation's error.
    Io(std::io::Error),
    /// The original ignore-aware walk error, including its path context.
    Walk(ignore::Error),
    /// The original public loader diagnostic; parser error text is not invented.
    Diagnostic(Diagnostic),
    /// The original validation reason from `TraceabilityModel::validate`.
    Model(String),
    /// The invalid selector, if the failure names one.
    Selection {
        reason: SelectionFailure,
        name: Option<String>,
    },
    /// What the shared declared-table parser actually observed.
    Scan {
        expected: Vec<String>,
        observed: Vec<String>,
    },
    /// All observed conflicting authored-row ordinals.
    Duplicate {
        authored_id: String,
        rows: Vec<usize>,
    },
    /// The requested emitted-entry boundary, not an inferred exhaustion.
    Limit { limit: usize },
}

impl fmt::Display for InventoryCauseDetail {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "{error}"),
            Self::Walk(error) => write!(formatter, "{error}"),
            Self::Diagnostic(diagnostic) => match diagnostic {
                Diagnostic::SearchPathNotADirectory { path } => {
                    write!(
                        formatter,
                        "document root {} is not a directory",
                        path.display()
                    )
                }
                Diagnostic::DocumentUnreadable { path, reason } => {
                    write!(formatter, "cannot read {}: {reason}", path.display())
                }
                Diagnostic::DocumentWithoutFrontmatter { path, malformed } => {
                    let state = if *malformed { "malformed" } else { "absent" };
                    write!(formatter, "{state} frontmatter in {}", path.display())
                }
                Diagnostic::UntypedArtifact { path, .. } => {
                    write!(formatter, "no usable document type in {}", path.display())
                }
                _ => write!(
                    formatter,
                    "loader classification diagnostic (see structured detail)"
                ),
            },
            Self::Model(reason) => write!(formatter, "{reason}"),
            Self::Selection { reason, name } => {
                let description = match reason {
                    SelectionFailure::Empty => "no obligation source was selected",
                    SelectionFailure::UnknownSource => "unknown obligation source",
                    SelectionFailure::DuplicateSource => "repeated obligation source",
                };
                write!(formatter, "{description}")?;
                if let Some(name) = name {
                    write!(formatter, ": {name}")?;
                }
                Ok(())
            }
            Self::Scan { expected, observed } => write!(
                formatter,
                "cannot read declared source data; expected [{}], observed [{}]",
                expected.join(", "),
                observed.join(", ")
            ),
            Self::Duplicate { authored_id, rows } => write!(
                formatter,
                "authored identity {authored_id} is repeated in rows {}",
                rows.iter()
                    .map(usize::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::Limit { limit } => write!(
                formatter,
                "emitted-entry limit {limit} reached before end-of-scan"
            ),
        }
    }
}

impl std::error::Error for InventoryCauseDetail {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Walk(error) => Some(error),
            _ => None,
        }
    }
}

/// A blocking cause with its source-authored repair locus when known.
#[derive(Debug, thiserror::Error)]
#[error("{code:?}: {detail}")]
pub struct InventoryCause {
    code: InventoryCauseCode,
    source_name: Option<String>,
    document: Option<PathBuf>,
    row: Option<usize>,
    line: Option<usize>,
    #[source]
    detail: InventoryCauseDetail,
}

impl InventoryCause {
    /// Stable classification, independent of the human-readable payload.
    pub fn code(&self) -> InventoryCauseCode {
        self.code
    }

    /// The declaration whose scan observed the failure, when known.
    pub fn source_name(&self) -> Option<&str> {
        self.source_name.as_deref()
    }

    /// The original path, when the underlying operation identified one.
    pub fn document(&self) -> Option<&Path> {
        self.document.as_deref()
    }

    /// One-based authored row ordinal, when known.
    pub fn row(&self) -> Option<usize> {
        self.row
    }

    /// One-based document line, when known.
    pub fn line(&self) -> Option<usize> {
        self.line
    }

    /// The original error/diagnostic or exact structured scan observation.
    pub fn detail(&self) -> &InventoryCauseDetail {
        &self.detail
    }
}

/// The requested input data, preserved even when validation cannot issue identity.
#[derive(Debug, Clone, PartialEq)]
pub struct InventoryRequest {
    root: PathBuf,
    model: TraceabilityModel,
    source_names: Vec<String>,
    stop_limit: Option<usize>,
}

impl InventoryRequest {
    /// The caller's root before canonicalization.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The supplied model, including invalid declarations if validation failed.
    pub fn model(&self) -> &TraceabilityModel {
        &self.model
    }

    /// Source names in caller-supplied order; identity uses model order instead.
    pub fn source_names(&self) -> &[String] {
        &self.source_names
    }

    /// Requested emitted-entry stop boundary.
    pub fn stop_limit(&self) -> Option<usize> {
        self.stop_limit
    }
}

/// A validated source declaration and its resolved target, if target-bound.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectedSource {
    declaration: ObligationSource,
    target: Option<TraceTarget>,
}

impl SelectedSource {
    /// The original obligation-source declaration.
    pub fn declaration(&self) -> &ObligationSource {
        &self.declaration
    }

    /// The resolved target definition for an authored-row source.
    pub fn target(&self) -> Option<&TraceTarget> {
        self.target.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq)]
struct CapturedDocument {
    relative: String,
    loaded: LoadedDocument,
}

/// Read-only selected identity, issued only by the native corpus producer.
///
/// Equality compares canonical root, resolved selected declarations and effective
/// exclusions, then the exact relative-path/byte snapshot of selected documents.
/// The identity does not itself assert scan completion or evidence freshness.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectionIdentity {
    root: PathBuf,
    sources: Vec<SelectedSource>,
    model_exclude: Vec<String>,
    documents: Vec<CapturedDocument>,
}

impl SelectionIdentity {
    /// Canonical filesystem document root.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Selected declarations in their validated model order.
    pub fn sources(&self) -> &[SelectedSource] {
        &self.sources
    }

    /// The model-level exclusions participating in this identity.
    pub fn model_exclusions(&self) -> &[String] {
        &self.model_exclude
    }

    /// Selected scope-relative paths and exact captured bytes, in path order.
    pub fn documents(&self) -> impl ExactSizeIterator<Item = (&str, &str)> {
        self.documents
            .iter()
            .map(|document| (document.relative.as_str(), document.loaded.raw()))
    }
}

/// Identity origin: authored row IDs are distinct from rendered owner IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CriterionOrigin<'a> {
    /// An ID literally authored in a target-bound row.
    AuthoredRow { authored_id: &'a str },
    /// An `id_format` result for a row with no authored ID.
    RenderedRow { rendered_id: &'a str, row: usize },
    /// An `id_format` result over the owner's complete configuration table.
    CombinatorialTable { rendered_id: &'a str },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum OriginKind {
    AuthoredRow,
    RenderedRow,
    CombinatorialTable,
}

#[derive(Debug)]
struct Entry {
    obligation: Obligation,
    kind: OriginKind,
    row: Option<usize>,
    line: Option<usize>,
    document_index: usize,
}

impl Entry {
    fn origin(&self) -> CriterionOrigin<'_> {
        match self.kind {
            OriginKind::AuthoredRow => CriterionOrigin::AuthoredRow {
                authored_id: &self.obligation.id,
            },
            OriginKind::RenderedRow => CriterionOrigin::RenderedRow {
                rendered_id: &self.obligation.id,
                row: self.row.expect("rendered rows retain their source ordinal"),
            },
            OriginKind::CombinatorialTable => CriterionOrigin::CombinatorialTable {
                rendered_id: &self.obligation.id,
            },
        }
    }
}

/// The origin component of an exact key, separate from its ID string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct OriginKey {
    kind: OriginKind,
    row: Option<usize>,
}

impl<'a> CriterionOrigin<'a> {
    fn key(self) -> (&'a str, OriginKey) {
        match self {
            Self::AuthoredRow { authored_id } => (
                authored_id,
                OriginKey {
                    kind: OriginKind::AuthoredRow,
                    row: None,
                },
            ),
            Self::RenderedRow { rendered_id, row } => (
                rendered_id,
                OriginKey {
                    kind: OriginKind::RenderedRow,
                    row: Some(row),
                },
            ),
            Self::CombinatorialTable { rendered_id } => (
                rendered_id,
                OriginKey {
                    kind: OriginKind::CombinatorialTable,
                    row: None,
                },
            ),
        }
    }
}

#[derive(Debug)]
enum LookupMatch {
    Unique(usize),
    Ambiguous(Vec<usize>),
}

impl LookupMatch {
    fn add(&mut self, slot: usize) {
        match self {
            Self::Unique(first) => *self = Self::Ambiguous(vec![*first, slot]),
            Self::Ambiguous(slots) => slots.push(slot),
        }
    }
}

type OriginMatches = BTreeMap<OriginKey, LookupMatch>;
type IdMatches = BTreeMap<String, OriginMatches>;
type DocumentMatches = BTreeMap<String, IdMatches>;

/// Built once alongside emission; lookups neither walk the corpus nor scan rows.
#[derive(Debug, Default)]
struct EntryIndex {
    sources: BTreeMap<String, DocumentMatches>,
}

impl EntryIndex {
    fn insert(&mut self, entry: &Entry, slot: usize) {
        let (id, origin) = entry.origin().key();
        self.sources
            .entry(entry.obligation.source.clone())
            .or_default()
            .entry(entry.obligation.document.clone())
            .or_default()
            .entry(id.to_string())
            .or_default()
            .entry(origin)
            .and_modify(|matched| matched.add(slot))
            .or_insert(LookupMatch::Unique(slot));
    }

    fn lookup(
        &self,
        source: &str,
        document: &str,
        origin: CriterionOrigin<'_>,
    ) -> Option<&LookupMatch> {
        let (id, origin) = origin.key();
        self.sources
            .get(source)?
            .get(document)?
            .get(id)?
            .get(&origin)
    }

    fn authored_duplicates(&self) -> Vec<Vec<usize>> {
        let authored = OriginKey {
            kind: OriginKind::AuthoredRow,
            row: None,
        };
        let mut duplicates = Vec::new();
        for documents in self.sources.values() {
            for identities in documents.values() {
                for origins in identities.values() {
                    if let Some(LookupMatch::Ambiguous(slots)) = origins.get(&authored) {
                        duplicates.push(slots.clone());
                    }
                }
            }
        }
        duplicates
    }
}

/// A borrowed key tied to the owner's immutable selected snapshot.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CriterionIdentity<'a> {
    selection: &'a SelectionIdentity,
    source: &'a str,
    document: &'a str,
    origin: CriterionOrigin<'a>,
}

impl<'a> CriterionIdentity<'a> {
    /// The exact owner selection from which this key was borrowed.
    pub fn selection(&self) -> &'a SelectionIdentity {
        self.selection
    }

    /// Source declaration name.
    pub fn source_name(&self) -> &'a str {
        self.source
    }

    /// Scope-relative document path.
    pub fn document(&self) -> &'a str {
        self.document
    }

    /// The authored or rendered origin, without converting one into the other.
    pub fn origin(&self) -> CriterionOrigin<'a> {
        self.origin
    }
}

/// One borrowed inventory entry and its exact owner data.
#[derive(Debug, Clone, Copy)]
pub struct CriterionEntry<'a> {
    selection: &'a SelectionIdentity,
    entry: &'a Entry,
}

impl<'a> CriterionEntry<'a> {
    /// Selection-bound origin-aware key.
    pub fn identity(&self) -> CriterionIdentity<'a> {
        CriterionIdentity {
            selection: self.selection,
            source: &self.entry.obligation.source,
            document: &self.entry.obligation.document,
            origin: self.entry.origin(),
        }
    }

    /// The existing owner obligation record, borrowed without reparsing.
    pub fn obligation(&self) -> &'a Obligation {
        &self.entry.obligation
    }

    /// One-based source row ordinal; absent for a whole-table origin.
    pub fn row(&self) -> Option<usize> {
        self.entry.row
    }

    /// One-based source document line when the table parser supplied one.
    pub fn line(&self) -> Option<usize> {
        self.entry.line
    }
}

/// Exact-lookup refusals, distinct from inventory enumeration failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InventoryLookupError {
    #[error("criterion belongs to a different selected corpus")]
    /// The borrowed identity belongs to a different captured selection.
    SelectionMismatch,
    #[error("the selected corpus contains no such criterion")]
    /// The exact key names no retained entry in this inventory.
    UnknownIdentity,
    #[error("the selected authored identity names multiple rows")]
    /// More than one retained authored row has the exact key.
    AmbiguousIdentity,
}

/// An immutable owner-issued result, including partial rows on failure.
///
/// No public constructor or deserializer can install a `Complete` state.
#[derive(Debug)]
#[must_use]
pub struct SelectedInventory {
    request: InventoryRequest,
    selection: Option<SelectionIdentity>,
    entries: Vec<Entry>,
    index: EntryIndex,
    causes: Vec<InventoryCause>,
    advisories: Vec<Diagnostic>,
    terminal: InventoryTerminal,
}

impl SelectedInventory {
    /// Original request, including rejected input data.
    pub fn request(&self) -> &InventoryRequest {
        &self.request
    }

    /// Validated selection and captured snapshot, if input validation succeeded.
    /// Its presence alone does not attest exhaustion; inspect `terminal()`.
    pub fn selection(&self) -> Option<&SelectionIdentity> {
        self.selection.as_ref()
    }

    /// Successfully emitted entries, including partial entries after failures.
    pub fn entries(&self) -> impl ExactSizeIterator<Item = CriterionEntry<'_>> {
        self.entries.iter().map(|entry| CriterionEntry {
            selection: self
                .selection
                .as_ref()
                .expect("entries are emitted only for a validated selection"),
            entry,
        })
    }

    /// Observed emitted-entry count, derived from the retained entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether no entry was emitted, independently of completion.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Owner-issued exhaustive or incomplete terminal state.
    pub fn terminal(&self) -> InventoryTerminal {
        self.terminal
    }

    /// Typed blocking causes, with original diagnostics/errors and known loci.
    pub fn causes(&self) -> &[InventoryCause] {
        &self.causes
    }

    /// Loader advisories that do not prevent reading selected criteria.
    pub fn advisories(&self) -> &[Diagnostic] {
        &self.advisories
    }

    /// Resolve a borrowed owner identity, refusing a foreign selection first.
    pub fn lookup(
        &self,
        identity: &CriterionIdentity<'_>,
    ) -> Result<CriterionEntry<'_>, InventoryLookupError> {
        let selection = self
            .selection
            .as_ref()
            .ok_or(InventoryLookupError::SelectionMismatch)?;
        if !std::ptr::eq(selection, identity.selection) && selection != identity.selection {
            return Err(InventoryLookupError::SelectionMismatch);
        }
        self.lookup_key(identity.source, identity.document, identity.origin)
    }

    /// Look up an untrusted exact key within this inventory's own selection.
    pub fn lookup_key(
        &self,
        source: &str,
        document: &str,
        origin: CriterionOrigin<'_>,
    ) -> Result<CriterionEntry<'_>, InventoryLookupError> {
        let selection = self
            .selection
            .as_ref()
            .ok_or(InventoryLookupError::UnknownIdentity)?;
        let matched = self
            .index
            .lookup(source, document, origin)
            .ok_or(InventoryLookupError::UnknownIdentity)?;
        let slot = match matched {
            LookupMatch::Unique(slot) => *slot,
            LookupMatch::Ambiguous(_) => return Err(InventoryLookupError::AmbiguousIdentity),
        };
        let entry = &self.entries[slot];
        Ok(CriterionEntry { selection, entry })
    }
}

/// Enumerate selected obligation sources through the owner's real loader.
///
/// Input validation precedes a zero stop limit. Limits stop immediately after
/// the requested emitted-entry count, without claiming a subsequent end event.
/// Existing tolerant `derive` and `load_repo` callers retain their contracts.
pub fn inventory(
    root: &Path,
    model: &TraceabilityModel,
    source_names: &[&str],
    stop_limit: Option<usize>,
) -> SelectedInventory {
    let mut result = SelectedInventory {
        request: InventoryRequest {
            root: root.to_path_buf(),
            model: model.clone(),
            source_names: source_names
                .iter()
                .map(|name| (*name).to_string())
                .collect(),
            stop_limit,
        },
        selection: None,
        entries: Vec::new(),
        index: EntryIndex::default(),
        causes: Vec::new(),
        advisories: Vec::new(),
        terminal: InventoryTerminal::Incomplete,
    };
    if let Err(reason) = model.validate() {
        result.cause(
            InventoryCauseCode::InvalidModel,
            None,
            None,
            None,
            None,
            InventoryCauseDetail::Model(reason),
        );
    }
    let canonical = match std::fs::metadata(root) {
        Ok(metadata) if metadata.is_dir() => match root.canonicalize() {
            Ok(path) => Some(path),
            Err(error) => {
                result.root_error(InventoryCauseCode::RootUnreadable, error);
                None
            }
        },
        Ok(_) => {
            result.cause(
                InventoryCauseCode::RootNotDirectory,
                None,
                Some(root),
                None,
                None,
                InventoryCauseDetail::Diagnostic(Diagnostic::SearchPathNotADirectory {
                    path: root.to_path_buf(),
                }),
            );
            None
        }
        Err(error) => {
            let code = if error.kind() == std::io::ErrorKind::NotFound {
                InventoryCauseCode::RootMissing
            } else {
                InventoryCauseCode::RootUnreadable
            };
            result.root_error(code, error);
            None
        }
    };
    let requested: BTreeSet<&str> = source_names.iter().copied().collect();
    if requested.is_empty() {
        result.selection_error(SelectionFailure::Empty, None);
    }
    let mut seen = BTreeSet::new();
    for name in source_names {
        if !seen.insert(*name) {
            result.selection_error(SelectionFailure::DuplicateSource, Some(name));
        } else if !model.obligations.iter().any(|source| source.name == *name) {
            result.selection_error(SelectionFailure::UnknownSource, Some(name));
        }
    }
    if !result.causes.is_empty() {
        return result;
    }
    let Some(canonical) = canonical else {
        return result;
    };
    let sources = model
        .obligations
        .iter()
        .filter(|source| requested.contains(source.name.as_str()))
        .map(|source| SelectedSource {
            declaration: source.clone(),
            target: source
                .target
                .as_ref()
                .and_then(|name| {
                    model
                        .trace_targets
                        .iter()
                        .find(|target| &target.name == name)
                })
                .cloned(),
        })
        .collect::<Vec<_>>();
    result.selection = Some(SelectionIdentity {
        root: canonical.clone(),
        sources: sources.clone(),
        model_exclude: model.exclude.clone(),
        documents: Vec::new(),
    });
    if stop_limit == Some(0) {
        result.limit_reached(0);
        return result;
    }
    let model_exclude = ExcludeSet::compile_validated(&model.exclude);
    let excludes = sources
        .iter()
        .map(|source| ExcludeSet::compile_validated(&source.declaration.exclude))
        .collect::<Vec<_>>();
    let discovery = walk::discover_files_observed(&canonical, &WalkOptions::default());
    for error in discovery.errors {
        if !walk_error_excluded(&error, &canonical, &model_exclude, &excludes) {
            let path = walk_error_path(&error).map(Path::to_path_buf);
            result.cause(
                InventoryCauseCode::WalkFailed,
                None,
                path.as_deref(),
                None,
                None,
                InventoryCauseDetail::Walk(error),
            );
        }
    }
    let mut files = discovery.files;
    files.sort();
    let mut documents = Vec::new();
    for path in files {
        if excluded_from_all(&canonical, &path, &model_exclude, &excludes) {
            continue;
        }
        match walk::parse_one(&path) {
            Outcome::Loaded { doc, diags } => {
                if doc.concept_type().is_none_or(|kind| kind.trim().is_empty()) {
                    result.cause(
                        InventoryCauseCode::UnclassifiableDocument,
                        None,
                        Some(&path),
                        None,
                        None,
                        InventoryCauseDetail::Diagnostic(Diagnostic::UntypedArtifact {
                            id: doc.id.clone(),
                            path: path.clone(),
                        }),
                    );
                    continue;
                }
                result.advisories.extend(diags);
                if sources.iter().zip(&excludes).any(|(source, exclude)| {
                    let resolved = resolve(&source.declaration, model)
                        .expect("validated selected declaration resolves");
                    doc.concept_type() == Some(resolved.archetype)
                        && !exclude.excludes(&canonical, &path)
                }) {
                    documents.push(*doc);
                }
            }
            Outcome::Failed { diag } => result.cause(
                InventoryCauseCode::DocumentUnreadable,
                None,
                Some(&path),
                None,
                None,
                InventoryCauseDetail::Diagnostic(diag),
            ),
            Outcome::NotADocument { diag, .. } => result.cause(
                InventoryCauseCode::UnclassifiableDocument,
                None,
                Some(&path),
                None,
                None,
                InventoryCauseDetail::Diagnostic(diag),
            ),
        }
    }
    result
        .selection
        .as_mut()
        .expect("validated selection")
        .documents = documents
        .iter()
        .map(|document| CapturedDocument {
            relative: declared_tables::relative_path(&canonical, &document.path),
            loaded: document.clone(),
        })
        .collect();
    for (source, exclude) in sources.iter().zip(&excludes) {
        let declaration = &source.declaration;
        let resolved =
            resolve(declaration, model).expect("validated selected declaration resolves");
        for (document_index, document) in documents.iter().enumerate() {
            if document.concept_type() != Some(resolved.archetype)
                || exclude.excludes(&canonical, &document.path)
            {
                continue;
            }
            if !result.scan_document(&canonical, declaration, &resolved, document, document_index) {
                return result.finish(false);
            }
        }
    }
    result.finish(true)
}

impl SelectedInventory {
    fn cause(
        &mut self,
        code: InventoryCauseCode,
        source: Option<&str>,
        document: Option<&Path>,
        row: Option<usize>,
        line: Option<usize>,
        detail: InventoryCauseDetail,
    ) {
        self.causes.push(InventoryCause {
            code,
            source_name: source.map(str::to_string),
            document: document.map(Path::to_path_buf),
            row,
            line,
            detail,
        });
    }

    fn root_error(&mut self, code: InventoryCauseCode, error: std::io::Error) {
        let root = self.request.root.clone();
        self.cause(
            code,
            None,
            Some(&root),
            None,
            None,
            InventoryCauseDetail::Io(error),
        );
    }

    fn selection_error(&mut self, reason: SelectionFailure, name: Option<&str>) {
        self.cause(
            InventoryCauseCode::InvalidSelection,
            name,
            None,
            None,
            None,
            InventoryCauseDetail::Selection {
                reason,
                name: name.map(str::to_string),
            },
        );
    }

    fn limit_reached(&mut self, limit: usize) {
        self.cause(
            InventoryCauseCode::LimitReached,
            None,
            None,
            None,
            None,
            InventoryCauseDetail::Limit { limit },
        );
    }

    fn push_entry(&mut self, entry: Entry) -> bool {
        self.index.insert(&entry, self.entries.len());
        self.entries.push(entry);
        if let Some(limit) = self.request.stop_limit {
            if self.entries.len() == limit {
                self.limit_reached(limit);
                return false;
            }
        }
        true
    }

    fn scan_document(
        &mut self,
        root: &Path,
        source: &ObligationSource,
        resolved: &Resolved<'_>,
        document: &LoadedDocument,
        document_index: usize,
    ) -> bool {
        let scanned = declared_tables::tables_of(document.body(), &document.path, resolved.section);
        if scanned.is_empty() {
            self.cause(
                InventoryCauseCode::MissingSection,
                Some(&source.name),
                Some(&document.path),
                None,
                None,
                InventoryCauseDetail::Scan {
                    expected: resolved.section.names().to_vec(),
                    observed: crate::query::sections(document.body(), None)
                        .iter()
                        .map(|section| section.heading.clone())
                        .collect(),
                },
            );
            return true;
        }
        if !scanned
            .iter()
            .any(|(_, table)| matches!(table, ScannedTable::Table { .. }))
        {
            self.cause(
                InventoryCauseCode::MissingTable,
                Some(&source.name),
                Some(&document.path),
                None,
                None,
                InventoryCauseDetail::Scan {
                    expected: resolved.section.names().to_vec(),
                    observed: scanned.iter().map(|(heading, _)| heading.clone()).collect(),
                },
            );
            return true;
        }
        let mut ordinal = 0;
        let mut combinatorial_rows = Vec::new();
        for (_, table) in scanned {
            let ScannedTable::Table {
                headers,
                rows,
                line,
            } = table
            else {
                continue;
            };
            let required =
                std::iter::once(source.statement_column.as_str()).chain(resolved.id_column);
            let mut missing_header = false;
            for column in required {
                if !headers
                    .iter()
                    .any(|header| header.eq_ignore_ascii_case(column))
                {
                    self.cause(
                        InventoryCauseCode::MissingColumn,
                        Some(&source.name),
                        Some(&document.path),
                        None,
                        Some(line),
                        InventoryCauseDetail::Scan {
                            expected: vec![column.to_string()],
                            observed: headers.clone(),
                        },
                    );
                    missing_header = true;
                }
            }
            for row in rows {
                ordinal += 1;
                if missing_header {
                    continue;
                }
                if source.combinatorial.is_some() {
                    combinatorial_rows.push((ordinal, row));
                    continue;
                }
                let relative = declared_tables::relative_path(root, &row.path);
                let Some(obligation) = row_obligation(source, resolved, &row, ordinal, &relative)
                else {
                    self.cause(
                        InventoryCauseCode::UnusableRow,
                        Some(&source.name),
                        Some(&row.path),
                        Some(ordinal),
                        Some(row.line),
                        InventoryCauseDetail::Scan {
                            expected: std::iter::once(source.statement_column.clone())
                                .chain(resolved.id_column.map(str::to_string))
                                .collect(),
                            observed: row
                                .cells
                                .iter()
                                .filter(|(_, value)| !value.is_empty())
                                .map(|(column, _)| column.clone())
                                .collect(),
                        },
                    );
                    continue;
                };
                if obligation.id.trim().is_empty() {
                    self.cause(
                        InventoryCauseCode::UnusableRow,
                        Some(&source.name),
                        Some(&row.path),
                        Some(ordinal),
                        Some(row.line),
                        InventoryCauseDetail::Scan {
                            expected: vec!["nonempty rendered identity".to_string()],
                            observed: Vec::new(),
                        },
                    );
                    continue;
                }
                let kind = if resolved.id_column.is_some() {
                    OriginKind::AuthoredRow
                } else {
                    OriginKind::RenderedRow
                };
                if !self.push_entry(Entry {
                    obligation,
                    kind,
                    row: Some(ordinal),
                    line: Some(row.line),
                    document_index,
                }) {
                    return false;
                }
            }
        }
        if let Some(columns) = &source.combinatorial {
            self.scan_combinatorial(
                root,
                source,
                resolved,
                columns,
                &combinatorial_rows,
                document_index,
            )
        } else {
            true
        }
    }

    fn scan_combinatorial(
        &mut self,
        root: &Path,
        source: &ObligationSource,
        resolved: &Resolved<'_>,
        columns: &crate::traceability::CombinatorialColumns,
        rows: &[(usize, ScannedRow)],
        document_index: usize,
    ) -> bool {
        if rows.is_empty() {
            return true;
        }
        for (ordinal, row) in rows {
            self.validate_combinatorial_row(source, columns, *ordinal, row);
        }
        let mut obligations = Vec::new();
        collect_combinatorial(
            source,
            0,
            resolved,
            columns,
            rows.iter().map(|(_, row)| row),
            root,
            &mut obligations,
        );
        if obligations.is_empty() {
            self.cause(
                InventoryCauseCode::UnusableRow,
                Some(&source.name),
                Some(&rows[0].1.path),
                None,
                Some(rows[0].1.line),
                InventoryCauseDetail::Scan {
                    expected: vec![
                        columns.dimension_column.clone(),
                        columns.values_column.clone(),
                    ],
                    observed: rows
                        .iter()
                        .filter_map(|(_, row)| {
                            row.cell(&columns.dimension_column).map(str::to_string)
                        })
                        .collect(),
                },
            );
        }
        for (_, obligation) in obligations {
            if !self.push_entry(Entry {
                obligation,
                kind: OriginKind::CombinatorialTable,
                row: None,
                line: rows.first().map(|(_, row)| row.line),
                document_index,
            }) {
                return false;
            }
        }
        true
    }

    fn validate_combinatorial_row(
        &mut self,
        source: &ObligationSource,
        columns: &crate::traceability::CombinatorialColumns,
        ordinal: usize,
        row: &ScannedRow,
    ) {
        let dimension = row.cell(&columns.dimension_column);
        let values = row.cell(&columns.values_column);
        let exclusion = columns
            .excludes_column
            .as_deref()
            .and_then(|column| row.cell(column));
        let parsed_exclusion = exclusion.and_then(crate::combinatorial::parse_exclusion);
        let exclusion_only = dimension.is_none() && values.is_none() && parsed_exclusion.is_some();
        let mut unusable = Vec::new();
        if !exclusion_only {
            if dimension.is_none() {
                unusable.push(columns.dimension_column.clone());
            }
            if values.is_none_or(|cell| crate::combinatorial::split_values(cell).is_empty()) {
                unusable.push(columns.values_column.clone());
            }
        }
        if exclusion.is_some() && parsed_exclusion.is_none() {
            if let Some(column) = &columns.excludes_column {
                unusable.push(column.clone());
            }
        }
        if !unusable.is_empty() {
            self.cause(
                InventoryCauseCode::UnusableRow,
                Some(&source.name),
                Some(&row.path),
                Some(ordinal),
                Some(row.line),
                InventoryCauseDetail::Scan {
                    expected: unusable,
                    observed: row
                        .cells
                        .iter()
                        .map(|(column, value)| format!("{column}={value}"))
                        .collect(),
                },
            );
        }
    }

    fn finish(mut self, exhausted: bool) -> Self {
        for slots in self.index.authored_duplicates() {
            let first = &self.entries[slots[0]];
            let source = first.obligation.source.clone();
            let authored_id = first.obligation.id.clone();
            let row = first.row.expect("authored row ordinal");
            let line = first.line.expect("authored row line");
            let original_path = self
                .selection
                .as_ref()
                .expect("emitted entries have a selection")
                .documents[first.document_index]
                .loaded
                .path
                .clone();
            let rows = slots
                .iter()
                .map(|slot| self.entries[*slot].row.expect("authored row ordinal"))
                .collect();
            self.cause(
                InventoryCauseCode::DuplicateIdentity,
                Some(&source),
                Some(&original_path),
                Some(row),
                Some(line),
                InventoryCauseDetail::Duplicate { authored_id, rows },
            );
        }
        if exhausted && self.causes.is_empty() {
            self.terminal = InventoryTerminal::Complete;
        }
        self
    }
}

fn excluded_from_all(root: &Path, path: &Path, model: &ExcludeSet, sources: &[ExcludeSet]) -> bool {
    model.excludes(root, path) || sources.iter().all(|exclude| exclude.excludes(root, path))
}

fn walk_error_path(error: &ignore::Error) -> Option<&Path> {
    match error {
        ignore::Error::WithPath { path, .. } => Some(path),
        ignore::Error::WithLineNumber { err, .. } | ignore::Error::WithDepth { err, .. } => {
            walk_error_path(err)
        }
        ignore::Error::Loop { child, .. } => Some(child),
        _ => None,
    }
}

fn walk_error_excluded(
    error: &ignore::Error,
    root: &Path,
    model: &ExcludeSet,
    sources: &[ExcludeSet],
) -> bool {
    match error {
        ignore::Error::Partial(errors) => errors
            .iter()
            .all(|error| walk_error_excluded(error, root, model, sources)),
        _ => {
            walk_error_path(error).is_some_and(|path| excluded_from_all(root, path, model, sources))
        }
    }
}
