//! FR-061 — combinatorial obligations (TC-925..TC-930).

use quire_rs::combinatorial::{
    parse_exclusion, split_values, ConfigurationSpace, Dimension, Exclusion,
};

fn dim(name: &str, values: &[&str]) -> Dimension {
    Dimension {
        name: name.to_string(),
        values: values.iter().map(|v| (*v).to_string()).collect(),
    }
}

fn space(dimensions: Vec<Dimension>, exclusions: Vec<Exclusion>) -> ConfigurationSpace {
    ConfigurationSpace {
        dimensions,
        exclusions,
    }
}

// TC-925, FR-061-AC-1: the t-way tuple count is the sum over every set of t
// dimensions of the product of their value counts.
//
// Hand-computable on purpose. Sizes 2, 3, 2 at t=2 give 2·3 + 2·2 + 3·2 = 16.
// A number an obligation rests on must be one a reader can check by hand on a
// small case, or nobody will ever check it on a large one.
#[test]
fn tc925_the_tuple_count_is_the_declared_number() {
    let s = space(
        vec![
            dim("policy", &["tolerant", "strict"]),
            dim("features", &["default", "python", "wasm"]),
            dim("target", &["linux", "wasm32"]),
        ],
        vec![],
    );
    assert_eq!(s.tuples(2), 16);
    // 3-way over three dimensions is the full product: 2·3·2 = 12.
    assert_eq!(s.tuples(3), 12);
    // 1-way is every value: 2 + 3 + 2 = 7.
    assert_eq!(s.tuples(1), 7);
}

// TC-926, FR-061-AC-2: a strength greater than the number of dimensions is 0,
// not an error and not the full product.
//
// There is no 3-way interaction among two dimensions. Reporting one would put
// an obligation on the spec that no test run could ever discharge.
#[test]
fn tc926_strength_beyond_the_dimensions_is_zero() {
    let s = space(vec![dim("a", &["1", "2"]), dim("b", &["x", "y"])], vec![]);
    assert_eq!(s.tuples(3), 0);
    assert_eq!(s.tuples(0), 0);
    assert_eq!(s.tuples(2), 4);
}

// TC-927, FR-061-AC-3: a forbidden combination is removed from the count.
//
// Counting a combination that cannot exist makes the target permanently
// unreachable, which is the fastest way to get a coverage number ignored.
#[test]
fn tc927_a_forbidden_combination_is_not_an_obligation() {
    let dims = vec![
        dim("features", &["default", "python", "wasm"]),
        dim("target", &["linux", "wasm32"]),
    ];
    assert_eq!(space(dims.clone(), vec![]).tuples(2), 6);

    // `python` cannot be built for `wasm32`.
    let excluded = space(
        dims,
        vec![Exclusion {
            assignments: vec![
                ("features".to_string(), "python".to_string()),
                ("target".to_string(), "wasm32".to_string()),
            ],
        }],
    );
    assert_eq!(excluded.tuples(2), 5);
}

// TC-928, FR-061-AC-4: an exclusion forbids every WIDER tuple containing it.
//
// A two-value constraint has to bite at strength 3 as well, or a space would
// become less constrained as the strength rises — which is backwards.
#[test]
fn tc928_an_exclusion_bites_at_higher_strength() {
    let dims = vec![
        dim("features", &["default", "python"]),
        dim("target", &["linux", "wasm32"]),
        dim("policy", &["tolerant", "strict"]),
    ];
    assert_eq!(space(dims.clone(), vec![]).tuples(3), 8);
    let excluded = space(
        dims,
        vec![Exclusion {
            assignments: vec![
                ("features".to_string(), "python".to_string()),
                ("target".to_string(), "wasm32".to_string()),
            ],
        }],
    );
    // The two 3-way tuples containing both forbidden values are gone.
    assert_eq!(excluded.tuples(3), 6);
}

// TC-929, FR-061-AC-5: the statement carries every declared value, so any
// change to the space changes its hash.
//
// This is the entire suspect-link mechanism, inherited rather than reinvented:
// adding a value to a dimension really does invalidate a coverage claim made
// before that value existed.
#[test]
fn tc929_the_statement_covers_the_whole_space() {
    let before = space(vec![dim("policy", &["tolerant", "strict"])], vec![]);
    let after = space(vec![dim("policy", &["tolerant", "strict", "warn"])], vec![]);
    assert_ne!(before.statement(2), after.statement(2));
    assert!(before.statement(2).contains("2-way over"));
    // Strength is part of the statement: promoting 2-way to 3-way is a
    // different obligation, not the same one measured differently.
    assert_ne!(before.statement(2), before.statement(3));
}

