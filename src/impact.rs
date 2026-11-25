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
    ExtractionUnavailable { path: String, reason: String },
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
                    let matches = trace_search::search(graph, trace_search::Query::Id(id.clone()));
                    for test in matches.verifies {
                        selected
                            .entry((test.path, test.symbol))
                            .or_default()
                            .insert(ImpactReason::ImplementedId {
                                changed_path: change.path.clone(),
                                changed_lines: range,
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
                            implementation: symbol.qualified_name.clone(),
                            trace_id: id.clone(),
                        };
                        related.insert((other.path, other.symbol, id.clone(), reason));
                    }
                }
            }
            for diagnostic in extraction
                .diagnostics
                .iter()
                .filter(|item| item.path == change.path)
            {
                report.findings.push(ImpactFinding::ExtractionUnavailable {
                    path: diagnostic.path.clone(),
                    reason: diagnostic.reason.clone(),
                });
            }
            if !extraction.unavailable_languages.is_empty() {
                report.findings.push(ImpactFinding::ExtractionUnavailable {
                    path: change.path.clone(),
                    reason: format!(
                        "symbol extraction unavailable for {:?}",
                        extraction.unavailable_languages
                    ),
                });
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
    report.selection_complete = report.findings.is_empty();
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

    #[trace("FR-099-AC-4")]
    #[trace("FR-099-AC-5")]
    #[trace("FR-099-AC-6")]
    #[test]
    fn exact_implemented_id_selects_tests_and_lists_twin_implementers() {
        let root = tempfile::tempdir().unwrap();
        let registry = model_with_implements_form(root.path());
        let model = registry.traceability().unwrap();
        let source = "#[implements(\"FR-099-AC-4\")]\nfn changed() {}\n#[implements(\"FR-099-AC-4\")]\nfn twin() {}\n#[test]\n#[trace(\"FR-099-AC-4\")]\nfn matching_test() {}\n#[test]\n#[trace(\"FR-099-AC-3\")]\nfn different_test() {}\n";
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
            .any(|item| item.name == "twin" && item.trace_id == "FR-099-AC-4"));
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
}
