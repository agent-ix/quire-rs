//! Source-diff impact selection using FR-062 bindings and FR-077 exact lookup.

use std::collections::{BTreeMap, BTreeSet};

use ix_trace_rs::implements;
use serde::Serialize;

use crate::symbols::{extract_file, trace, trace_search, Symbol, SymbolExtraction, SymbolKind};
use crate::traceability::{SourceLanguage, TraceabilityModel};

/// Inclusive, one-based changed line range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct LineRange {
    /// First changed line.
    pub start: usize,
    /// Last changed line.
    pub end: usize,
}

/// Source files at one revision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSnapshot {
    /// Stable revision label.
    pub revision: String,
    /// Repo-relative file contents.
    pub files: BTreeMap<String, String>,
}

/// Changed lines in one path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChange {
    /// Repo-relative path.
    pub path: String,
    /// Base-side lines, absent for an added file.
    pub base: Option<LineRange>,
    /// Head-side lines, absent for a deleted file.
    pub head: Option<LineRange>,
}

/// Inputs to an impact calculation.
pub struct ImpactInput<'a> {
    /// Base source tree.
    pub base: Option<&'a SourceSnapshot>,
    /// Head source tree.
    pub head: Option<&'a SourceSnapshot>,
    /// Changed paths and line ranges.
    pub changes: &'a [FileChange],
    /// Existing declared marker grammar.
    pub traceability: &'a TraceabilityModel,
    /// IDs resolvable in the supplied specification set.
    pub available_spec_ids: &'a BTreeSet<String>,
    /// IDs whose specification could not be loaded.
    pub unavailable_spec_ids: &'a BTreeSet<String>,
}

/// Revision side containing a changed symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RevisionSide {
    /// Base tree.
    Base,
    /// Head tree.
    Head,
}

/// Symbol overlapping a changed range.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ChangedSymbol {
    /// Revision side.
    pub side: RevisionSide,
    /// Stable extracted identity.
    pub id: String,
    /// Repo-relative path.
    pub path: String,
    /// Qualified name.
    pub name: String,
    /// Kind label.
    pub kind: String,
    /// Extracted span.
    pub lines: LineRange,
    /// Bound implementation ids.
    pub implemented_ids: Vec<String>,
}

/// A changed range outside all extracted symbols.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UnmappedChange {
    /// Repo-relative path.
    pub path: String,
    /// Revision side.
    pub side: RevisionSide,
    /// Changed lines.
    pub lines: LineRange,
}

/// Reason chain for a selected test.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ImpactReason {
    /// Test directly changed by the diff.
    DirectTestChange {
        changed_path: String,
        changed_lines: LineRange,
        test_symbol: String,
    },
    /// Changed implementation and test share an exact id.
    ImplementedId {
        changed_path: String,
        changed_lines: LineRange,
        implementation_path: String,
        implementation: String,
        trace_id: String,
    },
}

/// Test selected for the Rust runner.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SelectedTest {
    /// Test path.
    pub path: String,
    /// Qualified test name.
    pub name: String,
    /// Runner substring filter.
    pub filter: String,
    /// Additive selection reasons.
    pub reasons: Vec<ImpactReason>,
}

/// Other code implementing an id on changed code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RelatedImplementation {
    /// Source path.
    pub path: String,
    /// Qualified name.
    pub name: String,
    /// Shared exact relation id.
    pub trace_id: String,
    /// Source-grounded reason.
    pub reason: ImpactReason,
}

/// Finding that makes the analysis incomplete or identifies unlinked code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ImpactFinding {
    /// Changed production code has no bound implementation relation.
    UnlinkedSymbol {
        path: String,
        symbol: String,
        changed_lines: LineRange,
    },
    /// Existing extraction reported a parse problem.
    ExtractionUnavailable {
        path: String,
        side: RevisionSide,
        reason: String,
    },
    /// The build lacks an extractor for a language present in the snapshot.
    LanguagesUnavailable {
        side: RevisionSide,
        languages: Vec<String>,
    },
    /// A mentioned id has no entry in the supplied specifications.
    UnresolvedId { path: String, trace_id: String },
    /// A mentioned id belongs to an unavailable specification.
    UnavailableSpecification { path: String, trace_id: String },
    /// An id-bearing marker did not bind to the changed symbol.
    NonBindingMarker {
        path: String,
        trace_id: String,
        line: usize,
    },
    /// Source is outside the Rust-only support boundary.
    UnsupportedSource { path: String },
    /// Existing trace search could not prove a complete result.
    TraceSearchUnavailable { trace_id: String, reason: String },
}

/// Deterministically ordered impact report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ImpactReport {
    /// Base revision label.
    pub base_revision: String,
    /// Head revision label.
    pub head_revision: String,
    /// Changed symbols on each side.
    pub changed_symbols: Vec<ChangedSymbol>,
    /// Unmapped changed ranges.
    pub unmapped_changes: Vec<UnmappedChange>,
    /// Additive test selection with complete reasons.
    pub selected_tests: Vec<SelectedTest>,
    /// Other implementers of changed ids.
    pub related_implementations: Vec<RelatedImplementation>,
    /// False when any changed scope could not be completely interpreted.
    pub selection_complete: bool,
    /// Unlinked and incomplete areas.
    pub findings: Vec<ImpactFinding>,
}

/// Inputs for comparing one dependency's two local and upstream revisions.
pub struct DependencyMoveInput<'a> {
    /// Cargo package name as it appears in Cargo.lock.
    pub dependency: &'a str,
    /// Local repository before the dependency move.
    pub base: &'a SourceSnapshot,
    /// Local repository after the dependency move.
    pub head: &'a SourceSnapshot,
    /// Upstream source before the move, when available.
    pub upstream_base: Option<&'a SourceSnapshot>,
    /// Upstream source after the move, when available.
    pub upstream_head: Option<&'a SourceSnapshot>,
    /// Existing declared marker grammar.
    pub traceability: &'a TraceabilityModel,
}

/// One changed file and the requirement/test items found in its changed contents.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct UpstreamChange {
    /// Upstream repository-relative path.
    pub path: String,
    /// Stable item identifiers mentioned by the changed file.
    pub items: Vec<String>,
}

/// A local source symbol referring to an upstream file or item.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct DependencyLocalReference {
    /// Upstream changed file this reference connects to.
    pub upstream_path: String,
    /// Local source path.
    pub path: String,
    /// Local extracted symbol.
    pub symbol: String,
    /// Upstream file or item that this symbol references.
    pub upstream_reference: String,
}

/// A dependency-move reason chain from upstream source to a local test.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct DependencyTestReason {
    /// Upstream file path.
    pub upstream_path: String,
    /// Upstream item changed in that file.
    pub upstream_item: String,
    /// Local source path containing the reference.
    pub local_path: String,
    /// Local symbol containing the reference.
    pub local_symbol: String,
    /// Selected test path.
    pub test_path: String,
    /// Selected test symbol.
    pub test_symbol: String,
    /// Local call path from the test to the upstream reference symbol.
    pub call_path: Vec<String>,
}

