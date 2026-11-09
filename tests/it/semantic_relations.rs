//! FR-076 relationships extraction (TC-1852..TC-1863, TC-1865, TC-1867). Oracles:
//! `relationships.md`, `relationships.expected.json`, and
//! `relationships-cases.json` from the `quire-fixtures` crate (PLAT-901).
//! The extraction context is built from each fixture's recorded `context`,
//! never from `module-ok`.

use std::collections::BTreeSet;

use ix_trace_rs::trace;
use jsonschema::Validator;
use quire_rs::semantic::python_entry::extract_semantic_json;
use quire_rs::semantic::{compile_module_schema, SEMANTIC_V1_SCHEMA};
use serde_json::{json, Value};

fn mapping_json(name: &str) -> Value {
    quire_fixtures::mapping_json(name)
}

fn mapping_text(name: &str) -> String {
    quire_fixtures::mapping_fixture(name)
}

/// The semantic-core version the fixtures run under: the embedded bundle's own.
fn declared_core() -> &'static str {
    quire_rs::semantic::embedded::embedded_semantic_core_version()
}

fn semantic_v1() -> Validator {
    compile_module_schema(
        &serde_json::from_str::<Value>(SEMANTIC_V1_SCHEMA).unwrap(),
        &|_| None,
        "https://agent-ix.github.io/quire-rs/schemas/output/",
    )
    .unwrap()
}

fn relation_decl_gate(core: &str) -> Validator {
    let schema = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://schemas.agent-ix.org/agent-ix/quire-rs/0.1.0/RelationDeclGate.json",
        "$ref": format!("https://schemas.agent-ix.org/semantic-core/{core}/RelationDecl.json")
    });
    compile_module_schema(
        &schema,
        &|_| None,
        "https://schemas.agent-ix.org/agent-ix/quire-rs/",
    )
    .unwrap()
}

/// The FR-076 request for `markdown` under a fixture's recorded `context`,
/// with the case's `mappings`, `withoutRelationVocabulary`, and
/// `withoutBundleIndex`, and `withoutBundlePackage` applied when given.
fn request(fixture: &Value, markdown: &str, path: &str, case: Option<&Value>) -> Value {
    let context = &fixture["context"];
    let flag = |key: &str| case.is_some_and(|c| c[key] == json!(true));
    let mappings = case.and_then(|c| c.get("mappings"));
    let bundle = &context["bundle"];
    let imports: serde_json::Map<String, Value> = bundle["imports"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| (p.as_str().unwrap().to_string(), json!("*")))
        .collect();
    // The fixture predates `BundleArtifact.operations` (FR-075 Inputs) and
    // is not edited in place (PLAT-901: it is quoin's own fixture, moved
    // not owned-for-editing), so this backfills the now-required field here
    // instead; none of these fixture cases exercise systems-model
    // operations, so every artifact declares none.
    let artifacts: Vec<Value> = bundle["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| {
            let mut a = a.as_object().unwrap().clone();
            a.insert("operations".to_string(), json!([]));
            Value::Object(a)
        })
        .collect();
    let mut request = json!({
        "markdown": markdown,
        "module": {
            "contractVersion": "1.0.0",
            "package": "agent-ix/spec-objects-business",
            "exports": ["entity"],
            "imports": imports,
            "mappings": mappings.unwrap_or(&context["mappings"]),
        },
        "path": path,
        "sourceIdentity": context["sourceIdentity"],
        "bundle": { "package": bundle["package"], "artifacts": artifacts },
        "relationVocabulary": {
            "objectType": context["objectType"],
            "edgeTypes": context["edgeTypes"],
            "roles": context["roles"],
            "allowedLinks": context["allowedLinks"],
        },
    });
    let request_map = request.as_object_mut().unwrap();
    if flag("withoutBundlePackage") {
        request_map["bundle"]["package"] = json!("");
        request_map.remove("sourceIdentity");
    }
    if flag("withoutRelationVocabulary") {
        request_map.remove("relationVocabulary");
    }
    if flag("withoutBundleIndex") {
        request_map["bundle"]
            .as_object_mut()
            .unwrap()
            .remove("artifacts");
    }
    request
}

