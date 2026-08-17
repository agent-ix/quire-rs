//! FR-056 — requirement-quality lints (TC-861..TC-869).

use std::fs;
use std::path::PathBuf;

use quire_rs::grammar::quality::AmbiguityTerms;
use quire_rs::{GrammarSeverity, Registry};

fn iso_module() -> PathBuf {
    PathBuf::from("/home/peter/dev/spec-artifacts-iso/spec_artifacts_iso")
}

/// A module declaring the FR archetype on `iso-spec-core`, plus whatever
/// `ambiguity_terms` the caller wants.
fn module_with_terms(suffix: &str, terms: &[&str]) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("quire-rs-quality-{}-{suffix}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("mkdir");
    let mut yaml = String::from(
        "name: quality-fixture\nmanifest_version: 1.0.0\nversion: 0.0.1\nartifact_types:\n\
         - name: FR\n  grammar_ref: iso-spec-core\n",
    );
    if !terms.is_empty() {
        yaml.push_str("ambiguity_terms:\n");
        for t in terms {
            yaml.push_str(&format!("  {t}: {{definition: a house word}}\n"));
        }
    }
    fs::write(root.join("manifest.yaml"), yaml).expect("write manifest");
    root
}

fn fr(description: &str) -> String {
    format!(
        "---\nid: FR-001\ntype: FR\ntitle: A requirement\n---\n\n## Description\n\n{description}\n"
    )
}

/// The `quality:` findings a document produces, as `(check, message)`.
fn quality_findings(registry: &Registry, doc: &str) -> Vec<(String, String)> {
    let archetype = registry.archetype("FR").expect("FR archetype");
    quire_rs::validate_document_in_registry(registry, archetype, doc)
        .warnings
        .iter()
        .filter_map(|w| {
            let rest = w.message.strip_prefix("[quality:")?;
            let (check, tail) = rest.split_once(']')?;
            Some((check.to_string(), tail.trim().to_string()))
        })
        .collect()
}

fn checks(registry: &Registry, doc: &str) -> Vec<String> {
    quality_findings(registry, doc)
        .into_iter()
        .map(|(c, _)| c)
        .collect()
}

/// A registry with no module-declared terms. Takes a suffix because the tests
/// run concurrently and each needs its own module directory — a shared one gets
/// deleted out from under a sibling mid-load.
fn plain(suffix: &str) -> Registry {
    Registry::load_module(&module_with_terms(suffix, &[])).expect("load")
}

// TC-861 (FR-056-AC-1): a built-in ambiguity term fires and names itself.
#[test]
fn tc861_builtin_ambiguity_term_fires_and_names_the_term() {
    let registry = plain("861");
    let findings = quality_findings(
        &registry,
        &fr("The system shall provide adequate throughput."),
    );
    let ambiguous: Vec<_> = findings
        .iter()
        .filter(|(c, _)| c == "ambiguous-term")
        .collect();
    assert_eq!(ambiguous.len(), 1, "{findings:#?}");
    assert!(ambiguous[0].1.contains("`adequate`"), "{:?}", ambiguous[0]);

    // Remove the term and the finding goes with it.
    let clean = checks(
        &registry,
        &fr("The system shall sustain 500 requests per second."),
    );
    assert!(
        !clean.iter().any(|c| c == "ambiguous-term"),
        "a quantified statement must not fire: {clean:?}",
    );
}

// TC-862 (FR-056-AC-2): the longest matching term names the finding, so the
// report says what the author wrote.
#[test]
fn tc862_longest_term_names_the_finding() {
    let registry = plain("862");
    let findings = quality_findings(&registry, &fr("The system shall retry as appropriate."));
    let ambiguous = findings
        .iter()
        .find(|(c, _)| c == "ambiguous-term")
        .expect("fires");
    assert!(
        ambiguous.1.contains("`as appropriate`"),
        "reported the substring instead of the phrase: {ambiguous:?}",
    );
}

// TC-863 (FR-056-AC-3): module terms layer OVER the built-ins, never replace.
#[test]
fn tc863_module_terms_extend_the_builtins() {
    let registry = Registry::load_module(&module_with_terms("extend", &["snappy"])).expect("load");

    let declared = quality_findings(&registry, &fr("The interface shall feel snappy."));
    assert!(
        declared
            .iter()
            .any(|(c, m)| c == "ambiguous-term" && m.contains("`snappy`")),
        "the declared term did not fire: {declared:#?}",
    );

    // The built-ins survive the extension — the failure mode of a registry that
    // replaces rather than layers.
    let builtin = quality_findings(&registry, &fr("The system shall be robust."));
    assert!(
        builtin
            .iter()
            .any(|(c, m)| c == "ambiguous-term" && m.contains("`robust`")),
        "a built-in stopped firing once a module declared its own: {builtin:#?}",
    );
}

// TC-864 (FR-056-AC-4): the check is about missing allocation, not the voice.
#[test]
fn tc864_agentless_passive_is_about_allocation_not_voice() {
    let registry = plain("864");
    let agentless = quality_findings(&registry, &fr("The input shall be validated."));
    assert!(
        agentless
            .iter()
            .any(|(c, m)| c == "agentless-passive" && m.contains("validated")),
        "{agentless:#?}",
    );

    let allocated = checks(
        &registry,
        &fr("The input shall be validated by the parser."),
    );
    assert!(
        !allocated.iter().any(|c| c == "agentless-passive"),
        "naming the agent must clear the finding — the passive voice is not the defect: {allocated:?}",
    );

    let active = checks(&registry, &fr("The parser shall validate the input."));
    assert!(
        !active.iter().any(|c| c == "agentless-passive"),
        "{active:?}"
    );
}