/// Deterministic dependency revision impact report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DependencyMoveReport {
    /// Cargo package name.
    pub dependency: String,
    /// Resolved package revision before the move, when read from Cargo.lock.
    pub old_revision: Option<String>,
    /// Resolved package revision after the move, when read from Cargo.lock.
    pub new_revision: Option<String>,
    /// Which upstream snapshot side was not supplied.
    pub unavailable_upstream: Vec<String>,
    /// Changed upstream files and extracted item identifiers.
    pub upstream_changes: Vec<UpstreamChange>,
    /// Local symbols that refer to changed upstream files or items.
    pub local_references: Vec<DependencyLocalReference>,
    /// Tests selected through an upstream-to-local-to-test chain.
    pub selected_tests: Vec<DependencyTestReason>,
    /// Changed upstream files/items for which no local reference was found.
    pub uncovered: Vec<UpstreamChange>,
    /// True when either upstream snapshot was unavailable or the local source
    /// extraction was incomplete.
    pub unresolved: bool,
}

/// Analyze a dependency revision move using Cargo.lock, upstream snapshots,
/// extracted local symbols, and the existing exact trace index.
#[implements("FR-099-AC-10")]
pub fn analyze_dependency_move(input: DependencyMoveInput<'_>) -> DependencyMoveReport {
    let old_revision = dependency_revision(input.base, input.dependency);
    let new_revision = dependency_revision(input.head, input.dependency);
    let Some((upstream_base, upstream_head)) = input.upstream_base.zip(input.upstream_head) else {
        let unavailable_upstream = unavailable_move_inputs(
            input.upstream_base.is_none(),
            input.upstream_head.is_none(),
            old_revision.is_none(),
            new_revision.is_none(),
        );
        return DependencyMoveReport {
            dependency: input.dependency.to_owned(),
            old_revision,
            new_revision,
            unavailable_upstream,
            upstream_changes: Vec::new(),
            local_references: Vec::new(),
            selected_tests: Vec::new(),
            uncovered: Vec::new(),
            unresolved: true,
        };
    };

    let changed_paths: BTreeSet<&str> = upstream_base
        .files
        .keys()
        .chain(upstream_head.files.keys())
        .filter(|path| upstream_base.files.get(*path) != upstream_head.files.get(*path))
        .map(String::as_str)
        .collect();
    let mut upstream_changes = Vec::new();
    for path in changed_paths {
        let before = upstream_base
            .files
            .get(path)
            .map(String::as_str)
            .unwrap_or("");
        let after = upstream_head
            .files
            .get(path)
            .map(String::as_str)
            .unwrap_or("");
        let mut items = upstream_items(path, before, after);
        items.sort();
        items.dedup();
        upstream_changes.push(UpstreamChange {
            path: path.to_owned(),
            items,
        });
    }

    let local_extraction = extract_snapshot(input.head);
    let mut references = BTreeSet::new();
    let mut reasons = BTreeSet::new();
    let extraction_available = local_extraction.require_available().is_ok();
    let mut unresolved = !extraction_available
        || !local_extraction.diagnostics.is_empty()
        || old_revision.is_none()
        || new_revision.is_none();
    if extraction_available {
        let graph = trace::bind(&local_extraction, input.traceability);
        let symbol_sources: Vec<(&Symbol, String)> = local_extraction
            .symbols
            .iter()
            .map(|symbol| {
                let source = local_extraction
                    .source_of(&symbol.path)
                    .map(|source| symbol.attached_source(source))
                    .unwrap_or_default();
                (symbol, source)
            })
            .collect();
        let test_sources: Vec<(&Symbol, String)> = symbol_sources
            .iter()
            .filter(|(symbol, _)| matches!(symbol.kind, SymbolKind::TestFunction))
            .map(|(symbol, source)| (*symbol, source.clone()))
            .collect();
        for changed in &upstream_changes {
            for item in &changed.items {
                let local_item = qualified_upstream_item(item);
                let mut matching_symbols = Vec::new();
                for (symbol, source) in &symbol_sources {
                    let by_item = contains_upstream_item(source, item)
                        || contains_upstream_item(source, &local_item);
                    let by_path = source.contains(&changed.path);
                    if by_item || by_path {
                        let reference = if by_item {
                            item.as_str()
                        } else {
                            changed.path.as_str()
                        };
                        references.insert(DependencyLocalReference {
                            upstream_path: changed.path.clone(),
                            path: symbol.path.clone(),
                            symbol: symbol.qualified_name.clone(),
                            upstream_reference: reference.to_owned(),
                        });
                        matching_symbols.push((*symbol, by_item, by_path));
                        if matches!(symbol.kind, SymbolKind::TestFunction) {
                            reasons.insert(DependencyTestReason {
                                upstream_path: changed.path.clone(),
                                upstream_item: item.clone(),
                                local_path: symbol.path.clone(),
                                local_symbol: symbol.qualified_name.clone(),
                                test_path: symbol.path.clone(),
                                test_symbol: symbol.qualified_name.clone(),
                                call_path: vec![symbol.qualified_name.clone()],
                            });
                        }
                    }
                }
                for (symbol, by_item, by_path) in &matching_symbols {
                    let reference_name = symbol.bare_name();
                    for (test, test_source) in &test_sources {
                        let calls_reference = invokes_with_item(test_source, reference_name, item);
                        let calls_item_bearing_reference =
                            *by_item && test_source.contains(&format!("{reference_name}("));
                        if (*by_path && calls_reference) || calls_item_bearing_reference {
                            reasons.insert(DependencyTestReason {
                                upstream_path: changed.path.clone(),
                                upstream_item: item.clone(),
                                local_path: symbol.path.clone(),
                                local_symbol: symbol.qualified_name.clone(),
                                test_path: test.path.clone(),
                                test_symbol: test.qualified_name.clone(),
                                call_path: vec![
                                    test.qualified_name.clone(),
                                    symbol.qualified_name.clone(),
                                ],
                            });
                        }
                    }
                }
                if !matching_symbols.is_empty() {
                    if let Ok(matches) = trace_search::search(
                        &graph,
                        &local_extraction,
                        &trace_search::Query::Id(local_item),
                    ) {
                        for test in matches.verifies {
                            if let Some((symbol, _, _)) = matching_symbols.first() {
                                reasons.insert(DependencyTestReason {
                                    upstream_path: changed.path.clone(),
                                    upstream_item: item.clone(),
                                    local_path: symbol.path.clone(),
                                    local_symbol: symbol.qualified_name.clone(),
                                    test_path: test.path,
                                    test_symbol: test.symbol.clone(),
                                    call_path: vec![test.symbol, symbol.qualified_name.clone()],
                                });
                            }
                        }
                    } else {
                        unresolved = true;
                    }
                }
            }
        }
    }
    let uncovered = upstream_changes
        .iter()
        .filter_map(|change| {
            let file_referenced = references.iter().any(|reference| {
                reference.upstream_path == change.path
                    && reference.upstream_reference == change.path
            });
            let unmatched_items: Vec<String> = change
                .items
                .iter()
                .filter(|item| {
                    !file_referenced
                        && !references.iter().any(|reference| {
                            reference.upstream_path == change.path
                                && (reference.upstream_reference == item.as_str()
                                    || reference.upstream_reference
                                        == qualified_upstream_item(item))
                        })
                })
                .cloned()
                .collect();
            (!file_referenced && change.items.is_empty() || !unmatched_items.is_empty()).then(
                || UpstreamChange {
                    path: change.path.clone(),
                    items: unmatched_items,
                },
            )
        })
        .collect();
    let unavailable_upstream =
        unavailable_move_inputs(false, false, old_revision.is_none(), new_revision.is_none());
    DependencyMoveReport {
        dependency: input.dependency.to_owned(),
        old_revision,
        new_revision,
        unavailable_upstream,
        upstream_changes,
        local_references: references.into_iter().collect(),
        selected_tests: reasons.into_iter().collect(),
        uncovered,
        unresolved,
    }
}

