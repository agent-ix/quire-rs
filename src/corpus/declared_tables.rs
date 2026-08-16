//! Scanning the tables a traceability model declares (FR-049 / FR-050).
//!
//! Both the reference-integrity check and the coverage rollup read the same
//! shape: "the table under section S of every document of archetype A, or of a
//! declared auxiliary document". This is that one scan, so the two consumers
//! cannot drift apart on which rows they see.

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

/// What one declaration says about where its rows live: an archetype (every
/// bundle document of that type), an auxiliary `document` under `root`
/// harvested off-corpus, or both — minus anything `exclude` matches (CR-038).
///
/// Passed as a struct rather than four positional arguments because the two
/// consumers construct it from two different declaration types, and a
/// same-typed pair of `Option`s is exactly the call site that gets transposed.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DeclaredScope<'a> {
    /// The declaration's own name, so a diagnostic can say which one of them
    /// scanned nothing (CR-054).
    pub name: &'a str,
    pub archetype: Option<&'a str>,
    pub document: Option<&'a Path>,
    pub exclude: &'a [String],
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
    /// A declared auxiliary document could not be read. The CR-045 class:
    /// the read failed, every id it would have minted vanished, and the only
    /// symptom was a distant dangling-reference count.
    UnreadableDocument { path: PathBuf, error: String },
}

/// Rows plus a per-path harvest cache and the diagnostics above, shared across
/// every declaration in one reconciliation.
///
/// The cache is the reason this is a context rather than four arguments:
/// `harvest` re-read and re-parsed its document once per declaration, and the
/// document a model names is typically `spec/tests.md` — the largest file in
/// the repo — read once per trace target *and* once per document reference.
/// CR-049's "the model bounds what is parsed" was inverted for exactly the
/// documents the model names explicitly (CR-054).
#[derive(Debug, Default)]
pub(crate) struct ScanContext {
    harvested: BTreeMap<PathBuf, Result<QuireDocument, String>>,
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
    /// declared archetype no document has is reported only when it did not:
    /// a model that mints normally while one optional declaration matches
    /// nothing is healthy, and saying so on every repo would be noise nobody
    /// reads. An unreadable declared document is reported either way — the
    /// model named that file, and the file did not open.
    pub(crate) fn into_diagnostics(self, minted_anything: bool) -> Vec<(String, ScanDiagnostic)> {
        let mut out = self.diagnostics;
        if minted_anything {
            out.retain(|(_, d)| !matches!(d, ScanDiagnostic::ArchetypeMatchesNothing { .. }));
        }
        out.sort();
        out
    }

    /// Read + parse `document` under `root`, **once per path** however many
    /// declarations name it.
    ///
    /// The failure is cached too, and re-reported against each declaration
    /// that asked: the read happening once is a performance property, while
    /// which declarations are affected is what the operator acts on, and
    /// collapsing the second into the first would answer the wrong question.
    fn harvest(
        &mut self,
        declaration: &str,
        root: &Path,
        document: &Path,
    ) -> Option<&QuireDocument> {
        let path = root.join(document);
        if !self.harvested.contains_key(&path) {
            let read = std::fs::read_to_string(&path)
                .map(|text| crate::parse_document(&text))
                .map_err(|error| error.to_string());
            self.harvested.insert(path.clone(), read);
        }
        match self.harvested.get(&path) {
            Some(Ok(_)) => self.harvested.get(&path).and_then(|r| r.as_ref().ok()),
            Some(Err(error)) => {
                let error = error.clone();
                self.note(
                    declaration,
                    ScanDiagnostic::UnreadableDocument { path, error },
                );
                None
            }
            None => None,
        }
    }
}

impl DeclaredScope<'_> {
    /// True when `path` is excluded from this declaration.
    ///
    /// Matching is on the **scope-relative** path, which is the only form a
    /// module can author against: an absolute path is a property of the
    /// machine, not of the repository.
    pub(crate) fn excludes(&self, root: &Path, path: &Path) -> bool {
        if self.exclude.is_empty() {
            return false;
        }
        let relative = relative_path(root, path);
        self.exclude.iter().any(|pattern| {
            // Patterns are validated at module load; an uncompilable one here
            // would mean the model was never validated, so it excludes nothing
            // rather than silently swallowing every document.
            globset::Glob::new(pattern)
                .map(|glob| glob.compile_matcher().is_match(&relative))
                .unwrap_or(false)
        })
    }
}

