use std::path::Path;

use ix_trace_rs::trace;
use quire_rs::Registry;

const BASELINE_ARTIFACT_TYPE_COUNT: usize = 28;
const BASELINE_REGISTRY_COUNT: usize = BASELINE_ARTIFACT_TYPE_COUNT + 1;

#[trace("FR-013-AC-4", "NFR-007-AC-1")]
#[test]
fn baseline_artifact_corpus_loads_all_pinned_artifact_types() {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/modules/baseline-artifacts");
    let registry = Registry::load_from(&[root.as_path()]).expect("load pinned baseline");
    let names: Vec<_> = registry.archetype_names().collect();

    assert_eq!(names.len(), BASELINE_REGISTRY_COUNT, "{names:#?}");
    for name in [
        "FR",
        "NFR",
        "StR",
        "US",
        "IT",
        "TC",
        "master-requirements",
        "index",
        "Glossary",
        "ApplicationSpec",
        "IntendedUse",
        "FeatureRiskRating",
        "ADR",
        "Anomaly",
        "DossierDocument",
        "Plan",
        "Task",
        "Review",
        "SpecReview",
        "Finding",
        "Feedback",
        "TestMatrixIndex",
        "TestMatrix",
        "Standard",
        "SuiteRegistry",
        "Inspections",
        "RoleSpec",
        "Rubric",
    ] {
        assert!(
            registry.archetype(name).is_some(),
            "missing artifact type {name}"
        );
    }
    assert!(
        registry.archetype("standard").is_some(),
        "missing module archetype"
    );
}
