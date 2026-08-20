//! Project Ubiquitous-Language term harvesting (FR-044).
//!
//! A repository authors its own concrete vocabulary — a `Glossary` artifact's
//! `## Terms` table, or a `## Ubiquitous Language` section on a domain object.
//! These terms are harvested from a loaded [`Spec`] and composed with the merged
//! module lexicon ([`crate::Registry::lexicon_with`]) so a repo's domain terms
//! are accepted as concrete objects in its own EARS grammar check (FR-042/043).

use std::path::Path;

use crate::ast::QuireDocument;
use crate::corpus::walk::{discover_files, is_document, WalkOptions};
use crate::corpus::Spec;
use crate::query::{self, ListPattern};

/// Harvest the repo's project glossary terms from an **already-loaded** `Spec`:
/// the `Term` (or `Name`) column of every `## Terms` table, and the bold term
/// of every `## Ubiquitous Language` bullet (and any `## Ubiquitous Language`
/// table). Deduplicated and sorted; a corpus with no glossary yields an empty
/// vec. Use this on the corpus path (`validate_bundle`), where the `Spec` is
/// already in hand — it adds no I/O. On the per-document validate path, use
/// [`glossary_terms_from_path`], which never loads the whole corpus.
pub fn glossary_terms(spec: &Spec) -> Vec<String> {
    let mut terms: Vec<String> = Vec::new();
    for doc in &spec.inner.documents {
        // Same cheap pre-filter as the path harvester (CR-047): only a
        // glossary-bearing document pays for a body parse, so
        // `validate_bundle` does not force every body in the corpus.
        if !has_glossary_heading(doc.raw()) {
            continue;
        }
        collect_doc_terms(doc.body(), &mut terms);
    }
    terms.sort();
    terms.dedup();
    terms
}

/// Harvest project glossary terms by scanning `root`, parsing **only** the
/// documents that carry a glossary section — never the whole corpus (FR-044).
///
/// `Spec::from_path` parses and graph-resolves every file under `root`; that is
/// far too much work when a repo's glossary lives in a handful of documents.
/// This enumerates `*.md` files, reads each file's raw text, and parses only
/// the ones whose text contains a `## Terms` or `## Ubiquitous Language`
/// heading (a cheap line scan — [`has_glossary_heading`]). Deduplicated and
/// sorted; a repo with no glossary yields an empty vec.
pub fn glossary_terms_from_path(root: &Path) -> Vec<String> {
    glossary_terms_from_path_with_diagnostics(root).0
}

/// [`glossary_terms_from_path`], plus the membership diagnostics it used to
/// swallow (CR-055).
///
/// CR-048 inverted the walk's silence about frontmatter-less markdown under
/// the document root, but this — the **second** consumer of the same
/// membership rule (FR-024-AC-11) — kept skipping in silence. A
/// `spec/glossary.md` that lost its front block contributes no terms, shrinks
/// the composed EARS lexicon, and says nothing about why; the symptom surfaces
/// as `vague-response` firing on the repo's own domain nouns, one indirection
/// away from the cause. The diagnostic is the same
/// [`Diagnostic::DocumentWithoutFrontmatter`](crate::diagnostic::Diagnostic)
/// the walk emits, because it is the same rule.
pub fn glossary_terms_from_path_with_diagnostics(
    root: &Path,
) -> (Vec<String>, Vec<crate::diagnostic::Diagnostic>) {
    let opts = WalkOptions::default();
    let mut terms: Vec<String> = Vec::new();
    let mut diagnostics: Vec<crate::diagnostic::Diagnostic> = Vec::new();
    for path in discover_files(root, &opts) {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        // Same membership rule as the corpus walk (CR-044). `discover_files`
        // used to apply a `{README.md, tests.md}` skip that this function
        // inherited without asking for it; when the skip went, the scope here
        // would have silently widened to every stray `.md`. A repository's
        // ubiquitous language is defined by its documents, so a file that is
        // not a document does not get to name terms.
        if !is_document(&text) {
            // Reported only when the file looked like it meant to contribute:
            // every README under the root is frontmatter-less and legitimately
            // so, and warning about each would bury the one that matters.
            if has_glossary_heading(&text) {
                diagnostics.push(crate::diagnostic::Diagnostic::DocumentWithoutFrontmatter {
                    path: path.clone(),
                    malformed: crate::parser::frontmatter::extract_frontmatter_ref(&text).status
                        == crate::parser::FrontmatterStatus::Malformed,
                });
            }
            continue;
        }
        if !has_glossary_heading(&text) {
            continue; // not a glossary-bearing doc — never parsed.
        }
        collect_doc_terms(&crate::parse_document(&text), &mut terms);
    }
    terms.sort();
    terms.dedup();
    diagnostics.sort_by_key(|d| format!("{d:?}"));
    (terms, diagnostics)
}

