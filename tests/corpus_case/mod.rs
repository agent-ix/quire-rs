//! The shared corpus-case harness (FR-050-AC-29, FR-065, CR-098 / CR-106).
//!
//! A case declares a whole miniature repository and the envelope it expects
//! out. The harness runs the real `compute` path over it and asserts.
//!
//! **The inputs are static files, read in place** (FR-065-AC-1). They were
//! strings inside one JSON blob, materialised into a tempdir under a hardcoded
//! `module/`/`spec/`/`src/` layout — which meant no case could express a
//! `tests/` topology or exercise `source_exclude`, and no case could be read
//! without running the harness. They now live in `agent-ix/qa-corpus`, pinned
//! as a submodule at `corpus/`, and this reads the directory the operator can
//! `cd` into.
//!
//! **Detection is graded, not boolean** (FR-065-AC-11/AC-12). Each expectation
//! belongs to a level, and a failure names the level lost — "the case failed"
//! and "the message stopped naming the row" are different repairs.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// The detection ladder. `L1 < L2 < L3`, so the first level lost is the
/// minimum over the failures a case produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    /// Did anything fire?
    L1Detected,
    /// Did it name the right `path:line`?
    L2Localised,
    /// Did the message name the thing to change?
    L3Actionable,
}

impl Level {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::L1Detected => "L1 detected",
            Self::L2Localised => "L2 localised",
            Self::L3Actionable => "L3 actionable",
        }
    }
}

/// One assertion that did not hold, and the level it belongs to.
#[derive(Debug)]
pub struct Mismatch {
    pub level: Level,
    pub detail: String,
}

/// What a case run produced.
#[derive(Debug)]
pub struct Outcome {
    pub case: String,
    pub issue_ref: String,
    pub mismatches: Vec<Mismatch>,
}

impl Outcome {
    pub fn passed(&self) -> bool {
        self.mismatches.is_empty()
    }

    /// The deepest level the case reached before losing one — `None` when it
    /// lost at L1, because it reached nothing.
    pub fn level_reached(&self) -> Option<Level> {
        match self.level_lost() {
            None => Some(Level::L3Actionable),
            Some(Level::L1Detected) => None,
            Some(Level::L2Localised) => Some(Level::L1Detected),
            Some(Level::L3Actionable) => Some(Level::L2Localised),
        }
    }

    /// The first level lost. Reported instead of a bare failure because L1 and
    /// L3 losses are different repairs: one is a detector that stopped firing,
    /// the other is a message that stopped naming what to change.
    pub fn level_lost(&self) -> Option<Level> {
        self.mismatches.iter().map(|m| m.level).min()
    }

    /// The failure report. Names the case, its filing, and the level lost, so
    /// a red run says what to go and read.
    pub fn report(&self) -> String {
        let lost = self.level_lost().map(|l| l.as_str()).unwrap_or("nothing");
        let reached = self
            .level_reached()
            .map(|l| l.as_str())
            .unwrap_or("no level");
        let mut out = format!(
            "{} ({}) — reached {reached}, LOST {lost}\n",
            self.case, self.issue_ref
        );
        for m in &self.mismatches {
            out.push_str(&format!("    [{}] {}\n", m.level.as_str(), m.detail));
        }
        out
    }
}

