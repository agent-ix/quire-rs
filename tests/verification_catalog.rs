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
///
/// `suffix` is per **test**, not per process: tests in one binary share a pid
/// and run on parallel threads, so a root keyed on the pid alone let one test's
/// `remove_dir_all` race another's `load_from` — which is how TC-853 failed
/// intermittently with two merged methods instead of three (#150).
fn merged(suffix: &str) -> Registry {
    let root =
        std::env::temp_dir().join(format!("quire-rs-catalog-{}-{suffix}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("mkdir");
    for name in ["one", "two"] {
        let dst = root.join(name);
        fs::create_dir_all(&dst).expect("mkdir");
        fs::copy(
            fixture(name).join("manifest.yaml"),
            dst.join("manifest.yaml"),
        )
        .expect("copy");
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
    let registry = merged("845");
    let catalog = registry.verification_catalog().expect("catalog declared");

    // Three ids: two from `one`, one from `two`. The collision did not add one.
    assert_eq!(
        catalog.len(),
        3,
        "{:#?}",
        catalog.keys().collect::<Vec<_>>()
    );
    // First-wins: `catalog-one` loads first (lexicographic search-root order),
    // so its body survives and `catalog-two`'s is skipped.
    assert_eq!(catalog["property-based-testing"].class, "Test");
    assert_eq!(
        catalog["property-based-testing"].name,
        "Property-based testing"
    );
    assert!(catalog.contains_key("fault-injection"));

    let reported: Vec<String> = registry
        .diagnostics()
        .iter()
        .map(|d| d.to_string())
        .filter(|d| d.starts_with("DuplicateVerificationMethod"))
        .collect();
    assert_eq!(reported.len(), 1, "{reported:#?}");
    assert!(
        reported[0].contains("property-based-testing"),
        "{reported:#?}"
    );
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
    let reasons: Vec<&str> = outcome.failures.iter().map(|f| f.reason.as_str()).collect();
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
    let reasons: Vec<&str> = outcome.failures.iter().map(|f| f.reason.as_str()).collect();
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
    let registry = merged("849");
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
        arch.applicability
            .get("forbidden_dependency_edges")
            .unwrap(),
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
    let with = merged("852");

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
    let merged = merged("853");
    assert_eq!(merged.column_vocabulary("verification_method").len(), 3);
}

// TC-874 (FR-054-AC-11): a declared method that is in no catalog is reported.
//
// This is what finally gives the derived vocabularies a reader. Before it,
// `column_vocabulary("verification_method")` was built at registry construction
// and consulted by exactly one test — so a `Verification` cell reading `CI Gate`
// sat in the report looking exactly like one reading `Test`, and quoin's
// conformance check skipped it silently. Measured across the ecosystem when this
// landed: 55 of 577 obligations declared a method no catalog carried.
#[test]
fn tc874_uncatalogued_method_is_reported() {
    let dir = std::env::temp_dir().join(format!("quire-rs-uncat-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let scope = dir.join("spec");
    fs::create_dir_all(&scope).expect("mkdir");
    fs::write(
        scope.join("FR-001.md"),
        "---\nid: FR-001\ntype: FR\ntitle: A requirement\n---\n\n\
         ## Acceptance Criteria\n\n\
         | ID | Criteria | Verification |\n|----|----------|--------------|\n\
         | FR-001-AC-1 | The system shall do it. | Test |\n\
         | FR-001-AC-2 | The system shall also do it. | property-based-testing |\n\
         | FR-001-AC-3 | The system shall keep doing it. | CI Gate |\n\
         | FR-001-AC-4 | The system shall still do it. | CI Gate |\n",
    )
    .expect("write");
    let src = dir.join("src");
    fs::create_dir_all(&src).expect("mkdir");
    fs::write(src.join("lib.rs"), "//! empty\n").expect("write");

    let module = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("traceability")
        .join("obligations-catalog");
    let registry = Registry::load_module(&module).expect("load module");
    let spec = quire_rs::Spec::from_path(&scope);
    let model = registry.traceability().cloned().expect("model");
    let graph = quire_rs::symbols::trace::bind(&quire_rs::symbols::extract_tree(&src), &model);
    let report = quire_rs::coverage::compute(&spec, &registry, &graph, &scope).expect("report");

    let reported: Vec<_> = report
        .diagnostics
        .iter()
        .filter(|d| d.reason == "uncatalogued-verification-method")
        .collect();

    // One diagnostic, not two: the same unknown method on two rows is one
    // decision to make, and a diagnostic per row is how a report stops being
    // read. `Test` (a declared class) and `property-based-testing` (a declared
    // id) are both catalogued, so neither is reported.
    assert_eq!(reported.len(), 1, "{:#?}", report.diagnostics);
    assert!(
        reported[0].message.contains("CI Gate"),
        "{:#?}",
        reported[0]
    );
    assert!(
        reported[0].message.contains("2 row(s)"),
        "{:#?}",
        reported[0]
    );
    assert_eq!(reported[0].declaration, "acceptance-criterion");
    assert_eq!(reported[0].path.as_deref(), Some("FR-001.md"));
    assert!(report
        .to_json()
        .contains("uncatalogued-verification-method"));
}

// TC-875 (FR-054-AC-11): with no catalog declared, the check is silent.
//
// An absent catalog means the question cannot be asked, which is a different
// answer from "yes" — and it is what keeps FR-050-AC-7 byte-identity for every
// module that has not adopted one.
#[test]
fn tc875_no_catalog_asks_no_question() {
    let dir = std::env::temp_dir().join(format!("quire-rs-uncat-none-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let scope = dir.join("spec");
    fs::create_dir_all(&scope).expect("mkdir");
    fs::write(
        scope.join("FR-001.md"),
        "---\nid: FR-001\ntype: FR\ntitle: A requirement\n---\n\n\
         ## Acceptance Criteria\n\n\
         | ID | Criteria | Verification | Priority |\n|----|----------|--------------|----------|\n\
         | FR-001-AC-1 | The system shall do it. | CI Gate | P1 |\n",
    )
    .expect("write");
    let src = dir.join("src");
    fs::create_dir_all(&src).expect("mkdir");
    fs::write(src.join("lib.rs"), "//! empty\n").expect("write");

    let module = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("traceability")
        .join("obligations");
    let registry = Registry::load_module(&module).expect("load module");
    let spec = quire_rs::Spec::from_path(&scope);
    let model = registry.traceability().cloned().expect("model");
    let graph = quire_rs::symbols::trace::bind(&quire_rs::symbols::extract_tree(&src), &model);
    let report = quire_rs::coverage::compute(&spec, &registry, &graph, &scope).expect("report");

    assert!(
        !report
            .diagnostics
            .iter()
            .any(|d| d.reason == "uncatalogued-verification-method"),
        "a module declaring no catalog cannot be told its method is uncatalogued: {:#?}",
        report.diagnostics,
    );
}
