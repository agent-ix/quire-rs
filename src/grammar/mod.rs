//! Requirement-grammar checking (FR-042).
//!
//! A *grammar* evaluates the natural-language requirement statements inside
//! requirement-bearing artifacts. This is a posture distinct from declarative
//! lint ([`crate::lint`], FR-036) and structural validation
//! ([`crate::validate_document`], FR-032): a grammar classifies and checks the
//! *prose of individual normative statements*, and its findings carry a
//! [`GrammarSeverity`] that routes them into the validation result.
//!
//! Archetypes bind to a grammar **bundle** via the manifest `grammar_ref`
//! field (e.g. `grammar_ref: iso-spec-core`). The framework holds a registry of
//! bundles by name and dispatches the right sub-grammar by
//! `(archetype, section)` inside the bundle. v1 ships the `iso-spec-core`
//! bundle implementing **EARS** for FR/NFR/StR normative sections; GWT (for
//! acceptance criteria) and the `US` story grammar register onto the same
//! framework later.

pub mod ac;
pub mod ears;

use std::sync::OnceLock;

use regex::Regex;

use crate::ast::QuireDocument;

/// A precompiled matcher over the merged module `lexicon` (FR-043): the set of
/// accepted **concrete terms** the EARS vague-response check (FR-042) treats as
/// verifiable objects. Built once from the `Registry`'s merged lexicon; the
/// engine carries no hardcoded concrete-noun list. An empty lexicon (the
/// type-only `validate_document` path, or `check_grammar` without a module)
/// matches nothing — generic mechanism/bound/backtick suppression still applies.
#[derive(Debug, Default)]
pub struct GrammarLexicon {
    matcher: Option<Regex>,
}

impl GrammarLexicon {
    /// Build a matcher from concrete terms (the merged lexicon keys). Terms are
    /// matched case-insensitively at word boundaries. An empty term set yields a
    /// matcher that recognises nothing.
    pub fn from_terms<'a, I: IntoIterator<Item = &'a str>>(terms: I) -> Self {
        // Each term contributes both its singular and its plural form, so
        // corpus objects match regardless of number — including irregular
        // plurals (`policy`→`policies`, `box`→`boxes`).
        let mut forms: Vec<String> = Vec::new();
        for t in terms {
            let t = t.trim();
            if t.is_empty() {
                continue;
            }
            forms.push(regex::escape(t));
            forms.push(regex::escape(&pluralize(t)));
        }
        let matcher = if forms.is_empty() {
            None
        } else {
            Regex::new(&format!(r"(?i)\b({})\b", forms.join("|"))).ok()
        };
        Self { matcher }
    }

    /// The empty lexicon — recognises no concrete term.
    pub fn empty() -> Self {
        Self { matcher: None }
    }

    /// True when `text` contains an accepted concrete term (word-boundary).
    pub fn contains_term(&self, text: &str) -> bool {
        self.matcher.as_ref().is_some_and(|r| r.is_match(text))
    }
}

/// The built-in **observable-result verb** vocabulary (FR-047): base forms of
/// the verbs whose presence in an outcome clause makes the result externally
/// checkable. Module data extends this set (ADR 0009); the built-ins are the
/// lowest-precedence layer, never replaced.
const BUILTIN_OBSERVABLE_VERBS: &[&str] = &[
    "return", "emit", "yield", "report", "record", "reject", "fail", "exit", "persist", "print",
    "contain", "equal", "match",
];

/// A precompiled matcher over the observable-result verbs (FR-047-AC-5/AC-12):
/// the built-in defaults merged with any module-declared `observable_verbs`
/// registry. Each verb contributes its inflections, so an outcome clause
/// matches regardless of tense (`return` → `returns`/`returned`/`returning`).
#[derive(Debug)]
pub struct ObservableVerbs {
    matcher: Option<Regex>,
}

impl Default for ObservableVerbs {
    /// The built-in default set, applied when no module declares the registry.
    fn default() -> Self {
        Self::with_module_verbs(std::iter::empty())
    }
}

impl ObservableVerbs {
    /// Build a matcher from the built-in defaults plus `module_verbs` (the
    /// merged `observable_verbs` registry keys). The built-ins sit at lowest
    /// precedence: a module extends the vocabulary, it does not replace it.
    pub fn with_module_verbs<'a, I: IntoIterator<Item = &'a str>>(module_verbs: I) -> Self {
        let mut forms: Vec<String> = Vec::new();
        for verb in BUILTIN_OBSERVABLE_VERBS
            .iter()
            .copied()
            .chain(module_verbs.into_iter())
        {
            let verb = verb.trim();
            if verb.is_empty() {
                continue;
            }
            for form in inflect(verb) {
                forms.push(regex::escape(&form));
            }
        }
        let matcher = if forms.is_empty() {
            None
        } else {
            Regex::new(&format!(r"(?i)\b({})\b", forms.join("|"))).ok()
        };
        Self { matcher }
    }