#[implements("FR-099-AC-10")]
fn locked_revision(lock: &str, package: &str) -> Option<String> {
    lock.split("[[package]]").skip(1).find_map(|block| {
        let name = block
            .lines()
            .find_map(|line| line.trim().strip_prefix("name = \"")?.strip_suffix('"'))?;
        if name != package {
            return None;
        }
        let source = block
            .lines()
            .find_map(|line| line.trim().strip_prefix("source = \"")?.strip_suffix('"'))?;
        source
            .rsplit_once('#')
            .map(|(_, revision)| revision.to_owned())
    })
}

#[implements("FR-099-AC-10")]
fn dependency_revision(snapshot: &SourceSnapshot, package: &str) -> Option<String> {
    snapshot
        .files
        .get("Cargo.lock")
        .and_then(|lock| locked_revision(lock, package))
        .or_else(|| {
            snapshot
                .files
                .get("Cargo.toml")
                .and_then(|manifest| declared_revision(manifest, package))
        })
}

#[implements("FR-099-AC-10")]
fn declared_revision(manifest: &str, package: &str) -> Option<String> {
    let mut declaration = String::new();
    let mut in_dependency = false;
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_dependency = trimmed == "[dependencies]" || trimmed == "[workspace.dependencies]";
        }
        if in_dependency
            && (trimmed.starts_with(&format!("{package} ="))
                || trimmed.contains(&format!("package = \"{package}\"")))
        {
            declaration.clear();
            declaration.push_str(trimmed);
            if trimmed.contains('}') {
                break;
            }
            continue;
        }
        if !declaration.is_empty() {
            declaration.push_str(trimmed);
            if trimmed.contains('}') {
                break;
            }
        }
    }
    declaration.lines().find_map(|line| {
        let value = line
            .split_once("rev = \"")
            .map(|(_, rest)| rest)
            .or_else(|| line.split_once("tag = \"").map(|(_, rest)| rest))?;
        value
            .split_once('"')
            .map(|(revision, _)| revision.to_owned())
    })
}

#[implements("FR-099-AC-10")]
fn upstream_items(path: &str, before: &str, after: &str) -> Vec<String> {
    if path.ends_with(".json") {
        if let (Ok(before), Ok(after)) = (
            serde_json::from_str::<serde_json::Value>(before),
            serde_json::from_str::<serde_json::Value>(after),
        ) {
            let mut items = BTreeSet::new();
            changed_json_items(None, Some(&before), Some(&after), false, &mut items);
            if !items.is_empty() {
                return items.into_iter().collect();
            }
        }
    }
    if path.ends_with(".rs") {
        let before_items = rust_items(path, before);
        let after_items = rust_items(path, after);
        let mut items = BTreeSet::new();
        for name in before_items.keys().chain(after_items.keys()) {
            if before_items.get(name) != after_items.get(name) {
                items.insert(name.clone());
            }
        }
        if !items.is_empty() {
            return items.into_iter().collect();
        }
    }
    if let Some(item) = artifact_path_item(path) {
        return vec![item];
    }
    Vec::new()
}

#[implements("FR-099-AC-10")]
fn artifact_path_item(path: &str) -> Option<String> {
    let file = path.rsplit('/').next()?;
    let stem = file.split_once('.')?.0;
    let mut parts = stem.split('-');
    let prefix = parts.next()?;
    let number = parts.next()?;
    if matches!(prefix, "FR" | "TC") && number.chars().all(|c| c.is_ascii_digit()) {
        Some(format!("{prefix}-{number}"))
    } else {
        None
    }
}

#[implements("FR-099-AC-10")]
fn qualified_upstream_item(item: &str) -> String {
    if item.starts_with("FSpec-") || !(item.starts_with("FR-") || item.starts_with("TC-")) {
        item.to_owned()
    } else {
        format!("FSpec-{item}")
    }
}

#[implements("FR-099-AC-10")]
fn changed_json_items(
    key: Option<&str>,
    before: Option<&serde_json::Value>,
    after: Option<&serde_json::Value>,
    in_definitions: bool,
    items: &mut BTreeSet<String>,
) {
    if before == after {
        return;
    }
    if in_definitions {
        if let (Some(serde_json::Value::Object(before)), Some(serde_json::Value::Object(after))) =
            (before, after)
        {
            for definition in before.keys().chain(after.keys()).collect::<BTreeSet<_>>() {
                if before.get(definition) != after.get(definition) {
                    items.insert(definition.to_owned());
                }
            }
        } else if let Some(key) = key {
            items.insert(key.to_owned());
        }
        return;
    }
    match (before, after) {
        (Some(serde_json::Value::Object(before)), Some(serde_json::Value::Object(after))) => {
            for key in before.keys().chain(after.keys()).collect::<BTreeSet<_>>() {
                changed_json_items(
                    Some(key),
                    before.get(key),
                    after.get(key),
                    key == "$defs" || key == "definitions",
                    items,
                );
            }
        }
        _ => {
            if let Some(key) = key {
                items.insert(key.to_owned());
            }
        }
    }
}

#[implements("FR-099-AC-10")]
fn rust_items(path: &str, source: &str) -> BTreeMap<String, String> {
    let extraction = extract_file(path, SourceLanguage::Rust, source);
    extraction
        .symbols
        .iter()
        .map(|symbol| {
            (
                symbol.qualified_name.clone(),
                symbol.attached_source(source),
            )
        })
        .collect()
}

#[implements("FR-099-AC-10")]
fn invokes_with_item(source: &str, function: &str, item: &str) -> bool {
    let needle = format!("{function}(");
    source.match_indices(&needle).any(|(start, _)| {
        let arguments_start = start + needle.len();
        source[arguments_start..].find(')').is_some_and(|end| {
            contains_upstream_item(&source[arguments_start..arguments_start + end], item)
        })
    })
}

#[implements("FR-099-AC-10")]
fn contains_upstream_item(source: &str, item: &str) -> bool {
    source.match_indices(item).any(|(start, _)| {
        let before = source[..start].chars().next_back();
        let after = source[start + item.len()..].chars().next();
        let is_identifier_character =
            |character: char| character.is_ascii_alphanumeric() || matches!(character, '_' | '-');
        before.is_none_or(|character| !is_identifier_character(character))
            && after.is_none_or(|character| !is_identifier_character(character))
    })
}

#[implements("FR-099-AC-10")]
fn unavailable_move_inputs(
    missing_base: bool,
    missing_head: bool,
    missing_base_revision: bool,
    missing_head_revision: bool,
) -> Vec<String> {
    [
        missing_base.then(|| "base upstream snapshot".to_owned()),
        missing_head.then(|| "head upstream snapshot".to_owned()),
        missing_base_revision.then(|| "base dependency revision".to_owned()),
        missing_head_revision.then(|| "head dependency revision".to_owned()),
    ]
    .into_iter()
    .flatten()
    .collect()
}

