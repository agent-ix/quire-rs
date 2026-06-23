//! Project Ubiquitous-Language term harvesting (FR-044).
//!
//! A repository authors its own concrete vocabulary — a `Glossary` artifact's
//! `## Terms` table, or a `## Ubiquitous Language` section on a domain object.
//! These terms are harvested from a loaded [`Spec`] and composed with the merged
//! module lexicon ([`crate::Registry::lexicon_with`]) so a repo's domain terms
//! are accepted as concrete objects in its own EARS grammar check (FR-042/043).

use crate::ast::QuireDocument;
use crate::corpus::Spec;
use crate::query::{self, ListPattern};

/// Harvest the repo's project glossary terms from a loaded `Spec`: the `Term`
/// (or `Name`) column of every `## Terms` table, and the bold term of every
/// `## Ubiquitous Language` bullet (and any `## Ubiquitous Language` table).
/// Deduplicated and sorted; a corpus with no glossary yields an empty vec.
pub fn glossary_terms(spec: &Spec) -> Vec<String> {
    let mut terms: Vec<String> = Vec::new();
    for doc in &spec.inner.documents {
        // `## Terms` table — a `Glossary` artifact.
        push_table_terms(&doc.doc, "Terms", &mut terms);
        // `## Ubiquitous Language` — bullet form (`- **Term** — …`) …
        if let Some(section) = query::section(&doc.doc, "Ubiquitous Language") {
            for item in
                query::parse_bullet_list(&section.content, Some(ListPattern::BoldDescription))
            {
                push_term(&item.title, &mut terms);
            }
        }
        // … and/or table form under the same heading.
        push_table_terms(&doc.doc, "Ubiquitous Language", &mut terms);
    }
    terms.sort();
    terms.dedup();
    terms
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

    // TC-674 (FR-044-AC-1): harvest the Term column from a `## Terms` table.
    #[test]
    fn tc674_harvest_terms_table() {
        let terms = harvest_fixture("674");
        assert!(terms.contains(&"Widget".to_string()));
        assert!(terms.contains(&"Sprocket".to_string()));
    }

    // TC-675 (FR-044-AC-2): harvest the bold term from `## Ubiquitous Language`.
    #[test]
    fn tc675_harvest_ubiquitous_language() {
        let terms = harvest_fixture("675");
        assert!(terms.contains(&"Place".to_string()));
        assert!(terms.contains(&"Capture".to_string()));
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
        );
        let _ = std::fs::remove_dir_all(&base);
        report
    }

    // TC-678 (FR-044-AC-5): validate_bundle harvests the Spec's project terms
    // and applies the combined lexicon — the FR's `widget` is suppressed.
    #[test]
    fn tc678_validate_bundle_harvests_and_applies() {
        let report = glossary_bundle_report("678");
        assert!(!report
            .warnings
            .iter()
            .any(|w| w.message.contains("[ears:vague-response]")));
    }

    // TC-679 (FR-044-AC-6): the project-glossary suppression is advisory —
    // grammar findings never become bundle errors.
    #[test]
    fn tc679_validate_bundle_grammar_is_advisory() {
        let report = glossary_bundle_report("679");
        assert!(!report.errors.iter().any(|e| e.reason == "grammar"));
    }

    // TC-680 (FR-044-AC-7): a repo with no glossary harvests nothing.
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
}