    /// True when `text` carries an observable-result verb (word-boundary).
    pub fn contains_verb(&self, text: &str) -> bool {
        self.matcher.as_ref().is_some_and(|r| r.is_match(text))
    }
}

/// The shared built-in observable-verb vocabulary for the registry-free paths.
pub fn default_observable_verbs() -> &'static ObservableVerbs {
    static DEFAULT: OnceLock<ObservableVerbs> = OnceLock::new();
    DEFAULT.get_or_init(ObservableVerbs::default)
}

/// Naive English verb inflector: base, 3rd-person singular, past, and present
/// participle. Vocabulary verbs are lowercase ASCII, so byte handling is safe.
fn inflect(verb: &str) -> Vec<String> {
    let lower = verb.to_ascii_lowercase();
    let third = if lower.ends_with('s')
        || lower.ends_with('x')
        || lower.ends_with('z')
        || lower.ends_with("ch")
        || lower.ends_with("sh")
    {
        format!("{lower}es")
    } else {
        format!("{lower}s")
    };
    let (past, participle) = match lower.strip_suffix('e') {
        Some(stem) => (format!("{lower}d"), format!("{stem}ing")),
        None => (format!("{lower}ed"), format!("{lower}ing")),
    };
    vec![lower, third, past, participle]
}

/// Naive English pluralizer for lexicon terms: consonant+`y`→`ies`, sibilant
/// (`s`/`x`/`z`/`ch`/`sh`)→`es`, else `+s`. Lexicon terms are lowercase ASCII
/// nouns, so byte handling is safe.
fn pluralize(term: &str) -> String {
    let lower = term.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let n = bytes.len();
    if n >= 2 && bytes[n - 1] == b'y' && !b"aeiou".contains(&bytes[n - 2]) {
        format!("{}ies", &term[..n - 1])
    } else if lower.ends_with('s')
        || lower.ends_with('x')
        || lower.ends_with('z')
        || lower.ends_with("ch")
        || lower.ends_with("sh")
    {
        format!("{term}es")
    } else {
        format!("{term}s")
    }
}

/// The shared **all-default** severity map (FR-048-AC-7) for the registry-free
/// paths: every check keeps its default `warning` level.
pub fn default_severity() -> &'static GrammarSeverityMap {
    static DEFAULT: OnceLock<GrammarSeverityMap> = OnceLock::new();
    DEFAULT.get_or_init(GrammarSeverityMap::new)
}

/// The shared empty lexicon for the registry-free paths.
pub fn empty_lexicon() -> &'static GrammarLexicon {
    static EMPTY: OnceLock<GrammarLexicon> = OnceLock::new();
    EMPTY.get_or_init(GrammarLexicon::empty)
}

/// Severity of a grammar finding. `Warning` is advisory (never fails
/// validation); `Error` blocks. Severity is **policy**: a deployment may
/// promote a grammar from advisory to enforcing without a code change — the
/// migration lever for adopting EARS across the corpus (FR-042).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrammarSeverity {
    Warning,
    Error,
}

impl GrammarSeverity {
    /// Stable machine-readable string for surfaces (CLI, wheel).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

/// A configured severity **level** for one `<grammar>:<check>` pair (FR-048).
///
/// Distinct from [`GrammarSeverity`], which is the severity a *finding*
/// carries: `Off` is a configuration state with no finding counterpart — an
/// `off`-mapped check is suppressed entirely (no warning, no error, no
/// `--summary` entry). Modules declare this vocabulary in their manifest
/// `grammar_severity` registry; the CLI `--severity` flag overrides it per key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GrammarSeverityLevel {
    Off,
    Warning,
    Error,
}

