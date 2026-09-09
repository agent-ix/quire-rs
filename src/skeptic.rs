//! The skeptic layer (FR-064): the finding classes only manual review caught.
//!
//! Battletest pass 2's verdict on this toolchain was **good reporters, poor
//! skeptics**. Every conclusion-changing finding of that pass came from a human
//! reading code, and two of them are mechanizable:
//!
//! - a property suite that **asserted nothing** in a measured 97.7% of samples
//!   and was green throughout (`agent-ix/quire-rs#235`);
//! - a test oracle that was a **character-for-character copy** of the code under
//!   test, redundant branch included. Replacing it with a real oracle
//!   immediately exposed a genuine containment gap (`agent-ix/quire-rs#236`).
//!
//! ## Suspicions, never failures
//!
//! Everything here emits a **suspicion**: a claim that something looks like a
//! known-bad shape, carrying what it measured so a reader can dismiss it in one
//! look. Neither check can be certain — a guarded assertion is sometimes exactly
//! right, and an oracle legitimately resembles the code when the behaviour is a
//! transformation with one obvious spelling.
//!
//! That is advisory-first for the usual reason (blast radius), and for a sharper
//! one: these fire on *test* code, and a check that can fail somebody's build
//! over a heuristic about their assertions will be turned off within a week.

use std::sync::OnceLock;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::symbols::{Symbol, SymbolExtraction, SymbolKind};
use crate::traceability::SourceLanguage;

/// What kind of doubt this is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SuspicionKind {
    /// Every assertion in the symbol sits behind a narrowing guard, so an input
    /// that does not enter it passes without being checked.
    VacuousUnderGuard,
    /// An oracle that closely resembles the implementation it judges.
    OracleResemblesImplementation,
}

impl SuspicionKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::VacuousUnderGuard => "vacuous-under-guard",
            Self::OracleResemblesImplementation => "oracle-resembles-implementation",
        }
    }
}

/// One thing that looks wrong, with the measurement that made it look wrong.
///
/// `evidence` is not decoration. A suspicion a reader cannot check in one look
/// is one they learn to scroll past, which is how an advisory check becomes
/// noise rather than a skeptic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Suspicion {
    pub kind: String,
    pub path: String,
    pub symbol: String,
    pub line: usize,
    pub message: String,
    /// The numbers behind the claim, rendered.
    pub evidence: String,
    /// Stable structured repair guidance, additive to the human message.
    #[serde(default, flatten)]
    pub guidance: Option<crate::finding::FindingGuidance>,
}

impl Suspicion {
    pub(crate) fn structured_guidance(&self) -> crate::finding::FindingGuidance {
        let subject = format!("test symbol `{}`", self.symbol);
        let target = format!("{}:{} in `{}`", self.path, self.line, self.symbol);
        match self.kind.as_str() {
            "oracle-resembles-implementation" => crate::finding::FindingGuidance::remedy(
                subject,
                target,
                "replace the copied calculation or helper with an independent expectation",
            ),
            "vacuous-under-guard" => crate::finding::FindingGuidance::diagnostic(
                subject,
                target,
                "inspect inputs that bypass the narrowing guard; add an unconditional oracle or constrain the generator when those inputs are invalid",
            ),
            _ => crate::finding::FindingGuidance::diagnostic(
                subject,
                target,
                "inspect the reported evidence before changing the test",
            ),
        }
    }
}

/// Assertion macros this recognizes. Deliberately a closed list: an open
/// heuristic ("any call containing `assert`") binds `assert_within_budget`, a
/// helper that may itself assert nothing, and the point is to be *right* about
/// the shape rather than to catch every spelling.
const RUST_ASSERTIONS: &[&str] = &[
    "assert!",
    "assert_eq!",
    "assert_ne!",
    "assert_matches!",
    "prop_assert!",
    "prop_assert_eq!",
    "prop_assert_ne!",
    "panic!",
    "unreachable!",
    "expect(",
    "unwrap()",
];

/// The `vitest` / `jest` vocabulary. `expect(` is the whole surface in
/// practice; `assert(` covers the `node:assert` spelling a few suites use.
const TYPESCRIPT_ASSERTIONS: &[&str] = &["expect(", "assert(", "assert."];

/// `pytest` asserts with the bare statement; `self.assert` catches the
/// `unittest` spelling and `pytest.raises` the context-manager oracle.
const PYTHON_ASSERTIONS: &[&str] = &["assert ", "self.assert", "pytest.raises"];

fn assertions_for(language: SourceLanguage) -> &'static [&'static str] {
    match language {
        SourceLanguage::Rust => RUST_ASSERTIONS,
        SourceLanguage::Typescript => TYPESCRIPT_ASSERTIONS,
        SourceLanguage::Python => PYTHON_ASSERTIONS,
    }
}

/// Guard openers that NARROW the input space — the shape that makes an
/// assertion conditional on the sample.
///
/// `if let` / `let ... else` / a `match` arm each admit a subset. A plain `if`
/// on a boolean does too, but it is also how every table-driven test is
/// written, so it is excluded: including it made the check fire on most of the
/// corpus for a reason that had nothing to do with vacuity.
const RUST_NARROWING_GUARDS: &[&str] = &["if let ", "while let ", "=> {"];

/// **Empty, deliberately (CR-102).** Neither language has a binding-and-testing
/// construct like `if let`, and the shape this check reports has not been
/// measured in either.
///
/// The previous revision shared Rust's list, whose `"=> {"` entry means a
/// `match` arm in Rust and an **arrow function** in TypeScript — so every
/// `it("…", () => { … })` body opened a guard and every assertion inside one
/// counted as guarded. Measured on `agent-ix/quoin` that was **549 suspicions
/// from 551 candidates**, against 2 of 883 on this crate. Sampled three, all
/// rule, none real.
///
/// This is a narrowing, and its justification is independent of the count:
/// `=> {` in TypeScript is a different construct, not a narrowing guard. What
/// the equivalent shape *is* in these languages is an open question and gets
/// its own measurement before anything fires here.
const NO_MEASURED_GUARDS: &[&str] = &[];

fn narrowing_guards_for(language: SourceLanguage) -> &'static [&'static str] {
    match language {
        SourceLanguage::Rust => RUST_NARROWING_GUARDS,
        SourceLanguage::Typescript | SourceLanguage::Python => NO_MEASURED_GUARDS,
    }
}

/// Property suites whose assertions may never run (FR-064-AC-1).
///
/// The measured case: 4,000 samples through `Ok(Some)` 2.3%, `Ok(None)` 79.0%,
/// `Err` 18.8% — with the assertion inside the `Ok(Some)` arm. The suite was
/// green and checked 2.3% of what it claimed to.
///
/// This is the **static** shape of that: every assertion nested under a
/// narrowing guard, with no assertion outside one. It cannot know the sample
/// distribution — that needs a run — so it reports the structure and says so.
pub fn vacuous_property_suites(extraction: &SymbolExtraction) -> Vec<Suspicion> {
    let mut out = Vec::new();
    for symbol in &extraction.symbols {
        if !symbol.kind.binds_trace_ids() {
            continue;
        }
        let Some(source) = extraction.source_of(&symbol.path) else {
            continue;
        };
        let span = symbol.attached_source(source);
        let (total, guarded) = assertion_positions(&span, symbol.language);

        // `total == 0` is deliberately NOT a finding. Measured on this
        // repository it was 57 of 65 suspicions and **12 of 12 sampled were
        // rule, 0 real**: in Rust a test fails on panic, so absence of an
        // assertion macro is not absence of an oracle.
        //
        //   `fn assert_send_sync<T: Send + Sync>() {} assert_send_sync::<T>()`
        //       — the assertion is at COMPILE time; it cannot fail at runtime
        //         because it already failed to build.
        //   `fn never_panics(s in "\\PC*") { let _ = parse_document(&s); }`
        //       — the oracle IS the absence of a panic, which is a real oracle.
        //
        // The finding #235 describes is assertions that do not RUN, not
        // assertions that do not exist.
        if total > 0 && guarded == total {
            out.push(suspicion(
                symbol,
                SuspicionKind::VacuousUnderGuard,
                "every assertion sits behind a narrowing guard, so an input that \
                 does not enter it passes unchecked — the shape of a suite that \
                 was green while checking 2.3% of its samples",
                format!("{guarded} of {total} assertions guarded, 0 unguarded"),
            ));
        }
    }
    out.sort_by(|a, b| (&a.path, a.line, &a.symbol).cmp(&(&b.path, b.line, &b.symbol)));
    out
}