/// Extract; every record validates against semantic-v1 and every relation
/// against the fixture's semantic-core `RelationDecl.json`.
fn extract(request: &Value, core: &str) -> Value {
    let record = serde_json::to_value(extract_semantic_json(request).unwrap()).unwrap();
    let errors: Vec<String> = semantic_v1()
        .iter_errors(&record)
        .map(|e| format!("{e} at {}", e.instance_path()))
        .collect();
    assert!(errors.is_empty(), "{errors:?}\n{record:#}");
    let gate = relation_decl_gate(core);
    for relation in record["relations"].as_array().into_iter().flatten() {
        assert!(gate.is_valid(relation), "{relation:#}");
    }
    record
}

fn cases() -> Value {
    mapping_json("relationships-cases.json")
}

fn case_markdown(file: &Value, case: &Value) -> String {
    match case["artifact"].as_str() {
        Some(artifact) => artifact.to_string(),
        None => format!(
            "{}{}",
            file["artifactHead"].as_str().unwrap(),
            case["relationships"].as_str().unwrap()
        ),
    }
}

/// Is `line` (1-based) of `md` the shape a fixture `locus` names?
fn locus_holds(md: &str, line: u64, locus: &str) -> bool {
    let lines: Vec<&str> = md.split('\n').collect();
    let at = |n: u64| lines.get(n as usize - 1).copied().unwrap_or("");
    let is_separator = |t: &str| t.starts_with('|') && t.contains("---");
    match locus {
        "header" | "second-header" => at(line).starts_with('|') && is_separator(at(line + 1)),
        "row" | "second-row" => at(line).starts_with('|') && !is_separator(at(line)),
        "list" => at(line).starts_with("- "),
        "heading" | "second-heading" => at(line) == "## Relationships",
        other => panic!("unknown locus {other:?}"),
    }
}

/// Run one `relationships-cases.json` case and assert every field it records.
fn assert_case(file: &Value, case: &Value) {
    let id = case["id"].as_str().unwrap();
    let md = case_markdown(file, case);
    let core = declared_core();
    let record = extract(&request(file, &md, "case.md", Some(case)), core);
    let actual = record["diagnostics"].as_array().unwrap();
    assert_eq!(
        actual.len() as u64,
        case["exactDiagnostics"].as_u64().unwrap(),
        "{id}: exactDiagnostics\n{actual:#?}"
    );
    let expected = case["diagnostics"].as_array().unwrap();
    assert_eq!(
        actual.len(),
        expected.len(),
        "{id}: diagnostics\n{actual:#?}"
    );
    let lines: Vec<&str> = md.split('\n').collect();
    for (got, want) in actual.iter().zip(expected) {
        for key in ["code", "severity", "line", "section", "reason"] {
            assert_eq!(got[key], want[key], "{id}: diagnostic {key}\n{got:#}");
        }
        let line = want["line"].as_u64().unwrap();
        let locus = want["locus"].as_str().unwrap();
        assert!(
            locus_holds(&md, line, locus),
            "{id}: line {line} is not a {locus}"
        );
        let span = &got["sourceSpan"];
        assert_eq!(span["startLine"], line, "{id}");
        assert_eq!(span["endLine"], line, "{id}");
        assert_eq!(span["startColumn"], 1, "{id}");
        assert_eq!(
            span["endColumn"].as_u64().unwrap(),
            lines[line as usize - 1].len() as u64 + 1,
            "{id}"
        );
        let message = got["message"].as_str().unwrap();
        for needle in ["case.md", "relationships"] {
            assert!(message.contains(needle), "{id}: {message}");
        }
        for needle in want["messageContains"].as_array().into_iter().flatten() {
            let needle = needle.as_str().unwrap();
            assert!(
                message.contains(needle),
                "{id}: {message:?} lacks {needle:?}"
            );
        }
    }
    if case["relations"].is_null() {
        assert!(record.get("relations").is_none(), "{id}: relations emitted");
        assert!(
            record.get("relationSources").is_none(),
            "{id}: sources emitted"
        );
    } else {
        assert_eq!(record["relations"], case["relations"], "{id}: relations");
        assert_eq!(
            record["relationSources"], case["relationSources"],
            "{id}: relationSources"
        );
    }
    match case["availability"].get("relations") {
        None => assert!(
            record["availability"].get("relations").is_none(),
            "{id}: availability.relations present"
        ),
        Some(want) => assert_reason_loci(
            id,
            &record["availability"]["relations"],
            want,
            case["diagnostics"].as_array().unwrap(),
        ),
    }
}