/// Invalid revision or diff input.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ImpactError {
    /// Base or head tree was unavailable.
    #[error("{side} revision is unavailable")]
    UnavailableRevision { side: &'static str },
    /// Revision labels must be present and distinct.
    #[error("invalid revision pair")]
    InvalidRevisionPair,
    /// A changed path is absent from its expected tree.
    #[error("changed path {path} is missing from {side} revision {revision}")]
    MissingChangedPath {
        path: String,
        side: &'static str,
        revision: String,
    },
    /// Changed range has invalid bounds.
    #[error("invalid line range for {path}: {start}..={end}")]
    InvalidLineRange {
        path: String,
        start: usize,
        end: usize,
    },
}

/// Analyze changed lines using the existing source extractor and trace binder.
#[implements("FR-099-AC-1")]
#[implements("FR-099-AC-2")]
#[implements("FR-099-AC-3")]
#[implements("FR-099-AC-4")]
#[implements("FR-099-AC-5")]
#[implements("FR-099-AC-6")]
#[implements("FR-099-AC-7")]
#[implements("FR-099-AC-8")]
#[implements("FR-099-AC-11")]
#[implements("FR-099-CON-4")]
pub fn analyze(input: ImpactInput<'_>) -> Result<ImpactReport, ImpactError> {
    let base = input
        .base
        .ok_or(ImpactError::UnavailableRevision { side: "base" })?;
    let head = input
        .head
        .ok_or(ImpactError::UnavailableRevision { side: "head" })?;
    if base.revision.trim().is_empty()
        || head.revision.trim().is_empty()
        || base.revision == head.revision
    {
        return Err(ImpactError::InvalidRevisionPair);
    }
    for change in input.changes {
        for range in [change.base, change.head].into_iter().flatten() {
            if range.start == 0 || range.end < range.start {
                return Err(ImpactError::InvalidLineRange {
                    path: change.path.clone(),
                    start: range.start,
                    end: range.end,
                });
            }
        }
        if change.base.is_some() && !base.files.contains_key(&change.path) {
            return Err(ImpactError::MissingChangedPath {
                path: change.path.clone(),
                side: "base",
                revision: base.revision.clone(),
            });
        }
        if change.head.is_some() && !head.files.contains_key(&change.path) {
            return Err(ImpactError::MissingChangedPath {
                path: change.path.clone(),
                side: "head",
                revision: head.revision.clone(),
            });
        }
    }
    let base_extraction = extract_snapshot(base);
    let head_extraction = extract_snapshot(head);
    let base_graph = trace::bind(&base_extraction, input.traceability);
    let head_graph = trace::bind(&head_extraction, input.traceability);
    let mut report = ImpactReport {
        base_revision: base.revision.clone(),
        head_revision: head.revision.clone(),
        changed_symbols: Vec::new(),
        unmapped_changes: Vec::new(),
        selected_tests: Vec::new(),
        related_implementations: Vec::new(),
        selection_complete: true,
        findings: Vec::new(),
    };
    let mut selected: BTreeMap<(String, String), BTreeSet<ImpactReason>> = BTreeMap::new();
    let mut related = BTreeSet::new();
    for (side, extraction) in [
        (RevisionSide::Base, &base_extraction),
        (RevisionSide::Head, &head_extraction),
    ] {
        for diagnostic in &extraction.diagnostics {
            report.findings.push(ImpactFinding::ExtractionUnavailable {
                path: diagnostic.path.clone(),
                side,
                reason: diagnostic.reason.clone(),
            });
        }
        if !extraction.unavailable_languages.is_empty() {
            report.findings.push(ImpactFinding::LanguagesUnavailable {
                side,
                languages: extraction
                    .unavailable_languages
                    .iter()
                    .map(|language| language.as_str().to_string())
                    .collect(),
            });
        }
    }
    for change in input.changes {
        for (side, range, extraction, graph) in [
            (
                RevisionSide::Base,
                change.base,
                &base_extraction,
                &base_graph,
            ),
            (
                RevisionSide::Head,
                change.head,
                &head_extraction,
                &head_graph,
            ),
        ] {
            let Some(range) = range else { continue };
            let symbols: Vec<&Symbol> = extraction
                .symbols
                .iter()
                .filter(|symbol| {
                    symbol.path == change.path
                        && symbol.line <= range.end
                        && range.start <= symbol.end_line
                })
                .collect();
            for uncovered in uncovered_ranges(range, &symbols) {
                report.unmapped_changes.push(UnmappedChange {
                    path: change.path.clone(),
                    side,
                    lines: uncovered,
                });
            }
            for symbol in symbols {
                let ids: Vec<String> = graph
                    .implements
                    .iter()
                    .filter(|edge| edge.symbol_id == symbol.id)
                    .map(|edge| edge.trace_id.clone())
                    .collect();
                report.changed_symbols.push(ChangedSymbol {
                    side,
                    id: symbol.id.clone(),
                    path: symbol.path.clone(),
                    name: symbol.qualified_name.clone(),
                    kind: symbol.kind.as_str().to_string(),
                    lines: LineRange {
                        start: symbol.line,
                        end: symbol.end_line,
                    },
                    implemented_ids: ids.clone(),
                });
                if symbol.kind == SymbolKind::TestFunction && side == RevisionSide::Head {
                    selected
                        .entry((symbol.path.clone(), symbol.qualified_name.clone()))
                        .or_default()
                        .insert(ImpactReason::DirectTestChange {
                            changed_path: change.path.clone(),
                            changed_lines: range,
                            test_symbol: symbol.qualified_name.clone(),
                        });
                } else if symbol.kind.carries_implements() && ids.is_empty() {
                    report.findings.push(ImpactFinding::UnlinkedSymbol {
                        path: symbol.path.clone(),
                        symbol: symbol.qualified_name.clone(),
                        changed_lines: range,
                    });
                    for mention in graph.mentions.iter().filter(|mention| {
                        mention.path == symbol.path
                            && mention.symbol.as_deref() == Some(symbol.qualified_name.as_str())
                    }) {
                        let normalized = trace::normalized_trace_id(&mention.trace_id);
                        let available = input
                            .available_spec_ids
                            .iter()
                            .any(|id| trace::normalized_trace_id(id) == normalized);
                        let unavailable = input
                            .unavailable_spec_ids
                            .iter()
                            .any(|id| trace::normalized_trace_id(id) == normalized);
                        let has_nonbinding_attr = symbol.attributes.iter().any(|attribute| {
                            attribute.line == mention.line
                                && attribute.text.contains(&mention.trace_id)
                        });
                        if unavailable {
                            report
                                .findings
                                .push(ImpactFinding::UnavailableSpecification {
                                    path: mention.path.clone(),
                                    trace_id: mention.trace_id.clone(),
                                });
                        } else if !available {
                            report.findings.push(ImpactFinding::UnresolvedId {
                                path: mention.path.clone(),
                                trace_id: mention.trace_id.clone(),
                            });
                        } else if has_nonbinding_attr {
                            report.findings.push(ImpactFinding::NonBindingMarker {
                                path: mention.path.clone(),
                                trace_id: mention.trace_id.clone(),
                                line: mention.line,
                            });
                        }
                    }
                }
                for id in ids {
                    let normalized_id = trace::normalized_trace_id(&id);
                    let available = input
                        .available_spec_ids
                        .iter()
                        .any(|candidate| trace::normalized_trace_id(candidate) == normalized_id);
                    let unavailable = input
                        .unavailable_spec_ids
                        .iter()
                        .any(|candidate| trace::normalized_trace_id(candidate) == normalized_id);
                    if unavailable {
                        report
                            .findings
                            .push(ImpactFinding::UnavailableSpecification {
                                path: symbol.path.clone(),
                                trace_id: id.clone(),
                            });
                    } else if !available {
                        report.findings.push(ImpactFinding::UnresolvedId {
                            path: symbol.path.clone(),
                            trace_id: id.clone(),
                        });
                    }
                    let matches = match trace_search::search(
                        &head_graph,
                        &head_extraction,
                        &trace_search::Query::Id(id.clone()),
                    ) {
                        Ok(matches) => matches,
                        Err(error) => {
                            report.findings.push(ImpactFinding::TraceSearchUnavailable {
                                trace_id: id,
                                reason: error.to_string(),
                            });
                            continue;
                        }
                    };
                    for test in matches.verifies {
                        selected
                            .entry((test.path, test.symbol))
                            .or_default()
                            .insert(ImpactReason::ImplementedId {
                                changed_path: change.path.clone(),
                                changed_lines: range,
                                implementation_path: symbol.path.clone(),
                                implementation: symbol.qualified_name.clone(),
                                trace_id: id.clone(),
                            });
                    }
                    for other in matches
                        .implements
                        .into_iter()
                        .filter(|edge| edge.symbol_id != symbol.id)
                    {
                        let reason = ImpactReason::ImplementedId {
                            changed_path: change.path.clone(),
                            changed_lines: range,
                            implementation_path: symbol.path.clone(),
                            implementation: symbol.qualified_name.clone(),
                            trace_id: id.clone(),
                        };
                        related.insert((other.path, other.symbol, id.clone(), reason));
                    }
                }
            }
            if !change.path.ends_with(".rs") {
                report.findings.push(ImpactFinding::UnsupportedSource {
                    path: change.path.clone(),
                });
            }
        }
    }
    report.selected_tests = selected
        .into_iter()
        .map(|((path, name), reasons)| SelectedTest {
            path,
            filter: name.clone(),
            name,
            reasons: reasons.into_iter().collect(),
        })
        .collect();
    report.related_implementations = related
        .into_iter()
        .map(|(path, name, trace_id, reason)| RelatedImplementation {
            path,
            name,
            trace_id,
            reason,
        })
        .collect();
    report.selection_complete = report.findings.is_empty() && report.unmapped_changes.is_empty();
    report.changed_symbols.sort_by(|a, b| {
        (&a.path, a.lines.start, &a.name, a.side).cmp(&(&b.path, b.lines.start, &b.name, b.side))
    });
    report
        .unmapped_changes
        .sort_by(|a, b| (&a.path, a.side, a.lines).cmp(&(&b.path, b.side, b.lines)));
    report
        .findings
        .sort_by_key(|item| serde_json::to_string(item).unwrap_or_default());
    Ok(report)
}

