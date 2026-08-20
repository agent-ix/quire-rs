//! FR-060 — dereference a named vocabulary instead of restating it
//! (TC-919..TC-923).

use std::fs;
use std::path::PathBuf;

use quire_rs::Registry;

fn tmpdir(tag: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("quire_vocabrefs_{tag}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).expect("mkdir");
    p
}

/// A module whose TestMatrix contract NAMES the vocabulary rather than
/// restating it, and whose traceability model declares it.
fn module(tag: &str, assert_block: &str) -> PathBuf {
    module_with(tag, "table_row", "under_section: Cases", assert_block)
}

/// Same, over a chosen locator kind — `from_vocabulary` is a SCALAR constraint
/// and is illegal on `table_row`, exactly as `choices` is (CR-010).
fn module_scalar(tag: &str, assert_block: &str) -> PathBuf {
    module_with(tag, "section_body", "after_heading: Cases", assert_block)
}

fn module_with(tag: &str, kind: &str, locator_key: &str, assert_block: &str) -> PathBuf {
    let dir = tmpdir(tag);
    fs::write(
        dir.join("manifest.yaml"),
        format!(
            "name: vocab-refs\n\
             version: 0.1.0\n\
             artifact_types:\n\
             - name: TestMatrix\n  \
                 body_extraction:\n    \
                   yield_pattern:\n      \
                     match:\n        \
                       rows:\n          \
                         from: {kind}\n          \
                         {locator_key}\n{assert_block}\n\
             traceability:\n  \
               vocabularies:\n    \
                 test_type: [Unit, Integration, Property]\n"
        ),
    )
    .expect("write manifest");
    dir
}

fn matrix_assert(registry: &Registry) -> quire_rs::extract::locator::LocatorAssert {
    let archetype = registry.archetype("TestMatrix").expect("TestMatrix");
    let dsl = archetype.body_extraction().expect("dsl");
    let locator = dsl
        .yield_pattern
        .r#match
        .as_ref()
        .expect("match")
        .get("rows")
        .expect("rows");
    locator.canonical().assert().expect("assert").clone()
}

// TC-919, FR-060-AC-1: `column_vocabularies` resolves to the declared values.
//
// The contract names `test_type`; the values come from the traceability model
// and appear nowhere in the archetype. That is the whole point — a second copy
// of the list is what this FR removes.
#[test]
fn tc919_a_named_column_vocabulary_resolves() {
    let dir = module(
        "919",
        "          assert:\n            column_vocabularies:\n              Type: test_type\n",
    );
    let registry = Registry::load_module(&dir).expect("load");
    let assert = matrix_assert(&registry);

    assert_eq!(
        assert.column_choices.as_ref().and_then(|c| c.get("Type")),
        Some(&vec![
            "Unit".to_string(),
            "Integration".to_string(),
            "Property".to_string()
        ]),
        "the named vocabulary is dereferenced into literal choices"
    );
    // The reference is consumed, so nothing downstream has to understand it.
    assert!(assert.column_vocabularies.is_none());
    fs::remove_dir_all(&dir).ok();
}

// TC-920, FR-060-AC-2: `from_vocabulary` resolves the scalar counterpart.
#[test]
fn tc920_a_named_scalar_vocabulary_resolves() {
    let dir = module_scalar(
        "920",
        "          assert:\n            from_vocabulary: test_type\n",
    );
    let registry = Registry::load_module(&dir).expect("load");
    let assert = matrix_assert(&registry);

    assert_eq!(
        assert.choices,
        Some(vec![
            "Unit".to_string(),
            "Integration".to_string(),
            "Property".to_string()
        ])
    );
    assert!(assert.from_vocabulary.is_none());
    fs::remove_dir_all(&dir).ok();
}

// TC-921, FR-060-AC-3: a name nothing declares resolves to an EMPTY choice
// set, not to "no constraint".
//
// The distinction matters. Dropping the constraint would let a typo silently
// WIDEN the contract — every value would pass — which is the same
// quiet-wrong-answer class as CR-075's dead `from`. An empty set fails every
// cell instead, which is loud and diagnosable.
#[test]
fn tc921_an_unknown_vocabulary_is_empty_not_absent() {
    let dir = module_scalar(
        "921",
        "          assert:\n            from_vocabulary: no_such_vocabulary\n",
    );
    let registry = Registry::load_module(&dir).expect("load");
    let assert = matrix_assert(&registry);

    assert_eq!(
        assert.choices,
        Some(Vec::new()),
        "an unknown name is an empty constraint, never an absent one"
    );
    fs::remove_dir_all(&dir).ok();
}

// TC-922, FR-060-AC-4: a literal beside a reference wins, and the reference is
// dropped rather than merged.
//
// Two sources for one constraint is the duplication this FR removes. Unioning
// them would recreate it inside a single assert, where it is even harder to see.
#[test]
fn tc922_a_literal_wins_over_a_reference() {
    let dir = module_scalar(
        "922",
        "          assert:\n            from_vocabulary: test_type\n            \
         choices: [OnlyThis]\n",
    );
    let registry = Registry::load_module(&dir).expect("load");
    let assert = matrix_assert(&registry);

    assert_eq!(assert.choices, Some(vec!["OnlyThis".to_string()]));
    assert!(assert.from_vocabulary.is_none());
    fs::remove_dir_all(&dir).ok();
}

// TC-923, FR-060-AC-5: an archetype naming no vocabulary is untouched.
//
// The byte-identity guarantee. A module that has not adopted this must compile
// to exactly what it compiled to before the FR existed.
#[test]
fn tc923_an_archetype_naming_nothing_is_unchanged() {
    let dir = module(
        "923",
        "          assert:\n            column_choices:\n              Type: [Unit]\n",
    );
    let registry = Registry::load_module(&dir).expect("load");
    let assert = matrix_assert(&registry);

    assert_eq!(
        assert.column_choices.as_ref().and_then(|c| c.get("Type")),
        Some(&vec!["Unit".to_string()]),
        "the literal is preserved exactly"
    );
    assert!(assert.from_vocabulary.is_none() && assert.column_vocabularies.is_none());
    fs::remove_dir_all(&dir).ok();
}

// TC-924, FR-060-AC-6: a reference obeys the same per-locator-kind rules as
// the literal it stands in for.
//
// Found by a test failure rather than by design: `choices` is illegal on
// `table_row` (CR-010) and the first implementation let `from_vocabulary` sit
// there freely. A reference that could sit where its literal could not is a way
// to smuggle a scalar constraint onto a table, where nothing would enforce it —
// declared and unenforceable is worse than rejected.
#[test]
fn tc924_a_reference_obeys_its_literal_s_kind_rules() {
    // `from_vocabulary` is scalar: illegal on a table row.
    let bad_scalar = module(
        "924a",
        "          assert:\n            from_vocabulary: test_type\n",
    );
    let registry = Registry::load_module(&bad_scalar).expect("module still loads");
    assert!(
        registry.archetype("TestMatrix").is_none(),
        "from_vocabulary on table_row is rejected, as `choices` is"
    );
    assert!(
        registry
            .failures()
            .iter()
            .any(|f| format!("{f:?}").contains("from_vocabulary")),
        "and the failure names the offending key: {:?}",
        registry.failures()
    );

    // `column_vocabularies` is table-only: illegal on a scalar locator.
    let bad_column = module_scalar(
        "924b",
        "          assert:\n            column_vocabularies:\n              Type: test_type\n",
    );
    let registry = Registry::load_module(&bad_column).expect("module still loads");
    assert!(
        registry.archetype("TestMatrix").is_none(),
        "column_vocabularies on section_body is rejected, as `column_choices` is"
    );
    fs::remove_dir_all(&bad_scalar).ok();
    fs::remove_dir_all(&bad_column).ok();
}
