//! Gate G5 — corpus correctness, dogfooded on quire-rs's own `spec/`.
//!
//! Loads this repository's real spec tree (~56 cross-referencing
//! artifacts) into a `Spec`, resolves the references, and asserts facts
//! a healthy spec must satisfy. This is living regression coverage: if
//! `load_repo`/resolution/query break, the gate fails on real data, not
//! a toy fixture. Tests run with CWD = crate root, so `spec/` is the
//! path.

use std::path::{Path, PathBuf};

use quire_rs::corpus::resolve::Resolution;
use quire_rs::grammar::{
    apply_severity, GrammarSeverity, GrammarSeverityLevel, GrammarSeverityMap, GrammarVocabularies,
};
use quire_rs::{check_document_grammar, Spec};

fn dogfood() -> Spec {
    Spec::from_path(Path::new("spec"))
}

// ─── The promoted-severity contract (FR-048-AC-11) ──────────────────────────

/// The checks this repository's own `spec/` must be free of at `error`
/// severity. No in-repo fixture module carries a `grammar_severity` block, so
/// without this gate a check promoted to `error` by a module would be
/// invisible to this repo's own CI.
const PROMOTED_ERRORS: &[&str] = &["ac:non-singular", "ac:vacuous-outcome"];

/// The bundle every requirement archetype binds to.
const ISO_BUNDLE: &str = "iso-spec-core";

/// The severity map to judge this repo's own `spec/` by.
fn promoted_severity() -> GrammarSeverityMap {
    let mut map = GrammarSeverityMap::new();
    for key in PROMOTED_ERRORS {
        map.insert((*key).to_string(), GrammarSeverityLevel::Error);
    }
    map
}

/// Every typed document under `spec/`, paired with its frontmatter `type`.
///
/// This used to hand-roll a `read_dir` recursion, for one reason: the corpus
/// walk dropped `spec/tests.md` by filename, so `Spec::from_path` could not
/// see the matrix TC-794 has to grammar-check. That was the type-driven
/// membership rule, implemented here, in a test, as a workaround for the
/// engine not implementing it — and it made this the third independent
/// markdown walker in the tree. CR-044 moved the rule into the walk, so this
/// now goes through the engine like every other caller.
fn spec_documents() -> Vec<(PathBuf, String, String)> {
    let mut out = Vec::new();
    for doc in quire_rs::load_repo(Path::new("spec")).documents {
        // Untyped documents are corpus members but carry no archetype to
        // check against; validation diagnoses them, not this test.
        let Some(ty) = doc.concept_type().map(str::to_string) else {
            continue;
        };
        let Ok(text) = std::fs::read_to_string(&doc.path) else {
            continue;
        };
        out.push((doc.path.clone(), ty, text));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// TC-794, FR-048-AC-11 — this repository's own `spec/` carries no finding of
/// a promoted check.
///
/// The gate this closes: nothing else in this repo's CI would notice if
/// `spec/` violated `ac:non-singular` or `ac:vacuous-outcome` — every other
/// test validates against a fixture module with no `grammar_severity` block.
///
/// The vocabularies are the engine defaults, not the module's: a module lexicon
/// only ever *suppresses* findings, so judging on the defaults is the stricter
/// reading and cannot go green on a vocabulary accident.
#[test]
fn tc794_own_spec_carries_no_promoted_error_finding() {
    let severity = promoted_severity();
    let vocab = GrammarVocabularies::defaults();

    let mut errors = Vec::new();
    for (path, archetype, text) in spec_documents() {
        let doc = quire_rs::parse_document(&text);
        // `body_line_offset` is crate-private; the leading-line count is the
        // same quantity and only shifts the reported line, never the finding.
        let body = quire_rs::extract_frontmatter(&text).body;
        let line_offset = text.lines().count().saturating_sub(body.lines().count());
        let findings = apply_severity(
            check_document_grammar(ISO_BUNDLE, &archetype, &doc, line_offset, vocab),
            &severity,
        );
        for f in findings
            .iter()
            .filter(|f| f.severity == GrammarSeverity::Error)
        {
            errors.push(format!(
                "{}:{} [{}:{}] {}",
                path.display(),
                f.line.unwrap_or(0),
                f.grammar,
                f.check,
                f.statement
            ));
        }
    }

    assert!(
        errors.is_empty(),
        "quire-rs `spec/` fails the severity promotion:\n{}",
        errors.join("\n")
    );
}

#[test]
fn loads_the_real_spec_corpus() {
    let spec = dogfood();
    // ~57 artifacts (StR+US+FR+NFR+spec.md+ADRs+tests.md). Since CR-044 the
    // matrix `spec/tests.md` (`type: TestMatrix`, ~1200 lines) is a corpus
    // document like any other; only frontmatter-less files such as `README.md`
    // are absent. A lower bound guards against a green-but-empty regression
    // (a bad root silently yields len()==0 per FR-024-AC-7).
    assert!(
        spec.len() >= 50,
        "expected a populated corpus, got {}",
        spec.len()
    );
}

#[test]
fn finds_the_core_artifact_types() {
    let spec = dogfood();
    let frs: Vec<_> = spec.by_type("FR").iter().map(|d| d.id.clone()).collect();
    let strs: Vec<_> = spec.by_type("StR").iter().map(|d| d.id.clone()).collect();
    for id in ["FR-023", "FR-024", "FR-025", "FR-026", "FR-027"] {
        assert!(frs.contains(&id.to_string()), "missing {id} in by_type(FR)");
    }
    for id in ["StR-005", "StR-006"] {
        assert!(
            strs.contains(&id.to_string()),
            "missing {id} in by_type(StR)"
        );
    }
}

#[test]
fn v03_frs_each_trace_to_a_stakeholder_requirement() {
    let spec = dogfood();
    for fr in ["FR-023", "FR-024", "FR-025", "FR-026", "FR-027"] {
        let has_resolved_implements_to_str = spec.outgoing(fr).iter().any(|e| {
            e.edge_type == "implements"
                && e.resolution == Resolution::Resolved
                && e.target.starts_with("StR-")
        });
        assert!(
            has_resolved_implements_to_str,
            "{fr} has no resolved `implements` edge to a StR"
        );
    }
}

#[test]
fn reverse_lookup_finds_the_referencing_frs() {
    let spec = dogfood();
    // StR-006 is implemented by the corpus FRs FR-025/026/027.
    let referrers: Vec<_> = spec
        .referencing("StR-006")
        .iter()
        .map(|e| e.source.clone())
        .collect();
    for fr in ["FR-025", "FR-026", "FR-027"] {
        assert!(
            referrers.contains(&fr.to_string()),
            "{fr} should reference StR-006; got {referrers:?}"
        );
    }
}

#[test]
fn real_stakeholder_targets_are_not_dangling() {
    let spec = dogfood();
    // Both stakeholder requirements exist on disk, so no edge to them is
    // dangling. (Ids kept out of a tag-shaped position: `StR-005 /` would bind.)
    for str_id in ["StR-005", "StR-006"] {
        assert!(
            spec.dangling().iter().all(|e| e.target != str_id),
            "edge to existing {str_id} should resolve, not dangle"
        );
        assert!(
            spec.by_id(str_id).is_some(),
            "{str_id} should be in the corpus"
        );
    }
}
