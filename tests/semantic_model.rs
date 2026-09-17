//! FR-075 model feature extraction (TC-1840..TC-1846).

use std::fs;
use std::path::PathBuf;

use ix_trace_rs::trace;
use jsonschema::JSONSchema;
use quire_rs::semantic::python_entry::extract_semantic_json;
use quire_rs::semantic::SEMANTIC_V1_SCHEMA;
use serde_json::{json, Value};

const PATH: &str = "spec/functional/FR-100.md";

fn schema() -> JSONSchema {
    JSONSchema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .compile(&serde_json::from_str::<Value>(SEMANTIC_V1_SCHEMA).unwrap())
        .unwrap()
}

/// Extract `md` under `mappings` and `body_extraction`; every record must
/// validate against semantic-v1 (FR-075-AC-7).
fn extract(md: &str, mappings: &[&str], body_extraction: Value) -> Value {
    let request = json!({
        "markdown": md,
        "module": {
            "contractVersion": "1.0.0",
            "semanticCore": "0.1.0",
            "package": "agent-ix/spec-objects-fixture",
            "exports": ["entity"],
            "mappings": mappings,
        },
        "path": PATH,
        "sourceIdentity": "ix://agent-ix/shop/spec",
        "bundle": {
            "package": "agent-ix/shop",
            "objects": [
                { "id": "FR-100", "names": ["Order"] },
                { "id": "FR-101", "names": ["Customer"] }
            ]
        },
        "bodyExtraction": body_extraction,
    });
    let record = extract_semantic_json(&request).unwrap();
    let value = serde_json::to_value(&record).unwrap();
    let errors: Vec<String> = match schema().validate(&value) {
        Ok(()) => Vec::new(),
        Err(errs) => errs
            .map(|e| format!("{} at {}", e, e.instance_path))
            .collect(),
    };
    assert!(errors.is_empty(), "{errors:?}\n{value:#}");
    value
}

fn line_of(md: &str, needle: &str) -> u64 {
    let (needle, exact) = match needle.strip_suffix('\n') {
        Some(n) => (n, true),
        None => (needle, false),
    };
    md.split('\n')
        .position(|l| {
            if exact {
                l == needle
            } else {
                l.contains(needle)
            }
        })
        .unwrap_or_else(|| panic!("{needle:?} not in document")) as u64
        + 1
}

fn codes(record: &Value) -> Vec<(String, u64)> {
    record["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| {
            (
                d["code"].as_str().unwrap().to_string(),
                d["line"].as_u64().unwrap_or(0),
            )
        })
        .collect()
}

fn has(record: &Value, code: &str, line: u64) -> bool {
    codes(record).iter().any(|(c, l)| c == code && *l == line)
}

fn assert_span(entry: &Value, md: &str, line: u64) {
    let text = md.split('\n').nth(line as usize - 1).unwrap();
    assert_eq!(
        entry["sourceSpan"],
        json!({
            "sourceIdentity": "ix://agent-ix/shop/spec",
            "path": PATH,
            "startLine": line,
            "startColumn": 1,
            "endLine": line,
            "endColumn": text.len() + 1,
        }),
        "{entry:#}"
    );
}

fn tables() -> Value {
    let row = |section: &str, columns: &[&str]| json!({ "from": "table_row", "under_section": section, "assert": { "columns": columns } });
    json!({ "fields": {
        "population": row("Members", &["Type", "Extent"]),
        "values": row("Values", &["Value", "Description"]),
        "states": row("States", &["State", "Description"]),
        "transitions": row("Transitions", &["From", "To", "Trigger", "Guard", "Emits"]),
        "steps": row("Steps", &["Step", "Kind", "Consumes", "Emits", "Description"]),
        "members": row("Aggregate Members", &["Member", "Multiplicity"]),
        "vocabulary": row("Ubiquitous Language", &["Term", "Description"]),
    }})
}

