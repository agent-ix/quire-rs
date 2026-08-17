//! Scanning the tables a traceability model declares (FR-049 / FR-050).
//!
//! Both the reference-integrity check and the coverage rollup read the same
//! shape: "the table under section S of every document of archetype A". This is
//! that one scan, so the two consumers cannot drift apart on which rows they
//! see.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::spec::Spec;
use crate::ast::QuireDocument;
use crate::query::{parse_table, section};

/// One scanned table row: the document it lives in and its cells keyed by the
/// table's own column headers (trimmed, as authored).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScannedRow {
    pub path: PathBuf,
    pub cells: BTreeMap<String, String>,
}

impl ScannedRow {
    /// The value of `column`, case-insensitively, when non-empty.
    pub fn cell(&self, column: &str) -> Option<&str> {
        self.cells
            .iter()
            .find(|(header, _)| header.eq_ignore_ascii_case(column))
            .map(|(_, value)| value.as_str())
            .filter(|v| !v.is_empty())
    }
}

/// What one declaration says about where its rows live: every bundle document
/// of an archetype, minus anything `exclude` matches (CR-038, CR-060).
///
/// Passed as a struct rather than positional arguments because the two
/// consumers construct it from two different declaration types.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DeclaredScope<'a> {
    /// The declaration's own name, so a diagnostic can say which one of them
    /// scanned nothing (CR-054).
    pub name: &'a str,
    pub archetype: &'a str,
    /// This declaration's own `exclude:` — "these documents mint no ids for
    /// *me*" (CR-038).
    pub exclude: &'a ExcludeSet,
    /// The model-level `exclude:` — "these paths hold no traceable data"
    /// (CR-060). Applied in addition to the declaration's own, never instead.
    pub model_exclude: &'a ExcludeSet,
}

/// Compiled exclusion globs.
///
/// Compiled **once** rather than per call: `excludes` is asked about every
/// document of a declared archetype, and since CR-060 about every document in
/// the corpus for the criteria walk, so compiling each pattern per question
/// would put a glob build on the NFR-015 walk.
///
/// An empty set matches nothing, which is what an undeclared `exclude:` means.
#[derive(Debug, Default)]
pub(crate) struct ExcludeSet {
    set: Option<globset::GlobSet>,
}

impl ExcludeSet {
    /// Compile `patterns`. Patterns are validated at module load, so one that
    /// does not compile here would mean the model was never validated: it is
    /// dropped rather than allowed to swallow every document.
    pub(crate) fn compile(patterns: &[String]) -> Self {
        if patterns.is_empty() {
            return Self::default();
        }
        let mut builder = globset::GlobSetBuilder::new();
        for pattern in patterns {
            if let Ok(glob) = globset::Glob::new(pattern) {
                builder.add(glob);
            }
        }
        Self {
            set: builder.build().ok(),
        }
    }

    /// True when `relative` — a scope-relative, `/`-separated path — is
    /// excluded.
    fn matches(&self, relative: &str) -> bool {
        self.set.as_ref().is_some_and(|set| set.is_match(relative))
    }

    /// True when `path` is excluded, deriving the scope-relative form only
    /// when there is something to match it against.
    pub(crate) fn excludes(&self, root: &Path, path: &Path) -> bool {
        !self.is_empty() && self.matches(&relative_path(root, path))
    }

    fn is_empty(&self) -> bool {
        self.set.is_none()
    }
}

/// What a scan noticed about the **declaration**, as opposed to its rows
/// (CR-054). CR-049 made body selection load-bearing on the declaration, so a
/// declaration that selects nothing no longer merely reports nothing — it
/// stops the engine from parsing anything. Each of these is a scan that
/// produced no rows for a reason the operator can act on.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ScanDiagnostic {
    /// The declared archetype names no document in the corpus.
    ///
    /// Reported **only when the model minted nothing at all** — see
    /// [`ScanContext::into_diagnostics`]. On its own this is ordinary: a model
    /// legitimately declares archetypes an individual repo has no instance of
    /// (the ISO model's "a TC authored as its own document" is one), and
    /// reporting each would be noise on every healthy repo. It is the cause
    /// worth naming only when nothing minted, which is the shape a typo in the
    /// one archetype that mattered produces.
    ///
    /// Counted before `exclude` applies, so excluding every match is not
    /// reported as a missing archetype.
    ArchetypeMatchesNothing { archetype: String },
}

