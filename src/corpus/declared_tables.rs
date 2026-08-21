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
    /// Compile `patterns`, refusing the **whole list** on the first pattern
    /// that does not compile (#215). Until then an uncompilable pattern was
    /// silently dropped while the rest still applied — partial filtering with
    /// no diagnostic, for any caller not routed through
    /// [`TraceabilityModel::validate`](crate::traceability::TraceabilityModel::validate).
    /// All-or-nothing mirrors what validation does at module load: an invalid
    /// glob list never filters, it errors.
    ///
    /// The error names the offending pattern; the caller names the key it was
    /// authored under.
    pub(crate) fn compile(patterns: &[String]) -> Result<Self, String> {
        if patterns.is_empty() {
            return Ok(Self::default());
        }
        let mut builder = globset::GlobSetBuilder::new();
        for pattern in patterns {
            let glob = globset::Glob::new(pattern)
                .map_err(|e| format!("invalid glob pattern '{pattern}': {e}"))?;
            builder.add(glob);
        }
        let set = builder
            .build()
            .map_err(|e| format!("glob set does not build: {e}"))?;
        Ok(Self { set: Some(set) })
    }

    /// [`Self::compile`] for patterns that already passed
    /// `TraceabilityModel::validate` at module load (FR-050-AC-2), where an
    /// error is unreachable. A hand-built model that skipped validation fails
    /// the debug assertion in tests; in release it filters **nothing** — never
    /// partially — which the count/diagnostic surfaces of #215 make visible.
    pub(crate) fn compile_validated(patterns: &[String]) -> Self {
        match Self::compile(patterns) {
            Ok(set) => set,
            Err(reason) => {
                debug_assert!(
                    false,
                    "unvalidated exclude patterns reached compile: {reason}"
                );
                Self::default()
            }
        }
    }

    /// True when `relative` — a scope-relative, `/`-separated path — is
    /// excluded.
    ///
    /// `pub(crate)` since CR-085: the symbol walk has already normalized the
    /// relative path for its own identity, and going through [`excludes`] would
    /// make it build a `Path` and re-derive that string for every file.
    pub(crate) fn matches(&self, relative: &str) -> bool {
        self.set.as_ref().is_some_and(|set| set.is_match(relative))
    }

    /// True when `path` is excluded, deriving the scope-relative form only
    /// when there is something to match it against.
    pub(crate) fn excludes(&self, root: &Path, path: &Path) -> bool {
        !self.is_empty() && self.matches(&relative_path(root, path))
    }

    pub(crate) fn is_empty(&self) -> bool {
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
/// Run to a **fixpoint** (CR-069). One pass leaves a chained range half-read:
/// `FR-001..FR-003..FR-005` expands its leftmost range and leaves `..FR-005`
/// behind, which the declaration's pattern then rejects — so a cell the engine
/// could have read reports as a dangling trace reference instead. Nested
/// annotations behave the same way, `((a))` needing two passes.
///
/// **[RAN]** before changing it: zero cells across the 237 `~/dev` spec bundles
/// contain a chained range, so this closes a latent defect and moves no current
/// output.
///
/// Each pass consumes at least one `..` it can expand, and an expansion emits
/// comma-separated ids containing no `..` of its own, so the loop terminates.
/// The bound is a guard on the fuzz surface, not the termination argument.
pub(crate) fn normalize_reference_cell(
    cell: &str,
    strip_annotations: bool,
    expand_ranges: bool,
) -> String {
    const MAX_PASSES: usize = 16;
    let mut out = cell.to_string();
    for pass in 0..MAX_PASSES {
        let mut once = out.clone();
        if strip_annotations {
            once = re_parenthetical().replace_all(&once, " ").to_string();
        }
        if expand_ranges {
            once = re_range()
                .replace_all(&once, |caps: &regex::Captures<'_>| expand(caps))
                .to_string();
        }
        if once == out {
            return out;
        }
        out = once;
        // Reaching the ceiling means the termination argument above is wrong —
        // each pass consumes at least one `..` and an expansion emits none — so
        // it would take a cell with 16 chained ranges. Fail in tests rather than
        // silently returning a non-fixpoint the pattern will then reject
        // (CR-072).
        debug_assert!(
            pass + 1 < MAX_PASSES,
            "normalize_reference_cell did not converge in {MAX_PASSES} passes: {cell:?}"
        );
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

#[cfg(test)]
mod props_metamorphic {
    use proptest::prelude::*;

    /// Cells built from the tokens the two opt-in normalizations rewrite —
    /// parentheses and `..` ranges — so chained and nested forms are reached.
    fn cellish() -> impl Strategy<Value = String> {
        let tok = prop_oneof![
            Just("(".to_string()),
            Just(")".to_string()),
            Just("..".to_string()),
            Just(", ".to_string()),
            "FR-[0-9]{3}".prop_map(|s| s),
            "[a-z]{1,3}".prop_map(|s| s),
        ];
        proptest::collection::vec(tok, 0..10).prop_map(|v| v.concat())
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(3000))]

        /// TC-892 (FR-049, CR-069, Property): cell normalization is idempotent
        /// under every combination of the two opt-in flags. Its counterexample
        /// before the fixpoint loop was the chained range
        /// `FR-375..FR-432..FR-432`, which left a `..` the pattern rejects.
        #[test]
        fn tc892_reference_cell_normalization_is_idempotent(
            s in cellish(),
            strip in any::<bool>(),
            expand in any::<bool>(),
        ) {
            let once = super::normalize_reference_cell(&s, strip, expand);
            prop_assert_eq!(
                super::normalize_reference_cell(&once, strip, expand),
                once.clone(),
                "input={:?} strip={} expand={}", s, strip, expand
            );
        }

        /// TC-892 (FR-049, CR-069, Property): with both flags off the cell is
        /// returned unchanged — normalization is opt-in (CR-015), so the
        /// no-declaration path must be the identity.
        #[test]
        fn tc892_no_flags_is_the_identity(s in cellish()) {
            prop_assert_eq!(super::normalize_reference_cell(&s, false, false), s.clone());
        }
    }
}

#[cfg(test)]
mod cr069_regressions {
    /// TC-892 regression (FR-049, CR-069): the generator's minimized witness.
    /// A chained range expanded only its leftmost pair and left a `..` behind,
    /// which the declaration's pattern then rejects — so a cell the engine could
    /// have read reported as a dangling trace reference instead.
    #[test]
    fn tc892_chained_ranges_expand_completely() {
        let out = super::normalize_reference_cell("FR-001..FR-003..FR-005", false, true);
        assert_eq!(out, "FR-001, FR-002, FR-003, FR-004, FR-005");
        assert!(!out.contains(".."), "no range marker may survive: {out}");

        // A nested annotation leaves a stray `)`: the annotation regex is not
        // nesting-aware, so `((note))` matches `((note)` and the outer `)`
        // survives. Recorded as it is rather than asserted away — the residual
        // carries no id, so it cannot mint a false reference, and making the
        // stripper nesting-aware is a separate decision with its own blast
        // radius. What CR-069 fixes is that the result is now a **fixpoint**:
        // a second pass changes nothing.
        let once = super::normalize_reference_cell("TC-001 ((note))", true, false);
        assert_eq!(once, "TC-001  )");
        assert_eq!(
            super::normalize_reference_cell(&once, true, false),
            once,
            "whatever it leaves behind must be stable"
        );
    }
}
