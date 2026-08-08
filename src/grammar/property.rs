//! Acceptance-criteria **property-shape classification** (FR-052) — a second,
//! orthogonal axis over the same criteria the [`ac`](super::ac) grammar binds
//! to.
//!
//! Where `ac` asks *is this criterion well-formed* and answers with findings,
//! this module asks *does this criterion quantify over an input domain, and if
//! so which clauses carry its generator domain, its precondition and its
//! oracle*. The answer is **metadata, never a verdict**: a criterion that is
//! not property-shaped is not a defect (a specific scenario verified by
//! Inspection is a legitimate criterion), and a pass/fail gate on
//! property-shapedness would invite an author to reword a criterion to satisfy
//! a checker rather than to describe the system.
//!
//! **FR-052-CON-1 is structural, not conventional.** This module does not
//! import [`GrammarFinding`](super::GrammarFinding) and its return types cannot
//! express one, so there is no severity key, no promotion path and `--strict`
//! immunity holds by construction. `scripts/check_property_purity.sh` (wired
//! into `make ci`) fails the build if the name ever appears here, which makes
//! adding a finding a one-line reviewable change rather than a silent drift.
//!
//! Detection reads the CR-017 masked copy of the statement — a backticked
//! phrase is a *mention*, not a use — while spans are emitted against the
//! unmasked original. [`mask_code_spans`](super::ac::mask_code_spans) is
//! byte-length preserving, the same trick `outcome_clause` relies on.
//!
//! # Quantification is read at two positions (agent-ix/quire-rs#45, CR-029)
//!
//! The 2026-08-07 corpus sweep measured `extractable` at ~90% precision and
//! ~30% recall, and attributed the loss to the reach of one regex: the
//! universal determiner was anchored at `^\W*`, so a conditional universal
//! (`When a command exceeds its timeout, …`) fired nothing. Three candidate
//! widenings were built behind measurement-only cargo features and measured
//! factorially over one frozen snapshot of 196 repos / 3,304 documents /
//! 13,801 binding criteria
//! (`~/dev/reports/2026-08-07-ac-property-recall-experiment.md`), against a gate
//! fixed in advance: **≥85% precision on each rule's own newly-caught cells**,
//! n=30 seeded.
//!
//! **One passed and is now ordinary classifier behaviour**, the other two were
//! deleted rather than narrowed:
//!
//! | Rule | Newly-caught cells | Precision (n=30) | Outcome |
//! |---|---|---|---|
//! | `recall-subject-determiner` | 628 | **30/30 = 100%** | **ships**, unconditional |
//! | `recall-shape-axis` | 1,006 | 21/30 = 70.0% | deleted |
//! | `recall-generic-subject` | 4,139 | 17/30 = 56.7% | deleted |
//!
//! [`recall_subject_determiner`] keeps the shipped rule's closed determiner set
//! and widens only the *position*, to the two subject heads a fronted clause
//! creates. It retains its `recall:subject-determiner:` signal ids: attribution
//! stays useful after adoption, because a corpus census can still separate the
//! cells this rule reaches from the ones the sentence-initial anchor already
//! reached.
//!
//! A rejected rule is **deleted, not narrowed and not left feature-gated**.
//! Narrowing a rule until its number looks acceptable is the failure that
//! retired `no-observable-outcome` (CR-014), and a dormant feature is a rule
//! nobody measures again. A narrower variant — a GWT-only shape rule, say — is a
//! new question with its own experiment, not a rescue of this one.

use std::sync::OnceLock;

use regex::Regex;
use serde::{Deserialize, Serialize};

use super::ac::{self, AcShape};
use super::GrammarVocabularies;
use crate::ast::QuireDocument;

/// The property shape of one acceptance criterion (FR-052).
///
/// The enum is **closed** (FR-052-CON-3): extension happens by declaring idiom
/// phrases, never by adding a variant, because a shape with no downstream
/// generator silently drops criteria. There is deliberately no `SchemaShaped`
/// variant — its signal would be repo state (*"an OpenAPI schema exists"*)
/// rather than statement text, which would break determinism (NFR-006) and
/// make two corpora incomparable.
///
/// **Declaration order is the fixed precedence** of FR-052, first match wins,
/// so the derived [`Ord`] *is* the precedence relation and structure outranks
/// quantification: a universally quantified round-trip criterion classifies
/// [`RoundTrip`](PropertyShape::RoundTrip).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PropertyShape {
    /// Two operations composed, with an identity back-reference in the outcome.
    RoundTrip,
    /// Applying an operation a second time yields the first result.
    Idempotence,
    /// An ordering or stable-sort property over a collection.
    Ordering,
    /// A property asserted to hold unconditionally.
    Invariant,
    /// A named failure over a class of bad inputs.
    ErrorCase,
    /// A quantification over an ordered sequence of state transitions.
    Lifecycle,
    /// A quantification over interleavings rather than over values.
    Concurrency,
    /// Universally quantified over a generator domain by its determiner.
    Universal,
    /// One specific scenario, not quantified. A **first-class outcome**, never
    /// a defect.
    Example,
    /// No signal fired. Also a first-class outcome.
    Unclassified,
}

impl PropertyShape {
    /// Stable machine-readable label, carried on every surface.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RoundTrip => "round-trip",
            Self::Idempotence => "idempotence",
            Self::Ordering => "ordering",
            Self::Invariant => "invariant",
            Self::ErrorCase => "error-case",
            Self::Lifecycle => "lifecycle",
            Self::Concurrency => "concurrency",
            Self::Universal => "universal",
            Self::Example => "example",
            Self::Unclassified => "unclassified",
        }
    }

    /// True when a criterion of this shape quantifies over an input domain at
    /// all — the shape half of [`AcClassification::extractable`].
    ///
    /// `Example` and `Unclassified` are the only shapes that describe a single
    /// point rather than a domain. Neither is a defect.
    pub fn is_quantified(self) -> bool {
        !matches!(self, Self::Example | Self::Unclassified)
    }

    /// The `idiom:<shape>` signal id, as a `'static` string per variant so the
    /// audit trail allocates nothing.
    fn idiom_signal(self) -> &'static str {
        match self {
            Self::RoundTrip => "idiom:round-trip",
            Self::Idempotence => "idiom:idempotence",
            Self::Ordering => "idiom:ordering",
            Self::Invariant => "idiom:invariant",
            Self::ErrorCase => "idiom:error-case",
            Self::Lifecycle => "idiom:lifecycle",
            Self::Concurrency => "idiom:concurrency",
            Self::Universal => "idiom:universal",
            Self::Example => "idiom:example",
            Self::Unclassified => "idiom:unclassified",
        }
    }

    /// Every variant, in precedence order. The closed enum's own iteration
    /// source — no map, so no iteration order leaks into output (NFR-006).
    const ALL: &'static [Self] = &[
        Self::RoundTrip,
        Self::Idempotence,
        Self::Ordering,
        Self::Invariant,
        Self::ErrorCase,
        Self::Lifecycle,
        Self::Concurrency,
        Self::Universal,
        Self::Example,
        Self::Unclassified,
    ];
}

/// One clause of a decomposed criterion: a byte range into the **untruncated**
/// statement plus the text it covers.
///
/// Both, deliberately. Offsets so the engine can assert the spans of one record
/// are in bounds, non-overlapping and ascending and a viewer can render
/// highlights; text because a UTF-16 consumer handed raw UTF-8 byte offsets is
/// a defect generator. The redundancy is checkable, and TC-784 checks it.
///
/// Offsets are **statement-relative, never file-relative**: a supplement
/// statement is synthesised by joining wrapped source lines, so an offset does
/// not index the file. Only [`AcClassification::line`] is file-relative.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub text: String,
}

