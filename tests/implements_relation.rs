//! FR-062 — the requirement→production-code relation (TC-936..TC-938).
//!
//! The point of these tests is the SEPARATION, not the extraction. CR-061
//! stopped `verifies` binding production symbols because a doc comment citing
//! `FR-053-AC-1` would otherwise count as evidence backing it — letting
//! unverified code claim coverage. `implements` answers a different question
//! and must not reopen that door.

use quire_rs::symbols::trace;
use quire_rs::traceability::{SourceLanguage, TraceMarkerForm, TraceabilityModel};

fn model() -> TraceabilityModel {
    let mut model = TraceabilityModel::default();
    model.trace_tags.markers.push(TraceMarkerForm {
        name: "rust-trace-attr".to_string(),
        language: SourceLanguage::Rust,
        pattern: r#"#\[trace\("([^"]+)"\)\]"#.to_string(),
        template: None,
    });
    model.trace_tags.implements.push(TraceMarkerForm {
        name: "rust-implements-attr".to_string(),
        language: SourceLanguage::Rust,
        pattern: r#"#\[implements\("([^"]+)"\)\]"#.to_string(),
        template: None,
    });
    model
}

// TC-936 (FR-062-AC-1): a production function carries the requirement it
// implements, and that relation is NOT evidence.
#[test]
fn tc936_a_production_symbol_implements_without_backing() {
    let dir = std::env::temp_dir().join(format!("quire-impl-936-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("mkdir");
    std::fs::write(
        dir.join("lib.rs"),
        "#[implements(\"FR-053\")]\npub fn parse_manifest(text: &str) -> usize { text.len() }\n",
    )
    .expect("write");

    let graph = trace::bind(&quire_rs::symbols::extract_tree(&dir), &model());

    assert_eq!(graph.implements.len(), 1, "{:?}", graph.implements);
    assert_eq!(graph.implements[0].trace_id, "FR-053");
    assert_eq!(graph.implements[0].form, "rust-implements-attr");

    // THE guarantee. `implements` is scope, not evidence: nothing it names may
    // appear as backed, or unverified code could claim coverage by citing a
    // requirement — the exact backdoor CR-061 closed.
    assert!(
        graph.backed_trace_ids().is_empty(),
        "implements must never back a trace id: {:?}",
        graph.backed_trace_ids()
    );
    assert!(graph.verifies.is_empty());
}

// TC-937 (FR-062-AC-2): the two markers cannot be confused, because the symbol
// kinds they attach to are complements.
#[test]
fn tc937_the_two_relations_do_not_cross() {
    let dir = std::env::temp_dir().join(format!("quire-impl-937-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("mkdir");
    // An `implements` marker on a TEST, and a `trace` marker on PRODUCTION
    // code. Both are mis-authored, and both must bind nothing rather than bind
    // the other relation.
    std::fs::write(
        dir.join("lib.rs"),
        "#[trace(\"FR-001-AC-1\")]\npub fn production(x: usize) -> usize { x }\n\n\
         #[cfg(test)]\nmod tests {\n\
         #[implements(\"FR-001\")]\n    #[test]\n    fn a_test() {}\n}\n",
    )
    .expect("write");

    let graph = trace::bind(&quire_rs::symbols::extract_tree(&dir), &model());

    // The production `#[trace]` does not become evidence — CR-061 unchanged.
    assert!(
        graph.backed_trace_ids().is_empty(),
        "a production trace marker must not back: {:?}",
        graph.backed_trace_ids()
    );
    // And the `implements` on a test binds nothing, because a test is not the
    // code a requirement is about.
    assert!(
        graph
            .implements
            .iter()
            .all(|r| !r.symbol.contains("a_test")),
        "{:?}",
        graph.implements
    );
}

// TC-938 (FR-062-AC-3): one requirement named by several markers yields one
// relation, and the ordering is deterministic (NFR-006).
#[test]
fn tc938_relations_are_deduped_and_ordered() {
    let dir = std::env::temp_dir().join(format!("quire-impl-938-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("mkdir");
    std::fs::write(
        dir.join("lib.rs"),
        "#[implements(\"FR-053, FR-050\")]\n#[implements(\"FR-053\")]\n\
         pub fn one(x: usize) -> usize { x }\n",
    )
    .expect("write");

    let graph = trace::bind(&quire_rs::symbols::extract_tree(&dir), &model());

    let ids: Vec<_> = graph
        .implements
        .iter()
        .map(|r| r.trace_id.as_str())
        .collect();
    // FR-050 before FR-053, and FR-053 once despite two markers naming it.
    assert_eq!(ids, vec!["FR-050", "FR-053"], "{:?}", graph.implements);
}
