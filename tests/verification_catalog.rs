//! FR-054 — the verification-method catalog (TC-844..TC-853).

use std::fs;
use std::path::PathBuf;

use quire_rs::Registry;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("catalog")
        .join(name)
}

fn registry(name: &str) -> Registry {
    Registry::load_module(&fixture(name)).expect("load module")
}

/// Both fixture modules under one search root, so the merge runs for real.
fn merged() -> Registry {
    let root = std::env::temp_dir().join(format!("quire-rs-catalog-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("mkdir");
    for name in ["one", "two"] {
        let dst = root.join(name);
        fs::create_dir_all(&dst).expect("mkdir");
        fs::copy(fixture(name).join("manifest.yaml"), dst.join("manifest.yaml")).expect("copy");
    }
    Registry::load_from(&[root.as_path()]).expect("load merged")
}

// TC-844 (FR-054-AC-1): every declared field survives to the accessor.
#[test]
fn tc844_catalog_entries_are_exposed_intact() {
    let registry = registry("one");
    let catalog = registry.verification_catalog().expect("catalog declared");

    assert_eq!(catalog.len(), 2, "{catalog:#?}");
    let pbt = &catalog["property-based-testing"];
    assert_eq!(pbt.name, "Property-based testing");
    assert_eq!(pbt.class, "Test");
    assert!(pbt.definition.contains("generated inputs"));
    assert_eq!(pbt.evidence_kind.as_deref(), Some("test-run"));
    assert_eq!(
        pbt.applicability.get("property_shapes").unwrap(),
        &vec![
            "round-trip".to_string(),
            "idempotence".to_string(),
            "invariant".to_string()
        ]
    );
    assert_eq!(
        pbt.tooling,
        vec![
            "proptest".to_string(),
            "fast-check".to_string(),
            "hypothesis".to_string()
        ]
    );
}

// TC-845 (FR-054-AC-2): a colliding id merges first-wins and is reported.
#[test]
fn tc845_duplicate_method_is_first_wins_and_reported() {
    let registry = merged();
    let catalog = registry.verification_catalog().expect("catalog declared");

    // Three ids: two from `one`, one from `two`. The collision did not add one.
    assert_eq!(catalog.len(), 3, "{:#?}", catalog.keys().collect::<Vec<_>>());
    // First-wins: `catalog-one` loads first (lexicographic search-root order),
    // so its body survives and `catalog-two`'s is skipped.
    assert_eq!(catalog["property-based-testing"].class, "Test");
    assert_eq!(catalog["property-based-testing"].name, "Property-based testing");
    assert!(catalog.contains_key("fault-injection"));

    let reported: Vec<String> = registry
        .diagnostics()
        .iter()
        .map(|d| d.to_string())
        .filter(|d| d.starts_with("DuplicateVerificationMethod"))
        .collect();
    assert_eq!(reported.len(), 1, "{reported:#?}");
    assert!(reported[0].contains("property-based-testing"), "{reported:#?}");
    assert!(reported[0].contains("catalog-one"), "{reported:#?}");
    assert!(reported[0].contains("catalog-two"), "{reported:#?}");
}

// TC-846 (FR-054-AC-3): undeclared is None, not an empty map.
#[test]
fn tc846_no_catalog_is_undeclared_not_empty() {
    assert!(
        registry("none").verification_catalog().is_none(),
        "a module declaring no catalog must read as undeclared, not as empty",
    );
}

// TC-847 (FR-054-AC-4): an unknown key fails load naming the key.
#[test]
fn tc847_unknown_key_fails_load() {
    let outcome = quire_rs::loader::load_single_module(&fixture("bad-key"));
    let reasons: Vec<&str> = outcome
        .failures
        .iter()
        .map(|f| f.reason.as_str())
        .collect();
    assert!(
        reasons.iter().any(|r| r.contains("evidence_knid")),
        "the typo must be named, not discarded: {reasons:#?}",
    );
    assert!(outcome.modules.is_empty());
}

// TC-848 (FR-054-AC-5): an empty required field fails load naming the method.
#[test]
fn tc848_empty_required_field_fails_load() {
    let outcome = quire_rs::loader::load_single_module(&fixture("empty-field"));
    let reasons: Vec<&str> = outcome
        .failures
        .iter()
        .map(|f| f.reason.as_str())
        .collect();
    assert!(
        reasons
            .iter()
            .any(|r| r.contains("smoke-testing") && r.contains("definition")),
        "the offending method and field must both be named: {reasons:#?}",
    );
    assert!(outcome.modules.is_empty());
}

// TC-849 (FR-054-AC-6): the two derived vocabularies come from the catalog.
#[test]
fn tc849_derived_vocabularies_come_from_the_catalog() {
    let registry = merged();
    assert_eq!(
        registry.column_vocabulary("verification_method"),
        [
            "architecture-conformance".to_string(),
            "fault-injection".to_string(),
            "property-based-testing".to_string()
        ]
    );
    // Distinct classes, each once, sorted. `catalog-two`'s Demonstration lost
    // the merge, so it is absent — the vocabulary tracks the merged catalog.
    assert_eq!(
        registry.column_vocabulary("verification_class"),
        ["Analysis".to_string(), "Test".to_string()]
    );
}

// TC-850 (FR-054-AC-7): `test_type` is unchanged; an unknown name is empty.
#[test]
fn tc850_test_type_unchanged_and_unknown_name_is_empty() {
    let none = registry("none");
    assert_eq!(
        none.column_vocabulary("test_type"),
        [
            "Unit".to_string(),
            "Integration".to_string(),
            "Property".to_string()
        ]
    );
    assert!(none.column_vocabulary("no-such-vocabulary").is_empty());
    // A module with a catalog but no traceability vocabulary answers the
    // derived names and not `test_type` — the three are independent.
    let one = registry("one");
    assert!(one.column_vocabulary("test_type").is_empty());
    assert!(!one.column_vocabulary("verification_method").is_empty());
}

// TC-851 (FR-054-AC-8): applicability is carried verbatim and never
// interpreted — including a rule name the engine has never heard of (CON-2).
#[test]
fn tc851_applicability_is_opaque() {
    let catalog = registry("one");
    let catalog = catalog.verification_catalog().unwrap();

    let arch = &catalog["architecture-conformance"];
    assert_eq!(
        arch.applicability.get("forbidden_dependency_edges").unwrap(),
        &vec!["*".to_string()],
        "an unknown rule name must survive verbatim",
    );
    // And an entry declaring none carries an empty map rather than a default.
    let two = registry("two");
    let two = two.verification_catalog().unwrap();
    assert!(two["property-based-testing"].applicability.is_empty());
}

// TC-852 (FR-054-AC-9): declaring a catalog moves no finding (CON-3).
#[test]
fn tc852_catalog_changes_no_finding() {
    let doc = "---\nid: FR-001\ntype: FR\ntitle: A requirement\n---\n\n\
               ## Description\n\nshall process the input.\n\n\
               ## Acceptance Criteria\n\n\
               | ID | Criteria | Verification |\n|----|----------|--------------|\n\
               | FR-001-AC-1 | The system shall support pagination. | Test |\n";

    let without = registry("none");
    let with = merged();

    // Both registries declare an `FR` archetype; only `catalog-none` gives it a
    // grammar_ref, so that registry is the one with findings to move. The
    // catalog registry must produce no error and no catalog-shaped warning.
    let arch_without = without.archetype("FR").expect("FR");
    let a = quire_rs::validate_document_in_registry(&without, arch_without, doc);
    let arch_with = with.archetype("FR").expect("FR");
    let b = quire_rs::validate_document_in_registry(&with, arch_with, doc);

    assert!(
        b.errors.is_empty(),
        "a catalog must contribute no error: {:#?}",
        b.errors
    );
    for finding in a.warnings.iter().chain(b.warnings.iter()) {
        assert!(
            !finding.message.contains("verification_catalog")
                && !finding.message.contains("property-based-testing"),
            "the catalog leaked into a finding: {finding:#?}",
        );
    }
}

// TC-853 (FR-054-AC-10): the derived vocabularies are never read from a
// separate declaration (CON-4).
#[test]
fn tc853_derived_vocabularies_need_no_separate_declaration() {
    let one = registry("one");
    // `catalog-one` declares a catalog and no `vocabularies:` block at all.
    assert!(one.column_vocabulary("test_type").is_empty());
    assert_eq!(
        one.column_vocabulary("verification_method"),
        [
            "architecture-conformance".to_string(),
            "property-based-testing".to_string()
        ]
    );
    // And they move when the catalog moves: merging in a second module adds
    // exactly the id that module contributed.
    let merged = merged();
    assert_eq!(merged.column_vocabulary("verification_method").len(), 3);
}
