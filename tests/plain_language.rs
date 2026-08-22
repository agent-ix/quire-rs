use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use ix_trace_rs::trace;
use quire_rs::{
    check_plain_language, check_plain_language_at, extract, parse_document, update_section,
    validate_document_in_registry, ExtractionDsl, PlainLanguageProfile, Registry,
};
use tempfile::tempdir;

fn profile() -> PlainLanguageProfile {
    PlainLanguageProfile {
        version: "1.0.0".to_string(),
        document_types: Vec::new(),
        sentence_word_limit: 40,
        max_heading_level_step: 1,
        known_acronyms: BTreeMap::from([(
            "API".to_string(),
            "application programming interface".to_string(),
        )]),
        ignored_uppercase_terms: BTreeSet::new(),
    }
}

fn manifest(profile_body: &str) -> String {
    format!(
        "name: docs\nversion: 1.0.0\narchetypes: []\nplain_language_profiles:\n  docs:\n{profile_body}"
    )
}

#[trace("TC-976", "FR-063-AC-7")]
#[test]
fn typed_profile_loads_and_malformed_profiles_fail_load() {
    let valid = manifest(
        "    version: 1.0.0\n    sentence_word_limit: 30\n    max_heading_level_step: 1\n    known_acronyms:\n      API: application programming interface\n",
    );
    let registry = Registry::from_inline_parts(valid.as_bytes(), &BTreeMap::new()).unwrap();
    assert_eq!(
        registry
            .plain_language_profile("docs")
            .expect("loaded profile")
            .sentence_word_limit,
        30
    );

    for (body, needle) in [
        (
            "    version: ''\n    sentence_word_limit: 30\n    max_heading_level_step: 1\n",
            "version",
        ),
        (
            "    version: 1\n    sentence_word_limit: 0\n    max_heading_level_step: 1\n",
            "sentence_word_limit",
        ),
        (
            "    version: 1\n    sentence_word_limit: 30\n    max_heading_level_step: 0\n",
            "max_heading_level_step",
        ),
        (
            "    version: 1\n    sentence_word_limit: 30\n    max_heading_level_step: 1\n    known_acronyms:\n      api: bad\n",
            "acronym",
        ),
    ] {
        let registry =
            Registry::from_inline_parts(manifest(body).as_bytes(), &BTreeMap::new()).unwrap();
        assert!(
            registry
                .failures()
                .iter()
                .any(|failure| failure.reason.contains(needle)),
            "expected {needle:?} in {:?}",
            registry.failures()
        );
    }
}

#[trace("TC-977", "FR-063-AC-8")]
#[test]
fn profile_merge_is_first_wins_and_has_no_default() {
    let root = tempdir().unwrap();
    for (name, limit) in [("a", 20), ("b", 80)] {
        let module = root.path().join(name);
        fs::create_dir_all(&module).unwrap();
        fs::write(
            module.join("manifest.yaml"),
            format!(
                "name: {name}\narchetypes: []\nplain_language_profiles:\n  shared:\n    version: {name}\n    sentence_word_limit: {limit}\n    max_heading_level_step: 1\n"
            ),
        )
        .unwrap();
    }
    let registry = Registry::load_from(&[root.path()]).unwrap();
    let selected = registry.plain_language_profile("shared").unwrap();
    assert_eq!(selected.version, "a");
    assert_eq!(selected.sentence_word_limit, 20);
    assert!(registry.plain_language_profile("missing").is_none());
}

#[trace("TC-978", "FR-063-AC-9")]
#[test]
fn batch_report_accounts_for_clean_empty_and_non_documents() {
    let root = tempdir().unwrap();
    fs::write(
        root.path().join("clean.md"),
        "---\nid: clean\ntype: Note\n---\n# Clear heading\nShort readable prose.",
    )
    .unwrap();
    fs::write(
        root.path().join("code.md"),
        "---\nid: code\ntype: Note\n---\n```rust\nfn hidden() {}\n```",
    )
    .unwrap();
    fs::write(
        root.path().join("other.md"),
        "---\nid: other\ntype: Other\n---\n# Out of scope\nReadable but not applicable.",
    )
    .unwrap();
    fs::write(root.path().join("note.md"), "no frontmatter").unwrap();
    fs::write(root.path().join("broken.md"), "---\nnot: [valid\n---\nbody").unwrap();

    let mut selected_profile = profile();
    selected_profile.document_types.push("Note".to_string());
    let report = check_plain_language_at(root.path(), "docs", &selected_profile);
    assert_eq!(report.documents_examined, 3);
    assert_eq!(report.readable_documents, 1);
    assert!(report.readable_blocks > 0);
    assert!(report.findings.is_empty(), "{:#?}", report.findings);
    let reasons: Vec<&str> = report
        .skipped_inputs
        .iter()
        .map(|input| input.reason.as_str())
        .collect();
    assert!(reasons.contains(&"no-readable-prose"));
    assert!(reasons.contains(&"no-frontmatter"));
    assert!(reasons.contains(&"malformed-frontmatter"));
    assert!(reasons.contains(&"profile-not-applicable"));

    let empty_root = tempdir().unwrap();
    fs::write(
        empty_root.path().join("code.md"),
        "---\nid: code\ntype: Note\n---\n```\nonly code\n```",
    )
    .unwrap();
    let empty = check_plain_language_at(empty_root.path(), "docs", &selected_profile);
    assert_eq!(empty.readable_blocks, 0);
    assert!(empty.findings.is_empty());
    assert_eq!(empty.skipped_inputs[0].reason, "no-readable-prose");
}

#[trace("TC-981", "FR-063-AC-12")]
#[test]
fn advisory_run_does_not_change_other_engine_results() {
    let manifest = br#"
name: docs
archetypes:
  - name: FR
    grammar_ref: iso-spec-core
plain_language_profiles:
  docs:
    version: 1.0.0
    sentence_word_limit: 40
    max_heading_level_step: 1
"#;
    let registry = Registry::from_inline_parts(manifest, &BTreeMap::new()).unwrap();
    let archetype = registry.archetype("FR").expect("FR archetype");
    let markdown = "---\nid: FR-1\ntype: FR\n---\n# FR-1\n## Description\nThe system shall return a result.\n## Notes\nOriginal body.";
    let parsed_before = parse_document(markdown);
    let validation_before = validate_document_in_registry(&registry, archetype, markdown);
    let writeback_before = update_section(&parsed_before, "Notes", "Replacement").unwrap();
    let dsl: ExtractionDsl = serde_yaml::from_str(
        "yield_pattern:\n  match:\n    description:\n      from: section_body\n      after_heading: Description\n",
    )
    .unwrap();
    let extraction_before = extract(&parsed_before, &dsl).unwrap();

    let selected = registry.plain_language_profile("docs").unwrap();
    let _advice = check_plain_language(Path::new("FR-1.md"), markdown, selected);

    let parsed_after = parse_document(markdown);
    assert_eq!(parsed_before, parsed_after);
    assert_eq!(
        validation_before,
        validate_document_in_registry(&registry, archetype, markdown)
    );
    assert_eq!(
        writeback_before,
        update_section(&parsed_after, "Notes", "Replacement").unwrap()
    );
    assert_eq!(extraction_before, extract(&parsed_after, &dsl).unwrap());
}
