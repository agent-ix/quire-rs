use std::collections::BTreeMap;
use std::path::Path;

use ix_trace_rs::trace;
use quire_bench::collection::{collect, Collection, RawEvidence};
use quire_bench::export::build_collection;
use quire_bench::{Manifest, Observed};
use serde_json::{json, Number};

#[trace("FR-050-AC-30")]
#[test]
fn collection_is_plan_derived_and_complete() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let manifest: Manifest =
        serde_json::from_str(include_str!("../../../bench/manifest.json")).unwrap();
    let observations = manifest
        .metrics
        .keys()
        .map(|name| (name.clone(), Number::from(1)))
        .collect();
    let measurements = Collection {
        observed: Observed::from([("fixture".to_owned(), observations)]),
        raw_evidence: BTreeMap::from([(
            "fixture".to_owned(),
            RawEvidence {
                module: None,
                payload: json!({"totals": {"backed": 1}}),
            },
        )]),
    };
    let collection = build_collection(
        &root,
        &manifest,
        measurements,
        "2026-08-27T00:00:00.000Z",
        Path::new("/consumer"),
        None,
    )
    .unwrap();
    let value = serde_json::to_value(collection).unwrap();
    assert_eq!(value["collectionId"], "quire-bench-20260827000000000");
    assert_eq!(value["schemaVersion"], 2);
    assert_eq!(
        value["rawEvidence"]["fixture"]["payload"]["totals"]["backed"],
        1
    );
    assert!(value["observations"].as_array().unwrap().iter().any(|row| {
        row["metric"] == "coverage.dead_tags"
            && row["planId"] == "MP-203"
            && row["definitionVersion"] == "coverage.dead-tags-v1"
            && row["shape"] == "count"
            && row["population"]["complete"] == true
    }));
}

#[trace("FR-050-AC-30")]
#[test]
fn governed_collection_refuses_unreadable_corpus() {
    let temp = tempfile::tempdir().unwrap();
    let mut manifest: Manifest =
        serde_json::from_str(include_str!("../../../bench/manifest.json")).unwrap();
    manifest.corpora.truncate(1);
    manifest.corpora[0].path = "missing".to_owned();
    manifest.corpora[0].module = Some("missing-module".to_owned());
    manifest.corpora[0].require_clean = true;
    let error = collect(temp.path(), &manifest, Path::new("/no-engine"), None, true)
        .err()
        .unwrap()
        .to_string();
    assert!(error.contains("specific-shape-control"));
    assert!(error.contains("absent"));
}