const FRONTMATTER: &str = "---\nid: FR-102\ntitle: PriorityOrder\nobject: entity\nabstract: true\nrelationships:\n  - target: \"ix://agent-ix/shop/spec/functional/FR-100\"\n    type: \"specializes\"\n  - target: \"ix://agent-ix/shop/spec/functional/FR-101\"\n    type: \"traces_to\"\n  - target: \"ix://agent-ix/shop/spec/functional/FR-103\"\n    type: \"specializes\"\n---\n# FR-102: PriorityOrder\n";

const PROPERTIES: &str = "---\nid: FR-102\ntitle: PriorityOrder\nobject: entity\n---\n# FR-102: PriorityOrder\n\n## Properties\n\n| Field | Type | Multiplicity | Constraints | Presence | Subsets | Redefines |\n|---|---|---|---|---|---|---|\n| id | UUID | 1 | identity | required | | |\n| lines | String | 0..* | | | parts, items | |\n| parts | String | 0..* | | | | |\n| label | String | 0..1 | | optional | | name |\n";

fn operations(requires: &str) -> String {
    format!("---\nid: FR-102\ntitle: PriorityOrder\nobject: entity\n---\n# FR-102: PriorityOrder\n\n## Invariants\n\n### placed\n\n```ocl\nx\n```\n\n### shipped\n\n```ocl\ny\n```\n\n## Operations\n\n### ship\n\n{requires}\nEnsures: shipped\nModifies: self.status, total\nCreates: Shipment\nDeletes: cart.items\n")
}

const POPULATION: &str = "---\nid: POP-001\ntitle: Shop Population\nobject: population\n---\n# POP-001: Shop Population\n\n## Members\n\n| Type | Extent |\n|---|---|\n| Order | 0..* |\n| Customer | 1..* |\n";

const OBJECT_SECTIONS: &str = "---\nid: FR-104\ntitle: Fulfilment\nobject: process\n---\n# FR-104: Fulfilment\n\n## Values\n\n| Value | Description |\n|---|---|\n| express | Next day |\n| standard | |\n\n## States\n\n| State | Description |\n|---|---|\n| open | Accepting lines |\n| shipped | Left the warehouse |\n\n## Transitions\n\n| From | To | Trigger | Guard | Emits |\n|---|---|---|---|---|\n| open | shipped | ship | ready | OrderShipped |\n\n## Steps\n\n| Step | Kind | Consumes | Emits | Description |\n|---|---|---|---|---|\n| pick | command | OrderPlaced | Picked, Packed | Pick the lines |\n| await | wait | | | |\n\n## Aggregate Members\n\n| Member | Multiplicity |\n|---|---|\n| OrderLine | 1..* |\n\n## Ubiquitous Language\n\n| Term | Description |\n|---|---|\n| Backorder | A line awaiting stock |\n\n## Invariants\n\n### ready\n\n```ocl\nx\n```\n\n## Operations\n\n### ship\n\nReturns: String[1]\n";

#[trace("TC-1840", "FR-075-AC-1")]
#[test]
fn generalization_and_abstract_from_frontmatter() {
    let record = extract(
        FRONTMATTER,
        &["generalization", "abstract-types"],
        json!(null),
    );
    let model = &record["model"];
    let supertypes = model["supertypes"].as_array().unwrap();
    assert_eq!(supertypes.len(), 2, "{model:#}");
    assert_eq!(
        supertypes[0]["target"],
        "ix://agent-ix/shop/spec/functional/FR-100"
    );
    assert_eq!(
        supertypes[1]["target"],
        "ix://agent-ix/shop/spec/functional/FR-103"
    );
    let item_lines: Vec<u64> = FRONTMATTER
        .split('\n')
        .enumerate()
        .filter(|(_, l)| l.starts_with("  - target:"))
        .map(|(i, _)| i as u64 + 1)
        .collect();
    assert_span(&supertypes[0], FRONTMATTER, item_lines[0]);
    assert_span(&supertypes[1], FRONTMATTER, item_lines[2]);
    assert_eq!(model["abstract"]["value"], true);
    assert_span(
        &model["abstract"],
        FRONTMATTER,
        line_of(FRONTMATTER, "abstract:"),
    );
    assert_eq!(record["availability"]["model"]["state"], "available");

    let bad = FRONTMATTER.replace("abstract: true", "abstract: sometimes");
    let record = extract(&bad, &["generalization", "abstract-types"], json!(null));
    assert!(
        has(
            &record,
            "semantic.invalid-model-cell",
            line_of(&bad, "abstract:")
        ),
        "{record:#}"
    );
    assert_eq!(record["availability"]["model"]["state"], "unavailable");
    assert!(record.get("model").is_none());
}