/// `(total assertions, assertions nested under a narrowing guard)`.
///
/// Brace depth relative to the guard, not a parser: this reads the same text
/// the binder does and must not acquire a Rust front-end to answer a question
/// about shape.
///
/// A guard that opens **and closes on one line** covers that line only, and is
/// handled here rather than by the depth tracking — it never enters
/// `guard_depths`, because by the next line it is already closed (CR-102). The
/// previous revision tested the assertion before considering the guard and
/// pushed only when `opens > closes`, so `if let Some(x) = y { assert!(x) }`
/// reported **0 suspicions** while its multi-line spelling reported one.
/// The code part of a line: everything before a line comment (CR-102).
///
/// A comment is prose, not an oracle. Left in, prose that *quotes* code is read
/// as code — this crate's own TC-1003 was reported vacuous because a comment
/// explaining the TypeScript arrow-function bug contained the token it names,
/// which is the wrong-language misread one level up. Braces are counted on the
/// stripped text for the same reason.
///
/// Not a lexer: `//` inside a string literal is treated as a comment. The
/// alternative is a front-end per language, which FR-064-CON-2 rules out, and
/// an assertion sharing a line with a `://` is not a shape worth the cost.
fn strip_comment(line: &str, language: SourceLanguage) -> &str {
    let marker = match language {
        SourceLanguage::Rust | SourceLanguage::Typescript => "//",
        SourceLanguage::Python => "#",
    };
    match line.find(marker) {
        Some(at) => &line[..at],
        None => line,
    }
}

fn assertion_positions(span: &str, language: SourceLanguage) -> (usize, usize) {
    let assertions = assertions_for(language);
    let guards = narrowing_guards_for(language);
    let mut total = 0usize;
    let mut guarded = 0usize;
    let mut guard_depths: Vec<usize> = Vec::new();
    let mut depth = 0usize;

    for line in span.lines() {
        let code = strip_comment(line, language);
        let trimmed = code.trim();
        // Close first: a `}` on this line ends a guard that opened above it.
        let opens = code.matches('{').count();
        let closes = code.matches('}').count();

        // Resolve the guard BEFORE the assertion: a guard opening on this line
        // covers an assertion that sits on it.
        let opens_guard = guards.iter().any(|g| trimmed.contains(g));
        let closed_on_this_line = opens_guard && opens > 0 && closes >= opens;

        if assertions.iter().any(|a| trimmed.contains(a)) {
            total += 1;
            if !guard_depths.is_empty() || closed_on_this_line {
                guarded += 1;
            }
        }

        if opens_guard && opens > closes {
            guard_depths.push(depth);
        }
        depth = depth + opens - closes.min(depth + opens);
        while guard_depths.last().is_some_and(|d| depth <= *d) {
            guard_depths.pop();
        }
    }
    (total, guarded)
}

/// The similarity floor above which an oracle is called a copy (FR-064-AC-2).
///
/// Chosen to catch the measured case — a character-for-character copy scores
/// 1.0 — while leaving room for an oracle that legitimately names the same
/// nouns as the code. Deliberately high: a false suspicion on somebody's test
/// costs more than a missed one, because this check's whole value is that a
/// reader trusts it enough to look.
pub const ORACLE_SIMILARITY_FLOOR: f64 = 0.75;

/// An oracle that resembles the implementation it judges (FR-064-AC-2).
///
/// Pass 2's highest-value manual finding: TC-1598's oracle was a
/// character-for-character copy of the code under test, **redundant branch
/// included**. It passed, forever, and replacing it with a real oracle
/// immediately exposed a genuine Windows containment gap.
///
/// A copy cannot fail: it computes the same answer the same way, so it asserts
/// that the code equals itself.
pub fn oracle_copies(pairs: &[(OracleUnderTest, &str)]) -> Vec<Suspicion> {
    let mut out = Vec::new();
    for (oracle, implementation) in pairs {
        let score = token_similarity(&oracle.text, implementation);
        if score >= ORACLE_SIMILARITY_FLOOR {
            out.push(Suspicion {
                kind: SuspicionKind::OracleResemblesImplementation
                    .as_str()
                    .to_string(),
                path: oracle.path.clone(),
                symbol: oracle.symbol.clone(),
                line: oracle.line,
                message: "the oracle closely resembles the implementation it judges; \
                          a copy computes the same answer the same way and therefore \
                          asserts only that the code equals itself"
                    .to_string(),
                evidence: format!(
                    "token similarity {score:.2} (floor {ORACLE_SIMILARITY_FLOOR:.2})"
                ),
                guidance: None,
            });
        }
    }
    out.sort_by(|a, b| (&a.path, a.line, &a.symbol).cmp(&(&b.path, b.line, &b.symbol)));
    out
}

/// Find copied test oracles in the extracted source tree (FR-064-AC-6).
///
/// This is intentionally a narrow, static join rather than a guess about every
/// expression in a test. A candidate must have all three parts:
///
/// - an explicit binding named `expected` or `oracle`;
/// - an assertion comparing a direct production-function call with that
///   binding; and
/// - a uniquely resolved production function with a directly extractable
///   return (or Rust tail) expression.
///
/// Those constraints make the two compared fragments expression-to-expression,
/// which is the population the similarity floor was measured on. Ambiguous or
/// compound shapes stand down instead of manufacturing a plausible pair.
pub fn oracle_copies_in(extraction: &SymbolExtraction) -> Vec<Suspicion> {
    let mut out = Vec::new();
    for test in &extraction.symbols {
        if !test.kind.binds_trace_ids() {
            continue;
        }
        let Some(source) = extraction.source_of(&test.path) else {
            continue;
        };
        let span = test.attached_source(source);
        let masked = crate::symbols::trace::mask_source_string_contents(&span, test.language);
        let code = strip_source_comments(&masked, test.language);
        let SpanOracleCandidates { direct, helper } = span_oracle_candidates(&code, test.language);
        if let Some(candidate) = direct {
            if let Some((implementation, expression)) =
                resolve_named_implementation(extraction, test, &candidate.function)
            {
                let score = token_similarity(&candidate.expression, &expression);
                if score >= ORACLE_SIMILARITY_FLOOR {
                    out.push(Suspicion {
                        kind: SuspicionKind::OracleResemblesImplementation
                            .as_str()
                            .to_string(),
                        path: test.path.clone(),
                        symbol: test.qualified_name.clone(),
                        line: test.leading_line + candidate.line_offset,
                        message: format!(
                            "the `{}` oracle closely resembles `{}` in {}; replace the copied calculation with an independent expectation",
                            candidate.binding, implementation.qualified_name, implementation.path
                        ),
                        evidence: format!(
                            "token similarity {score:.2} (floor {ORACLE_SIMILARITY_FLOOR:.2}); compared `{}` with `{}::{}`",
                            candidate.binding, implementation.path, implementation.qualified_name
                        ),
                        guidance: None,
                    });
                }
            }
        }

        // The pinned wild shape behind #236 uses a named helper as the oracle:
        // `prop_assert_eq!(production_call(), !oracle_helper())`. The helper
        // copied a private production predicate in another file, so neither an
        // `expected` binding nor direct call-name resolution can reach it.
        if test.language == SourceLanguage::Rust {
            if let Some(candidate) = helper {
                if let Some((helper, oracle_expression)) =
                    resolve_same_file_helper(extraction, test, &candidate.function)
                {
                    let matches: Vec<(&Symbol, String, f64)> = extraction
                        .symbols
                        .iter()
                        .filter(|symbol| {
                            symbol.language == SourceLanguage::Rust
                                && symbol.kind == SymbolKind::Function
                                && symbol.path != helper.path
                        })
                        .filter_map(|symbol| {
                            let expression = comparable_expression(extraction, symbol)?;
                            let score = token_similarity(&oracle_expression, &expression);
                            (score >= ORACLE_SIMILARITY_FLOOR)
                                .then_some((symbol, expression, score))
                        })
                        .collect();

                    // Similarity is evidence, not name resolution. More than
                    // one matching production predicate leaves the subject
                    // ambiguous, so stand down instead of guessing.
                    if let [(implementation, _, score)] = matches.as_slice() {
                        out.push(Suspicion {
                            kind: SuspicionKind::OracleResemblesImplementation
                                .as_str()
                                .to_string(),
                            path: test.path.clone(),
                            symbol: test.qualified_name.clone(),
                            line: test.leading_line + candidate.line_offset,
                            message: format!(
                                "the `{}` oracle helper closely resembles `{}` in {}; replace the copied helper with an independent expectation",
                                helper.qualified_name,
                                implementation.qualified_name,
                                implementation.path
                            ),
                            evidence: format!(
                                "token similarity {score:.2} (floor {ORACLE_SIMILARITY_FLOOR:.2}); compared helper `{}::{}` with `{}::{}`",
                                helper.path,
                                helper.qualified_name,
                                implementation.path,
                                implementation.qualified_name
                            ),
                            guidance: None,
                        });
                    }
                }
            }
        }
    }
    out.sort_by(|a, b| (&a.path, a.line, &a.symbol).cmp(&(&b.path, b.line, &b.symbol)));
    out
}

