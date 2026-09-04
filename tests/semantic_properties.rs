//! FR-070 typed Properties extraction (TC-1610..TC-1618, TC-1621, TC-1647).
//! Plan-003 Task-018. Oracles: the quoin golden fixtures pinned under
//! `tests/fixtures/semantic/quoin/mapping/` and `config-version.bundle.json`.

use std::fs;
use std::path::PathBuf;

use ix_trace_rs::trace;
use proptest::prelude::*;
use quire_rs::semantic::{
    compile_module_schema, extract_fields, AvailabilityState, BundleEntry, BundleIndex, FieldsForm,
    SemanticContext, SemanticSeverity,
};
use quire_rs::Registry;
use serde_json::{json, Value};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn mapping(name: &str) -> String {
    fs::read_to_string(
        root()
            .join("tests/fixtures/semantic/quoin/mapping")
            .join(name),
    )
    .unwrap()
}

fn mapping_json(name: &str) -> Value {
    serde_json::from_str(&mapping(name)).unwrap()
}

fn bundle() -> BundleIndex {
    serde_json::from_slice(
        &fs::read(root().join("tests/fixtures/semantic/config-version.bundle.json")).unwrap(),
    )
    .unwrap()
}

fn context(path: &str, bundle: BundleIndex) -> SemanticContext {
    let registry =
        Registry::load_module(&root().join("tests/fixtures/semantic/quoin/module-ok")).unwrap();
    let module = registry
        .semantic_module("spec-objects-fixture")
        .unwrap()
        .clone();
    SemanticContext::new(module, path, bundle)
}

fn field_decl_gate() -> jsonschema::JSONSchema {
    let schema = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://schemas.agent-ix.org/agent-ix/quire-rs/0.1.0/Gate.json",
        "$ref": "https://schemas.agent-ix.org/semantic-core/0.1.0/FieldDecl.json"
    });
    compile_module_schema(
        &schema,
        &|_| None,
        "0.1.0",
        "https://schemas.agent-ix.org/agent-ix/quire-rs/",
    )
    .unwrap()
}

fn fields_json(outcome: &quire_rs::semantic::FieldsOutcome) -> Value {
    serde_json::to_value(outcome.fields.as_ref().expect("fields available")).unwrap()
}

#[trace("TC-1610", "FR-070-AC-1")]
// the golden table extracts to the normalized FieldDecl[] and validates.
#[test]
fn golden_table_extracts_to_expected_fields() {
    let outcome = extract_fields(
        &mapping("config-version.table.md"),
        &context("config-version.table.md", bundle()),
    );
    assert_eq!(
        outcome.availability.state,
        AvailabilityState::Available,
        "{:?}",
        outcome.diagnostics
    );
    assert_eq!(outcome.form, Some(FieldsForm::Table));
    assert_eq!(
        fields_json(&outcome),
        mapping_json("config-version.expected.json")["fields"]
    );
    let gate = field_decl_gate();
    for field in outcome.fields.as_ref().unwrap() {
        assert!(
            gate.is_valid(&serde_json::to_value(field).unwrap()),
            "{field:?}"
        );
    }
    assert!(!outcome.availability.lossy);
    assert!(outcome.diagnostics.is_empty(), "{:?}", outcome.diagnostics);
}

#[trace("TC-1611", "FR-070-AC-2")]
// the fence form yields the identical normalized array.
#[test]
fn golden_fence_extracts_to_identical_fields() {
    let table = extract_fields(
        &mapping("config-version.table.md"),
        &context("config-version.table.md", bundle()),
    );
    let fence = extract_fields(
        &mapping("config-version.fence.md"),
        &context("config-version.fence.md", bundle()),
    );
    assert_eq!(
        fence.availability.state,
        AvailabilityState::Available,
        "{:?}",
        fence.diagnostics
    );
    assert_eq!(fence.form, Some(FieldsForm::Fence));
    assert_eq!(fields_json(&fence), fields_json(&table));
    assert_eq!(
        serde_json::to_string(&fence.fields).unwrap(),
        serde_json::to_string(&table.fields).unwrap(),
        "byte-identical normalized form"
    );
}

fn artifact(properties_body: &str) -> String {
    format!("---\nid: FR-900\ntitle: Thing\nobject: entity\n---\n# FR-900: Thing\n\n## Properties\n\n{properties_body}\n\n## Notes\n\nprose\n")
}

