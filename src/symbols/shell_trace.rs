//! Trace binding for shell symbols (FR-051-AC-38..44, PLAT-362).
//!
//! The shell adapter ([`super::shell`]) mints an `entry_point` for every `.sh`
//! file and a `function` for every definition, and attaches to each the `#`
//! comment nodes of its tag channel. This module is the binding stage's half:
//!
//! - an `entry_point` binds `verifies` ids read from its header comments, and
//!   only where its repo-relative path matches an `entry_paths` glob of a
//!   loaded `language: shell` form in `trace_tags.markers` (AC-38, AC-39);
//! - a function is production: it binds no `verifies`, and carries
//!   `implements` only through a `language: shell` form in
//!   `trace_tags.implements`, read from its comment block;
//! - a declared marker that matches in a comment and binds nothing is reported
//!   with the reason it did not bind (AC-44).
//!
//! Shell forms read comment nodes whatever `syntax` they declare: shell has no
//! other construct a marker can be (FR-051, Trace-tag grammar).

use std::collections::BTreeSet;

use regex::Regex;

use super::trace::{
    marker_ids, range_pattern, ImplementsRelation, RangeInTraceTag, SymbolGraph, TraceDiagnostic,
    TraceProvenance, VerifiesRelation,
};
use super::{SourceAttribute, Symbol, SymbolExtraction, SymbolKind};
use crate::corpus::declared_tables::ExcludeSet;
use crate::traceability::{SourceLanguage, TraceMarkerForm, TraceabilityModel};

/// Why a declared shell marker that matched a comment bound nothing (AC-44).
pub const OUTSIDE_ENTRY_PATHS: &str = "outside-entry-paths";
/// The marker sits in a shell function's comment block, and a function is not
/// evidence (AC-38).
pub const FUNCTION_NOT_EVIDENCE: &str = "function-not-evidence";
/// The marker sits in a comment that is neither a function's block nor the
/// file header.
pub const NOT_IN_TAG_CHANNEL: &str = "not-in-tag-channel";
/// A shell marker the binder saw and could not bind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellFinding {
    /// One of the reason tokens above.
    pub reason: &'static str,
    /// Repo-relative path.
    pub path: String,
    /// 1-based line of the comment.
    pub line: usize,
    /// The trace id.
    pub value: String,
}

struct Form<'m> {
    form: &'m TraceMarkerForm,
    pattern: Regex,
}

/// The shell half of the module's trace-tag grammar, compiled once per `bind`.
pub(crate) struct ShellBinder<'m> {
    markers: Vec<Form<'m>>,
    implements: Vec<Form<'m>>,
    entry_paths: ExcludeSet,
}

impl<'m> ShellBinder<'m> {
    pub(crate) fn new(model: &'m TraceabilityModel, graph: &mut SymbolGraph) -> Self {
        let compile = |forms: &'m [TraceMarkerForm]| -> Vec<Form<'m>> {
            forms
                .iter()
                .filter(|form| form.language == SourceLanguage::Shell)
                // An uncompilable pattern is refused at module load; a
                // hand-built model that skipped validation reads nothing here,
                // as it reads nothing in the other languages.
                .filter_map(|form| {
                    Some(Form {
                        form,
                        pattern: Regex::new(&form.pattern).ok()?,
                    })
                })
                .collect()
        };
        let markers = compile(&model.trace_tags.markers);
        let implements = compile(&model.trace_tags.implements);
        // One union across every loaded shell form (AC-39). A list that does
        // not compile is refused as one unit, never as a partial subset, and
        // the finding names the pattern (FR-050-AC-25).
        let globs: Vec<String> = markers
            .iter()
            .filter_map(|f| f.form.entry_paths.as_deref())
            .flatten()
            .cloned()
            .collect();
        let entry_paths = match ExcludeSet::compile(&globs) {
            Ok(set) => set,
            Err(reason) => {
                let pattern = globs
                    .iter()
                    .find(|p| globset::Glob::new(p).is_err())
                    .cloned()
                    .unwrap_or_default();
                graph.diagnostics.push(TraceDiagnostic {
                    path: pattern,
                    symbol: String::new(),
                    trace_id: String::new(),
                    reason: format!("entry_paths not applied: {reason}"),
                });
                ExcludeSet::default()
            }
        };
        Self {
            markers,
            implements,
            entry_paths,
        }
    }

    fn eligible(&self, path: &str) -> bool {
        !self.entry_paths.is_empty() && self.entry_paths.matches(path)
    }

    /// Bind one shell symbol. Called for every shell symbol; kinds that carry
    /// neither relation are left alone.
    pub(crate) fn bind_symbol(&self, symbol: &Symbol, graph: &mut SymbolGraph) {
        match symbol.kind {
            SymbolKind::EntryPoint if self.eligible(&symbol.path) => {
                self.bind_entry_point(symbol, graph);
            }
            SymbolKind::Function => self.bind_function(symbol, graph),
            _ => {}
        }
    }