/// The three-way decomposition of a universally quantified criterion: what a
/// generator produces, which of those values are in scope, and what must hold
/// of them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spans {
    /// The quantified noun phrase after the determiner.
    pub domain: Span,
    /// The restrictive filter clause, when the criterion carries one.
    pub precondition: Option<Span>,
    /// The predicate and outcome.
    pub oracle: Span,
}

/// One criterion's property classification (FR-052). Emitted by
/// [`classify_document`]; never routed into a validation result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcClassification {
    /// The criterion's own id (`FR-047-AC-3`), from the `ID` column or the
    /// supplement heading. Without it a downstream generator cannot emit the
    /// tracking tag coverage reconciliation keys on.
    pub row_id: Option<String>,
    /// The **untruncated** statement. `GrammarFinding.statement` is excerpted
    /// at 160 chars, so classification cannot ride on a finding-shaped record;
    /// every span offset indexes this field.
    pub statement: String,
    /// 1-based, file-relative line of the criterion.
    pub line: Option<usize>,
    /// The FR-047 shape axis, carried through unchanged.
    pub shape: AcShape,
    /// The FR-052 shape axis.
    pub property: PropertyShape,
    /// Whether a generator can extract a property from this criterion.
    ///
    /// Derived in **exactly one place** (see [`classify_property`]) so the
    /// coverage column, any summary line and any downstream consumer cannot
    /// disagree about the ratio — and derived from the *structural* shape
    /// only, never from a module-declared idiom (FR-052-CON-4).
    pub extractable: bool,
    pub domain: Option<Span>,
    pub precondition: Option<Span>,
    pub oracle: Option<Span>,
    /// Stable ids of the signals that fired, in evaluation order.
    ///
    /// There is no confidence float: a float is unreviewable and invites
    /// threshold tuning, the failure mode this program has already hit.
    /// `Unclassified` is the explicit outcome instead, and this trail is what
    /// makes every record re-derivable from its own output.
    pub signals: Vec<&'static str>,
}

/// The outcome of classifying one statement — everything
/// [`AcClassification`] carries that does not come from the statement's
/// position in the document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Classified {
    pub property: PropertyShape,
    pub extractable: bool,
    pub spans: Option<Spans>,
    pub signals: Vec<&'static str>,
}

/// Classify one criterion's property shape (FR-052).
///
/// The order is fixed and total, first match wins:
///
/// 1. the **closed structural signals** — `RoundTrip`, `Idempotence`,
///    `Ordering`, `Invariant` — which are engine-owned and need no registry;
/// 2. universal quantification by determiner;
/// 3. `Example` for a predicated single scenario, `Unclassified` for a cell
///    that fired nothing at all;
/// 4. the module idiom registry as a **booster**: it may sharpen the label to
///    a higher-precedence shape, and it may catch a residual `Example` — but
///    it can never change `extractable`, which is derived from the structural
///    shape alone (FR-052-CON-4). A missed idiom therefore degrades a label to
///    a less specific shape; it never removes a criterion from extraction.
pub fn classify_property(statement: &str, idioms: &PropertyIdioms) -> Classified {
    let masked = ac::mask_code_spans(statement);
    let markers = predicate_markers(&masked);
    let mut signals: Vec<&'static str> = Vec::new();

    let quantified = quantification(&masked);
    let structural = match (structural_shape(&masked, &mut signals), quantified) {
        (Some(shape), _) => shape,
        (None, Some(q)) => {
            signals.push(q.signal);
            PropertyShape::Universal
        }
        (None, None) if !markers.is_empty() => {
            signals.push("example:specific-scenario");
            PropertyShape::Example
        }
        (None, None) => {
            signals.push("unclassified:no-signal");
            PropertyShape::Unclassified
        }
    };

    // `extractable` is derived here and nowhere else, from the *structural*
    // shape and the presence of an oracle — a predicate marker of any strength.
    // Neither input can be influenced by a module declaration, which is what
    // FR-052-CON-4 requires and TC-790 pins.
    let extractable = structural.is_quantified() && !markers.is_empty();

    // The registry is applied last: it refines the label of a criterion the
    // structure already caught, and catches the residual. Precedence still
    // decides, so a declared `Ordering` idiom cannot outrank a structural
    // `RoundTrip`.
    let property = match idioms.shape_of(&masked) {
        Some(declared) => {
            signals.push(declared.idiom_signal());
            structural.min(declared)
        }
        None => structural,
    };

    // Only a universally quantified criterion is decomposed. The metamorphic
    // shapes carry no spans in v1, and decomposition keys on the *structural*
    // shape so the registry cannot add or remove spans either.
    let spans = if structural == PropertyShape::Universal {
        quantified
            .and_then(|q| decompose(statement, &masked, q.domain_start, &markers, &mut signals))
    } else {
        None
    };

    Classified {
        property,
        extractable,
        spans,
        signals,
    }
}

/// Classify every binding criterion of `doc` (FR-052-AC-9). The criteria are
/// exactly the ones [`ac::check`] reads — both call [`ac::statements`] — so a
/// `US` or `IT` document yields no records, as it yields no findings.
pub fn classify_document(
    archetype: &str,
    doc: &QuireDocument,
    line_offset: usize,
    vocab: GrammarVocabularies<'_>,
) -> Vec<AcClassification> {
    ac::statements(archetype, doc, line_offset)
        .into_iter()
        .map(|stmt| {
            // The FR-047 axis is carried through on the record unchanged; the
            // FR-052 axis is computed independently from the statement text.
            let shape = ac::classify(&stmt.text, vocab);
            let classified = classify_property(&stmt.text, vocab.idioms);
            let (domain, precondition, oracle) = match classified.spans {
                Some(s) => (Some(s.domain), s.precondition, Some(s.oracle)),
                None => (None, None, None),
            };
            AcClassification {
                row_id: stmt.row_id,
                shape,
                statement: stmt.text,
                line: stmt.line,
                property: classified.property,
                extractable: classified.extractable,
                domain,
                precondition,
                oracle,
                signals: classified.signals,
            }
        })
        .collect()
}

// ─── Closed structural signals ──────────────────────────────────────────────

/// The four metamorphic shapes reachable from **engine-owned closed signal
/// sets** alone (FR-052), in precedence order. A repo declaring no registry
/// still earns every one of them, which is what keeps the open idiom space
/// from becoming load-bearing (CR-014's rule).
fn structural_shape(masked: &str, signals: &mut Vec<&'static str>) -> Option<PropertyShape> {
    // RoundTrip — a composition of two operations *plus* an identity
    // back-reference in the outcome. The back-reference set is closed English,
    // not domain vocabulary, which is what makes the genuinely open verb-pair
    // space (`parse`/`serialize`, `encode`/`decode`, …) unnecessary here.
    if re_composition().is_match(masked) && re_identity_back_reference().is_match(masked) {
        signals.push("round-trip:composition");
        signals.push("round-trip:identity-back-reference");
        return Some(PropertyShape::RoundTrip);
    }
    // Idempotence — a repetition adverb together with an equality verb or
    // `the same`. Both sets are closed: quantity adverbs and equality verbs.
    if re_repetition().is_match(masked) && re_equality().is_match(masked) {
        signals.push("idempotence:repetition");
        signals.push("idempotence:equality");
        return Some(PropertyShape::Idempotence);
    }
    // Ordering — a small ordering lexicon. A bare `deterministic`, `before` or
    // `order` is deliberately absent: each swallows a large slice of `Example`.
    if re_ordering().is_match(masked) {
        signals.push("ordering:lexicon");
        return Some(PropertyShape::Ordering);
    }
    // Invariant — English function words asserting unconditionality.
    if re_invariant().is_match(masked) {
        signals.push("invariant:absolute");
        return Some(PropertyShape::Invariant);
    }
    None
}