/// The cheap pre-filter that keeps both harvesters from parsing non-glossary
/// documents: true when `text` has a heading (any level) that
/// [`query::section`] would match against `Terms` or `Ubiquitous Language`.
///
/// **It must agree with the lookup it gates** (CR-055). It originally compared
/// the raw title verbatim while `section` matches through
/// [`query::normalize_heading`], which treats ISO section numbering as
/// decorative — so `## 3.2 Ubiquitous Language` and `### 4. Terms`, the
/// standard ISO heading form, were filtered out before the lookup that would
/// have found them. A pre-filter stricter than what it gates does not save
/// work, it silently changes the answer: those repos' domain nouns dropped out
/// of the composed EARS lexicon, where their only symptom is `vague-response`
/// newly firing on a repo's own vocabulary.
pub(crate) fn has_glossary_heading(text: &str) -> bool {
    text.lines().any(|line| {
        let trimmed = line.trim_start();
        if !trimmed.starts_with('#') {
            return false;
        }
        // Normalize exactly as the parsed section heading would be: the walk
        // strips a trailing Pandoc `{#block-id}` from heading text, and
        // `matches_heading` then normalizes the section number away.
        let mut title = trimmed.trim_start_matches('#').trim().to_string();
        crate::parser::walk::strip_trailing_block_id(&mut title);
        let title = query::normalize_heading(&title);
        title.eq_ignore_ascii_case("Terms") || title.eq_ignore_ascii_case("Ubiquitous Language")
    })
}

/// Collect a single document's glossary terms into `terms` (shared by the
/// `Spec`-based and path-based harvesters).
fn collect_doc_terms(doc: &QuireDocument, terms: &mut Vec<String>) {
    // `## Terms` table — a `Glossary` artifact.
    push_table_terms(doc, "Terms", terms);
    // `## Ubiquitous Language` — bullet form (`- **Term** — …`) …
    if let Some(section) = query::section(doc, "Ubiquitous Language") {
        for item in query::parse_bullet_list(&section.content, Some(ListPattern::BoldDescription)) {
            push_term(&item.title, terms);
        }
    }
    // … and/or table form under the same heading.
    push_table_terms(doc, "Ubiquitous Language", terms);
}

/// Push the `Term`/`Name` column of a named section's table into `out`.
fn push_table_terms(doc: &QuireDocument, heading: &str, out: &mut Vec<String>) {
    let Some(table) = query::table_from_section(doc, heading) else {
        return;
    };
    let Some(idx) = table.headers.iter().position(|h| {
        let h = h.trim().to_ascii_lowercase();
        h == "term" || h == "name"
    }) else {
        return;
    };
    for row in &table.rows {
        if let Some(cell) = row.get(idx) {
            push_term(cell, out);
        }
    }
}