    fn bind_entry_point(&self, symbol: &Symbol, graph: &mut SymbolGraph) {
        let mut seen: BTreeSet<String> = BTreeSet::new();
        for Form { form, pattern } in &self.markers {
            for comment in &symbol.tag_comments {
                for (trace_id, range) in read_ids(pattern, comment, symbol) {
                    if let Some(range) = range {
                        graph.range_diagnostics.push(range);
                        continue;
                    }
                    if seen.insert(trace_id.clone()) {
                        graph.verifies.push(VerifiesRelation {
                            symbol_id: symbol.id.clone(),
                            symbol: symbol.qualified_name.clone(),
                            path: symbol.path.clone(),
                            trace_id,
                            provenance: TraceProvenance::Canonical,
                            form: form.name.clone(),
                            line: symbol.line,
                            column: symbol.column,
                            kind: symbol.kind,
                            ignored: symbol.ignored,
                        });
                    }
                }
            }
        }
    }

    fn bind_function(&self, symbol: &Symbol, graph: &mut SymbolGraph) {
        let before = graph.implements.len();
        for Form { form, pattern } in &self.implements {
            for comment in &symbol.tag_comments {
                for (trace_id, range) in read_ids(pattern, comment, symbol) {
                    if let Some(range) = range {
                        graph.range_diagnostics.push(range);
                        continue;
                    }
                    graph.implements.push(ImplementsRelation {
                        symbol_id: symbol.id.clone(),
                        symbol: symbol.qualified_name.clone(),
                        path: symbol.path.clone(),
                        trace_id,
                        form: form.name.clone(),
                    });
                }
            }
        }
        graph.implements_candidates += 1;
        if graph.implements.len() > before {
            graph.implements_bound += 1;
        }
    }

    /// AC-44: every declared `markers` match in a shell comment that bound
    /// nothing, with the reason. Run after every symbol is bound.
    pub(crate) fn report_unbound(&self, extraction: &SymbolExtraction, graph: &mut SymbolGraph) {
        if self.markers.is_empty() {
            return;
        }
        for file in &extraction.files {
            if file.language != SourceLanguage::Shell {
                continue;
            }
            let in_channel = |kind: SymbolKind| -> BTreeSet<usize> {
                extraction
                    .symbols
                    .iter()
                    .filter(|s| s.path == file.path && s.kind == kind)
                    .flat_map(|s| s.tag_comments.iter().map(|c| c.line))
                    .collect()
            };
            let header = in_channel(SymbolKind::EntryPoint);
            let blocks = in_channel(SymbolKind::Function);
            let eligible = self.eligible(&file.path);
            for comment in &file.comments {
                let reason = if header.contains(&comment.line) {
                    if eligible {
                        continue;
                    }
                    OUTSIDE_ENTRY_PATHS
                } else if blocks.contains(&comment.line) {
                    FUNCTION_NOT_EVIDENCE
                } else {
                    NOT_IN_TAG_CHANNEL
                };
                for Form { pattern, .. } in &self.markers {
                    for caps in pattern.captures_iter(&comment.text) {
                        let Some(args) = caps.get(1) else { continue };
                        for trace_id in marker_ids(args.as_str()) {
                            if range_pattern().is_match(&trace_id) {
                                continue;
                            }
                            graph.shell_findings.push(ShellFinding {
                                reason,
                                path: file.path.clone(),
                                line: comment.line,
                                value: trace_id,
                            });
                        }
                    }
                }
            }
        }
        graph.shell_findings.sort_by(|a, b| {
            (&a.path, a.line, &a.value, a.reason).cmp(&(&b.path, b.line, &b.value, b.reason))
        });
        graph.shell_findings.dedup();
    }
}

/// The ids one comment yields under `pattern`, each paired with the range it
/// spells when it is one (a range binds nothing, FR-051-AC-28).
fn read_ids(
    pattern: &Regex,
    comment: &SourceAttribute,
    symbol: &Symbol,
) -> Vec<(String, Option<RangeInTraceTag>)> {
    let mut out = Vec::new();
    for caps in pattern.captures_iter(&comment.text) {
        let Some(args) = caps.get(1) else { continue };
        for trace_id in marker_ids(args.as_str()) {
            let range = range_pattern()
                .is_match(&trace_id)
                .then(|| RangeInTraceTag {
                    path: symbol.path.clone(),
                    symbol: symbol.qualified_name.clone(),
                    line: comment.line,
                    range_text: trace_id.clone(),
                });
            out.push((trace_id, range));
        }
    }
    out
}