// TC-865 (FR-056-AC-5): two modals is the defect; one is not.
#[test]
fn tc865_mixed_modal_needs_two_modals() {
    let registry = plain("865");
    let mixed = quality_findings(
        &registry,
        &fr("The system shall retry the request and should log the attempt."),
    );
    let finding = mixed
        .iter()
        .find(|(c, _)| c == "mixed-modal")
        .expect("fires");
    assert!(
        finding.1.contains("shall") && finding.1.contains("should"),
        "{finding:?}"
    );

    let single = checks(&registry, &fr("The system shall retry the request."));
    assert!(!single.iter().any(|c| c == "mixed-modal"), "{single:?}");
}

// TC-866 (FR-056-AC-6): CR-017 parity — a quoted term is a mention.
#[test]
fn tc866_quoted_term_is_a_mention_not_a_use() {
    let registry = plain("866");
    let quoted = checks(
        &registry,
        &fr("The loader shall reject the manifest key `optimize` at load time."),
    );
    assert!(
        !quoted.iter().any(|c| c == "ambiguous-term"),
        "a term inside a code span is a mention, not an ambiguous requirement: {quoted:?}",
    );

    let used = checks(
        &registry,
        &fr("The loader shall optimize the manifest at load time."),
    );
    assert!(
        used.iter().any(|c| c == "ambiguous-term"),
        "the same term unquoted must fire: {used:?}",
    );
}

// TC-867 (FR-056-AC-7): advisory on arrival, and individually silenceable.
#[test]
fn tc867_advisory_and_individually_addressable() {
    let registry = plain("867");
    let archetype = registry.archetype("FR").expect("FR");
    let doc = fr("The system shall be adequately robust.");

    // Every quality finding routes to `warnings`, never `errors` — which is
    // what `Warning` severity means on this path (CON-1).
    let result = quire_rs::validate_document_in_registry(&registry, archetype, &doc);
    assert!(
        result.errors.is_empty(),
        "an advisory pack must contribute no error: {:#?}",
        result.errors
    );
    assert!(!checks(&registry, &doc).is_empty());
    assert_eq!(GrammarSeverity::Warning, GrammarSeverity::Warning);

    // FR-048: the check is addressable on its own key.
    let silenced = registry.with_grammar_severity(
        [(
            "quality:ambiguous-term".to_string(),
            quire_rs::grammar::GrammarSeverityLevel::Off,
        )]
        .into_iter()
        .collect(),
    );
    let after = checks(&silenced, &doc);
    assert!(
        !after.iter().any(|c| c == "ambiguous-term"),
        "`off` must remove the check entirely: {after:?}",
    );
}

// TC-868 (FR-056-AC-8): the pack adds a grammar, it does not reinterpret the
// two that exist (CON-4).
#[test]
fn tc868_ears_and_ac_findings_are_unchanged() {
    // The ISO module carries the real archetypes, so this runs over the same
    // grammar surface production does.
    let registry = Registry::load_module(&iso_module()).expect("load iso");
    let archetype = registry.archetype("FR").expect("FR");
    let doc = "---\nid: FR-001\ntype: FR\ntitle: A requirement\n---\n\n\
               ## Description\n\nshall process the input adequately.\n\n\
               ## Acceptance Criteria\n\n\
               | ID | Criteria | Verification |\n|----|----------|--------------|\n\
               | FR-001-AC-1 | The system shall support pagination. | Test |\n";
    let result = quire_rs::validate_document_in_registry(&registry, archetype, doc);

    // Silencing the whole quality pack must leave the other two streams
    // byte-for-byte identical — the only honest way to assert CON-4.
    let silenced = registry.with_grammar_severity(
        [
            (
                "quality:ambiguous-term",
                quire_rs::grammar::GrammarSeverityLevel::Off,
            ),
            (
                "quality:agentless-passive",
                quire_rs::grammar::GrammarSeverityLevel::Off,
            ),
            (
                "quality:mixed-modal",
                quire_rs::grammar::GrammarSeverityLevel::Off,
            ),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect(),
    );
    let baseline = quire_rs::validate_document_in_registry(&silenced, archetype, doc);

    let non_quality = |r: &quire_rs::ValidationResult| -> Vec<String> {
        r.warnings
            .iter()
            .filter(|w| !w.message.starts_with("[quality:"))
            .map(|w| format!("{}@{:?}", w.message, w.line))
            .collect()
    };
    assert_eq!(
        non_quality(&result),
        non_quality(&baseline),
        "the quality pack moved an `ears` or `ac` finding",
    );
    assert!(
        !non_quality(&result).is_empty(),
        "the fixture must produce other findings to compare"
    );
}

// TC-869 (FR-056-AC-9): checks are independent, so two defects report twice.
#[test]
fn tc869_two_defects_report_two_findings() {
    let registry = plain("869");
    let found = checks(
        &registry,
        &fr("The input shall be validated and should be robust."),
    );
    assert!(found.iter().any(|c| c == "agentless-passive"), "{found:?}");
    assert!(found.iter().any(|c| c == "mixed-modal"), "{found:?}");
    assert!(found.iter().any(|c| c == "ambiguous-term"), "{found:?}");
    assert!(found.len() >= 3, "checks must be independent: {found:?}");
}

// Direct-matcher coverage for the layering rule, independent of a Registry.
#[test]
fn ambiguity_terms_layer_over_builtins() {
    let builtin = AmbiguityTerms::builtin();
    let extended = AmbiguityTerms::with_module_terms(["snappy"].into_iter());
    assert_eq!(extended.len(), builtin.len() + 1);
}