#[implements("FR-099-AC-2")]
fn uncovered_ranges(changed: LineRange, symbols: &[&Symbol]) -> Vec<LineRange> {
    let mut covered: Vec<(usize, usize)> = symbols
        .iter()
        .filter_map(|symbol| {
            let start = symbol.line.max(changed.start);
            let end = symbol.end_line.min(changed.end);
            (start <= end).then_some((start, end))
        })
        .collect();
    covered.sort_unstable();
    let mut result = Vec::new();
    let mut next = changed.start;
    for (start, end) in covered {
        if start > next {
            result.push(LineRange {
                start: next,
                end: start - 1,
            });
        }
        if end >= next {
            next = match end.checked_add(1) {
                Some(after) => after,
                None => return result,
            };
        }
    }
    if next <= changed.end {
        result.push(LineRange {
            start: next,
            end: changed.end,
        });
    }
    result
}

#[implements("FR-099-AC-1")]
#[implements("FR-099-AC-8")]
#[implements("FR-099-AC-10")]
fn extract_snapshot(snapshot: &SourceSnapshot) -> SymbolExtraction {
    let mut all = SymbolExtraction::default();
    for (path, source) in &snapshot.files {
        if !path.ends_with(".rs") {
            continue;
        }
        let part = extract_file(path, SourceLanguage::Rust, source);
        all.symbols.extend(part.symbols);
        all.files.extend(part.files);
        all.diagnostics.extend(part.diagnostics);
        all.unavailable_languages.extend(part.unavailable_languages);
    }
    all.symbols.sort_by(|a, b| {
        (&a.path, a.line, &a.qualified_name).cmp(&(&b.path, b.line, &b.qualified_name))
    });
    all.files.sort_by(|a, b| a.path.cmp(&b.path));
    all.diagnostics
        .sort_by(|a, b| (&a.path, &a.reason).cmp(&(&b.path, &b.reason)));
    all.unavailable_languages.sort();
    all.unavailable_languages.dedup();
    all
}

#[cfg(test)]
mod tests {
    use super::*;
    use ix_trace_rs::trace;

    fn snapshot(revision: &str, source: &str) -> SourceSnapshot {
        SourceSnapshot {
            revision: revision.into(),
            files: BTreeMap::from([("src/lib.rs".into(), source.into())]),
        }
    }
    fn run(base: &str, head: &str, change: FileChange) -> Result<ImpactReport, ImpactError> {
        let base = snapshot("base", base);
        let head = snapshot("head", head);
        analyze(ImpactInput {
            base: Some(&base),
            head: Some(&head),
            changes: &[change],
            traceability: &TraceabilityModel::default(),
            available_spec_ids: &BTreeSet::new(),
            unavailable_spec_ids: &BTreeSet::new(),
        })
    }

    fn model_with_implements_form(root: &std::path::Path) -> crate::Registry {
        let module = root.join("module");
        std::fs::create_dir_all(&module).unwrap();
        let manifest = serde_json::json!({
            "name": "impact-fixture", "version": "0.1.0", "artifact_types": [],
            "traceability": { "trace_tags": {
                "markers": [{
                    "name": "rust-trace", "language": "rust",
                    "pattern": r#"#\[trace\(([^)]*)\)\]"#,
                    "template": "#[trace({ids})]"
                }],
                "implements": [{
                    "name": "rust-implements", "language": "rust",
                    "pattern": r#"#\[implements\(([^)]*)\)\]"#,
                    "template": "#[implements({ids})]", "syntax": "rust_attribute"
                }]
            }}
        });
        std::fs::write(
            module.join("manifest.yaml"),
            serde_json::to_string(&manifest).unwrap(),
        )
        .unwrap();
        crate::Registry::load_module(&module).unwrap()
    }