// TC-930, FR-061-AC-6: declared cells are parsed the way a spec author writes
// them, and a repeated value is not counted twice.
//
// A duplicate would inflate every tuple count its dimension takes part in, so
// the obligation would demand coverage of combinations that do not exist —
// the same defect a forbidden combination causes, arriving by typo.
#[test]
fn tc930_cells_parse_as_authored() {
    assert_eq!(
        split_values("`tolerant`, strict , tolerant"),
        vec!["tolerant", "strict"]
    );
    assert_eq!(split_values(" , "), Vec::<String>::new());

    let exclusion = parse_exclusion("features=python & target=wasm32").expect("parsed");
    assert_eq!(exclusion.assignments.len(), 2);
    // A single assignment is not an interaction constraint — it says a value is
    // never used, which is a shorter values list. Two ways of saying one thing
    // is how they come to disagree.
    assert!(parse_exclusion("features=python").is_none());
    assert!(parse_exclusion("").is_none());
    assert!(parse_exclusion("nonsense & target=wasm32").is_none());
}

// TC-931, FR-061-AC-7: a declared configuration space mints ONE obligation
// through the real minter, carrying the number in `parameters`.
//
// Stated over `obligation::for_document`, the function every consumer of the
// FR-055 contract reaches — not over the counting helper, which TC-925 already
// covers. A number nothing surfaces is a number nobody can act on.
#[test]
fn tc931_a_declared_space_mints_one_obligation() {
    use quire_rs::traceability::{CombinatorialColumns, ObligationSource, TraceabilityModel};

    let model = TraceabilityModel {
        obligations: vec![ObligationSource {
            name: "configuration-space".to_string(),
            target: None,
            archetype: Some("FR".to_string()),
            section: Some("Configuration Dimensions".to_string()),
            id_format: Some("{document}-COMB".to_string()),
            exclude: vec![],
            statement_column: "Dimension".to_string(),
            method_column: None,
            criticality_column: None,
            parameters: Default::default(),
            combinatorial: Some(CombinatorialColumns {
                dimension_column: "Dimension".to_string(),
                values_column: "Values".to_string(),
                excludes_column: Some("Excludes".to_string()),
                strength: 2,
            }),
        }],
        ..TraceabilityModel::default()
    };
    model.validate().expect("a well-formed source loads");

    let text = "---\nid: FR-001\ntype: FR\ntitle: A requirement\n---\n\n\
        ## Configuration Dimensions\n\n\
        | Dimension | Values | Excludes |\n\
        |---|---|---|\n\
        | features | default, python, wasm | features=python & target=wasm32 |\n\
        | target | linux, wasm32 | |\n\
        | policy | tolerant, strict | |\n";
    let doc = quire_rs::parse_document(text);
    let out = quire_rs::obligation::for_document(&model, "FR", &doc, None);

    // ONE obligation for the whole table, not one per dimension.
    assert_eq!(out.len(), 1, "{out:?}");
    let obligation = out.get("FR-001-COMB").expect("id rendered from id_format");
    assert_eq!(
        obligation.parameters.get("strength").map(String::as_str),
        Some("2")
    );
    assert_eq!(
        obligation.parameters.get("dimensions").map(String::as_str),
        Some("3")
    );
    // 3·2 + 3·2 + 2·2 = 16 pairs, less the one forbidden by the exclusion.
    assert_eq!(
        obligation.parameters.get("tuples").map(String::as_str),
        Some("15")
    );
    assert!(!obligation.statement_hash.is_empty());
}

// TC-932, FR-061-AC-8: a space with fewer than two real dimensions mints
// nothing.
//
// A one-dimension "space" takes part in no interaction. Minting an obligation
// for it would put a permanently-satisfied row in the report that reads exactly
// like a real one — the failure mode this whole program exists to remove.
#[test]
fn tc932_a_space_with_no_interaction_mints_nothing() {
    use quire_rs::traceability::{CombinatorialColumns, ObligationSource, TraceabilityModel};

    let model = TraceabilityModel {
        obligations: vec![ObligationSource {
            name: "configuration-space".to_string(),
            target: None,
            archetype: Some("FR".to_string()),
            section: Some("Configuration Dimensions".to_string()),
            id_format: Some("{document}-COMB".to_string()),
            exclude: vec![],
            statement_column: "Dimension".to_string(),
            method_column: None,
            criticality_column: None,
            parameters: Default::default(),
            combinatorial: Some(CombinatorialColumns {
                dimension_column: "Dimension".to_string(),
                values_column: "Values".to_string(),
                excludes_column: None,
                strength: 2,
            }),
        }],
        ..TraceabilityModel::default()
    };

    // One dimension with two values, and one "dimension" with a single value —
    // which is a constant, not an axis.
    let text = "---\nid: FR-002\ntype: FR\ntitle: A requirement\n---\n\n\
        ## Configuration Dimensions\n\n\
        | Dimension | Values |\n\
        |---|---|\n\
        | policy | tolerant, strict |\n\
        | mode | only-one |\n";
    let doc = quire_rs::parse_document(text);
    assert!(quire_rs::obligation::for_document(&model, "FR", &doc, None).is_empty());
}