/// The lines an `entry-errors: lines <lines>` reason lists.
fn reason_lines(reason: &str) -> Vec<u64> {
    reason
        .strip_prefix("entry-errors: lines ")
        .unwrap_or_else(|| panic!("not an entry-errors reason: {reason:?}"))
        .split(", ")
        .map(|n| n.parse().unwrap())
        .collect()
}

/// Compare an availability to the expectation. An expected `reasonLoci` is
/// resolved to lines through the case's own diagnostics (each `locus` names
/// the `line` it sits on) and compared with the lines of the actual reason;
/// every other field compares verbatim.
fn assert_reason_loci(id: &str, got: &Value, want: &Value, diagnostics: &[Value]) {
    let Some(loci) = want.get("reasonLoci") else {
        assert_eq!(got, want, "{id}: availability.relations");
        return;
    };
    assert!(
        want.get("reason").is_none(),
        "{id}: reason beside reasonLoci"
    );
    let resolved: Vec<u64> = loci
        .as_array()
        .unwrap()
        .iter()
        .map(|locus| {
            diagnostics
                .iter()
                .find(|d| d["locus"] == *locus)
                .unwrap_or_else(|| panic!("{id}: no diagnostic at locus {locus}"))["line"]
                .as_u64()
                .unwrap()
        })
        .collect();
    assert_eq!(
        reason_lines(got["reason"].as_str().unwrap_or_default()),
        resolved,
        "{id}: availability.relations reason lines"
    );
    for key in ["state", "lossy"] {
        assert_eq!(got[key], want[key], "{id}: availability.relations {key}");
    }
}

