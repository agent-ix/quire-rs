//! FR-075 model feature extraction (TC-1840..TC-1847).

use std::fs;
use std::path::PathBuf;

use ix_trace_rs::trace;
use jsonschema::Validator;
use quire_rs::semantic::python_entry::extract_semantic_json;
use quire_rs::semantic::{compile_module_schema, SEMANTIC_V1_SCHEMA};
use serde_json::{json, Value};

const PATH: &str = "spec/functional/FR-100.md";

fn schema() -> Validator {
    compile_module_schema(
        &serde_json::from_str::<Value>(SEMANTIC_V1_SCHEMA).unwrap(),
        &|_| None,
        "https://agent-ix.github.io/quire-rs/schemas/output/",
    )
    .unwrap()
}

/// Extract `md` under `mappings` and `body_extraction`; every record must
/// validate against semantic-v1 (FR-075-AC-7).
fn extract(md: &str, mappings: &[&str], body_extraction: Value) -> Value {
    let request = json!({
        "markdown": md,
        "module": {
            "contractVersion": "1.0.0",
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
    extract_request(&request)
}

/// Extract `request`; the record must validate against semantic-v1.
fn extract_request(request: &Value) -> Value {
    let record = extract_semantic_json(request).unwrap();
    let value = serde_json::to_value(&record).unwrap();
    let errors: Vec<String> = schema()
        .iter_errors(&value)
        .map(|e| format!("{} at {}", e, e.instance_path()))
        .collect();
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

/// The one diagnostic with `code` and `reason`.
fn diagnostic<'r>(record: &'r Value, code: &str, reason: &str) -> &'r Value {
    record["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["code"] == code && d["reason"] == reason)
        .unwrap_or_else(|| panic!("{code} {reason}: {record:#}"))
}

fn assert_model_unavailable(record: &Value) {
    assert_eq!(
        record["availability"]["model"]["state"], "unavailable",
        "{record:#}"
    );
    assert!(record.get("model").is_none(), "{record:#}");
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

/// A `table_row` locator under `section` asserting `columns`.
fn row(section: &str, columns: &[&str], optional: &[&str]) -> Value {
    json!({
        "from": "table_row",
        "under_section": section,
        "assert": { "columns": columns, "optional_columns": optional },
    })
}

/// A typed extraction DSL whose `yield_pattern.match` holds `locators`.
fn dsl(locators: Value) -> Value {
    json!({ "yield_pattern": { "match": locators } })
}

fn tables() -> Value {
    dsl(json!({
        "population": row("Members", &["Type"], &[]),
        "values": row("Values", &["Value", "Description"], &["Description"]),
        "states": row("States", &["State", "Description"], &[]),
        "transitions": row(
            "Transitions",
            &["From", "To", "Trigger", "Guard", "Emits"],
            &["Guard", "Emits"],
        ),
        "steps": row("Steps", &["Step", "Kind", "Consumes", "Emits", "Description"], &[]),
        "members": row("Aggregate Members", &["Member", "Multiplicity"], &[]),
        "vocabulary": row("Ubiquitous Language", &["Term", "Description"], &[]),
    }))
}

const FRONTMATTER: &str = "---\nid: FR-102\ntitle: PriorityOrder\nobject: entity\nabstract: true\nrelationships:\n  - target: \"ix://agent-ix/shop/spec/functional/FR-100\"\n    type: \"specializes\"\n  - target: \"ix://agent-ix/shop/spec/functional/FR-101\"\n    type: \"traces_to\"\n  - target: \"ix://agent-ix/shop/spec/functional/FR-103\"\n    type: \"specializes\"\n---\n# FR-102: PriorityOrder\n";

const PROPERTIES: &str = "---\nid: FR-102\ntitle: PriorityOrder\nobject: entity\n---\n# FR-102: PriorityOrder\n\n## Properties\n\n| Field | Type | Multiplicity | Constraints | Presence | Subsets | Redefines |\n|---|---|---|---|---|---|---|\n| id | UUID | 1 | identity | required | | |\n| lines | String | 0..* | | | parts, items | |\n| parts | String | 0..* | | | | |\n| label | String | 0..1 | | optional | | name |\n";

fn operations(pre: &str) -> String {
    format!("---\nid: FR-102\ntitle: PriorityOrder\nobject: entity\n---\n# FR-102: PriorityOrder\n\n## Invariants\n\n### placed\n\n```quire\nx\n```\n\n### shipped\n\n```quire\ny\n```\n\n## Operations\n\n### ship\n\n{pre}\nPost: shipped\nModifies: self.status, total\nCreates: Shipment\nDeletes: cart.items\n")
}

const POPULATION: &str = "---\nid: POP-001\ntitle: Shop Population\nobject: population\nextent: closed\n---\n# POP-001: Shop Population\n\n## Members\n\n| Type |\n|---|\n| Order |\n| Customer |\n";

const OBJECT_SECTIONS: &str = "---\nid: FR-104\ntitle: Fulfilment\nobject: process\n---\n# FR-104: Fulfilment\n\n## Values\n\n| Value | Description |\n|---|---|\n| express | Next day |\n| standard | |\n\n## States\n\n| State | Description |\n|---|---|\n| open | Accepting lines |\n| shipped | Left the warehouse |\n\n## Transitions\n\n| From | To | Trigger | Guard | Emits |\n|---|---|---|---|---|\n| open | shipped | ship | ready | OrderShipped |\n\n## Steps\n\n| Step | Kind | Consumes | Emits | Description |\n|---|---|---|---|---|\n| pick | command | OrderPlaced | Picked, Packed | Pick the lines |\n| await | wait | | | |\n\n## Aggregate Members\n\n| Member | Multiplicity |\n|---|---|\n| OrderLine | 1..* |\n\n## Ubiquitous Language\n\n| Term | Description |\n|---|---|\n| Backorder | A line awaiting stock |\n\n## Invariants\n\n### ready\n\n```quire\nx\n```\n\n## Operations\n\n### ship\n\nReturns: String[1]\n";

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
    assert_model_unavailable(&record);

    // A second specializes of the same target, and a non-string target.
    for (to, code) in [
        (
            "FR-100\"\n    type: \"specializes\"\n---",
            "semantic.duplicate-model-entry",
        ),
        (
            "x\"\n    type: \"specializes\"\n---",
            "semantic.invalid-model-cell",
        ),
    ] {
        let mut bad = FRONTMATTER.replace("FR-103\"\n    type: \"specializes\"\n---", to);
        if code == "semantic.invalid-model-cell" {
            bad = bad.replace(
                "target: \"ix://agent-ix/shop/spec/functional/x\"",
                "target: 7",
            );
        }
        let record = extract(&bad, &["generalization", "abstract-types"], json!(null));
        assert!(has(&record, code, item_lines[2]), "{code}: {record:#}");
        assert_model_unavailable(&record);
    }

    // A refused operation line makes the whole model unavailable, even
    // though the frontmatter features read cleanly.
    let mixed =
        operations("Pre: placed").replace("object: entity\n", "object: entity\nabstract: true\n");
    let record = extract(&mixed, &["abstract-types"], json!(null));
    let refusal = diagnostic(&record, "semantic.feature-not-extractable", "effect-frames");
    assert_eq!(refusal["line"], line_of(&mixed, "Modifies:"));
    assert_model_unavailable(&record);
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
    assert_eq!(record["availability"]["model"]["state"], "available");

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
    assert_model_unavailable(&record);

    // An unrecognised or repeated feature column after the typed header is
    // refused; the table is never read as a legacy form.
    let header = line_of(PROPERTIES, "| Field |");
    for column in ["| Colour |", "| Presence |"] {
        let bad = PROPERTIES.replace("| Redefines |", column);
        let record = extract(
            &bad,
            &["presence", "subsetting", "redefinition"],
            json!(null),
        );
        let d = record["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .find(|d| d["code"] == "semantic.invalid-model-cell")
            .unwrap_or_else(|| panic!("{column}: {record:#}"));
        assert_eq!(d["line"], header, "{column}");
        assert_eq!(d["section"], "Properties", "{column}");
        assert_span(d, &bad, header);
        assert_eq!(
            record["diagnostics"].as_array().unwrap().len(),
            1,
            "{record:#}"
        );
        assert_eq!(record["availability"]["fields"]["state"], "unavailable");
        assert_model_unavailable(&record);
    }

    // A typed feature table in the preamble is refused.
    let preamble = PROPERTIES.replace("## Properties\n\n", "");
    let record = extract(
        &preamble,
        &["presence", "subsetting", "redefinition"],
        json!(null),
    );
    let d = diagnostic(&record, "semantic.feature-not-extractable", "presence");
    assert_eq!(d["section"], "preamble");
    assert_eq!(d["line"], line_of(&preamble, "| Field |"));
    assert_model_unavailable(&record);
}

#[trace("TC-1842", "FR-075-AC-3")]
#[test]
fn operation_contract_and_frame_lines() {
    let md = operations("Pre: placed");
    let record = extract(&md, &["effect-frames"], json!(null));
    let op = &record["operations"][0];
    assert_eq!(op["pre"][0]["clauseId"], "placed", "{record:#}");
    assert_eq!(op["post"][0]["clauseId"], "shipped");
    let frame = &record["model"]["operationFrames"][0];
    assert_eq!(frame["operation"], "ship");
    assert_eq!(frame["pre"], json!(["placed"]));
    assert_eq!(frame["post"], json!(["shipped"]));
    assert_eq!(frame["modifies"], json!(["self.status", "total"]));
    assert_eq!(frame["creates"], json!(["Shipment"]));
    assert_eq!(frame["deletes"], json!(["cart.items"]));
    assert_span(frame, &md, line_of(&md, "### ship\n"));
    assert_eq!(record["availability"]["model"]["state"], "available");

    let md = operations("Pre: placed\nPre: placed");
    let second = md
        .split('\n')
        .collect::<Vec<_>>()
        .iter()
        .rposition(|l| l.starts_with("Pre:"))
        .unwrap() as u64
        + 1;
    let record = extract(&md, &["effect-frames"], json!(null));
    assert!(
        has(&record, "semantic.duplicate-operation-line", second),
        "{record:#}"
    );

    let md = operations("Pre: placed\nPost: shipped");
    let second = md
        .split('\n')
        .collect::<Vec<_>>()
        .iter()
        .rposition(|l| l.starts_with("Post:"))
        .unwrap() as u64
        + 1;
    let record = extract(&md, &["effect-frames"], json!(null));
    assert!(
        has(&record, "semantic.duplicate-operation-line", second),
        "{record:#}"
    );

    let md = operations("Pre: nowhere");
    let record = extract(&md, &["effect-frames"], json!(null));
    assert!(
        has(
            &record,
            "semantic.dangling-clause-ref",
            line_of(&md, "Pre:")
        ),
        "{record:#}"
    );
    assert_eq!(record["availability"]["operations"]["state"], "unavailable");
    assert_model_unavailable(&record);

    let md = operations("Pre: placed\nModifies: status");
    let record = extract(&md, &["effect-frames"], json!(null));
    let second = md
        .split('\n')
        .collect::<Vec<_>>()
        .iter()
        .rposition(|l| l.starts_with("Modifies:"))
        .unwrap() as u64
        + 1;
    assert!(
        has(&record, "semantic.duplicate-operation-line", second),
        "{record:#}"
    );
    assert_model_unavailable(&record);

    let md = operations("Pre: placed").replace("self.status", "self..status");
    let record = extract(&md, &["effect-frames"], json!(null));
    assert!(
        has(
            &record,
            "semantic.invalid-model-cell",
            line_of(&md, "Modifies:")
        ),
        "{record:#}"
    );
    assert_model_unavailable(&record);
}

#[trace("TC-1843", "FR-075-AC-4")]
#[test]
fn population_members_with_one_extent() {
    let record = extract(POPULATION, &[], tables());
    assert_eq!(
        record["model"]["population"]["extent"], "closed",
        "{record:#}"
    );
    let members = record["model"]["population"]["members"]
        .as_array()
        .unwrap_or_else(|| panic!("{record:#}"));
    assert_eq!(members.len(), 2);
    assert_eq!(
        members[0]["type"]["target"],
        json!({ "kind": "SemanticId", "value": "ix://agent-ix/shop/type/Order" }),
        "{record:#}"
    );
    assert!(members[0].get("extent").is_none(), "{record:#}");
    assert_span(
        &json!({ "sourceSpan": record["model"]["population"]["extentSpan"] }),
        POPULATION,
        line_of(POPULATION, "extent:"),
    );
    assert_span(&members[0], POPULATION, line_of(POPULATION, "| Order |"));
    assert_span(&members[1], POPULATION, line_of(POPULATION, "| Customer |"));
}

#[trace("TC-1843", "FR-075-AC-4")]
#[test]
fn population_extent_open_and_required() {
    let open = POPULATION.replace("extent: closed", "extent: open");
    let record = extract(&open, &[], tables());
    assert_eq!(
        record["model"]["population"]["extent"], "open",
        "{record:#}"
    );

    let missing = POPULATION.replace("extent: closed\n", "");
    let record = extract(&missing, &[], tables());
    assert!(
        has(
            &record,
            "semantic.invalid-model-cell",
            line_of(&missing, "| Type |")
        ),
        "{record:#}"
    );
    assert_model_unavailable(&record);

    let old = POPULATION.replace(
        "| Type |\n|---|\n| Order |\n| Customer |",
        "| Type | Extent |\n|---|---|\n| Order | 0..* |\n| Customer | 1..* |",
    );
    let record = extract(&old, &[], tables());
    assert!(
        has(
            &record,
            "semantic.feature-not-extractable",
            line_of(&old, "| Type |")
        ),
        "{record:#}"
    );
    assert_model_unavailable(&record);

    let bad = POPULATION.replace("closed", "bounded");
    let record = extract(&bad, &[], tables());
    assert!(
        has(
            &record,
            "semantic.invalid-model-cell",
            line_of(&bad, "extent:")
        ),
        "{record:#}"
    );
    assert_model_unavailable(&record);
}

/// FR-075-AC-4: the retired `Type | Extent` table is refused whatever the
/// manifest declares, naming the current shape.
#[trace("TC-1843", "FR-075-AC-4")]
#[test]
fn legacy_population_table_is_refused_under_either_manifest() {
    let old = POPULATION.replace("extent: closed\n", "").replace(
        "| Type |\n|---|\n| Order |\n| Customer |",
        "| Type | Extent |\n|---|---|\n| Order | 0..* |\n| Customer | 1..* |",
    );
    let old_manifest = dsl(json!({
        "population": row("Members", &["Type", "Extent"], &[]),
    }));
    for manifest in [old_manifest, tables()] {
        let record = extract(&old, &[], manifest);
        let d = diagnostic(&record, "semantic.feature-not-extractable", "population");
        assert_eq!(d["line"], line_of(&old, "| Type |"), "{record:#}");
        assert_eq!(d["section"], "Members", "{record:#}");
        assert!(
            d["message"]
                .as_str()
                .unwrap()
                .contains("frontmatter `extent"),
            "{record:#}"
        );
        assert_model_unavailable(&record);
    }
}

/// FR-075-AC-6: `extent` is a population feature; a module that declares no
/// population refuses it, and a population type with no `Type` table refuses
/// an `extent` that has nothing to describe.
#[trace("TC-1845", "FR-075-AC-6")]
#[test]
fn extent_without_a_declared_population_is_refused() {
    let record = extract(POPULATION, &[], dsl(json!({})));
    let d = diagnostic(&record, "semantic.feature-not-extractable", "population");
    assert_eq!(d["line"], line_of(POPULATION, "extent:"), "{record:#}");
    assert_eq!(d["section"], "frontmatter", "{record:#}");
    assert_model_unavailable(&record);

    let no_table = POPULATION.replace(
        "| Type |\n|---|\n| Order |\n| Customer |\n",
        "Nothing here.\n",
    );
    let record = extract(&no_table, &[], tables());
    assert!(
        has(
            &record,
            "semantic.invalid-model-cell",
            line_of(&no_table, "extent:")
        ),
        "{record:#}"
    );
    assert_model_unavailable(&record);
}

#[trace("TC-1844", "FR-075-AC-5")]
#[test]
fn object_type_sections() {
    let md = OBJECT_SECTIONS;
    let extract = |md: &str, _: &[&str], body: Value| extract_request(&shop_request(md, body));
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
            &json!("ix://agent-ix/shop/EV-002")
        )
    );
    assert_span(t, md, line_of(md, "| open | shipped |"));
    assert_eq!(model["steps"][0]["kind"], "command");
    assert_eq!(
        model["steps"][0]["emits"],
        json!(["ix://agent-ix/shop/EV-003", "ix://agent-ix/shop/EV-004"])
    );
    assert_eq!(model["steps"][1]["kind"], "wait");
    assert_eq!(model["members"][0]["target"], "OrderLine");
    assert_eq!(model["members"][0]["multiplicity"]["lower"], 1);
    assert_eq!(model["vocabulary"][0]["term"], "Backorder");
    assert_span(&model["vocabulary"][0], md, line_of(md, "| Backorder |"));

    for (from, to, code) in [
        (
            "| ready | OrderShipped |",
            "| nowhere | OrderShipped |",
            "semantic.dangling-clause-ref",
        ),
        (
            "| standard | |",
            "| express | |",
            "semantic.duplicate-model-entry",
        ),
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
        assert_model_unavailable(&record);
    }

    // A second table under a declared section.
    let twice = md.replace(
        "| Backorder | A line awaiting stock |\n",
        "| Backorder | A line awaiting stock |\n\n| Term | Description |\n|---|---|\n| Stockout | |\n",
    );
    let record = extract(&twice, &[], tables());
    assert!(
        has(
            &record,
            "semantic.duplicate-section",
            line_of(&twice, "| Stockout |") - 2
        ),
        "{record:#}"
    );
    assert_model_unavailable(&record);

    // A failed states table suppresses the unknown-state check.
    let bad = md
        .replace(
            "| open | Accepting lines |",
            "| open now | Accepting lines |",
        )
        .replace("| open | shipped | ship |", "| gone | shipped | ship |");
    let record = extract(&bad, &[], tables());
    assert!(
        has(
            &record,
            "semantic.invalid-model-cell",
            line_of(&bad, "| open now |")
        ),
        "{record:#}"
    );
    assert!(
        !codes(&record)
            .iter()
            .any(|(c, _)| c == "semantic.unknown-state"),
        "{record:#}"
    );

    // Optional columns may be omitted; a required column may not.
    let slim = md.replace(
        "| From | To | Trigger | Guard | Emits |\n|---|---|---|---|---|\n| open | shipped | ship | ready | OrderShipped |",
        "| From | To | Trigger |\n|---|---|---|\n| open | shipped | ship |",
    );
    let record = extract(&slim, &[], tables());
    let t = &record["model"]["transitions"][0];
    assert_eq!(t["trigger"], "ship", "{record:#}");
    assert!(
        t.get("guard").is_none() && t.get("emits").is_none(),
        "{t:#}"
    );
    let short = slim.replace(
        "| From | To | Trigger |\n|---|---|---|\n| open | shipped | ship |",
        "| From | To |\n|---|---|\n| open | shipped |",
    );
    let record = extract(&short, &[], tables());
    let d = diagnostic(&record, "semantic.feature-not-extractable", "transitions");
    assert_eq!(d["line"], line_of(&short, "| From | To |"));
    assert_model_unavailable(&record);

    // A declared section whose content is not the declared table is refused.
    let prose = "---\nid: FR-105\ntitle: Cart\nobject: aggregate\n---\n# FR-105: Cart\n\n## States & Transitions\n\n```mermaid\nstateDiagram-v2\n  open --> shipped\n```\n\n## Members\n\n- OrderLine 1..*\n";
    let locators = dsl(json!({
        "states": row("States & Transitions", &["State", "Description"], &[]),
        "transitions": row("States & Transitions", &["From", "To", "Trigger"], &[]),
        "members": row("Members", &["Member", "Multiplicity"], &[]),
    }));
    let record = extract(prose, &[], locators);
    for (reason, section, line) in [
        (
            "states",
            "States & Transitions",
            line_of(prose, "```mermaid"),
        ),
        ("members", "Members", line_of(prose, "- OrderLine")),
    ] {
        let d = diagnostic(&record, "semantic.feature-not-extractable", reason);
        assert_eq!(d["section"], section, "{reason}");
        assert_eq!(d["line"], line, "{reason}");
        assert_span(d, prose, line);
    }
    assert_model_unavailable(&record);
}

/// Provenance: agent-ix/quire-rs#569
#[trace("TC-1844", "FR-075-AC-5")]
#[test]
fn declared_section_body_can_contain_prose_beside_a_model_table() {
    let md = "---\nid: FR-200\ntitle: Assessment Domain\nobject: domain\n---\n# FR-200: Assessment Domain\n\n## Ubiquitous Language\n\n- **Authored** — written by a domain expert.\n\n| Term | Description |\n|---|---|\n| Derived | Produced from other declarations. |\n";
    let body_extraction = dsl(json!({
        "ubiquitous_language": {
            "from": "section_body",
            "after_heading": "Ubiquitous Language",
            "required": false,
        },
        "vocabulary": row("Ubiquitous Language", &["Term", "Description"], &[]),
    }));

    let record = extract(md, &[], body_extraction);
    assert_eq!(
        record["availability"]["model"]["state"], "available",
        "{record:#}"
    );
    assert_eq!(record["model"]["vocabulary"][0]["term"], "Derived");
    assert!(
        record["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .all(|diagnostic| diagnostic["code"] != "semantic.feature-not-extractable"),
        "{record:#}"
    );

    let table_only = extract(
        md,
        &[],
        dsl(json!({
            "vocabulary": row("Ubiquitous Language", &["Term", "Description"], &[]),
        })),
    );
    assert!(
        table_only["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|diagnostic| diagnostic["code"] == "semantic.feature-not-extractable"),
        "without a section-body declaration, list content remains refused: {table_only:#}"
    );
}

#[trace("TC-1845", "FR-075-AC-6")]
#[test]
fn undeclared_features_are_refused() {
    let ops = operations("Pre: placed");
    // (document, feature, declaring line, owning kind, section)
    let cases: [(&str, &str, u64, &str, &str); 13] = [
        (
            FRONTMATTER,
            "abstract-types",
            line_of(FRONTMATTER, "abstract:"),
            "model",
            "frontmatter",
        ),
        (
            FRONTMATTER,
            "generalization",
            line_of(FRONTMATTER, "  - target:"),
            "model",
            "frontmatter",
        ),
        (
            PROPERTIES,
            "presence",
            line_of(PROPERTIES, "| Field |"),
            "fields",
            "Properties",
        ),
        (
            PROPERTIES,
            "subsetting",
            line_of(PROPERTIES, "| Field |"),
            "fields",
            "Properties",
        ),
        (
            PROPERTIES,
            "redefinition",
            line_of(PROPERTIES, "| Field |"),
            "fields",
            "Properties",
        ),
        (
            &ops,
            "effect-frames",
            line_of(&ops, "Modifies:"),
            "operations",
            "Operations / ship",
        ),
        (
            POPULATION,
            "population",
            line_of(POPULATION, "| Type |"),
            "model",
            "Members",
        ),
        (
            OBJECT_SECTIONS,
            "values",
            line_of(OBJECT_SECTIONS, "| Value |"),
            "model",
            "Values",
        ),
        (
            OBJECT_SECTIONS,
            "states",
            line_of(OBJECT_SECTIONS, "| State |"),
            "model",
            "States",
        ),
        (
            OBJECT_SECTIONS,
            "transitions",
            line_of(OBJECT_SECTIONS, "| From |"),
            "model",
            "Transitions",
        ),
        (
            OBJECT_SECTIONS,
            "steps",
            line_of(OBJECT_SECTIONS, "| Step |"),
            "model",
            "Steps",
        ),
        (
            OBJECT_SECTIONS,
            "vocabulary",
            line_of(OBJECT_SECTIONS, "| Term |"),
            "model",
            "Ubiquitous Language",
        ),
        (
            OBJECT_SECTIONS,
            "members",
            line_of(OBJECT_SECTIONS, "| Member |"),
            "model",
            "Aggregate Members",
        ),
    ];
    for (md, feature, line, kind, section) in cases {
        let record = extract(md, &[], json!(null));
        let refusal = record["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .find(|d| {
                d["code"] == "semantic.feature-not-extractable"
                    && d["reason"] == feature
                    && d["line"] == line
            })
            .unwrap_or_else(|| panic!("{feature}: {record:#}"));
        assert_eq!(refusal["severity"], "error", "{feature}");
        assert_eq!(refusal["section"], section, "{feature}");
        assert_span(refusal, md, line);
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
    assert_eq!(refusal["section"], "Parts");

    // So is a model table before the first `##` heading.
    let preamble = "---\nid: FR-104\ntitle: Fulfilment\nobject: process\n---\n# FR-104: Fulfilment\n\n| Value | Description |\n|---|---|\n| express | |\n\n## Values\n\nNone.\n";
    let record = extract(preamble, &[], tables());
    let refusal = diagnostic(&record, "semantic.feature-not-extractable", "values");
    assert_eq!(refusal["section"], "preamble");
    assert_eq!(refusal["line"], line_of(preamble, "| Value |"));
    assert_model_unavailable(&record);
}

#[trace("TC-1847", "FR-075-AC-8")]
#[test]
fn model_identity_and_display_name() {
    let record = extract(
        FRONTMATTER,
        &["generalization", "abstract-types"],
        json!(null),
    );
    let model = &record["model"];
    assert_eq!(model["identity"]["value"], "FR-102", "{model:#}");
    assert_span(
        &model["identity"],
        FRONTMATTER,
        line_of(FRONTMATTER, "id: FR-102"),
    );
    assert_eq!(model["displayName"]["value"], "PriorityOrder");
    assert_span(
        &model["displayName"],
        FRONTMATTER,
        line_of(FRONTMATTER, "title:"),
    );
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
    // A case whose object type declares model tables (`bodyExtraction`) or
    // whose module maps `effect-frames` is a model case, not one of the records
    // without model features.
    for case in cases["cases"].as_array().unwrap().iter().filter(|c| {
        c["input"].get("bodyExtraction").is_none()
            && c["input"]["module"]["mappings"]
                .as_array()
                .is_none_or(Vec::is_empty)
    }) {
        let name = case["name"].as_str().unwrap();
        let input = case["input"].clone();
        let value = serde_json::to_value(extract_semantic_json(&input).unwrap()).unwrap();
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

/// The bundle `entries` declare: each `(id, object, names)` is a
/// `BundleIndex.artifacts` entry and a `BundleIndex.objects` entry.
fn bundle_of(entries: &[(&str, Option<&str>, &[&str])]) -> Value {
    let artifacts: Vec<Value> = entries
        .iter()
        .map(|(id, object, _)| json!({ "id": id, "object": object, "operations": [] }))
        .collect();
    let objects: Vec<Value> = entries
        .iter()
        .map(|(id, _, names)| json!({ "id": id, "names": names }))
        .collect();
    json!({ "package": "agent-ix/shop", "objects": objects, "artifacts": artifacts })
}

/// Events EV-001 `OrderPlaced`, EV-002 `OrderShipped`, EV-003 `Picked`,
/// EV-004 `Packed`, and the entity ENT-001 `Order`.
fn shop_bundle() -> Value {
    bundle_of(&[
        ("EV-001", Some("event"), &["OrderPlaced"]),
        ("EV-002", Some("event"), &["OrderShipped"]),
        ("EV-003", Some("event"), &["Picked"]),
        ("EV-004", Some("event"), &["Packed"]),
        ("ENT-001", Some("entity"), &["Order"]),
    ])
}

/// The request for `md` under `body_extraction` and the shop bundle.
fn shop_request(md: &str, body_extraction: Value) -> Value {
    json!({
        "markdown": md,
        "module": {
            "contractVersion": "1.0.0",
            "package": "agent-ix/spec-objects-fixture",
            "exports": ["state_machine"],
        },
        "path": PATH,
        "sourceIdentity": "ix://agent-ix/shop/spec",
        "bundle": shop_bundle(),
        "bodyExtraction": body_extraction,
    })
}

/// A state machine whose transition `Emits` is `emits` and whose one step
/// consumes `consumes` and emits `step_emits`; the transition row is line 19
/// and the step row line 25.
fn machine(emits: &str, consumes: &str, step_emits: &str) -> String {
    format!(
        "---\nid: SM-001\ntitle: OrderLifecycle\nobject: state_machine\n---\n# SM-001: OrderLifecycle\n\n## States\n\n| State | Description |\n|---|---|\n| open | |\n| shipped | |\n\n## Transitions\n\n| From | To | Trigger | Guard | Emits |\n|---|---|---|---|---|\n| open | shipped | ship | | {emits} |\n\n## Steps\n\n| Step | Kind | Consumes | Emits | Description |\n|---|---|---|---|---|\n| pick | command | {consumes} | {step_emits} | |\n\n## Operations\n\n### ship\n\nReturns: String[1]\n"
    )
}

const TRANSITION_LINE: u64 = 19;
const STEP_LINE: u64 = 25;

fn machine_tables() -> Value {
    dsl(json!({
        "states": row("States", &["State", "Description"], &[]),
        "transitions": row(
            "Transitions",
            &["From", "To", "Trigger", "Guard", "Emits"],
            &["Guard", "Emits"],
        ),
        "steps": row("Steps", &["Step", "Kind", "Consumes", "Emits", "Description"], &[]),
    }))
}

fn machine_request(emits: &str, consumes: &str, step_emits: &str) -> Value {
    shop_request(&machine(emits, consumes, step_emits), machine_tables())
}

/// A relation vocabulary for `object_type` with `allowed` links; events
/// carry the `event-like` role.
fn vocabulary(object_type: &str, allowed: Value) -> Value {
    json!({
        "objectType": object_type,
        "roles": { "event": ["event-like"], "entity": ["domain-object"] },
        "allowedLinks": { object_type: allowed },
    })
}

/// `request` with `key` set to `value`.
fn with(mut request: Value, key: &str, value: Value) -> Value {
    request[key] = value;
    request
}

/// `request` under a state-machine construct admitting `event-like` for
/// transitions and steps.
fn under_construct(request: Value) -> Value {
    let request = with(
        request,
        "relationVocabulary",
        vocabulary("state_machine", json!({})),
    );
    with(
        request,
        "constructReferences",
        json!({ "transitions": ["event-like"], "steps": ["event-like"] }),
    )
}

/// The declaration-reference diagnostics of `record`: code, reason, line.
fn reference_findings(record: &Value) -> Vec<(String, Option<String>, u64)> {
    record["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| {
            [
                "semantic.unknown-reference",
                "semantic.reference-kind-mismatch",
                "semantic.ambiguous-reference",
                "semantic.duplicate-model-entry",
                "semantic.invalid-model-cell",
                "semantic.unresolved-target",
            ]
            .contains(&d["code"].as_str().unwrap())
        })
        .map(|d| {
            (
                d["code"].as_str().unwrap().to_string(),
                d["reason"].as_str().map(str::to_string),
                d["line"].as_u64().unwrap(),
            )
        })
        .collect()
}

fn finding(code: &str, reason: Option<&str>, line: u64) -> (String, Option<String>, u64) {
    (code.to_string(), reason.map(str::to_string), line)
}

const EV1: &str = "ix://agent-ix/shop/EV-001";
const EV2: &str = "ix://agent-ix/shop/EV-002";

#[trace("TC-1993", "FR-075-AC-22", "FR-075-CON-2")]
// `Emits` and `Consumes` items resolve by title or id to SemanticIds of the
// bundle, never to the cell text.
#[test]
fn declaration_references_resolve_to_semantic_ids() {
    let record = extract_request(&machine_request(
        "OrderPlaced",
        "OrderPlaced, EV-002",
        "OrderShipped",
    ));
    assert_eq!(
        record["availability"]["model"]["state"], "available",
        "{record:#}"
    );
    let model = &record["model"];
    assert_eq!(model["transitions"][0]["emits"], EV1);
    assert_eq!(model["steps"][0]["consumes"], json!([EV1, EV2]));
    assert_eq!(model["steps"][0]["emits"], json!([EV2]));
    for (value, cell) in [
        (&model["transitions"][0]["emits"], "OrderPlaced"),
        (&model["steps"][0]["consumes"][0], "OrderPlaced"),
        (&model["steps"][0]["consumes"][1], "EV-002"),
        (&model["steps"][0]["emits"][0], "OrderShipped"),
    ] {
        let value = value.as_str().unwrap();
        assert!(value.starts_with("ix://"), "{value}");
        assert_ne!(value, cell);
    }
    assert!(reference_findings(&record).is_empty(), "{record:#}");

    let record = extract_request(&machine_request("", "OrderPlaced", ""));
    assert!(
        record["model"]["transitions"][0].get("emits").is_none(),
        "{record:#}"
    );
    assert!(record["model"]["steps"][0].get("emits").is_none());
}

#[trace("TC-1994", "FR-075-AC-23")]
// An unknown name and a declaration of a type no module source admits are
// refused; admission reads the construct references, then allowed_links,
// then admits any.
#[test]
fn unknown_and_unadmitted_references_are_refused() {
    let record = extract_request(&machine_request("OrderLost", "", ""));
    assert_eq!(
        reference_findings(&record),
        [finding(
            "semantic.unknown-reference",
            Some("unknown-name"),
            TRANSITION_LINE
        )]
    );
    let d = diagnostic(&record, "semantic.unknown-reference", "unknown-name");
    assert_eq!(d["severity"], "error");
    assert_eq!(d["section"], "Transitions");
    assert_eq!(d["sourceSpan"]["startLine"], TRANSITION_LINE);
    for needle in [PATH, "Transitions", "transitions", "OrderLost"] {
        assert!(d["message"].as_str().unwrap().contains(needle), "{d:#}");
    }
    assert_model_unavailable(&record);
    assert_eq!(
        record["availability"]["model"]["reason"],
        format!("entry-errors: lines {TRANSITION_LINE}")
    );

    // The construct entry `transitions: [event-like]` refuses an entity.
    let record = extract_request(&under_construct(machine_request("Order", "", "")));
    assert_eq!(
        reference_findings(&record),
        [finding(
            "semantic.reference-kind-mismatch",
            None,
            TRANSITION_LINE
        )]
    );
    assert_model_unavailable(&record);

    // A process with `allowed_links.emits: [event]` and no construct entry
    // for `steps` refuses it too.
    let process = with(
        with(
            machine_request("", "", "Order"),
            "relationVocabulary",
            vocabulary("process", json!({ "emits": ["event"] })),
        ),
        "constructReferences",
        json!({ "transitions": ["event-like"] }),
    );
    let record = extract_request(&process);
    assert_eq!(
        reference_findings(&record),
        [finding("semantic.reference-kind-mismatch", None, STEP_LINE)]
    );
    assert_model_unavailable(&record);

    // A module declaring neither source admits the entity.
    let neither = with(
        machine_request("Order", "", "Order"),
        "relationVocabulary",
        vocabulary("process", json!({})),
    );
    let record = extract_request(&neither);
    assert!(reference_findings(&record).is_empty(), "{record:#}");
    assert_eq!(
        record["model"]["transitions"][0]["emits"],
        "ix://agent-ix/shop/ENT-001"
    );

    // The control: every item resolves under the construct.
    let record = extract_request(&under_construct(machine_request(
        "OrderPlaced",
        "OrderShipped",
        "EV-001",
    )));
    assert!(reference_findings(&record).is_empty(), "{record:#}");
    assert_eq!(record["availability"]["model"]["state"], "available");
}

#[trace("TC-1995", "FR-075-AC-24")]
// Candidates are every distinct declaration matched by id, title or name;
// the admitted ones decide; `ix://` items are never matched by name.
#[test]
fn reference_candidates_and_ambiguity() {
    let in_bundle = |request: Value, bundle: Value| with(request, "bundle", bundle);

    // Two events titled `OrderPlaced`.
    let record = extract_request(&in_bundle(
        machine_request("OrderPlaced", "", ""),
        bundle_of(&[
            ("EV-001", Some("event"), &["OrderPlaced"]),
            ("EV-005", Some("event"), &["OrderPlaced"]),
        ]),
    ));
    assert_eq!(
        reference_findings(&record),
        [finding(
            "semantic.ambiguous-reference",
            None,
            TRANSITION_LINE
        )]
    );
    let message = record["diagnostics"][0]["message"].as_str().unwrap();
    assert!(
        message.contains("EV-001") && message.contains("EV-005"),
        "{message}"
    );

    // An event and an entity both titled `OrderPlaced`: the admitted event.
    let record = extract_request(&in_bundle(
        under_construct(machine_request("OrderPlaced", "", "")),
        bundle_of(&[
            ("EV-001", Some("event"), &["OrderPlaced"]),
            ("ENT-001", Some("entity"), &["OrderPlaced"]),
        ]),
    ));
    assert!(reference_findings(&record).is_empty(), "{record:#}");
    assert_eq!(record["model"]["transitions"][0]["emits"], EV1);

    // One event authored in two forms under one id.
    let mut two_forms = bundle_of(&[
        ("EV-001", Some("event"), &["OrderPlaced"]),
        ("EV-001", Some("event"), &["OrderPlaced", "Order Placed"]),
    ]);
    two_forms["package"] = json!("agent-ix/shop");
    let record = extract_request(&in_bundle(
        machine_request("OrderPlaced", "", ""),
        two_forms,
    ));
    assert!(reference_findings(&record).is_empty(), "{record:#}");
    assert_eq!(record["model"]["transitions"][0]["emits"], EV1);

    // An event with id `OrderPlaced` and another titled `OrderPlaced`.
    let record = extract_request(&in_bundle(
        machine_request("OrderPlaced", "", ""),
        bundle_of(&[
            ("OrderPlaced", Some("event"), &[]),
            ("EV-001", Some("event"), &["OrderPlaced"]),
        ]),
    ));
    assert_eq!(
        reference_findings(&record),
        [finding(
            "semantic.ambiguous-reference",
            None,
            TRANSITION_LINE
        )]
    );

    // The id of an entity and the title of an event: the admitted event.
    let record = extract_request(&in_bundle(
        under_construct(machine_request("Shipment", "", "")),
        bundle_of(&[
            ("Shipment", Some("entity"), &[]),
            ("EV-007", Some("event"), &["Shipment"]),
        ]),
    ));
    assert!(reference_findings(&record).is_empty(), "{record:#}");
    assert_eq!(
        record["model"]["transitions"][0]["emits"],
        "ix://agent-ix/shop/EV-007"
    );

    // `ix://` resolves as an FR-076 Target does, never by name.
    let record = extract_request(&machine_request("ix://agent-ix/shop/EV-001", "", ""));
    assert!(reference_findings(&record).is_empty(), "{record:#}");
    assert_eq!(record["model"]["transitions"][0]["emits"], EV1);
    let record = extract_request(&machine_request("ix://agent-ix/shop/OrderPlaced", "", ""));
    assert_eq!(
        reference_findings(&record),
        [finding(
            "semantic.unknown-reference",
            Some("unknown-name"),
            TRANSITION_LINE
        )]
    );

    // The extracted artifact by its own title: resolved when admitted.
    let record = extract_request(&machine_request("OrderLifecycle", "", ""));
    assert!(reference_findings(&record).is_empty(), "{record:#}");
    assert_eq!(
        record["model"]["transitions"][0]["emits"],
        "ix://agent-ix/shop/SM-001"
    );
    let record = extract_request(&under_construct(machine_request("OrderLifecycle", "", "")));
    assert_eq!(
        reference_findings(&record),
        [finding(
            "semantic.reference-kind-mismatch",
            None,
            TRANSITION_LINE
        )]
    );
}

#[trace("TC-1996", "FR-075-AC-25")]
// Cell-shape refusals, imported identities, untyped declarations, and the
// construct entry deciding over allowed_links.
#[test]
fn reference_cell_rules() {
    let record = extract_request(&machine_request("", "OrderPlaced, EV-001", ""));
    assert_eq!(
        reference_findings(&record),
        [finding(
            "semantic.duplicate-model-entry",
            Some("repeated-reference"),
            STEP_LINE
        )]
    );
    assert_model_unavailable(&record);

    for cell in ["OrderPlaced,,EV-002", "OrderPlaced,"] {
        let record = extract_request(&machine_request("", cell, ""));
        assert_eq!(
            reference_findings(&record),
            [finding(
                "semantic.invalid-model-cell",
                Some("empty-item"),
                STEP_LINE
            )],
            "{cell}"
        );
        assert_model_unavailable(&record);
    }

    let record = extract_request(&machine_request("OrderPlaced, OrderShipped", "", ""));
    assert_eq!(
        reference_findings(&record),
        [finding(
            "semantic.invalid-model-cell",
            Some("too-many-references"),
            TRANSITION_LINE
        )]
    );
    assert_model_unavailable(&record);

    // An identity in an imported package resolves unchecked.
    let mut imported = under_construct(machine_request("ix://agent-ix/billing/INV-1", "", ""));
    imported["module"]["imports"] = json!({ "agent-ix/billing": "*" });
    let record = extract_request(&imported);
    assert!(reference_findings(&record).is_empty(), "{record:#}");
    assert_eq!(
        record["model"]["transitions"][0]["emits"],
        "ix://agent-ix/billing/INV-1"
    );

    // A declaration with no object type: admitted by an unrestricted cell,
    // refused under `allowed_links.emits: [event]`.
    let untyped = bundle_of(&[("LOOSE-1", None, &["Loose"])]);
    let record = extract_request(&with(
        machine_request("", "", "Loose"),
        "bundle",
        untyped.clone(),
    ));
    assert!(reference_findings(&record).is_empty(), "{record:#}");
    assert_eq!(
        record["model"]["steps"][0]["emits"],
        json!(["ix://agent-ix/shop/LOOSE-1"])
    );
    let record = extract_request(&with(
        with(machine_request("", "", "Loose"), "bundle", untyped),
        "relationVocabulary",
        vocabulary("process", json!({ "emits": ["event"] })),
    ));
    assert_eq!(
        reference_findings(&record),
        [finding("semantic.reference-kind-mismatch", None, STEP_LINE)]
    );

    // A construct entry and an allowed_links entry: the construct decides.
    let both = |emits: &str| {
        with(
            with(
                machine_request(emits, "", ""),
                "relationVocabulary",
                vocabulary("state_machine", json!({ "emits": ["entity"] })),
            ),
            "constructReferences",
            json!({ "transitions": ["event-like"] }),
        )
    };
    let record = extract_request(&both("OrderPlaced"));
    assert!(reference_findings(&record).is_empty(), "{record:#}");
    let record = extract_request(&both("Order"));
    assert_eq!(
        reference_findings(&record),
        [finding(
            "semantic.reference-kind-mismatch",
            None,
            TRANSITION_LINE
        )]
    );
}

const MACHINE_MODULE: &str = "manifest_version: 1.0.0\nname: machines\nobject_types:\n- name: state_machine\n  data_schema:\n    type: object\n  body_extraction:\n    yield_pattern:\n      match:\n        states: { from: table_row, under_section: States, assert: { columns: [State, Description] } }\n        transitions: { from: table_row, under_section: Transitions, assert: { columns: [From, To, Trigger, Guard, Emits], optional_columns: [Guard, Emits] } }\n        steps: { from: table_row, under_section: Steps, assert: { columns: [Step, Kind, Consumes, Emits, Description] } }\nsemantic:\n  contract_version: 1.0.0\n  package: agent-ix/machines\n  exports: [state_machine]\n  targets: [json-schema, markdown]\n";

#[trace("TC-1997", "FR-075-AC-26")]
// With no bundle index or no bundle package the rows advise, the model is
// unavailable with that reason and no error, and validate_document passes.
#[test]
fn unresolvable_bundle_advises_and_keeps_validating() {
    let unindexed = with(
        machine_request("OrderPlaced", "OrderPlaced", ""),
        "bundle",
        json!({
            "package": "agent-ix/shop",
            "objects": [{ "id": "EV-001", "names": ["OrderPlaced"] }],
            "artifacts": []
        }),
    );
    let record = extract_request(&unindexed);
    assert_eq!(
        reference_findings(&record),
        [
            finding(
                "semantic.unresolved-target",
                Some("no-bundle-index"),
                TRANSITION_LINE
            ),
            finding(
                "semantic.unresolved-target",
                Some("no-bundle-index"),
                STEP_LINE
            ),
        ]
    );
    assert!(record["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .all(|d| d["severity"] != "error"));
    assert_eq!(
        record["availability"]["model"],
        json!({ "state": "unavailable", "reason": "no-bundle-index", "lossy": false })
    );
    assert!(record.get("model").is_none(), "{record:#}");

    let registry =
        quire_rs::Registry::from_inline_parts(MACHINE_MODULE.as_bytes(), &Default::default())
            .unwrap();
    assert!(registry.failures().is_empty(), "{:?}", registry.failures());
    let machine_type = registry.archetype("state_machine").unwrap();
    let doc = machine("OrderPlaced", "OrderPlaced", "");
    for result in [
        quire_rs::validate_document_in_bundle(&registry, machine_type, &doc, "agent-ix/shop", None),
        quire_rs::validate_document_in_registry(&registry, machine_type, &doc),
    ] {
        assert!(result.is_valid, "{:?}", result.errors);
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.message.starts_with("semantic.unresolved-target")),
            "{:?}",
            result.warnings
        );
    }

    // A bundle with no package and no source identity.
    let mut unpackaged = machine_request("OrderPlaced", "", "");
    unpackaged["bundle"]["package"] = json!("");
    unpackaged.as_object_mut().unwrap().remove("sourceIdentity");
    let record = extract_request(&unpackaged);
    assert_eq!(
        reference_findings(&record),
        [finding(
            "semantic.unresolved-target",
            Some("no-bundle-package"),
            TRANSITION_LINE
        )]
    );
    assert_eq!(
        record["availability"]["model"],
        json!({ "state": "unavailable", "reason": "no-bundle-package", "lossy": false })
    );
    assert!(record.get("model").is_none(), "{record:#}");

    // A bundle listing only the extracted artifact is indexed.
    let record = extract_request(&with(
        machine_request("OrderPlaced", "", ""),
        "bundle",
        bundle_of(&[("SM-001", Some("state_machine"), &["OrderLifecycle"])]),
    ));
    assert_eq!(
        reference_findings(&record),
        [finding(
            "semantic.unknown-reference",
            Some("unknown-name"),
            TRANSITION_LINE
        )]
    );
}

/// FR-075-AC-19: a manifest declaring none of the dimension, terms and unit
/// locators refuses each table with the feature as the `reason`, and an
/// artifact with none of the tables carries no such member.
#[trace("TC-1950", "FR-075-AC-19")]
#[test]
fn undeclared_dimension_tables_are_refused_and_absent_ones_leave_no_member() {
    let md = "---\nid: FR-100\ntitle: Order\nobject: entity\n---\n# FR-100: Order\n\n\
## Basis\n\n| Base |\n|---|\n| true |\n\n\
## Powers\n\n| Dimension | Exponent |\n|---|---|\n| Length | 1 |\n\n\
## Scales\n\n| Dimension | Target | Scale | Offset |\n|---|---|---|---|\n| Time | s | 1 | 0 |\n";
    let record = extract(md, &[], dsl(json!({})));
    for (reason, section, header) in [
        ("dimension", "Basis", "| Base |"),
        ("terms", "Powers", "| Dimension | Exponent |"),
        ("unit", "Scales", "| Dimension | Target |"),
    ] {
        let d = diagnostic(&record, "semantic.feature-not-extractable", reason);
        assert_eq!(d["line"], line_of(md, header), "{record:#}");
        assert_eq!(d["section"], section, "{record:#}");
        assert_eq!(d["sourceSpan"]["startLine"], d["line"], "{record:#}");
    }
    assert_model_unavailable(&record);

    let plain = "---\nid: FR-100\ntitle: Order\nobject: entity\n---\n# FR-100: Order\n\nProse.\n";
    let record = extract(plain, &[], dsl(json!({})));
    let undeclared = extract(plain, &[], json!(null));
    assert_eq!(
        serde_json::to_string(&record).unwrap(),
        serde_json::to_string(&undeclared).unwrap(),
        "an artifact with none of the tables is the FR-072 record byte for byte"
    );
    assert!(record.get("model").is_none(), "{record:#}");
    for member in ["dimension", "terms", "unit"] {
        assert!(record.get(member).is_none(), "{member}: {record:#}");
    }
    assert!(
        codes(&record)
            .iter()
            .all(|(c, _)| c != "semantic.feature-not-extractable"),
        "{record:#}"
    );
}

/// FR-070-AC-11: a `Clock` field validates against semantic-v1 and carries
/// the target `Clock`.
#[trace("TC-1945", "FR-070-AC-11")]
#[test]
fn clock_field_record_validates_against_semantic_v1() {
    let md = "---\nid: FR-100\ntitle: Order\nobject: entity\n---\n# FR-100: Order\n\n\
## Properties\n\n| Field | Type | Multiplicity | Constraints |\n|---|---|---|---|\n| c | Clock | 1 | |\n";
    let record = extract(md, &[], dsl(json!({})));
    assert_eq!(
        record["fields"][0]["type"]["target"],
        json!({ "kind": "SemanticId", "value": "ix://quire/native/Clock" }),
        "{record:#}"
    );
    assert_eq!(record["availability"]["fields"]["state"], "available");
}

/// A declared `dimension`, `terms` or `unit` locator is refused visibly
/// until its reader exists: the table never vanishes without a diagnostic.
#[trace("TC-1950", "FR-075-AC-19")]
#[test]
fn declared_dimension_tables_without_a_reader_are_refused() {
    let md = "---\nid: FR-100\ntitle: Order\nobject: entity\n---\n# FR-100: Order\n\n\
## Basis\n\n| Base |\n|---|\n| true |\n";
    let record = extract(
        md,
        &[],
        dsl(json!({ "dimension": row("Basis", &["Base"], &[]) })),
    );
    let d = diagnostic(&record, "semantic.feature-not-extractable", "dimension");
    assert_eq!(d["line"], line_of(md, "| Base |"), "{record:#}");
    assert_eq!(d["section"], "Basis", "{record:#}");
    assert_model_unavailable(&record);
}