impl GrammarSeverityLevel {
    /// Stable machine-readable string for surfaces (CLI, wheel).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

/// The merged per-check severity registry (FR-048): `<grammar>:<check>` →
/// level. `BTreeMap` because the merge order and any diagnostic derived from it
/// are observable (NFR-006). An absent key means `warning`, so the empty map is
/// the **all-default** map applied by the registry-free paths.
pub type GrammarSeverityMap = std::collections::BTreeMap<String, GrammarSeverityLevel>;

/// The registry key for a `(grammar, check)` pair — the `<grammar>:<check>`
/// form modules and the `--severity` CLI flag both author.
pub fn severity_key(grammar: &str, check: &str) -> String {
    format!("{grammar}:{check}")
}

/// The configured level for `(grammar, check)`, defaulting to `warning` when
/// the key is absent from `severity` (FR-048-AC-4).
pub fn severity_level(
    severity: &GrammarSeverityMap,
    grammar: &str,
    check: &str,
) -> GrammarSeverityLevel {
    severity
        .get(&severity_key(grammar, check))
        .copied()
        .unwrap_or(GrammarSeverityLevel::Warning)
}

/// A malformed `--severity <grammar>:<check>=<level>` entry (FR-048-AC-10).
/// The `Display` text is the usage diagnostic a surface prints before exiting
/// non-zero; it always quotes the offending entry.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SeverityOverrideError {
    #[error("--severity entry '{entry}' is malformed: expected '<grammar>:<check>=<level>'")]
    MalformedEntry { entry: String },
    #[error(
        "--severity entry '{entry}' has an unknown level '{level}': expected off|warning|error"
    )]
    UnknownLevel { entry: String, level: String },
}

/// Parse the repeatable `--severity <grammar>:<check>=<level>` option and layer
/// the entries **over** `manifest` (FR-048-AC-5): a CLI-supplied entry wins for
/// its key, every other manifest entry survives. A malformed entry is rejected
/// with a usage diagnostic and no map is produced, so a surface can exit
/// non-zero before validation runs (FR-048-AC-10).
pub fn merge_severity_overrides<'a, I: IntoIterator<Item = &'a str>>(
    manifest: &GrammarSeverityMap,
    entries: I,
) -> Result<GrammarSeverityMap, SeverityOverrideError> {
    let mut merged = manifest.clone();
    for entry in entries {
        let (key, level) = parse_severity_entry(entry)?;
        merged.insert(key, level);
    }
    Ok(merged)
}

/// Parse one `<grammar>:<check>=<level>` entry. The key must be the same
/// well-formed `<grammar>:<check>` shape the manifest registry accepts.
fn parse_severity_entry(
    entry: &str,
) -> Result<(String, GrammarSeverityLevel), SeverityOverrideError> {
    let malformed = || SeverityOverrideError::MalformedEntry {
        entry: entry.to_string(),
    };
    let (key, level) = entry.split_once('=').ok_or_else(malformed)?;
    if !is_severity_key(key) {
        return Err(malformed());
    }
    let level = match level {
        "off" => GrammarSeverityLevel::Off,
        "warning" => GrammarSeverityLevel::Warning,
        "error" => GrammarSeverityLevel::Error,
        other => {
            return Err(SeverityOverrideError::UnknownLevel {
                entry: entry.to_string(),
                level: other.to_string(),
            })
        }
    };
    Ok((key.to_string(), level))
}

/// True when `key` is a well-formed `<grammar>:<check>` registry key: exactly
/// one colon, two non-empty halves, no whitespace. Shared by the `--severity`
/// parser and the manifest loader so both surfaces accept the same vocabulary.
pub fn is_severity_key(key: &str) -> bool {
    key.matches(':').count() == 1
        && !key.contains(char::is_whitespace)
        && key
            .split_once(':')
            .is_some_and(|(grammar, check)| !grammar.is_empty() && !check.is_empty())
}

/// Histogram grammar findings by the generic `[<grammar>:<check>]` message
/// prefix (FR-047-AC-8) — the `--summary` grouping, covering **every** grammar
/// in the bundle rather than a hardcoded `[ears:` prefix. Messages without the
/// prefix (diagnostics from other validation layers) are ignored. `BTreeMap`
/// keeps the histogram deterministic (NFR-006).
pub fn summarize_findings<'a, I: IntoIterator<Item = &'a str>>(
    messages: I,
) -> std::collections::BTreeMap<String, usize> {
    let mut out: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for message in messages {
        if let Some(key) = finding_prefix_key(message) {
            *out.entry(key.to_string()).or_insert(0) += 1;
        }
    }
    out
}

/// The `<grammar>:<check>` key of a message carrying the generic finding
/// prefix `[<grammar>:<check>] …`, or `None` when it carries no such prefix.
pub fn finding_prefix_key(message: &str) -> Option<&str> {
    let rest = message.strip_prefix('[')?;
    let (key, _) = rest.split_once(']')?;
    is_severity_key(key).then_some(key)
}

