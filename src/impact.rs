//! Source-diff impact selection using FR-062 bindings and FR-077 exact lookup.

use std::collections::{BTreeMap, BTreeSet};

use ix_trace_rs::implements;
use quire_code_extraction::tree_sitter::Node;
use quire_code_extraction::{parse_file, Language as CodeLanguage};
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
    /// Optional finite reverse-call search depth for unlinked changed symbols.
    /// `None` disables caller following; `Some(0)` reports truncation if any
    /// callers exist without traversing them.
    pub caller_search_depth: Option<usize>,
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
    /// Linked caller reached from an unlinked changed symbol.
    CallerFollowing {
        changed_path: String,
        changed_lines: LineRange,
        changed_symbol: String,
        call_path: Vec<String>,
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
    /// Caller following encountered a truncated call graph or unresolved call.
    CallerSearchIncomplete { path: String, symbol: String },
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
    /// Optional bounded caller searches for unlinked changed symbols.
    pub caller_searches: Vec<CallerSearch>,
    /// False when any changed scope could not be completely interpreted.
    pub selection_complete: bool,
    /// Unlinked and incomplete areas.
    pub findings: Vec<ImpactFinding>,
}

/// Caller-search result for one unlinked changed symbol.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CallerSearch {
    /// Changed source path.
    pub changed_path: String,
    /// Qualified changed symbol.
    pub changed_symbol: String,
    /// Requested maximum caller depth.
    pub depth_limit: usize,
    /// Nearest linked callers and their test evidence.
    pub linked_callers: Vec<LinkedCaller>,
    /// Unresolved call sites whose callee name could refer to this symbol.
    pub unresolved_calls: Vec<UnresolvedCallSite>,
    /// Whether additional caller edges remained beyond `depth_limit`.
    pub truncated: bool,
}

/// A linked caller reached during caller following.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct LinkedCaller {
    /// Source path containing the caller.
    pub path: String,
    /// Qualified caller symbol.
    pub symbol: String,
    /// Caller distance from the changed symbol.
    pub depth: usize,
    /// Requirement ids linked to the caller.
    pub linked_ids: Vec<String>,
    /// Full path from the changed symbol through callers to this symbol.
    pub call_path: Vec<String>,
}