/// One case's declaration, from `case.yaml`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseMeta {
    pub id: String,
    /// The filing this case is the regression for. Required, and the reason
    /// the harness exists: a fixture whose origin is not recorded becomes a
    /// fixture nobody dares change (FR-065-AC-3, `agent-ix/quire-rs#234`).
    pub issue_ref: String,
    pub mode: String,
    pub language: String,
    pub module: String,
    pub kind: String,
    #[serde(default)]
    pub control_for: Option<String>,
    /// The ticket that will make this case pass. Present means the case
    /// asserts behaviour the engine does not have yet, and is EXPECTED to fail.
    ///
    /// This is what makes "corpus case red before fix" (EPIC #264 rule 3)
    /// workable: a defect gets its fixture the day it is found, the fixture
    /// fails honestly, and the suite still goes green. Without it the only
    /// options are a red build nobody can merge past, or writing the fixture
    /// after the fix — at which point the "before" was never captured and the
    /// regression is untested.
    ///
    /// A pending case that PASSES is itself a failure: the fix landed and the
    /// marker is now lying about the state of the engine.
    #[serde(default)]
    pub pending: Option<String>,
    #[serde(default)]
    pub findable: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    /// The inventory row this fixture claims, when its `id` differs — a
    /// control's id is `<case>-control`, and it covers nothing on its own.
    #[serde(default)]
    pub case: Option<String>,
    /// The ticket a variant binding is sizing (FR-065-CON-3). Required
    /// whenever `module` is not `ecosystem`; `bounds.py` rejects its absence.
    #[serde(default)]
    pub relaxation_ticket: Option<String>,
    /// The invocation that reproduces this case by hand (FR-065-AC-18).
    /// Modelled rather than ignored: `deny_unknown_fields` is only a gate if
    /// every legitimate field is declared, and an ignored one is a field
    /// nothing checks.
    #[serde(default)]
    pub reproduce: Option<String>,
    #[serde(default)]
    pub comment: Option<String>,
    /// Why the case is pending — what the engine does not do yet. Prose, but
    /// required-by-convention beside `pending`: a marker with no reason is one
    /// nobody can decide whether to remove.
    #[serde(default)]
    pub pending_reason: Option<String>,
}

/// What the emitted envelope must say.
///
/// Every field is optional: a case asserts the facts it is about and stays
/// silent on the rest, so an unrelated engine change does not fail forty cases
/// that were never about it (FR-065-AC-5).
/// `deny_unknown_fields`: a typo'd expectation (`diagnostic_reason`,
/// `no_symbol_row`) was silently dropped, so the CI gate graded a case on
/// fewer assertions than its author wrote. `verify.py` caught it and this
/// did not — the stricter checker was not the gate.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseExpect {
    pub backed: Option<usize>,
    pub total: Option<usize>,
    /// L1. `reason` tokens that MUST be present.
    #[serde(default)]
    pub diagnostic_reasons: Vec<String>,
    /// L1. `reason` tokens that must NOT be — the half a fixture usually
    /// forgets, and the half that catches a check firing on healthy input.
    #[serde(default)]
    pub absent_diagnostic_reasons: Vec<String>,
    /// L1 for the counts, L2 for `unbound_example`.
    #[serde(default)]
    pub binding_census: Vec<ExpectCensus>,
    /// L2. Where a diagnostic points, `reason` -> `path` (#261).
    ///
    /// Asserting the reason alone is satisfied by a finding pointing anywhere,
    /// and "it named a place" is the whole claim being made.
    #[serde(default)]
    pub diagnostic_paths: BTreeMap<String, String>,
    /// L3. Substrings each diagnostic's message must carry, `reason` -> texts.
    ///
    /// A finding can carry a correct path while its prose names nothing a
    /// reader can act on. Measured: removing the example criterion from
    /// `catch-all-universal`'s message left every path assertion passing.
    ///
    /// A LIST, because L3 for a mismatch is two facts — what was found and
    /// what was declared — and one substring is satisfied by naming either.
    #[serde(default)]
    pub diagnostic_message_contains: BTreeMap<String, Vec<String>>,
    /// L2. Rows the declaration minted that no symbol backs, EXACTLY.
    ///
    /// This is the field that tells two minting defects apart. A wrong
    /// section name strands the whole table — nothing mints, so the list is
    /// empty. A wrong id column still reads the table and mints a row with a
    /// **null identity**. The `totals`, `groups`, `diagnostics` and
    /// `binding_census` of the two cases are byte-identical; only this
    /// differs, and until it was asserted the corpus could not distinguish
    /// them (reviewed, #297).
    ///
    /// An empty list is an assertion, not an omission: it says the
    /// declaration minted nothing at all.
    #[serde(default)]
    pub unbacked_rows: Option<Vec<ExpectUnbackedRow>>,
    /// L1. Per-document mint counts by target kind, EXACTLY.
    ///
    /// A control's real job is proving the row it is about MINTS — `test-case
    /// 1/1` present. `total` alone cannot say that: it is satisfied by any two
    /// backed ids from anywhere.
    #[serde(default)]
    pub groups: Option<Vec<ExpectGroup>>,
    /// L1. Row ids explained as verified by a method that mints no source
    /// symbol (#259). Asserted by id, not by count: a count is satisfied by
    /// exempting the wrong row.
    #[serde(default)]
    pub no_symbol_rows: Option<Vec<String>>,
    /// L1. Per-metric expectations, keyed on the metric name.
    #[serde(default)]
    pub metrics: Vec<ExpectMetric>,
    /// Substrings `quire validate` must report, for a case whose family is a
    /// STRUCTURAL defect rather than a coverage one.
    ///
    /// `undeclared-type-value` is the first: a `Type` cell outside the declared
    /// vocabulary produces a coverage payload **byte-identical to a healthy
    /// control's**, so a corpus that ran only `coverage` asserted nothing about
    /// the family its fixture was named for.
    #[serde(default)]
    pub validate_contains: Vec<String>,
    /// Substrings `quire validate` must NOT report — the control half. Every
    /// fixture here shares three structural findings, so asserting presence
    /// without absence would restate what all of them produce.
    #[serde(default)]
    pub validate_absent: Vec<String>,
}

