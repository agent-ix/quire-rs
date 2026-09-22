//! Plan-003 Task-015: embedded-schema completeness and pre-change baselines
//! (FR-069-AC-8, FR-069-AC-9/CON-3, FR-072-AC-9, NFR-021-AC-3).
//!
//! The baselines under `tests/fixtures/semantic/baseline/` were minted from
//! `main` (e3352a0) before any semantic extraction code landed. A later diff
//! against them is a defect in the change, never a reason to re-mint. Set
//! `UPDATE_SEMANTIC_BASELINES=1` only on a clean `main` checkout.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use ix_trace_rs::trace;
use jsonschema::JSONSchema;
use quire_rs::{extract_filament_core, FilamentExtractionInput, Registry};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
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
// bundle, and each bundle's content still matches the digest this repo
// pinned before CR-181. CR-181 deleted the check that hashed a *committed*
// copy of the bundle bytes against `schemas/vendored/PROVENANCE.json` (and a
// committed copy of filament-core-data's target enum, `common.schema.json`);
// CR-184 deleted the third committed copy, `schemas/vendored/
// module-manifest.schema.json`, once `@agent-ix/semantic-schema` published
// it. Nothing under `schemas/vendored/` remains: every embedded schema comes
// from a published `@agent-ix` package at build time (see `build.rs`), never
// from a file in this repository, and the target enum was dead code (see
// `contract.rs`'s step-3/step-6 note). FR-069-CON-2 (which required a
// provenance record to pin against) is removed for the same reason. But the
// *content* digest itself is not ceremony — it is what proves the
// build-fetched bytes are the exact bundle this crate's tests were written
// against — so it is restored here, computed straight over the
// build-fetched, embedded bundle bytes rather than over a committed copy.
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

        // Bundle digest: "<name>\n<bytes>" over every schema file, in the
        // sorted order `build.rs` already emits them in.
        let mut hasher = Sha256::new();
        for (name, text) in bundle {
            hasher.update(name.as_bytes());
            hasher.update(b"\n");
            hasher.update(text.as_bytes());
        }
        let digest = format!("sha256:{:x}", hasher.finalize());
        let pinned = match *version {
            "0.3.0" => "sha256:65b4e8d4c71a343e270618c9a8ca7e33687f10324ef5e9fe68d150056101c627",
            other => panic!("semantic-core {other} has no pinned digest"),
        };
        assert_eq!(
            digest, pinned,
            "semantic-core {version} bundle digest changed from the pinned value; if \
             @agent-ix/semantic-core@{version} genuinely republished with different \
             content, update `pinned` to the new digest — don't relax this assertion"
        );
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
    raw_schema_sha256: String,
    frontmatter_schema_sha256: Option<String>,
    data_schema_sha256: Option<String>,
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
            let sha = |v: &Value| sha256_hex(serde_json::to_string(v).unwrap().as_bytes());
            ArchetypeProjection {
                module: a.module.clone(),
                name: a.name.clone(),
                raw_schema_sha256: sha(&a.raw_schema),
                frontmatter_schema_sha256: a.frontmatter_schema.as_deref().map(sha),
                data_schema_sha256: a.data_schema.as_deref().map(sha),
                body_extraction: a.body_extraction().map(|d| format!("{d:?}")),
                carry_over: format!("{:?}", a.carry_over),
            }
        })
        .collect()
}

#[trace("TC-1607", "FR-069-AC-9", "FR-069-CON-3", "FR-031-AC-7")]
// fixture modules without a `semantic` block load to the archetype projection
// minted on main.
#[test]
fn fixture_module_registries_match_baseline() {
    let modules = ["bundle", "demo", "iso"];
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

#[trace("TC-1639", "FR-072-AC-9", "FR-072-CON-1")]
// coverage-v1, properties-v1, and assurance-v1 schemas are byte-identical to
// the baseline minted on main: no existing contract gains a required key.
#[test]
fn published_schemas_match_baseline() {
    let mut digests: BTreeMap<String, String> = BTreeMap::new();
    for name in [
        "coverage-v1.schema.json",
        "properties-v1.schema.json",
        "assurance-v1.schema.json",
    ] {
        let bytes = fs::read(root().join("schemas/output").join(name)).unwrap();
        digests.insert(name.to_string(), sha256_hex(&bytes));
        let schema: Value = serde_json::from_slice(&bytes).unwrap();
        assert!(
            schema["required"].is_array(),
            "{name}: required array present"
        );
    }
    let actual = format!("{}\n", serde_json::to_string_pretty(&digests).unwrap());
    write_or_compare(
        &root().join("tests/fixtures/semantic/baseline/published-schemas.json"),
        &actual,
    );
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