/// The diagnostics above, shared across every declaration in one
/// reconciliation.
///
/// Held a per-path harvest cache until CR-062: `harvest` re-read and re-parsed
/// the document a model named once per declaration. With binding by archetype
/// only, every minting document is a corpus document whose body is already
/// behind the FR-025 per-document once-cell, so the cache had nothing left to
/// cache.
#[derive(Debug, Default)]
pub(crate) struct ScanContext {
    diagnostics: Vec<(String, ScanDiagnostic)>,
}

impl ScanContext {
    fn note(&mut self, declaration: &str, diagnostic: ScanDiagnostic) {
        let entry = (declaration.to_string(), diagnostic);
        if !self.diagnostics.contains(&entry) {
            self.diagnostics.push(entry);
        }
    }

    /// The declaration's diagnostics, sorted so the order is a property of the
    /// model rather than of the walk (NFR-006).
    ///
    /// `minted_anything` says whether the model produced any id at all. A
    /// declared archetype no document has is reported only when it did not: a
    /// model that mints normally while one optional declaration selects nothing
    /// is healthy, and saying so on every repo would be noise nobody reads.
    ///
    /// Since CR-062 that is the only diagnostic this scan produces. A minting
    /// document that cannot be *read* is now reported by the walk, as
    /// `DocumentUnreadable` / `MissingUuid` — an improvement over the
    /// `document:` form, whose reader returned `None` and said nothing.
    pub(crate) fn into_diagnostics(self, minted_anything: bool) -> Vec<(String, ScanDiagnostic)> {
        let mut out = self.diagnostics;
        if minted_anything {
            out.retain(|(_, d)| !matches!(d, ScanDiagnostic::ArchetypeMatchesNothing { .. }));
        }
        out.sort();
        out
    }
}

impl DeclaredScope<'_> {
    /// True when `path` is excluded from this declaration, by its own
    /// `exclude:` or by the model-level one (CR-060).
    ///
    /// Matching is on the **scope-relative** path, which is the only form a
    /// module can author against: an absolute path is a property of the
    /// machine, not of the repository.
    pub(crate) fn excludes(&self, root: &Path, path: &Path) -> bool {
        if self.exclude.is_empty() && self.model_exclude.is_empty() {
            return false;
        }
        let relative = relative_path(root, path);
        self.exclude.matches(&relative) || self.model_exclude.matches(&relative)
    }
}

/// Scan the table under `heading` for every corpus document of `scope`'s
/// archetype, in corpus order.
pub(crate) fn scan(
    spec: &Spec,
    root: &Path,
    scope: DeclaredScope<'_>,
    heading: &str,
    ctx: &mut ScanContext,
) -> Vec<ScannedRow> {
    let mut out = Vec::new();
    let mut of_archetype = 0usize;
    for doc in &spec.inner.documents {
        // Header-tier gate first (CR-047): only an archetype-matching,
        // non-excluded document pays for a body parse.
        if doc.concept_type() != Some(scope.archetype) {
            continue;
        }
        of_archetype += 1;
        if scope.excludes(root, &doc.path) {
            continue;
        }
        out.extend(rows_of(doc.body(), &doc.path, heading));
    }
    // Counted before `exclude`: a declaration that deliberately excludes
    // all of its matches is not a missing archetype (CR-054).
    if of_archetype == 0 {
        ctx.note(
            scope.name,
            ScanDiagnostic::ArchetypeMatchesNothing {
                archetype: scope.archetype.to_string(),
            },
        );
    }
    out
}

/// A path relative to the scope root, `/`-separated so declarations and reports
/// are stable across platforms. Paths outside the root are used as-is.
pub(crate) fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("/")
}

