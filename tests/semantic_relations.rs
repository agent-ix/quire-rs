//! FR-076 relationships extraction (TC-1852..TC-1860). Oracles: the quoin
//! FR-104 fixtures `relationships.md`, `relationships.expected.json`, and
//! `relationships-cases.json` (vendored at `31ca54d`). The extraction context
//! is built from each fixture's recorded `context`, never from `module-ok`.

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use ix_trace_rs::trace;
use jsonschema::JSONSchema;
use quire_rs::semantic::python_entry::extract_semantic_json;
use quire_rs::semantic::{compile_module_schema, SEMANTIC_V1_SCHEMA};
use serde_json::{json, Value};

fn mapping_json(name: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/semantic/quoin/mapping")
        .join(name);
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

fn mapping_text(name: &str) -> String {
    fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/semantic/quoin/mapping")
            .join(name),
    )
    .unwrap()
}

fn semantic_v1() -> JSONSchema {
    JSONSchema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .compile(&serde_json::from_str::<Value>(SEMANTIC_V1_SCHEMA).unwrap())
        .unwrap()
}

fn relation_decl_gate(core: &str) -> JSONSchema {
    let schema = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://schemas.agent-ix.org/agent-ix/quire-rs/0.1.0/RelationDeclGate.json",
        "$ref": format!("https://schemas.agent-ix.org/semantic-core/{core}/RelationDecl.json")
    });
    compile_module_schema(
        &schema,
        &|_| None,
        core,
        "https://schemas.agent-ix.org/agent-ix/quire-rs/",
    )
    .unwrap()
}

/// The FR-076 request for `markdown` under a fixture's recorded `context`,
/// with `mappings` replacing the context's when given.
fn request(fixture: &Value, markdown: &str, path: &str, mappings: Option<&Value>) -> Value {
    let context = &fixture["context"];
    let bundle = &context["bundle"];
    let imports: serde_json::Map<String, Value> = bundle["imports"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| (p.as_str().unwrap().to_string(), json!("*")))
        .collect();
    json!({
        "markdown": markdown,
        "module": {
            "contractVersion": "1.0.0",
            "semanticCore": fixture["semanticCore"],
            "package": "agent-ix/spec-objects-business",
            "exports": ["entity"],
            "imports": imports,
            "mappings": mappings.unwrap_or(&context["mappings"]),
        },
        "path": path,
        "sourceIdentity": context["sourceIdentity"],
        "bundle": { "package": bundle["package"], "artifacts": bundle["artifacts"] },
        "relationVocabulary": {
            "objectType": context["objectType"],
            "edgeTypes": context["edgeTypes"],
            "roles": context["roles"],
            "allowedLinks": context["allowedLinks"],
        },
    })
}

/// Extract; every record validates against semantic-v1 and every relation
/// against the fixture's semantic-core `RelationDecl.json`.
fn extract(request: &Value, core: &str) -> Value {
    let record = serde_json::to_value(extract_semantic_json(request).unwrap()).unwrap();
    let errors: Vec<String> = match semantic_v1().validate(&record) {
        Ok(()) => Vec::new(),
        Err(errs) => errs
            .map(|e| format!("{e} at {}", e.instance_path))
            .collect(),
    };
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
        "second-heading" => at(line) == "## Relationships",
        other => panic!("unknown locus {other:?}"),
    }
}

/// Run one `relationships-cases.json` case and assert every field it records.
fn assert_case(file: &Value, case: &Value) {
    let id = case["id"].as_str().unwrap();
    let md = case_markdown(file, case);
    let core = file["semanticCore"].as_str().unwrap();
    let record = extract(&request(file, &md, "case.md", case.get("mappings")), core);
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
        Some(want) => assert_eq!(
            &record["availability"]["relations"], want,
            "{id}: availability.relations"
        ),
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
];
const GATING_CASES: &[&str] = &[
    "mapping-not-declared",
    "subset-header-unowned-section",
    "preamble-table",
    "prose-without-mapping",
];
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
    let core = expected["semanticCore"].as_str().unwrap();
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
    assert_eq!(in_fixture.len(), 31);
    assert_eq!(unique, in_fixture);
}

#[trace("TC-1860", "FR-076-AC-9")]
// no token and no relationship table: no relations keys; declaration_record
// carries relations; a renamed verb keeps its registry category and
// composition (CON-2).
#[test]
fn record_compatibility_and_registry_authority() {
    let file = cases();
    let core = file["semanticCore"].as_str().unwrap();
    let prose = format!(
        "{}- `overlay`: references FR-005\n",
        file["artifactHead"].as_str().unwrap()
    );
    let record = extract(
        &request(&file, &prose, "case.md", Some(&json!(["typed-table"]))),
        core,
    );
    assert!(record.get("relations").is_none());
    assert!(record.get("relationSources").is_none());
    assert!(record["availability"].get("relations").is_none());

    let expected = mapping_json("relationships.expected.json");
    let md = mapping_text("relationships.md");
    let typed = extract_semantic_json(&request(&expected, &md, "relationships.md", None)).unwrap();
    assert_eq!(
        typed.declaration_record()["relations"],
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