    #[trace("FR-099-AC-1")]
    #[trace("FR-099-AC-2")]
    #[test]
    fn maps_both_sides_and_reports_unmapped_ranges() {
        let report = run(
            "fn old() {}\n",
            "fn new() {}\n",
            FileChange {
                path: "src/lib.rs".into(),
                base: Some(LineRange { start: 1, end: 1 }),
                head: Some(LineRange { start: 1, end: 1 }),
            },
        )
        .unwrap();
        assert_eq!(report.changed_symbols.len(), 2);
        let report = run(
            "\n",
            "\n",
            FileChange {
                path: "src/lib.rs".into(),
                base: Some(LineRange { start: 1, end: 1 }),
                head: Some(LineRange { start: 1, end: 1 }),
            },
        )
        .unwrap();
        assert_eq!(report.unmapped_changes.len(), 2);
        let report = run(
            "use std::fmt;\nfn changed() {}\n",
            "use std::fmt;\nfn changed() {}\n",
            FileChange {
                path: "src/lib.rs".into(),
                base: Some(LineRange { start: 1, end: 2 }),
                head: Some(LineRange { start: 1, end: 2 }),
            },
        )
        .unwrap();
        assert_eq!(
            report.unmapped_changes[0].lines,
            LineRange { start: 1, end: 1 }
        );
        let report = run(
            "mod outer { fn changed() {} }\n",
            "mod outer { fn changed() {} }\n",
            FileChange {
                path: "src/lib.rs".into(),
                base: Some(LineRange { start: 1, end: 1 }),
                head: Some(LineRange { start: 1, end: 1 }),
            },
        )
        .unwrap();
        assert!(report
            .changed_symbols
            .iter()
            .any(|symbol| symbol.name == "outer"));
        assert!(report
            .changed_symbols
            .iter()
            .any(|symbol| symbol.name.ends_with("changed")));
    }

    #[trace("FR-099-AC-1")]
    #[test]
    fn missing_revision_is_an_error() {
        let head = snapshot("head", "fn changed() {}\n");
        let empty = BTreeSet::new();
        let result = analyze(ImpactInput {
            base: None,
            head: Some(&head),
            changes: &[],
            traceability: &TraceabilityModel::default(),
            available_spec_ids: &empty,
            unavailable_spec_ids: &empty,
        });
        assert_eq!(
            result,
            Err(ImpactError::UnavailableRevision { side: "base" })
        );
    }

    #[trace("FR-099-AC-3")]
    #[trace("FR-099-AC-5")]
    #[test]
    fn changed_test_gets_a_filter_and_direct_reason() {
        let report = run(
            "",
            "#[test]\nfn added_test() {}\n",
            FileChange {
                path: "src/lib.rs".into(),
                base: None,
                head: Some(LineRange { start: 1, end: 2 }),
            },
        )
        .unwrap();
        assert_eq!(report.selected_tests[0].filter, "added_test");
        assert!(matches!(
            report.selected_tests[0].reasons[0],
            ImpactReason::DirectTestChange { .. }
        ));
    }

    #[trace("FR-099-AC-3")]
    #[test]
    fn deleted_test_is_not_selected() {
        let report = run(
            "#[test]\nfn deleted_test() {}\n",
            "",
            FileChange {
                path: "src/lib.rs".into(),
                base: Some(LineRange { start: 1, end: 2 }),
                head: None,
            },
        )
        .unwrap();
        assert!(report.selected_tests.is_empty());
    }

    #[trace("FR-099-AC-3")]
    #[trace("FR-099-AC-4")]
    #[test]
    fn changed_base_implementation_does_not_select_a_test_deleted_at_head() {
        let root = tempfile::tempdir().unwrap();
        let registry = model_with_implements_form(root.path());
        let model = registry.traceability().unwrap();
        let base = snapshot(
            "base",
            "#[implements(\"FR-099-AC-4\")]\nfn changed() {}\n#[test]\n#[trace(\"FR-099-AC-4\")]\nfn deleted_test() {}\n",
        );
        let head = snapshot(
            "head",
            "#[implements(\"FR-099-AC-4\")]\nfn changed() { let _ = 1; }\n",
        );
        let available = BTreeSet::from(["FR-099-AC-4".to_string()]);
        let unavailable = BTreeSet::new();
        let report = analyze(ImpactInput {
            base: Some(&base),
            head: Some(&head),
            changes: &[FileChange {
                path: "src/lib.rs".into(),
                base: Some(LineRange { start: 2, end: 2 }),
                head: Some(LineRange { start: 2, end: 2 }),
            }],
            traceability: model,
            available_spec_ids: &available,
            unavailable_spec_ids: &unavailable,
        })
        .unwrap();
        assert!(!report
            .selected_tests
            .iter()
            .any(|test| test.name == "deleted_test"));
    }

    #[trace("FR-099-AC-4")]
    #[trace("FR-099-AC-5")]
    #[trace("FR-099-AC-6")]
    #[test]
    fn exact_implemented_id_selects_tests_and_lists_twin_implementers() {
        let root = tempfile::tempdir().unwrap();
        let registry = model_with_implements_form(root.path());
        let model = registry.traceability().unwrap();
        let source = "#[implements(\"fr_099_ac_4\")]\nfn changed() {}\n#[implements(\"FR-099-AC-4\")]\nfn twin() {}\n#[test]\n#[trace(\"FR-099-AC-4\")]\nfn matching_test() {}\n#[test]\n#[trace(\"FR-099-AC-3\")]\nfn different_test() {}\n";
        let base = snapshot("base", "");
        let head = snapshot("head", source);
        let available = BTreeSet::from(["FR-099-AC-4".to_string(), "FR-099-AC-3".to_string()]);
        let unavailable = BTreeSet::new();
        let report = analyze(ImpactInput {
            base: Some(&base),
            head: Some(&head),
            changes: &[FileChange {
                path: "src/lib.rs".into(),
                base: None,
                head: Some(LineRange { start: 1, end: 2 }),
            }],
            traceability: model,
            available_spec_ids: &available,
            unavailable_spec_ids: &unavailable,
        })
        .unwrap();
        assert!(report
            .selected_tests
            .iter()
            .any(|test| test.name == "matching_test"));
        assert!(!report
            .selected_tests
            .iter()
            .any(|test| test.name == "different_test"));
        assert!(report
            .related_implementations
            .iter()
            .any(|item| item.name == "twin" && item.trace_id == "fr_099_ac_4"));
    }

    #[trace("FR-099-AC-6")]
    #[test]
    fn requirement_and_repository_prefixed_ids_remain_distinct() {
        let root = tempfile::tempdir().unwrap();
        let registry = model_with_implements_form(root.path());
        let model = registry.traceability().unwrap();
        let source = "#[implements(\"FR-099\")]\nfn requirement_impl() {}\n#[implements(\"FR-099-AC-4\")]\nfn criterion_impl() {}\n#[implements(\"FSpec-FR-099-AC-4\")]\nfn foreign_impl() {}\n#[test]\n#[trace(\"FR-099\")]\nfn requirement_test() {}\n#[test]\n#[trace(\"FR-099-AC-4\")]\nfn criterion_test() {}\n#[test]\n#[trace(\"FSpec-FR-099-AC-4\")]\nfn foreign_test() {}\n";
        let base = snapshot("base", "");
        let head = snapshot("head", source);
        let available = BTreeSet::from([
            "FR-099".to_string(),
            "FR-099-AC-4".to_string(),
            "FSpec-FR-099-AC-4".to_string(),
        ]);
        let unavailable = BTreeSet::new();
        let run_range = |start, end| {
            analyze(ImpactInput {
                base: Some(&base),
                head: Some(&head),
                changes: &[FileChange {
                    path: "src/lib.rs".into(),
                    base: None,
                    head: Some(LineRange { start, end }),
                }],
                traceability: model,
                available_spec_ids: &available,
                unavailable_spec_ids: &unavailable,
            })
            .unwrap()
        };

        let requirement = run_range(1, 2);
        assert_eq!(
            requirement
                .selected_tests
                .iter()
                .map(|test| test.name.as_str())
                .collect::<Vec<_>>(),
            ["requirement_test"]
        );
        let criterion = run_range(3, 4);
        assert_eq!(
            criterion
                .selected_tests
                .iter()
                .map(|test| test.name.as_str())
                .collect::<Vec<_>>(),
            ["criterion_test"]
        );
        assert!(!criterion
            .selected_tests
            .iter()
            .any(|test| test.name == "foreign_test"));
    }