#[trace("TC-1841", "FR-075-AC-2")]
#[test]
fn presence_subsets_redefines_columns() {
    let record = extract(
        PROPERTIES,
        &["presence", "subsetting", "redefinition"],
        json!(null),
    );
    assert_eq!(
        record["availability"]["fields"]["state"], "available",
        "{record:#}"
    );
    assert_eq!(record["fields"].as_array().unwrap().len(), 4);
    let features = record["model"]["fieldFeatures"].as_array().unwrap();
    assert_eq!(features.len(), 3, "the `parts` row declares no feature");
    assert_eq!(features[0]["field"], "id");
    assert_eq!(features[0]["presence"], "required");
    assert_span(&features[0], PROPERTIES, line_of(PROPERTIES, "| id |"));
    assert_eq!(features[1]["subsets"], json!(["parts", "items"]));
    assert_span(&features[1], PROPERTIES, line_of(PROPERTIES, "| lines |"));
    assert_eq!(features[2]["presence"], "optional");
    assert_eq!(features[2]["redefines"], "name");

    let bad = PROPERTIES.replace("| required |", "| maybe |");
    let record = extract(
        &bad,
        &["presence", "subsetting", "redefinition"],
        json!(null),
    );
    assert!(
        has(
            &record,
            "semantic.invalid-model-cell",
            line_of(&bad, "| id |")
        ),
        "{record:#}"
    );
    assert_eq!(record["availability"]["fields"]["state"], "unavailable");
    assert!(record.get("model").is_none());
}

#[trace("TC-1842", "FR-075-AC-3")]
#[test]
fn operation_contract_and_frame_lines() {
    let md = operations("Requires: placed");
    let record = extract(&md, &["operation-contracts", "effect-frames"], json!(null));
    let op = &record["operations"][0];
    assert_eq!(op["pre"][0]["clauseId"], "placed", "{record:#}");
    assert_eq!(op["post"][0]["clauseId"], "shipped");
    let frame = &record["model"]["operationFrames"][0];
    assert_eq!(frame["operation"], "ship");
    assert_eq!(frame["requires"], json!(["placed"]));
    assert_eq!(frame["ensures"], json!(["shipped"]));
    assert_eq!(frame["modifies"], json!(["self.status", "total"]));
    assert_eq!(frame["creates"], json!(["Shipment"]));
    assert_eq!(frame["deletes"], json!(["cart.items"]));
    assert_span(frame, &md, line_of(&md, "### ship\n"));

    let md = operations("Pre: placed\nRequires: placed");
    let record = extract(&md, &["operation-contracts", "effect-frames"], json!(null));
    assert!(
        has(
            &record,
            "semantic.duplicate-operation-line",
            line_of(&md, "Requires:")
        ),
        "{record:#}"
    );

    let md = operations("Requires: nowhere");
    let record = extract(&md, &["operation-contracts", "effect-frames"], json!(null));
    assert!(
        has(
            &record,
            "semantic.dangling-clause-ref",
            line_of(&md, "Requires:")
        ),
        "{record:#}"
    );
    assert_eq!(record["availability"]["operations"]["state"], "unavailable");
    assert!(record.get("model").is_none());
}

