//! FR-075 systems-model tables: part, port, connection, allocation
//! (TC-1872, TC-1873).

use std::fs;
use std::path::PathBuf;

use ix_trace_rs::trace;
use jsonschema::JSONSchema;
use quire_rs::semantic::python_entry::extract_semantic_json;
use quire_rs::semantic::SEMANTIC_V1_SCHEMA;
use quire_rs::Registry;
use serde_json::{json, Value};

const PATH: &str = "spec/architecture/part.md";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn module_root() -> PathBuf {
    root().join("tests/fixtures/semantic/spec-objects-architecture")
}

/// The four systems locators as spec-objects-architecture declares them.
fn body_extraction(kind: &str) -> Value {
    let (section, columns): (&str, &[&str]) = match kind {
        "part" => ("Part", &["Owner", "Declared Type", "Multiplicity"]),
        "port" => ("Port", &["Owner", "Direction", "Interface", "Multiplicity"]),
        "connection" => (
            "Connection",
            &[
                "Source",
                "Source Multiplicity",
                "Target",
                "Target Multiplicity",
                "Direction",
            ],
        ),
        "allocation" => ("Allocation", &["Source", "Target"]),
        other => panic!("{other}"),
    };
    json!({ "yield_pattern": { "match": { kind: {
        "from": "table_row",
        "under_section": section,
        "required": true,
        "assert": { "columns": columns, "min_rows": 1 },
    } } } })
}

/// Extract `md` under `body_extraction`, against a bundle holding
/// `artifacts`; the record must validate against semantic-v1.
fn extract(md: &str, body_extraction: Value, artifacts: &[&str]) -> Value {
    let artifacts: Vec<Value> = artifacts.iter().map(|id| json!({ "id": id })).collect();
    let request = json!({
        "markdown": md,
        "module": {
            "contractVersion": "1.0.0",
            "semanticCore": "0.1.0",
            "package": "agent-ix/spec-objects-architecture",
            "exports": ["part", "port", "connection", "allocation"],
        },
        "path": PATH,
        "sourceIdentity": "ix://agent-ix/shop/spec",
        "bundle": {
            "package": "agent-ix/shop",
            "objects": [{ "id": "quant_codec", "names": ["QuantCodec"] }],
            "artifacts": artifacts,
        },
        "bodyExtraction": body_extraction,
    });
    let record = extract_semantic_json(&request).unwrap();
    let value = serde_json::to_value(&record).unwrap();
    let schema = JSONSchema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .compile(&serde_json::from_str::<Value>(SEMANTIC_V1_SCHEMA).unwrap())
        .unwrap();
    let errors: Vec<String> = match schema.validate(&value) {
        Ok(()) => Vec::new(),
        Err(errs) => errs
            .map(|e| format!("{e} at {}", e.instance_path))
            .collect(),
    };
    assert!(errors.is_empty(), "{errors:?}\n{value:#}");
    value
}

fn line_of(md: &str, needle: &str) -> u64 {
    md.split('\n')
        .position(|l| l.contains(needle))
        .unwrap_or_else(|| panic!("{needle:?} not in document")) as u64
        + 1
}

/// Whether the record carries an error `code` at `line`, and no `model`.
fn refused(record: &Value, code: &str, line: u64) -> bool {
    let at =
        record["diagnostics"].as_array().unwrap().iter().any(|d| {
            d["code"] == code && d["severity"] == "error" && d["line"].as_u64() == Some(line)
        });
    at && record.get("model").is_none() && record["availability"]["model"]["state"] == "unavailable"
}

fn doc(id: &str, kind: &str, section: &str, table: &str) -> String {
    format!("---\nid: {id}\ntitle: \"{id}\"\ntype: {kind}\nobject: {kind}\n---\n# [{id}] {id}\n\n## {section}\n\n{table}")
}