/// One row the declaration minted that no symbol backs.
///
/// Every field is REQUIRED. `row_id` in particular: `row_id: null` is the
/// whole claim the id-column fixture makes — the row minted, and its identity
/// did not — and an optional field would let an author omit exactly that.
#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ExpectUnbackedRow {
    pub document: String,
    pub row_id: Option<String>,
    pub target_ids: Vec<String>,
}

/// One document's mint count for one declared target kind. All fields
/// required: a partial group is satisfied by the wrong target minting.
#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ExpectGroup {
    pub document: String,
    pub target: String,
    pub backed: usize,
    pub total: usize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectCensus {
    pub language: String,
    pub candidates: Option<usize>,
    pub bound: Option<usize>,
    /// Where the census says one unbound candidate is, `path:line` (#256).
    ///
    /// A count cannot be opened, and `no-symbol-bound` named the language and
    /// nothing else. The exact locus is asserted rather than its presence:
    /// "carries an example" is satisfied by an example pointing anywhere.
    pub unbound_example: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectMetric {
    pub name: String,
    /// `measured` or `not_computed`.
    pub state: Option<String>,
    pub value: Option<u64>,
    pub population: Option<u64>,
    pub examined: Option<u64>,
    pub matched: Option<u64>,
    /// Whether the metric is a ratio over input it could not read.
    pub hollow: Option<bool>,
}

/// One case on disk: its directory, and both declarations.
#[derive(Debug)]
pub struct Case {
    pub dir: PathBuf,
    pub meta: CaseMeta,
    /// What must hold NOW. Graded like any case's, pending or not.
    pub expect: CaseExpect,
    /// What the ticket in `pending:` will make hold, and must NOT hold yet.
    ///
    /// Split out because a single block made `pending:` swallow live
    /// assertions: every expectation in a pending case is graded, and any
    /// failure reads as "expected to fail, and did". So the rule became
    /// "a pending fixture asserts ONLY what is pending", and the facts that
    /// were true today went unasserted — including `unbacked_rows`, the one
    /// field that tells the section-name defect from the id-column one. Both
    /// fixtures could have regressed to minting nothing at all and stayed
    /// green (reviewed, #297).
    ///
    /// Now `expect.yaml` is the live contract and this is the forward one.
    pub expect_pending: Option<CaseExpect>,
}

impl Case {
    pub fn input(&self) -> PathBuf {
        self.dir.join("input")
    }
}

/// Load every module under a module PATH.
///
/// `Registry::load_from` treats each argument as a **search root** whose
/// children are inspected for module manifests — which is exactly a module
/// path. The first version of this reached for `IX_FILAMENT_MODULES_PATH` and
/// `std::env::set_var` behind a `Once`, justified by a claim that
/// `Registry::from_env` was the only public surface reading a search path.
/// That claim was false: `load_from` is public, documented, and already used by
/// three other test files here.
///
/// The env route also wrote process-wide state from a test binary whose eight
/// tests run as threads, and `set_var` is `unsafe fn` from edition 2024 — in a
/// crate whose root carries `forbid(unsafe_code)` and whose ADR-0006 retired
/// Miri on the reasoning that there is no first-party unsafe. One safe line
/// replaces all of it.
fn load_module_path(path: &Path, case_id: &str) -> quire_rs::Registry {
    quire_rs::Registry::load_from(&[path])
        .unwrap_or_else(|e| panic!("{case_id}: module path {} failed: {e}", path.display()))
}

/// Deserialize a merged declaration, naming the directory on failure.
fn parse_meta(value: &serde_yaml::Value, dir: &Path) -> CaseMeta {
    serde_yaml::from_value(value.clone())
        .unwrap_or_else(|e| panic!("{}: case.yaml: {e}", dir.display()))
}

/// Set a string key on a mapping, for the fields a set derives per variant.
fn set_str(value: &mut serde_yaml::Value, key: &str, text: &str) {
    if let serde_yaml::Value::Mapping(map) = value {
        map.insert(
            serde_yaml::Value::String(key.to_string()),
            serde_yaml::Value::String(text.to_string()),
        );
    }
}

/// A case's expectations, from the directory holding its `input/`.
fn read_expect(dir: &Path) -> CaseExpect {
    serde_yaml::from_str(
        &std::fs::read_to_string(dir.join("expect.yaml"))
            .unwrap_or_else(|e| panic!("{}: expect.yaml: {e}", dir.display())),
    )
    .unwrap_or_else(|e| panic!("{}: expect.yaml: {e}", dir.display()))
}

/// A case's FORWARD expectations, if it declares any.
fn read_expect_pending(dir: &Path) -> Option<CaseExpect> {
    let path = dir.join("expect-pending.yaml");
    if !path.is_file() {
        return None;
    }
    Some(
        serde_yaml::from_str(
            &std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("{}: expect-pending.yaml: {e}", dir.display())),
        )
        .unwrap_or_else(|e| panic!("{}: expect-pending.yaml: {e}", dir.display())),
    )
}