fn resolve_named_implementation<'a>(
    extraction: &'a SymbolExtraction,
    test: &Symbol,
    function: &str,
) -> Option<(&'a Symbol, String)> {
    let implementations: Vec<(&Symbol, String)> = extraction
        .symbols
        .iter()
        .filter(|symbol| {
            symbol.language == test.language
                && symbol.kind == SymbolKind::Function
                && simple_name(&symbol.qualified_name) == function
        })
        .filter_map(|symbol| {
            implementation_expression_for(extraction, symbol).map(|expression| (symbol, expression))
        })
        .collect();

    // A textual call name is not enough to disambiguate overloads or two
    // modules exporting the same function. Prefer the same file, but stand
    // down unless that produces exactly one subject.
    let same_file: Vec<_> = implementations
        .iter()
        .filter(|(symbol, _)| symbol.path == test.path)
        .collect();
    if same_file.len() == 1 {
        Some((same_file[0].0, same_file[0].1.clone()))
    } else if same_file.is_empty() && implementations.len() == 1 {
        Some((implementations[0].0, implementations[0].1.clone()))
    } else {
        None
    }
}

fn resolve_same_file_helper<'a>(
    extraction: &'a SymbolExtraction,
    test: &Symbol,
    function: &str,
) -> Option<(&'a Symbol, String)> {
    let helpers: Vec<_> = extraction
        .symbols
        .iter()
        .filter(|symbol| {
            symbol.language == test.language
                && symbol.kind == SymbolKind::Function
                && symbol.path == test.path
                && simple_name(&symbol.qualified_name) == function
        })
        .filter_map(|symbol| comparable_expression(extraction, symbol).map(|body| (symbol, body)))
        .collect();
    let [(helper, expression)] = helpers.as_slice() else {
        return None;
    };
    Some((*helper, expression.clone()))
}

fn implementation_expression_for(extraction: &SymbolExtraction, symbol: &Symbol) -> Option<String> {
    let source = extraction.source_of(&symbol.path)?;
    let span = symbol.attached_source(source);
    let masked = crate::symbols::trace::mask_source_string_contents(&span, symbol.language);
    let code = strip_source_comments(&masked, symbol.language);
    implementation_expression(&code, symbol.language)
}

fn comparable_expression(extraction: &SymbolExtraction, symbol: &Symbol) -> Option<String> {
    if let Some(expression) = implementation_expression_for(extraction, symbol) {
        return Some(expression);
    }
    if symbol.language != SourceLanguage::Rust {
        return None;
    }
    let source = extraction.source_of(&symbol.path)?;
    let span = symbol.attached_source(source);
    let masked = crate::symbols::trace::mask_source_string_contents(&span, symbol.language);
    let code = strip_source_comments(&masked, symbol.language);
    let body = code.split_once('{')?.1.rsplit_once('}')?.0.trim();

    static DECISION_IF: OnceLock<Regex> = OnceLock::new();
    let decision_if = DECISION_IF.get_or_init(|| {
        Regex::new(r"(?s)\bif\s+(.+?)\s*\{").expect("Rust decision-if pattern compiles")
    });
    if let Some(found) = decision_if.captures(body) {
        let whole = found.get(0)?;
        let condition = found.get(1)?.as_str();
        let prefix = &body[..whole.start()];
        return Some(format!("{prefix} {condition}"));
    }

    // A helper oracle may bind an intermediate and return a tail predicate.
    // Keep both: the wild copy duplicated the normalization assignment as well
    // as every branch of the predicate, which is the causal evidence.
    body.rsplit_once(';')
        .is_some_and(|(_, tail)| !tail.trim().is_empty())
        .then(|| body.to_string())
}

#[derive(Debug, Eq, PartialEq)]
struct OracleCandidate {
    binding: String,
    expression: String,
    function: String,
    line_offset: usize,
}

#[derive(Debug, Eq, PartialEq)]
struct HelperOracleCandidate {
    function: String,
    line_offset: usize,
}

#[derive(Debug, Default, Eq, PartialEq)]
struct SpanOracleCandidates {
    direct: Option<OracleCandidate>,
    helper: Option<HelperOracleCandidate>,
}

#[derive(Debug)]
struct OracleCandidateAtByte {
    binding: String,
    expression: String,
    function: String,
    byte_offset: usize,
}

impl OracleCandidateAtByte {
    fn with_line_offset(self, line_offsets: &[usize]) -> OracleCandidate {
        OracleCandidate {
            binding: self.binding,
            expression: self.expression,
            function: self.function,
            line_offset: line_offset_at(line_offsets, self.byte_offset),
        }
    }
}

#[derive(Debug)]
struct HelperOracleCandidateAtByte {
    function: String,
    byte_offset: usize,
}

impl HelperOracleCandidateAtByte {
    fn with_line_offset(self, line_offsets: &[usize]) -> HelperOracleCandidate {
        HelperOracleCandidate {
            function: self.function,
            line_offset: line_offset_at(line_offsets, self.byte_offset),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OracleBinding {
    Expected,
    Oracle,
}

impl OracleBinding {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Expected => "expected",
            Self::Oracle => "oracle",
        }
    }
}

impl TryFrom<&str> for OracleBinding {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "expected" => Ok(Self::Expected),
            "oracle" => Ok(Self::Oracle),
            _ => Err(()),
        }
    }
}

#[derive(Default)]
struct OracleAssertions<'a> {
    expected: Option<&'a str>,
    oracle: Option<&'a str>,
}

impl<'a> OracleAssertions<'a> {
    fn insert_first(&mut self, binding: OracleBinding, function: &'a str) {
        match binding {
            OracleBinding::Expected => self.expected.get_or_insert(function),
            OracleBinding::Oracle => self.oracle.get_or_insert(function),
        };
    }

