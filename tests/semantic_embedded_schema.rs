//! Plan-003 Task-015: embedded-schema completeness (FR-069-AC-8, FR-031-AC-8).

use ix_trace_rs::trace;
use jsonschema::JSONSchema;
use serde_json::{json, Value};

#[trace("TC-1606", "FR-069-AC-8")]
// Every supported semantic-core version is a complete, valid embedded
// bundle.
#[test]
fn embedded_semantic_core_versions_are_complete_bundles() {
    assert!(
        !quire_rs::semantic::embedded::semantic_core_versions().is_empty(),
        "no supported semantic-core version"
    );
    for version in quire_rs::semantic::embedded::semantic_core_versions() {
        let bundle = quire_rs::semantic::embedded::semantic_core_bundle(version)
            .unwrap_or_else(|| panic!("semantic-core {version} has no embedded bundle"));
        assert!(
            !bundle.is_empty(),
            "semantic-core {version} bundle is empty"
        );
        for (name, text) in bundle {
            let schema: Value = serde_json::from_str(text)
                .unwrap_or_else(|e| panic!("semantic-core {version}/{name}: not JSON: {e}"));
            assert_eq!(
                schema["$schema"],
                Value::String("https://json-schema.org/draft/2020-12/schema".to_string()),
                "semantic-core {version}/{name}: not a draft 2020-12 JSON Schema"
            );
        }
    }
}

#[trace("TC-1877", "FR-031-AC-8")]
// quire-rs's own loader never validates `object_types[].construct` against
// the embedded schema — only `properties.semantic` is compiled out of it
// (`contract::block_validator`) — so TC-1876's loader test cannot, on its
// own, prove the *schema* admits `immutable`. A manifest-validating consumer
// (e.g. the Python `validate_manifest` binding, called with this embedded
// schema's bytes) is what actually checks a `construct` value against
// `$defs/ConstructDeclaration`. Compile that $def directly out of the
// embedded `MODULE_MANIFEST_SCHEMA` bytes (fetched at build time from the
// published `@agent-ix/semantic-schema` package, CR-184) and assert it
// admits `immutable: true` and still refuses an unknown key, so a
// regression to the pre-#455 schema shape fails here even though nothing in
// quire-rs's own load path would notice.
#[test]
fn tc1877_construct_declaration_schema_admits_immutable() {
    let schema: Value = serde_json::from_str(quire_rs::semantic::embedded::MODULE_MANIFEST_SCHEMA)
        .expect("embedded module-manifest schema is JSON");
    let construct_decl = schema["$defs"]["ConstructDeclaration"].clone();
    assert!(
        construct_decl.is_object(),
        "embedded schema has no $defs/ConstructDeclaration"
    );
    let validator = JSONSchema::options()
        .compile(&construct_decl)
        .expect("$defs/ConstructDeclaration compiles standalone (no external $ref)");

    let mut construct = json!({
        "identity": "identified",
        "shape": "record",
        "members": { "fields": "required" },
        "meaning": "quire.meaning.event",
        "immutable": true
    });
    assert!(
        validator.is_valid(&construct),
        "a construct declaring immutable: true must validate against the \
         published $defs/ConstructDeclaration (filament-core-service \
         FR-035-AC-17, #455)"
    );

    // additionalProperties: false still holds — an unknown key is refused.
    construct["bogus"] = Value::Bool(true);
    assert!(
        !validator.is_valid(&construct),
        "$defs/ConstructDeclaration no longer refuses an unrecognized key"
    );
}

fn adjacent_fixture() -> Value {
    serde_json::from_str(include_str!(
        "fixtures/semantic/ir994-adjacent-emission.json"
    ))
    .unwrap()
}

