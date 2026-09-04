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
use quire_rs::{extract_filament_core, FilamentExtractionInput, Registry};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
    #[serde(rename = "semanticCore")]
    semantic_core: BTreeMap<String, BundleProvenance>,
}

#[derive(Deserialize)]
struct ProvenanceFile {
    repository: String,
    revision: String,
    path: String,
    sha256: String,
}

#[derive(Deserialize)]
struct BundleProvenance {
    #[serde(rename = "bundleDigest")]
    bundle_digest: String,
}

#[trace("TC-1606", "FR-069-AC-8", "FR-069-CON-2")]
// every vendored file hashes to its provenance record; the 0.1.0 bundle
// digest equals the filament-core-data toolchain.json constant.
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

    // Bundle digest: "<name>\n<bytes>" over every schema file in sorted order,
    // excluding toolchain.json — the filament-core-data rule.
    for (version, bundle) in &provenance.semantic_core {
        let bundle_dir = dir.join("semantic-core").join(version);
        let mut names: Vec<String> = fs::read_dir(&bundle_dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .filter(|n| n.ends_with(".json") && n != "toolchain.json")
            .collect();
        names.sort();
        let mut hasher = Sha256::new();
        for name in &names {
            hasher.update(name.as_bytes());
            hasher.update(b"\n");
            hasher.update(fs::read(bundle_dir.join(name)).unwrap());
        }
        let digest = format!("sha256:{:x}", hasher.finalize());
        assert_eq!(
            digest, bundle.bundle_digest,
            "semantic-core {version} bundle digest"
        );
        let toolchain: Value =
            serde_json::from_slice(&fs::read(bundle_dir.join("toolchain.json")).unwrap()).unwrap();
        assert_eq!(
            toolchain["digest"],
            Value::String(digest),
            "toolchain.json digest"
        );
        if version == "0.1.0" {
            assert_eq!(
                bundle.bundle_digest,
                "sha256:dd33c886f70e908b14507c35e078d163b76308c3d170d2b54ddf933d1a4ebb52"
            );
        }
    }
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

#[trace("TC-1607", "FR-069-AC-9", "FR-069-CON-3")]
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

#[trace("TC-1643", "NFR-021-AC-3")]
// every Filament graph case output equals the baseline minted on main.
#[trace("TC-1639", "FR-072-AC-9", "FR-072-CON-1")]
// the Filament API result carries no `semantic` key and no new severity for
// snapshots without a semantic context; coverage-v1 / properties-v1 /
// assurance-v1 stay pinned by TC-1090 and TC-1089.
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