#[trace("TC-1843", "FR-075-AC-4")]
#[test]
fn population_members_with_extent() {
    let record = extract(POPULATION, &[], tables());
    let members = record["model"]["population"]["members"]
        .as_array()
        .unwrap_or_else(|| panic!("{record:#}"));
    assert_eq!(members.len(), 2);
    assert_eq!(
        members[0]["type"]["target"], "ix://agent-ix/shop/type/Order",
        "{record:#}"
    );
    assert_eq!(members[0]["extent"]["lower"], 0);
    assert_eq!(members[1]["extent"]["lower"], 1);
    assert_span(&members[0], POPULATION, line_of(POPULATION, "| Order |"));
    assert_span(&members[1], POPULATION, line_of(POPULATION, "| Customer |"));
}

#[trace("TC-1844", "FR-075-AC-5")]
#[test]
fn object_type_sections() {
    let md = OBJECT_SECTIONS;
    let record = extract(md, &[], tables());
    assert_eq!(
        record["availability"]["model"]["state"], "available",
        "{record:#}"
    );
    let model = &record["model"];
    assert_eq!(model["values"][0]["value"], "express");
    assert_eq!(model["values"][0]["doc"], "Next day");
    assert!(model["values"][1].get("doc").is_none());
    assert_span(&model["values"][1], md, line_of(md, "| standard |"));
    assert_eq!(model["states"][1]["value"], "shipped");
    let t = &model["transitions"][0];
    assert_eq!(
        (
            &t["from"],
            &t["to"],
            &t["trigger"],
            &t["guard"],
            &t["emits"]
        ),
        (
            &json!("open"),
            &json!("shipped"),
            &json!("ship"),
            &json!("ready"),
            &json!("OrderShipped")
        )
    );
    assert_span(t, md, line_of(md, "| open | shipped |"));
    assert_eq!(model["steps"][0]["kind"], "command");
    assert_eq!(model["steps"][0]["emits"], json!(["Picked", "Packed"]));
    assert_eq!(model["steps"][1]["kind"], "wait");
    assert_eq!(model["members"][0]["target"], "OrderLine");
    assert_eq!(model["members"][0]["multiplicity"]["lower"], 1);
    assert_eq!(model["vocabulary"][0]["term"], "Backorder");
    assert_span(&model["vocabulary"][0], md, line_of(md, "| Backorder |"));

    for (from, to, code) in [
        (
            "| open | shipped |",
            "| open | closed |",
            "semantic.unknown-state",
        ),
        (
            "| ship | ready |",
            "| cancel | ready |",
            "semantic.unknown-trigger",
        ),
        (
            "| pick | command |",
            "| pick | shout |",
            "semantic.invalid-model-cell",
        ),
    ] {
        let bad = md.replace(from, to);
        let record = extract(&bad, &[], tables());
        assert!(has(&record, code, line_of(&bad, to)), "{code}: {record:#}");
        assert_eq!(record["availability"]["model"]["state"], "unavailable");
        assert!(record.get("model").is_none());
    }
}

