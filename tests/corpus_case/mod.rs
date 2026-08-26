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

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use serde::Deserialize;

mod execution;
mod grading;
mod loading;

pub use execution::run;
pub use grading::{grade, grade_against, grade_with, ValidateSource};
pub use loading::{corpus_root, load_cases};

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
    /// Every level in ladder order. TC-1021 compares this list with the
    /// `corpus.yaml` declaration, and [`grade_against`] rejects a mismatch whose
    /// level is absent here (FR-065-AC-20; historical correction in CR-132).
    pub const ALL: [Level; 3] = [Self::L1Detected, Self::L2Localised, Self::L3Actionable];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::L1Detected => "L1 detected",
            Self::L2Localised => "L2 localised",
            Self::L3Actionable => "L3 actionable",
        }
    }

    /// The declaration token, derived from the rendered label so the harness
    /// carries only one spelling of each level.
    pub fn token(self) -> &'static str {
        let label = self.as_str();
        label.split(' ').next().unwrap_or(label)
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
    /// The failure cases this control is the healthy counterpart of.
    ///
    /// A LIST, always. One control can legitimately serve several failure
    /// cases — the healthy repair of two single-cell defects in one document
    /// is the same document — and measured, `section-name-mismatch-control`
    /// and `id-column-mismatch-control` were byte-identical expectations over
    /// input trees differing by one blank line. Swapping their `control_for`
    /// left every gate green, because neither asserted anything the other did
    /// not.
    #[serde(default)]
    pub control_for: Option<Vec<String>>,
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
    /// Whether anything is expected to fire on this case's input.
    ///
    /// **Not** `#[serde(default)]`, and the corpus declares it in
    /// `case_schema.required`. It was defaulted, and one case had simply
    /// omitted it — `false` arrived from the derive rather than from an author,
    /// and nothing could tell the two apart. A default is how a required field
    /// stops being one (`agent-ix/quire-rs#336`).
    pub findable: bool,
    /// At least one `TC-` id, asserted by TC-1021 since the ladder landed —
    /// which made this required by a gate while every declaration called it
    /// optional. Required here too, so the two agree.
    pub tags: Vec<String>,
    /// The inventory row this fixture claims, when its `id` differs — a
    /// control's id is `<case>-control`, and it covers nothing on its own.
    #[serde(default)]
    pub case: Option<String>,
    /// The ticket a variant binding is sizing (FR-065-CON-3). Required
    /// whenever `module` is not `ecosystem`; `bounds.py` rejects its absence.
    #[serde(default)]
    pub relaxation_ticket: Option<String>,
    /// Why a variant declaration is itself the case's subject (#330).
    /// Mutually exclusive with `relaxation_ticket`.
    #[serde(default)]
    pub declaration_under_test: Option<String>,
    /// The invocation that reproduces this case by hand (FR-065-AC-18).
    /// Modelled rather than ignored: `deny_unknown_fields` is only a gate if
    /// every legitimate field is declared, and an ignored one is a field
    /// nothing checks.
    ///
    /// Required, not defaulted. AC-18 says every case carries one; defaulting
    /// it to `None` meant a case with no reproduction was a case the reader
    /// accepted, and `verify.py` — which reads it — would have died on a
    /// `KeyError` where this one shrugged. Two readers, two behaviours, one
    /// declaration (`case_schema.required`, `agent-ix/quire-rs#336`).
    pub reproduce: String,
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
///
/// `Clone` because FR-065-AC-46 grades a RESTRICTED copy of a block — see
/// [`CaseExpect::restricted_to`].
#[derive(Debug, Default, Deserialize, Clone)]
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
    /// L2. Symbols binding a trace id that no minted row answers for.
    ///
    /// The field that already makes #272's defect observable: rows spread
    /// across headings the declaration cannot reach mint nothing, so the tests
    /// answering for them land here instead — with a path and a line, today.
    #[serde(default)]
    pub untracked_symbols: Option<Vec<ExpectUntracked>>,
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
    /// L1–L3. Suspicions the payload MUST carry (#358).
    ///
    /// `coverage` emits findings on two channels and this corpus could assert
    /// one of them. `suspicions[]` — `vacuous-under-guard`,
    /// `oracle-resembles-implementation` — had no key in either reader, so the
    /// `skeptic` mode family, whose entire subject IS suspicions, could not
    /// name its own subject.
    ///
    /// Measured cost: `cases/skeptic/vacuous-property-suite` is DETECTED, at
    /// `src/lib.rs:7` with a symbol and an evidence string, and its whole
    /// `expect.yaml` was `backed`/`total`/`binding_census` — true of any
    /// healthy three-row tree. That is why it was byte-identical to two other
    /// fixtures and why swapping their input trees left the gate green.
    #[serde(default)]
    pub suspicions: Vec<ExpectSuspicion>,
    /// The control half: `kind` tokens the payload must NOT carry.
    ///
    /// Kinds only, not loci. A control's claim is that the detector stayed
    /// silent, and a locus on an absence names a place nothing was found —
    /// which is not a stronger assertion, only a longer one.
    #[serde(default)]
    pub absent_suspicions: Vec<String>,
}