    #[trace("FR-099-AC-8")]
    #[test]
    fn unlinked_code_is_reported_with_empty_test_selection() {
        let report = run(
            "fn helper() {}\n",
            "fn helper() { let _ = 1; }\n",
            FileChange {
                path: "src/lib.rs".into(),
                base: Some(LineRange { start: 1, end: 1 }),
                head: Some(LineRange { start: 1, end: 1 }),
            },
        )
        .unwrap();
        assert!(report.selected_tests.is_empty());
        assert!(report.findings.iter().any(|item| matches!(item, ImpactFinding::UnlinkedSymbol { symbol, .. } if symbol == "helper")));
        assert!(!report.selection_complete);
    }

    #[trace("FR-099-AC-7")]
    #[trace("FR-099-CON-4")]
    #[test]
    fn unmatched_requirement_mentions_are_classified_by_spec_availability() {
        let root = tempfile::tempdir().unwrap();
        let registry = model_with_implements_form(root.path());
        let model = registry.traceability().unwrap();
        let source = "#[some::implements(\"FR-099-AC-4\")]\nfn changed() {}\n";
        let base = snapshot("base", "");
        let head = snapshot("head", source);
        let unavailable = BTreeSet::from(["fr_099_ac_4".to_string()]);
        let report = analyze(ImpactInput {
            base: Some(&base),
            head: Some(&head),
            changes: &[FileChange {
                path: "src/lib.rs".into(),
                base: None,
                head: Some(LineRange { start: 1, end: 2 }),
            }],
            traceability: model,
            available_spec_ids: &BTreeSet::new(),
            unavailable_spec_ids: &unavailable,
        })
        .unwrap();
        assert!(report.findings.iter().any(|item| matches!(item, ImpactFinding::UnavailableSpecification { trace_id, .. } if trace_id == "FR-099-AC-4")));
        assert!(!report.selection_complete);

        let known = BTreeSet::from(["FR-099-AC-4".to_string()]);
        let report = analyze(ImpactInput {
            base: Some(&base),
            head: Some(&head),
            changes: &[FileChange {
                path: "src/lib.rs".into(),
                base: None,
                head: Some(LineRange { start: 1, end: 2 }),
            }],
            traceability: model,
            available_spec_ids: &known,
            unavailable_spec_ids: &BTreeSet::new(),
        })
        .unwrap();
        assert!(report.findings.iter().any(|item| matches!(item, ImpactFinding::NonBindingMarker { trace_id, .. } if trace_id == "FR-099-AC-4")));

        let report = analyze(ImpactInput {
            base: Some(&base),
            head: Some(&head),
            changes: &[FileChange {
                path: "src/lib.rs".into(),
                base: None,
                head: Some(LineRange { start: 1, end: 2 }),
            }],
            traceability: model,
            available_spec_ids: &BTreeSet::new(),
            unavailable_spec_ids: &BTreeSet::new(),
        })
        .unwrap();
        assert!(report.findings.iter().any(|item| matches!(item, ImpactFinding::UnresolvedId { trace_id, .. } if trace_id == "FR-099-AC-4")));
    }

    #[trace("FR-099-AC-8")]
    #[test]
    fn unsupported_changed_source_marks_selection_incomplete() {
        let base = SourceSnapshot {
            revision: "base".into(),
            files: BTreeMap::from([("src/lib.py".into(), "def changed(): pass\n".into())]),
        };
        let head = SourceSnapshot {
            revision: "head".into(),
            files: BTreeMap::from([("src/lib.py".into(), "def changed(): return 1\n".into())]),
        };
        let report = analyze(ImpactInput {
            base: Some(&base),
            head: Some(&head),
            changes: &[FileChange {
                path: "src/lib.py".into(),
                base: Some(LineRange { start: 1, end: 1 }),
                head: Some(LineRange { start: 1, end: 1 }),
            }],
            traceability: &TraceabilityModel::default(),
            available_spec_ids: &BTreeSet::new(),
            unavailable_spec_ids: &BTreeSet::new(),
        })
        .unwrap();
        assert!(report.findings.iter().any(
            |item| matches!(item, ImpactFinding::UnsupportedSource { path } if path == "src/lib.py")
        ));
        assert!(!report.selection_complete);
        assert!(report.selected_tests.is_empty());
    }

    #[trace("FR-099-AC-8")]
    #[test]
    fn parser_failure_marks_selection_incomplete() {
        let broken = include_str!("../tests/fixtures/symbols/broken/truncated.rs");
        let report = run(
            "",
            broken,
            FileChange {
                path: "src/lib.rs".into(),
                base: None,
                head: Some(LineRange { start: 1, end: 4 }),
            },
        )
        .unwrap();
        assert!(report.findings.iter().any(
            |item| matches!(item, ImpactFinding::ExtractionUnavailable { path, .. } if path == "src/lib.rs")
        ));
        assert!(!report.selection_complete);
    }

    #[trace("FR-099-AC-8")]
    #[test]
    fn unchanged_malformed_file_keeps_whole_snapshot_selection_incomplete() {
        let malformed = include_str!("../tests/fixtures/symbols/broken/truncated.rs").to_string();
        let files = BTreeMap::from([
            (
                "src/lib.rs".to_string(),
                "#[test]\nfn touched() {}\n".to_string(),
            ),
            ("src/broken.rs".to_string(), malformed),
        ]);
        let base = SourceSnapshot {
            revision: "base".into(),
            files: files.clone(),
        };
        let head = SourceSnapshot {
            revision: "head".into(),
            files,
        };
        let report = analyze(ImpactInput {
            base: Some(&base),
            head: Some(&head),
            changes: &[FileChange {
                path: "src/lib.rs".into(),
                base: Some(LineRange { start: 2, end: 2 }),
                head: Some(LineRange { start: 2, end: 2 }),
            }],
            traceability: &TraceabilityModel::default(),
            available_spec_ids: &BTreeSet::new(),
            unavailable_spec_ids: &BTreeSet::new(),
        })
        .unwrap();
        assert!(report.findings.iter().any(
            |finding| matches!(finding, ImpactFinding::ExtractionUnavailable { path, .. } if path == "src/broken.rs")
        ));
        assert!(!report.selection_complete);
    }

    #[trace("FR-099-AC-11")]
    #[test]
    fn repeated_inputs_have_identical_bytes() {
        let change = FileChange {
            path: "src/lib.rs".into(),
            base: Some(LineRange { start: 1, end: 1 }),
            head: Some(LineRange { start: 1, end: 1 }),
        };
        let a = run(
            "fn helper() {}\n",
            "fn helper() { let _ = 1; }\n",
            change.clone(),
        )
        .unwrap();
        let b = run("fn helper() {}\n", "fn helper() { let _ = 1; }\n", change).unwrap();
        assert_eq!(
            serde_json::to_vec(&a).unwrap(),
            serde_json::to_vec(&b).unwrap()
        );
    }