const PART: &str = "| Owner | Declared Type | Multiplicity |\n|---|---|---|\n| search_service | QuantCodec | 0..* |\n";
const PORT: &str = "| Owner | Direction | Interface | Multiplicity |\n|---|---|---|---|\n| scoring_engine | inout | QuantCodec | 1..1 |\n";
const CONNECTION: &str = "| Source | Source Multiplicity | Target | Target Multiplicity | Direction |\n|---|---|---|---|---|\n| planner_out | 1..1 | score_in | | bidirectional |\n";
const ALLOCATION: &str =
    "| Source | Target |\n|---|---|\n| quant_codec/score_ip_batch | scoring_engine |\n";

const BUNDLE: &[&str] = &[
    "search_service",
    "scoring_engine",
    "planner_out",
    "score_in",
    "quant_codec",
];

#[trace("TC-1872", "FR-075-AC-12")]
#[test]
fn systems_tables_lower_to_record_keys_and_refuse_bad_rows() {
    let pkg = "ix://agent-ix/shop";

    let md = doc("engine", "part", "Part", PART);
    let record = extract(&md, body_extraction("part"), BUNDLE);
    let part = &record["model"]["part"];
    assert_eq!(part["owner"], format!("{pkg}/search_service"), "{record:#}");
    assert_eq!(
        part["declaredType"],
        json!({ "target": format!("{pkg}/type/QuantCodec") })
    );
    assert_eq!(part["multiplicity"], json!({ "lower": 0 }));
    assert_eq!(
        part["sourceSpan"]["startLine"],
        line_of(&md, "| search_service")
    );

    let md = doc("score_in", "port", "Port", PORT);
    let record = extract(&md, body_extraction("port"), BUNDLE);
    let port = &record["model"]["port"];
    assert_eq!(port["owner"], format!("{pkg}/scoring_engine"), "{record:#}");
    assert_eq!(port["direction"], "inout");
    assert_eq!(
        port["interfaceType"],
        json!({ "target": format!("{pkg}/type/QuantCodec") })
    );
    assert_eq!(port["multiplicity"], json!({ "lower": 1, "upper": 1 }));

    let md = doc("wire", "connection", "Connection", CONNECTION);
    let record = extract(&md, body_extraction("connection"), BUNDLE);
    let connection = &record["model"]["connection"];
    assert_eq!(
        connection["sourceEnd"],
        json!({ "type": format!("{pkg}/planner_out"), "multiplicity": { "lower": 1, "upper": 1 } }),
        "{record:#}"
    );
    // An empty end multiplicity cell states none.
    assert_eq!(
        connection["targetEnd"],
        json!({ "type": format!("{pkg}/score_in") })
    );
    assert_eq!(connection["flowDirection"], "bidirectional");

    let md = doc("alloc", "allocation", "Allocation", ALLOCATION);
    let record = extract(&md, body_extraction("allocation"), BUNDLE);
    let allocation = &record["model"]["allocation"];
    assert_eq!(
        allocation["sourceElement"],
        format!("{pkg}/quant_codec/score_ip_batch"),
        "{record:#}"
    );
    assert_eq!(allocation["targetElement"], format!("{pkg}/scoring_engine"));
    assert_eq!(record["availability"]["model"]["state"], "available");

    // A name no bundle artifact carries.
    let md = doc("engine", "part", "Part", PART);
    let record = extract(&md, body_extraction("part"), &["scoring_engine"]);
    assert!(
        refused(
            &record,
            "semantic.unknown-reference",
            line_of(&md, "| search_service")
        ),
        "{record:#}"
    );
    // A bundle without artifacts lowers the name unchecked, with an advisory.
    let record = extract(&md, body_extraction("part"), &[]);
    assert_eq!(
        record["model"]["part"]["owner"],
        format!("{pkg}/search_service")
    );
    assert!(record["diagnostics"].as_array().unwrap().iter().any(|d| {
        d["code"] == "semantic.unresolved-target"
            && d["severity"] == "advisory"
            && d["reason"] == "no-bundle-index"
    }));

    // An unknown port direction and an unknown connection direction.
    let md = doc(
        "score_in",
        "port",
        "Port",
        &PORT.replace("inout", "sideways"),
    );
    let record = extract(&md, body_extraction("port"), BUNDLE);
    assert!(
        refused(
            &record,
            "semantic.invalid-model-cell",
            line_of(&md, "sideways")
        ),
        "{record:#}"
    );
    let md = doc(
        "wire",
        "connection",
        "Connection",
        &CONNECTION.replace("bidirectional", "undirected"),
    );
    let record = extract(&md, body_extraction("connection"), BUNDLE);
    assert!(
        refused(
            &record,
            "semantic.invalid-model-cell",
            line_of(&md, "undirected")
        ),
        "{record:#}"
    );

    // A second row.
    let two = format!("{ALLOCATION}| scoring_engine | search_service |\n");
    let md = doc("alloc", "allocation", "Allocation", &two);
    let record = extract(&md, body_extraction("allocation"), BUNDLE);
    assert!(
        refused(
            &record,
            "semantic.duplicate-model-entry",
            line_of(&md, "| scoring_engine | search_service")
        ),
        "{record:#}"
    );

    // `<id>/<member>` names an allocation source only.
    let md = doc(
        "engine",
        "part",
        "Part",
        &PART.replace("search_service", "search_service/run"),
    );
    let record = extract(&md, body_extraction("part"), BUNDLE);
    assert!(
        refused(
            &record,
            "semantic.invalid-model-cell",
            line_of(&md, "search_service/run")
        ),
        "{record:#}"
    );

    // Undeclared: a `Source | Target` table is the allocation feature, not
    // the connection feature.
    let md = doc("alloc", "allocation", "Elsewhere", ALLOCATION);
    let record = extract(&md, body_extraction("allocation"), BUNDLE);
    let refusal = record["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["code"] == "semantic.feature-not-extractable")
        .unwrap_or_else(|| panic!("{record:#}"));
    assert_eq!(refusal["reason"], "allocation");
    assert_eq!(refusal["section"], "Elsewhere");
}

#[trace("TC-1873", "FR-075-AC-13")]
#[test]
fn spec_objects_architecture_systems_skeletons_validate_with_zero_errors() {
    let registry = Registry::load_module(&module_root()).unwrap();
    let keys: [(&str, &[&str]); 4] = [
        ("part", &["owner", "declaredType", "multiplicity"]),
        (
            "port",
            &["owner", "direction", "interfaceType", "multiplicity"],
        ),
        ("connection", &["sourceEnd", "targetEnd", "flowDirection"]),
        ("allocation", &["sourceElement", "targetElement"]),
    ];
    for (kind, record_keys) in keys {
        let text = fs::read_to_string(module_root().join(format!("skeletons/{kind}.md"))).unwrap();
        let arch = registry.archetype(kind).unwrap();
        let result = quire_rs::validate_document_in_registry(&registry, arch, &text);
        assert!(result.is_valid, "{kind}: {:?}", result.errors);
        assert!(result.errors.is_empty(), "{kind}: {:?}", result.errors);

        // The declaration record the data schema validated carries the keys.
        let mut ctx = quire_rs::semantic::SemanticContext::new(
            registry
                .semantic_module("spec-objects-architecture")
                .unwrap()
                .clone(),
            format!("skeletons/{kind}.md"),
            quire_rs::semantic::BundleIndex::default(),
        );
        ctx = ctx.with_body_extraction(arch.body_extraction().unwrap());
        let record = quire_rs::semantic::extract_semantic(
            &text,
            &ctx,
            None,
            &quire_rs::semantic::RequiredSections::default(),
        )
        .declaration_record();
        let mut found: Vec<&str> = record
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        found.sort_unstable();
        let mut expected = record_keys.to_vec();
        expected.sort_unstable();
        assert_eq!(found, expected, "{kind}: {record:#}");
    }
}
