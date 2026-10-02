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
