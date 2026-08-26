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
        let Some(candidate) = oracle_candidate(&code, test.language) else {
            continue;
        };
        let implementations: Vec<(&Symbol, String)> = extraction
            .symbols
            .iter()
            .filter(|symbol| {
                symbol.language == test.language
                    && symbol.kind == SymbolKind::Function
                    && simple_name(&symbol.qualified_name) == candidate.function
            })
            .filter_map(|symbol| {
                let source = extraction.source_of(&symbol.path)?;
                let span = symbol.attached_source(source);
                let masked =
                    crate::symbols::trace::mask_source_string_contents(&span, symbol.language);
                let code = strip_source_comments(&masked, symbol.language);
                implementation_expression(&code, symbol.language)
                    .map(|expression| (symbol, expression))
            })
            .collect();

        // A textual call name is not enough to disambiguate overloads or two
        // modules exporting the same function. Prefer the same file, but stand
        // down unless that produces exactly one subject.
        let same_file: Vec<_> = implementations
            .iter()
            .filter(|(symbol, _)| symbol.path == test.path)
            .collect();
        let resolved = if same_file.len() == 1 {
            Some((same_file[0].0, same_file[0].1.clone()))
        } else if same_file.is_empty() && implementations.len() == 1 {
            Some((implementations[0].0, implementations[0].1.clone()))
        } else {
            None
        };
        let Some((implementation, expression)) = resolved else {
            continue;
        };

        let score = token_similarity(&candidate.expression, &expression);
        if score < ORACLE_SIMILARITY_FLOOR {
            continue;
        }
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
        });
    }
    out.sort_by(|a, b| (&a.path, a.line, &a.symbol).cmp(&(&b.path, b.line, &b.symbol)));
    out
}

#[derive(Debug)]
struct OracleCandidate {
    binding: String,
    expression: String,
    function: String,
    line_offset: usize,
}

fn oracle_candidate(span: &str, language: SourceLanguage) -> Option<OracleCandidate> {
    static RUST_BINDING: OnceLock<Regex> = OnceLock::new();
    static TS_BINDING: OnceLock<Regex> = OnceLock::new();
    static PYTHON_BINDING: OnceLock<Regex> = OnceLock::new();
    static RUST_ASSERTION: OnceLock<Regex> = OnceLock::new();
    static TS_ASSERTION: OnceLock<Regex> = OnceLock::new();
    static PYTHON_ASSERTION: OnceLock<Regex> = OnceLock::new();

    let binding_re = match language {
        SourceLanguage::Rust => RUST_BINDING.get_or_init(|| {
            Regex::new(r"(?s)\blet\s+(expected|oracle)(?:\s*:[^=;]+)?\s*=\s*(.+?);")
                .expect("Rust oracle-binding pattern compiles")
        }),
        SourceLanguage::Typescript => TS_BINDING.get_or_init(|| {
            Regex::new(r"(?s)\b(?:const|let)\s+(expected|oracle)(?:\s*:[^=;]+)?\s*=\s*(.+?);")
                .expect("TypeScript oracle-binding pattern compiles")
        }),
        SourceLanguage::Python => PYTHON_BINDING.get_or_init(|| {
            Regex::new(r"(?m)^\s*(expected|oracle)(?:\s*:[^=\n]+)?\s*=\s*(.+?)\s*$")
                .expect("Python oracle-binding pattern compiles")
        }),
    };
    let assertion_re = match language {
        SourceLanguage::Rust => RUST_ASSERTION.get_or_init(|| {
            Regex::new(
                r"(?s)\bassert_eq!\s*\(\s*([A-Za-z_][A-Za-z0-9_:]*)\s*\([^;]*?\)\s*,\s*(expected|oracle)\s*\)",
            )
            .expect("Rust oracle-assertion pattern compiles")
        }),
        SourceLanguage::Typescript => TS_ASSERTION.get_or_init(|| {
            Regex::new(
                r"(?s)\bexpect\s*\(\s*([A-Za-z_$][A-Za-z0-9_$]*)\s*\([^;]*?\)\s*\)\s*\.\s*(?:toBe|toEqual)\s*\(\s*(expected|oracle)\s*\)",
            )
            .expect("TypeScript oracle-assertion pattern compiles")
        }),
        SourceLanguage::Python => PYTHON_ASSERTION.get_or_init(|| {
            Regex::new(
                r"(?m)^\s*assert\s+([A-Za-z_][A-Za-z0-9_]*)\s*\([^\n]*?\)\s*==\s*(expected|oracle)\b",
            )
            .expect("Python oracle-assertion pattern compiles")
        }),
    };

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
