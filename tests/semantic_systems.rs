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

/// A `table_row` locator under `section` asserting `columns`.
fn locator(section: &str, columns: &[&str]) -> Value {
    json!({
        "from": "table_row",
        "under_section": section,
        "required": true,
        "assert": { "columns": columns, "min_rows": 1 },
    })
}

/// The systems locator of `kind` as spec-objects-architecture declares it.
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
    json!({ "yield_pattern": { "match": { kind: locator(section, columns) } } })
}

/// Extract `md` under `body_extraction`, against a bundle holding
/// `artifacts` as `(id, object)`; the record must validate against
/// semantic-v1.
fn extract(md: &str, body_extraction: Value, artifacts: &[(&str, &str)]) -> Value {
    let artifacts: Vec<Value> = artifacts
        .iter()
        .map(|(id, object)| json!({ "id": id, "object": object }))
        .collect();
    let request = json!({
        "markdown": md,
        "module": {
            "contractVersion": "1.0.0",
            "semanticCore": "0.1.0",
            "package": "agent-ix/spec-objects-architecture",
            "exports": ["part", "port", "connection", "allocation"],
            "imports": { "agent-ix/fleet": "*" },
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

fn advisories(record: &Value) -> usize {
    record["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["code"] == "semantic.unresolved-target")
        .count()
}

fn doc(id: &str, kind: &str, section: &str, table: &str) -> String {
    format!("---\nid: {id}\ntitle: \"{id}\"\ntype: {kind}\nobject: {kind}\n---\n# [{id}] {id}\n\n## {section}\n\n{table}")
}

const PART: &str = "| Owner | Declared Type | Multiplicity |\n|---|---|---|\n| search_service | QuantCodec | 0..* |\n";
const PORT: &str = "| Owner | Direction | Interface | Multiplicity |\n|---|---|---|---|\n| scoring_engine | inout | QuantCodec | 1..1 |\n";
const CONNECTION: &str = "| Source | Source Multiplicity | Target | Target Multiplicity | Direction |\n|---|---|---|---|---|\n| planner_out | 1..1 | score_in | | bidirectional |\n";
const ALLOCATION: &str =
    "| Source | Target |\n|---|---|\n| quant_codec/score_ip_batch | scoring_engine |\n";

const BUNDLE: &[(&str, &str)] = &[
    ("search_service", "part"),
    ("scoring_engine", "part"),
    ("planner_out", "port"),
    ("score_in", "port"),
    ("quant_codec", "interface"),
];

const PKG: &str = "ix://agent-ix/shop";

#[trace("TC-1872", "FR-075-AC-12")]
#[test]
fn systems_tables_lower_to_record_keys_and_refuse_bad_rows() {
    let pkg = PKG;

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
    let record = extract(&md, body_extraction("part"), &[("scoring_engine", "part")]);
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
    // ... but a row that fails carries no advisory.
    let bad = doc("engine", "part", "Part", &PART.replace("0..*", "lots"));
    let record = extract(&bad, body_extraction("part"), &[]);
    assert!(record.get("model").is_none(), "{record:#}");
    assert_eq!(advisories(&record), 0, "{record:#}");

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

    // A second row, and a table with no row at all.
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
    let md = doc(
        "alloc",
        "allocation",
        "Allocation",
        "| Source | Target |\n|---|---|\n",
    );
    let record = extract(&md, body_extraction("allocation"), BUNDLE);
    assert!(
        refused(
            &record,
            "semantic.invalid-model-cell",
            line_of(&md, "| Source | Target")
        ),
        "{record:#}"
    );

    // `<id>/<member>` names an allocation source only: not an owner, not a
    // connection end.
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
    for end in ["planner_out", "score_in"] {
        let md = doc(
            "wire",
            "connection",
            "Connection",
            &CONNECTION.replace(end, &format!("{end}/flow")),
        );
        let record = extract(&md, body_extraction("connection"), BUNDLE);
        assert!(
            refused(
                &record,
                "semantic.invalid-model-cell",
                line_of(&md, "/flow")
            ),
            "{end}: {record:#}"
        );
    }

    // An id outside the object-id alphabet (a hyphen) is malformed.
    let md = doc(
        "engine",
        "part",
        "Part",
        &PART.replace("search_service", "search-service"),
    );
    let bundle = [("search-service", "part")];
    let record = extract(&md, body_extraction("part"), &bundle);
    assert!(
        refused(
            &record,
            "semantic.invalid-model-cell",
            line_of(&md, "search-service")
        ),
        "{record:#}"
    );

    // Multiplicity columns the locator omits: the ends state none.
    let locator = json!({ "yield_pattern": { "match": {
        "connection": locator("Connection", &["Source", "Target", "Direction"]),
    } } });
    let md = doc(
        "wire",
        "connection",
        "Connection",
        "| Source | Target | Direction |\n|---|---|---|\n| planner_out | score_in | source-to-target |\n",
    );
    let record = extract(&md, locator, BUNDLE);
    assert_eq!(
        record["model"]["connection"]["sourceEnd"],
        json!({ "type": format!("{pkg}/planner_out") }),
        "{record:#}"
    );
    assert_eq!(
        record["model"]["connection"]["targetEnd"],
        json!({ "type": format!("{pkg}/score_in") })
    );

    // Undeclared: a `Source | Target` table is refused as `allocation`.
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

#[trace("TC-1872", "FR-075-AC-12")]
#[test]
fn systems_references_resolve_qualified_and_imported_names_of_the_admitted_kind() {
    let pkg = PKG;

    // An own-package `ix://` identity and an imported one resolve.
    let md = doc(
        "engine",
        "part",
        "Part",
        &PART.replace("search_service", &format!("{pkg}/search_service")),
    );
    let record = extract(&md, body_extraction("part"), BUNDLE);
    assert_eq!(
        record["model"]["part"]["owner"],
        format!("{pkg}/search_service"),
        "{record:#}"
    );
    let imported = "ix://agent-ix/fleet/dispatcher";
    let md = doc(
        "alloc",
        "allocation",
        "Allocation",
        &ALLOCATION.replace("scoring_engine", imported),
    );
    let record = extract(&md, body_extraction("allocation"), BUNDLE);
    assert_eq!(
        record["model"]["allocation"]["targetElement"], imported,
        "{record:#}"
    );
    // A qualified operation reference keeps its member.
    let md = doc(
        "alloc",
        "allocation",
        "Allocation",
        &ALLOCATION.replace("quant_codec/", &format!("{pkg}/quant_codec/")),
    );
    let record = extract(&md, body_extraction("allocation"), BUNDLE);
    assert_eq!(
        record["model"]["allocation"]["sourceElement"],
        format!("{pkg}/quant_codec/score_ip_batch"),
        "{record:#}"
    );
    // A package the module does not import names nothing.
    let md = doc(
        "alloc",
        "allocation",
        "Allocation",
        &ALLOCATION.replace("scoring_engine", "ix://agent-ix/elsewhere/dispatcher"),
    );
    let record = extract(&md, body_extraction("allocation"), BUNDLE);
    assert!(
        refused(
            &record,
            "semantic.unknown-reference",
            line_of(&md, "elsewhere")
        ),
        "{record:#}"
    );

    // Each cell names only its admitted object types. A Part `Owner` is not
    // in this list: FR-152 gives it no kind restriction (see
    // `part_owner_and_allocation_operation_source_admit_any_declared_kind`).
    let wrong_kind: [(&str, &str, &str, &str, &str); 6] = [
        ("port", "Port", PORT, "scoring_engine", "score_in"),
        (
            "connection",
            "Connection",
            CONNECTION,
            "planner_out",
            "search_service",
        ),
        (
            "connection",
            "Connection",
            CONNECTION,
            "score_in",
            "scoring_engine",
        ),
        (
            "allocation",
            "Allocation",
            ALLOCATION,
            "scoring_engine",
            "planner_out",
        ),
        (
            "allocation",
            "Allocation",
            ALLOCATION,
            "quant_codec/score_ip_batch",
            "quant_codec",
        ),
        // `<id>/<member>` names an interface operation; a port has none.
        (
            "allocation",
            "Allocation",
            ALLOCATION,
            "quant_codec/score_ip_batch",
            "score_in/flow",
        ),
    ];
    for (kind, section, table, from, to) in wrong_kind {
        let md = doc("subject", kind, section, &table.replace(from, to));
        let record = extract(&md, body_extraction(kind), BUNDLE);
        let row = md
            .split('\n')
            .position(|l| l.starts_with("| ") && l.contains(to))
            .unwrap() as u64
            + 1;
        assert!(
            refused(&record, "semantic.reference-kind-mismatch", row),
            "{kind} {to}: {record:#}"
        );
    }
    // A part cannot own itself.
    let md = doc("search_service", "part", "Part", PART);
    let record = extract(&md, body_extraction("part"), BUNDLE);
    assert!(
        refused(
            &record,
            "semantic.reference-kind-mismatch",
            line_of(&md, "| search_service |")
        ),
        "{record:#}"
    );
    // An allocation source may name a port.
    let md = doc(
        "alloc",
        "allocation",
        "Allocation",
        &ALLOCATION.replace("quant_codec/score_ip_batch", "planner_out"),
    );
    let record = extract(&md, body_extraction("allocation"), BUNDLE);
    assert_eq!(
        record["model"]["allocation"]["sourceElement"],
        format!("{pkg}/planner_out"),
        "{record:#}"
    );
}

/// FR-152/#461 (TC-197 Y01, Sys/Pump/sys_pump/pump_alloc): a Part `Owner`
/// names the owning composite type with no kind restriction, and an
/// allocation `Source` `<id>/<member>` admits any declaring kind but the
/// four systems-record kinds, which never declare operations. A Port
/// `Owner` and an allocation `Target` still admit only a `part`.
#[trace("TC-1872", "FR-075-AC-12")]
#[test]
fn part_owner_and_allocation_operation_source_admit_any_declared_kind() {
    let pkg = PKG;
    let bundle: &[(&str, &str)] = &[("Sys", "entity"), ("Pump", "entity"), ("sys_pump", "part")];

    // An `entity` owner of a part lifts (TC-197 Y01: `sys_pump (owner:
    // Sys)`, `Sys` an object type).
    let md = doc(
        "sys_pump",
        "part",
        "Part",
        &PART.replace("search_service", "Sys"),
    );
    let record = extract(&md, body_extraction("part"), bundle);
    assert_eq!(
        record["model"]["part"]["owner"],
        format!("{pkg}/Sys"),
        "{record:#}"
    );

    // An `entity` operation lifts as an allocation source (TC-197 Y01:
    // `pump_alloc` sourced from `Pump/run`, `Pump` an entity with its own
    // Operations).
    let md = doc(
        "pump_alloc",
        "allocation",
        "Allocation",
        &ALLOCATION
            .replace("quant_codec/score_ip_batch", "Pump/run")
            .replace("scoring_engine", "sys_pump"),
    );
    let record = extract(&md, body_extraction("allocation"), bundle);
    assert_eq!(
        record["model"]["allocation"]["sourceElement"],
        format!("{pkg}/Pump/run"),
        "{record:#}"
    );

    // An allocation source `<id>/<member>` whose `<id>` is a systems-record
    // kind (a `part`, here) still refuses: a part never declares
    // operations, so the named operation is necessarily undeclared.
    let md = doc(
        "pump_alloc",
        "allocation",
        "Allocation",
        &ALLOCATION
            .replace("quant_codec/score_ip_batch", "sys_pump/run")
            .replace("scoring_engine", "sys_pump"),
    );
    let record = extract(&md, body_extraction("allocation"), bundle);
    assert!(
        refused(
            &record,
            "semantic.reference-kind-mismatch",
            line_of(&md, "sys_pump/run")
        ),
        "{record:#}"
    );

    // A Port `Owner` naming a non-part (an entity) still refuses.
    let md = doc(
        "pump_out",
        "port",
        "Port",
        &PORT.replace("scoring_engine", "Pump"),
    );
    let record = extract(&md, body_extraction("port"), bundle);
    assert!(
        refused(
            &record,
            "semantic.reference-kind-mismatch",
            line_of(&md, "| Pump")
        ),
        "{record:#}"
    );

    // An allocation `Target` naming a non-part (an entity) still refuses.
    let md = doc(
        "pump_alloc",
        "allocation",
        "Allocation",
        &ALLOCATION
            .replace("quant_codec/score_ip_batch", "sys_pump")
            .replace("scoring_engine", "Pump"),
    );
    let record = extract(&md, body_extraction("allocation"), bundle);
    assert!(
        refused(
            &record,
            "semantic.reference-kind-mismatch",
            line_of(&md, "| Pump")
        ),
        "{record:#}"
    );
}

#[trace("TC-1872", "FR-075-AC-12")]
#[test]
fn systems_table_kind_comes_from_the_match_key() {
    // A connection declared with `[Source, Target]` reads a `Source | Target`
    // table as a connection, never as an allocation.
    let dsl = |key: &str| {
        json!({ "yield_pattern": { "match": {
            key: locator("Wiring", &["Source", "Target"]),
        } } })
    };
    let md = doc(
        "wire",
        "connection",
        "Wiring",
        "| Source | Target |\n|---|---|\n| planner_out | score_in |\n",
    );
    let record = extract(&md, dsl("connection"), BUNDLE);
    assert!(
        refused(
            &record,
            "semantic.invalid-model-cell",
            line_of(&md, "| planner_out")
        ),
        "{record:#}"
    );
    assert!(
        record["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["message"]
                .as_str()
                .is_some_and(|m| m.contains("connection direction"))),
        "{record:#}"
    );
    // The same locator under `allocation` reads it as an allocation.
    let md = md.replace(
        "| planner_out | score_in |",
        "| score_in | scoring_engine |",
    );
    let record = extract(&md, dsl("allocation"), BUNDLE);
    assert_eq!(
        record["model"]["allocation"]["targetElement"],
        format!("{PKG}/scoring_engine"),
        "{record:#}"
    );
    // A key that names no model table declares none.
    let record = extract(&md, dsl("wiring"), BUNDLE);
    assert!(
        record["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["code"] == "semantic.feature-not-extractable"),
        "{record:#}"
    );

    // One systems-model table per artifact.
    let both = json!({ "yield_pattern": { "match": {
        "part": locator("Part", &["Owner", "Declared Type", "Multiplicity"]),
        "port": locator("Port", &["Owner", "Direction", "Interface", "Multiplicity"]),
    } } });
    let md = format!("{}\n## Port\n\n{PORT}", doc("engine", "part", "Part", PART));
    let record = extract(&md, both, BUNDLE);
    assert!(
        refused(
            &record,
            "semantic.duplicate-section",
            line_of(&md, "| Owner | Direction")
        ),
        "{record:#}"
    );
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
        .declaration_record()
        .unwrap();
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

#[trace("TC-1872", "FR-075-AC-12")]
#[test]
fn a_declaration_with_two_systems_records_is_an_error() {
    let part = extract(
        &doc("p", "part", "Part", PART),
        body_extraction("part"),
        BUNDLE,
    );
    let port = extract(
        &doc("q", "port", "Port", PORT),
        body_extraction("port"),
        BUNDLE,
    );
    let mut both = part.clone();
    both["model"]["port"] = port["model"]["port"].clone();
    let typed: quire_rs::semantic::SemanticExtraction = serde_json::from_value(both).unwrap();
    assert!(typed.model.as_ref().unwrap().part.is_some());
    assert!(typed.model.as_ref().unwrap().port.is_some());
    let err = typed.declaration_record().unwrap_err();
    assert!(
        matches!(
            err,
            quire_rs::semantic::DeclarationError::MultipleSystemsRecords
        ),
        "{err}"
    );
    let single: quire_rs::semantic::SemanticExtraction = serde_json::from_value(part).unwrap();
    assert_eq!(
        single.declaration_record().unwrap()["owner"],
        format!("{PKG}/search_service")
    );
}