#[trace(
    "TC-1610",
    "TC-1613",
    "TC-1615",
    "FR-070-AC-1",
    "FR-070-AC-4",
    "FR-070-AC-6"
)]
#[test]
fn ir994_public_extraction_emits_every_adjacent_constraint_form() {
    use quire_rs::semantic::{extract_fields, AvailabilityState, BundleIndex, SemanticContext};
    let registry = quire_rs::Registry::load_module(&quire_fixtures::module_ok_dir()).unwrap();
    let module = registry
        .semantic_module("spec-objects-fixture")
        .unwrap()
        .clone();
    let bundle = serde_json::from_str::<BundleIndex>(
        r#"{"package":"agent-ix/fixture","objects":[],"enumerations":[],"imports":{}}"#,
    )
    .unwrap();
    let context = SemanticContext::new(module, "ir994.md", bundle);
    let fixture = adjacent_fixture();
    assert!(fixture["issue_ref"].as_str().unwrap().contains('#'));
    for case in fixture["cases"].as_array().unwrap() {
        let markdown = format!(
            "---\nid: FR-100\nobject: entity\ntype: FR\n---\n# Test\n\n## Properties\n\n```sysml\nattribute value : {}[1] {{ {} }}\n```\n",
            case["type"].as_str().unwrap(), case["source"].as_str().unwrap()
        );
        let outcome = extract_fields(&markdown, &context);
        assert_eq!(
            outcome.availability.state,
            AvailabilityState::Available,
            "{}: {:?}",
            case["name"],
            outcome.diagnostics
        );
        assert!(
            outcome.diagnostics.is_empty(),
            "{}: {:?}",
            case["name"],
            outcome.diagnostics
        );
        let fields = serde_json::to_value(outcome.fields.unwrap()).unwrap();
        assert_eq!(
            fields[0]["type"]["target"],
            json!({"kind":"KernelScalar","value":case["type"]}),
            "{}",
            case["name"]
        );
        assert_eq!(
            fields[0]["constraints"],
            json!([case["expected"]]),
            "{}",
            case["name"]
        );
    }
}

#[trace("TC-1621", "TC-1638", "FR-070-AC-10", "FR-072-AC-8")]
#[test]
fn ir994_adjacent_declarations_round_trip_and_old_wire_is_refused() {
    use quire_rs::semantic::{compile_module_schema, embedded, FieldDecl};
    let version = embedded::embedded_semantic_core_version();
    let gate = compile_module_schema(
        &json!({
            "$schema":"https://json-schema.org/draft/2020-12/schema",
            "$id":"https://schemas.agent-ix.org/quire-rs/IR994Gate.json",
            "$ref":format!("https://schemas.agent-ix.org/semantic-core/{version}/FieldDecl.json")
        }),
        &|_| None,
        version,
        "https://schemas.agent-ix.org/quire-rs/",
    )
    .unwrap();
    let fixture = adjacent_fixture();
    let mut constraints: Vec<Value> = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|case| case["expected"].clone())
        .collect();
    constraints.push(json!({"kind":"EnumValuesConstraint","value":{
        "keyword":"enumValues","values":fixture["additional_decoded_enum_values"]}}));
    for target in [
        json!({"kind":"KernelScalar","value":"Integer"}),
        json!({"kind":"SemanticId","value":"ix://agent-ix/fixture/type/Order"}),
    ] {
        for constraint in &constraints {
            let value = json!({"name":"id","type":{"target":target},"constraints":[constraint]});
            assert!(
                gate.is_valid(&value),
                "owner schema refuses canonical control: {value:#}"
            );
            let decoded: FieldDecl =
                serde_json::from_value(value.clone()).expect("current adjacent declaration");
            assert_eq!(serde_json::to_value(decoded).unwrap(), value);
            let mut old_constraint = value.clone();
            old_constraint["constraints"] = json!([constraint["value"]]);
            assert!(!gate.is_valid(&old_constraint));
            assert!(serde_json::from_value::<FieldDecl>(old_constraint).is_err());
            match constraint["kind"].as_str().unwrap() {
                "MinConstraint"
                | "MaxConstraint"
                | "ExclusiveMinConstraint"
                | "ExclusiveMaxConstraint" => {
                    let mut raw_numeric = value.clone();
                    raw_numeric["constraints"][0]["value"]["value"] =
                        constraint["value"]["value"]["value"].clone();
                    assert!(!gate.is_valid(&raw_numeric));
                    assert!(serde_json::from_value::<FieldDecl>(raw_numeric).is_err());
                }
                "EnumValuesConstraint" => {
                    let mut raw_values = value.clone();
                    raw_values["constraints"][0]["value"]["values"] = Value::Array(
                        constraint["value"]["values"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|item| item["value"].clone())
                            .collect(),
                    );
                    assert!(!gate.is_valid(&raw_values));
                    assert!(serde_json::from_value::<FieldDecl>(raw_values).is_err());
                }
                _ => {}
            }
        }
    }
    let old = json!({"name":"id","type":{"target":"Integer"}});
    assert!(!gate.is_valid(&old));
    assert!(serde_json::from_value::<FieldDecl>(old).is_err());
}
