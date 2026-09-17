//! FR-075 model feature extraction (TC-1840..TC-1847).

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
            "semanticCore": "0.2.0",
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
        "population": row("Members", &["Type", "Extent"], &[]),
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

const POPULATION: &str = "---\nid: POP-001\ntitle: Shop Population\nobject: population\n---\n# POP-001: Shop Population\n\n## Members\n\n| Type | Extent |\n|---|---|\n| Order | 0..* |\n| Customer | 1..* |\n";

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
            &json!(["OrderShipped"])
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
            .find(|d| d["code"] == "semantic.feature-not-extractable" && d["reason"] == feature)
            .unwrap_or_else(|| panic!("{feature}: {record:#}"));
        assert_eq!(refusal["line"], line, "{feature}");
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