    const fn get(&self, binding: OracleBinding) -> Option<&'a str> {
        match binding {
            OracleBinding::Expected => self.expected,
            OracleBinding::Oracle => self.oracle,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OracleScanEvent {
    LineIndex,
    BindingPass,
    AssertionPass,
    Join,
}

fn observed_captures_iter<'r, 'h>(
    pattern: &'r Regex,
    span: &'h str,
    event: OracleScanEvent,
    observe: &mut impl FnMut(OracleScanEvent),
) -> regex::CaptureMatches<'r, 'h> {
    observe(event);
    pattern.captures_iter(span)
}

fn observed_line_offsets(span: &str, observe: &mut impl FnMut(OracleScanEvent)) -> Vec<usize> {
    observe(OracleScanEvent::LineIndex);
    crate::parser::line_offsets(span)
}

fn span_oracle_candidates(span: &str, language: SourceLanguage) -> SpanOracleCandidates {
    span_oracle_candidates_observed(span, language, |_| {})
}

fn span_oracle_candidates_observed(
    span: &str,
    language: SourceLanguage,
    mut observe: impl FnMut(OracleScanEvent),
) -> SpanOracleCandidates {
    let direct = oracle_candidate_observed(span, language, &mut observe);
    let helper = if language == SourceLanguage::Rust {
        helper_oracle_candidate_at_byte(span)
    } else {
        None
    };
    if direct.is_none() && helper.is_none() {
        return SpanOracleCandidates::default();
    }

    let line_offsets = observed_line_offsets(span, &mut observe);
    SpanOracleCandidates {
        direct: direct.map(|candidate| candidate.with_line_offset(&line_offsets)),
        helper: helper.map(|candidate| candidate.with_line_offset(&line_offsets)),
    }
}

fn rust_helper_assertion_pattern() -> &'static Regex {
    static RUST_HELPER_ASSERTION: OnceLock<Regex> = OnceLock::new();
    RUST_HELPER_ASSERTION.get_or_init(|| {
        Regex::new(
            r"(?s)\b(?:assert_eq|prop_assert_eq)!\s*\([^;]*?,\s*!?\s*([A-Za-z_][A-Za-z0-9_:]*)\s*\(",
        )
        .expect("Rust helper-oracle assertion pattern compiles")
    })
}

fn helper_oracle_candidate_at_byte(span: &str) -> Option<HelperOracleCandidateAtByte> {
    let found = rust_helper_assertion_pattern().captures(span)?;
    let whole = found.get(0)?;
    Some(HelperOracleCandidateAtByte {
        function: found.get(1)?.as_str().rsplit("::").next()?.to_string(),
        byte_offset: whole.start(),
    })
}

#[cfg(test)]
fn helper_oracle_candidate(span: &str, line_offsets: &[usize]) -> Option<HelperOracleCandidate> {
    helper_oracle_candidate_at_byte(span).map(|candidate| candidate.with_line_offset(line_offsets))
}

#[cfg(test)]
fn oracle_candidate(
    span: &str,
    language: SourceLanguage,
    line_offsets: &[usize],
) -> Option<OracleCandidate> {
    oracle_candidate_observed(span, language, &mut |_| {})
        .map(|candidate| candidate.with_line_offset(line_offsets))
}

fn oracle_patterns(language: SourceLanguage) -> (&'static Regex, &'static Regex) {
    static RUST_BINDING: OnceLock<Regex> = OnceLock::new();
    static TS_BINDING: OnceLock<Regex> = OnceLock::new();
    static PYTHON_BINDING: OnceLock<Regex> = OnceLock::new();
    static RUST_ASSERTION: OnceLock<Regex> = OnceLock::new();
    static TS_ASSERTION: OnceLock<Regex> = OnceLock::new();
    static PYTHON_ASSERTION: OnceLock<Regex> = OnceLock::new();

    match language {
        SourceLanguage::Rust => (
            RUST_BINDING.get_or_init(|| {
                Regex::new(r"(?s)\blet\s+(expected|oracle)(?:\s*:[^=;]+)?\s*=\s*(.+?);")
                    .expect("Rust oracle-binding pattern compiles")
            }),
            RUST_ASSERTION.get_or_init(|| {
                Regex::new(
                    r"(?s)\bassert_eq!\s*\(\s*([A-Za-z_][A-Za-z0-9_:]*)\s*\([^;]*?\)\s*,\s*(expected|oracle)\s*\)",
                )
                .expect("Rust oracle-assertion pattern compiles")
            }),
        ),
        SourceLanguage::Typescript => (
            TS_BINDING.get_or_init(|| {
                Regex::new(r"(?s)\b(?:const|let)\s+(expected|oracle)(?:\s*:[^=;]+)?\s*=\s*(.+?);")
                    .expect("TypeScript oracle-binding pattern compiles")
            }),
            TS_ASSERTION.get_or_init(|| {
                Regex::new(
                    r"(?s)\bexpect\s*\(\s*([A-Za-z_$][A-Za-z0-9_$]*)\s*\([^;]*?\)\s*\)\s*\.\s*(?:toBe|toEqual)\s*\(\s*(expected|oracle)\s*\)",
                )
                .expect("TypeScript oracle-assertion pattern compiles")
            }),
        ),
        SourceLanguage::Python => (
            PYTHON_BINDING.get_or_init(|| {
                Regex::new(r"(?m)^\s*(expected|oracle)(?:\s*:[^=\n]+)?\s*=\s*(.+?)\s*$")
                    .expect("Python oracle-binding pattern compiles")
            }),
            PYTHON_ASSERTION.get_or_init(|| {
                Regex::new(
                    r"(?m)^\s*assert\s+([A-Za-z_][A-Za-z0-9_]*)\s*\([^\n]*?\)\s*==\s*(expected|oracle)\b",
                )
                .expect("Python oracle-assertion pattern compiles")
            }),
        ),
    }
}

fn oracle_candidate_observed(
    span: &str,
    language: SourceLanguage,
    observe: &mut impl FnMut(OracleScanEvent),
) -> Option<OracleCandidateAtByte> {
    let (binding_re, assertion_re) = oracle_patterns(language);

    let mut assignments =
        observed_captures_iter(binding_re, span, OracleScanEvent::BindingPass, observe).peekable();
    assignments.peek()?;

    // Keep the first assertion for each binding name. The former nested scan
    // selected that same assertion, but restarted at byte zero for every
    // assignment. One indexed pass preserves the selection rule in linear
    // time, including when bindings are repeated or assertions are reordered.
    let mut assertions = OracleAssertions::default();
    for capture in
        observed_captures_iter(assertion_re, span, OracleScanEvent::AssertionPass, observe)
    {
        observe(OracleScanEvent::Join);
        if let (Some(function), Some(binding)) = (capture.get(1), capture.get(2)) {
            if let Ok(binding) = OracleBinding::try_from(binding.as_str()) {
                assertions.insert_first(binding, function.as_str());
            }
        }
    }

    for assignment in assignments {
        observe(OracleScanEvent::Join);
        let binding = OracleBinding::try_from(assignment.get(1)?.as_str()).ok()?;
        let Some(function) = assertions.get(binding) else {
            continue;
        };
        return Some(OracleCandidateAtByte {
            binding: binding.as_str().to_string(),
            expression: assignment.get(2)?.as_str().trim().to_string(),
            function: function.rsplit("::").next()?.to_string(),
            byte_offset: assignment.get(0)?.start(),
        });
    }
    None
}

fn line_offset_at(line_offsets: &[usize], byte_offset: usize) -> usize {
    line_offsets
        .partition_point(|&line_start| line_start <= byte_offset)
        .saturating_sub(1)
}

/// Remove comments after strings have been masked, preserving line structure
/// so a candidate's offset still maps to its source line.
fn strip_source_comments(span: &str, language: SourceLanguage) -> String {
    let chars: Vec<char> = span.chars().collect();
    let mut out = String::with_capacity(span.len());
    let mut i = 0usize;
    let mut block = false;
    while i < chars.len() {
        let c = chars[i];
        if block {
            if c == '*' && chars.get(i + 1) == Some(&'/') {
                out.push_str("  ");
                i += 2;
                block = false;
            } else {
                out.push(if c == '\n' { '\n' } else { ' ' });
                i += 1;
            }
            continue;
        }
        if language != SourceLanguage::Python && c == '/' && chars.get(i + 1) == Some(&'*') {
            out.push_str("  ");
            i += 2;
            block = true;
            continue;
        }
        let line_comment = (language == SourceLanguage::Python && c == '#')
            || (language != SourceLanguage::Python && c == '/' && chars.get(i + 1) == Some(&'/'));
        if line_comment {
            while i < chars.len() && chars[i] != '\n' {
                out.push(' ');
                i += 1;
            }
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

fn simple_name(qualified_name: &str) -> &str {
    qualified_name
        .rsplit([':', '.'])
        .find(|part| !part.is_empty())
        .unwrap_or(qualified_name)
}

fn implementation_expression(span: &str, language: SourceLanguage) -> Option<String> {
    match language {
        SourceLanguage::Rust => {
            let body = span.split_once('{')?.1.rsplit_once('}')?.0.trim();
            if let Some(returned) = body.strip_prefix("return ") {
                return Some(returned.trim_end_matches(';').trim().to_string());
            }
            if body.contains(';') {
                return None;
            }
            Some(body.to_string())
        }
        SourceLanguage::Python | SourceLanguage::Typescript => {
            let body = match language {
                SourceLanguage::Python => span.split_once('\n')?.1,
                SourceLanguage::Typescript => span.split_once('{')?.1.rsplit_once('}')?.0,
                SourceLanguage::Rust => unreachable!(),
            };
            let returns: Vec<&str> = body
                .lines()
                .filter_map(|line| line.trim().strip_prefix("return "))
                .collect();
            (returns.len() == 1).then(|| returns[0].trim_end_matches(';').trim().to_string())
        }
    }
}

/// An oracle to judge, and where it came from.
#[derive(Debug, Clone)]
pub struct OracleUnderTest {
    pub path: String,
    pub symbol: String,
    pub line: usize,
    pub text: String,
}

/// Jaccard similarity over identifier-ish tokens.
///
/// Not a diff and not an edit distance: reordering a copied expression should
/// not disguise it, and reformatting should not create a suspicion. Punctuation
/// and keywords are dropped, because two pieces of Rust share those whatever
/// they do.
pub fn token_similarity(left: &str, right: &str) -> f64 {
    let a = tokens(left);
    let b = tokens(right);
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let intersection = a.intersection(&b).count() as f64;
    let union = a.union(&b).count() as f64;
    intersection / union
}

fn tokens(text: &str) -> std::collections::BTreeSet<String> {
    const NOISE: &[&str] = &[
        "let", "mut", "if", "else", "match", "fn", "return", "self", "true", "false", "and", "the",
        "a", "an", "is", "of", "to", "for", "in", "with", "that",
    ];
    text.split(|c: char| !c.is_alphanumeric() && c != '_')
        .map(|t| t.trim().to_ascii_lowercase())
        .filter(|t| t.len() > 1 && !NOISE.contains(&t.as_str()))
        .collect()
}

fn suspicion(symbol: &Symbol, kind: SuspicionKind, message: &str, evidence: String) -> Suspicion {
    Suspicion {
        kind: kind.as_str().to_string(),
        path: symbol.path.clone(),
        symbol: symbol.qualified_name.clone(),
        line: symbol.line,
        message: message.to_string(),
        evidence,
        guidance: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbols::extract_file;
    use crate::traceability::SourceLanguage;
    use ix_trace_rs::trace;

    fn extract(body: &str) -> SymbolExtraction {
        extract_file("src/lib.rs", SourceLanguage::Rust, body)
    }

    fn extract_ts(body: &str) -> SymbolExtraction {
        extract_file("tests/thing.test.ts", SourceLanguage::Typescript, body)
    }

    fn merge(mut left: SymbolExtraction, right: SymbolExtraction) -> SymbolExtraction {
        left.symbols.extend(right.symbols);
        left.files.extend(right.files);
        left.diagnostics.extend(right.diagnostics);
        left.symbols.sort_by(|a, b| {
            (&a.path, a.line, &a.qualified_name).cmp(&(&b.path, b.line, &b.qualified_name))
        });
        left.files.sort_by(|a, b| a.path.cmp(&b.path));
        left
    }

    /// Frozen pre-#412 direct-candidate algorithm used only for differential
    /// evidence. Keep the nested scan here: a regression test must preserve
    /// the old selection semantics without putting its complexity back on the
    /// production path.
    fn reference_oracle_candidate(span: &str, language: SourceLanguage) -> Option<OracleCandidate> {
        let (binding_re, assertion_re) = oracle_patterns(language);
        for assignment in binding_re.captures_iter(span) {
            let binding = assignment.get(1)?.as_str();
            let Some(assertion) = assertion_re.captures_iter(span).find(|capture| {
                capture
                    .get(2)
                    .is_some_and(|found| found.as_str() == binding)
            }) else {
                continue;
            };
            return Some(OracleCandidate {
                binding: binding.to_string(),
                expression: assignment.get(2)?.as_str().trim().to_string(),
                function: assertion.get(1)?.as_str().rsplit("::").next()?.to_string(),
                line_offset: span[..assignment.get(0)?.start()].matches('\n').count(),
            });
        }
        None
    }

    fn reference_helper_oracle_candidate(span: &str) -> Option<HelperOracleCandidate> {
        let found = rust_helper_assertion_pattern().captures(span)?;
        let whole = found.get(0)?;
        Some(HelperOracleCandidate {
            function: found.get(1)?.as_str().rsplit("::").next()?.to_string(),
            line_offset: span[..whole.start()].matches('\n').count(),
        })
    }

    fn generated_direct_span(language: SourceLanguage, newline: &str, scenario: usize) -> String {
        let prefix = match language {
            SourceLanguage::Rust | SourceLanguage::Typescript => "// café 雪",
            SourceLanguage::Python => "# café 雪",
        };
        let binding = |name: &str, expression: &str| match language {
            SourceLanguage::Rust => format!("let {name} = {expression};"),
            SourceLanguage::Typescript => format!("const {name} = {expression};"),
            SourceLanguage::Python => format!("{name} = {expression}"),
        };
        let assertion = |function: &str, name: &str| match language {
            SourceLanguage::Rust => format!("assert_eq!({function}(input), {name})"),
            SourceLanguage::Typescript => {
                format!("expect({function}(input)).toEqual({name})")
            }
            SourceLanguage::Python => format!("assert {function}(input) == {name}"),
        };

        let lines = match scenario {
            // Repeated binding name with assertions in the opposite order.
            0 => vec![
                prefix.to_string(),
                binding("expected", "first_expression()"),
                binding("expected", "repeated_expression()"),
                binding("oracle", "oracle_expression()"),
                assertion("oracle_subject", "oracle"),
                assertion("expected_subject", "expected"),
            ],
            // The first binding is unmatched, so selection must advance to
            // the first binding for which an assertion exists.
            1 => vec![
                prefix.to_string(),
                binding("expected", "unmatched_expression()"),
                binding("oracle", "matched_expression()"),
                assertion("oracle_subject", "oracle"),
            ],
            // The regex grammar permits an assertion before its binding; the
            // optimized join must retain the global first-match behavior.
            2 => vec![
                prefix.to_string(),
                assertion("expected_subject", "expected"),
                binding("expected", "later_expression()"),
            ],
            _ => unreachable!("generated scenario is bounded by the test"),
        };
        format!("{}{newline}", lines.join(newline))
    }

    #[test]
    fn suspicion_kinds_emit_complete_exclusive_guidance() {
        for kind in ["oracle-resembles-implementation", "vacuous-under-guard"] {
            let suspicion = Suspicion {
                kind: kind.to_string(),
                symbol: "property_holds".to_string(),
                path: "tests/property.rs".to_string(),
                line: 17,
                message: "causal claim".to_string(),
                evidence: "measured evidence".to_string(),
                guidance: None,
            };
            let guidance = suspicion.structured_guidance();
            assert!(!guidance.subject.trim().is_empty(), "{kind}: subject");
            assert!(
                !guidance.change_target.trim().is_empty(),
                "{kind}: change target"
            );
            let wire = serde_json::to_value(guidance).expect("guidance serializes");
            let actions = ["remedy", "next_diagnostic_step"]
                .iter()
                .filter(|key| wire.get(**key).is_some())
                .count();
            assert_eq!(actions, 1, "{kind}: exactly one action: {wire}");
        }
    }

    #[trace("TC-1002", "FR-064-AC-1")]
    // a narrowing guard that opens and closes on ONE line (CR-102)
    // still guards the assertion sitting on it.
    #[test]
    fn tc1002_a_single_line_guard_guards_the_assertion_on_it() {
        // Identical in meaning to TC-997's multi-line spelling. It reported
        // nothing until CR-102: the assertion was tested before the guard, and
        // the guard was pushed only when it stayed open past the line.
        let vacuous = extract(
            "#[cfg(test)]\nmod tests {\n\
             #[test]\n\
             fn tc1596_property() {\n\
             for sample in samples() {\n\
             if let Ok(Some(v)) = parse(sample) { prop_assert_eq!(v.len(), 3); }\n\
             }\n\
             }\n\
             }\n",
        );
        let found = vacuous_property_suites(&vacuous);
        assert_eq!(found.len(), 1, "{found:#?}");
        assert_eq!(found[0].kind, "vacuous-under-guard");
        assert!(
            found[0].evidence.contains("1 of 1"),
            "{}",
            found[0].evidence
        );

        // The control: same one-line brace shape, no narrowing guard. A `for`
        // body is not a guard, so its assertion is unguarded and nothing fires.
        let sound = extract(
            "#[cfg(test)]\nmod tests {\n\
             #[test]\n\
             fn tc1597_property() {\n\
             for sample in samples() { assert_eq!(parse(sample).len(), 3); }\n\
             }\n\
             }\n",
        );
        assert!(
            vacuous_property_suites(&sound).is_empty(),
            "{:#?}",
            vacuous_property_suites(&sound)
        );
    }

    #[trace("TC-1003", "FR-064-AC-5")]
    // an arrow function is not a narrowing guard: a TypeScript (CR-102)
    // suite of ordinary `it(... () => {...})` tests reports nothing.
    #[test]
    fn tc1003_typescript_arrow_functions_are_not_guards() {
        // Verbatim shape of the 549-of-551 false positive on agent-ix/quoin:
        // every vitest body is `() => {`, which the shared Rust guard list read
        // as a `match` arm.
        //
        // The fixture lives in a `.ts.txt` file rather than a string literal
        // here **because this check reads raw text**: inlined, its `() => {`
        // would sit in a Rust symbol, Rust's guard list would match it, and
        // this very test would be reported as vacuous — the same
        // wrong-language misread, one level up. `.txt` binds to no
        // `SourceLanguage`, so the walk never treats it as source.
        let ts = extract_ts(include_str!(
            "../tests/fixtures/skeptic/vitest_arrow_suite.ts.txt"
        ));
        assert!(
            !ts.symbols.is_empty(),
            "the TypeScript extractor bound nothing, so this asserts nothing"
        );
        assert_eq!(
            vacuous_property_suites(&ts),
            vec![],
            "an arrow function is not a narrowing guard"
        );
    }

    #[trace("TC-1004", "FR-064-AC-5")]
    // a comment is prose, not an oracle: code quoted inside one (CR-102)
    // counts as neither an assertion nor a guard.
    #[test]
    fn tc1004_code_quoted_in_a_comment_is_not_code() {
        // Both the guard and the assertion here exist ONLY in a comment. Before
        // CR-102 this symbol reported `1 of 1 assertions guarded` and was
        // called vacuous, which is how this crate's own TC-1003 got reported.
        let commented = extract(
            "#[cfg(test)]\nmod tests {\n\
             #[test]\n\
             fn tc999_documented() {\n\
             // if let Some(x) = y { assert_eq!(x, 1); }\n\
             let outcome = run();\n\
             }\n\
             }\n",
        );
        assert_eq!(
            vacuous_property_suites(&commented),
            vec![],
            "a commented-out guard and assertion are not a vacuous suite"
        );

        // The control: the same two tokens as real code DO report, so this is
        // measuring comment-stripping rather than the absence of a match.
        let real = extract(
            "#[cfg(test)]\nmod tests {\n\
             #[test]\n\
             fn tc999_documented() {\n\
             if let Some(x) = y { assert_eq!(x, 1); }\n\
             let outcome = run();\n\
             }\n\
             }\n",
        );
        assert_eq!(vacuous_property_suites(&real).len(), 1);
    }

    #[trace("TC-997", "FR-064-AC-1")]
    // a suite whose every assertion sits behind a (CR-100)
    // narrowing guard is reported; one with an unguarded assertion is not.
    #[test]
    fn tc997_assertions_behind_a_guard_are_suspect() {
        // The measured shape: 4,000 samples, the assertion inside the arm that
        // 2.3% of them entered. Green throughout.
        let vacuous = extract(
            "#[cfg(test)]\nmod tests {\n\
             #[test]\n\
             fn tc1596_property() {\n\
             for sample in samples() {\n\
             if let Ok(Some(v)) = parse(sample) {\n\
             assert_eq!(v.len(), 3);\n\
             }\n\
             }\n\
             }\n\
             }\n",
        );
        let found = vacuous_property_suites(&vacuous);
        assert_eq!(found.len(), 1, "{found:#?}");
        assert_eq!(found[0].kind, "vacuous-under-guard");
        assert!(
            found[0].evidence.contains("1 of 1"),
            "{}",
            found[0].evidence
        );

        // One unguarded assertion is enough: every sample is checked by
        // something, which is the property that makes the suite non-vacuous.
        let sound = extract(
            "#[cfg(test)]\nmod tests {\n\
             #[test]\n\
             fn tc1597_property() {\n\
             for sample in samples() {\n\
             let parsed = parse(sample);\n\
             assert!(parsed.is_ok() || parsed.is_err());\n\
             if let Ok(Some(v)) = parsed {\n\
             assert_eq!(v.len(), 3);\n\
             }\n\
             }\n\
             }\n\
             }\n",
        );
        assert!(
            vacuous_property_suites(&sound).is_empty(),
            "{:#?}",
            vacuous_property_suites(&sound)
        );
    }

    #[trace("TC-998", "FR-064-AC-1")]
    // a test with no assertion MACRO is not a finding: (CR-100)
    // in Rust a test fails on panic, so absence of a macro is not absence of
    // an oracle. Measured, that rule was wrong 12 times out of 12 sampled.
    #[test]
    fn tc998_absence_of_an_assertion_macro_is_not_a_finding() {
        // The oracle is the absence of a panic — a real oracle.
        let never_panics = extract(
            "#[cfg(test)]\nmod tests {\n\
             #[test]\n\
             fn never_panics_on_arbitrary_utf8() {\n\
             let _ = parse_document(\"x\");\n\
             }\n\
             }\n",
        );
        assert!(vacuous_property_suites(&never_panics).is_empty());

        // The assertion is at COMPILE time: it cannot fail at runtime because
        // it already failed to build.
        let compile_time = extract(
            "#[cfg(test)]\nmod tests {\n\
             #[test]\n\
             fn document_is_send_and_sync() {\n\
             fn assert_send_sync<T: Send + Sync>() {}\n\
             assert_send_sync::<QuireDocument>();\n\
             }\n\
             }\n",
        );
        assert!(vacuous_property_suites(&compile_time).is_empty());

        // Production code is not a test and is not judged: only symbols that
        // bind trace ids are candidates (CR-061).
        let production = extract("pub fn parse(s: &str) -> usize {\n    s.len()\n}\n");
        assert!(vacuous_property_suites(&production).is_empty());
    }

    #[trace("TC-999", "FR-064-AC-2")]
    // an oracle that is a copy of the code under test (CR-100)
    // is reported; one that judges the same subject differently is not.
    #[test]
    fn tc999_an_oracle_that_copies_the_implementation_is_suspect() {
        let implementation = "let normalized = path.strip_prefix(root).unwrap_or(path); \
             normalized.components().all(|c| c != Component::ParentDir)";

        // The measured case: character-for-character, redundant branch included.
        let copied = OracleUnderTest {
            path: "tests/containment.rs".into(),
            symbol: "tc1598_containment".into(),
            line: 12,
            text: implementation.to_string(),
        };
        let found = oracle_copies(&[(copied, implementation)]);
        assert_eq!(found.len(), 1, "{found:#?}");
        assert_eq!(found[0].kind, "oracle-resembles-implementation");
        assert!(
            found[0].evidence.contains("similarity 1.00"),
            "{}",
            found[0].evidence
        );

        // A real oracle names the same subject and judges it independently.
        let real = OracleUnderTest {
            path: "tests/containment.rs".into(),
            symbol: "tc1598_containment".into(),
            line: 12,
            text: "the resolved path stays inside the workspace root on every \
                   platform, including Windows UNC and short-name forms"
                .to_string(),
        };
        assert!(
            oracle_copies(&[(real, implementation)]).is_empty(),
            "an independent oracle is not a copy"
        );
    }

    #[trace("TC-1061", "FR-064-AC-6")]
    #[test]
    fn tc1061_explicit_oracle_bindings_join_to_production_expressions() {
        let cases = [
            (
                SourceLanguage::Rust,
                "src/lib.rs",
                r#"pub fn is_contained(path: &str) -> bool {
    !path.starts_with('/') && !path.contains("..") && !path.contains('\0')
}

#[test]
fn covers() {
    let path = "a/b";
    let expected = !path.starts_with('/') && !path.contains("..") && !path.contains('\0');
    assert_eq!(is_contained(path), expected);
}
"#,
                r#"pub fn is_contained(path: &str) -> bool {
    !path.starts_with('/') && !path.contains("..") && !path.contains('\0')
}

#[test]
fn covers() {
    let path = "a/b";
    let expected = true;
    assert_eq!(is_contained(path), expected);
}
"#,
            ),
            (
                SourceLanguage::Python,
                "src/lib.py",
                r#"def is_contained(path: str) -> bool:
    return not path.startswith("/") and ".." not in path and "\0" not in path

def test_covers():
    path = "a/b"
    expected = not path.startswith("/") and ".." not in path and "\0" not in path
    assert is_contained(path) == expected
"#,
                r#"def is_contained(path: str) -> bool:
    return not path.startswith("/") and ".." not in path and "\0" not in path

def test_covers():
    path = "a/b"
    expected = True
    assert is_contained(path) == expected
"#,
            ),
            (
                SourceLanguage::Typescript,
                "src/lib.test.ts",
                r#"export function isContained(path: string): boolean {
  return !path.startsWith("/") && !path.includes("..") && !path.includes("\0");
}

test("covers", () => {
  const path = "a/b";
  const expected = !path.startsWith("/") && !path.includes("..") && !path.includes("\0");
  expect(isContained(path)).toBe(expected);
});
"#,
                r#"export function isContained(path: string): boolean {
  return !path.startsWith("/") && !path.includes("..") && !path.includes("\0");
}

test("covers", () => {
  const path = "a/b";
  const expected = true;
  expect(isContained(path)).toBe(expected);
});
"#,
            ),
        ];

        for (language, path, copied, independent) in cases {
            let copied = extract_file(path, language, copied);
            let found = oracle_copies_in(&copied);
            assert_eq!(found.len(), 1, "{language:?}: {found:#?}");
            assert_eq!(found[0].kind, "oracle-resembles-implementation");
            assert!(found[0].evidence.contains("similarity 1.00"));
            assert!(found[0].evidence.contains(path));
            assert!(
                found[0].message.contains("independent expectation"),
                "{}",
                found[0].message
            );

            let independent = extract_file(path, language, independent);
            assert_eq!(
                oracle_copies_in(&independent),
                vec![],
                "an independently stated expectation is not a copy in {language:?}"
            );
        }

        let ambiguous = extract(
            r#"mod left {
    pub fn normalize(value: &str) -> bool { value.starts_with("ok") }
}
mod right {
    pub fn normalize(value: &str) -> bool { value.starts_with("ok") }
}
#[test]
fn covers() {
    let value = "okay";
    let expected = value.starts_with("ok");
    assert_eq!(normalize(value), expected);
}
"#,
        );
        assert_eq!(
            oracle_copies_in(&ambiguous),
            vec![],
            "a textual call name that resolves to two functions is not a pair"
        );

        let quoted = extract(
            r##"pub fn normalize(value: &str) -> bool { value.starts_with("ok") }
#[test]
fn documents_the_bad_shape() {
    let fixture = r#"let expected = value.starts_with("ok");
assert_eq!(normalize(value), expected);"#;
    // let expected = value.starts_with("ok");
    // assert_eq!(normalize(value), expected);
    assert!(!fixture.is_empty());
}
"##,
        );
        assert_eq!(
            oracle_copies_in(&quoted),
            vec![],
            "quoted fixture text and comments are not executable oracle code"
        );
    }

