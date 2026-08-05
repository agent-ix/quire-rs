//! `manifest.yaml` parser (FR-013 manifest section).
//!
//! Each module root contains a `manifest.yaml` enumerating its
//! archetypes. Only the fields we actually consume to load a module are
//! deserialized here — the rest (grammars, lint_rules, defaults) are
//! captured via `serde(flatten)` into a free-form map so authors can
//! evolve the manifest without breaking the loader.
//!
//! After the FR-031 unification (ADR 0003) there is **one** archetype
//! shape. A manifest entry MAY carry, all optional except `name`:
//!
//! - `frontmatter_schema_ref` — JSON Schema over the document frontmatter.
//! - `data_schema` — JSON Schema over the *extracted record*.
//! - `body_extraction` — the locator DSL (FR-011), driving both extract
//!   and `validate_document` (FR-032).
//! - carry-over fields: `defaults.id_pattern`, `allowed_links`,
//!   `has_plugin`, `grammar_ref`.
//!
//! Both `artifact_types:` and `object_types:` manifest sections (plus a
//! new `archetypes:` section) deserialize into the same [`Archetype`]
//! entry — the kind distinction is gone from the compiled model. The
//! retired fields `required_sections`, `variants`, and `template_ref`
//! (render removed) are **rejected** at load time (no
//! backward-compatibility layer, ADR 0003 / FR-031 / FR-035).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::vocab::{EdgeTypeDef, RoleDef};

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
    pub artifact_types: Vec<Archetype>,
    #[serde(default)]
    pub object_types: Vec<Archetype>,
    #[serde(default)]
    pub archetypes: Vec<Archetype>,
    /// Declarative advisory lint rules (FR-036). Previously this key
    /// was inert; it is now typed — a malformed rule fails manifest
    /// parse like any other shape error.
    #[serde(default)]
    pub lint_rules: Vec<crate::lint::LintRule>,
    /// Mergeable edge-type registry (FR-040): verb → {description,
    /// category, optional inverse}. Merged across modules first-wins.
    #[serde(default)]
    pub edge_types: BTreeMap<String, EdgeTypeDef>,
    /// Mergeable role registry (FR-040): role name → {description}.
    /// Object types opt into roles via their `roles:` list; cross-domain
    /// `allowed_links` target roles instead of concrete type names.
    #[serde(default)]
    pub roles: BTreeMap<String, RoleDef>,
    /// Mergeable concrete-term lexicon (FR-043): term → {definition,
    /// optional category}. Merged across modules first-wins; the EARS
    /// object-aware vague-response check (FR-042) consumes the merged keys
    /// as accepted concrete objects, so the engine carries no hardcoded list.
    #[serde(default)]
    pub lexicon: BTreeMap<String, crate::vocab::LexiconTermDef>,
    /// Mergeable per-check grammar severity registry (FR-048):
    /// `<grammar>:<check>` → `off` | `warning` | `error`. Merged across
    /// modules first-wins; the grammar framework keys each emitted finding
    /// against the merged map, so a deployment can promote or suppress one
    /// check without a code change. A malformed entry (unknown level,
    /// non-string key) fails manifest parse like any other shape error.
    #[serde(default)]
    pub grammar_severity: BTreeMap<String, crate::grammar::GrammarSeverityLevel>,
    /// Mergeable observable-result verb registry (FR-047): verb →
    /// {definition, optional category}. Merged across modules first-wins and
    /// layered **over** the engine's built-in defaults, which stay at lowest
    /// precedence. The `ac` grammar's `no-observable-outcome` check consumes
    /// the merged keys, so the vocabulary is module data (ADR 0009).
    #[serde(default)]
    pub observable_verbs: BTreeMap<String, crate::vocab::ObservableVerbDef>,
    /// Declarative traceability model (FR-050): which documents mint trace
    /// ids, which columns reference them, the status vocabulary, and the
    /// trace-tag grammar. Absent → the model is undeclared; malformed → module
    /// load fails like any other manifest shape error.
    #[serde(default)]
    pub traceability: crate::traceability::TraceabilityModel,
}

impl Manifest {
    /// Iterate every declared archetype across all manifest sections in
    /// declaration order (`artifact_types`, then `object_types`, then
    /// `archetypes`). After FR-031 they compile through one path.
    pub fn all_archetypes(&self) -> impl Iterator<Item = &Archetype> {
        self.artifact_types
            .iter()
            .chain(self.object_types.iter())
            .chain(self.archetypes.iter())
    }
}