fn push_term(raw: &str, out: &mut Vec<String>) {
    let t = raw.trim();
    if !t.is_empty() {
        out.push(t.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ix_trace_rs::trace;

    /// Harvest from a repo holding a `Glossary` `## Terms` table (Widget,
    /// Sprocket) and a `domain` `## Ubiquitous Language` (Place, Capture).
    /// Shared fixture for TC-674 and TC-675.
    fn harvest_fixture(tag: &str) -> Vec<String> {
        let tmp = std::env::temp_dir().join(format!("ql-glossary-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(
            tmp.join("GLO-001.md"),
            "---\nid: GLO-001\ntype: Glossary\n---\n# [GLO-001] G\n\n## Terms\n\n| Term | Definition |\n|------|------------|\n| Widget | a UI thing |\n| Sprocket | a gear |\n",
        )
        .unwrap();
        std::fs::write(
            tmp.join("domain.md"),
            "---\nid: dom-1\ntype: domain\n---\n# Domain\n\n## Ubiquitous Language\n\n- **Place** — convert a draft order.\n- **Capture** — confirm payment.\n",
        )
        .unwrap();
        let terms = glossary_terms(&Spec::from_path(&tmp));
        let _ = std::fs::remove_dir_all(&tmp);
        terms
    }

    #[trace("TC-674", "FR-044-AC-1")]
    // harvest the Term column from a `## Terms` table.
    #[test]
    fn tc674_harvest_terms_table() {
        let terms = harvest_fixture("674");
        assert!(terms.contains(&"Widget".to_string()));
        assert!(terms.contains(&"Sprocket".to_string()));
    }

    #[trace("TC-675", "FR-044-AC-2")]
    // harvest the bold term from `## Ubiquitous Language`.
    #[test]
    fn tc675_harvest_ubiquitous_language() {
        let terms = harvest_fixture("675");
        assert!(terms.contains(&"Place".to_string()));
        assert!(terms.contains(&"Capture".to_string()));
    }

    // The pre-filter that scopes the path harvester: only docs with an actual
    // `Terms` / `Ubiquitous Language` HEADING are parsed — not every doc that
    // merely mentions the words. This is what keeps the per-validate path off a
    // full-corpus parse.
    #[test]
    fn has_glossary_heading_gates_parsing() {
        assert!(has_glossary_heading("## Terms\n"));
        assert!(has_glossary_heading("# Title\n\n## Ubiquitous Language\n"));
        assert!(has_glossary_heading("### terms\n")); // any level, case-insensitive
                                                      // Not glossary-bearing → never parsed:
        assert!(!has_glossary_heading(
            "## Description\n\nThe system shall do X.\n"
        ));
        assert!(!has_glossary_heading("These are the terms of service.\n")); // prose, no heading
        assert!(!has_glossary_heading("| Term | Definition |\n")); // a table row, not a heading
    }

    // `glossary_terms_from_path` harvests the glossary terms WITHOUT loading the
    // whole corpus: a scope with one glossary doc among many plain FRs yields
    // exactly the glossary terms, and matches the Spec-based harvester (parity).
    // The plain FRs lack a glossary heading, so the gate above skips parsing
    // them entirely.
    #[test]
    fn glossary_terms_from_path_scopes_to_glossary_docs() {
        let dir = std::env::temp_dir().join(format!("ql-frompath-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("functional")).unwrap();
        std::fs::write(
            dir.join("glossary.md"),
            "---\nid: GLO-001\ntype: Glossary\n---\n# G\n\n## Terms\n\n| Term | Definition |\n|------|------------|\n| Sprocket | a streamed unit |\n",
        )
        .unwrap();
        // Many non-glossary docs — none carry a glossary heading.
        for i in 0..5 {
            std::fs::write(
                dir.join("functional").join(format!("FR-{i:03}.md")),
                format!("---\nid: FR-{i:03}\ntype: FR\n---\n# FR\n\n## Description\n\nThe system shall provide a sprocket.\n"),
            )
            .unwrap();
        }
        let from_path = glossary_terms_from_path(&dir);
        assert_eq!(from_path, vec!["Sprocket".to_string()]);
        // Parity with the Spec-based harvester over the same tree.
        assert_eq!(from_path, glossary_terms(&Spec::from_path(&dir)));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[trace("TC-808", "FR-024-AC-11")]
    // this scanner reads raw text rather than (CR-044)
    // building a `Spec`, so it needs the membership rule applied explicitly.
    // It used to inherit the walk's `{README.md, tests.md}` skip through
    // `discover_files`; when that skip went, the scope here would have widened
    // in silence to every stray `.md`. A repository's ubiquitous language is
    // defined by its documents.
    #[trace("TC-808", "FR-024-AC-11")]
    #[test]
    fn tc808_from_path_applies_the_corpus_membership_rule() {
        let dir = std::env::temp_dir().join(format!("ql-membership-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        // Identical glossary content, once without frontmatter and once with.
        let body = "# Project\n\n## Ubiquitous Language\n\n- **Sprocket** — a streamed unit.\n";
        std::fs::write(dir.join("README.md"), body).unwrap();
        assert_eq!(
            glossary_terms_from_path(&dir),
            Vec::<String>::new(),
            "a file that is not a document must not name project terms"
        );

        std::fs::write(
            dir.join("domain.md"),
            format!("---\nid: dom-1\ntype: domain\n---\n{body}"),
        )
        .unwrap();
        assert_eq!(glossary_terms_from_path(&dir), vec!["Sprocket".to_string()]);

        // And the two harvesters still agree over the same tree.
        assert_eq!(
            glossary_terms_from_path(&dir),
            glossary_terms(&Spec::from_path(&dir))
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Build a bundle report for a repo whose `Glossary` defines `widget` and
    /// whose `FR-001` says "shall provide a widget", validated Strict (so the
    /// EARS grammar runs). Shared fixture for TC-678 and TC-679.
    fn glossary_bundle_report(tag: &str) -> crate::corpus::BundleReport {
        let base = std::env::temp_dir().join(format!("ql-bundle-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let m = base.join("mod");
        std::fs::create_dir_all(m.join("schemas")).unwrap();
        std::fs::write(
            m.join("manifest.yaml"),
            "name: m\nartifact_types:\n- name: FR\n  frontmatter_schema_ref: schemas/s.json\n  grammar_ref: iso-spec-core\n- name: Glossary\n  frontmatter_schema_ref: schemas/s.json\n",
        )
        .unwrap();
        std::fs::write(m.join("schemas/s.json"), r#"{"type":"object"}"#).unwrap();
        let registry = crate::Registry::load_module(&m).expect("module loads");
        let docs = base.join("spec");
        std::fs::create_dir_all(&docs).unwrap();
        std::fs::write(
            docs.join("GLO-001.md"),
            "---\nid: GLO-001\ntype: Glossary\n---\n# G\n\n## Terms\n\n| Term | Definition |\n|------|------------|\n| widget | a UI thing |\n",
        )
        .unwrap();
        std::fs::write(
            docs.join("FR-001.md"),
            "---\nid: FR-001\ntype: FR\n---\n# FR\n\n## Description\n\nThe system shall provide a widget.\n",
        )
        .unwrap();
        let spec = Spec::from_path(&docs);
        let report = crate::corpus::validate_bundle(
            &spec,
            &registry,
            crate::corpus::BundlePosture::Strict,
            &docs,
            &docs,
        );
        let _ = std::fs::remove_dir_all(&base);
        report
    }

    #[trace("TC-678", "FR-044-AC-5")]
    // validate_bundle harvests the Spec's project terms
    // and applies the combined lexicon — the FR's `widget` is suppressed.
    #[test]
    fn tc678_validate_bundle_harvests_and_applies() {
        let report = glossary_bundle_report("678");
        assert!(!report
            .warnings
            .iter()
            .any(|w| w.message.contains("[ears:vague-response]")));
    }

    #[trace("TC-679", "FR-044-AC-6")]
    // the project-glossary suppression is advisory —
    // grammar findings never become bundle errors.
    #[test]
    fn tc679_validate_bundle_grammar_is_advisory() {
        let report = glossary_bundle_report("679");
        assert!(!report.errors.iter().any(|e| e.reason == "grammar"));
    }

    #[trace("TC-680", "FR-044-AC-7")]
    // a repo with no glossary harvests nothing.
    #[test]
    fn tc680_no_glossary_empty() {
        let tmp = std::env::temp_dir().join(format!("ql-noglossary-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(
            tmp.join("FR-001.md"),
            "---\nid: FR-001\ntype: FR\n---\n# FR\n\n## Description\n\nThe system shall do a thing.\n",
        )
        .unwrap();
        let spec = Spec::from_path(&tmp);
        assert!(glossary_terms(&spec).is_empty());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    // CON-2 (malformed half): a `## Terms` heading with no table harvests zero
    // terms and never panics (FR-044-AC-7's sibling constraint).
    #[test]
    fn malformed_glossary_harvests_nothing() {
        let tmp = std::env::temp_dir().join(format!("ql-malformed-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(
            tmp.join("GLO-001.md"),
            "---\nid: GLO-001\ntype: Glossary\n---\n# [GLO-001] G\n\n## Terms\n\nNo table here, just prose.\n",
        )
        .unwrap();
        let spec = Spec::from_path(&tmp);
        assert!(glossary_terms(&spec).is_empty());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[trace("TC-823", "FR-044-AC-8")]
    // the pre-filter agrees with the lookup it (CR-055)
    // gates. ISO heading form numbers its sections, and `query::section`
    // treats that numbering as decorative — so a pre-filter matching the raw
    // title verbatim filtered out exactly the documents the lookup would have
    // found, and their terms left the composed lexicon in silence.
    #[test]
    fn tc823_numbered_headings_still_harvest() {
        for heading in [
            "## 3.2 Ubiquitous Language",
            "### 4. Terms",
            "## 2.1. Ubiquitous Language",
            "## Ubiquitous Language {#blk-ul}",
        ] {
            assert!(
                has_glossary_heading(&format!("# Doc\n\n{heading}\n")),
                "{heading:?} must reach the section lookup"
            );
        }
        // Still not glossary-bearing — the filter did not become a pass-all.
        assert!(!has_glossary_heading("## 3.2 Scope\n"));
        assert!(!has_glossary_heading("## Terms of Service\n"));
    }

    #[trace("TC-823", "FR-044-AC-8")]
    // and end to end — a numbered heading (CR-055)
    // harvests the same terms an unnumbered one does, through both harvesters.
    #[test]
    fn tc823_numbered_heading_harvests_end_to_end() {
        let dir = std::env::temp_dir().join(format!("ql-numbered-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("domain.md"),
            "---\nid: dom-1\ntype: domain\n---\n# Domain\n\n## 3.2 Ubiquitous Language\n\n- **Sprocket** — a streamed unit.\n",
        )
        .unwrap();

        assert_eq!(
            glossary_terms_from_path(&dir),
            vec!["Sprocket".to_string()],
            "a numbered ISO heading must contribute its terms"
        );
        assert_eq!(
            glossary_terms(&Spec::from_path(&dir)),
            vec!["Sprocket".to_string()],
            "and both harvesters must agree"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[trace("TC-823", "FR-044-AC-8")]
    // a glossary-bearing file with no front (CR-055)
    // block is reported, not silently skipped. It stays out of the harvest —
    // the membership rule is unchanged — but the operator is told why the
    // terms are missing instead of meeting `vague-response` on their own
    // domain nouns one indirection later.
    #[test]
    fn tc823_a_frontmatterless_glossary_is_reported() {
        let dir = std::env::temp_dir().join(format!("ql-silent-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let body = "# Project\n\n## Ubiquitous Language\n\n- **Sprocket** — a streamed unit.\n";
        std::fs::write(dir.join("glossary.md"), body).unwrap();
        // A frontmatter-less file with no glossary heading is ordinary and
        // stays quiet — every repo has READMEs.
        std::fs::write(dir.join("README.md"), "# Readme\n\nprose.\n").unwrap();

        let (terms, diagnostics) = glossary_terms_from_path_with_diagnostics(&dir);
        assert!(terms.is_empty(), "membership rule unchanged: {terms:?}");
        assert_eq!(
            diagnostics.len(),
            1,
            "exactly the glossary-bearing non-document is reported: {diagnostics:?}"
        );
        assert!(
            format!("{:?}", diagnostics[0]).contains("glossary.md"),
            "names the path: {:?}",
            diagnostics[0]
        );

        // Give it a front block and both the terms and the silence return.
        std::fs::write(
            dir.join("glossary.md"),
            format!("---\nid: dom-1\ntype: domain\n---\n{body}"),
        )
        .unwrap();
        let (terms, diagnostics) = glossary_terms_from_path_with_diagnostics(&dir);
        assert_eq!(terms, vec!["Sprocket".to_string()]);
        assert!(diagnostics.is_empty(), "{diagnostics:?}");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