/// One grammar diagnostic against a single normative statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrammarFinding {
    /// Grammar bundle that produced the finding (e.g. `ears`).
    pub grammar: String,
    /// Stable check id (e.g. `non-singular`, `vague-response`).
    pub check: String,
    /// Classification label for the statement, when the grammar classifies it
    /// (e.g. an EARS pattern: `ubiquitous`, `event`, …, `unclassifiable`).
    pub pattern: Option<String>,
    /// Human-readable message.
    pub message: String,
    /// 1-based document line of the offending statement, when known.
    pub line: Option<usize>,
    /// The offending statement excerpt.
    pub statement: String,
    /// Severity (advisory `Warning` by default in v1).
    pub severity: GrammarSeverity,
}

/// Apply the merged `grammar_severity` map (FR-048) to freshly emitted
/// `findings`: each finding's severity is set from the map keyed by
/// `<grammar>:<check>`, defaulting to `warning` when the key is absent, and an
/// `off`-mapped finding is **dropped here** — before routing — so it appears
/// neither in `warnings`/`errors` nor in any downstream `--summary` histogram
/// (FR-048-AC-9). Input order is preserved (NFR-006).
pub fn apply_severity(
    findings: Vec<GrammarFinding>,
    severity: &GrammarSeverityMap,
) -> Vec<GrammarFinding> {
    findings
        .into_iter()
        .filter_map(
            |mut f| match severity_level(severity, &f.grammar, &f.check) {
                GrammarSeverityLevel::Off => None,
                GrammarSeverityLevel::Warning => {
                    f.severity = GrammarSeverity::Warning;
                    Some(f)
                }
                GrammarSeverityLevel::Error => {
                    f.severity = GrammarSeverity::Error;
                    Some(f)
                }
            },
        )
        .collect()
}