/// A call site that cannot be assigned to one extracted Rust symbol path.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct UnresolvedCallSite {
    /// Source path containing the call.
    pub path: String,
    /// Qualified caller symbol.
    pub caller: String,
    /// One-based line of the call expression.
    pub line: usize,
    /// Written callee expression.
    pub callee: String,
    /// Why the call could not be resolved syntactically.
    pub reason: String,
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
#[implements("FR-099-AC-9")]
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
        caller_searches: Vec::new(),
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
    if let Some(depth_limit) = input.caller_search_depth {
        let caller_data = caller_call_graph(&head_extraction);
        for changed in report.changed_symbols.iter().filter(|changed| {
            changed.side == RevisionSide::Head
                && changed.kind == SymbolKind::Function.as_str()
                && changed.implemented_ids.is_empty()
        }) {
            let Some(symbol) = head_extraction.symbols.iter().find(|symbol| {
                symbol.path == changed.path
                    && symbol.qualified_name == changed.name
                    && symbol.kind.carries_implements()
            }) else {
                continue;
            };
            let search = search_callers(
                symbol,
                depth_limit,
                &head_extraction,
                &head_graph,
                &caller_data,
            );
            if search.truncated || !search.unresolved_calls.is_empty() {
                report.findings.push(ImpactFinding::CallerSearchIncomplete {
                    path: changed.path.clone(),
                    symbol: changed.name.clone(),
                });
            }
            for caller in &search.linked_callers {
                for trace_id in &caller.linked_ids {
                    if let Ok(matches) = trace_search::search(
                        &head_graph,
                        &head_extraction,
                        &trace_search::Query::Id(trace_id.clone()),
                    ) {
                        for test in matches.verifies {
                            selected
                                .entry((test.path, test.symbol))
                                .or_default()
                                .insert(ImpactReason::CallerFollowing {
                                    changed_path: changed.path.clone(),
                                    changed_lines: changed.lines,
                                    changed_symbol: changed.name.clone(),
                                    call_path: caller.call_path.clone(),
                                    trace_id: trace_id.clone(),
                                });
                        }
                    } else {
                        report.findings.push(ImpactFinding::TraceSearchUnavailable {
                            trace_id: trace_id.clone(),
                            reason: "caller-following trace lookup failed".to_string(),
                        });
                    }
                }
                if let Some(test_symbol) = head_extraction.symbols.iter().find(|candidate| {
                    candidate.path == caller.path
                        && candidate.qualified_name == caller.symbol
                        && candidate.kind == SymbolKind::TestFunction
                }) {
                    let verifies: BTreeSet<&str> = head_graph
                        .verifies
                        .iter()
                        .filter(|edge| edge.symbol_id == test_symbol.id)
                        .map(|edge| edge.trace_id.as_str())
                        .collect();
                    if !verifies.is_empty() {
                        let reasons = selected
                            .entry((caller.path.clone(), caller.symbol.clone()))
                            .or_default();
                        for trace_id in verifies {
                            reasons.insert(ImpactReason::CallerFollowing {
                                changed_path: changed.path.clone(),
                                changed_lines: changed.lines,
                                changed_symbol: changed.name.clone(),
                                call_path: caller.call_path.clone(),
                                trace_id: trace_id.to_string(),
                            });
                        }
                    }
                }
            }
            report.caller_searches.push(search);
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
    report.caller_searches.sort_by(|left, right| {
        (&left.changed_path, &left.changed_symbol)
            .cmp(&(&right.changed_path, &right.changed_symbol))
    });
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SymbolKey {
    path: String,
    name: String,
}

impl SymbolKey {
    fn from_symbol(symbol: &Symbol) -> Self {
        Self {
            path: symbol.path.clone(),
            name: symbol.qualified_name.clone(),
        }
    }

    fn display(&self) -> String {
        format!("{}::{}", self.path, self.name)
    }
}

#[derive(Debug, Clone)]
struct ParsedCall {
    caller: SymbolKey,
    written: String,
    callee_path: Option<String>,
    callee_hint: Option<String>,
    line: usize,
    unresolved_reason: Option<&'static str>,
}

#[derive(Debug, Clone)]
struct CallerEdge {
    caller: SymbolKey,
}

#[derive(Debug, Clone)]
struct UnresolvedCall {
    caller: SymbolKey,
    written: String,
    hint: Option<String>,
    line: usize,
    reason: &'static str,
}

#[derive(Debug, Default)]
struct CallerCallGraph {
    callers_by_callee: BTreeMap<SymbolKey, Vec<CallerEdge>>,
    unresolved: Vec<UnresolvedCall>,
}

#[implements("FR-099-AC-9")]
fn caller_call_graph(extraction: &SymbolExtraction) -> CallerCallGraph {
    let mut graph = CallerCallGraph::default();
    for file in &extraction.files {
        if file.language != SourceLanguage::Rust {
            continue;
        }
        let Ok(parsed) = parse_file(CodeLanguage::Rust, &file.path, &file.source) else {
            continue;
        };
        if parsed.diagnostic().is_some() {
            continue;
        }
        let file_symbols: Vec<&Symbol> = extraction
            .symbols
            .iter()
            .filter(|symbol| symbol.path == file.path)
            .collect();
        let aliases = import_aliases(parsed.root_node(), &file.source);
        let mut calls = Vec::new();
        collect_calls(
            parsed.root_node(),
            &file.source,
            &file_symbols,
            None,
            &mut calls,
        );
        for call in calls {
            let Some(callee_path) = call.callee_path.as_deref() else {
                graph.unresolved.push(UnresolvedCall {
                    caller: call.caller,
                    written: call.written,
                    hint: call.callee_hint,
                    line: call.line,
                    reason: call.unresolved_reason.unwrap_or("dynamic-callee"),
                });
                continue;
            };
            if let Some(target) = aliases.get(callee_path) {
                graph.unresolved.push(UnresolvedCall {
                    caller: call.caller,
                    written: call.written,
                    hint: target.rsplit("::").next().map(str::to_string),
                    line: call.line,
                    reason: "import-alias-call",
                });
                continue;
            }
            match resolve_callee(callee_path, &call.caller, &extraction.symbols) {
                Ok(callee) => graph
                    .callers_by_callee
                    .entry(callee)
                    .or_default()
                    .push(CallerEdge {
                        caller: call.caller,
                    }),
                Err(reason) => graph.unresolved.push(UnresolvedCall {
                    caller: call.caller,
                    written: call.written,
                    hint: call.callee_hint,
                    line: call.line,
                    reason,
                }),
            }
        }
    }
    for callers in graph.callers_by_callee.values_mut() {
        callers.sort_by(|left, right| left.caller.cmp(&right.caller));
        callers.dedup_by(|left, right| left.caller == right.caller);
    }
    graph.unresolved.sort_by(|left, right| {
        (&left.caller, left.line, &left.written, left.reason).cmp(&(
            &right.caller,
            right.line,
            &right.written,
            right.reason,
        ))
    });
    graph.unresolved.dedup_by(|left, right| {
        left.caller == right.caller
            && left.line == right.line
            && left.written == right.written
            && left.reason == right.reason
    });
    graph
}

#[implements("FR-099-AC-9")]
fn collect_calls(
    node: Node,
    source: &str,
    symbols: &[&Symbol],
    owner: Option<SymbolKey>,
    calls: &mut Vec<ParsedCall>,
) {
    let owner = if node.kind() == "function_item" {
        let line = node.start_position().row + 1;
        let column = node.start_position().column + 1;
        symbols
            .iter()
            .find(|symbol| {
                symbol.line == line
                    && symbol.column == column
                    && matches!(
                        symbol.kind,
                        SymbolKind::Function | SymbolKind::TestFunction | SymbolKind::Benchmark
                    )
            })
            .map(|symbol| SymbolKey::from_symbol(symbol))
            .or(owner)
    } else {
        owner
    };

    if node.kind() == "call_expression" {
        if let (Some(caller), Some(function)) =
            (owner.clone(), node.child_by_field_name("function"))
        {
            if let Some(written) = source.get(function.start_byte()..function.end_byte()) {
                let written = written.trim().to_string();
                let (callee_path, hint, unresolved_reason) = call_path(function, source);
                calls.push(ParsedCall {
                    caller,
                    written,
                    callee_path,
                    callee_hint: hint,
                    line: node.start_position().row + 1,
                    unresolved_reason,
                });
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_calls(child, source, symbols, owner.clone(), calls);
    }
}

#[implements("FR-099-AC-9")]
fn import_aliases(node: Node, source: &str) -> BTreeMap<String, String> {
    let mut aliases = BTreeMap::new();
    if node.kind() == "use_declaration" {
        if let Some(text) = source.get(node.start_byte()..node.end_byte()) {
            let declaration = text
                .trim()
                .strip_prefix("use ")
                .unwrap_or(text.trim())
                .trim_end_matches(';')
                .trim();
            if !declaration.contains('{') && !declaration.contains('*') {
                let (target, alias) = match declaration.split_once(" as ") {
                    Some((target, alias)) => (target.trim(), alias.trim()),
                    None => (
                        declaration,
                        declaration
                            .rsplit("::")
                            .next()
                            .unwrap_or(declaration)
                            .trim(),
                    ),
                };
                let target = normalize_call_path(target);
                let alias: String = alias
                    .chars()
                    .filter(|character| !character.is_whitespace())
                    .collect();
                if !target.is_empty() && !alias.is_empty() {
                    aliases.insert(alias, target);
                }
            }
        }
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        aliases.extend(import_aliases(child, source));
    }
    aliases
}

#[implements("FR-099-AC-9")]
fn call_path(node: Node, source: &str) -> (Option<String>, Option<String>, Option<&'static str>) {
    match node.kind() {
        "identifier" | "scoped_identifier" | "scoped_type_identifier" => {
            let Some(text) = source.get(node.start_byte()..node.end_byte()) else {
                return (None, None, Some("invalid-callee-span"));
            };
            let path = normalize_call_path(text);
            let hint = path.rsplit("::").next().map(str::to_string);
            (Some(path), hint, None)
        }
        "generic_function" => match node.child_by_field_name("function") {
            Some(function) => call_path(function, source),
            None => (None, None, Some("unreadable-generic-callee")),
        },
        "field_expression" => {
            let hint = node
                .child_by_field_name("field")
                .and_then(|field| source.get(field.start_byte()..field.end_byte()))
                .map(str::to_string);
            (None, hint, Some("method-call-needs-type-resolution"))
        }
        _ => (None, None, Some("dynamic-or-unsupported-callee")),
    }
}

#[implements("FR-099-AC-9")]
fn normalize_call_path(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut normalized = String::with_capacity(text.len());
    for (index, character) in chars.iter().enumerate() {
        if !character.is_whitespace() {
            normalized.push(*character);
            continue;
        }
        let previous = chars[..index]
            .iter()
            .rev()
            .find(|next| !next.is_whitespace());
        let next = chars[index + 1..].iter().find(|next| !next.is_whitespace());
        if previous.is_some_and(|value| *value == ':') || next.is_some_and(|value| *value == ':') {
            continue;
        }
        if previous.is_some_and(|value| value.is_alphanumeric() || *value == '_')
            && next.is_some_and(|value| value.is_alphanumeric() || *value == '_')
            && !normalized.ends_with(' ')
        {
            normalized.push(' ');
        }
    }
    normalized
}

#[implements("FR-099-AC-9")]
fn resolve_callee(
    callee_path: &str,
    caller: &SymbolKey,
    symbols: &[Symbol],
) -> Result<SymbolKey, &'static str> {
    let mut candidate_paths = Vec::new();
    let mut prefix = Some(caller.name.as_str());
    while let Some(scope) = prefix {
        candidate_paths.push(format!("{scope}::{callee_path}"));
        prefix = scope.rsplit_once("::").map(|(parent, _)| parent);
    }
    candidate_paths.push(callee_path.to_string());

    for candidate in candidate_paths {
        let matches: Vec<&Symbol> = symbols
            .iter()
            .filter(|symbol| {
                symbol.qualified_name == candidate
                    && matches!(
                        symbol.kind,
                        SymbolKind::Function | SymbolKind::TestFunction | SymbolKind::Benchmark
                    )
            })
            .collect();
        match matches.as_slice() {
            [symbol] => return Ok(SymbolKey::from_symbol(symbol)),
            [] => {}
            _ => return Err("ambiguous-symbol-path"),
        }
    }
    Err("no-exact-symbol-path")
}

#[implements("FR-099-AC-9")]
fn search_callers(
    changed: &Symbol,
    depth_limit: usize,
    extraction: &SymbolExtraction,
    graph: &crate::symbols::trace::SymbolGraph,
    calls: &CallerCallGraph,
) -> CallerSearch {
    let start = SymbolKey::from_symbol(changed);
    let mut frontier = vec![(start.clone(), vec![start.clone()])];
    let mut visited = BTreeSet::from([start.clone()]);
    let mut reached = Vec::new();
    let mut unresolved = BTreeSet::new();
    let mut truncated = false;

    if depth_limit == 0 {
        let target_name = start
            .name
            .rsplit("::")
            .next()
            .unwrap_or(start.name.as_str());
        for call in calls
            .unresolved
            .iter()
            .filter(|call| call.hint.as_deref() == Some(target_name))
        {
            unresolved.insert(UnresolvedCallSite {
                path: call.caller.path.clone(),
                caller: call.caller.name.clone(),
                line: call.line,
                callee: call.written.clone(),
                reason: call.reason.to_string(),
            });
        }
        truncated = calls
            .callers_by_callee
            .get(&start)
            .is_some_and(|callers| !callers.is_empty())
            || !unresolved.is_empty();
        return CallerSearch {
            changed_path: changed.path.clone(),
            changed_symbol: changed.qualified_name.clone(),
            depth_limit,
            linked_callers: reached,
            unresolved_calls: unresolved.into_iter().collect(),
            truncated,
        };
    }

    for depth in 1..=depth_limit {
        let mut next_frontier = Vec::new();
        let mut linked_at_depth = Vec::new();
        for (target, path) in frontier {
            let target_name = target
                .name
                .rsplit("::")
                .next()
                .unwrap_or(target.name.as_str());
            for call in calls
                .unresolved
                .iter()
                .filter(|call| call.hint.as_deref() == Some(target_name))
            {
                unresolved.insert(UnresolvedCallSite {
                    path: call.caller.path.clone(),
                    caller: call.caller.name.clone(),
                    line: call.line,
                    callee: call.written.clone(),
                    reason: call.reason.to_string(),
                });
            }
            for edge in calls.callers_by_callee.get(&target).into_iter().flatten() {
                if !visited.insert(edge.caller.clone()) {
                    continue;
                }
                let Some(caller_symbol) = extraction.symbols.iter().find(|symbol| {
                    symbol.path == edge.caller.path && symbol.qualified_name == edge.caller.name
                }) else {
                    continue;
                };
                let mut linked_ids: BTreeSet<String> = graph
                    .implements
                    .iter()
                    .filter(|relation| relation.symbol_id == caller_symbol.id)
                    .map(|relation| relation.trace_id.clone())
                    .collect();
                linked_ids.extend(
                    graph
                        .verifies
                        .iter()
                        .filter(|relation| relation.symbol_id == caller_symbol.id)
                        .map(|relation| relation.trace_id.clone()),
                );
                if !linked_ids.is_empty() {
                    let mut call_path: Vec<String> = path.iter().map(SymbolKey::display).collect();
                    call_path.push(edge.caller.display());
                    call_path.reverse();
                    linked_at_depth.push(LinkedCaller {
                        path: caller_symbol.path.clone(),
                        symbol: caller_symbol.qualified_name.clone(),
                        depth,
                        linked_ids: linked_ids.into_iter().collect(),
                        call_path,
                    });
                } else {
                    let mut next_path = path.clone();
                    next_path.push(edge.caller.clone());
                    next_frontier.push((edge.caller.clone(), next_path));
                }
            }
        }
        if !linked_at_depth.is_empty() {
            reached = linked_at_depth;
            break;
        }
        if depth == depth_limit {
            truncated = next_frontier.iter().any(|(symbol, _)| {
                calls
                    .callers_by_callee
                    .get(symbol)
                    .is_some_and(|callers| !callers.is_empty())
            });
            break;
        }
        frontier = next_frontier;
        if frontier.is_empty() {
            break;
        }
    }

    reached.sort();
    CallerSearch {
        changed_path: changed.path.clone(),
        changed_symbol: changed.qualified_name.clone(),
        depth_limit,
        linked_callers: reached,
        unresolved_calls: unresolved.into_iter().collect(),
        truncated,
    }
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
            caller_search_depth: None,
        })
    }

    fn run_with_callers(source: &str, changed_lines: LineRange, depth: usize) -> ImpactReport {
        let root = tempfile::tempdir().unwrap();
        let registry = model_with_implements_form(root.path());
        let model = registry.traceability().unwrap();
        let base = snapshot("base", "");
        let head = snapshot("head", source);
        let available = BTreeSet::from(["FR-099-AC-9".to_string()]);
        analyze(ImpactInput {
            base: Some(&base),
            head: Some(&head),
            changes: &[FileChange {
                path: "src/lib.rs".into(),
                base: None,
                head: Some(changed_lines),
            }],
            traceability: model,
            available_spec_ids: &available,
            unavailable_spec_ids: &BTreeSet::new(),
            caller_search_depth: Some(depth),
        })
        .unwrap()
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
            caller_search_depth: None,
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
            caller_search_depth: None,
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
            caller_search_depth: None,
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
                caller_search_depth: None,
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

    #[trace("FR-099-AC-9")]
    #[test]
    fn direct_linked_caller_selects_its_verifying_test_with_path() {
        let report = run_with_callers(
            "fn changed() {}\n#[implements(\"FR-099-AC-9\")]\nfn caller() { changed(); }\n#[test]\n#[trace(\"FR-099-AC-9\")]\nfn caller_test() { caller(); }\n",
            LineRange { start: 1, end: 1 },
            3,
        );
        let search = report.caller_searches.first().unwrap();
        assert_eq!(search.linked_callers.len(), 1);
        assert_eq!(search.linked_callers[0].symbol, "caller");
        assert_eq!(
            search.linked_callers[0].call_path,
            ["src/lib.rs::caller", "src/lib.rs::changed"]
        );
        let selected = report
            .selected_tests
            .iter()
            .find(|test| test.name == "caller_test")
            .unwrap();
        assert!(selected.reasons.iter().any(|reason| matches!(
            reason,
            ImpactReason::CallerFollowing { call_path, trace_id, .. }
                if call_path == &["src/lib.rs::caller", "src/lib.rs::changed"]
                    && trace_id == "FR-099-AC-9"
        )));
    }

    #[trace("FR-099-AC-9")]
    #[test]
    fn multi_step_search_returns_nearest_linked_caller() {
        let report = run_with_callers(
            "fn changed() {}\nfn middle() { changed(); }\n#[implements(\"FR-099-AC-9\")]\nfn linked() { middle(); }\n#[test]\n#[trace(\"FR-099-AC-9\")]\nfn linked_test() { linked(); }\n",
            LineRange { start: 1, end: 1 },
            3,
        );
        let search = report.caller_searches.first().unwrap();
        assert_eq!(search.linked_callers.len(), 1);
        assert_eq!(search.linked_callers[0].symbol, "linked");
        assert_eq!(search.linked_callers[0].depth, 2);
        assert_eq!(
            search.linked_callers[0].call_path,
            [
                "src/lib.rs::linked",
                "src/lib.rs::middle",
                "src/lib.rs::changed"
            ]
        );
        assert!(report
            .selected_tests
            .iter()
            .any(|test| test.name == "linked_test"));
    }

    #[trace("FR-099-AC-9")]
    #[test]
    fn depth_limit_reports_truncation_without_selecting_farther_callers() {
        let report = run_with_callers(
            "fn changed() {}\nfn middle() { changed(); }\n#[implements(\"FR-099-AC-9\")]\nfn linked() { middle(); }\n#[test]\n#[trace(\"FR-099-AC-9\")]\nfn linked_test() { linked(); }\n",
            LineRange { start: 1, end: 1 },
            1,
        );
        let search = report.caller_searches.first().unwrap();
        assert!(search.linked_callers.is_empty());
        assert!(search.truncated);
        assert!(report.findings.iter().any(|finding| matches!(
            finding,
            ImpactFinding::CallerSearchIncomplete { symbol, .. } if symbol == "changed"
        )));
        assert!(!report
            .selected_tests
            .iter()
            .any(|test| test.name == "linked_test"));
    }

    #[trace("FR-099-AC-9")]
    #[test]
    fn caller_cycle_is_visited_once_without_false_truncation() {
        let report = run_with_callers(
            "fn changed() { helper(); }\nfn helper() { changed(); }\n",
            LineRange { start: 1, end: 1 },
            4,
        );
        let search = report.caller_searches.first().unwrap();
        assert!(search.linked_callers.is_empty());
        assert!(!search.truncated);
    }

    #[trace("FR-099-AC-9")]
    #[test]
    fn trait_qualified_caller_path_matches_one_extracted_symbol() {
        let report = run_with_callers(
            "trait Operations { fn changed() {} }\n#[implements(\"FR-099-AC-9\")]\nfn caller() { Operations::changed(); }\n#[test]\n#[trace(\"FR-099-AC-9\")]\nfn caller_test() { caller(); }\n",
            LineRange { start: 1, end: 1 },
            2,
        );
        let search = report.caller_searches.first().unwrap();
        assert_eq!(search.linked_callers.len(), 1);
        assert_eq!(search.linked_callers[0].symbol, "caller");
        assert!(report
            .selected_tests
            .iter()
            .any(|test| test.name == "caller_test"));
    }

    #[trace("FR-099-AC-9")]
    #[test]
    fn dynamic_method_call_is_unresolved_and_keeps_search_incomplete() {
        let report = run_with_callers(
            "fn changed() {}\n#[implements(\"FR-099-AC-9\")]\nfn caller(receiver: impl std::fmt::Debug) { receiver.changed(); }\n",
            LineRange { start: 1, end: 1 },
            2,
        );
        let search = report.caller_searches.first().unwrap();
        assert_eq!(search.unresolved_calls.len(), 1);
        assert_eq!(
            search.unresolved_calls[0].reason,
            "method-call-needs-type-resolution"
        );
        assert!(!report.selection_complete);
    }

    #[trace("FR-099-AC-9")]
    #[test]
    fn imported_alias_call_is_reported_unresolved_for_original_target() {
        let report = run_with_callers(
            "mod remote { pub fn changed() {} }\nuse remote::changed as renamed;\n#[implements(\"FR-099-AC-9\")]\nfn caller() { renamed(); }\n",
            LineRange { start: 1, end: 1 },
            2,
        );
        let search = report
            .caller_searches
            .iter()
            .find(|search| search.changed_symbol == "remote::changed")
            .unwrap();
        assert!(search.unresolved_calls.iter().any(|call| {
            call.caller == "caller"
                && call.callee == "renamed"
                && call.reason == "import-alias-call"
        }));
        assert!(!report.selection_complete);
    }

    #[trace("FR-099-AC-9")]
    #[test]
    fn duplicate_extracted_paths_make_the_caller_ambiguous() {
        let root = tempfile::tempdir().unwrap();
        let registry = model_with_implements_form(root.path());
        let model = registry.traceability().unwrap();
        let base = SourceSnapshot {
            revision: "base".into(),
            files: BTreeMap::from([
                ("src/a.rs".into(), "fn changed() {}\n".into()),
                ("src/b.rs".into(), "fn changed() {}\n".into()),
                (
                    "src/lib.rs".into(),
                    "#[implements(\"FR-099-AC-9\")]\nfn caller() { changed(); }\n".into(),
                ),
            ]),
        };
        let head = SourceSnapshot {
            revision: "head".into(),
            files: BTreeMap::from([
                ("src/a.rs".into(), "fn changed() { let _ = 1; }\n".into()),
                ("src/b.rs".into(), "fn changed() {}\n".into()),
                (
                    "src/lib.rs".into(),
                    "#[implements(\"FR-099-AC-9\")]\nfn caller() { changed(); }\n".into(),
                ),
            ]),
        };
        let report = analyze(ImpactInput {
            base: Some(&base),
            head: Some(&head),
            changes: &[FileChange {
                path: "src/a.rs".into(),
                base: Some(LineRange { start: 1, end: 1 }),
                head: Some(LineRange { start: 1, end: 1 }),
            }],
            traceability: model,
            available_spec_ids: &BTreeSet::from(["FR-099-AC-9".to_string()]),
            unavailable_spec_ids: &BTreeSet::new(),
            caller_search_depth: Some(2),
        })
        .unwrap();
        let search = report.caller_searches.first().unwrap();
        assert!(search
            .unresolved_calls
            .iter()
            .any(|call| { call.caller == "caller" && call.reason == "ambiguous-symbol-path" }));
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
            caller_search_depth: None,
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
            caller_search_depth: None,
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
            caller_search_depth: None,
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
            caller_search_depth: None,
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
            caller_search_depth: None,
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
