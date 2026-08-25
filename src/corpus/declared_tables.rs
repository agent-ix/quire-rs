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
use crate::traceability::SectionNames;

/// One scanned table row: the document it lives in, its cells keyed by the
/// table's own column headers (trimmed, as authored), and the 1-based document
/// line the row sits on (#210).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScannedRow {
    pub path: PathBuf,
    pub cells: BTreeMap<String, String>,
    /// 1-based line of the row in its document, frontmatter included — the
    /// same numbering `validate` findings use, so a consumer can render
    /// `path:line:` and an editor can jump to the authored row.
    pub line: usize,
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
    /// `Some(id_column)` when this scan is a **trace target** — a declaration
    /// that mints ids out of that column — and `None` when it merely reads an
    /// existing column (a document reference, an obligation source).
    ///
    /// One field rather than two, because both things the minting diagnostics
    /// need are the same fact: *this* declaration is where ids come from, and
    /// *that* column is where their identity is (CR-117, #270).
    ///
    /// The distinction is what keeps `section-matches-nothing` off healthy
    /// repositories. A reference declaration's section is legitimately
    /// optional — the ecosystem's `functional-coverage` reads `## Functional
    /// Requirement Coverage`, which the `spec-matrix` template emits only when
    /// there is something to put in it — so firing there would report a
    /// finding on every well-formed Test Matrix in the corpus. A trace
    /// target's section is not optional: it is the whole of what the
    /// declaration selects the document for, and an archetype-matching
    /// document without it mints nothing.
    pub mints: Option<&'a str>,
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
// The shared `MatchesNothing` postfix is the point, not an accident of naming:
// each variant is one-to-one with the machine reason token `scan_reason`
// returns, and those tokens — `archetype-matches-nothing`,
// `section-matches-nothing`, `id-column-matches-nothing` — are ecosystem
// vocabulary asserted by fixtures in `agent-ix/qa-corpus` and read by quoin.
// Renaming the variants to satisfy the lint would put the enum and the
// vocabulary one rename apart from disagreeing.
#[allow(clippy::enum_variant_names)]
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

    /// The archetype selected the document and the declared **section** is not
    /// in it (CR-117, `agent-ix/quire-rs#270`).
    ///
    /// The dominant ecosystem failure, and until now the silent one: `rows_of`
    /// returned an empty `Vec` for a section it could not find, so a Test
    /// Matrix heading one word off produced the same payload as a matrix with
    /// no rows. A candidate census over 239 repositories counted **3,514 TC ids across 88
    /// repositories** mint nothing for this reason alone, and those repositories
    /// report 6.77% of rows backed against 32.55% for repositories whose
    /// heading matches.
    ///
    /// Reported **per document**, and never gated on whether the model minted
    /// something else. A repository whose FR criteria mint normally while its
    /// Test Matrix strands 606 ids is exactly the shape a model-wide gate hides
    /// (`agent-ix/quire-rs#304`).
    ///
    /// Carries `id_column` although it cannot check it: the wrong heading
    /// strands the table before the column is ever read, so a reader who fixes
    /// only the heading meets a second mismatch on the next run. The message
    /// names what it could NOT check, so that loop is walked once.
    SectionMatchesNothing {
        /// Scope-relative, `/`-separated — the form a report path and an
        /// `exclude` glob are both matched against (CR-038).
        document: String,
        archetype: String,
        /// EVERY section the declaration asked for, as authored (CR-118).
        /// A declaration may name several since #272, and "the declared
        /// section was not found" naming one of three is a message whose
        /// reader has to open the manifest to learn what the other two were.
        sections: Vec<String>,
        /// The headings the document actually has, in document order.
        headings: Vec<String>,
        /// The id column this scan could not reach.
        id_column: String,
    },

    /// The section was found and the declared **id column** is not among the
    /// table's headers (CR-117, `agent-ix/quire-rs#270`).
    ///
    /// The near neighbour of [`Self::SectionMatchesNothing`], and it had to be
    /// its own token rather than one shared "matched nothing": the two produce
    /// payloads that agree in `totals`, `groups`, `diagnostics`,
    /// `binding_census`, `metrics` and `criteria`, and differ only in a
    /// `row_id: null` inside `unbacked_rows` — a field nobody reads off a
    /// coverage summary. Told only that *something* matched nothing, a reader of
    /// `agent-ix/identity` edits the heading, which is already correct, and
    /// leaves all 606 ids stranded.
    /// Every declared section the document has holds **no table**, so it mints
    /// nothing and no other diagnostic fires.
    ///
    /// A single section holding no table is ordinary — a parent heading whose
    /// rows live under its sub-headings — and reporting it would be noise on
    /// every nested matrix. It is a minting defect only when it is true of
    /// EVERY section the declaration matched, because then the document mints
    /// nothing and the two diagnostics that would have said so both stand
    /// down: `section-matches-nothing` needs no section to match, and
    /// `id-column-matches-nothing` needs a table to read headers from.
    ///
    /// Measured, and the reason this exists: widening the declared section in
    /// #272 made three repositories match a heading whose own content holds no
    /// table, so they LOST their `section-matches-nothing` while still minting
    /// zero — `filament-editor-app` went to a perfect 33/33
    /// `minting.section_hit_rate` with no minting diagnostic at all. That is
    /// the silent zero CR-117 landed one hour earlier to end, reintroduced by
    /// the fix after it.
    SectionHoldsNoTable {
        document: String,
        sections: Vec<String>,
        matched: Vec<String>,
    },
    IdColumnMatchesNothing {
        document: String,
        /// The heading of the section the table was actually found under —
        /// the matched one, not the declared pattern (CR-118). With several
        /// sections declared, "the table under `*Test Case Summary*`" names no
        /// table a reader can open.
        section: String,
        /// The id column the declaration asked for.
        id_column: String,
        /// The headers the table actually has, as authored.
        columns: Vec<String>,
    },
}

