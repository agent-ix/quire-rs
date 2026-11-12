//! FR-062 — the requirement→production-code relation (TC-936..TC-938).
//!
//! The point of these tests is the SEPARATION, not the extraction. CR-061
//! stopped `verifies` binding production symbols because a doc comment citing
//! `FR-053-AC-1` would otherwise count as evidence backing it — letting
//! unverified code claim coverage. `implements` answers a different question
//! and must not reopen that door.

use ix_trace_rs::trace;
use quire_rs::symbols::trace;
use quire_rs::traceability::{SourceLanguage, TraceMarkerForm, TraceabilityModel};

fn model() -> TraceabilityModel {
    let mut model = TraceabilityModel::default();
    model.trace_tags.markers.push(TraceMarkerForm {
        syntax: None,
        name: "rust-trace-attr".to_string(),
        language: SourceLanguage::Rust,
        pattern: r#"#\[trace\("([^"]+)"\)\]"#.to_string(),
        template: None,
        entry_paths: None,
    });
    model.trace_tags.implements.push(TraceMarkerForm {
        syntax: None,
        name: "rust-implements-attr".to_string(),
        language: SourceLanguage::Rust,
        pattern: r#"#\[implements\("([^"]+)"\)\]"#.to_string(),
        template: None,
        entry_paths: None,
    });
    model
}

#[trace("FR-062-AC-1")]
// a production function carries the requirement it
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

#[trace("FR-062-AC-2")]
// the two markers cannot be confused, because the symbol
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

#[trace("FR-062-AC-3")]
// one requirement named by several markers yields one
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

#[trace("FR-062-AC-4")]
// the relation reaches the JSON contract, and changes no
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
        syntax: None,
        name: "rust-implements-attr".to_string(),
        language: SourceLanguage::Rust,
        pattern: r#"#\[implements\("([^"]+)"\)\]"#.to_string(),
        template: None,
        entry_paths: None,
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

#[trace("FR-062-AC-5")]
// the forms reach `bind` when they come from a (CR-081)
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

const GUARDED_PATTERN: &str = r#"(?s)#\[\s*implements\s*\((.*?)\)\s*\]"#;

fn guarded_registry(
    dir: &std::path::Path,
    pattern: &str,
    syntax: Option<&str>,
    language: &str,
    section: &str,
) -> Result<quire_rs::Registry, String> {
    let module = dir.join("module");
    std::fs::create_dir_all(&module).expect("module directory");
    let mut form = serde_json::json!({
        "name": "declared-attribute", "language": language, "pattern": pattern,
        "template": "#[implements({ids})]"
    });
    if let Some(syntax) = syntax {
        form["syntax"] = syntax.into();
    }
    let manifest = serde_json::json!({
        "name": "guarded-fixture", "version": "0.1.0", "artifact_types": [],
        "traceability": { "trace_tags": { section: [form] } }
    });
    std::fs::write(
        module.join("manifest.yaml"),
        serde_json::to_string(&manifest).unwrap(),
    )
    .unwrap();
    let registry = quire_rs::Registry::load_module(&module).map_err(|error| error.to_string())?;
    if let Some(failure) = registry.failures().first() {
        return Err(failure.reason.clone());
    }
    Ok(registry)
}

fn syntax_cases() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../fixtures/corpus_cases/rust-implements-syntax.json"
    ))
    .unwrap()
}