    #[trace("TC-1810", "NFR-023-AC-1")]
    #[test]
    fn tc1810_oracle_candidate_scan_counts_are_bounded_by_matches_not_nesting() {
        #[derive(Default)]
        struct Counts {
            line_indexes: usize,
            binding_passes: usize,
            assertion_passes: usize,
            joins: usize,
        }

        fn measured(span: &str) -> (SpanOracleCandidates, Counts) {
            let mut counts = Counts::default();
            let candidates =
                span_oracle_candidates_observed(span, SourceLanguage::Rust, |event| match event {
                    OracleScanEvent::LineIndex => counts.line_indexes += 1,
                    OracleScanEvent::BindingPass => counts.binding_passes += 1,
                    OracleScanEvent::AssertionPass => counts.assertion_passes += 1,
                    OracleScanEvent::Join => counts.joins += 1,
                });
            (candidates, counts)
        }

        let zero = "// café\r\nlet unrelated = value();\r\n";
        let assertion_without_binding = "assert_eq!(subject(), expected);\n";
        let helper_only = "prop_assert_eq!(subject(), oracle_helper());\n";
        let direct_and_helper = "let expected = baseline();\nassert_eq!(subject(), expected);\nprop_assert_eq!(subject(), oracle_helper());\n";
        let one = "let expected = baseline();\nassert_eq!(subject(), expected);\n";
        let mut many = String::from("// café 雪\r\n");
        for index in 0..128 {
            many.push_str(&format!("let expected = candidate_{index}();\r\n"));
        }
        many.push_str("let oracle = matched();\r\nassert_eq!(subject(), oracle);\r\n");

        for (span, has_direct_candidate, has_helper_candidate) in [
            (zero, false, false),
            (assertion_without_binding, false, false),
            (helper_only, false, true),
            (direct_and_helper, true, true),
            (one, true, false),
            (many.as_str(), true, false),
        ] {
            let (candidates, counts) = measured(span);
            let (binding_re, assertion_re) = oracle_patterns(SourceLanguage::Rust);
            let matches =
                binding_re.captures_iter(span).count() + assertion_re.captures_iter(span).count();
            assert_eq!(
                counts.line_indexes,
                usize::from(has_direct_candidate || has_helper_candidate),
                "{span}"
            );
            assert_eq!(counts.binding_passes, 1, "{span}");
            assert_eq!(
                counts.assertion_passes,
                usize::from(has_direct_candidate),
                "{span}"
            );
            assert!(counts.joins <= matches, "{span}");
            assert_eq!(candidates.direct.is_some(), has_direct_candidate, "{span}");
            assert_eq!(candidates.helper.is_some(), has_helper_candidate, "{span}");
            if !has_direct_candidate && !has_helper_candidate {
                assert_eq!(candidates, SpanOracleCandidates::default());
            }
        }

        let (_, one_counts) = measured(one);
        let (many_candidates, many_counts) = measured(&many);
        assert_eq!(one_counts.binding_passes, many_counts.binding_passes);
        assert_eq!(one_counts.assertion_passes, many_counts.assertion_passes);
        assert_eq!(one_counts.line_indexes, many_counts.line_indexes);
        assert_eq!(
            many_candidates.direct.expect("many-span candidate").binding,
            "oracle"
        );
    }