/// The pinned corpus submodule.
pub fn corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("corpus")
}

/// Every case in the corpus, discovered by walking `cases/`.
///
/// A tree walk, not an `include_str!` of one hardcoded file: adding a case is
/// adding a directory and costs no `.rs` edit (FR-065 Behavior, #267 AC-3).
pub fn load_cases() -> Vec<Case> {
    let root = corpus_root().join("cases");
    assert!(
        root.is_dir(),
        "the corpus submodule is not checked out at {}. Run `git submodule update --init`.",
        root.display()
    );

    let mut cases = Vec::new();
    let mut modes: Vec<PathBuf> = std::fs::read_dir(&root)
        .expect("read cases/")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_dir())
        .collect();
    // Sorted at both levels so a run's order is a property of the data rather
    // than of the filesystem (NFR-006).
    modes.sort();
    for mode in modes {
        let mut dirs: Vec<PathBuf> = std::fs::read_dir(&mode)
            .expect("read mode dir")
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.join("case.yaml").is_file())
            .collect();
        dirs.sort();
        for dir in dirs {
            // Parsed as a VALUE first, then merged, then deserialized. A
            // language SET splits its declaration across two files — the shared
            // `case.yaml` carries no `language`, because that is what varies —
            // so deserializing the shared file alone fails on a required field.
            // `bounds.py` merges the same way; two readers of one corpus
            // disagreeing about what a case IS is the drift FR-065 prevents.
            let shared: serde_yaml::Value = serde_yaml::from_str(
                &std::fs::read_to_string(dir.join("case.yaml")).expect("read case.yaml"),
            )
            .unwrap_or_else(|e| panic!("{}: case.yaml: {e}", dir.display()));

            // Two layouts. A single-language case carries `input/` beside its
            // `case.yaml`; a LANGUAGE SET carries one `<language>/` directory
            // per language, sharing the case-level declaration. #268 authors
            // sixteen modes across three languages, and three sibling
            // directories with unrelated ids is not a set.
            if dir.join("input").is_dir() {
                cases.push(Case {
                    expect: read_expect(&dir),
                    expect_pending: read_expect_pending(&dir),
                    meta: parse_meta(&shared, &dir),
                    dir,
                });
                continue;
            }

            let mut variants: Vec<PathBuf> = std::fs::read_dir(&dir)
                .expect("read case dir")
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.join("input").is_dir())
                .collect();
            variants.sort();
            assert!(
                !variants.is_empty(),
                "{}: neither an `input/` nor any `<language>/input/`. A                  half-authored fixture read as an absent one would make                  `gap_count` mean something else, so this is an error rather                  than a skip.",
                dir.display()
            );
            for variant in variants {
                let language = variant
                    .file_name()
                    .and_then(|n| n.to_str())
                    .expect("language directory name")
                    .to_string();
                let mut merged = shared.clone();
                if variant.join("case.yaml").is_file() {
                    let per: serde_yaml::Value = serde_yaml::from_str(
                        &std::fs::read_to_string(variant.join("case.yaml"))
                            .expect("read case.yaml"),
                    )
                    .unwrap_or_else(|e| panic!("{}: case.yaml: {e}", variant.display()));
                    if let (serde_yaml::Value::Mapping(b), serde_yaml::Value::Mapping(o)) =
                        (&mut merged, &per)
                    {
                        for (k, v) in o {
                            b.insert(k.clone(), v.clone());
                        }
                    }
                }
                let base = merged
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or_else(|| dir.file_name().and_then(|n| n.to_str()).unwrap_or(""))
                    .to_string();
                if merged.get("case").is_none() {
                    set_str(&mut merged, "case", &base);
                }
                // The variant's id is its OWN. One id across three variants is
                // indistinguishable in a pending list, and a duplicate-id check
                // would call them one case.
                set_str(&mut merged, "id", &format!("{base}-{language}"));
                set_str(&mut merged, "language", &language);

                cases.push(Case {
                    expect: read_expect(&variant),
                    expect_pending: read_expect_pending(&variant),
                    meta: parse_meta(&merged, &variant),
                    dir: variant,
                });
            }
        }
    }
    cases
}