/// Scan the table under `heading` for every document `scope` covers. Bundle
/// documents come first, in corpus order; the auxiliary rows follow.
pub(crate) fn scan(
    spec: &Spec,
    root: &Path,
    scope: DeclaredScope<'_>,
    heading: &str,
    ctx: &mut ScanContext,
) -> Vec<ScannedRow> {
    let mut out = Vec::new();
    if let Some(archetype) = scope.archetype {
        let mut of_archetype = 0usize;
        for doc in &spec.inner.documents {
            // Header-tier gate first (CR-047): only an archetype-matching,
            // non-excluded document pays for a body parse.
            if doc.concept_type() != Some(archetype) {
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
                    archetype: archetype.to_string(),
                },
            );
        }
    }
    if let Some(document) = scope.document {
        let path = root.join(document);
        if !scope.excludes(root, &path) {
            if let Some(doc) = ctx.harvest(scope.name, root, document) {
                out.extend(rows_of(doc, &path, heading));
            }
        }
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
        ScanDiagnostic::UnreadableDocument { .. } => "unreadable-declared-document",
    }
}

/// The path a scan diagnostic is about and its human message. The path is the
/// unreadable document where there is one, and the scope root otherwise —
/// a declaration-level fault has no document to point at.
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
        ScanDiagnostic::UnreadableDocument { path, error } => (
            path.clone(),
            format!(
                "declaration '{declaration}' names document '{}', which could not be read \
                 ({error}) — every row it would have contributed is missing",
                path.display()
            ),
        ),
    }
}

// The targeted scan of a declared auxiliary source — a file the corpus walk
// excludes as a non-artifact (FR-044 glossary-harvester pattern) — lives on
// `ScanContext::harvest`, which caches it per path and reports a failed read
// instead of swallowing it (CR-054).

#[cfg(test)]
mod tests {
    use super::*;

    fn tmpdir(tag: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "quire_scan_{tag}_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    // TC-822 (FR-050-AC-19, CR-054): the harvest cache. A model names the same
    // auxiliary document from several declarations — typically `spec/tests.md`,
    // the largest file in the repo — and it must be read and parsed once, not
    // once per declaration. Deleting the file between calls makes the cache
    // observable: a second read would fail.
    #[test]
    fn tc822_harvest_reads_a_declared_document_once() {
        let root = tmpdir("harvest_cache");
        let doc = root.join("tests.md");
        std::fs::write(&doc, "# Matrix\n\n## Test Cases\n\nbody\n").unwrap();

        let mut ctx = ScanContext::default();
        let first = ctx
            .harvest("test-case", &root, Path::new("tests.md"))
            .expect("first read")
            .clone();

        std::fs::remove_file(&doc).unwrap();

        let second = ctx
            .harvest("traces-to", &root, Path::new("tests.md"))
            .expect("served from cache, not re-read");
        assert_eq!(&first, second);
        assert!(
            ctx.into_diagnostics(false).is_empty(),
            "a cached hit must not report the file as unreadable"
        );

        std::fs::remove_dir_all(&root).ok();
    }

    // TC-822 (FR-050-AC-19, CR-054): the failure is cached too — one read —
    // but every declaration that named the unreadable document is reported,
    // because which declarations are affected is what the operator acts on.
    #[test]
    fn tc822_an_unreadable_document_is_reported_per_declaration() {
        let root = tmpdir("harvest_missing");
        let mut ctx = ScanContext::default();

        assert!(ctx
            .harvest("test-case", &root, Path::new("missing.md"))
            .is_none());
        assert!(ctx
            .harvest("traces-to", &root, Path::new("missing.md"))
            .is_none());

        let reported: Vec<String> = ctx
            .into_diagnostics(false)
            .into_iter()
            .map(|(declaration, _)| declaration)
            .collect();
        assert_eq!(reported, vec!["test-case", "traces-to"]);

        std::fs::remove_dir_all(&root).ok();
    }
}