    #[trace("TC-1811", "NFR-023-AC-2", "NFR-023-AC-4")]
    #[test]
    fn tc1811_oracle_scan_source_shape_prevents_rescans_and_unbounded_join_state() {
        let source = include_str!("skeptic.rs");
        let tests_start = source
            .rfind("\nmod tests {")
            .expect("test module follows production source");
        let production = &source[..tests_start];
        let span_scan = production
            .split("fn span_oracle_candidates_observed")
            .nth(1)
            .and_then(|tail| tail.split("fn rust_helper_assertion_pattern").next())
            .expect("span scan source is delimited");
        let direct_scan = production
            .split("fn oracle_candidate_observed")
            .nth(1)
            .and_then(|tail| tail.split("fn line_offset_at").next())
            .expect("direct scan source is delimited");
        let traversal_wrapper = production
            .split("fn observed_captures_iter")
            .nth(1)
            .and_then(|tail| tail.split("fn span_oracle_candidates").next())
            .expect("measured traversal wrapper source is delimited");
        let assertion_pass = direct_scan
            .find("OracleScanEvent::AssertionPass")
            .expect("assertion traversal remains measured");
        let binding_loop = direct_scan
            .find("for assignment in assignments")
            .expect("binding join remains explicit");

        assert_eq!(span_scan.matches("observed_line_offsets(span").count(), 1);
        assert_eq!(
            production
                .matches("crate::parser::line_offsets(span)")
                .count(),
            1,
            "raw line-index construction is confined to its measured wrapper"
        );
        assert!(!production.contains(".matches('\\n').count()"));
        assert!(
            !direct_scan.contains(".captures_iter("),
            "raw direct-candidate traversals must go through the measured wrapper"
        );
        assert_eq!(traversal_wrapper.matches(".captures_iter(span)").count(), 1);
        assert_eq!(direct_scan.matches("observed_captures_iter(").count(), 2);
        assert!(
            assertion_pass < binding_loop,
            "assertions are indexed before the binding join"
        );
        assert!(direct_scan.contains("OracleAssertions"));
        assert!(direct_scan.contains("assertions.get(binding)"));
        assert!(!direct_scan.contains("BTreeMap"));
    }

