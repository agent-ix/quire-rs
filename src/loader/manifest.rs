//! `manifest.yaml` parser (FR-013 manifest section).
//!
//! Each module root contains a `manifest.yaml` enumerating its
//! archetypes. Only the fields we actually consume to load a module are
//! deserialized here — the rest (grammars, lint_rules, defaults) are
//! captured via `serde(flatten)` into a free-form map so authors can
//! evolve the manifest without breaking the loader.
//!
//! Both shapes documented in FR-013 are supported:
//!
//! - `artifact_types: [{name, template_ref, frontmatter_schema_ref}]`
//!   (canonical `spec-artifacts-*` shape)
//! - `object_types: [{name, ...}]` (ix-spec-objects shape — has no
//!   templates; object types are data-only)

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Top-level `manifest.yaml` shape.
///
/// `name` is optional at the YAML layer — when absent, the loader
/// derives one from the parent directory and emits a
/// `Diagnostic::ManifestMissingName` (FR-014-AC-7).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub artifact_types: Vec<ArtifactType>,
    #[serde(default)]
    pub object_types: Vec<ObjectType>,
}

/// One `artifact_types[*]` entry — the canonical FR-013 archetype:
/// template + schema referenced by relative path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArtifactType {
    pub name: String,
    pub template_ref: PathBuf,
    pub frontmatter_schema_ref: PathBuf,
    /// Free-form passthrough so authors can carry extra fields without
    /// the loader needing to know about them.
    #[serde(flatten)]
    pub extras: Map<String, Value>,
}

/// One `object_types[*]` entry — data-only archetype (no template).
/// Object types still get a JSON Schema (`data_schema`) compiled at
/// load time so consumers can validate inputs.
///
/// `body_extraction` is the optional DSL the loader validates at
/// load time (FR-011-AC-6/7/8). Structural failures (both `match`
/// and `iterate_over`, missing `from:`, unknown keys) surface as
/// `ArchetypeLoadFailure` so authoring tools see them immediately.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectType {
    pub name: String,
    #[serde(default)]
    pub data_schema: Option<Value>,
    #[serde(default)]
    pub body_extraction: Option<crate::extract::dsl::ExtractionDsl>,
    #[serde(flatten)]
    pub extras: Map<String, Value>,
}

/// Parse a `manifest.yaml` document from `bytes`.
///
/// Returns the typed [`Manifest`] or a `String` error message suitable
/// for wrapping in `QuireError::ManifestError`.
pub fn parse_manifest(bytes: &[u8]) -> Result<Manifest, String> {
    serde_yaml::from_slice::<Manifest>(bytes).map_err(|e| e.to_string())
}

/// Read + parse `manifest.yaml` from `module_root`.
pub fn load_manifest(module_root: &Path) -> Result<Manifest, String> {
    let manifest_path = module_root.join("manifest.yaml");
    let bytes = std::fs::read(&manifest_path)
        .map_err(|e| format!("could not read {}: {e}", manifest_path.display()))?;
    parse_manifest(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn tmpdir(suffix: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "quire-rs-manifest-test-{}-{suffix}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).expect("mkdir");
        p
    }

    #[test]
    fn parses_artifact_types() {
        let yaml = br#"
name: spec-artifacts-iso
version: 0.1.0
description: ISO artifact templates
artifact_types:
- name: FR
  template_ref: templates/fr.md.j2
  frontmatter_schema_ref: schemas/fr-frontmatter.schema.json
- name: NFR
  template_ref: templates/nfr.md.j2
  frontmatter_schema_ref: schemas/nfr-frontmatter.schema.json
"#;
        let m = parse_manifest(yaml).expect("parse");
        assert_eq!(m.name.as_deref(), Some("spec-artifacts-iso"));
        assert_eq!(m.artifact_types.len(), 2);
        assert_eq!(m.artifact_types[0].name, "FR");
        assert_eq!(
            m.artifact_types[0].template_ref,
            PathBuf::from("templates/fr.md.j2")
        );
    }

    #[test]
    fn name_is_optional_when_yaml_omits_it() {
        let yaml = br#"
version: 0.1.0
artifact_types: []
"#;
        let m = parse_manifest(yaml).expect("parse");
        assert!(m.name.is_none());
        assert_eq!(m.version.as_deref(), Some("0.1.0"));
    }

    #[test]
    fn parses_object_types() {
        let yaml = br#"
name: ix-spec-objects
object_types:
- name: domain
  data_schema:
    type: object
"#;
        let m = parse_manifest(yaml).expect("parse");
        assert_eq!(m.object_types.len(), 1);
        assert_eq!(m.object_types[0].name, "domain");
        assert!(m.object_types[0].data_schema.is_some());
    }

    #[test]
    fn extras_are_captured_via_flatten() {
        let yaml = br#"
name: m
artifact_types:
- name: FR
  template_ref: t/fr.md.j2
  frontmatter_schema_ref: s/fr.json
  grammar_ref: iso-spec-core
  defaults:
    id_pattern: FR-{next:03d}
"#;
        let m = parse_manifest(yaml).expect("parse");
        let a = &m.artifact_types[0];
        assert!(a.extras.contains_key("grammar_ref"));
        assert!(a.extras.contains_key("defaults"));
    }

    #[test]
    fn load_manifest_reads_disk() {
        let root = tmpdir("disk");
        fs::write(
            root.join("manifest.yaml"),
            "name: disk-mod\nartifact_types: []\n",
        )
        .unwrap();
        let m = load_manifest(&root).expect("load");
        assert_eq!(m.name.as_deref(), Some("disk-mod"));
    }
}