/// Run the grammar bundle named `grammar_ref` against `doc`, dispatching by
/// `archetype` (the **resolved** archetype name, so a `--archetype` override on
/// a typeless document is still checked). An unknown bundle name yields no
/// findings — grammar checking is advisory by construction and never errors on
/// an unrecognised binding (a module may declare a `grammar_ref` this engine
/// build does not implement).
///
/// `line_offset` is the number of leading lines (frontmatter + fence) before
/// the document body, so findings can carry 1-based document line numbers.
pub fn check_document_grammar(
    grammar_ref: &str,
    archetype: &str,
    doc: &QuireDocument,
    line_offset: usize,
    lexicon: &GrammarLexicon,
    observable: &ObservableVerbs,
) -> Vec<GrammarFinding> {
    match grammar_ref {
        // The bundle runs every grammar registered on it; each grammar's own
        // `(archetype, section)` binding decides whether it contributes
        // (FR-042-AC-6, FR-047-AC-6).
        "iso-spec-core" => {
            let mut findings = ears::check(archetype, doc, line_offset, lexicon);
            findings.extend(ac::check(archetype, doc, line_offset, lexicon, observable));
            findings
        }
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn finding(grammar: &str, check: &str) -> GrammarFinding {
        GrammarFinding {
            grammar: grammar.to_string(),
            check: check.to_string(),
            pattern: None,
            message: "m".to_string(),
            line: Some(1),
            statement: "s".to_string(),
            severity: GrammarSeverity::Warning,
        }
    }

    // TC-719 (FR-048-AC-4): a finding whose `<grammar>:<check>` key is absent
    // from the merged map defaults to `warning`.
    #[test]
    fn tc719_absent_key_defaults_to_warning() {
        let mut map = GrammarSeverityMap::new();
        // A sibling key is mapped; the finding's own key is not.
        map.insert("ac:unclassifiable".into(), GrammarSeverityLevel::Error);
        let out = apply_severity(vec![finding("ears", "vague-response")], &map);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].severity, GrammarSeverity::Warning);
    }

    // TC-752 (FR-048-AC-9): an `off`-mapped check records no finding at all,
    // while sibling checks of the same grammar still report.
    #[test]
    fn tc752_off_drops_the_finding_before_routing() {
        let mut map = GrammarSeverityMap::new();
        map.insert("ac:vague-response".into(), GrammarSeverityLevel::Off);
        let out = apply_severity(
            vec![
                finding("ac", "vague-response"),
                finding("ac", "non-singular"),
                finding("ears", "vague-response"),
            ],
            &map,
        );
        let keys: Vec<String> = out
            .iter()
            .map(|f| format!("{}:{}", f.grammar, f.check))
            .collect();
        assert_eq!(keys, vec!["ac:non-singular", "ears:vague-response"]);
    }

    // TC-714 (FR-047-AC-8): the summary histogram groups by the generic
    // `[<grammar>:<check>]` prefix, so a corpus emitting both `[ears:*]` and
    // `[ac:*]` findings shows both.
    #[test]
    fn tc714_summary_groups_every_grammar() {
        let messages = [
            "[ears:vague-response] vague response verb `support`",
            "[ac:unclassifiable] criterion matches neither shape",
            "[ears:vague-response] vague response verb `provide`",
            "[ac:no-observable-outcome] outcome clause names no result",
            "unprefixed diagnostic from another layer",
        ];
        let histogram = summarize_findings(messages.iter().copied());
        assert_eq!(histogram.get("ears:vague-response"), Some(&2));
        assert_eq!(histogram.get("ac:unclassifiable"), Some(&1));
        assert_eq!(histogram.get("ac:no-observable-outcome"), Some(&1));
        assert_eq!(histogram.len(), 3, "unprefixed lines are not histogrammed");
        // Deterministic ordering (NFR-006).
        assert_eq!(
            histogram.keys().collect::<Vec<_>>(),
            vec![
                "ac:no-observable-outcome",
                "ac:unclassifiable",
                "ears:vague-response"
            ]
        );
    }

    // TC-720 (FR-048-AC-5): `--severity` entries are repeatable and take
    // precedence over a conflicting manifest entry for the same key.
    #[test]
    fn tc720_cli_severity_overrides_manifest() {
        let mut manifest = GrammarSeverityMap::new();
        manifest.insert("ears:vague-response".into(), GrammarSeverityLevel::Warning);
        manifest.insert("ac:unclassifiable".into(), GrammarSeverityLevel::Error);

        let merged = merge_severity_overrides(
            &manifest,
            ["ears:vague-response=error", "ac:non-singular=off"],
        )
        .expect("well-formed entries");

        // CLI wins for its key…
        assert_eq!(merged["ears:vague-response"], GrammarSeverityLevel::Error);
        // …the repeated second entry also lands…
        assert_eq!(merged["ac:non-singular"], GrammarSeverityLevel::Off);
        // …and an untouched manifest entry survives.
        assert_eq!(merged["ac:unclassifiable"], GrammarSeverityLevel::Error);

        // The promoted key now routes a finding as an error.
        let out = apply_severity(vec![finding("ears", "vague-response")], &merged);
        assert_eq!(out[0].severity, GrammarSeverity::Error);
    }

    // TC-721 (FR-048-AC-6): `--strict` semantics are untouched. The engine
    // surface `--strict` reads is unchanged: with no severity map a grammar
    // finding is still a warning that leaves the document valid, so the exit
    // code stays the CLI's own decision.
    #[test]
    fn tc721_strict_semantics_unchanged() {
        let manifest = GrammarSeverityMap::new();
        let merged = merge_severity_overrides(&manifest, std::iter::empty::<&str>())
            .expect("no entries is well-formed");
        assert!(merged.is_empty(), "no overrides → the map is untouched");

        let out = apply_severity(vec![finding("ears", "vague-response")], &merged);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].severity, GrammarSeverity::Warning);
    }

    // TC-755 (FR-048-AC-10): a malformed `--severity` entry is rejected with a
    // usage diagnostic before validation runs.
    #[test]
    fn tc755_malformed_severity_entry_rejected() {
        let manifest = GrammarSeverityMap::new();
        for bad in [
            "ears:vague-response=fatal", // unknown level
            "ears:vague-response",       // missing `=`
            "vague-response=error",      // unparseable key (no `<grammar>:`)
            "ears:=error",               // empty check
            ":vague-response=error",     // empty grammar
            "ears:vague response=error", // whitespace in key
        ] {
            let err = merge_severity_overrides(&manifest, [bad])
                .expect_err(&format!("`{bad}` must be rejected"));
            let msg = err.to_string();
            assert!(msg.contains(bad), "diagnostic must quote the entry: {msg}");
            assert!(
                msg.contains("--severity"),
                "diagnostic must name the option: {msg}"
            );
        }

        // A well-formed entry is still accepted.
        assert!(merge_severity_overrides(&manifest, ["ac:unclassifiable=off"]).is_ok());
    }

    // FR-048: an `error` mapping promotes the emitted finding's severity.
    #[test]
    fn error_level_promotes_finding_severity() {
        let mut map = GrammarSeverityMap::new();
        map.insert("ac:unclassifiable".into(), GrammarSeverityLevel::Error);
        let out = apply_severity(vec![finding("ac", "unclassifiable")], &map);
        assert_eq!(out[0].severity, GrammarSeverity::Error);
    }
}
