//! Gate G5 — corpus correctness, dogfooded on quire-rs's own `spec/`.
//!
//! Loads this repository's real spec tree (~56 cross-referencing
//! artifacts) into a `Spec`, resolves the references, and asserts facts
//! a healthy spec must satisfy. This is living regression coverage: if
//! `load_repo`/resolution/query break, the gate fails on real data, not
//! a toy fixture. Tests run with CWD = crate root, so `spec/` is the
//! path.

use std::path::Path;

use quire_rs::corpus::resolve::Resolution;
use quire_rs::Spec;

fn dogfood() -> Spec {
    Spec::from_path(Path::new("spec"))
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