// ─── Quantification detection ───────────────────────────────────────────────

/// A criterion found to quantify over a domain: **where the domain begins** and
/// **which rule found it**.
///
/// One type for both determiner positions, so a rule can only ever move the
/// domain boundary and name itself — it cannot reach `extractable`, the idiom
/// registry, or the span logic by any other route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Quantification {
    /// Byte offset into the masked statement at which the quantified domain
    /// starts. Byte-equal in the original — the mask is length-preserving.
    domain_start: usize,
    /// The stable signal id recorded on the classification record.
    signal: &'static str,
}

/// Does this criterion quantify over a domain, and where does that domain open?
///
/// A universal determiner in **sentence-initial** determiner position is read
/// first and always; [`recall_subject_determiner`] is read only when that
/// declined. Exactly one rule ever fires, so the `signals` trail attributes the
/// classification without re-running the engine.
fn quantification(masked: &str) -> Option<Quantification> {
    if let Some(m) = re_universal_determiner().find(masked) {
        return Some(Quantification {
            domain_start: m.end(),
            signal: "universal:determiner",
        });
    }
    recall_subject_determiner(masked)
}

/// The determiner at the **subject**, not at offset 0.
///
/// The shipped rule anchors with `^\W*`, so a criterion that fronts its
/// condition (`When a command exceeds its timeout, the result carries …`)
/// leaves its quantifier unreachable. This rule keeps the same closed
/// determiner set and widens only the *position*, to the two heads a fronted
/// clause creates:
///
/// - the **subordinate clause's own subject**, immediately after a
///   sentence-initial subordinator (`When` **`a command`** `exceeds …`);
/// - the **main clause's subject**, immediately after the fronting comma
///   (`In strict mode,` **`every finding`** `…`).
///
/// It is emphatically **not "determiner anywhere"**. Both positions are pinned
/// to a function word — a sentence-initial subordinator or preposition, or the
/// comma that closes the material it fronts — so `returns a diagnostic` cannot
/// match: `a diagnostic` is the outcome, not the domain, and nothing anchors it
/// to a subject. Where the subject position cannot be bounded — a fronted
/// phrase with no comma, a determiner buried in a trailing prepositional phrase
/// — the rule **refuses** rather than guessing, the same discipline the
/// `span:refused-weak-boundary` guard already applies to clause boundaries.
///
/// Measured over the 2026-08-07 snapshot it newly caught 628 criteria at 30/30
/// precision (90% under the strictest reading of three degenerate
/// preconditions), which is why it is unconditional rather than gated (CR-029).
fn recall_subject_determiner(masked: &str) -> Option<Quantification> {
    if let Some(m) = re_fronted_subject_determiner().find(masked) {
        return Some(Quantification {
            domain_start: m.end(),
            signal: "recall:subject-determiner:fronted-subject",
        });
    }
    let m = re_main_subject_determiner().find(masked)?;
    Some(Quantification {
        domain_start: m.end(),
        signal: "recall:subject-determiner:main-subject",
    })
}

/// A universal determiner at the head of a **fronted subordinate clause's**
/// subject: `When` **`a command`** `exceeds its timeout, …`.
fn re_fronted_subject_determiner() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(
            r"(?i)^\W*(given|when|if|while|where|unless|once|after|before)\s+(a|an|any|every|each|all|no)\s+",
        )
        .expect("fronted-subject-determiner regex")
    })
}

/// A universal determiner at the head of the **main clause's** subject, after a
/// fronted subordinate clause or prepositional phrase closed by a comma:
/// `In strict mode,` **`every finding`** `…`.
///
/// The fronted material is bounded at the *first* comma (`[^,]*`) — the
/// conservative choice, since a fronted clause carrying its own comma cannot be
/// delimited without parsing, and the rule refuses there rather than guessing.
fn re_main_subject_determiner() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(
            r"(?i)^\W*(given|when|if|while|where|unless|once|after|before|in|for|following|during|upon)\b[^,]*,\s*(a|an|any|every|each|all|no)\s+",
        )
        .expect("main-subject-determiner regex")
    })
}

// ─── Decomposition ──────────────────────────────────────────────────────────

/// How confidently a predicate marker identifies a verb.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Strength {
    /// A modal, a copula or an irregular past form — unambiguous.
    Strong,
    /// An `-s`/`-ed`/`-ing` inflection, which cannot tell a verb from a plural
    /// noun (CR-019 measured 23.4% of cells satisfying the predicate test
    /// through this branch alone).
    Weak,
}

/// One predicate marker: `(start, end, strength)` into the masked statement.
type Marker = (usize, usize, Strength);

/// Every predicate marker in `masked`, ascending by start.
///
/// No new vocabulary: the Strong markers are `ac`'s existing
/// [`re_modal_or_copula`](ac::re_modal_or_copula) and
/// [`re_irregular_past`](ac::re_irregular_past), the Weak ones its
/// [`re_inflected_verb`](ac::re_inflected_verb).
///
/// A Weak marker in **modifier position** — directly after an article,
/// determiner or preposition (`the merged map`, `to warning`) — is dropped: it
/// is an attributive participle or a plural noun inside a noun phrase, not the
/// clause's predicate. This is a function-word guard, not part-of-speech
/// tagging; it reads the same closed English word classes
/// [`re_elided_copula`](ac) already relies on.
fn predicate_markers(masked: &str) -> Vec<Marker> {
    let mut out: Vec<Marker> = Vec::new();
    for m in ac::re_modal_or_copula().find_iter(masked) {
        out.push((m.start(), m.end(), Strength::Strong));
    }
    for m in ac::re_irregular_past().find_iter(masked) {
        out.push((m.start(), m.end(), Strength::Strong));
    }
    for m in ac::re_inflected_verb().find_iter(masked) {
        if out.iter().any(|(start, _, _)| *start == m.start()) {
            continue;
        }
        if re_modifier_context().is_match(&masked[..m.start()]) {
            continue;
        }
        out.push((m.start(), m.end(), Strength::Weak));
    }
    out.sort_by_key(|(start, _, _)| *start);
    out.dedup_by_key(|(start, _, _)| *start);
    out
}

/// The first predicate marker starting at or after `from`.
fn next_predicate_marker(markers: &[Marker], from: usize) -> Option<Marker> {
    markers.iter().copied().find(|(start, _, _)| *start >= from)
}

/// Decompose a universally quantified criterion into `{domain, precondition,
/// oracle}` (FR-052). `determiner_end` is the byte offset just past the
/// quantifying determiner.
///
/// Spans are emitted against the **original** statement while every boundary is
/// found in the masked copy, so a backticked phrase never decides a boundary.
///
/// Returns `None` — deliberately, and without changing the criterion's shape or
/// its `extractable` value — when the statement carries **no Strong predicate
/// marker at all**. The clause split then rests entirely on the
/// `-s`/`-ed`/`-ing` marker, which cannot distinguish a verb from a plural
/// noun, and a confidently wrong span is worse for a generator than an absent
/// one.
fn decompose(
    statement: &str,
    masked: &str,
    determiner_end: usize,
    markers: &[Marker],
    signals: &mut Vec<&'static str>,
) -> Option<Spans> {
    if !markers
        .iter()
        .any(|(_, _, strength)| *strength == Strength::Strong)
    {
        signals.push("span:refused-weak-boundary");
        return None;
    }

    // The restrictive filter clause, when there is one, opens the precondition.
    let filter = re_filter_marker()
        .find_iter(masked)
        .find(|m| m.start() >= determiner_end);
    // The oracle opens at the first predicate marker outside the precondition:
    // past the relative clause's own verb when a filter clause is present, and
    // past the determiner otherwise.
    let search_from = match filter {
        Some(f) => next_predicate_marker(markers, f.end()).map_or(f.end(), |(_, end, _)| end),
        None => determiner_end,
    };
    let (oracle_start, _, _) = next_predicate_marker(markers, search_from)?;

    let domain_end = filter.map_or(oracle_start, |f| f.start());
    let domain = span_of(statement, determiner_end, domain_end)?;
    let precondition = match filter {
        Some(f) => Some(span_of(statement, f.start(), oracle_start)?),
        None => None,
    };
    let oracle = span_of(statement, oracle_start, statement.len())?;

    signals.push("span:domain");
    if precondition.is_some() {
        signals.push("span:precondition");
    }
    signals.push("span:oracle");
    Some(Spans {
        domain,
        precondition,
        oracle,
    })
}