impl ScanDiagnostic {
    /// The scope-relative document this diagnostic is about, when it is about
    /// one (FR-050-AC-33 L2).
    ///
    /// `None` for [`Self::ArchetypeMatchesNothing`], which is a fault of the
    /// declaration rather than of any one document — there is no file to open.
    pub(crate) fn document(&self) -> Option<&str> {
        match self {
            Self::ArchetypeMatchesNothing { .. } => None,
            Self::SectionMatchesNothing { document, .. }
            | Self::SectionHoldsNoTable { document, .. }
            | Self::IdColumnMatchesNothing { document, .. } => Some(document),
        }
    }
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
    census: MintingCensus,
}

/// What the minting scans were offered and what they could read
/// (`minting.section_hit_rate`, FR-063-AC-7).
///
/// Counted over **(trace target, document) pairs** rather than over distinct
/// documents. One archetype has one minting declaration in every ecosystem
/// module written so far, so the two coincide today; where they would not, the
/// pair is the honest unit — it is a declaration that failed to read a
/// document, and two declarations failing on one document are two failures to
/// fix.
///
/// Counted **after** `exclude:` applies, unlike the archetype count above: a
/// document the declaration deliberately excludes was never offered to it, so
/// scoring it as a miss would make every repository with a fixture tree look
/// like a repository with a wrong heading.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MintingCensus {
    /// (trace target, document) pairs the archetype selected and `exclude` kept.
    pub selected: usize,
    /// Of those, the pairs whose declared section was found.
    pub section_found: usize,
}

impl ScanContext {
    fn note(&mut self, declaration: &str, diagnostic: ScanDiagnostic) {
        let entry = (declaration.to_string(), diagnostic);
        if !self.diagnostics.contains(&entry) {
            self.diagnostics.push(entry);
        }
    }

    /// The minting census so far. Read before [`Self::into_diagnostics`]
    /// consumes the context.
    pub(crate) fn census(&self) -> MintingCensus {
        self.census
    }