/// Run the real coverage path over a case's `input/`, **in place**.
///
/// No tempdir, no copy, no materialisation: the directory the harness reads is
/// the directory an operator reproduces with. The module is the SHARED one the
/// case names under `corpus/modules/`, and the code walk excludes `spec/`
/// exactly as the CLI does — which is what makes a `tests/` topology
/// expressible, since the input tree is now real rather than three hardcoded
/// directories.
pub fn run(case: &Case) -> quire_rs::CoverageReport {
    let input = case.input();
    // The SHARED module the case names, not a per-case copy. Eleven copies of
    // one manifest is how `module:` came to be false for two cases in #266's
    // review: the field named one thing and the file loaded another, because
    // the file was the copy. Resolving through the field makes it load-bearing.
    let module = corpus_root().join("modules").join(&case.meta.module);
    // A module id names either ONE module (`manifest.yaml` directly) or a
    // module PATH — a directory of module directories. `ecosystem` is the
    // second: the real declaration is `spec-artifacts-process` AND
    // `spec-artifacts-iso`, and loading only the first left criteria
    // classification silently producing nothing while totals looked identical
    // (agent-ix/quire-rs#292).
    let registry = if module.join("manifest.yaml").is_file() {
        quire_rs::Registry::load_module(&module).unwrap_or_else(|e| {
            panic!(
                "{}: module `{}` failed to load from {}: {e}",
                case.meta.id,
                case.meta.module,
                module.display()
            )
        })
    } else {
        load_module_path(&module, &case.meta.id)
    };
    let spec = quire_rs::Spec::from_path(&input.join("spec"));
    let model = registry.traceability().cloned().unwrap_or_default();
    let extraction = quire_rs::symbols::extract_tree_scoped(
        &input,
        // `module/` is not excluded: the per-case copies are gone, and the
        // CLI invocation a case documents has no such exclusion either.
        &[Path::new("spec")],
        &model.source_exclude,
    );
    let graph = quire_rs::symbols::trace::bind(&extraction, &model);
    quire_rs::coverage::compute(&spec, &registry, &graph, &input)
        .unwrap_or_else(|e| panic!("{}: compute failed: {e}", case.meta.id))
}