/// The span covering `statement[start..end]` with surrounding whitespace and
/// separators trimmed off, or `None` when the range is empty, out of bounds, or
/// not on a character boundary.
///
/// The mask replaces each character with as many `x` bytes as it occupied, so a
/// boundary found in the mask can land inside a multi-byte character of the
/// original. Refusing the span there is the honest outcome; panicking is not.
fn span_of(statement: &str, start: usize, end: usize) -> Option<Span> {
    let slice = statement.get(start..end)?;
    let trim = |c: char| c.is_whitespace() || c == ',' || c == ';';
    let lead = slice.len() - slice.trim_start_matches(trim).len();
    let tail = slice.len() - slice.trim_end_matches(trim).len();
    let (start, end) = (start + lead, end - tail);
    let text = statement.get(start..end)?;
    if text.is_empty() {
        return None;
    }
    Some(Span {
        start,
        end,
        text: text.to_string(),
    })
}

// ─── The module idiom registry (a booster, never a prerequisite) ─────────────

/// The engine's built-in idiom phrases.
///
/// They cover exactly the three shapes with **no closed structural signal** —
/// `ErrorCase`, `Lifecycle`, `Concurrency`. The four metamorphic shapes are
/// reachable from structure alone and need no phrase list; a module may still
/// declare phrases for them to sharpen a label.
const BUILTIN_PROPERTY_IDIOMS: &[(&str, PropertyShape)] = &[
    ("malformed input", PropertyShape::ErrorCase),
    ("invalid input", PropertyShape::ErrorCase),
    ("rejects with", PropertyShape::ErrorCase),
    ("raises an error", PropertyShape::ErrorCase),
    ("returns an error", PropertyShape::ErrorCase),
    ("state transition", PropertyShape::Lifecycle),
    ("transitions from", PropertyShape::Lifecycle),
    ("lifecycle", PropertyShape::Lifecycle),
    ("in sequence", PropertyShape::Lifecycle),
    ("concurrently", PropertyShape::Concurrency),
    ("in parallel", PropertyShape::Concurrency),
    ("interleaving", PropertyShape::Concurrency),
    ("simultaneously", PropertyShape::Concurrency),
];

/// A precompiled matcher over the property idiom vocabulary (FR-052): the
/// engine built-ins with any module-declared `property_idioms` registry merged
/// **first-wins over** them, so a module redeclaring a built-in phrase under a
/// different shape wins for that phrase.
///
/// One alternation `Regex` per shape, held in the closed enum's own precedence
/// order — never a map iteration, which would leak iteration order into output
/// (NFR-006). Multi-word phrases tolerate any run of whitespace, as
/// [`VacuousPredicates`](super::VacuousPredicates) does.
#[derive(Debug)]
pub struct PropertyIdioms {
    /// Indexed by [`PropertyShape::ALL`] position.
    by_shape: Vec<Option<Regex>>,
}

impl Default for PropertyIdioms {
    /// The built-in idioms alone, applied when no module declares the registry.
    fn default() -> Self {
        Self::with_module_idioms(std::iter::empty())
    }
}

impl PropertyIdioms {
    /// Build the matcher from the built-ins with `module_idioms` layered over
    /// them first-wins by phrase.
    pub fn with_module_idioms<'a, I: IntoIterator<Item = (&'a str, PropertyShape)>>(
        module_idioms: I,
    ) -> Self {
        // A module phrase replaces the built-in of the same name; the merge is
        // by phrase, so the built-ins survive wherever the module is silent.
        let mut merged: std::collections::BTreeMap<String, PropertyShape> = BUILTIN_PROPERTY_IDIOMS
            .iter()
            .map(|(phrase, shape)| (phrase.trim().to_ascii_lowercase(), *shape))
            .collect();
        for (phrase, shape) in module_idioms {
            let phrase = phrase.trim().to_ascii_lowercase();
            if phrase.is_empty() {
                continue;
            }
            merged.insert(phrase, shape);
        }

        let by_shape = PropertyShape::ALL
            .iter()
            .map(|shape| {
                let forms: Vec<String> = merged
                    .iter()
                    .filter(|(_, s)| *s == shape)
                    .map(|(phrase, _)| {
                        let words: Vec<String> =
                            phrase.split_whitespace().map(regex::escape).collect();
                        words.join(r"\s+")
                    })
                    .collect();
                if forms.is_empty() {
                    None
                } else {
                    Regex::new(&format!(r"(?i)\b({})\b", forms.join("|"))).ok()
                }
            })
            .collect();
        Self { by_shape }
    }

    /// The highest-precedence declared shape whose phrase appears in `masked`.
    fn shape_of(&self, masked: &str) -> Option<PropertyShape> {
        PropertyShape::ALL
            .iter()
            .zip(self.by_shape.iter())
            .find(|(_, matcher)| matcher.as_ref().is_some_and(|r| r.is_match(masked)))
            .map(|(shape, _)| *shape)
    }
}

/// The shared built-in idiom vocabulary for the registry-free paths.
pub fn default_property_idioms() -> &'static PropertyIdioms {
    static DEFAULT: OnceLock<PropertyIdioms> = OnceLock::new();
    DEFAULT.get_or_init(PropertyIdioms::default)
}

// ─── Compile-once regexes ───────────────────────────────────────────────────

/// A composition of two operations, or an explicit round-trip verb.
fn re_composition() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"(?i)(\bthen\b|\bfollowed\s+by\b|\bround[- ]?trips?\b|\bround[- ]?tripped\b)")
            .expect("composition regex")
    })
}

/// The closed identity back-reference set: the outcome names the input again.
fn re_identity_back_reference() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(
            r"(?i)((yields|returns|produces|gives|reproduces)\s+the\s+(input|original|source)|\bbyte-identical\b|equals\s+the\s+(input|original|source)|identical\s+to\s+the\s+(input|original|source)|\blosslessly\b|\bround[- ]?trips?\b)",
        )
        .expect("identity-back-reference regex")
    })
}

/// A repetition adverb — closed quantity vocabulary.
fn re_repetition() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"(?i)(\btwice\b|\bagain\b|a\s+second\s+time\b|\brepeatedly\b|\bre-?run\b|\bre-?applied\b)")
            .expect("repetition regex")
    })
}

/// An equality verb, or `the same`.
fn re_equality() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"(?i)(\b(yields|produces|returns|leaves|equals|matches)\b|the\s+same\b)")
            .expect("equality regex")
    })
}

/// The ordering lexicon. A bare `deterministic`, `before` or `order` is
/// **deliberately absent** — each swallows a large slice of `Example`.
fn re_ordering() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(
            r"(?i)(\bsorted\b|\bsort\s+order\b|\bprecedes\b|\bstable\s+order\b|\bfirst-wins\b|\bin\s+(corpus|path|input|document|declaration)\s+order\b|\border\s+is\s+preserved\b)",
        )
        .expect("ordering regex")
    })
}

/// Unconditionality — closed English function words.
fn re_invariant() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(
            r"(?i)(\balways\b|\bnever\b|\binvariant\b|\bregardless\s+of\b|\bunder\s+any\b|\bfor\s+any\b|\bin\s+every\s+case\b|\bno\s+matter\b)",
        )
        .expect("invariant regex")
    })
}

