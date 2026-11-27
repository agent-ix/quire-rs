//! Focused source-symbol extraction contracts (FR-051).

use ix_trace_rs::trace;

fn fixture_module(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/traceability")
        .join(name)
}

fn trace_model() -> quire_rs::traceability::TraceabilityModel {
    quire_rs::Registry::load_module(&fixture_module("iso"))
        .expect("load traceability fixture")
        .traceability()
        .cloned()
        .expect("traceability model")
}

fn write(root: &std::path::Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().expect("parent directory")).expect("create parent");
    std::fs::write(path, contents).expect("write fixture");
}

#[cfg(feature = "rust-symbols")]
#[trace("FR-051-AC-16")]
#[test]
fn rust_marker_with_spaced_ids_and_trailing_comma_binds_each_id_once() {
    use quire_rs::symbols::{extract_file, trace::bind};
    use quire_rs::traceability::SourceLanguage;

    let source = concat!(
        "#[trace( \"FR-001-AC-1\" ,  \"FR-001-AC-2\", )]\n",
        "#[test]\n",
        "fn covers_both() {}\n"
    );
    let extraction = extract_file("tests.rs", SourceLanguage::Rust, source);
    let graph = bind(&extraction, &trace_model());
    let mut ids: Vec<&str> = graph
        .verifies
        .iter()
        .map(|relation| relation.trace_id.as_str())
        .collect();
    ids.sort_unstable();

    assert_eq!(ids, ["FR-001-AC-1", "FR-001-AC-2"]);
    assert!(graph
        .verifies
        .iter()
        .all(|relation| !relation.trace_id.is_empty()));
}

#[cfg(feature = "rust-symbols")]
#[test]
fn uncompiled_tree_with_missing_dependency_still_extracts_and_binds() {
    use quire_rs::symbols::{extract_tree, trace::bind, SymbolKind};

    let root = tempfile::tempdir().expect("temporary fixture tree");
    write(
        root.path(),
        "Cargo.toml",
        "[package]\nname = \"uncompiled-fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nnot-installed-fixture-dependency = { path = \"../missing-dependency\" }\n",
    );
    write(
        root.path(),
        "src/lib.rs",
        "use not_installed_fixture_dependency::MissingType;\n\n#[trace(\"FR-001-AC-1\")]\n#[test]\nfn uses_an_unresolved_type() { let _: MissingType = unimplemented!(); }\n",
    );

    let missing_dependency = root.path().join("../missing-dependency");
    assert!(
        !missing_dependency.exists(),
        "the fixture dependency is absent"
    );
    assert!(!root.path().join("target").exists());
    assert!(!root.path().join("Cargo.lock").exists());

    let extraction = extract_tree(root.path());
    let graph = bind(&extraction, &trace_model());
    assert!(
        extraction.diagnostics.is_empty(),
        "{:?}",
        extraction.diagnostics
    );
    assert!(extraction.symbols.iter().any(|symbol| {
        symbol.kind == SymbolKind::TestFunction
            && symbol.qualified_name == "uses_an_unresolved_type"
    }));
    assert_eq!(graph.verifies.len(), 1);
    assert_eq!(graph.verifies[0].trace_id, "FR-001-AC-1");
    assert!(
        !root.path().join("target").exists(),
        "extraction made no build artifacts"
    );
    assert!(
        !root.path().join("Cargo.lock").exists(),
        "extraction invoked no package manager"
    );
}

#[cfg(all(feature = "python-symbols", feature = "typescript-symbols"))]
#[trace("FR-051-AC-47")]
#[test]
fn coverage_json_uses_python_and_typescript_test_names_without_suite_prefixes() {
    use quire_rs::coverage::compute;
    use quire_rs::symbols::{extract_tree, trace::bind};
    use quire_rs::{Registry, Spec};

    let root = tempfile::tempdir().expect("temporary fixture tree");
    let scope = root.path().join("spec");
    let source = root.path().join("src");
    write(
        &scope,
        "FR-001.md",
        "---\nid: FR-001\ntype: FR\ntitle: Requirement\n---\n\n## Acceptance Criteria\n\n| ID | Criteria | Verification |\n|----|----------|--------------|\n| FR-001-AC-1 | The system shall return the result. | Test (TC-001) |\n",
    );
    write(
        &scope,
        "tests.md",
        "---\nid: TM-001\ntype: TestMatrix\ntitle: Test Matrix\n---\n\n## Test Cases\n\n| ID | Traces To | Status |\n|----|-----------|--------|\n| TC-001 | FR-001-AC-1 | ✅ |\n",
    );
    write(
        &source,
        "tests/test_x.py",
        "import pytest\n\n@pytest.mark.trace(\"FR-001-AC-1\")\ndef test_top(): pass\n\nclass TestC:\n    @pytest.mark.trace(\"FR-001-AC-1\")\n    def test_m(self): pass\n\nclass TestOuter:\n    class TestInner:\n        @pytest.mark.trace(\"FR-001-AC-1\")\n        def test_n(self): pass\n\n@pytest.mark.parametrize(\"value\", [1, 2])\n@pytest.mark.trace(\"FR-001-AC-1\")\ndef test_param(value): pass\n",
    );
    write(
        &source,
        "tests/x.test.ts",
        "describe('outer', () => {\n  describe('inner', () => {\n    // trace(\"FR-001-AC-1\")\n    it('adds', () => {});\n    // trace(\"FR-001-AC-1\")\n    it.each([1, 2])('adds %s', (value: number) => {});\n  });\n});\n",
    );

    let registry = Registry::load_module(&fixture_module("iso-obligations"))
        .expect("load obligations fixture");
    let spec = Spec::from_path(&scope);
    let extraction = extract_tree(&source);
    let graph = bind(&extraction, registry.traceability().expect("model"));
    let report = compute(&spec, &registry, &graph, &scope).expect("coverage report");
    let json: serde_json::Value = serde_json::from_str(&report.to_json()).expect("coverage JSON");
    let criterion = json["coverage_matrix"]
        .as_array()
        .expect("coverage matrix")
        .iter()
        .flat_map(|requirement| requirement["criteria"].as_array().into_iter().flatten())
        .find(|criterion| criterion["id"] == "FR-001-AC-1")
        .expect("the fixture criterion is in the coverage report");
    let mut pairs: Vec<(String, String)> = criterion["binders"]
        .as_array()
        .expect("criterion binders")
        .iter()
        .map(|binder| {
            (
                binder["path"].as_str().expect("binder path").to_string(),
                binder["qualified_name"]
                    .as_str()
                    .expect("qualified name")
                    .to_string(),
            )
        })
        .collect();
    pairs.sort();

    assert_eq!(
        pairs,
        [
            ("tests/test_x.py".to_string(), "TestC.test_m".to_string()),
            (
                "tests/test_x.py".to_string(),
                "TestOuter.TestInner.test_n".to_string()
            ),
            ("tests/test_x.py".to_string(), "test_param".to_string()),
            ("tests/test_x.py".to_string(), "test_top".to_string()),
            ("tests/x.test.ts".to_string(), "adds".to_string()),
            ("tests/x.test.ts".to_string(), "adds %s".to_string()),
        ]
    );
}