#[trace(
    "FR-062-AC-12",
    "FR-062-AC-13",
    "FR-062-AC-14",
    "FR-062-AC-15"
)]
#[test]
fn declared_attribute_syntax_excludes_data_and_preserves_exact_owners() {
    for case in syntax_cases()["cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        if name == "partial" || name == "range" {
            continue;
        }
        let dir = tempfile::tempdir().unwrap();
        let registry = guarded_registry(
            dir.path(),
            GUARDED_PATTERN,
            Some("rust_attribute"),
            "rust",
            "implements",
        )
        .unwrap();
        let model = registry.traceability().unwrap();
        let serialized = serde_json::to_value(model).unwrap();
        assert_eq!(
            serialized["trace_tags"]["implements"][0]["syntax"],
            "rust_attribute"
        );
        let roundtrip: TraceabilityModel = serde_json::from_value(serialized).unwrap();
        assert_eq!(&roundtrip, model);
        let src = dir.path().join("src");
        std::fs::create_dir(&src).unwrap();
        std::fs::write(src.join("lib.rs"), case["source"].as_str().unwrap()).unwrap();
        let extraction = quire_rs::symbols::extract_tree(&src);
        assert!(
            extraction.diagnostics.is_empty(),
            "{name}: {:?}",
            extraction.diagnostics
        );
        let graph = trace::bind(&extraction, model);
        let actual: std::collections::BTreeSet<_> = graph
            .implements
            .iter()
            .map(|r| (r.symbol.as_str(), r.trace_id.as_str()))
            .collect();
        let expected: std::collections::BTreeSet<_> = case["expected"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| (row[0].as_str().unwrap(), row[1].as_str().unwrap()))
            .collect();
        assert_eq!(actual, expected, "{name}");
        assert!(graph.verifies.is_empty(), "{name}");
        assert!(graph.backed_trace_ids().is_empty(), "{name}");
        assert!(
            graph.partial_attribute_matches.is_empty()
                || name == "real-and-data"
                || name == "ownership",
            "{name}: {:?}",
            graph.partial_attribute_matches
        );
    }
}

#[trace("FR-062-AC-12")]
#[test]
fn invalid_attribute_guards_refuse_the_named_module_form() {
    for (syntax, language, section) in [
        ("unknown", "rust", "implements"),
        ("rust_attribute", "python", "implements"),
        ("rust_attribute", "typescript", "implements"),
        ("rust_attribute", "rust", "markers"),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let error = guarded_registry(dir.path(), GUARDED_PATTERN, Some(syntax), language, section)
            .unwrap_err();
        assert!(
            error.contains("declared-attribute"),
            "{syntax}/{language}/{section}: {error}"
        );
    }
}

#[trace("FR-062-AC-16")]
#[test]
fn omitted_guard_keeps_each_languages_declared_comment_form() {
    for (language, file, source) in [
        ("rust", "lib.rs", "// Implements: FR-601\nfn actual() {}\n"),
        (
            "python",
            "lib.py",
            "# Implements: FR-601\ndef actual():\n    pass\n",
        ),
        (
            "typescript",
            "lib.ts",
            "// Implements: FR-601\nfunction actual() {}\n",
        ),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let registry = guarded_registry(
            dir.path(),
            r"Implements: (FR-\d+)",
            None,
            language,
            "implements",
        )
        .unwrap();
        let src = dir.path().join("src");
        std::fs::create_dir(&src).unwrap();
        std::fs::write(src.join(file), source).unwrap();
        let model = registry.traceability().unwrap();
        assert!(model.trace_tags.implements[0].syntax.is_none());
        let graph = trace::bind(&quire_rs::symbols::extract_tree(&src), model);
        let owners: std::collections::BTreeSet<_> = graph
            .implements
            .iter()
            .map(|r| (r.symbol.as_str(), r.trace_id.as_str()))
            .collect();
        let mut expected = std::collections::BTreeSet::from([("actual", "FR-601")]);
        if language != "rust" {
            expected.insert(("lib", "FR-601"));
        }
        assert_eq!(owners, expected, "{language}");
        assert!(graph.verifies.is_empty());
    }
}