/// Every `quire validate` finding over a case's own spec tree, as one string.
///
/// A second command, because a structural defect is not a coverage one — and
/// running only `coverage` is how a fixture came to assert exactly what the
/// healthy control asserts.
fn validate_report(case: &Case) -> String {
    let input = case.input();
    let module = corpus_root().join("modules").join(&case.meta.module);
    let registry = if module.join("manifest.yaml").is_file() {
        quire_rs::Registry::load_module(&module)
    } else {
        quire_rs::Registry::load_from(&[&module])
    }
    .unwrap_or_else(|e| panic!("{}: module load failed: {e}", case.meta.id));

    let report = quire_rs::validate_bundle_at(
        &input.join("spec"),
        &registry,
        quire_rs::BundlePosture::Strict,
    );
    // Errors AND warnings: a structural defect's severity is the module's
    // decision, and a case asserting the finding should not also have to know
    // which bucket the declaration put it in.
    report
        .errors
        .iter()
        .chain(report.warnings.iter())
        .map(|f| format!("{}: [{}] {}", f.path.display(), f.reason, f.message))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Grade `report` against the case's expectations, collecting every mismatch
/// with the level it belongs to.
///
/// Collects rather than asserting eagerly: a case that loses L1 usually loses
/// L2 and L3 too, and the useful report is the FIRST level lost, which cannot
/// be computed from a panic on the first failed assertion.
pub fn grade(case: &Case, report: &quire_rs::CoverageReport) -> Outcome {
    grade_with(case, report, &case.expect)
}

/// Grade one payload against an ARBITRARY expectation block, so the live
/// contract (`expect.yaml`) and the forward one (`expect-pending.yaml`) are
/// graded by the same code rather than by two that can drift.
pub fn grade_with(case: &Case, report: &quire_rs::CoverageReport, e: &CaseExpect) -> Outcome {
    let mut m: Vec<Mismatch> = Vec::new();
    let mut fail = |level: Level, detail: String| m.push(Mismatch { level, detail });

    if let Some(backed) = e.backed {
        if report.totals.backed != backed {
            fail(
                Level::L1Detected,
                format!("backed: expected {backed}, got {}", report.totals.backed),
            );
        }
    }
    if let Some(total) = e.total {
        if report.totals.total != total {
            fail(
                Level::L1Detected,
                format!("total: expected {total}, got {}", report.totals.total),
            );
        }
    }

    let reasons: Vec<&str> = report
        .diagnostics
        .iter()
        .map(|d| d.reason.as_str())
        .collect();
    for want in &e.diagnostic_reasons {
        if !reasons.contains(&want.as_str()) {
            fail(
                Level::L1Detected,
                format!("expected diagnostic `{want}`, got {reasons:?}"),
            );
        }
    }
    for unwanted in &e.absent_diagnostic_reasons {
        if reasons.contains(&unwanted.as_str()) {
            fail(
                Level::L1Detected,
                format!("`{unwanted}` fired on a case that is not about it: {reasons:?}"),
            );
        }
    }

    for want in &e.binding_census {
        let Some(got) = report
            .binding_census
            .iter()
            .find(|c| c.language == want.language)
        else {
            fail(
                Level::L1Detected,
                format!(
                    "no `{}` census in {:?}",
                    want.language, report.binding_census
                ),
            );
            continue;
        };
        if let Some(candidates) = want.candidates {
            if got.candidates != candidates {
                fail(
                    Level::L1Detected,
                    format!(
                        "{} candidates: expected {candidates}, got {}",
                        got.language, got.candidates
                    ),
                );
            }
        }
        if let Some(bound) = want.bound {
            if got.bound != bound {
                fail(
                    Level::L1Detected,
                    format!(
                        "{} bound: expected {bound}, got {}",
                        got.language, got.bound
                    ),
                );
            }
        }
        // L2: the census names WHERE, and that is the level being claimed.
        if let Some(at) = &want.unbound_example {
            match &got.unbound_example {
                None => fail(
                    Level::L2Localised,
                    format!("{} census carries no unbound example", got.language),
                ),
                Some(example) => {
                    let actual = format!("{}:{}", example.path, example.line);
                    if actual != *at {
                        fail(
                            Level::L2Localised,
                            format!(
                                "{} unbound example: expected {at}, got {actual}",
                                got.language
                            ),
                        );
                    }
                }
            }
        }
    }

    for (reason, want) in &e.diagnostic_paths {
        let actual = report
            .diagnostics
            .iter()
            .find(|d| &d.reason == reason)
            .and_then(|d| d.path.clone());
        if actual.as_deref() != Some(want.as_str()) {
            fail(
                Level::L2Localised,
                format!("{reason} path: expected {want:?}, got {actual:?}"),
            );
        }
    }

    for (reason, fragments) in &e.diagnostic_message_contains {
        let message = report
            .diagnostics
            .iter()
            .find(|d| &d.reason == reason)
            .map(|d| d.message.clone());
        for fragment in fragments {
            match &message {
                Some(text) if text.contains(fragment.as_str()) => {}
                other => fail(
                    Level::L3Actionable,
                    format!("{reason} message lacks {fragment:?}; got {other:?}"),
                ),
            }
        }
    }

    // EXACT, both directions. A subset match would let the id-column case pass
    // on a payload that minted nothing — which is the section case, and telling
    // those two apart is the only thing this field exists for.
    if let Some(want) = &e.unbacked_rows {
        let got: Vec<ExpectUnbackedRow> = report
            .unbacked_rows
            .iter()
            .map(|r| ExpectUnbackedRow {
                document: r.document.clone(),
                row_id: r.row_id.clone(),
                target_ids: r.target_ids.clone(),
            })
            .collect();
        if &got != want {
            fail(
                Level::L2Localised,
                format!("unbacked_rows: expected {want:?}, got {got:?}"),
            );
        }
    }

    if let Some(want) = &e.groups {
        let got: Vec<ExpectGroup> = report
            .groups
            .iter()
            .map(|g| ExpectGroup {
                document: g.document.clone(),
                target: g.target.clone(),
                backed: g.backed,
                total: g.total,
            })
            .collect();
        if &got != want {
            fail(
                Level::L1Detected,
                format!("groups: expected {want:?}, got {got:?}"),
            );
        }
    }

    if let Some(want) = &e.no_symbol_rows {
        let mut got: Vec<String> = report
            .no_symbol_rows
            .iter()
            .filter_map(|r| r.row_id.clone())
            .collect();
        got.sort();
        let mut want = want.clone();
        want.sort();
        if got != want {
            fail(
                Level::L1Detected,
                format!("no_symbol_rows: expected {want:?}, got {got:?}"),
            );
        }
    }

    for want in &e.metrics {
        let Some(metric) = report.metrics.iter().find(|m| m.name == want.name) else {
            fail(
                Level::L1Detected,
                format!("metric `{}` absent from the payload", want.name),
            );
            continue;
        };
        let (state, value, population, examined, matched) = match metric.measurement {
            quire_rs::metric::Measurement::Measured {
                value,
                population,
                examined,
                matched,
            } => (
                "measured",
                Some(value),
                Some(population),
                Some(examined),
                Some(matched),
            ),
            quire_rs::metric::Measurement::NotComputed { .. } => {
                ("not_computed", None, None, None, None)
            }
        };
        if let Some(w) = &want.state {
            if w != state {
                fail(
                    Level::L1Detected,
                    format!("{}.state: expected {w}, got {state}", want.name),
                );
            }
        }
        for (label, expected, actual) in [
            ("value", want.value, value),
            ("population", want.population, population),
            ("examined", want.examined, examined),
            ("matched", want.matched, matched),
        ] {
            if let Some(expected) = expected {
                if actual != Some(expected) {
                    fail(
                        Level::L1Detected,
                        format!("{}.{label}: expected {expected}, got {actual:?}", want.name),
                    );
                }
            }
        }
        if let Some(hollow) = want.hollow {
            if metric.is_hollow() != hollow {
                fail(
                    Level::L1Detected,
                    format!(
                        "{}.hollow: expected {hollow}, got {}",
                        want.name,
                        metric.is_hollow()
                    ),
                );
            }
        }
    }

    if !e.validate_contains.is_empty() || !e.validate_absent.is_empty() {
        let report = validate_report(case);
        for want in &e.validate_contains {
            if !report.contains(want.as_str()) {
                fail(Level::L1Detected, format!("validate output lacks {want:?}"));
            }
        }
        for unwanted in &e.validate_absent {
            if report.contains(unwanted.as_str()) {
                fail(
                    Level::L1Detected,
                    format!("validate reports {unwanted:?} on input that must be clean of it"),
                );
            }
        }
    }

    Outcome {
        case: case.meta.id.clone(),
        issue_ref: case.meta.issue_ref.clone(),
        mismatches: m,
    }
}