    #[trace("TC-1812", "NFR-023-AC-3")]
    #[test]
    fn tc1812_oracle_candidates_match_the_frozen_reference_across_edge_cases() {
        #[derive(Deserialize)]
        struct CandidateFixture {
            name: String,
            issue_ref: String,
            tags: Vec<String>,
            candidates: Vec<CandidateCase>,
            helper: HelperCase,
        }
        #[derive(Deserialize)]
        struct CandidateCase {
            language: SourceLanguage,
            span: String,
            binding: String,
            expression: String,
            function: String,
            line_offset: usize,
        }
        #[derive(Deserialize)]
        struct HelperCase {
            span: String,
            function: String,
            line_offset: usize,
        }

        let fixture: CandidateFixture = serde_json::from_str(include_str!(
            "../tests/fixtures/corpus_cases/issue_412_oracle_selection.json"
        ))
        .expect("issue #412 fixture is valid");
        assert_eq!(fixture.issue_ref, "agent-ix/quire-rs#412");
        assert!(!fixture.name.trim().is_empty());
        assert!(fixture.tags.iter().any(|tag| tag == "TC-1812"));

        for case in fixture.candidates {
            let line_offsets = crate::parser::line_offsets(&case.span);
            let candidate = oracle_candidate(&case.span, case.language, &line_offsets)
                .unwrap_or_else(|| panic!("{:?}: expected an oracle candidate", case.language));
            assert_eq!(candidate.binding, case.binding, "{:?}", case.language);
            assert_eq!(candidate.expression, case.expression, "{:?}", case.language);
            assert_eq!(candidate.function, case.function, "{:?}", case.language);
            assert_eq!(
                candidate.line_offset, case.line_offset,
                "{:?}",
                case.language
            );
        }

        let line_offsets = crate::parser::line_offsets(&fixture.helper.span);
        let candidate = helper_oracle_candidate(&fixture.helper.span, &line_offsets)
            .expect("expected a helper-oracle candidate");
        assert_eq!(candidate.function, fixture.helper.function);
        assert_eq!(candidate.line_offset, fixture.helper.line_offset);

        for language in [
            SourceLanguage::Rust,
            SourceLanguage::Python,
            SourceLanguage::Typescript,
        ] {
            for newline in ["\n", "\r\n"] {
                for scenario in 0..3 {
                    let span = generated_direct_span(language, newline, scenario);
                    let expected = reference_oracle_candidate(&span, language);
                    let line_offsets = crate::parser::line_offsets(&span);
                    let actual = oracle_candidate(&span, language, &line_offsets);
                    assert_eq!(
                        actual, expected,
                        "{language:?}, newline={newline:?}, scenario={scenario}"
                    );
                }
            }
        }

        for newline in ["\n", "\r\n"] {
            let span = [
                "// café 雪",
                "let input = sample();",
                "",
                "prop_assert_eq!(subject(input), !module::oracle_helper(input));",
            ]
            .join(newline);
            let expected = reference_helper_oracle_candidate(&span);
            let line_offsets = crate::parser::line_offsets(&span);
            let actual = helper_oracle_candidate(&span, &line_offsets);
            assert_eq!(actual, expected, "helper newline={newline:?}");
        }
    }