/// The rows of the table under `heading` in one document.
pub(crate) fn rows_of(doc: &QuireDocument, path: &Path, heading: &str) -> Vec<ScannedRow> {
    let Some(sec) = section(doc, heading) else {
        return Vec::new();
    };
    let Some(table) = parse_table(&sec.content) else {
        return Vec::new();
    };
    table
        .rows
        .iter()
        .map(|row| {
            let mut cells = BTreeMap::new();
            for (idx, header) in table.headers.iter().enumerate() {
                cells.insert(
                    header.trim().to_string(),
                    row.get(idx).map(|v| v.trim()).unwrap_or("").to_string(),
                );
            }
            ScannedRow {
                path: path.to_path_buf(),
                cells,
            }
        })
        .collect()
}

/// Normalize a reference cell before ids are extracted, per the declaration's
/// opt-in flags (CR-015). Both consumers — FR-049's dangling check and FR-050's
/// rollup — call this, so they cannot disagree about what a cell contains.
///
/// `strip_annotations` removes parenthetical spans, so a qualifier never
/// contributes a reference. `expand_ranges` rewrites `FR-001..FR-003` into
/// `FR-001, FR-002, FR-003`; a range whose ends disagree on prefix, or whose
/// bounds are inverted, is left untouched for the pattern to reject.
pub(crate) fn normalize_reference_cell(
    cell: &str,
    strip_annotations: bool,
    expand_ranges: bool,
) -> String {
    let mut out = cell.to_string();
    if strip_annotations {
        out = re_parenthetical().replace_all(&out, " ").to_string();
    }
    if expand_ranges {
        out = re_range()
            .replace_all(&out, |caps: &regex::Captures<'_>| expand(caps))
            .to_string();
    }
    out
}

/// Expand one matched range, or return it unchanged when it is not a coherent
/// same-prefix ascending range.
fn expand(caps: &regex::Captures<'_>) -> String {
    let whole = caps.get(0).map_or("", |m| m.as_str()).to_string();
    let (prefix, from, to) = (&caps[1], &caps[2], &caps[4]);
    if prefix != &caps[3] {
        return whole; // `FR-001..NFR-003` is not a range
    }
    let width = from.len();
    let (Ok(start), Ok(end)) = (from.parse::<u32>(), to.parse::<u32>()) else {
        return whole;
    };
    if end < start || end - start > MAX_RANGE_SPAN {
        return whole;
    }
    (start..=end)
        .map(|n| format!("{prefix}-{n:0width$}"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// A range wider than this is treated as authoring noise rather than expanded —
/// it would swamp the report with ids nobody enumerated deliberately.
const MAX_RANGE_SPAN: u32 = 200;

fn re_parenthetical() -> &'static regex::Regex {
    static R: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    R.get_or_init(|| regex::Regex::new(r"\([^)]*\)").expect("parenthetical regex"))
}

fn re_range() -> &'static regex::Regex {
    static R: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    R.get_or_init(|| {
        regex::Regex::new(r"([A-Za-z]+)-(\d+)\s*\.\.\s*([A-Za-z]+)-(\d+)").expect("range regex")
    })
}

/// The stable machine reason for a scan diagnostic — one vocabulary, so
/// `quire validate` and `quire coverage` cannot disagree about what to call
/// the same finding (CR-054).
pub(crate) fn scan_reason(diagnostic: &ScanDiagnostic) -> &'static str {
    match diagnostic {
        ScanDiagnostic::ArchetypeMatchesNothing { .. } => "archetype-matches-nothing",
    }
}

/// The path a scan diagnostic is about and its human message. The path is the
/// scope root: a declaration-level fault has no single document to point at.
pub(crate) fn scan_finding(
    declaration: &str,
    diagnostic: &ScanDiagnostic,
    root: &Path,
) -> (PathBuf, String) {
    match diagnostic {
        ScanDiagnostic::ArchetypeMatchesNothing { archetype } => (
            root.to_path_buf(),
            format!(
                "declaration '{declaration}' declares archetype '{archetype}', which no \
                 document in the corpus has — it scans nothing and mints no rows"
            ),
        ),
    }
}
