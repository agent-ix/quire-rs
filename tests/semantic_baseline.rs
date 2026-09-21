//! Plan-003 Task-015: vendored-schema provenance and pre-change baselines
//! (FR-069-AC-8/CON-2, FR-069-AC-9/CON-3, FR-072-AC-9, NFR-021-AC-3).
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

#[derive(Deserialize)]
struct Provenance {
    files: BTreeMap<String, ProvenanceFile>,
}

#[derive(Deserialize)]
struct ProvenanceFile {
    repository: String,
    revision: String,
    path: String,
    sha256: String,
}

#[trace("TC-1606", "FR-069-AC-8", "FR-069-CON-2")]
// every vendored file hashes to its provenance record; each semantic-core
// bundle digest equals its filament-core-data toolchain.json constant, and
// every vendored version is embedded.
#[test]
fn vendored_schemas_match_provenance() {
    let dir = root().join("schemas/vendored");
    let provenance: Provenance =
        serde_json::from_slice(&fs::read(dir.join("PROVENANCE.json")).unwrap()).unwrap();
    assert!(
        !provenance.files.is_empty(),
        "PROVENANCE.json lists no files"
    );

    let mut on_disk: Vec<String> = Vec::new();
    for entry in walkdir(&dir) {
        let rel = entry
            .strip_prefix(&dir)
            .unwrap()
            .to_string_lossy()
            .to_string();
        if rel == "PROVENANCE.json" {
            continue;
        }
        on_disk.push(rel);
    }
    on_disk.sort();
    let recorded: Vec<String> = provenance.files.keys().cloned().collect();
    assert_eq!(
        on_disk, recorded,
        "vendored files and PROVENANCE.json disagree"
    );

    for (rel, record) in &provenance.files {
        let bytes = fs::read(dir.join(rel)).unwrap();
        assert_eq!(
            format!("sha256:{}", sha256_hex(&bytes)),
            record.sha256,
            "{rel}: bytes differ from provenance ({}@{} {})",
            record.repository,
            record.revision,
            record.path
        );
        assert!(
            record.revision.len() == 40,
            "{rel}: revision is not a full commit id"
        );
    }

    // schemas/vendored/ no longer carries a semantic-core bundle at all
    // (PLAT-906): 0.2.0 resolves through the `filament-core-data` dependency
    // at build time (checked below, against the embedded bundle rather than
    // a directory) and 0.1.0 has no embedded bundle to check — see
    // `quire_rs::semantic::vendored`'s module doc comment.
    assert_eq!(
        quire_rs::semantic::vendored::SEMANTIC_CORE_VERSIONS,
        &["0.2.0"]
    );
    assert!(
        quire_rs::semantic::vendored::semantic_core_bundle("0.1.0").is_none(),
        "semantic-core 0.1.0 has no embedded bundle (PLAT-906): fcd does not \
         publish it in a resolvable form"
    );
    let bundle_02 = quire_rs::semantic::vendored::semantic_core_bundle("0.2.0")
        .expect("0.2.0 is a listed SEMANTIC_CORE_VERSIONS entry");
    let mut hasher = Sha256::new();
    for (name, bytes) in bundle_02 {
        hasher.update(name.as_bytes());
        hasher.update(b"\n");
        hasher.update(bytes.as_bytes());
    }
    assert_eq!(
        format!("sha256:{:x}", hasher.finalize()),
        "sha256:ef79c5dea98c19643b20daa8899951a4782d6248527a0647c114c6f76cca8aea",
        "semantic-core 0.2.0 bundle (resolved via filament-core-data, PLAT-906)"
    );
}

#[trace("TC-1877", "FR-031-AC-8")]
// quire-rs's own loader never validates `object_types[].construct` against
// the vendored schema — only `properties.semantic` is compiled out of it
// (`contract::block_validator`) — so TC-1876's loader test cannot, on its
// own, prove the *schema* admits `immutable`. A manifest-validating consumer
// (e.g. the Python `validate_manifest` binding, called with this vendored
// file's path) is what actually checks a `construct` value against
// `$defs/ConstructDeclaration`. Compile that $def directly out of the
// embedded `MODULE_MANIFEST_SCHEMA` bytes and assert it admits
// `immutable: true` and still refuses an unknown key, so a regression to
// the pre-#455 vendored bytes fails here even though nothing in quire-rs's
// own load path would notice.
#[test]
fn tc1877_construct_declaration_schema_admits_immutable() {
    let schema: Value = serde_json::from_str(quire_rs::semantic::vendored::MODULE_MANIFEST_SCHEMA)
        .expect("vendored module-manifest schema is JSON");
    let construct_decl = schema["$defs"]["ConstructDeclaration"].clone();
    assert!(
        construct_decl.is_object(),
        "vendored schema has no $defs/ConstructDeclaration"
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
         re-vendored $defs/ConstructDeclaration (filament-core-service \
         FR-035-AC-17, #455)"
    );

    // additionalProperties: false still holds — an unknown key is refused.
    construct["bogus"] = Value::Bool(true);
    assert!(
        !validator.is_valid(&construct),
        "$defs/ConstructDeclaration no longer refuses an unrecognized key"
    );
}

fn walkdir(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            out.extend(walkdir(&path));
        } else {
            out.push(path);
        }
    }
    out
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