/// Run the named cases; each must exist in the fixture.
fn run(ids: &[&str]) {
    let file = cases();
    for id in ids {
        let case = file["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["id"] == *id)
            .unwrap_or_else(|| panic!("no case {id}"));
        assert_case(&file, case);
    }
}

const VERB_CASES: &[&str] = &[
    "verb-unknown",
    "verb-inverse-label",
    "verb-not-allowed",
    "verb-specializes",
];
const TARGET_CASES: &[&str] = &[
    "target-title",
    "target-undeclared-id",
    "target-own-package-undeclared-id",
    "target-qualified-not-imported",
    "target-not-allowed-object-type",
    "target-not-allowed-behavioral",
    "target-qualified-import",
];
const MULTIPLICITY_CASES: &[&str] = &[
    "multiplicity-malformed",
    "multiplicity-inverted",
    "multiplicity-empty",
];
const NAME_CASES: &[&str] = &[
    "name-not-identifier",
    "duplicate-name",
    "declared-in-frontmatter",
];
const SHAPE_CASES: &[&str] = &[
    "column-missing",
    "column-extra",
    "bullet-list",
    "second-table",
    "second-section",
    "list-after-table",
];
const GATING_CASES: &[&str] = &[
    "mapping-not-declared",
    "subset-header-unowned-section",
    "preamble-table",
    "prose-without-mapping",
    "token-table-other-section",
    "token-preamble-table",
];
const NO_VOCABULARY_CASES: &[&str] = &[
    "no-relation-vocabulary",
    "no-relation-vocabulary-second-table",
];
const MIXED_ROWS_CASES: &[&str] = &["no-bundle-index-mixed-rows"];
const NO_BUNDLE_PACKAGE_CASES: &[&str] = &["no-bundle-package", "no-bundle-package-second-table"];
const NO_BUNDLE_INDEX_CASES: &[&str] = &[
    "no-bundle-index",
    "no-bundle-index-title-target",
    "no-bundle-index-own-package-identity",
    "no-bundle-index-bad-multiplicity",
];
const NO_BLOCK_CASES: &[&str] = &["prose-only-section"];
const AVAILABILITY_CASES: &[&str] = &[
    "good-and-bad-rows",
    "two-error-lines",
    "header-only",
    "section-absent",
    "first-failing-check-only",
];

#[trace("TC-1852", "FR-076-AC-1")]
// the golden `relationships.md` extracts to `relationships.expected.json`.
#[test]
fn golden_relationships() {
    let expected = mapping_json("relationships.expected.json");
    let md = mapping_text("relationships.md");
    let core = declared_core();
    let record = extract(&request(&expected, &md, "relationships.md", None), core);
    assert_eq!(record["diagnostics"], expected["diagnostics"]);
    assert_eq!(record["relations"], expected["relations"]);
    assert_eq!(record["relationSources"], expected["relationSources"]);
    assert_eq!(
        record["availability"]["relations"],
        expected["availability"]["relations"]
    );
}

#[trace("TC-1853", "FR-076-AC-2")]
#[test]
fn verb_cases() {
    run(VERB_CASES);
}

#[trace("TC-1854", "FR-076-AC-3")]
#[test]
fn target_cases() {
    run(TARGET_CASES);
}

#[trace("TC-1855", "FR-076-AC-4")]
#[test]
fn multiplicity_cases() {
    run(MULTIPLICITY_CASES);
}

#[trace("TC-1856", "FR-076-AC-5")]
#[test]
fn name_and_frontmatter_cases() {
    run(NAME_CASES);
}

#[trace("TC-1857", "FR-076-AC-6")]
#[test]
fn table_shape_cases() {
    run(SHAPE_CASES);
}

#[trace("TC-1858", "FR-076-AC-7")]
#[test]
fn mapping_gating_cases() {
    run(GATING_CASES);
}

#[trace("TC-1859", "FR-076-AC-8")]
// availability cases, and every fixture case is run by exactly one criterion.
#[test]
fn availability_cases_and_full_coverage() {
    run(AVAILABILITY_CASES);
    let groups = [
        VERB_CASES,
        TARGET_CASES,
        MULTIPLICITY_CASES,
        NAME_CASES,
        SHAPE_CASES,
        GATING_CASES,
        AVAILABILITY_CASES,
        NO_VOCABULARY_CASES,
        NO_BUNDLE_INDEX_CASES,
        NO_BLOCK_CASES,
        MIXED_ROWS_CASES,
        NO_BUNDLE_PACKAGE_CASES,
    ];
    let assigned: Vec<&str> = groups.iter().flat_map(|g| g.iter().copied()).collect();
    let unique: BTreeSet<&str> = assigned.iter().copied().collect();
    assert_eq!(assigned.len(), unique.len(), "a case is assigned twice");
    let file = cases();
    let in_fixture: BTreeSet<&str> = file["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["id"].as_str().unwrap())
        .collect();
    assert_eq!(in_fixture.len(), 44);
    assert_eq!(unique, in_fixture);
}

#[trace("TC-1861", "FR-076-AC-10")]
// no relation vocabulary: one heading advisory and `no-relation-vocabulary`,
// unless a shape error already makes the feature unavailable.
#[test]
fn no_relation_vocabulary_cases() {
    run(NO_VOCABULARY_CASES);
}

#[trace("TC-1862", "FR-076-AC-11")]
// no bundle index: a lowering row's unchecked target carries the
// `no-bundle-index` advisory; a title and a row error are still refused.
#[test]
fn no_bundle_index_cases() {
    run(NO_BUNDLE_INDEX_CASES);
}

#[trace("TC-1863", "FR-076-AC-12")]
// a section with prose and no block warns at the heading, not_applicable.
#[test]
fn no_block_cases() {
    run(NO_BLOCK_CASES);
}

#[trace("TC-1865", "FR-076-AC-14")]
// no bundle index and a row error: the lowering row's advisory is dropped;
// only the error remains.
#[test]
fn no_bundle_index_mixed_rows() {
    run(MIXED_ROWS_CASES);
}

#[trace("TC-1867", "FR-076-AC-15")]
// with no bundle package, a source identity qualifies targets; with neither,
// relationships are unavailable with `no-bundle-package`.
#[test]
fn bundle_package_from_source_identity_or_unavailable() {
    let file = cases();
    let core = declared_core();
    let md = format!(
        "{}| Name | Verb | Target | Multiplicity |\n|---|---|---|---|\n| overlay | references | FR-005 | 1..1 |\n",
        file["artifactHead"].as_str().unwrap()
    );
    let mut req = request(&file, &md, "case.md", None);
    req["bundle"]
        .as_object_mut()
        .unwrap()
        .insert("package".to_string(), json!(""));
    req["sourceIdentity"] = json!("ix://agent-ix/other-bundle/spec");
    let record = extract(&req, core);
    assert_eq!(
        record["relations"][0]["target"], "ix://agent-ix/other-bundle/FR-005",
        "{record:#}"
    );

    req.as_object_mut().unwrap().remove("sourceIdentity");
    let record = extract(&req, core);
    assert!(record.get("relations").is_none(), "{record:#}");
    assert_eq!(
        record["availability"]["relations"],
        json!({ "state": "unavailable", "reason": "no-bundle-package", "lossy": false })
    );
    let diagnostics = record["diagnostics"].as_array().unwrap();
    let advisories: Vec<&Value> = diagnostics
        .iter()
        .filter(|d| d["code"] == "semantic.relationships-no-bundle-package")
        .collect();
    assert_eq!(advisories.len(), 1, "{diagnostics:#?}");
    assert_eq!(advisories[0]["severity"], "advisory");
    assert_eq!(advisories[0]["reason"], "no-bundle-package");
    assert_eq!(advisories[0]["line"], 16);

    run(NO_BUNDLE_PACKAGE_CASES);
}

#[trace("TC-1860", "FR-076-AC-9")]
// no token and no relationship table: no relations keys; declaration_record
// carries relations; a renamed verb keeps its registry category and
// composition (CON-2).
#[test]
fn record_compatibility_and_registry_authority() {
    let file = cases();
    let core = declared_core();
    let prose = format!(
        "{}- `overlay`: references FR-005\n",
        file["artifactHead"].as_str().unwrap()
    );
    let record = extract(
        &request(
            &file,
            &prose,
            "case.md",
            Some(&json!({ "mappings": ["typed-table"] })),
        ),
        core,
    );
    assert!(record.get("relations").is_none());
    assert!(record.get("relationSources").is_none());
    assert!(record["availability"].get("relations").is_none());

    let expected = mapping_json("relationships.expected.json");
    let md = mapping_text("relationships.md");
    let typed = extract_semantic_json(&request(&expected, &md, "relationships.md", None)).unwrap();
    assert_eq!(
        typed.declaration_record().unwrap()["relations"],
        expected["relations"]
    );

    // `contains` renamed to `holds` with the same registry entry: only the
    // verb changes; category and composite still come from the entry.
    let mut renamed = request(
        &expected,
        &md.replace("| contains |", "| holds |"),
        "relationships.md",
        None,
    );
    let vocabulary = &mut renamed["relationVocabulary"];
    let rename = |map: &mut Value| {
        let map = map.as_object_mut().unwrap();
        let value = map.remove("contains").unwrap();
        map.insert("holds".to_string(), value);
    };
    rename(&mut vocabulary["edgeTypes"]);
    rename(&mut vocabulary["allowedLinks"]["entity"]);
    let record = extract(&renamed, core);
    let mut want = expected["relations"].clone();
    want[1]["verb"] = json!("holds");
    assert_eq!(record["relations"], want);
}

/// A data schema whose top-level `keys` are `SemanticId[]`, with `relations`
/// when `relations` is set.
fn typed_schema(keys: &[&str], relations: bool) -> Value {
    let core = format!(
        "https://schemas.agent-ix.org/semantic-core/{}/",
        declared_core()
    );
    let mut properties = serde_json::Map::new();
    for key in keys {
        properties.insert(
            key.to_string(),
            json!({ "type": "array", "items": { "$ref": format!("{core}SemanticId.json") } }),
        );
    }
    if relations {
        properties.insert(
            "relations".to_string(),
            json!({ "type": "array", "items": { "$ref": format!("{core}RelationDecl.json") } }),
        );
    }
    json!({ "type": "object", "properties": properties })
}

/// The schema of each typed object type: `process` and `repository` are
/// typed-only, `aggregate_root` declares `relations` and `emits`.
fn schema_of(object_type: &str) -> Value {
    match object_type {
        "process" => typed_schema(&["emits"], false),
        "repository" => typed_schema(&["persists"], false),
        "aggregate_root" => typed_schema(&["emits"], true),
        other => panic!("no schema for {other}"),
    }
}

const TYPED_PACKAGE: &str = "agent-ix/shop";

/// The FR-076 request for an artifact of `object_type` with `rows` under
/// `## Relationships` (the first row is line 12) and `frontmatter` lines
/// appended to its frontmatter, against an indexed shop bundle.
fn typed_request(object_type: &str, frontmatter: &str, rows: &str) -> Value {
    let md = format!(
        "---\nid: X-001\ntitle: Example\nobject: {object_type}\n{frontmatter}---\n# X-001: Example\n\n## Relationships\n\n| Name | Verb | Target | Multiplicity |\n|---|---|---|---|\n{rows}"
    );
    json!({
        "markdown": md,
        "module": {
            "contractVersion": "1.0.0",
            "package": "agent-ix/spec-objects-business",
            "exports": [object_type],
            "mappings": ["relationships"],
        },
        "path": "x.md",
        "sourceIdentity": "ix://agent-ix/shop/spec",
        "bundle": {
            "package": TYPED_PACKAGE,
            "objects": [{ "id": "event-001", "names": ["event-001", "OrderPlaced"] }],
            "artifacts": [
                { "id": "event-001", "object": "event", "operations": [] },
                { "id": "aggregate-root-001", "object": "aggregate_root", "operations": [] },
                { "id": "entity-001", "object": "entity", "operations": [] },
            ],
        },
        "relationVocabulary": {
            "objectType": object_type,
            "edgeTypes": {
                "emits": { "category": "dataflow", "inverse": "emitted_by" },
                "persists": { "category": "dependency" },
                "references": { "category": "traceability" },
                "triggers": { "category": "behavioral" },
            },
            "roles": {},
            "allowedLinks": {
                "aggregate_root": { "emits": ["event"], "references": ["aggregate_root"] },
                "process": { "emits": ["event"], "triggers": ["process"] },
                "repository": { "persists": ["aggregate_root", "entity"] },
            },
        },
        "dataSchema": schema_of(object_type),
    })
}

/// Extract an FR-076 request, schema-checked against semantic-v1.
fn typed_extract(request: &Value) -> Value {
    extract(request, declared_core())
}

fn identity(id: &str) -> String {
    format!("ix://{TYPED_PACKAGE}/{id}")
}

/// The `code`/`reason` of every diagnostic of `record`.
fn findings(record: &Value) -> Vec<(String, String)> {
    record["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| {
            (
                d["code"].as_str().unwrap().to_string(),
                d["reason"].as_str().unwrap_or_default().to_string(),
            )
        })
        .collect()
}

#[trace("TC-1999", "FR-076-AC-16", "FR-076-CON-3")]
// Rows whose verb is a typed relation key of the data schema lower into that
// key as SemanticIds; a typed-only type carries no `relations`.
#[test]
fn typed_relation_keys_lower_from_rows() {
    let record = typed_extract(&typed_request(
        "process",
        "",
        "| done | emits | event-001 | 0..1 |\n",
    ));
    assert_eq!(
        record["emits"],
        json!([identity("event-001")]),
        "{record:#}"
    );
    assert!(record.get("relations").is_none(), "{record:#}");
    assert_eq!(record["availability"]["relations"]["state"], "available");
    let sources = record["relationSources"].as_array().unwrap();
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0]["name"], "done");
    assert_eq!(sources[0]["verb"], "emits");
    assert_eq!(sources[0]["target"], "event-001");
    assert_eq!(sources[0]["multiplicity"], "0..1");
    assert_eq!(sources[0]["sourceSpan"]["startLine"], 12);

    let record = typed_extract(&typed_request(
        "repository",
        "",
        "| orders | persists | aggregate-root-001 | 0..* |\n",
    ));
    assert_eq!(
        record["persists"],
        json!([identity("aggregate-root-001")]),
        "{record:#}"
    );
    assert!(record.get("relations").is_none());

    let mut two_events = typed_request(
        "aggregate_root",
        "",
        "| shipped | emits | event-002 | 0..* |\n| placed | emits | ix://agent-ix/shop/event-001 | 0..* |\n",
    );
    two_events["bundle"]["artifacts"]
        .as_array_mut()
        .unwrap()
        .push(json!({ "id": "event-002", "object": "event", "operations": [] }));
    let record = typed_extract(&two_events);
    assert_eq!(
        record["emits"],
        json!([identity("event-002"), identity("event-001")]),
        "{record:#}"
    );
    for value in record["emits"].as_array().unwrap() {
        assert!(value.as_str().unwrap().starts_with("ix://"), "{value}");
    }
}

