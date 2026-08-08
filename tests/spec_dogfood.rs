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
use quire_rs::{check_document_grammar, Registry, Spec};

fn dogfood() -> Spec {
    Spec::from_path(Path::new("spec"))
}

// ─── The shipped module contract (FR-048-AC-11) ─────────────────────────────

/// The `grammar_severity` promotion `spec-artifacts-iso` ships (v0.8.0,
/// `manifest.yaml`). Mirrored here because the rest of this suite validates
/// against `tests/fixtures/modules/iso`, a fixture that carries **no**
/// `grammar_severity` block — so a check the published module promotes to
/// `error` was invisible to this repo's own CI, and quire-rs could ship a
/// `spec/` its own module contract rejects.
///
/// Keeping a mirror rather than a checked-in module copy is deliberate: CI has
/// no network and no `spec-artifacts-iso` checkout, and vendoring the manifest
/// would rot silently. The mirror is instead **verified against the real
/// module** whenever one is reachable — see `iso_module_path`.
const ISO_PROMOTED_ERRORS: &[&str] = &["ac:non-singular", "ac:vacuous-outcome"];

/// The bundle every ISO archetype binds to.
const ISO_BUNDLE: &str = "iso-spec-core";

/// Where a real `spec-artifacts-iso` module lives, when one is reachable:
/// `$QUIRE_ISO_MODULE`, else the conventional developer checkout. `None` in
/// CI, which falls back to [`ISO_PROMOTED_ERRORS`].
fn iso_module_path() -> Option<(PathBuf, bool)> {
    let (candidate, pinned) = match std::env::var_os("QUIRE_ISO_MODULE") {
        Some(v) => (PathBuf::from(v), true),
        None => (
            PathBuf::from(std::env::var_os("HOME")?).join("dev/spec-artifacts-iso/spec_artifacts_iso"),
            false,
        ),
    };
    candidate
        .join("manifest.yaml")
        .is_file()
        .then_some((candidate, pinned))
}

/// The severity map to judge this repo's own `spec/` by: the real module's
/// merged map when one is reachable, else the mirrored promotion.
///
/// When both are available they must agree — that assertion is the drift gate
/// on the mirror.
fn shipped_severity() -> GrammarSeverityMap {
    let mut mirror = GrammarSeverityMap::new();
    for key in ISO_PROMOTED_ERRORS {
        mirror.insert((*key).to_string(), GrammarSeverityLevel::Error);
    }

    let Some((module, pinned)) = iso_module_path() else {
        return mirror;
    };
    let registry = Registry::load_module(&module).unwrap_or_else(|e| {
        panic!(
            "failed to load the real module at {}: {e}",
            module.display()
        )
    });
    let real = registry.grammar_severity().clone();

    let real_errors: Vec<&str> = real
        .iter()
        .filter(|(_, level)| **level == GrammarSeverityLevel::Error)
        .map(|(key, _)| key.as_str())
        .collect();
    // Drift is only a hard failure when the module was named explicitly via
    // `QUIRE_ISO_MODULE` — that is a deliberate "judge me against this module".
    // The conventional `~/dev` checkout sits at whatever branch its developer
    // left it on, so asserting against it turns an unrelated experiment into a
    // failure here. Warn instead, and still judge `spec/` by the real map: a
    // mirror that has genuinely rotted is visible, without the flake.
    if real_errors != ISO_PROMOTED_ERRORS {
        let message = format!(
            "ISO_PROMOTED_ERRORS {ISO_PROMOTED_ERRORS:?} disagrees with the module at {}, \
             which declares {real_errors:?}",
            module.display()
        );
        assert!(!pinned, "{message}; update the mirror");
        eprintln!("warning: {message} (unpinned checkout — set QUIRE_ISO_MODULE to enforce)");
    }
    real
}

/// Every `.md` under `spec/` paired with its frontmatter `type`.
fn spec_documents() -> Vec<(PathBuf, String, String)> {
    fn walk(dir: &Path, out: &mut Vec<(PathBuf, String, String)>) {
        let entries = std::fs::read_dir(dir).unwrap_or_else(|e| panic!("read {dir:?}: {e}"));
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|e| e == "md") {
                let Ok(text) = std::fs::read_to_string(&path) else {
                    continue;
                };
                let ty = quire_rs::extract_frontmatter(&text)
                    .frontmatter
                    .as_ref()
                    .and_then(|fm| fm.get("type"))
                    .and_then(|v| v.as_str())
                    .map(str::to_string);
                if let Some(ty) = ty {
                    out.push((path, ty, text));
                }
            }
        }
    }
    let mut out = Vec::new();
    walk(Path::new("spec"), &mut out);
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// TC-794 (FR-048-AC-11) — this repository's own `spec/` carries no finding of
/// a check the shipped module promotes to `error`.
///
/// The gate this closes: on 2026-08-07 `spec-artifacts-iso` v0.8.0 promoted
/// `ac:non-singular` and `ac:vacuous-outcome` to `error`, and nothing in this
/// repo's CI could have noticed if `spec/` violated them — every other test
/// validates against a fixture module with no `grammar_severity` block.
///
/// The vocabularies are the engine defaults, not the module's: a module lexicon
/// only ever *suppresses* findings, so judging on the defaults is the stricter
/// reading and cannot go green on a vocabulary accident.
#[test]
fn tc794_own_spec_carries_no_promoted_error_finding() {
    let severity = shipped_severity();
    let vocab = GrammarVocabularies {
        lexicon: quire_rs::grammar::empty_lexicon(),
        observable: quire_rs::grammar::default_observable_verbs(),
        vacuous: quire_rs::grammar::default_vacuous_predicates(),
    };

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
        "quire-rs `spec/` fails the severity promotion its own module ships:\n{}",
        errors.join("\n")
    );
}

#[test]
fn loads_the_real_spec_corpus() {
    let spec = dogfood();
    // ~56 artifacts (StR+US+FR+NFR+spec.md+ADRs); README.md/tests.md skipped.
    // A lower bound guards against a green-but-empty regression (a bad
    // root silently yields len()==0 per FR-024-AC-7).
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
    // StR-005 / StR-006 exist on disk, so no edge to them is dangling.
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