/// A universal quantifier in determiner position. The leading `\W*` absorbs a
/// bolded or parenthetical prefix; `no` is included because negative existence
/// is universally quantified (the CR-019 elided-copula class).
fn re_universal_determiner() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"(?i)^\W*(a|an|any|every|each|all|no)\s+").expect("universal-determiner regex")
    })
}

/// The head of a restrictive filter clause: a finite relative pronoun or a
/// reduced relative.
fn re_filter_marker() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(
            r"(?i)\b(whose|that|which|who|when|if|where|with|without|containing|carrying|lacking|absent\s+from|present\s+in)\b",
        )
        .expect("filter-marker regex")
    })
}

/// Modifier position: an article, determiner or preposition immediately before
/// the marker, which makes an `-s`/`-ed`/`-ing` word part of a noun phrase
/// rather than a predicate.
fn re_modifier_context() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(
            r"(?i)\b(a|an|the|any|every|each|all|no|its|their|his|her|our|your|this|that|these|those|some|one|two|both|either|neither|of|in|to|from|with|for|on|at|by|into|over|under|per|via|across|between|about)\s+$",
        )
        .expect("modifier-context regex")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::{GrammarLexicon, ObservableVerbs, VacuousPredicates};
    use proptest::prelude::*;

    fn idioms() -> PropertyIdioms {
        PropertyIdioms::default()
    }

    fn shape(statement: &str) -> PropertyShape {
        classify_property(statement, &idioms()).property
    }

    /// The engine-only classification — no module registry, and no built-in
    /// idioms either, so only the closed structural signals can fire.
    fn structural_only() -> PropertyIdioms {
        PropertyIdioms {
            by_shape: PropertyShape::ALL.iter().map(|_| None).collect(),
        }
    }

    fn vocab<'a>(
        lexicon: &'a GrammarLexicon,
        observable: &'a ObservableVerbs,
        vacuous: &'a VacuousPredicates,
        idioms: &'a PropertyIdioms,
    ) -> GrammarVocabularies<'a> {
        GrammarVocabularies {
            lexicon,
            observable,
            vacuous,
            idioms,
        }
    }

    /// An FR whose `Acceptance Criteria` table carries `cells` as rows.
    fn ac_doc(archetype: &str, section: &str, kind: &str, cells: &[&str]) -> QuireDocument {
        let mut md = format!(
            "---\nid: {archetype}-001\ntype: {archetype}\n---\n## {section}\n\n\
             | ID | Criteria | Verification |\n|----|----------|--------------|\n"
        );
        for (i, cell) in cells.iter().enumerate() {
            md.push_str(&format!(
                "| {archetype}-001-{kind}-{} | {cell} | Test |\n",
                i + 1
            ));
        }
        crate::parse_document(&md)
    }

    fn classify_cells(archetype: &str, cells: &[&str]) -> Vec<AcClassification> {
        let (section, kind) = match archetype {
            "StR" => ("Validation Criteria", "VC"),
            _ => ("Acceptance Criteria", "AC"),
        };
        let (lx, ob, vc, id) = (
            GrammarLexicon::empty(),
            ObservableVerbs::default(),
            VacuousPredicates::default(),
            idioms(),
        );
        classify_document(
            archetype,
            &ac_doc(archetype, section, kind, cells),
            0,
            vocab(&lx, &ob, &vc, &id),
        )
    }

    /// The issue's own cell — the worked example FR-052 is written against.
    const ISSUE_CELL: &str =
        "A finding whose key is absent from the merged map defaults to warning";

    // TC-779 (FR-052-AC-1): the issue's own cell classifies `Universal` with
    // all three spans populated.
    #[test]
    fn tc779_universal_decomposition_yields_three_spans() {
        let c = classify_property(ISSUE_CELL, &idioms());
        assert_eq!(c.property, PropertyShape::Universal);
        assert!(c.extractable);
        let spans = c.spans.expect("the issue's own cell decomposes");
        assert_eq!(spans.domain.text, "finding");
        assert_eq!(
            spans
                .precondition
                .as_ref()
                .map(|s| s.text.as_str())
                .unwrap_or_default(),
            "whose key is absent from the merged map"
        );
        assert_eq!(spans.oracle.text, "defaults to warning");
        // The audit trail names every signal that fired, in evaluation order.
        assert_eq!(
            c.signals,
            vec![
                "universal:determiner",
                "span:domain",
                "span:precondition",
                "span:oracle"
            ]
        );
    }

    // TC-780 (FR-052-AC-2): each metamorphic idiom labels to its own shape, and
    // a universally quantified round-trip labels `RoundTrip` — structure
    // outranks quantification under the fixed precedence.
    #[test]
    fn tc780_metamorphic_shapes_and_fixed_precedence() {
        assert_eq!(
            shape("Serializing a document then parsing it yields the input"),
            PropertyShape::RoundTrip
        );
        assert_eq!(
            shape("Applying the migration twice yields the same result"),
            PropertyShape::Idempotence
        );
        assert_eq!(
            shape("Findings are emitted in declaration order"),
            PropertyShape::Ordering
        );
        assert_eq!(
            shape("The loader never mutates the source file"),
            PropertyShape::Invariant
        );

        // Universally quantified *and* a round trip: precedence ranks structure
        // above quantification, so the label is `RoundTrip`, not `Universal`.
        let quantified = "Every document that is serialized then parsed yields the input";
        assert!(
            re_universal_determiner().is_match(quantified),
            "the fixture must really be universally quantified"
        );
        assert_eq!(shape(quantified), PropertyShape::RoundTrip);
        // …and the metamorphic shapes carry no spans in v1.
        assert!(classify_property(quantified, &idioms()).spans.is_none());
    }

    // TC-781 (FR-052-AC-3): CR-017 parity — an idiom phrase inside an inline
    // code span is a mention and fires no signal; unquoted, it fires.
    #[test]
    fn tc781_quoted_idioms_are_mentions() {
        let quoted = "The message template embeds `always` verbatim.";
        let used = "The message template embeds always verbatim.";
        assert_ne!(shape(quoted), PropertyShape::Invariant);
        assert_eq!(shape(used), PropertyShape::Invariant);

        // Same for a module/built-in idiom phrase.
        let quoted_idiom = "The report lists every `concurrently` marker it saw.";
        let used_idiom = "The report lists every marker it saw concurrently.";
        assert_ne!(shape(quoted_idiom), PropertyShape::Concurrency);
        assert_eq!(shape(used_idiom), PropertyShape::Concurrency);
    }

    // TC-782 (FR-052-AC-4): a criterion whose oracle boundary rests only on the
    // weak inflected-verb marker classifies `Universal` with all three spans
    // absent — a confidently wrong span is worse for a generator than none —
    // and refusing spans changes neither the shape nor `extractable`.
    #[test]
    fn tc782_weak_only_boundary_refuses_spans() {
        let weak = "Each cached entry expires after the configured interval";
        let c = classify_property(weak, &idioms());
        assert_eq!(c.property, PropertyShape::Universal);
        assert!(c.spans.is_none());
        assert!(
            c.extractable,
            "refusing spans must not change `extractable`"
        );
        assert!(c.signals.contains(&"span:refused-weak-boundary"));

        // The same criterion with a Strong marker anchoring the split does
        // decompose — the refusal is about the evidence, not the sentence.
        let anchored = "Each cached entry that is stale expires after the interval";
        assert!(classify_property(anchored, &idioms()).spans.is_some());
    }

    // TC-783 (FR-052-AC-5): a specific-scenario criterion classifies `Example`
    // with `extractable == false`, and contributes zero findings to any `ac`
    // check — classification is metadata, never a verdict (FR-052-CON-1).
    #[test]
    fn tc783_example_is_not_extractable_and_not_a_defect() {
        let cell = "The loader emits a `Duplicate` diagnostic for the second declaration.";
        let c = classify_property(cell, &idioms());
        assert_eq!(c.property, PropertyShape::Example);
        assert!(!c.extractable);
        assert!(c.spans.is_none());

        let (lx, ob, vc, id) = (
            GrammarLexicon::empty(),
            ObservableVerbs::default(),
            VacuousPredicates::default(),
            idioms(),
        );
        let findings = ac::check(
            "FR",
            &ac_doc("FR", "Acceptance Criteria", "AC", &[cell]),
            0,
            vocab(&lx, &ob, &vc, &id),
        );
        assert!(
            findings.is_empty(),
            "an `Example` criterion is not a defect: {findings:?}"
        );
    }

    // TC-785 (FR-052-AC-7, FR-052-CON-1): the `ac` finding stream over a
    // fixture corpus is unchanged finding-for-finding, field-for-field and
    // order-for-order by the classifier's presence. This is what guarantees no
    // existing corpus number moves.
    #[test]
    fn tc785_ac_finding_stream_is_unchanged_by_classification() {
        let corpus: &[(&str, &str, &str)] = &[
            ("FR", "Acceptance Criteria", "AC"),
            ("NFR", "Acceptance Criteria", "AC"),
            ("StR", "Validation Criteria", "VC"),
            ("US", "Acceptance Criteria", "AC"),
            ("IT", "Acceptance Criteria", "AC"),
        ];
        let cells = [
            "Structural evaluation",
            "The system shall emit `A` and shall persist `B`.",
            "Given a token, when it expires, then `401` is returned.",
            "It all works end to end.",
            ISSUE_CELL,
            "Serializing a document then parsing it yields the input",
            "The system shall support pagination.",
        ];
        let (lx, ob, vc, id) = (
            GrammarLexicon::empty(),
            ObservableVerbs::default(),
            VacuousPredicates::default(),
            idioms(),
        );
        let v = vocab(&lx, &ob, &vc, &id);

        for (archetype, section, kind) in corpus {
            let doc = ac_doc(archetype, section, kind, &cells);
            let before = ac::check(archetype, &doc, 0, v);
            let records = classify_document(archetype, &doc, 0, v);
            let after = ac::check(archetype, &doc, 0, v);
            assert_eq!(
                before, after,
                "{archetype}: classification must not perturb the `ac` stream"
            );
            // …and running the classifier first cannot change it either.
            let _ = records;
            assert_eq!(ac::check(archetype, &doc, 0, v), before);
        }
    }

    // TC-786 (FR-052-AC-8): a module `property_idioms` registry merges
    // first-wins over the engine built-ins; an absent declaration classifies
    // against the built-ins alone (mirrors TC-757).
    #[test]
    fn tc786_module_idioms_merge_first_wins() {
        let cell = "Two writers append to the log in parallel";
        // Built-ins alone: `in parallel` is a `Concurrency` idiom.
        assert_eq!(
            classify_property(cell, &PropertyIdioms::default()).property,
            PropertyShape::Concurrency
        );
        // A module redeclaring the same phrase wins for that phrase.
        let redeclared =
            PropertyIdioms::with_module_idioms([("in parallel", PropertyShape::Lifecycle)]);
        assert_eq!(
            classify_property(cell, &redeclared).property,
            PropertyShape::Lifecycle
        );
        // …while the built-ins the module is silent about survive.
        assert_eq!(
            classify_property("The record transitions from `draft` to `live`", &redeclared)
                .property,
            PropertyShape::Lifecycle
        );
        // A module may also declare a new phrase entirely.
        let extended =
            PropertyIdioms::with_module_idioms([("encode and decode", PropertyShape::RoundTrip)]);
        assert_eq!(
            classify_property("The codec encode and decode paths agree", &extended).property,
            PropertyShape::RoundTrip
        );
        assert_ne!(
            classify_property("The codec encode and decode paths agree", &idioms()).property,
            PropertyShape::RoundTrip
        );
    }

    // TC-787 (FR-052-AC-9, CR-020): classification sees exactly the cells
    // `ac::check` sees — FR/NFR `Acceptance Criteria`, StR `Validation
    // Criteria`, and their supplements — while a US or IT document yields no
    // records and no finding.
    #[test]
    fn tc787_binding_parity_with_the_ac_grammar() {
        for archetype in ["FR", "NFR", "StR"] {
            let records = classify_cells(archetype, &[ISSUE_CELL, "Structural evaluation"]);
            assert_eq!(records.len(), 2, "{archetype} criteria must be classified");
            // `row_id` is read from the `ID` column — without it the Phase C
            // tracking tag cannot be emitted.
            let kind = if archetype == "StR" { "VC" } else { "AC" };
            assert_eq!(
                records[0].row_id.as_deref(),
                Some(format!("{archetype}-001-{kind}-1").as_str())
            );
            assert!(records[0].line.is_some());
            // The FR-047 axis is carried through unchanged.
            assert_eq!(records[0].shape, AcShape::Assertion);
            assert_eq!(records[1].shape, AcShape::Unstructured);
            assert_eq!(records[1].property, PropertyShape::Unclassified);
            assert!(!records[1].extractable);
        }

        // US and IT declare no binding-criteria table: no records, no findings.
        let (lx, ob, vc, id) = (
            GrammarLexicon::empty(),
            ObservableVerbs::default(),
            VacuousPredicates::default(),
            idioms(),
        );
        for archetype in ["US", "IT"] {
            assert!(classify_cells(archetype, &[ISSUE_CELL]).is_empty());
            assert!(ac::check(
                archetype,
                &ac_doc(archetype, "Acceptance Criteria", "AC", &[ISSUE_CELL]),
                0,
                vocab(&lx, &ob, &vc, &id),
            )
            .is_empty());
        }

        // A supplement subsection is classified too, carrying the criterion id
        // from its `### <doc-id>-<kind>-N` heading.
        let supplement = crate::parse_document(
            "---\nid: FR-001\ntype: FR\n---\n## Notes\n\n### FR-001-AC-1\n\n\
             A finding whose key is absent from the merged map defaults to warning.\n",
        );
        let records = classify_document("FR", &supplement, 0, vocab(&lx, &ob, &vc, &id));
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].row_id.as_deref(), Some("FR-001-AC-1"));
        assert_eq!(records[0].property, PropertyShape::Universal);
    }

    // TC-791 (FR-052-AC-13): the closed structural signals fire with no
    // registry at all, and the ordering lexicon deliberately excludes the words
    // that would swallow `Example`.
    #[test]
    fn tc791_closed_structural_signals_need_no_registry() {
        let bare = structural_only();
        let shape = |s: &str| classify_property(s, &bare).property;

        assert_eq!(
            shape("Serializing a document then parsing it yields the input"),
            PropertyShape::RoundTrip
        );
        assert_eq!(
            shape("Running the migration a second time yields the same result"),
            PropertyShape::Idempotence
        );
        // A bare `deterministic`, `before` or `order` is **not** an ordering
        // signal — each swallows a large slice of `Example`.
        for not_ordering in [
            "The output is deterministic",
            "The cache is warmed before the first query",
            "The report lists one row per order",
        ] {
            assert_ne!(
                shape(not_ordering),
                PropertyShape::Ordering,
                "{not_ordering}"
            );
        }
        // …while the lexicon itself still fires.
        assert_eq!(
            shape("The entries are sorted by path"),
            PropertyShape::Ordering
        );

        // With every idiom removed, extraction coverage is unchanged: the
        // structural signals carry it (FR-052-CON-4's premise).
        assert!(classify_property(ISSUE_CELL, &bare).extractable);
    }

    // ── Quantification at a bounded subject position (CR-029) ───────────────

    /// One real corpus criterion the sentence-initial determiner anchor drops,
    /// and the signal [`recall_subject_determiner`] fires on it — or `None`
    /// where the rule declines it. Declining is a first-class outcome: a rule
    /// that cannot bound the subject position refuses rather than guessing.
    struct RecallFixture {
        /// `<repo> <criterion id>`, so a fixture can be traced to its source.
        id: &'static str,
        statement: &'static str,
        subject_determiner: Option<&'static str>,
    }

    /// The recall sample from `~/dev/reports/2026-08-07-ac-property-shape-sweep.md`
    /// §3 — 16 criteria a hand-labelling pass judged genuinely property-shaped
    /// and the classifier marked `extractable: false`.
    ///
    /// Provenance, because it is not a clean single source: the report records
    /// **six** of the sixteen verbatim, and those six are here byte-for-byte
    /// (with the two the report elides restored from their repos). The other
    /// ten are drawn from re-running the report's own seeded recall frame
    /// (`--properties --sample 30 --seed 20260807`, stream `seed + 1`) and
    /// re-applying the report's rubric — *could a generator produce members of
    /// a domain and check the stated oracle against them?*
    ///
    /// The table is kept **whole** after CR-029, not trimmed to the two cells
    /// the shipped rule reaches: the fourteen it declines are the negative
    /// space, and a fixture set holding only the cells a rule catches measures
    /// nothing. It is also the standing reminder the experiment produced — the
    /// two deleted rules reached 1 and 11 of these sixteen respectively, which
    /// predicted neither's corpus precision. A fixture table is an existence
    /// proof, not a measurement.
    const RECALL_FIXTURES: &[RecallFixture] = &[
        // ── The six the report names verbatim ───────────────────────────────
        RecallFixture {
            id: "ix-agent-tools FR-007-AC-2",
            statement: "When a command exceeds its timeout the result carries `timed_out` true",
            subject_determiner: Some("recall:subject-determiner:fronted-subject"),
        },
        RecallFixture {
            id: "ts-auth-sdk FR-003-AC-3",
            statement: "Given the identity service returns a non-2xx response, when \
                        `getProfile()` is called, the SDK MUST return `null` silently (not throw).",
            subject_determiner: None,
        },
        RecallFixture {
            id: "config-overlay FR-005-AC-2",
            statement: "Setting a field to `null` in the overlay removes it from the effective \
                        config",
            subject_determiner: None,
        },
        RecallFixture {
            id: "filament-ide-rs FR-083-AC-2",
            statement: "Following `next_cursor` to exhaustion yields every project graph node and \
                        edge exactly once, with totals matching the delivered counts",
            subject_determiner: None,
        },
        RecallFixture {
            id: "identity FR-029-AC-5",
            statement: "Exit codes match §5 contract for each documented failure mode",
            subject_determiner: None,
        },
        RecallFixture {
            id: "sync-github FR-004-AC-4",
            statement: "In metadata-only mode the snapshot carries the source identity, branch, \
                        and head SHA, `container_path` is `None`, and no tarball is downloaded",
            subject_determiner: None,
        },
        // ── The ten recovered from the re-run of the seeded recall frame ────
        RecallFixture {
            id: "auth FR-026-AC-5",
            statement: "`POST /api/auth/device/approve` with a valid session but missing/invalid \
                        CSRF token returns 403 and records no consent",
            subject_determiner: None,
        },
        RecallFixture {
            id: "filament-view-ecosystem FR-005-AC-2",
            statement: "Given the component unmounts, the ResizeObserver is disconnected during \
                        cleanup",
            subject_determiner: None,
        },
        RecallFixture {
            id: "workflow-worker-pool FR-006-AC-2",
            statement: "When broker connected and subscriptions active, `/health/ready` returns \
                        200 with `{\"ready\": true}`",
            subject_determiner: None,
        },
        RecallFixture {
            id: "user-admin-ui FR-017-AC-3",
            statement: "Auditor sessions render read-only audit context without mutation \
                        affordances.",
            subject_determiner: None,
        },
        RecallFixture {
            id: "filament-view-ecosystem FR-008-AC-3",
            statement: "Given a category is toggled off, clicking it again restores its nodes and \
                        links",
            subject_determiner: Some("recall:subject-determiner:fronted-subject"),
        },
        RecallFixture {
            id: "glitchtip-mcp NFR-003-AC-3",
            statement: "4xx errors do not suggest retry",
            subject_determiner: None,
        },
        RecallFixture {
            id: "code-diff-editor FR-081-AC-2",
            statement: "Given split mode (selectable via `ViewModeSelector`), unchanged lines \
                        align horizontally across columns",
            subject_determiner: None,
        },
        RecallFixture {
            id: "user-admin-ui FR-022-AC-2",
            statement: "Creating a token shows the raw token value once and displays copy \
                        affordance in the success state.",
            subject_determiner: None,
        },
        RecallFixture {
            id: "markdown-editor FR-096-AC-4",
            statement: "**Given** `text` nodes carrying `comment` or `reviewHighlight` marks \
                        **When** the document is serialized **Then** those marks are omitted from \
                        the output while the underlying text is preserved.",
            subject_determiner: None,
        },
        RecallFixture {
            id: "identity FR-057-AC-9",
            statement: "Rate-limited WebAuthn endpoints return stable errors without consuming \
                        challenges or changing credential state.",
            subject_determiner: None,
        },
    ];

    /// Cells that must stay out of extraction, whichever subject position is
    /// read.
    ///
    /// `returns a diagnostic` is the discipline case: `a diagnostic` is the
    /// outcome, not the domain, so no position may reach it. The `Metrics: …`
    /// fragment was the known false positive of `recall-generic-subject`, the
    /// rule the 2026-08-07 experiment deleted at 56.7% precision; it is kept
    /// here as a must-not-fire control, which is what deleting rather than
    /// narrowing that rule bought.
    const RECALL_CONTROLS: &[(&str, &str)] = &[
        (
            "determiner in the outcome, not the subject",
            "The loader returns a diagnostic for the second declaration",
        ),
        (
            "a specific scenario is not a property",
            "The loader emits a `Duplicate` diagnostic for the second declaration.",
        ),
        (
            "a generic subject with no determiner is not quantification",
            "Metrics: operation counts, durations, error rates",
        ),
        ("no subject a rule can bound", "Unknown user_code → 404"),
    ];

    fn classify_fixture(statement: &str) -> Classified {
        classify_property(statement, &idioms())
    }

    /// True when a classification was reached at a widened subject position
    /// rather than by the sentence-initial anchor.
    fn recall_signal(c: &Classified) -> Option<&'static str> {
        c.signals.iter().find(|s| s.starts_with("recall:")).copied()
    }

    /// TC-792 (FR-052-AC-14): the determiner is read at a bounded subject
    /// position as well as at offset 0 — and the rule refuses everywhere it
    /// cannot bound one.
    #[test]
    fn tc792_subject_determiner_widens_position_not_vocabulary() {
        for f in RECALL_FIXTURES {
            let c = classify_fixture(f.statement);
            match f.subject_determiner {
                Some(signal) => {
                    assert_eq!(
                        recall_signal(&c),
                        Some(signal),
                        "{}: expected the rule to fire — got {:?}",
                        f.id,
                        c.signals
                    );
                    assert_eq!(c.property, PropertyShape::Universal, "{}", f.id);
                    assert!(
                        c.extractable,
                        "{}: a recovered criterion is extractable",
                        f.id
                    );
                }
                None => assert_eq!(
                    recall_signal(&c),
                    None,
                    "{}: the rule must decline this cell, not guess at it",
                    f.id
                ),
            }
        }

        // The main-clause position fires too, when a fronted phrase closes with
        // a comma.
        assert_eq!(
            recall_signal(&classify_fixture(
                "In strict mode, every finding is promoted to an error"
            )),
            Some("recall:subject-determiner:main-subject")
        );

        // The determiner set is unchanged — only the position widened.
        assert_eq!(
            recall_signal(&classify_fixture(
                "When the request fails, the caller sees a diagnostic"
            )),
            None,
            "`the` is definite, not universal"
        );
        // And an unbounded fronted phrase is refused rather than guessed at.
        assert_eq!(
            recall_signal(&classify_fixture(
                "In metadata-only mode the snapshot carries a source identity"
            )),
            None,
            "no comma bounds the fronted phrase, so no subject head is bounded"
        );

        for (why, statement) in RECALL_CONTROLS {
            assert!(
                recall_signal(&classify_fixture(statement)).is_none(),
                "{why}: {statement:?}"
            );
        }
    }

    /// TC-793 (FR-052-AC-15): widening the determiner's position moved the
    /// classification of exactly the cells it claims and **nothing else** — no
    /// `ac` finding moved, and every other shape the classifier reaches lands
    /// byte-identically where it did before CR-029.
    ///
    /// This is the pin the recall experiment was carried on: it started life
    /// asserting a default build was byte-identical to the shipped classifier
    /// while three rules sat behind cargo features. One of those rules now
    /// ships, so the pin is rebased onto the adopted behaviour rather than
    /// retired — the finding stream is still the thing that may not move.
    #[test]
    fn tc793_widening_moves_no_finding_and_no_other_shape() {
        // 1. The `ac` finding stream is untouched. The property axis is
        //    metadata (FR-052-CON-1), so no corpus finding count may move
        //    because a criterion is now read as quantified.
        let (lx, ob, vc, id) = (
            GrammarLexicon::empty(),
            ObservableVerbs::default(),
            VacuousPredicates::default(),
            idioms(),
        );
        let v = vocab(&lx, &ob, &vc, &id);
        let cells: Vec<&str> = RECALL_FIXTURES.iter().map(|f| f.statement).collect();

        for archetype in ["FR", "NFR", "StR"] {
            let (section, kind) = match archetype {
                "StR" => ("Validation Criteria", "VC"),
                _ => ("Acceptance Criteria", "AC"),
            };
            let doc = ac_doc(archetype, section, kind, &cells);
            let before = ac::check(archetype, &doc, 0, v);
            let records = classify_document(archetype, &doc, 0, v);
            assert_eq!(records.len(), cells.len());
            assert_eq!(
                ac::check(archetype, &doc, 0, v),
                before,
                "{archetype}: the property axis must not move an `ac` finding"
            );
        }

        // 2. Every fixture the rule declines is exactly where the shipped
        //    classifier left it, shape, `extractable` and signal trail alike.
        let fingerprint = |c: &Classified| {
            format!(
                "{}|{}|{}",
                c.property.as_str(),
                c.extractable,
                c.signals.join(",")
            )
        };
        for f in RECALL_FIXTURES {
            if f.subject_determiner.is_some() {
                continue;
            }
            assert_eq!(
                fingerprint(&classify_fixture(f.statement)),
                "example|false|example:specific-scenario",
                "{}",
                f.id
            );
        }

        // 3. …and so is every shape the module's own fixtures reach, so the pin
        //    covers the classifier's whole output surface and not just the
        //    cells the widening is aimed at.
        let pinned: &[(&str, &str)] = &[
            (
                ISSUE_CELL,
                "universal|true|universal:determiner,span:domain,span:precondition,span:oracle",
            ),
            (
                "Serializing a document then parsing it yields the input",
                "round-trip|true|round-trip:composition,round-trip:identity-back-reference",
            ),
            (
                "Applying the migration twice yields the same result",
                "idempotence|true|idempotence:repetition,idempotence:equality",
            ),
            (
                "Findings are emitted in declaration order",
                "ordering|true|ordering:lexicon",
            ),
            (
                "The loader never mutates the source file",
                "invariant|true|invariant:absolute",
            ),
            (
                "Each cached entry expires after the configured interval",
                "universal|true|universal:determiner,span:refused-weak-boundary",
            ),
            (
                "Structural evaluation",
                "unclassified|false|unclassified:no-signal",
            ),
            (
                "Two writers append to the log in parallel",
                "concurrency|false|unclassified:no-signal,idiom:concurrency",
            ),
        ];
        for (statement, expected) in pinned {
            assert_eq!(
                fingerprint(&classify_fixture(statement)),
                *expected,
                "{statement}"
            );
        }
    }

    // ── Property tests ──────────────────────────────────────────────────────

    /// Statement-shaped strings: determiners, filter markers, idiom phrases,
    /// code spans and multi-byte text, so the generator reaches every branch
    /// that can emit a span.
    fn any_statement() -> impl Strategy<Value = String> {
        let word = prop_oneof![
            Just("A"),
            Just("Every"),
            Just("no"),
            Just("finding"),
            Just("whose"),
            Just("that"),
            Just("key"),
            Just("is"),
            Just("absent"),
            Just("from"),
            Just("the"),
            Just("merged"),
            Just("map"),
            Just("defaults"),
            Just("to"),
            Just("warning"),
            Just("then"),
            Just("yields the input"),
            Just("twice"),
            Just("sorted"),
            Just("always"),
            Just("in parallel"),
            Just("`quoted`"),
            Just("`always`"),
            Just("ünïcödé"),
            Just("."),
        ];
        proptest::collection::vec(word, 0..12).prop_map(|words| words.join(" "))
    }

    proptest! {
        // TC-784 (FR-052-AC-6): for every emitted span,
        // `statement[start..end] == text`; the spans of one record are in
        // bounds, non-overlapping and ascending by start offset.
        #[test]
        fn tc784_span_offsets_and_text_agree(statement in any_statement()) {
            let c = classify_property(&statement, &idioms());
            let Some(spans) = c.spans else { return Ok(()); };
            let ordered: Vec<&Span> = [Some(&spans.domain), spans.precondition.as_ref(), Some(&spans.oracle)]
                .into_iter()
                .flatten()
                .collect();
            let mut prev_end = 0usize;
            for span in ordered {
                prop_assert!(span.start < span.end, "empty span in {statement:?}");
                prop_assert!(span.end <= statement.len(), "out of bounds in {statement:?}");
                prop_assert_eq!(
                    statement.get(span.start..span.end),
                    Some(span.text.as_str()),
                    "offsets and text disagree in {:?}",
                    statement
                );
                prop_assert!(
                    span.start >= prev_end,
                    "spans overlap or descend in {statement:?}"
                );
                prev_end = span.end;
            }
        }

        // TC-790 (FR-052-AC-12, FR-052-CON-4): `extractable` is identical with
        // the idiom registry present and absent — only the `property` label may
        // differ — so a missed idiom degrades a label and never coverage.
        #[test]
        fn tc790_extractable_is_registry_independent(statement in any_statement()) {
            let declared = PropertyIdioms::with_module_idioms([
                ("finding", PropertyShape::RoundTrip),
                ("map", PropertyShape::ErrorCase),
                ("warning", PropertyShape::Lifecycle),
                ("in parallel", PropertyShape::Ordering),
            ]);
            let without = classify_property(&statement, &structural_only());
            let with_builtins = classify_property(&statement, &idioms());
            let with_module = classify_property(&statement, &declared);
            prop_assert_eq!(without.extractable, with_builtins.extractable);
            prop_assert_eq!(without.extractable, with_module.extractable);
            // …and the spans are registry-independent too: a declaration can
            // never remove a decomposition a consumer already relies on.
            prop_assert_eq!(&without.spans, &with_module.spans);
        }
    }
}