    /// The declaration's diagnostics, sorted so the order is a property of the
    /// model rather than of the walk (NFR-006).
    ///
    /// `minted_anything` says whether the model produced any id at all. A
    /// declared archetype no document has is reported only when it did not: a
    /// model that mints normally while one optional declaration selects nothing
    /// is healthy, and saying so on every repo would be noise nobody reads.
    ///
    /// **The gate is scoped to that one variant, deliberately** (CR-117). The
    /// two minting diagnostics below it are per-document facts, and a
    /// model-wide "did anything mint?" switch suppresses one declaration's
    /// finding because another declaration succeeded — which is the defect
    /// `agent-ix/quire-rs#304` is filed for. `agent-ix/identity` mints its FR
    /// criteria normally and strands 606 TC ids; under a shared gate it would
    /// report neither.
    ///
    /// A minting document that cannot be *read* is reported by the walk, as
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

/// Scan the tables under the declared `sections` for every corpus document of
/// `scope`'s archetype, in corpus order.
///
/// Every section a declaration names contributes (CR-118, #272), in **document
/// order** — a matrix that groups its rows under three headings mints all
/// three tables, and the order of the rows is a property of the document rather
/// than of the order the module happened to list its headings in (NFR-006).
pub(crate) fn scan(
    spec: &Spec,
    root: &Path,
    scope: DeclaredScope<'_>,
    sections: &SectionNames,
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
        // `scanned` distinguishes the two shapes `rows_of` collapsed into one
        // empty `Vec`: no declared section is there at all, and a declared
        // section that is there and holds no table (CR-117). Only the first is
        // a minting defect; the second is an empty matrix, which is a
        // repository with no tests yet.
        let scanned = tables_of(doc.body(), &doc.path, sections);
        if scope.mints.is_some() {
            ctx.census.selected += 1;
            if !scanned.is_empty() {
                ctx.census.section_found += 1;
            }
        }
        if scanned.is_empty() {
            if let Some(id_column) = scope.mints {
                ctx.note(
                    scope.name,
                    ScanDiagnostic::SectionMatchesNothing {
                        document: relative_path(root, &doc.path),
                        archetype: scope.archetype.to_string(),
                        sections: sections.names().to_vec(),
                        headings: document_headings(doc.body()),
                        id_column: id_column.to_string(),
                    },
                );
            }
            continue;
        }
        // Every matched section holding NO table is a silent zero: the
        // document mints nothing and neither existing diagnostic can fire.
        // One such section among others is ordinary (a parent heading whose
        // rows live under its sub-headings), so this is a property of the
        // document, not of a section.
        if scope.mints.is_some()
            && !scanned.is_empty()
            && !scanned
                .iter()
                .any(|(_, t)| matches!(t, ScannedTable::Table { .. }))
        {
            ctx.note(
                scope.name,
                ScanDiagnostic::SectionHoldsNoTable {
                    document: relative_path(root, &doc.path),
                    sections: sections.names().to_vec(),
                    matched: scanned.iter().map(|(h, _)| h.clone()).collect(),
                },
            );
        }
        for (heading, table) in scanned {
            let ScannedTable::Table { headers, rows } = table else {
                continue; // this section holds none; the check above covers
                          // the case where NONE of them does
            };
            // The id column is read off the CALLER's `TraceTarget`
            // (`coverage.rs` does `row.cell(&target.id_column)`), so the
            // check could equally have lived there. It lives here because
            // this is the only place that holds the table's headers when
            // the table has **no rows** — a caller iterating `ScannedRow`s
            // sees nothing to inspect — and because putting it beside the
            // section check keeps the two halves of one declaration's
            // failure in one arm each, rather than one here and one three
            // call sites away.
            //
            // Per MATCHED section since CR-118: a matrix whose `Test Case
            // Summary (plugin scope)` table is right and whose `(discovery
            // scope)` table names the wrong column has one working table and
            // one broken one, and a check over "the" table can only describe
            // one of them.
            if let Some(id_column) = scope.mints {
                if !headers.iter().any(|h| h.eq_ignore_ascii_case(id_column)) {
                    ctx.note(
                        scope.name,
                        ScanDiagnostic::IdColumnMatchesNothing {
                            document: relative_path(root, &doc.path),
                            section: heading,
                            id_column: id_column.to_string(),
                            columns: headers,
                        },
                    );
                }
            }
            out.extend(rows);
        }
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

/// What one **matched** section offered a declared scan (CR-117).
///
/// Two of the three cases `rows_of` used to answer with the same empty `Vec`.
/// The third — no section by that name at all — is not a
/// variant here since CR-118: with several names to try, "not found" is a fact
/// about the whole declaration rather than about one section, and
/// [`tables_of`] says it by returning nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ScannedTable {
    /// The section is there and holds no table.
    NoTable,
    /// The table, with its headers kept alongside its rows — a zero-row table
    /// still has columns to check an `id_column` against.
    Table {
        headers: Vec<String>,
        rows: Vec<ScannedRow>,
    },
}

/// The rows of every declared section's table in one document.
///
/// The lossy view of [`tables_of`], kept for the callers that only ever wanted
/// rows (`crate::obligation`'s per-document harvest).
pub(crate) fn rows_of(
    doc: &QuireDocument,
    path: &Path,
    sections: &SectionNames,
) -> Vec<ScannedRow> {
    tables_of(doc, path, sections)
        .into_iter()
        .flat_map(|(_, table)| match table {
            ScannedTable::Table { rows, .. } => rows,
            ScannedTable::NoTable => Vec::new(),
        })
        .collect()
}

/// Every section of `doc` a declaration names, in document order, paired with
/// what it held (CR-118).
///
/// An empty result is the one shape that means **no declared section is in this
/// document at all** — the `section-matches-nothing` defect, whose near-miss
/// list the caller derives once from [`document_headings`]. A section that is
/// present and holds no table is an entry whose value is
/// [`ScannedTable::NoTable`], which is an empty matrix rather than a
/// declaration fault.
pub(crate) fn tables_of(
    doc: &QuireDocument,
    path: &Path,
    sections: &SectionNames,
) -> Vec<(String, ScannedTable)> {
    // Document order, over EVERY matching section rather than the first: since
    // #272 the whole point is a matrix whose rows are spread across several
    // headings. `crate::query::section` returns the first match, which is what
    // `table_of` still does for a single heading.
    //
    // **[RAN]** over the 393 `type: TestMatrix` documents in `~/dev`: not one
    // repeats the ecosystem's declared heading, so for a single-name
    // declaration this reads exactly the section `section()` used to.
    crate::query::sections(doc, None)
        .into_iter()
        .filter(|sec| sections.matches(&sec.heading))
        .map(|sec| (sec.heading.trim().to_string(), table_in(doc, path, sec)))
        .collect()
}

/// The table one already-selected section holds.
fn table_in(doc: &QuireDocument, path: &Path, sec: &crate::ast::QuireSection) -> ScannedTable {
    let Some((table, row_lines)) = crate::query::parse_table_with_lines(&sec.content) else {
        return ScannedTable::NoTable;
    };
    // The frontmatter's line count, so a section's body-relative `start_line`
    // converts to the 1-based document line `validate` findings use (#210).
    // Parsed line information was discarded here from v0.1: `ScannedRow` had
    // no line, so no coverage record could say which authored row it was
    // about, and `path:line:` output was impossible downstream.
    let line_offset = crate::validate_document::body_line_offset(&doc.raw);
    let headers: Vec<String> = table.headers.iter().map(|h| h.trim().to_string()).collect();
    let rows = table
        .rows
        .iter()
        .zip(row_lines)
        .map(|(row, rel)| {
            let mut cells = BTreeMap::new();
            for (idx, header) in headers.iter().enumerate() {
                cells.insert(
                    header.clone(),
                    row.get(idx).map(|v| v.trim()).unwrap_or("").to_string(),
                );
            }
            ScannedRow {
                path: path.to_path_buf(),
                cells,
                // `validate_document::to_doc_line` arithmetic over the row's
                // body line: content index `rel` sits `rel + 1` lines below
                // the 0-based heading line, and `+ 1` converts to 1-based.
                // Hand-verified against authored fixtures (TC-955); NOT
                // `ears::abs_line`, which is one line short of the file for
                // exactly this shape — a latent defect in the grammar
                // findings' `line`, out of #210's scope and noted there.
                line: line_offset + sec.start_line + rel + 2,
            }
        })
        .collect();
    ScannedTable::Table { headers, rows }
}

/// Every heading the document carries, in document order, deduped.
///
/// The near-miss list a `section-matches-nothing` message names. Deduped
/// because a document repeating `### Notes` under six parents would spend the
/// whole message saying so, and the reader is scanning for the one heading that
/// looks like the declared one.
fn document_headings(doc: &QuireDocument) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for section in crate::query::sections(doc, None) {
        let heading = section.heading.trim();
        if !heading.is_empty() && !out.iter().any(|h| h == heading) {
            out.push(heading.to_string());
        }
    }
    out
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
        ScanDiagnostic::SectionMatchesNothing { .. } => "section-matches-nothing",
        ScanDiagnostic::SectionHoldsNoTable { .. } => "section-holds-no-table",
        ScanDiagnostic::IdColumnMatchesNothing { .. } => "id-column-matches-nothing",
    }
}