// TC-933, FR-061-AC-9: strength 0 is rejected at load.
#[test]
fn tc933_strength_zero_is_rejected() {
    use quire_rs::traceability::{CombinatorialColumns, ObligationSource, TraceabilityModel};
    let model = TraceabilityModel {
        obligations: vec![ObligationSource {
            name: "configuration-space".to_string(),
            target: None,
            archetype: Some("FR".to_string()),
            section: Some("Configuration Dimensions".to_string()),
            id_format: Some("{document}-COMB".to_string()),
            exclude: vec![],
            statement_column: "Dimension".to_string(),
            method_column: None,
            criticality_column: None,
            parameters: Default::default(),
            combinatorial: Some(CombinatorialColumns {
                dimension_column: "Dimension".to_string(),
                values_column: "Values".to_string(),
                excludes_column: None,
                strength: 0,
            }),
        }],
        ..TraceabilityModel::default()
    };
    let err = model.validate().expect_err("strength 0 is rejected");
    assert!(err.contains("strength 0"), "{err}");
}

// TC-934, FR-061-AC-10: a declared space mints ONE obligation through the
// CORPUS path too, identically to the single-document path.
//
// TC-931 states the same contract over `obligation::for_document` and calls it
// "the function every consumer of the FR-055 contract reaches". That was wrong,
// and the wrongness is the whole reason this test exists: `quire coverage` —
// the surface quoin actually reads — calls `obligation::derive`, which had no
// combinatorial branch at all. A declared configuration matrix therefore minted
// one obligation PER DIMENSION ROW, the exact shape the source exists to
// replace, and quoin FR-035 could never see a combinatorial obligation however
// the module was declared (CR-076).
//
// The two paths must agree on more than arity: an obligation minted by one and
// read by the other has to be the SAME obligation, or a binding made during
// validation would not match the one coverage reports.
#[test]
fn tc934_the_corpus_path_mints_the_same_one_obligation() {
    use quire_rs::corpus::Spec;
    use quire_rs::traceability::{CombinatorialColumns, ObligationSource, TraceabilityModel};

    let model = TraceabilityModel {
        obligations: vec![ObligationSource {
            name: "configuration-space".to_string(),
            target: None,
            archetype: Some("FR".to_string()),
            section: Some("Configuration Dimensions".to_string()),
            id_format: Some("{document}-COMB".to_string()),
            exclude: vec![],
            statement_column: "Dimension".to_string(),
            method_column: None,
            criticality_column: None,
            parameters: Default::default(),
            combinatorial: Some(CombinatorialColumns {
                dimension_column: "Dimension".to_string(),
                values_column: "Values".to_string(),
                excludes_column: Some("Excludes".to_string()),
                strength: 2,
            }),
        }],
        ..TraceabilityModel::default()
    };
    model.validate().expect("a well-formed source loads");

    let text = "---\nid: FR-001\ntype: FR\ntitle: A requirement\n---\n\n\
        ## Configuration Dimensions\n\n\
        | Dimension | Values | Excludes |\n\
        |---|---|---|\n\
        | features | default, python, wasm | features=python & target=wasm32 |\n\
        | target | linux, wasm32 | |\n\
        | policy | tolerant, strict | |\n";

    let mut root = std::env::temp_dir();
    root.push(format!(
        "quire_comb_derive_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("FR-001.md"), text).unwrap();

    let spec = Spec::from_path(&root);
    let (obligations, _skipped) = quire_rs::obligation::derive(&spec, &root, &model);

    // ONE for the whole table, not one per dimension row.
    assert_eq!(obligations.len(), 1, "{obligations:?}");
    let obligation = &obligations[0];
    assert_eq!(obligation.id, "FR-001-COMB");
    assert_eq!(obligation.source, "configuration-space");

    // The same numbers `for_document` carries — 3·2 + 3·2 + 2·2 = 16 pairs,
    // less the one the exclusion forbids.
    assert_eq!(
        obligation.parameters.get("strength").map(String::as_str),
        Some("2")
    );
    assert_eq!(
        obligation.parameters.get("dimensions").map(String::as_str),
        Some("3")
    );
    assert_eq!(
        obligation.parameters.get("tuples").map(String::as_str),
        Some("15")
    );

    // And the same statement hash, which is what makes a binding portable
    // between the two paths.
    let doc = quire_rs::parse_document(text);
    let by_document = quire_rs::obligation::for_document(&model, "FR", &doc, None);
    let twin = by_document
        .get("FR-001-COMB")
        .expect("the single-document path mints the same id");
    assert_eq!(obligation.statement_hash, twin.statement_hash);

    std::fs::remove_dir_all(&root).ok();
}