#[trace("TC-1845", "FR-075-AC-6")]
#[test]
fn undeclared_features_are_refused() {
    let ops = operations("Requires: placed");
    // (document, feature, declaring line, owning kind)
    let cases: [(&str, &str, u64, &str); 13] = [
        (
            FRONTMATTER,
            "abstract-types",
            line_of(FRONTMATTER, "abstract:"),
            "model",
        ),
        (
            FRONTMATTER,
            "generalization",
            line_of(FRONTMATTER, "  - target:"),
            "model",
        ),
        (
            PROPERTIES,
            "presence",
            line_of(PROPERTIES, "| Field |"),
            "fields",
        ),
        (
            PROPERTIES,
            "subsetting",
            line_of(PROPERTIES, "| Field |"),
            "fields",
        ),
        (
            PROPERTIES,
            "redefinition",
            line_of(PROPERTIES, "| Field |"),
            "fields",
        ),
        (
            &ops,
            "operation-contracts",
            line_of(&ops, "Requires:"),
            "operations",
        ),
        (
            &ops,
            "effect-frames",
            line_of(&ops, "Modifies:"),
            "operations",
        ),
        (
            POPULATION,
            "population",
            line_of(POPULATION, "| Type |"),
            "model",
        ),
        (
            OBJECT_SECTIONS,
            "values",
            line_of(OBJECT_SECTIONS, "| Value |"),
            "model",
        ),
        (
            OBJECT_SECTIONS,
            "states",
            line_of(OBJECT_SECTIONS, "| State |"),
            "model",
        ),
        (
            OBJECT_SECTIONS,
            "transitions",
            line_of(OBJECT_SECTIONS, "| From |"),
            "model",
        ),
        (
            OBJECT_SECTIONS,
            "steps",
            line_of(OBJECT_SECTIONS, "| Step |"),
            "model",
        ),
        (
            OBJECT_SECTIONS,
            "vocabulary",
            line_of(OBJECT_SECTIONS, "| Term |"),
            "model",
        ),
    ];
    for (md, feature, line, kind) in cases {
        let record = extract(md, &[], json!(null));
        let refusal = record["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .find(|d| d["code"] == "semantic.feature-not-extractable" && d["reason"] == feature)
            .unwrap_or_else(|| panic!("{feature}: {record:#}"));
        assert_eq!(refusal["line"], line, "{feature}");
        assert_eq!(refusal["severity"], "error", "{feature}");
        let message = refusal["message"].as_str().unwrap();
        assert!(message.contains(PATH), "{feature}: {message}");
        assert_eq!(
            record["availability"][kind]["state"], "unavailable",
            "{feature}"
        );
        assert!(record.get("model").is_none(), "{feature}");
    }

    // A table under a section its locator does not name is refused too.
    let moved = OBJECT_SECTIONS.replace("## Aggregate Members", "## Parts");
    let record = extract(&moved, &[], tables());
    let refusal = record["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["code"] == "semantic.feature-not-extractable")
        .unwrap_or_else(|| panic!("{record:#}"));
    assert_eq!(refusal["reason"], "members");
    assert_eq!(refusal["line"], line_of(&moved, "| Member |"));
    assert!(refusal["message"].as_str().unwrap().contains("`## Parts`"));
}

#[trace("TC-1846", "FR-075-AC-7")]
#[trace("TC-1846", "FR-075-CON-1")]
#[test]
fn records_without_model_features_are_unchanged() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cases: Value =
        serde_json::from_slice(&fs::read(root.join("tests/fixtures/semantic/cases.json")).unwrap())
            .unwrap();
    let schema = schema();
    for case in cases["cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let value = serde_json::to_value(extract_semantic_json(&case["input"]).unwrap()).unwrap();
        assert!(schema.is_valid(&value), "{name}");
        assert!(value.get("model").is_none(), "{name}");
        assert!(value["availability"].get("model").is_none(), "{name}");
    }
    // The FR-070 typed table without feature columns declares nothing.
    let plain = "---\nid: FR-102\ntitle: PriorityOrder\nobject: entity\n---\n# FR-102: PriorityOrder\n\n## Properties\n\n| Field | Type | Multiplicity | Constraints |\n|---|---|---|---|\n| id | UUID | 1 | identity |\n";
    let record = extract(
        plain,
        &["presence", "subsetting", "redefinition"],
        json!(null),
    );
    assert_eq!(
        record["availability"]["fields"]["state"], "available",
        "{plain}\n{record:#}"
    );
    assert!(record.get("model").is_none());
    assert!(record["availability"].get("model").is_none());
}