/// How many near-miss names a message lists before it stops.
///
/// A message is read by somebody comparing two strings, and past a dozen the
/// list stops being a comparison and becomes a document dump —
/// `agent-ix/identity`'s matrix carries roughly thirty headings.
const NAMES_SHOWN: usize = 12;

/// `'a', 'b', 'c'`, capped, with the remainder counted rather than dropped.
fn quoted_list(names: &[String]) -> String {
    if names.is_empty() {
        return "none at all".to_string();
    }
    let mut out = names
        .iter()
        .take(NAMES_SHOWN)
        .map(|n| format!("'{n}'"))
        .collect::<Vec<_>>()
        .join(", ");
    if names.len() > NAMES_SHOWN {
        out.push_str(&format!(" and {} more", names.len() - NAMES_SHOWN));
    }
    out
}

/// The path a scan diagnostic is about and its human message.
///
/// A declaration-level fault has no single document to point at, so its path is
/// the scope root. The two minting faults do, and theirs is the document
/// (FR-050-AC-33): the fix is a one-word edit in one file, and a finding that
/// cannot say which file leaves the reader to grep for it.
///
/// Every message names **what was found and what was declared**, in that order.
/// "the declared id column was not found" is a sentence a reader cannot act on
/// without opening two files; naming both values makes the diff the sentence.
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
        ScanDiagnostic::SectionMatchesNothing {
            document,
            archetype,
            sections,
            headings,
            id_column,
        } => (
            root.join(document),
            format!(
                "declaration '{declaration}' selected {document} by archetype \
                 '{archetype}' and found no section matching {} in it — the headings the \
                 document has are {}. The declared table is never reached, so every id \
                 under it is stranded and the document mints nothing. The declared id \
                 column '{id_column}' could NOT be checked either: fix the heading first, \
                 then confirm that column exists, or the same rows come back unminted \
                 for the second reason",
                // EVERY declared name (CR-118). A declaration naming three
                // headings and a message naming one leaves its reader to guess
                // which of the three to spell like the document.
                quoted_list(sections),
                quoted_list(headings)
            ),
        ),
        ScanDiagnostic::SectionHoldsNoTable {
            document,
            sections,
            matched,
        } => (
            root.join(document),
            format!(
                "declaration '{declaration}' found {} in {document} — every section it                  declares that this document has — and not one of them holds a table.                  The declared sections are {}. So the document mints nothing, and the two                  diagnostics that would say so both stand down: the section WAS found,                  and there is no table to read an id column from. A heading whose rows                  live under its sub-headings is the usual cause — declare those                  sub-headings, or the pattern that reaches them",
                quoted_list(matched),
                quoted_list(sections)
            ),
        ),
        ScanDiagnostic::IdColumnMatchesNothing {
            document,
            section,
            id_column,
            columns,
        } => (
            root.join(document),
            format!(
                "declaration '{declaration}' declares id column '{id_column}', which the \
                 table under '{section}' in {document} does not have — the columns it has \
                 are {}. The rows are read and none of them yields an id, so the document \
                 mints nothing and every row it should have minted reports with no \
                 identity",
                quoted_list(columns)
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
mod cr110_scanned_table {
    use std::path::Path;

    use ix_trace_rs::trace;

    use super::{document_headings, tables_of, ScannedTable};
    use crate::traceability::SectionNames;

    fn parse(body: &str) -> crate::ast::QuireDocument {
        crate::parser::parse_document(body)
    }

    #[trace("TC-1041", "FR-050-AC-35")]
    // every matched section holding NO table is a silent
    // zero, and one among others is not (CR-120). #272's widening made three
    // repositories match a table-less heading and LOSE their
    // `section-matches-nothing` while still minting zero — one of them
    // reporting a perfect 33/33 `minting.section_hit_rate` with no minting
    // diagnostic at all, which is the shape CR-117 landed one hour earlier to
    // end.
    #[test]
    fn tc1041_a_matched_heading_with_no_table_is_not_silence() {
        let path = Path::new("spec/tests.md");

        // The regressed shape, taken from `agent-duncan/spec/tests.md`: the
        // declared heading is THERE and its own content holds no table,
        // because its rows live under a sub-heading the declaration does not
        // name.
        let doc = parse(
            "---\nid: TM-001\ntype: TestMatrix\n---\n\n\
             ## Integration Test Matrix\n\n### External Service Integrations\n\n\
             | Test ID |\n|---|\n| TC-001 |\n",
        );
        let found = tables_of(&doc, path, &SectionNames::from("Integration Test Matrix"));
        assert_eq!(
            found,
            vec![("Integration Test Matrix".to_string(), ScannedTable::NoTable)],
            "the section is found and holds no table — neither sibling \
             diagnostic can fire on this, which is why it needs its own"
        );
        // NOT empty: `section-matches-nothing` is therefore unreachable here.
        assert!(
            !found.is_empty(),
            "a matched-but-table-less section must not read as a missing one, \
             or the silent zero comes back"
        );
        // And no table means no headers, so `id-column-matches-nothing` has
        // nothing to read either. Both siblings stand down; the document mints
        // nothing. That is the whole of the defect.
        assert!(
            !found
                .iter()
                .any(|(_, t)| matches!(t, ScannedTable::Table { .. })),
            "no table means no headers to check an id column against"
        );

        // One table-less section AMONG OTHERS is ordinary — a parent heading
        // whose rows live below it — and must stay unreported, or every nested
        // matrix in the ecosystem grows a finding.
        let nested = parse(
            "---\nid: TM-001\ntype: TestMatrix\n---\n\n\
             ## Suites\n\nProse.\n\n## Test Case Summary\n\n\
             | Test ID |\n|---|\n| TC-001 |\n",
        );
        let both = tables_of(
            &nested,
            path,
            &SectionNames::new(vec!["Suites".to_string(), "Test Case Summary".to_string()]),
        );
        assert!(
            both.iter()
                .any(|(_, t)| matches!(t, ScannedTable::Table { .. })),
            "a document that mints from ANY matched section is not a silent \
             zero, whatever its other headings hold"
        );
    }

    #[trace("TC-1033", "FR-050-AC-33")]
    // the three ways a scan produces no rows are three
    // ANSWERS, not one empty `Vec` (CR-117). `rows_of` collapsed them, so a
    // heading one word off and a matrix with no rows were the same fact
    // downstream — which is why stranded ids reported as a smaller
    // denominator and nothing else.
    #[test]
    fn tc1033_the_scan_says_which_way_it_found_nothing() {
        let path = Path::new("spec/tests.md");

        // A heading the document does not have — and the near misses named,
        // because the actionable half of the finding is the list to compare
        // against. Since CR-118 "no such section" is an empty result rather
        // than a variant: with several declared names, not-found is a fact
        // about the declaration and not about one of them.
        let doc = parse(
            "---\nid: TM-001\ntype: TestMatrix\n---\n\n## Overview\n\nProse.\n\n\
             ## Test Cases\n\n| Test ID |\n|---|\n| TC-001 |\n",
        );
        assert!(
            tables_of(&doc, path, &SectionNames::from("Test Case Summary")).is_empty(),
            "a missing section must not read as a missing table"
        );
        assert_eq!(document_headings(&doc), vec!["Overview", "Test Cases"]);

        // The section is there and holds no table: an empty matrix, which is a
        // repository with no tests yet rather than a declaration defect.
        assert_eq!(
            tables_of(&doc, path, &SectionNames::from("Overview")),
            vec![("Overview".to_string(), ScannedTable::NoTable)],
            "prose under the declared heading is not a missing heading"
        );

        // The table, with its HEADERS alongside its rows. A zero-row table has
        // no `ScannedRow` to read a header off, so a caller checking an
        // `id_column` against the rows alone cannot see the column at all.
        let found = tables_of(&doc, path, &SectionNames::from("Test Cases"));
        let [(heading, ScannedTable::Table { headers, rows })] = found.as_slice() else {
            panic!("the declared table must be read: {found:?}");
        };
        assert_eq!(heading, "Test Cases");
        assert_eq!(headers, &vec!["Test ID".to_string()]);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].cell("test id"), Some("TC-001"));

        let empty = parse(
            "---\nid: TM-001\ntype: TestMatrix\n---\n\n## Test Cases\n\n\
             | Test ID | Status |\n|---|---|\n",
        );
        let found = tables_of(&empty, path, &SectionNames::from("Test Cases"));
        let [(_, ScannedTable::Table { headers, rows })] = found.as_slice() else {
            panic!("a header row with no data rows is still a table: {found:?}");
        };
        assert_eq!(headers, &vec!["Test ID".to_string(), "Status".to_string()]);
        assert!(rows.is_empty());
    }

    #[trace("TC-1033", "FR-050-AC-33")]
    // headings are deduped and kept in document order:
    // a document repeating `### Notes` under six parents would spend the whole
    // message saying so, and the reader is scanning for one near miss.
    #[test]
    fn tc1033_the_heading_list_is_deduped_in_document_order() {
        let doc = parse(
            "---\nid: TM-001\ntype: TestMatrix\n---\n\n## Alpha\n\n### Notes\n\n\
             ## Beta\n\n### Notes\n\n## Alpha\n",
        );
        assert!(tables_of(&doc, Path::new("x.md"), &SectionNames::from("Missing")).is_empty());
        assert_eq!(document_headings(&doc), vec!["Alpha", "Notes", "Beta"]);
    }

    #[trace("TC-1037", "FR-050-AC-34")]
    // every section a declaration names contributes its (CR-118)
    // own table, in DOCUMENT order rather than in the order the module happened
    // to list its headings — the row order in a payload is a property of the
    // document (NFR-006).
    #[test]
    fn tc1037_every_named_section_contributes_in_document_order() {
        let doc = parse(
            "---\nid: TM-001\ntype: TestMatrix\n---\n\n\
             ## Test Case Summary\n\n| Test ID |\n|---|\n| TC-001 |\n\n\
             ## Edge Cases\n\n| Test ID |\n|---|\n| TC-002 |\n\n\
             ## Test Case Summary (plugin scope)\n\n| Test ID |\n|---|\n| TC-003 |\n",
        );
        let path = Path::new("spec/tests.md");
        // Declared LAST-first, and matched by a pattern and by a name.
        let declared = SectionNames::new(vec!["Edge Cases".to_string(), "*Summary*".to_string()]);
        let rows = super::rows_of(&doc, path, &declared);
        assert_eq!(
            rows.iter()
                .filter_map(|r| r.cell("Test ID"))
                .collect::<Vec<_>>(),
            vec!["TC-001", "TC-002", "TC-003"],
            "document order, not declaration order"
        );
        assert_eq!(
            tables_of(&doc, path, &declared)
                .iter()
                .map(|(h, _)| h.as_str())
                .collect::<Vec<_>>(),
            vec![
                "Test Case Summary",
                "Edge Cases",
                "Test Case Summary (plugin scope)"
            ]
        );
    }
}

#[cfg(test)]
mod cr069_regressions {
    /// TC-892: regression (FR-049, CR-069) — the generator's minimized witness.
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