#[trace("FR-062-AC-12")]
#[test]
fn partial_attribute_matches_reach_the_coverage_warning_channel() {
    let dir = tempfile::tempdir().unwrap();
    let registry = guarded_registry(
        dir.path(),
        GUARDED_PATTERN,
        Some("rust_attribute"),
        "rust",
        "implements",
    )
    .unwrap();
    let case = syntax_cases()["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["name"] == "partial")
        .unwrap()
        .clone();
    let src = dir.path().join("src");
    std::fs::create_dir(&src).unwrap();
    std::fs::write(src.join("lib.rs"), case["source"].as_str().unwrap()).unwrap();
    let graph = trace::bind(
        &quire_rs::symbols::extract_tree(&src),
        registry.traceability().unwrap(),
    );
    assert_eq!(graph.implements.len(), 1);
    assert_eq!(graph.implements[0].trace_id, "FR-502");
    assert_eq!(graph.partial_attribute_matches.len(), 1);
    let spec = quire_rs::Spec::from_path(&src);
    let report = quire_rs::coverage::compute(&spec, &registry, &graph, &src).unwrap();
    let findings: Vec<_> = report
        .diagnostics
        .iter()
        .filter(|d| d.reason == "partial-implements-attribute-match")
        .collect();
    assert_eq!(findings.len(), 1);
    let finding = findings[0];
    assert_eq!(finding.declaration, "declared-attribute");
    assert_eq!(finding.path.as_deref(), Some("lib.rs"));
    assert_eq!(finding.line, Some(1));
    assert_eq!(
        finding.value.as_deref(),
        Some("#[doc = r#\"#[implements(\"FR-501\")]\"#]")
    );
    assert!(finding.message.contains("partial"));
    assert_eq!(report.implements.len(), 1);
    assert!(report.unbacked_rows.is_empty());
}

#[trace("FR-062-AC-17")]
#[test]
fn syntax_guard_preserves_identity_range_locations_and_file_failure_isolation() {
    let dir = tempfile::tempdir().unwrap();
    let registry = guarded_registry(
        dir.path(),
        GUARDED_PATTERN,
        Some("rust_attribute"),
        "rust",
        "implements",
    )
    .unwrap();
    let src = dir.path().join("src");
    std::fs::create_dir(&src).unwrap();
    let case = syntax_cases()["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["name"] == "range")
        .unwrap()
        .clone();
    std::fs::write(src.join("lib.rs"), case["source"].as_str().unwrap()).unwrap();
    std::fs::write(src.join("bad.rs"), "fn broken( {").unwrap();
    let extraction = quire_rs::symbols::extract_tree(&src);
    assert_eq!(extraction.diagnostics.len(), 1);
    assert_eq!(extraction.diagnostics[0].path, "bad.rs");
    let model = registry.traceability().unwrap();
    let first = trace::bind(&extraction, model);
    let second = trace::bind(&extraction, model);
    assert_eq!(first, second);
    assert_eq!(first.implements.len(), 1);
    assert_eq!(first.range_diagnostics.len(), 1);
    assert_eq!(first.range_diagnostics[0].line, 3);
    assert_eq!(first.range_diagnostics[0].range_text, "FR-401..FR-403");
    assert!(first.verifies.is_empty());
    let before: Vec<_> = extraction.symbols.iter().map(|s| s.id.clone()).collect();
    std::fs::write(
        src.join("lib.rs"),
        format!("\n{}", case["source"].as_str().unwrap()),
    )
    .unwrap();
    let after = quire_rs::symbols::extract_tree(&src);
    assert_eq!(
        before,
        after
            .symbols
            .iter()
            .map(|s| s.id.clone())
            .collect::<Vec<_>>()
    );
    let graph = trace::bind(&after, model);
    assert_eq!(graph.range_diagnostics[0].line, 4);
}

#[trace("FR-062-AC-12", "FR-062-AC-15")]
#[test]
fn guarded_forms_require_complete_matches_and_do_not_expand_cfg_attr() {
    for (source, pattern, expected_ids, expected_partial) in [
        (
            "#[implements(\"FR-701\")]\nfn candidate() {}",
            r#"implements\(([^)]*)\)"#,
            vec![],
            1,
        ),
        (
            "#[implements(\"FR-701\")]\nfn candidate() {}",
            r#"#\[different\(([^)]*)\)\]"#,
            vec![],
            0,
        ),
        ("fn candidate() {}", GUARDED_PATTERN, vec![], 0),
        (
            "#[cfg_attr(feature = \"x\", implements(\"FR-702\"))]\nfn candidate() {}",
            r#"#\[cfg_attr\(feature = "x", implements\(([^)]*)\)\)\]"#,
            vec!["FR-702"],
            0,
        ),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let registry = guarded_registry(
            dir.path(),
            pattern,
            Some("rust_attribute"),
            "rust",
            "implements",
        )
        .unwrap();
        let src = dir.path().join("src");
        std::fs::create_dir(&src).unwrap();
        std::fs::write(src.join("lib.rs"), source).unwrap();
        let extraction = quire_rs::symbols::extract_tree(&src);
        assert!(extraction.diagnostics.is_empty(), "{source}");
        let graph = trace::bind(&extraction, registry.traceability().unwrap());
        assert_eq!(
            graph
                .implements
                .iter()
                .map(|r| r.trace_id.as_str())
                .collect::<Vec<_>>(),
            expected_ids,
            "{source}/{pattern}"
        );
        assert_eq!(
            graph.partial_attribute_matches.len(),
            expected_partial,
            "{source}/{pattern}"
        );
        assert!(graph.verifies.is_empty());
    }
}

#[trace("FR-062-AC-15")]
#[test]
fn recovered_attribute_is_a_real_missing_token_not_a_valid_candidate() {
    let cases = syntax_cases();
    let source = cases["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["name"] == "recovered-attribute")
        .unwrap()["source"]
        .as_str()
        .unwrap();
    let parsed =
        quire_code_extraction::parse_file(quire_code_extraction::Language::Rust, "lib.rs", source)
            .unwrap();
    assert!(
        parsed.diagnostic().is_none(),
        "healthy declaration must remain extractable"
    );
    let mut nodes = vec![parsed.root_node()];
    let mut recovered = Vec::new();
    while let Some(node) = nodes.pop() {
        if node.kind() == "attribute_item" && node.has_error() {
            recovered.push(node.utf8_text(source.as_bytes()).unwrap());
            assert!(node.to_sexp().contains("MISSING"));
        }
        let mut cursor = node.walk();
        nodes.extend(node.children(&mut cursor));
    }
    assert_eq!(recovered, vec!["#[implements(\"FR-801\")"]);
    // The attributed corpus case above independently asserts exact healthy
    // ownership and no edge for this adapter-recovered candidate.
}

fn coverage_registry_with_evidence(
    dir: &std::path::Path,
    syntax: Option<&str>,
) -> quire_rs::Registry {
    guarded_registry(dir, GUARDED_PATTERN, syntax, "rust", "implements").unwrap();
    let path = dir.join("module/manifest.yaml");
    let mut manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    manifest["artifact_types"] = serde_json::json!([{"name":"FR"}]);
    manifest["traceability"]["trace_targets"] = serde_json::json!([{"name":"criterion","archetype":"FR","section":"Acceptance Criteria","id_column":"ID"}]);
    manifest["traceability"]["trace_tags"]["markers"] = serde_json::json!([{
        "name":"evidence", "language":"rust", "pattern":r#"#\[trace\(([^)]*)\)\]"#
    }]);
    std::fs::write(&path, serde_json::to_string(&manifest).unwrap()).unwrap();
    let registry = quire_rs::Registry::load_module(&dir.join("module")).unwrap();
    assert!(registry.failures().is_empty(), "{:?}", registry.failures());
    registry
}

#[trace("FR-062-AC-17")]
#[test]
fn guarded_scope_changes_preserve_real_verification_backing_and_symbol_identity() {
    let dir = tempfile::tempdir().unwrap();
    let registry = coverage_registry_with_evidence(dir.path(), Some("rust_attribute"));
    let scope = dir.path().join("scope");
    let src = scope.join("src");
    let docs = scope.join("spec");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::create_dir(&docs).unwrap();
    std::fs::write(docs.join("FR-901.md"), "---\nid: FR-901\ntype: FR\n---\n\n## Acceptance Criteria\n\n| ID | Criteria |\n|---|---|\n| FR-901-AC-1 | Preserved backed control. |\n").unwrap();
    let evidence =
        "#[trace(\"FR-901-AC-1\")]\n#[test]\nfn evidence() {}\n#[trace(\"FR-999-AC-1\")]\n#[test]\nfn untracked() {}\n";
    std::fs::write(
        src.join("lib.rs"),
        format!("{evidence}fn production() {{}}\n"),
    )
    .unwrap();
    let before_extraction = quire_rs::symbols::extract_tree(&src);
    let before = trace::bind(&before_extraction, registry.traceability().unwrap());
    assert_eq!(before.verifies.len(), 2);
    assert!(before.implements.is_empty());
    let spec = quire_rs::Spec::from_path(&scope);
    let before_report = quire_rs::coverage::compute(&spec, &registry, &before, &scope).unwrap();
    assert_eq!(before_report.untracked_symbols.len(), 1);
    assert_eq!(before_report.untracked_symbols[0].symbol, "untracked");
    assert_eq!(before_report.totals.backed, 1);
    assert_eq!(before_report.totals.total, 1);
    assert_eq!(
        before.backed_trace_ids(),
        std::collections::BTreeSet::from(["FR-901-AC-1", "FR-999-AC-1"])
    );
    let after_source = format!("{evidence}// unrelated source\n#[implements(\n \"FR-902\", \"FR-903\"\n)] // trailing comment\nfn production() {{}}\n#[doc = r#\"#[implements(\"FR-904\")]\"#]\nfn partial() {{}}\n");
    std::fs::write(src.join("lib.rs"), &after_source).unwrap();
    let after_extraction = quire_rs::symbols::extract_tree(&src);
    let production = after_extraction
        .symbols
        .iter()
        .find(|symbol| symbol.qualified_name == "production")
        .unwrap();
    assert_eq!(production.attributes.len(), 1);
    let attribute = &production.attributes[0];
    assert_eq!(attribute.line, 8);
    assert_eq!(attribute.text, "#[implements(\n \"FR-902\", \"FR-903\"\n)]");
    let candidate_byte = after_source.find(&attribute.text).unwrap();
    assert_eq!(
        candidate_byte,
        format!("{evidence}// unrelated source\n").len()
    );
    assert_eq!(
        after_source[..candidate_byte]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count()
            + 1,
        attribute.line
    );
    let after = trace::bind(&after_extraction, registry.traceability().unwrap());
    let repeated = trace::bind(&after_extraction, registry.traceability().unwrap());
    assert_eq!(
        after, repeated,
        "complete ordered graph includes partial and range diagnostics"
    );
    assert_eq!(after.verifies, before.verifies);
    assert_eq!(after.backed_trace_ids(), before.backed_trace_ids());
    let expected = vec![("production", "FR-902"), ("production", "FR-903")];
    assert_eq!(
        after
            .implements
            .iter()
            .map(|r| (r.symbol.as_str(), r.trace_id.as_str()))
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(after.partial_attribute_matches.len(), 1);
    assert_eq!(after.partial_attribute_matches[0].line, 12);
    let after_report = quire_rs::coverage::compute(&spec, &registry, &after, &scope).unwrap();
    assert_eq!(after_report.totals, before_report.totals);
    assert_eq!(
        after_report.untracked_symbols,
        before_report.untracked_symbols
    );
    let repeated_report = quire_rs::coverage::compute(&spec, &registry, &repeated, &scope).unwrap();
    assert_eq!(after_report.diagnostics, repeated_report.diagnostics);
    let unguarded = coverage_registry_with_evidence(dir.path(), None);
    let unguarded_extraction = quire_rs::symbols::extract_tree(&src);
    let identity = |extraction: &quire_rs::symbols::SymbolExtraction| {
        extraction
            .symbols
            .iter()
            .map(|s| {
                (
                    s.id.clone(),
                    s.path.clone(),
                    s.kind,
                    s.line,
                    s.column,
                    s.leading_line,
                    s.end_line,
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(identity(&after_extraction), identity(&unguarded_extraction));
    // For a declaration with only eligible real attributes, the old and
    // guarded models retain exactly the same relation endpoint identities.
    let unguarded_graph = trace::bind(&unguarded_extraction, unguarded.traceability().unwrap());
    let old_production: Vec<_> = unguarded_graph
        .implements
        .iter()
        .filter(|r| r.symbol == "production")
        .collect();
    assert_eq!(old_production, after.implements.iter().collect::<Vec<_>>());
    assert_eq!(unguarded_graph.verifies, after.verifies);
}

#[trace("FR-062-AC-12")]
#[test]
fn present_invalid_syntax_values_never_become_an_unguarded_form() {
    for value in [
        serde_json::Value::Null,
        serde_json::json!(false),
        serde_json::json!(4),
        serde_json::json!({}),
    ] {
        let dir = tempfile::tempdir().unwrap();
        guarded_registry(dir.path(), GUARDED_PATTERN, None, "rust", "implements").unwrap();
        let path = dir.path().join("module/manifest.yaml");
        let mut manifest: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        manifest["traceability"]["trace_tags"]["implements"][0]["syntax"] = value.clone();
        std::fs::write(&path, serde_json::to_string(&manifest).unwrap()).unwrap();
        let error = quire_rs::Registry::load_module(&dir.path().join("module"))
            .expect_err("present syntax must not silently become unguarded");
        assert!(error.to_string().contains("declared-attribute"), "{error}");
    }
}

#[trace("FR-051-AC-31", "FR-051-AC-32")]
#[test]
fn trait_impl_methods_get_trait_qualified_ids() {
    let cases: serde_json::Value = serde_json::from_str(include_str!(
        "../fixtures/corpus_cases/trait-impl-symbol-ids.json"
    ))
    .unwrap();
    for case in cases["cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let out = quire_rs::symbols::extract_file(
            "src/a.rs",
            SourceLanguage::Rust,
            case["source"].as_str().unwrap(),
        );
        let mut names: Vec<&str> = out
            .symbols
            .iter()
            .filter(|s| s.kind != quire_rs::symbols::SymbolKind::Container)
            .map(|s| s.qualified_name.as_str())
            .collect();
        names.sort_unstable();
        let mut expected: Vec<&str> = case["expected_names"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| n.as_str().unwrap())
            .collect();
        expected.sort_unstable();
        assert_eq!(names, expected, "{name}");
        let reasons: Vec<&str> = out.diagnostics.iter().map(|d| d.reason.as_str()).collect();
        for want in case["diagnostic_reasons"].as_array().into_iter().flatten() {
            let want = want.as_str().unwrap();
            assert!(
                reasons.iter().any(|r| r.starts_with(want)),
                "{name}: {reasons:?}"
            );
        }
        for absent in case["absent_diagnostic_reasons"]
            .as_array()
            .into_iter()
            .flatten()
        {
            let absent = absent.as_str().unwrap();
            assert!(
                !reasons.iter().any(|r| r.starts_with(absent)),
                "{name}: {reasons:?}"
            );
        }
    }
}

#[trace("FR-051-AC-32")]
#[test]
fn trait_impl_method_is_contained_by_its_target_type() {
    let src = "pub struct Halt;\nimpl From<Finish> for Halt {\n    fn from(_: Finish) -> Self { Halt }\n}\nmod m {\n    pub struct W;\n    impl Drop for W {\n        fn drop(&mut self) {}\n    }\n}\n";
    let out = quire_rs::symbols::extract_file("src/a.rs", SourceLanguage::Rust, src);
    let graph = trace::bind(&out, &model());
    let edges: Vec<(&str, &str)> = graph
        .contains
        .iter()
        .map(|(parent, child, _)| (parent.as_str(), child.as_str()))
        .collect();
    assert!(
        edges.contains(&("Halt", "<Halt as From<Finish>>::from")),
        "{edges:?}"
    );
    assert!(
        edges.contains(&("m::W", "m::<W as Drop>::drop")),
        "{edges:?}"
    );
}
