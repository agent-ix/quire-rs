use quire_rs::{extract_filament_core, CoreExtractionResult, FilamentExtractionInput};
use serde::Deserialize;
use serde_json::Value;

const FIXTURES: &str = include_str!("fixtures/filament_core/graph_cases.json");

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Fixture {
    name: String,
    input: FilamentExtractionInput,
    expect: Expected,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Expected {
    #[serde(default)]
    nodes: Vec<ExpectedNode>,
    #[serde(default)]
    edges: Vec<ExpectedEdge>,
    #[serde(default)]
    diagnostic_codes: Vec<String>,
    #[serde(default)]
    errors_contain: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExpectedNode {
    object_type: Option<String>,
    name: Option<String>,
    #[serde(rename = "ref")]
    ref_: Option<String>,
    data: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExpectedEdge {
    source_ref: Option<String>,
    target_ref: Option<String>,
    edge_type: Option<String>,
    data: Option<Value>,
}

// TC-703 (FR-045-AC-3): shared corpus isolation — a failing extraction fixture
// (e.g. `frontmatter-unparsable-error`) does not poison later fixtures, since
// each runs through an independent `extract_filament_core` call.
#[test]
fn filament_core_graph_fixture_corpus() {
    for fixture in fixtures() {
        let result = extract_filament_core(fixture.input.clone());
        assert_expected(&fixture.name, &result, &fixture.expect);
    }
}

// TC-704 (FR-045-AC-5, NFR-020-AC-2): shared corpus determinism.
#[test]
fn filament_core_graph_fixture_corpus_is_deterministic() {
    for fixture in fixtures() {
        let first = extract_filament_core(fixture.input.clone());
        let second = extract_filament_core(fixture.input);
        assert_eq!(
            serde_json::to_value(&first).unwrap(),
            serde_json::to_value(&second).unwrap(),
            "{} did not produce deterministic output",
            fixture.name
        );
    }
}

fn fixtures() -> Vec<Fixture> {
    serde_json::from_str(FIXTURES).expect("filament core fixtures parse")
}

fn assert_expected(name: &str, result: &CoreExtractionResult, expected: &Expected) {
    assert_eq!(
        result.nodes.len(),
        expected.nodes.len(),
        "{name}: node count mismatch\n{result:#?}"
    );
    assert_eq!(
        result.edges.len(),
        expected.edges.len(),
        "{name}: edge count mismatch\n{result:#?}"
    );

    for (idx, expected_node) in expected.nodes.iter().enumerate() {
        let actual = &result.nodes[idx];
        if let Some(object_type) = &expected_node.object_type {
            assert_eq!(
                &actual.object_type, object_type,
                "{name}: node[{idx}] objectType"
            );
        }
        if let Some(node_name) = &expected_node.name {
            assert_eq!(&actual.name, node_name, "{name}: node[{idx}] name");
        }
        if let Some(ref_) = &expected_node.ref_ {
            assert_eq!(&actual.ref_, ref_, "{name}: node[{idx}] ref");
        }
        if let Some(data) = &expected_node.data {
            let actual_data: Value =
                serde_json::from_str(&actual.data_json).expect("node dataJson parses");
            assert_json_subset(name, &format!("node[{idx}].data"), data, &actual_data);
        }
    }

    for (idx, expected_edge) in expected.edges.iter().enumerate() {
        let actual = &result.edges[idx];
        if let Some(source_ref) = &expected_edge.source_ref {
            assert_eq!(
                &actual.source_ref, source_ref,
                "{name}: edge[{idx}] sourceRef"
            );
        }
        if let Some(target_ref) = &expected_edge.target_ref {
            assert_eq!(
                &actual.target_ref, target_ref,
                "{name}: edge[{idx}] targetRef"
            );
        }
        if let Some(edge_type) = &expected_edge.edge_type {
            assert_eq!(&actual.edge_type, edge_type, "{name}: edge[{idx}] edgeType");
        }
        if let Some(data) = &expected_edge.data {
            let actual_data: Value =
                serde_json::from_str(&actual.data_json).expect("edge dataJson parses");
            assert_json_subset(name, &format!("edge[{idx}].data"), data, &actual_data);
        }
    }

    let diagnostic_codes: Vec<&str> = result
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect();
    for code in &expected.diagnostic_codes {
        assert!(
            diagnostic_codes.contains(&code.as_str()),
            "{name}: missing diagnostic code {code:?}; got {diagnostic_codes:?}"
        );
    }

    for needle in &expected.errors_contain {
        assert!(
            result.errors.iter().any(|error| error.contains(needle)),
            "{name}: missing error containing {needle:?}; got {:?}",
            result.errors
        );
    }

    if expected.errors_contain.is_empty() {
        assert_eq!(
            result.errors,
            Vec::<String>::new(),
            "{name}: unexpected errors"
        );
    }
}

fn assert_json_subset(name: &str, path: &str, expected: &Value, actual: &Value) {
    match expected {
        Value::Object(map) => {
            let actual_map = actual
                .as_object()
                .unwrap_or_else(|| panic!("{name}: {path} expected object, got {actual:?}"));
            for (key, expected_value) in map {
                let actual_value = actual_map.get(key).unwrap_or_else(|| {
                    panic!("{name}: {path} missing expected key {key:?}; got {actual:?}")
                });
                assert_json_subset(name, &format!("{path}.{key}"), expected_value, actual_value);
            }
        }
        Value::Array(values) => {
            let actual_values = actual
                .as_array()
                .unwrap_or_else(|| panic!("{name}: {path} expected array, got {actual:?}"));
            assert_eq!(
                actual_values.len(),
                values.len(),
                "{name}: {path} array length mismatch"
            );
            for (idx, expected_value) in values.iter().enumerate() {
                assert_json_subset(
                    name,
                    &format!("{path}[{idx}]"),
                    expected_value,
                    &actual_values[idx],
                );
            }
        }
        _ => assert_eq!(actual, expected, "{name}: {path} mismatch"),
    }
}