    #[trace("TC-1080", "FR-064-AC-7")]
    #[test]
    fn tc1080_the_pinned_helper_oracle_copy_is_detected_without_guessing() {
        const IMPLEMENTATION: &str =
            include_str!("../tests/fixtures/skeptic/tc1598_validate_artifact_path.rs.txt");
        const WILD_COPY: &str =
            include_str!("../tests/fixtures/skeptic/tc1598_copied_oracle.rs.txt");
        let extraction = merge(
            extract_file(
                "crates/filament-shell/src/open.rs",
                SourceLanguage::Rust,
                IMPLEMENTATION,
            ),
            extract_file(
                "crates/filament-shell/tests/property_suite.rs",
                SourceLanguage::Rust,
                WILD_COPY,
            ),
        );

        let found = oracle_copies_in(&extraction);
        assert_eq!(found.len(), 1, "wild 59a180a7 slice: {found:#?}");
        let finding = &found[0];
        assert_eq!(finding.kind, "oracle-resembles-implementation");
        assert_eq!(
            finding.path,
            "crates/filament-shell/tests/property_suite.rs"
        );
        assert_eq!(
            finding.symbol,
            "tc1598_artifact_acceptance_matches_the_containment_rule"
        );
        assert_eq!(
            finding.line, 17,
            "assertion locus is recovered: {finding:#?}"
        );
        for detail in [
            "escapes_the_root",
            "validate_artifact_path",
            "crates/filament-shell/src/open.rs",
            "independent expectation",
        ] {
            assert!(
                finding.message.contains(detail),
                "message names `{detail}`: {}",
                finding.message
            );
        }
        assert!(
            finding.evidence.contains("similarity 1.00"),
            "{}",
            finding.evidence
        );

        // The post-review oracle asks the containment question independently.
        let independent = WILD_COPY.replacen(
            r#"let normalized = artifact.replace('\\', "/");
    normalized.is_empty()
        || normalized.starts_with('/')
        || normalized.contains('\0')
        || normalized
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")"#,
            r#"let mut depth = 0isize;
    for segment in artifact.replace('\\', "/").split('/') {
        depth += if segment == ".." { -1 } else { 1 };
        if depth < 0 { return true; }
    }
    false"#,
            1,
        );
        let control = merge(
            extract_file("src/open.rs", SourceLanguage::Rust, IMPLEMENTATION),
            extract_file(
                "tests/property_suite.rs",
                SourceLanguage::Rust,
                &independent,
            ),
        );
        assert_eq!(
            oracle_copies_in(&control),
            vec![],
            "an independently implemented helper is not a copy"
        );

        // A similar helper is not an oracle merely because it exists in a test
        // file; it must be used as the expected side of an equality assertion.
        let unused = WILD_COPY.replace("!escapes_the_root(&artifact)", "true");
        let unused = merge(
            extract_file("src/open.rs", SourceLanguage::Rust, IMPLEMENTATION),
            extract_file("tests/property_suite.rs", SourceLanguage::Rust, &unused),
        );
        assert_eq!(oracle_copies_in(&unused), vec![]);

        // Two equally similar production predicates leave no safe subject.
        let ambiguous = merge(
            extraction,
            extract_file("src/other.rs", SourceLanguage::Rust, IMPLEMENTATION),
        );
        assert_eq!(
            oracle_copies_in(&ambiguous),
            vec![],
            "similarity does not authorize guessing between two subjects"
        );
    }

    #[trace("TC-1000", "FR-064-AC-3")]
    // similarity is over tokens, so reordering does not (CR-100)
    // disguise a copy and reformatting does not manufacture one.
    #[test]
    fn tc1000_similarity_reads_tokens_not_layout() {
        let a = "normalized.components().all(|c| c != Component::ParentDir)";
        let reformatted = "normalized\n    .components()\n    .all(|c| c != Component::ParentDir)";
        assert!(
            (token_similarity(a, reformatted) - 1.0).abs() < f64::EPSILON,
            "reformatting is not a difference: {}",
            token_similarity(a, reformatted)
        );

        // Shared keywords alone must not make two unrelated pieces similar.
        let unrelated = "let mut total = 0; for item in items { total += item.weight }";
        assert!(
            token_similarity(a, unrelated) < ORACLE_SIMILARITY_FLOOR,
            "{}",
            token_similarity(a, unrelated)
        );

        // An empty side scores 0 rather than dividing by zero.
        assert_eq!(token_similarity("", a), 0.0);
        assert_eq!(token_similarity(a, ""), 0.0);
    }
}
