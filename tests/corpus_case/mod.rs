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
#[derive(Debug, Deserialize)]
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
}

/// What the emitted envelope must say.
///
/// Every field is optional: a case asserts the facts it is about and stays
/// silent on the rest, so an unrelated engine change does not fail forty cases
/// that were never about it (FR-065-AC-5).
#[derive(Debug, Default, Deserialize)]
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
    /// L3. A substring each diagnostic's message must carry, `reason` -> text.
    ///
    /// A finding can carry a correct path while its prose names nothing a
    /// reader can act on. Measured: removing the example criterion from
    /// `catch-all-universal`'s message left every path assertion passing.
    #[serde(default)]
    pub diagnostic_message_contains: BTreeMap<String, String>,
    /// L1. Row ids explained as verified by a method that mints no source
    /// symbol (#259). Asserted by id, not by count: a count is satisfied by
    /// exempting the wrong row.
    #[serde(default)]
    pub no_symbol_rows: Option<Vec<String>>,
    /// L1. Per-metric expectations, keyed on the metric name.
    #[serde(default)]
    pub metrics: Vec<ExpectMetric>,
}

#[derive(Debug, Deserialize)]
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
    pub expect: CaseExpect,
}

impl Case {
    pub fn input(&self) -> PathBuf {
        self.dir.join("input")
    }
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
            let meta: CaseMeta = serde_yaml::from_str(
                &std::fs::read_to_string(dir.join("case.yaml")).expect("read case.yaml"),
            )
            .unwrap_or_else(|e| panic!("{}: case.yaml: {e}", dir.display()));
            let expect: CaseExpect = serde_yaml::from_str(
                &std::fs::read_to_string(dir.join("expect.yaml")).expect("read expect.yaml"),
            )
            .unwrap_or_else(|e| panic!("{}: expect.yaml: {e}", dir.display()));
            cases.push(Case { dir, meta, expect });
        }
    }
    cases
}

/// Run the real coverage path over a case's `input/`, **in place**.
///
/// No tempdir, no copy, no materialisation: the directory the harness reads is
/// the directory an operator reproduces with. The module is the case's own
/// `input/module`, and the code walk excludes `spec/` exactly as the CLI does
/// — which is what makes a `tests/` topology expressible, since the input tree
/// is now real rather than three hardcoded directories.
pub fn run(case: &Case) -> quire_rs::CoverageReport {
    let input = case.input();
    // The SHARED module the case names, not a per-case copy. Eleven copies of
    // one manifest is how `module:` came to be false for two cases in #266's
    // review: the field named one thing and the file loaded another, because
    // the file was the copy. Resolving through the field makes it load-bearing.
    let module = corpus_root().join("modules").join(&case.meta.module);
    let registry = quire_rs::Registry::load_module(&module).unwrap_or_else(|e| {
        panic!(
            "{}: module `{}` failed to load from {}: {e}",
            case.meta.id,
            case.meta.module,
            module.display()
        )
    });
    let spec = quire_rs::Spec::from_path(&input.join("spec"));
    let model = registry.traceability().cloned().unwrap_or_default();
    let extraction = quire_rs::symbols::extract_tree_scoped(
        &input,
        &[Path::new("spec"), Path::new("module")],
        &model.source_exclude,
    );
    let graph = quire_rs::symbols::trace::bind(&extraction, &model);
    quire_rs::coverage::compute(&spec, &registry, &graph, &input)
        .unwrap_or_else(|e| panic!("{}: compute failed: {e}", case.meta.id))
}

/// Grade `report` against the case's expectations, collecting every mismatch
/// with the level it belongs to.
///
/// Collects rather than asserting eagerly: a case that loses L1 usually loses
/// L2 and L3 too, and the useful report is the FIRST level lost, which cannot
/// be computed from a panic on the first failed assertion.
pub fn grade(case: &Case, report: &quire_rs::CoverageReport) -> Outcome {
    let e = &case.expect;
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

    for (reason, fragment) in &e.diagnostic_message_contains {
        let message = report
            .diagnostics
            .iter()
            .find(|d| &d.reason == reason)
            .map(|d| d.message.clone());
        match message {
            Some(text) if text.contains(fragment.as_str()) => {}
            other => fail(
                Level::L3Actionable,
                format!("{reason} message lacks {fragment:?}; got {other:?}"),
            ),
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

    Outcome {
        case: case.meta.id.clone(),
        issue_ref: case.meta.issue_ref.clone(),
        mismatches: m,
    }
}
