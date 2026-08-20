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

// TC-936, FR-062-AC-1: a production function carries the requirement it
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

// TC-937, FR-062-AC-2: the two markers cannot be confused, because the symbol
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

// TC-938, FR-062-AC-3: one requirement named by several markers yields one
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

// TC-939, FR-062-AC-4: the relation reaches the JSON contract, and changes no
// coverage number.
//
// FR-061 shipped a combinatorial branch that existed only on the single-document
// path, so `quire coverage` — the surface every consumer reads — never carried
// it (CR-076). This asserts the equivalent for `implements` rather than assuming
// that minting a relation makes it reachable: a relation in `SymbolGraph` that
// no consumer can see is a capability nothing reaches.
#[test]
fn tc939_implements_reaches_the_report_without_moving_a_total() {
    use quire_rs::Registry;

    let dir = std::env::temp_dir().join(format!("quire-impl-939-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let scope = dir.join("spec");
    std::fs::create_dir_all(&scope).expect("mkdir");
    std::fs::write(
        scope.join("FR-001.md"),
        "---\nid: FR-001\ntype: FR\ntitle: A requirement\n---\n\n\
         ## Acceptance Criteria\n\n\
         | ID | Criteria | Verification |\n|----|----------|--------------|\n\
         | FR-001-AC-1 | The system shall do it. | Test |\n",
    )
    .expect("write");
    let src = dir.join("src");
    std::fs::create_dir_all(&src).expect("mkdir");
    std::fs::write(
        src.join("lib.rs"),
        "#[implements(\"FR-001\")]\npub fn does_it(x: usize) -> usize { x }\n",
    )
    .expect("write");

    let module = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("traceability")
        .join("obligations-catalog");
    let registry = Registry::load_module(&module).expect("load module");
    let spec = quire_rs::Spec::from_path(&scope);

    let mut model = registry.traceability().cloned().expect("model");
    model.trace_tags.implements.push(TraceMarkerForm {
        name: "rust-implements-attr".to_string(),
        language: SourceLanguage::Rust,
        pattern: r#"#\[implements\("([^"]+)"\)\]"#.to_string(),
        template: None,
    });
    let graph = trace::bind(&quire_rs::symbols::extract_tree(&src), &model);
    let report = quire_rs::coverage::compute(&spec, &registry, &graph, &scope).expect("report");

    // It reaches the report a consumer reads.
    assert_eq!(report.implements.len(), 1, "{:?}", report.implements);
    assert_eq!(report.implements[0].trace_id, "FR-001");
    assert_eq!(report.implements[0].form, "rust-implements-attr");

    // And moves no coverage number: the criterion is still unbacked, because
    // production code citing a requirement is scope, not evidence.
    assert_eq!(report.totals.backed, 0, "{:?}", report.totals);
    assert!(
        report.untracked_symbols.is_empty(),
        "an implements edge is not an untracked trace tag: {:?}",
        report.untracked_symbols
    );

    // The serialized payload carries it too — a field the struct has and the
    // JSON drops is the same defect one layer down.
    let json = serde_json::to_value(&report).expect("serialize");
    assert!(
        json.get("implements").is_some(),
        "the JSON contract must carry it: {json}"
    );

    std::fs::remove_dir_all(&dir).ok();
}

// TC-940, FR-062-AC-5 (CR-081): the forms reach `bind` when they come from a
// module manifest, which is the only way a consumer ever supplies them.
//
// Every test above builds the model in memory, and that is exactly why they all
// passed while the relation was dead in the field. `merge_traceability` and
// `TraceabilityModel::is_empty` are both hand-maintained per-field functions,
// and neither listed `trace_tags.implements` — so a module declaring the forms
// had them dropped between the manifest and the graph, and `quire coverage`
// reported an empty `implements` array for a repository whose production code
// was correctly annotated.
//
// The distinction this test draws is load path vs. struct: `Registry::load_module`
// on the left, `TraceabilityModel::default()` on the right. Only the left one
// is what a consumer runs.
#[test]
fn tc940_declared_forms_survive_the_module_load() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/traceability/required-relations");
    let registry = quire_rs::registry::Registry::load_module(&fixture).expect("load");
    let model = registry
        .traceability()
        .expect("a model declaring implements forms is a declared model");

    let dir = std::env::temp_dir().join(format!("quire-impl-940-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("mkdir");
    std::fs::write(
        dir.join("lib.rs"),
        "#[implements(\"FR-053\")]\npub fn parse_manifest(text: &str) -> usize { text.len() }\n",
    )
    .expect("write");

    let graph = trace::bind(&quire_rs::symbols::extract_tree(&dir), model);

    assert_eq!(
        graph.implements.len(),
        1,
        "the manifest's forms never reached bind: {:?}",
        graph.implements
    );
    assert_eq!(graph.implements[0].trace_id, "FR-053");
    assert_eq!(graph.implements[0].form, "rust-implements-attr");
    assert!(graph.backed_trace_ids().is_empty());

    std::fs::remove_dir_all(&dir).ok();
}