#[trace("TC-2000", "FR-076-AC-17")]
// A typed row is refused for exactly the FR-076 reasons, plus a verb that is
// no typed key on a typed-only type; an error leaves no typed key.
#[test]
fn typed_rows_keep_the_relationship_refusals() {
    for (row, reason) in [
        ("| x | emits | nope | 1 |\n", "target-not-id"),
        ("| x | emits | OrderPlaced | 1 |\n", "target-not-id"),
        ("| x | emits | entity-001 | 1 |\n", "target-not-allowed"),
        ("| x | triggers | event-001 | 1 |\n", "verb-not-typed-key"),
        ("| x | emitted_by | event-001 | 1 |\n", "inverse-verb"),
    ] {
        let record = typed_extract(&typed_request("process", "", row));
        assert_eq!(
            findings(&record),
            [(
                "semantic.invalid-model-cell".to_string(),
                reason.to_string()
            )],
            "{row}"
        );
        assert_eq!(record["availability"]["relations"]["state"], "unavailable");
        for key in ["emits", "relations", "relationSources"] {
            assert!(record.get(key).is_none(), "{row}: {key}\n{record:#}");
        }
    }
    let record = typed_extract(&typed_request(
        "process",
        "",
        "| x | emits | event-001 | 1 |\n",
    ));
    assert!(findings(&record).is_empty(), "{record:#}");
}