#[trace("TC-1612", "FR-070-AC-3")]
// both forms, typed-then-legacy, and duplicate headings.
#[test]
fn both_forms_and_duplicate_sections_are_refused() {
    let both = extract_fields(
        &mapping("both-forms.md"),
        &context("both-forms.md", bundle()),
    );
    assert_eq!(both.availability.state, AvailabilityState::Unavailable);
    assert_eq!(both.availability.reason.as_deref(), Some("both-forms"));
    assert!(both.fields.is_none());
    let expected = mapping_json("both-forms.expected.json");
    let d = &both.diagnostics[0];
    assert_eq!(d.code, "semantic.properties-both-forms");
    assert_eq!(
        d.line,
        expected["expectedDiagnostic"]["line"]
            .as_u64()
            .map(|l| l as usize)
    );

    let mixed = artifact("| Field | Type | Multiplicity | Constraints |\n|---|---|---|---|\n| id | UUID | 1 | identity |\n\n- name: str — display name\n");
    let out = extract_fields(&mixed, &context("mixed.md", bundle()));
    assert_eq!(out.availability.reason.as_deref(), Some("both-forms"));
    assert_eq!(out.diagnostics[0].code, "semantic.properties-both-forms");
    assert_eq!(out.diagnostics[0].line, Some(14), "the list's line");

    let twice = format!(
        "{}\n## Properties\n\n| Field | Type | Multiplicity | Constraints |\n|---|---|---|---|\n",
        artifact("| Field | Type | Multiplicity | Constraints |\n|---|---|---|---|\n")
    );
    let out = extract_fields(&twice, &context("twice.md", bundle()));
    assert_eq!(
        out.availability.reason.as_deref(),
        Some("duplicate-section")
    );
    assert_eq!(out.diagnostics[0].code, "semantic.duplicate-section");
}

