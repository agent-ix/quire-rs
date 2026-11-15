//! Selected record keys on the relation report (FR-058-AC-36/37,
//! FR-084-AC-20..28).
//!
//! Every kind, verb and key is authored in the fixture module
//! (`tests/fixtures/traceability/record-keys`); the engine knows none.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use ix_trace_rs::trace;
use quire_rs::corpus::{relation_report, RelationReportMember, Spec};
use quire_rs::Registry;
use serde_json::{json, Value};
use tempfile::TempDir;

fn fixture() -> Registry {
    let registry = Registry::load_module(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/traceability/record-keys"),
    )
    .expect("fixture module loads");
    assert!(registry.failures().is_empty(), "{:?}", registry.failures());
    registry
}

fn write(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    fs::write(path, body).expect("write");
}

fn pointing(id: &str, ty: &str, record: &str, targets: &[&str]) -> String {
    let mut rels = String::from("relationships:\n");
    for target in targets {
        rels.push_str(&format!(
            "  - target: \"{target}\"\n    type: \"classified-as\"\n"
        ));
    }
    format!("---\nid: {id}\ntype: {ty}\ntitle: {id}\n{record}{rels}---\n\n# {id}\n")
}

fn connection(id: &str, targets: &[&str]) -> String {
    pointing(id, "Connection", "", targets)
}

fn classification(id: &str, record: &str) -> String {
    format!("---\nid: {id}\ntype: Classification\ntitle: {id}\n{record}---\n\n# {id}\n")
}

/// The corpus: CLS-001 states `cats` and `lvl`, CLS-002 only `lvl`, CLS-003 an
/// out-of-enum `cats`, CLS-004 is pointed at by nothing, CLS-005 authors the
/// mapped key and the literal one, CLS-006 only the mapped key, CLS-007 two
/// keys stated out of lexical order.
fn corpus() -> TempDir {
    let dir = TempDir::new().expect("tempdir");
    let docs = [
        (
            "CONN-001",
            connection("CONN-001", &["CLS-001", "CLS-002", "CLS-003"]),
        ),
        ("CONN-002", connection("CONN-002", &["CLS-005", "CLS-006"])),
        ("CONN-003", connection("CONN-003", &["CLS-007"])),
        (
            "LINK-001",
            pointing("LINK-001", "Link", "cats: [b]\n", &["CLS-001", "CLS-404"]),
        ),
        (
            "CLS-001",
            classification("CLS-001", "cats: [a, b]\nlvl: hi\n"),
        ),
        ("CLS-002", classification("CLS-002", "lvl: lo\n")),
        ("CLS-003", classification("CLS-003", "cats: [x]\n")),
        ("CLS-004", classification("CLS-004", "cats: [c]\n")),
        ("CLS-005", classification("CLS-005", "raw_tag: t\ntag: t\n")),
        ("CLS-006", classification("CLS-006", "raw_tag: q\n")),
        (
            "CLS-007",
            classification("CLS-007", "zeta: z\nalpha: a\nlvl: hi\n"),
        ),
    ];
    for (id, text) in docs {
        write(dir.path(), &format!("spec/{id}.md"), &text);
    }
    dir
}

fn report(dir: &TempDir, registry: &Registry) -> Value {
    let spec = Spec::from_path(&dir.path().join("spec"));
    let report = relation_report(
        &spec,
        registry,
        &[RelationReportMember {
            root: dir.path(),
            matrix: &[],
        }],
        &[],
    )
    .expect("report");
    serde_json::to_value(report).expect("serializes")
}

fn section<'a>(report: &'a Value, name: &str) -> &'a Value {
    report["relations"]
        .as_array()
        .expect("relations")
        .iter()
        .find(|s| s["name"] == name)
        .unwrap_or_else(|| panic!("no section {name}"))
}

fn row<'a>(section: &'a Value, id: &str) -> &'a Value {
    section["rows"]
        .as_array()
        .expect("rows")
        .iter()
        .find(|r| r["document"]["id"] == id)
        .unwrap_or_else(|| panic!("no row {id}"))
}