#[trace("TC-2001", "FR-076-AC-18")]
// A header-only table on a typed-only type yields the empty typed key; a type
// declaring `relations` and a typed key keeps `relationSources` aligned with
// `relations`.
#[test]
fn typed_keys_beside_relations() {
    let record = typed_extract(&typed_request("process", "", ""));
    assert_eq!(record["availability"]["relations"]["state"], "available");
    assert_eq!(record["emits"], json!([]), "{record:#}");
    assert!(record.get("relations").is_none());
    assert!(record.get("relationSources").is_none());

    let record = typed_extract(&typed_request(
        "aggregate_root",
        "",
        "| placed | emits | event-001 | 0..* |\n| parent | references | aggregate-root-001 | 0..1 |\n",
    ));
    assert_eq!(
        record["emits"],
        json!([identity("event-001")]),
        "{record:#}"
    );
    let relations = record["relations"].as_array().unwrap();
    assert_eq!(relations.len(), 1);
    assert_eq!(relations[0]["verb"], "references");
    let sources = record["relationSources"].as_array().unwrap();
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0]["name"], "parent");
    assert!(sources[0].get("verb").is_none(), "{record:#}");
}

#[trace("TC-2002", "FR-076-AC-19", "FR-076-CON-4")]
// A typed key in frontmatter is a second declaration site: refused at line 1,
// and never read into the key.
#[test]
fn frontmatter_typed_key_is_duplicate_authority() {
    let record = typed_extract(&typed_request(
        "repository",
        "persists: [aggregate-root-001]\n",
        "| orders | persists | entity-001 | 0..* |\n",
    ));
    let duplicate: Vec<&Value> = record["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["code"] == "semantic.duplicate-authority")
        .collect();
    assert_eq!(duplicate.len(), 1, "{record:#}");
    assert_eq!(duplicate[0]["reason"], "frontmatter-key");
    assert_eq!(duplicate[0]["line"], 1);
    assert_eq!(duplicate[0]["severity"], "error");
    assert!(record.get("persists").is_none(), "{record:#}");
    assert!(!record.to_string().contains(&identity("aggregate-root-001")));

    let record = typed_extract(&typed_request(
        "repository",
        "",
        "| orders | persists | entity-001 | 0..* |\n",
    ));
    assert!(findings(&record).is_empty(), "{record:#}");
    assert_eq!(record["persists"], json!([identity("entity-001")]));
}

#[trace("TC-2003", "FR-076-AC-20")]
// No bundle index lowers a typed row unchecked with the advisory; one
// declaration has one identity across a model reference item and a typed
// row; a title stays `target-not-id`; a type without typed keys is unchanged.
#[test]
fn typed_rows_share_identity_and_index_states() {
    let mut unindexed = typed_request("process", "", "| done | emits | event-001 | 1 |\n");
    unindexed["bundle"]
        .as_object_mut()
        .unwrap()
        .remove("artifacts");
    let record = typed_extract(&unindexed);
    assert_eq!(
        record["emits"],
        json!([identity("event-001")]),
        "{record:#}"
    );
    assert_eq!(
        findings(&record),
        [(
            "semantic.unresolved-target".to_string(),
            "no-bundle-index".to_string()
        )]
    );
    assert_eq!(record["availability"]["relations"]["state"], "available");

    // One declaration, one identity: a step `Emits` item by title and a
    // typed row by id.
    let mut both = typed_request("process", "", "| done | emits | event-001 | 1 |\n");
    let md = both["markdown"].as_str().unwrap().to_string()
        + "\n## Steps\n\n| Step | Kind | Consumes | Emits | Description |\n|---|---|---|---|---|\n| place | command | | OrderPlaced | |\n";
    both["markdown"] = json!(md);
    both["bodyExtraction"] = json!({ "yield_pattern": { "match": { "steps": {
        "from": "table_row", "under_section": "Steps",
        "assert": { "columns": ["Step", "Kind", "Consumes", "Emits", "Description"] }
    } } } });
    let record = typed_extract(&both);
    assert_eq!(
        record["model"]["steps"][0]["emits"][0], record["emits"][0],
        "{record:#}"
    );
    assert_eq!(record["emits"][0], identity("event-001"));

    let record = typed_extract(&typed_request(
        "process",
        "",
        "| x | emits | OrderPlaced | 1 |\n",
    ));
    assert_eq!(
        findings(&record),
        [(
            "semantic.invalid-model-cell".to_string(),
            "target-not-id".to_string()
        )]
    );

    // A schema with no typed key reads every row as before.
    let mut untyped = typed_request(
        "aggregate_root",
        "",
        "| placed | emits | event-001 | 0..* |\n",
    );
    untyped["dataSchema"] = typed_schema(&[], true);
    let with_schema = serde_json::to_string(&extract_semantic_json(&untyped).unwrap()).unwrap();
    untyped.as_object_mut().unwrap().remove("dataSchema");
    let without = serde_json::to_string(&extract_semantic_json(&untyped).unwrap()).unwrap();
    assert_eq!(with_schema, without);
    assert!(with_schema.contains("\"relations\""), "{with_schema}");

    // A schema with neither a typed key nor `relations` reads every row as
    // before, too: the AC-1 fixture under `{ type: object }` is unchanged.
    let fixture = mapping_json("relationships.expected.json");
    let md = mapping_text("relationships.md");
    let mut plain = request(&fixture, &md, "relationships.md", None);
    let before = serde_json::to_string(&extract_semantic_json(&plain).unwrap()).unwrap();
    plain["dataSchema"] = json!({ "type": "object" });
    let after = serde_json::to_string(&extract_semantic_json(&plain).unwrap()).unwrap();
    assert_eq!(before, after);
    let record: Value = serde_json::from_str(&after).unwrap();
    assert_eq!(record["relations"], fixture["relations"]);
    assert_eq!(record["relationSources"], fixture["relationSources"]);
    assert_eq!(record["diagnostics"], fixture["diagnostics"]);
}

#[trace("TC-2150", "FR-076-AC-22")]
#[test]
fn entry_error_reasons_are_stated_by_locus() {
    for name in ["relationships-cases.json", "operations-cases.json"] {
        for case in mapping_json(name)["cases"].as_array().unwrap() {
            for (kind, availability) in case["availability"].as_object().into_iter().flatten() {
                let reason = availability["reason"].as_str().unwrap_or_default();
                assert!(
                    !reason.starts_with("entry-errors"),
                    "{name} {}: {kind} states a string entry-errors reason",
                    case["id"]
                );
            }
        }
    }
    let two_rows = cases()["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["availability"]["relations"]["reasonLoci"] == json!(["row", "second-row"]))
        .expect("a case with two erroring rows")
        .clone();
    run(&[two_rows["id"].as_str().unwrap()]);
}

#[test]
#[should_panic(expected = "reason lines")]
fn reason_lines_differing_from_resolved_loci_fail() {
    let got = json!({ "state": "unavailable", "reason": "entry-errors: lines 20", "lossy": false });
    let want = json!({ "state": "unavailable", "reasonLoci": ["second-row"], "lossy": false });
    let diagnostics = [json!({ "locus": "second-row", "line": 21 })];
    assert_reason_loci("synthetic", &got, &want, &diagnostics);
}