/// Run one `cell-cases.json` case through a one-row table.
fn run_cell_case(case: &Value) -> (quire_rs::semantic::FieldsOutcome, usize) {
    let cell = &case["cell"];
    let type_cell = cell["Type"].as_str().unwrap_or("String");
    let mult = cell["Multiplicity"].as_str().unwrap_or("1");
    let cons = cell["Constraints"].as_str().unwrap_or("");
    let markdown = artifact(&format!(
        "| Field | Type | Multiplicity | Constraints |\n|---|---|---|---|\n| f | {type_cell} | {mult} | {} |",
        cons.replace('|', "\\|")
    ));
    let mut index = bundle();
    if let Some(b) = case.get("bundle") {
        index = BundleIndex {
            package: "agent-ix/config-service".into(),
            ..BundleIndex::default()
        };
        for o in b["objects"].as_array().map(Vec::as_slice).unwrap_or(&[]) {
            index.objects.push(BundleEntry {
                id: o["id"].as_str().unwrap().into(),
                names: vec![o["title"].as_str().unwrap().into()],
            });
        }
        for e in b["enumerations"]
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or(&[])
        {
            index.enumerations.push(BundleEntry {
                id: e["id"].as_str().unwrap().into(),
                names: vec![e["title"].as_str().unwrap().into()],
            });
        }
    }
    if let Some(imports) = case.get("imports").and_then(Value::as_object) {
        for (package, spec) in imports {
            let exports = spec["exports"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect();
            index.imports.insert(package.clone(), exports);
        }
    }
    let row_line = 12;
    (
        extract_fields(&markdown, &context("cell.md", index)),
        row_line,
    )
}

fn assert_case_diagnostics(
    id: &str,
    outcome: &quire_rs::semantic::FieldsOutcome,
    expected: &[Value],
    row_line: usize,
) {
    for d in expected {
        let code = d["code"].as_str().unwrap();
        let severity = match d["severity"].as_str().unwrap() {
            "advisory" => SemanticSeverity::Advisory,
            "warning" => SemanticSeverity::Warning,
            _ => SemanticSeverity::Error,
        };
        let line = match d["locus"].as_str().unwrap() {
            "row" => row_line,
            "fence-line" => row_line,
            other => panic!("{id}: locus {other}"),
        };
        assert!(
            outcome
                .diagnostics
                .iter()
                .any(|x| x.code == code && x.severity == severity && x.line == Some(line)),
            "{id}: expected {code} {severity:?} at {line}, got {:?}",
            outcome.diagnostics
        );
    }
}

fn cell_cases(prefix: &str) -> Vec<Value> {
    mapping_json("cell-cases.json")["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["id"].as_str().unwrap().starts_with(prefix))
        .cloned()
        .collect()
}

#[trace("TC-1613", "FR-070-AC-4")]
// every Type case, plus ambiguity, empty index, and non-identifier tokens.
#[test]
fn type_cells() {
    let cases = cell_cases("type-");
    assert!(cases.len() >= 7);
    for case in &cases {
        let id = case["id"].as_str().unwrap();
        let (outcome, row) = run_cell_case(case);
        if let Some(expected) = case.get("expected") {
            let fields = outcome
                .fields
                .as_ref()
                .unwrap_or_else(|| panic!("{id}: {:?}", outcome.diagnostics));
            let t = serde_json::to_value(&fields[0].type_ref).unwrap();
            for (k, v) in expected.as_object().unwrap() {
                assert_eq!(&t[k], v, "{id}: {k}");
            }
        }
        if let Some(diags) = case.get("diagnostics") {
            assert_case_diagnostics(id, &outcome, diags.as_array().unwrap(), row);
            if diags
                .as_array()
                .unwrap()
                .iter()
                .any(|d| d["severity"] == "error")
            {
                assert!(
                    outcome.fields.is_none(),
                    "{id}: an error yields no partial array"
                );
            }
        }
    }
    // The unresolved placeholder is lossy and carries its reason.
    let (outcome, _) = run_cell_case(&json!({ "id": "x", "cell": { "Type": "Mystery" } }));
    assert!(outcome.availability.lossy);
    assert_eq!(
        outcome.diagnostics[0].reason.as_deref(),
        Some("unknown-token")
    );
    // Empty index: explicit reason.
    let out = extract_fields(&artifact("| Field | Type | Multiplicity | Constraints |\n|---|---|---|---|\n| f | ConfigVersion | 1 | |"), &context("e.md", BundleIndex::default()));
    assert_eq!(
        out.diagnostics[0].reason.as_deref(),
        Some("no-bundle-index")
    );
    assert_eq!(
        out.fields.as_ref().unwrap()[0].type_ref.target,
        "ix://agent-ix/spec-objects-fixture/unresolved/ConfigVersion"
    );
    // Two names: ambiguous.
    let mut index = bundle();
    index.objects.push(BundleEntry {
        id: "FR-099".into(),
        names: vec!["ConfigVersion".into()],
    });
    let out = extract_fields(&artifact("| Field | Type | Multiplicity | Constraints |\n|---|---|---|---|\n| f | ConfigVersion | 1 | |"), &context("a.md", index));
    assert!(
        out.diagnostics
            .iter()
            .any(|d| d.code == "semantic.ambiguous-type"
                && d.message.contains("FR-006")
                && d.message.contains("FR-099")),
        "{:?}",
        out.diagnostics
    );
    // Not an identifier.
    let out = extract_fields(&artifact("| Field | Type | Multiplicity | Constraints |\n|---|---|---|---|\n| f | Config-Version | 1 | |"), &context("i.md", bundle()));
    assert!(
        out.diagnostics
            .iter()
            .any(|d| d.code == "semantic.invalid-type-token"),
        "{:?}",
        out.diagnostics
    );
    assert!(out.fields.is_none());
}

#[trace("TC-1614", "FR-070-AC-5")]
// every Multiplicity case plus the boundary set.
#[test]
fn multiplicity_cells() {
    for case in cell_cases("mult-") {
        let id = case["id"].as_str().unwrap();
        let (outcome, row) = run_cell_case(&case);
        if let Some(expected) = case.get("expected") {
            let fields = outcome
                .fields
                .as_ref()
                .unwrap_or_else(|| panic!("{id}: {:?}", outcome.diagnostics));
            assert_eq!(
                serde_json::to_value(&fields[0].type_ref).unwrap()["multiplicity"],
                expected["multiplicity"],
                "{id}"
            );
        }
        if let Some(diags) = case.get("diagnostics") {
            assert_case_diagnostics(id, &outcome, diags.as_array().unwrap(), row);
        }
    }
    let extra = [
        ("*", Some(json!({ "lower": 0 }))),
        ("0..*", Some(json!({ "lower": 0 }))),
        ("0..0", Some(json!({ "lower": 0, "upper": 0 }))),
        (
            "2..2 unique",
            Some(json!({ "lower": 2, "upper": 2, "unique": true })),
        ),
        ("a..b", None),
        ("3..* ordered", Some(json!({ "lower": 3, "ordered": true }))),
    ];
    for (cell, expected) in extra {
        let (outcome, row) =
            run_cell_case(&json!({ "id": cell, "cell": { "Multiplicity": cell } }));
        match expected {
            Some(m) => assert_eq!(
                serde_json::to_value(&outcome.fields.unwrap()[0].type_ref).unwrap()["multiplicity"],
                m,
                "{cell}"
            ),
            None => assert!(
                outcome
                    .diagnostics
                    .iter()
                    .any(|d| d.code == "semantic.invalid-multiplicity" && d.line == Some(row)),
                "{cell}: {:?}",
                outcome.diagnostics
            ),
        }
    }
}

#[trace("TC-1615", "FR-070-AC-6")]
// every Constraints case plus nullable, pattern with commas, duplicate names.
#[test]
fn constraints_cells() {
    for case in cell_cases("con-") {
        let id = case["id"].as_str().unwrap();
        let (outcome, row) = run_cell_case(&case);
        if let Some(expected) = case.get("expected") {
            let fields = outcome
                .fields
                .as_ref()
                .unwrap_or_else(|| panic!("{id}: {:?}", outcome.diagnostics));
            let f = serde_json::to_value(&fields[0]).unwrap();
            for (k, v) in expected.as_object().unwrap() {
                assert_eq!(&f[k], v, "{id}: {k}");
            }
        }
        if let Some(diags) = case.get("diagnostics") {
            assert_case_diagnostics(id, &outcome, diags.as_array().unwrap(), row);
        }
    }
    let (outcome, _) =
        run_cell_case(&json!({ "id": "nullable", "cell": { "Constraints": "nullable, min: 2" } }));
    let f = serde_json::to_value(&outcome.fields.unwrap()[0]).unwrap();
    assert_eq!(f["nullable"], json!(true));
    assert_eq!(f["constraints"], json!([{ "keyword": "min", "value": 2 }]));
    let (outcome, _) = run_cell_case(
        &json!({ "id": "pattern", "cell": { "Constraints": "pattern: /a,b|c/, nonEmpty" } }),
    );
    let f = serde_json::to_value(&outcome.fields.unwrap()[0]).unwrap();
    assert_eq!(
        f["constraints"],
        json!([{ "keyword": "pattern", "regex": "a,b|c", "dialect": "ecma-262" }, { "keyword": "nonEmpty" }])
    );
    let dup = artifact("| Field | Type | Multiplicity | Constraints |\n|---|---|---|---|\n| id | UUID | 1 | identity |\n| id | String | 1 | |");
    let out = extract_fields(&dup, &context("dup.md", bundle()));
    assert!(
        out.diagnostics
            .iter()
            .any(|d| d.code == "agent-ix.semantic-core.DUPLICATE_NAME" && d.line == Some(13)),
        "{:?}",
        out.diagnostics
    );
    assert!(out.fields.is_none());
}

#[trace("TC-1616", "FR-070-AC-7")]
// fence-line cases and opaque brace content.
#[test]
fn fence_lines() {
    for case in cell_cases("fence-") {
        let id = case["id"].as_str().unwrap();
        let line = case["fenceLine"].as_str().unwrap();
        let markdown = artifact(&format!("```sysml\n{line}\n```"));
        let outcome = extract_fields(&markdown, &context("f.md", bundle()));
        // fence opens at line 10, the line under test is 11
        assert_case_diagnostics(id, &outcome, case["diagnostics"].as_array().unwrap(), 11);
        assert!(outcome.fields.is_none(), "{id}");
    }
    let ok = artifact("```sysml\nattribute n : Integer[1] { min: 1, maxLength: 64 }\nref item p : ConfigOverlay[0..1]\n```");
    let out = extract_fields(&ok, &context("f.md", bundle()));
    let f = serde_json::to_value(
        out.fields
            .as_ref()
            .unwrap_or_else(|| panic!("{:?}", out.diagnostics)),
    )
    .unwrap();
    assert_eq!(
        f[0]["constraints"],
        json!([{ "keyword": "min", "value": 1 }, { "keyword": "maxLength", "value": 64 }])
    );
    assert_eq!(
        f[1]["type"]["target"],
        "ix://agent-ix/config-service/type/ConfigOverlay"
    );
    // Arbitrary UTF-8 brace text is the Constraints cell: the cell grammar
    // judges it, never a brace parser.
    let odd = artifact("```sysml\nattribute n : Integer[1] { ✓ weird: {nested}, min: 1 }\n```");
    let out = extract_fields(&odd, &context("f.md", bundle()));
    assert!(
        out.diagnostics
            .iter()
            .any(|d| d.code == "semantic.unknown-constraint-keyword" && d.line == Some(11)),
        "{:?}",
        out.diagnostics
    );
    assert!(!out
        .diagnostics
        .iter()
        .any(|d| d.code == "semantic.sysml-outside-subset"));
    // `ref item` to a kernel scalar is outside the subset.
    let bad = artifact("```sysml\nref item p : String[1]\n```");
    let out = extract_fields(&bad, &context("f.md", bundle()));
    assert!(out
        .diagnostics
        .iter()
        .any(|d| d.code == "semantic.sysml-outside-subset"));
}

#[trace("TC-1617", "FR-070-AC-8")]
// legacy forms per legacy.expected.json; `legacy_forms: error` promotes.
#[test]
fn legacy_forms() {
    let expected = mapping_json("legacy.expected.json");
    for case in expected["cases"].as_array().unwrap() {
        let file = case["file"].as_str().unwrap();
        let path = root()
            .join("tests/fixtures/semantic/quoin/mapping")
            .join(file);
        let markdown = fs::read_to_string(&path).unwrap();
        let outcome = extract_fields(&markdown, &context(file, bundle()));
        let form = case["form"].as_str().unwrap();
        let line = case["line"].as_u64().map(|l| l as usize);
        match form {
            "typed-table" => {
                assert_eq!(outcome.form, Some(FieldsForm::Table), "{file}");
                assert_eq!(outcome.block_line, line, "{file}");
                assert!(outcome
                    .diagnostics
                    .iter()
                    .all(|d| d.code != "semantic.legacy-properties-form"));
            }
            "sysml-fence" => {
                assert_eq!(outcome.form, Some(FieldsForm::Fence), "{file}");
                assert_eq!(outcome.block_line, line, "{file}");
            }
            legacy => {
                assert_eq!(
                    outcome.availability.state,
                    AvailabilityState::Unavailable,
                    "{file}"
                );
                assert_eq!(
                    outcome.availability.reason.as_deref(),
                    Some("legacy-form"),
                    "{file}"
                );
                assert_eq!(outcome.block_line, line, "{file}");
                let d = outcome
                    .diagnostics
                    .iter()
                    .find(|d| d.code == "semantic.legacy-properties-form")
                    .unwrap_or_else(|| panic!("{file}: {:?}", outcome.diagnostics));
                assert_eq!(d.severity, SemanticSeverity::Warning, "{file}");
                assert_eq!(d.line, line, "{file}");
                assert_eq!(d.reason.as_deref(), Some(legacy), "{file}");
                assert_eq!(case["diagnostic"]["form"], legacy);
                assert_eq!(case["diagnostic"]["migration"], "typed-table");
                assert!(d.message.contains("typed-table"));
            }
        }
    }
    // The `properties` string yielded by section_body stays untouched: the
    // module-ok DSL extracts it exactly as before (FR-070-CON-3).
    let registry =
        Registry::load_module(&root().join("tests/fixtures/semantic/quoin/module-ok")).unwrap();
    let dsl = registry
        .archetype("entity")
        .unwrap()
        .body_extraction()
        .unwrap();
    let doc = quire_rs::parse_document(&mapping("legacy-bullets.md"));
    let record = &quire_rs::extract(&doc, dsl).unwrap().records[0];
    assert!(record["properties"]
        .as_str()
        .unwrap()
        .contains("- id: UUID"));
    assert!(!record.contains_key("fields"));
    // legacy_forms: error promotes the finding.
    let mut ctx = context("legacy-bullets.md", bundle());
    ctx.module.legacy_forms = "error".into();
    let out = extract_fields(&mapping("legacy-bullets.md"), &ctx);
    assert_eq!(out.diagnostics[0].severity, SemanticSeverity::Error);
}

#[trace("TC-1618", "FR-070-AC-9", "FR-070-CON-3")]
// a module without a block yields the pre-change record: no fields key, no
// semantic diagnostic (the Filament byte-identity half is TC-1632).
#[test]
fn no_block_record_is_unchanged() {
    let registry = Registry::load_module(&root().join("tests/fixtures/modules/bundle")).unwrap();
    assert!(registry.semantic_modules().next().is_none());
    for name in registry.archetype_names() {
        let arch = registry.archetype(name).unwrap();
        assert!(arch.semantic_schema_digest.is_none(), "{name}");
    }
    assert!(!registry
        .diagnostics()
        .iter()
        .any(|d| matches!(d, quire_rs::Diagnostic::Semantic { .. })));
}

#[trace("TC-1647", "FR-070-AC-10")]
// one row error → unavailable naming the loci, no partial array; an empty
// typed table is available and empty.
#[test]
fn row_errors_and_empty_tables() {
    let md = artifact("| Field | Type | Multiplicity | Constraints |\n|---|---|---|---|\n| id | UUID | 1 | identity |\n| bad | Integer | 5..2 | |\n| worse | Integer | 1 | mnimum: 1 |");
    let out = extract_fields(&md, &context("r.md", bundle()));
    assert_eq!(out.availability.state, AvailabilityState::Unavailable);
    let reason = out.availability.reason.clone().unwrap();
    assert!(
        reason.starts_with("row-errors") && reason.contains("13") && reason.contains("14"),
        "{reason}"
    );
    assert!(out.fields.is_none());
    let empty = artifact("| Field | Type | Multiplicity | Constraints |\n|---|---|---|---|");
    let out = extract_fields(&empty, &context("e.md", bundle()));
    assert_eq!(out.availability.state, AvailabilityState::Available);
    assert_eq!(out.fields.as_ref().map(Vec::len), Some(0));
    assert_eq!(out.form, Some(FieldsForm::Table));
}

fn cell_strategy() -> impl Strategy<Value = (String, String, String)> {
    let ty = prop_oneof![
        Just("UUID".to_string()),
        Just("Integer".to_string()),
        Just("Decimal(10,2)".to_string()),
        Just("Decimal".to_string()),
        Just("Duration [ms]".to_string()),
        Just("String [kg]".to_string()),
        Just("ConfigVersion".to_string()),
        Just("Mystery".to_string()),
        "[A-Za-z_][A-Za-z0-9_]{0,6}",
        "[^|\\n]{0,8}",
    ];
    let mult = prop_oneof![
        Just(String::new()),
        Just("1".to_string()),
        Just("0..1".to_string()),
        Just("1..*".to_string()),
        Just("*".to_string()),
        Just("2..5 ordered unique".to_string()),
        Just("5..2".to_string()),
        Just("1 ordered".to_string()),
        "[0-9*.a-z ]{0,8}",
    ];
    let cons = prop_oneof![
        Just(String::new()),
        Just("identity".to_string()),
        Just("min: 1, maxLength: 64".to_string()),
        Just("pattern: /^[a-z]+$/".to_string()),
        Just("enumValues: a|b".to_string()),
        Just("format: agent-ix:email".to_string()),
        Just("nullable, nonEmpty, unique".to_string()),
        Just("mnimum: 1".to_string()),
        "[^|\\n]{0,12}",
    ];
    (ty, mult, cons)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]
    #[trace("TC-1621", "FR-070-AC-4", "FR-070-AC-5", "FR-070-AC-6", "FR-070-AC-10")]
    // generated cells: every produced FieldDecl validates, or the row is
    // diagnosed and no partial array is emitted.
    #[test]
    fn generated_cells_validate_or_diagnose((ty, mult, cons) in cell_strategy()) {
        let md = artifact(&format!(
            "| Field | Type | Multiplicity | Constraints |\n|---|---|---|---|\n| f | {} | {} | {} |",
            ty.replace('|', "\\|"), mult.replace('|', "\\|"), cons.replace('|', "\\|")
        ));
        let out = extract_fields(&md, &context("p.md", bundle()));
        let gate = field_decl_gate();
        match out.availability.state {
            AvailabilityState::Available => {
                for field in out.fields.as_ref().unwrap() {
                    prop_assert!(gate.is_valid(&serde_json::to_value(field).unwrap()), "{field:?}");
                }
                prop_assert!(!out.diagnostics.iter().any(|d| d.is_error()));
            }
            AvailabilityState::Unavailable => {
                prop_assert!(out.fields.is_none());
                prop_assert!(out.diagnostics.iter().any(|d| d.is_error() && d.line == Some(12)), "{:?}", out.diagnostics);
            }
            other => prop_assert!(false, "unexpected state {other:?}"),
        }
    }
}