    #[trace("FR-099-AC-10")]
    #[test]
    fn dependency_move_links_upstream_items_to_local_tests_and_reports_unmatched_items() {
        let root = tempfile::tempdir().unwrap();
        let registry = model_with_implements_form(root.path());
        let model = registry.traceability().unwrap();
        let lock = |revision: &str| {
            format!(
                "[[package]]\nname = \"filament-specification-qualification\"\nversion = \"0.1.1\"\nsource = \"git+https://example.invalid/spec?rev={revision}#{revision}\"\n"
            )
        };
        let local_before = SourceSnapshot {
            revision: "local-base".into(),
            files: BTreeMap::from([("Cargo.lock".into(), lock("old-rev"))]),
        };
        let local_after = SourceSnapshot {
            revision: "local-head".into(),
            files: BTreeMap::from([
                ("Cargo.lock".into(), lock("new-rev")),
                (
                    "tests/probability.rs".into(),
                    "fn provider_definition(name: &str) { let _ = \"proposals/backend-provider-v1/schema.json\"; let _ = name; }\n#[test]\n#[trace(\"FSpec-TC-374\")]\nfn probability_from_source() { provider_definition(\"CertificateIdentity\"); let _ = \"TC-374\"; }\n".into(),
                ),
            ]),
        };
        let upstream_before = SourceSnapshot {
            revision: "old-rev".into(),
            files: BTreeMap::from([
                (
                    "spec/test-cases/TC-374-probability.md".into(),
                    "TC-374 legacy semantics".into(),
                ),
                ("src/logic.rs".into(), "fn old_logic() {}".into()),
                (
                    "proposals/backend-provider-v1/schema.json".into(),
                    r#"{"$defs":{"CertificateIdentity":{"type":"string"}}}"#.into(),
                ),
            ]),
        };
        let upstream_after = SourceSnapshot {
            revision: "new-rev".into(),
            files: BTreeMap::from([
                (
                    "spec/test-cases/TC-374-probability.md".into(),
                    "TC-374 updated semantics".into(),
                ),
                (
                    "src/logic.rs".into(),
                    "fn old_logic() { let _ = 1; }".into(),
                ),
                (
                    "proposals/backend-provider-v1/schema.json".into(),
                    r#"{"$defs":{"CertificateIdentity":{"type":"object"}}}"#.into(),
                ),
            ]),
        };
        let report = analyze_dependency_move(DependencyMoveInput {
            dependency: "filament-specification-qualification",
            base: &local_before,
            head: &local_after,
            upstream_base: Some(&upstream_before),
            upstream_head: Some(&upstream_after),
            traceability: model,
        });
        assert_eq!(report.old_revision.as_deref(), Some("old-rev"));
        assert_eq!(report.new_revision.as_deref(), Some("new-rev"));
        assert_eq!(report.upstream_changes.len(), 3);
        assert!(report.upstream_changes.iter().any(|change| {
            change.path == "spec/test-cases/TC-374-probability.md"
                && change.items.contains(&"TC-374".to_owned())
        }));
        assert!(report.selected_tests.iter().any(|reason| {
            reason.test_symbol == "probability_from_source"
                && reason.upstream_item == "TC-374"
                && reason.local_symbol == "probability_from_source"
                && reason.call_path == ["probability_from_source"]
        }));
        assert!(report.upstream_changes.iter().any(|change| {
            change.path == "proposals/backend-provider-v1/schema.json"
                && change.items == ["CertificateIdentity"]
        }));
        assert!(report.selected_tests.iter().any(|reason| {
            reason.test_symbol == "probability_from_source"
                && reason.upstream_item == "CertificateIdentity"
                && reason.local_symbol == "provider_definition"
                && reason.call_path == ["probability_from_source", "provider_definition"]
        }));
        assert!(report
            .uncovered
            .iter()
            .any(|change| { change.path == "src/logic.rs" && change.items == ["old_logic"] }));
        assert!(!report.unresolved);
        assert_eq!(
            serde_json::to_vec(&report).unwrap(),
            serde_json::to_vec(&analyze_dependency_move(DependencyMoveInput {
                dependency: "filament-specification-qualification",
                base: &local_before,
                head: &local_after,
                upstream_base: Some(&upstream_before),
                upstream_head: Some(&upstream_after),
                traceability: model,
            }))
            .unwrap()
        );
    }

    #[trace("FR-099-AC-10")]
    #[test]
    fn dependency_move_does_not_match_a_short_item_id_prefix() {
        let lock = |revision: &str| {
            format!(
                "[[package]]\nname = \"example-dependency\"\nversion = \"0.1.0\"\nsource = \"git+https://example.invalid/dependency?rev={revision}#{revision}\"\n"
            )
        };
        let base = SourceSnapshot {
            revision: "local-base".into(),
            files: BTreeMap::from([("Cargo.lock".into(), lock("old-rev"))]),
        };
        let head = SourceSnapshot {
            revision: "local-head".into(),
            files: BTreeMap::from([
                ("Cargo.lock".into(), lock("new-rev")),
                (
                    "tests/unrelated.rs".into(),
                    "#[test]\n#[trace(\"FSpec-TC-374\")]\nfn unrelated_test() {}\n".into(),
                ),
            ]),
        };
        let upstream_base = SourceSnapshot {
            revision: "old-rev".into(),
            files: BTreeMap::from([(
                "spec/test-cases/TC-37-short-id.md".into(),
                "TC-37 old description".into(),
            )]),
        };
        let upstream_head = SourceSnapshot {
            revision: "new-rev".into(),
            files: BTreeMap::from([(
                "spec/test-cases/TC-37-short-id.md".into(),
                "TC-37 changed description".into(),
            )]),
        };

        let report = analyze_dependency_move(DependencyMoveInput {
            dependency: "example-dependency",
            base: &base,
            head: &head,
            upstream_base: Some(&upstream_base),
            upstream_head: Some(&upstream_head),
            traceability: &TraceabilityModel::default(),
        });

        assert!(!report
            .selected_tests
            .iter()
            .any(|selected| selected.test_symbol == "unrelated_test"));
        assert!(!report.local_references.iter().any(|reference| {
            reference.upstream_path == "spec/test-cases/TC-37-short-id.md"
                && reference.upstream_reference == "TC-37"
        }));
        assert!(report.uncovered.iter().any(|change| {
            change.path == "spec/test-cases/TC-37-short-id.md" && change.items == ["TC-37"]
        }));
    }

    #[trace("FR-099-AC-10")]
    #[test]
    fn unavailable_upstream_tree_remains_explicit_and_unresolved() {
        let local = snapshot("local", "");
        let report = analyze_dependency_move(DependencyMoveInput {
            dependency: "unknown-package",
            base: &local,
            head: &local,
            upstream_base: None,
            upstream_head: None,
            traceability: &TraceabilityModel::default(),
        });
        assert!(report.unresolved);
        assert_eq!(
            report.unavailable_upstream,
            [
                "base upstream snapshot",
                "head upstream snapshot",
                "base dependency revision",
                "head dependency revision"
            ]
        );
        assert!(report.upstream_changes.is_empty());
    }
}