impl CaseExpect {
    /// This block with every key outside `channels` dropped (FR-065-AC-46).
    ///
    /// The mode-specific witness is graded by RESTRICTION rather than by
    /// inspecting which mismatch fired. Restriction makes the claim exactly
    /// *the witness channel itself discriminates*; a mismatch list would only
    /// say that something fired somewhere, which is the weaker thing AC-42
    /// already asserts.
    ///
    /// The key names are the corpus's, not this file's — they come from
    /// `witness_channels` in `corpus.yaml`, and TC-1043's sibling assertion
    /// requires every name used there to be one this function knows. A channel
    /// this match arm did not list would otherwise be silently dropped from the
    /// restricted block, which would make the rule quietly weaker for exactly
    /// the mode that declared it.
    pub fn restricted_to(&self, channels: &BTreeSet<String>) -> Self {
        let on = |key: &str| channels.contains(key);
        Self {
            backed: on("backed").then_some(self.backed).flatten(),
            total: on("total").then_some(self.total).flatten(),
            diagnostic_reasons: if on("diagnostic_reasons") {
                self.diagnostic_reasons.clone()
            } else {
                Vec::new()
            },
            absent_diagnostic_reasons: if on("absent_diagnostic_reasons") {
                self.absent_diagnostic_reasons.clone()
            } else {
                Vec::new()
            },
            binding_census: if on("binding_census") {
                self.binding_census.clone()
            } else {
                Vec::new()
            },
            diagnostic_paths: if on("diagnostic_paths") {
                self.diagnostic_paths.clone()
            } else {
                BTreeMap::new()
            },
            diagnostic_message_contains: if on("diagnostic_message_contains") {
                self.diagnostic_message_contains.clone()
            } else {
                BTreeMap::new()
            },
            unbacked_rows: on("unbacked_rows")
                .then(|| self.unbacked_rows.clone())
                .flatten(),
            untracked_symbols: on("untracked_symbols")
                .then(|| self.untracked_symbols.clone())
                .flatten(),
            groups: on("groups").then(|| self.groups.clone()).flatten(),
            no_symbol_rows: on("no_symbol_rows")
                .then(|| self.no_symbol_rows.clone())
                .flatten(),
            metrics: if on("metrics") {
                self.metrics.clone()
            } else {
                Vec::new()
            },
            validate_contains: if on("validate_contains") {
                self.validate_contains.clone()
            } else {
                Vec::new()
            },
            validate_absent: if on("validate_absent") {
                self.validate_absent.clone()
            } else {
                Vec::new()
            },
            suspicions: if on("suspicions") {
                self.suspicions.clone()
            } else {
                Vec::new()
            },
            absent_suspicions: if on("absent_suspicions") {
                self.absent_suspicions.clone()
            } else {
                Vec::new()
            },
        }
    }