/// One unified archetype entry (FR-031). Every field except `name` is
/// optional; validatability/extractability is derived downstream from
/// which parts are present.
///
/// `body_extraction` is validated at load time (FR-011-AC-6/7/8) and
/// the retired `required_sections`/`variants`/`template_ref` fields are
/// rejected (`retired_field`) — each surfaces as `ArchetypeLoadFailure`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Archetype {
    pub name: String,
    #[serde(default)]
    pub frontmatter_schema_ref: Option<PathBuf>,
    #[serde(default)]
    pub data_schema: Option<Value>,
    #[serde(default)]
    pub body_extraction: Option<crate::extract::dsl::ExtractionDsl>,
    /// Free-form passthrough for carry-over fields (`grammar_ref`,
    /// `allowed_links`, `has_plugin`, `defaults`) and the retired fields
    /// the loader rejects.
    #[serde(flatten)]
    pub extras: Map<String, Value>,
}

impl Archetype {
    /// Reject the retired `required_sections` / `variants` / `template_ref`
    /// fields with a caller-facing reason (no backward-compatibility layer
    /// — ADR 0003, render removed). Returns the offending field name on
    /// rejection.
    pub fn retired_field(&self) -> Option<&'static str> {
        if self.extras.contains_key("required_sections") {
            Some("required_sections")
        } else if self.extras.contains_key("variants") {
            Some("variants")
        } else if self.extras.contains_key("template_ref") {
            Some("template_ref")
        } else {
            None
        }
    }

    /// Extract the carry-over fields from `extras` (FR-031-AC-3).
    pub fn carry_over(&self) -> crate::loader::compile::ArchetypeCarryOver {
        let id_pattern = self
            .extras
            .get("defaults")
            .and_then(|d| d.get("id_pattern"))
            .and_then(|v| v.as_str())
            .map(str::to_string);
        // `allowed_links` accepts the legacy flat-array form (normalized
        // to `{verb: ["*"]}`) or the FR-040 map form (verb → targets).
        let allowed_links = crate::vocab::normalize_allowed_links(self.extras.get("allowed_links"));
        // `roles` — capability tags this object type opts into (FR-040).
        let roles = self
            .extras
            .get("roles")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        let has_plugin = self
            .extras
            .get("has_plugin")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let grammar_ref = self
            .extras
            .get("grammar_ref")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        crate::loader::compile::ArchetypeCarryOver {
            id_pattern,
            allowed_links,
            roles,
            has_plugin,
            grammar_ref,
        }
    }
}

/// Parse a `manifest.yaml` document from `bytes`.
///
/// Returns the typed [`Manifest`] or a `String` error message suitable
/// for wrapping in `QuireError::ManifestError`.
pub fn parse_manifest(bytes: &[u8]) -> Result<Manifest, String> {
    let manifest = serde_yaml::from_slice::<Manifest>(bytes).map_err(|e| e.to_string())?;
    check_grammar_severity_keys(&manifest)?;
    // FR-050-AC-2: an incoherent `traceability:` declaration is a shape error.
    manifest.traceability.validate()?;
    Ok(manifest)
}

/// Reject a malformed `grammar_severity` key (FR-048-AC-8). The level side is
/// already contracted by [`crate::grammar::GrammarSeverityLevel`]'s enum
/// deserializer; the key side is not, because YAML stringifies a non-string
/// scalar key (`12:`) instead of failing. A well-formed key is exactly
/// `<grammar>:<check>` — two non-empty, whitespace-free halves.
fn check_grammar_severity_keys(manifest: &Manifest) -> Result<(), String> {
    for key in manifest.grammar_severity.keys() {
        if !crate::grammar::is_severity_key(key) {
            return Err(format!(
                "grammar_severity key '{key}' is malformed: expected '<grammar>:<check>'"
            ));
        }
    }
    Ok(())
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
description: ISO artifacts
artifact_types:
- name: FR
  frontmatter_schema_ref: schemas/fr-frontmatter.schema.json
- name: NFR
  frontmatter_schema_ref: schemas/nfr-frontmatter.schema.json
"#;
        let m = parse_manifest(yaml).expect("parse");
        assert_eq!(m.name.as_deref(), Some("spec-artifacts-iso"));
        assert_eq!(m.artifact_types.len(), 2);
        assert_eq!(m.artifact_types[0].name, "FR");
        assert_eq!(
            m.artifact_types[0].frontmatter_schema_ref,
            Some(PathBuf::from("schemas/fr-frontmatter.schema.json"))
        );
    }

    #[test]
    fn template_ref_is_a_retired_field() {
        // template_ref now lands in `extras` (flatten) and is rejected
        // by retired_field — the render feature is removed.
        let yaml = br#"
name: m
artifact_types:
- name: FR
  template_ref: templates/fr.md.j2
  frontmatter_schema_ref: schemas/fr.json
"#;
        let m = parse_manifest(yaml).expect("parse");
        let a = &m.artifact_types[0];
        assert!(a.extras.contains_key("template_ref"));
        assert_eq!(a.retired_field(), Some("template_ref"));
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