fn linked<'a>(row: &'a Value, id: &str) -> &'a Value {
    row["linked"]
        .as_array()
        .expect("linked")
        .iter()
        .find(|l| l["id"] == id)
        .unwrap_or_else(|| panic!("no linked {id}"))
}

#[trace("FR-084-AC-20", "FR-084-AC-21")]
#[test]
fn tc_record_is_carried_in_authored_order_and_absent_when_unstated() {
    let report = report(&corpus(), &fixture());
    let outgoing = row(section(&report, "conn-classified"), "CONN-001");

    assert_eq!(
        linked(outgoing, "CLS-001")["record"],
        json!({"cats": ["a", "b"]})
    );
    // The control: a classification stating no selected key has no `record`
    // key at all, neither `{}` nor `"cats": []`.
    let unstated = linked(outgoing, "CLS-002");
    assert!(unstated.get("record").is_none(), "{unstated}");
}

#[trace("FR-084-AC-22")]
#[test]
fn tc_unselected_keys_appear_nowhere() {
    let report = report(&corpus(), &fixture());
    let text = serde_json::to_string(section(&report, "conn-classified")).unwrap();
    assert!(text.contains("\"cats\""), "{text}");
    assert!(!text.contains("lvl"), "{text}");
    assert!(!text.contains("\"hi\""), "{text}");
    let text = serde_json::to_string(section(&report, "cls-used")).unwrap();
    assert!(!text.contains("lvl"), "{text}");
}

#[trace("FR-084-AC-24")]
#[test]
fn tc_record_is_omitted_without_schema_when_invalid_or_dangling() {
    let report = report(&corpus(), &fixture());
    let outgoing = row(section(&report, "conn-classified"), "CONN-001");
    // No resolved data_schema: the obliged connection carries no record.
    assert!(outgoing["document"].get("record").is_none());
    // Enum violation.
    assert!(linked(outgoing, "CLS-003").get("record").is_none());
    // Duplicate record key: `tag` is filled by `raw_tag` and authored literally.
    let tag = row(section(&report, "conn-classified-tag"), "CONN-002");
    assert!(linked(tag, "CLS-005").get("record").is_none());
    // The control: a valid document of the same relation keeps its keys.
    assert_eq!(linked(tag, "CLS-006")["record"], json!({"tag": "q"}));
    // A dangling entry has no document and carries no record.
    let any = row(section(&report, "link-classified-any"), "LINK-001");
    // The control: the resolved entry of the same row keeps its keys.
    assert_eq!(
        linked(any, "CLS-001")["record"],
        json!({"cats": ["a", "b"]})
    );
    assert_eq!(any["document"]["record"], json!({"cats": ["b"]}));
    let dangling = any["linked"]
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["resolution"] == "dangling")
        .expect("dangling entry");
    assert!(dangling.get("record").is_none(), "{dangling}");
}

#[trace("FR-084-AC-25")]
#[test]
fn tc_record_keys_are_ordered_by_name_and_reports_are_byte_identical() {
    let dir = corpus();
    let registry = fixture();
    let first = serde_json::to_string(&report(&dir, &registry)).unwrap();
    let second = serde_json::to_string(&report(&dir, &registry)).unwrap();
    assert_eq!(first, second);
    assert!(
        first.contains("\"record\":{\"alpha\":\"a\",\"zeta\":\"z\"}"),
        "{first}"
    );
}

#[trace("FR-084-AC-26")]
#[test]
fn tc_relation_without_record_keys_serializes_no_record() {
    let report = report(&corpus(), &fixture());
    let text = serde_json::to_string(section(&report, "conn-classified-plain")).unwrap();
    assert!(!text.contains("record"), "{text}");
}

#[trace("FR-084-AC-27")]
#[test]
fn tc_record_follows_criteria() {
    let report = report(&corpus(), &fixture());
    let outgoing = row(section(&report, "conn-classified"), "CONN-001");
    let text = serde_json::to_string(linked(outgoing, "CLS-001")).unwrap();
    let criteria = text.find("\"criteria\"").expect("criteria");
    let record = text.find("\"record\"").expect("record");
    assert!(criteria < record, "{text}");
}