    /// Every key `restricted_to` can carry. The set TC-1043's sibling holds
    /// `witness_channels` to, so a declared channel this file cannot restrict
    /// on is a hard failure rather than a silent drop.
    pub fn channel_names() -> BTreeSet<String> {
        [
            "backed",
            "total",
            "diagnostic_reasons",
            "absent_diagnostic_reasons",
            "binding_census",
            "diagnostic_paths",
            "diagnostic_message_contains",
            "unbacked_rows",
            "untracked_symbols",
            "groups",
            "no_symbol_rows",
            "metrics",
            "validate_contains",
            "validate_absent",
            "suspicions",
            "absent_suspicions",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }

    /// Whether this block asserts anything at all after restriction.
    pub fn is_empty(&self) -> bool {
        self.backed.is_none()
            && self.total.is_none()
            && self.diagnostic_reasons.is_empty()
            && self.absent_diagnostic_reasons.is_empty()
            && self.binding_census.is_empty()
            && self.diagnostic_paths.is_empty()
            && self.diagnostic_message_contains.is_empty()
            && self.unbacked_rows.is_none()
            && self.untracked_symbols.is_none()
            && self.groups.is_none()
            && self.no_symbol_rows.is_none()
            && self.metrics.is_empty()
            && self.validate_contains.is_empty()
            && self.validate_absent.is_empty()
            && self.suspicions.is_empty()
            && self.absent_suspicions.is_empty()
    }
}

/// One row the declaration minted that no symbol backs.
///
/// Every field is REQUIRED. `row_id` in particular: `row_id: null` is the
/// whole claim the id-column fixture makes — the row minted, and its identity
/// did not — and an optional field would let an author omit exactly that.
#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
#[serde(deny_unknown_fields)]
pub struct ExpectUnbackedRow {
    pub document: String,
    pub row_id: Option<String>,
    pub target_ids: Vec<String>,
}

/// One symbol binding a trace id no minted row answers for.
#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
#[serde(deny_unknown_fields)]
pub struct ExpectUntracked {
    pub symbol: String,
    pub trace_id: String,
    pub path: String,
}

/// One suspicion the payload must carry (#358).
///
/// `kind` is required and the locus fields are not, which is the same rule
/// `diagnostic_reasons` and `diagnostic_paths` split between two keys: naming
/// the kind is L1, and adding `path`/`line`/`symbol` is the fixture author
/// electing to claim L2 as well. Requiring all four would force every skeptic
/// fixture to pin a line number that moves when its input gains a comment.
#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
#[serde(deny_unknown_fields)]
pub struct ExpectSuspicion {
    pub kind: String,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub line: Option<usize>,
    #[serde(default)]
    pub symbol: Option<String>,
    /// Substrings the message must carry — L3, and the reason a suspicion that
    /// names the right place can still be one no reader can act on.
    #[serde(default)]
    pub message_contains: Vec<String>,
}

/// One document's mint count for one declared target kind. All fields
/// required: a partial group is satisfied by the wrong target minting.
#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
#[serde(deny_unknown_fields)]
pub struct ExpectGroup {
    pub document: String,
    pub target: String,
    pub backed: usize,
    pub total: usize,
}

#[derive(Debug, Deserialize, Clone)]
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

#[derive(Debug, Deserialize, Clone)]
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

impl CaseExpect {
    /// Whether this block asserts anything at all.
    ///
    /// A block that grades zero assertions trivially "passes", which for a
    /// forward block means every runner reports that its ticket has landed.
    pub fn asserts_something(&self) -> bool {
        self.backed.is_some()
            || self.total.is_some()
            || !self.diagnostic_reasons.is_empty()
            || !self.absent_diagnostic_reasons.is_empty()
            || !self.binding_census.is_empty()
            || !self.diagnostic_paths.is_empty()
            || !self.diagnostic_message_contains.is_empty()
            || self.no_symbol_rows.is_some()
            || !self.metrics.is_empty()
            || !self.validate_contains.is_empty()
            || !self.validate_absent.is_empty()
            || self.unbacked_rows.is_some()
            || self.groups.is_some()
            || self.untracked_symbols.is_some()
            || !self.suspicions.is_empty()
            || !self.absent_suspicions.is_empty()
    }
}
