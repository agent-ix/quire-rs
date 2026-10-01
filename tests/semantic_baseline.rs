//! Plan-003 Task-015: embedded-schema completeness and pre-change baselines
//! (FR-069-AC-8, FR-069-AC-9/CON-3, FR-072-AC-9, NFR-021-AC-3).
//!
//! The baselines under `tests/fixtures/semantic/baseline/` were minted before
//! any semantic extraction code landed. A later diff against them is a defect
//! in the change, never a reason to re-mint.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use ix_trace_rs::trace;
use jsonschema::JSONSchema;
use quire_rs::{extract_filament_core, FilamentExtractionInput, Registry};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn write_or_compare(path: &Path, actual: &str) {
    if std::env::var_os("UPDATE_SEMANTIC_BASELINES").is_some() {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, actual).unwrap();
        return;
    }
    let expected = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("baseline {} unreadable: {e}", path.display()));
    assert!(
        expected == actual,
        "{} differs from the checked-in baseline (minted on main before #388); this is a contract change, not a stale baseline",
        path.display()
    );
}

#[trace("TC-1606", "FR-069-AC-8")]
// Every supported semantic-core version is a complete, valid embedded
// bundle.
#[test]
fn embedded_semantic_core_versions_are_complete_bundles() {
    assert!(
        !quire_rs::semantic::embedded::SEMANTIC_CORE_VERSIONS.is_empty(),
        "no supported semantic-core version"
    );
    for version in quire_rs::semantic::embedded::SEMANTIC_CORE_VERSIONS {
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

#[derive(Serialize)]
struct ArchetypeProjection {
    module: String,
    name: String,
    body_extraction: Option<String>,
    carry_over: String,
}

fn registry_projection(registry: &Registry) -> Vec<ArchetypeProjection> {
    let mut names: Vec<&str> = registry.archetype_names().collect();
    names.sort_unstable();
    names
        .into_iter()
        .map(|name| {
            let a = registry.archetype(name).unwrap();
            ArchetypeProjection {
                module: a.module.clone(),
                name: a.name.clone(),
                body_extraction: a.body_extraction().map(|d| format!("{d:?}")),
                carry_over: format!("{:?}", a.carry_over),
            }
        })
        .collect()
}

#[trace("TC-1607", "FR-069-AC-9", "FR-069-CON-3", "FR-031-AC-7")]
// fixture modules without a `semantic` block load to the archetype projection
// recorded baseline.
#[test]
fn fixture_module_registries_match_baseline() {
    let modules = ["bundle", "demo", "req-fixture"];
    let mut all: BTreeMap<String, Vec<ArchetypeProjection>> = BTreeMap::new();
    for module in modules {
        let path = root().join("tests/fixtures/modules").join(module);
        let registry = Registry::load_module(&path).unwrap();
        assert!(
            registry.failures().is_empty(),
            "{module}: {:?}",
            registry.failures()
        );
        all.insert(module.to_string(), registry_projection(&registry));
    }
    let actual = format!("{}\n", serde_json::to_string_pretty(&all).unwrap());
    write_or_compare(
        &root().join("tests/fixtures/semantic/baseline/registry-archetypes.json"),
        &actual,
    );
}

#[derive(Deserialize)]
struct GraphCase {
    name: String,
    input: FilamentExtractionInput,
}

#[trace("TC-1643", "NFR-021-AC-3")]
// every Filament graph case output equals the baseline minted on main, and
// no diagnostic severity leaves the FR-045 set.
#[test]
fn filament_graph_cases_match_baseline() {
    let cases: Vec<GraphCase> =
        serde_json::from_str(include_str!("fixtures/filament_core/graph_cases.json")).unwrap();
    let mut outputs: BTreeMap<String, Value> = BTreeMap::new();
    for case in cases {
        let result = extract_filament_core(case.input);
        let value = serde_json::to_value(&result).unwrap();
        for d in value["diagnostics"].as_array().unwrap() {
            let severity = d["severity"].as_str().unwrap();
            assert!(
                matches!(severity, "info" | "warning" | "error"),
                "{}: severity {severity} outside the FR-045 set",
                case.name
            );
        }
        outputs.insert(case.name, value);
    }
    let actual = format!("{}\n", serde_json::to_string_pretty(&outputs).unwrap());
    write_or_compare(
        &root().join("tests/fixtures/semantic/baseline/filament-graph-cases.json"),
        &actual,
    );
}