#[trace("FR-084-AC-28")]
#[test]
fn tc_incoming_relation_is_the_converse_view() {
    let report = report(&corpus(), &fixture());
    let used = section(&report, "cls-used");
    let first = row(used, "CLS-001");
    assert_eq!(first["document"]["record"], json!({"cats": ["a", "b"]}));
    let ids: Vec<&str> = first["linked"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["CONN-001"]);
    // A classification no connection points at is an uncovered row.
    assert_eq!(row(used, "CLS-004")["state"], "uncovered");
    assert_eq!(row(used, "CLS-004")["linked"], json!([]));
    // Every pairing of the outgoing view appears as its converse here.
    let outgoing = section(&report, "conn-classified");
    for connection in outgoing["rows"].as_array().unwrap() {
        for entry in connection["linked"].as_array().unwrap() {
            let back = row(used, entry["id"].as_str().unwrap());
            assert!(
                back["linked"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|l| l["id"] == connection["document"]["id"]),
                "{entry}"
            );
        }
    }
}

#[trace("FR-058-AC-37")]
#[test]
fn tc_record_keys_do_not_change_which_documents_are_obliged_or_satisfied() {
    let report = report(&corpus(), &fixture());
    let states = |name: &str| -> Vec<(String, String)> {
        section(&report, name)["rows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| {
                (
                    r["document"]["id"].as_str().unwrap().to_string(),
                    r["state"].as_str().unwrap().to_string(),
                )
            })
            .collect()
    };
    assert_eq!(states("conn-classified"), states("conn-classified-plain"));
    assert!(!states("conn-classified").is_empty());
}

fn refusal(record_keys: &str, from: &str, to: &str) -> String {
    let manifest = format!(
        "name: rk\nversion: 0.1.0\nartifact_types:\n- name: Connection\n- name: Classification\n  data_schema:\n    schema: schemas/Classification.json\nsemantic:\n  contract_version: 1.0.0\n  package: agent-ix/rk\n  exports: [Classification]\n  targets: [json-schema, markdown]\ntraceability:\n  required_relations:\n  - name: r1\n    from: {from}\n    edges: [classified-as]\n    to: {to}\n    direction: outgoing\n    check: c1\n    record_keys: {record_keys}\n"
    );
    let schema = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/traceability/record-keys/schemas/Classification.json"),
    )
    .expect("schema");
    let schemas = BTreeMap::from([("schemas/Classification.json".to_string(), schema)]);
    match Registry::from_inline_parts(manifest.as_bytes(), &schemas) {
        Err(error) => error.to_string(),
        Ok(registry) if registry.failures().is_empty() => String::new(),
        Ok(registry) => format!("{:?}", registry.failures()),
    }
}

#[trace("FR-058-AC-36")]
#[test]
fn tc_record_keys_are_refused_at_load_unless_an_endpoint_declares_them() {
    for (record_keys, why) in [
        ("[]", "empty"),
        ("cats", "not a list"),
        ("null", "null"),
        ("[\" \"]", "blank"),
        ("[cats, cats]", "repeated"),
    ] {
        let message = refusal(record_keys, "Connection", "[Classification]");
        assert!(message.contains("r1"), "{why}: {message}");
        assert!(message.contains("record_keys"), "{why}: {message}");
    }
    let message = refusal("[nope]", "Connection", "[Classification]");
    assert!(
        message.contains("r1") && message.contains("nope"),
        "{message}"
    );

    // Declared by only one endpoint archetype: accepted.
    assert_eq!(refusal("[cats]", "Connection", "[Classification]"), "");
    // `to: []` leaves `from` the only endpoint, so a key only the counterpart
    // declares is refused ...
    let message = refusal("[cats]", "Connection", "[]");
    assert!(
        message.contains("r1") && message.contains("cats"),
        "{message}"
    );
    // ... and the same key is accepted once `from` declares it.
    assert_eq!(refusal("[cats]", "Classification", "[]"), "");
}
