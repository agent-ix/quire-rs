//! Schema compilation step (FR-013 load-time work).
//!
//! For each archetype in a module manifest the loader resolves the
//! referenced JSON Schema file, parses it, and caches the compiled
//! validator in a [`CompiledArchetype`]. Per-validate and per-extract
//! paths never re-read disk after this. The render/templating feature
//! is removed — no template files are read or compiled.
//!
//! Cross-file `$ref` is rejected at compile time (FR-002-AC-7) by
//! using the `jsonschema` crate without a network resolver — the
//! default file resolver only walks the document supplied at compile.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use jsonschema::JSONSchema;
use serde_json::Value;

use crate::error::ArchetypeLoadFailure;
use crate::extract::dsl::ExtractionDsl;

/// Carry-over fields that the unified archetype shape (FR-031, ADR
/// 0003) retains but which have no DSL representation. Defaults are all
/// empty/absent so an archetype that declares none reads as `None`/`[]`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ArchetypeCarryOver {
    /// `defaults.id_pattern` — ID-allocation hint.
    pub id_pattern: Option<String>,
    /// `allowed_links` — relationship edge vocabulary this archetype
    /// permits, as verb → allowed target tokens (FR-040, supersedes the
    /// FR-031 flat-array per CR-001). The legacy array form normalizes
    /// to `{verb: ["*"]}`.
    pub allowed_links: crate::vocab::AllowedLinks,
    /// `roles` — capability tags this (object) archetype opts into;
    /// cross-domain edges target these instead of concrete type names
    /// (FR-040).
    pub roles: Vec<String>,
    /// `has_plugin` — whether a host-side plugin augments this archetype.
    pub has_plugin: bool,
    /// `grammar_ref` — grammar this archetype's body conforms to.
    pub grammar_ref: Option<String>,
}

/// A unified compiled archetype (FR-031, ADR 0003): one shape that may
/// carry an optional frontmatter schema, an optional `body_extraction`
/// DSL, an optional `data_schema`, and the carry-over fields.
/// Validatability/extractability are *derived* from which parts are
/// present, not from a declared `artifact_type` / `object_type` kind.
/// The render/templating feature is removed — there is no template
/// field or renderability concept. Cloning is `Arc`-cheap.
pub struct CompiledArchetype {
    /// Registered archetype name.
    pub name: String,
    /// Module the archetype belongs to.
    pub module: String,
    /// Primary schema document returned by `schema_for` (FR-003) —
    /// byte-exact. This is the frontmatter schema when present, else the
    /// data schema, else an empty (permissive) object. Distinct
    /// frontmatter/data validators are available via the accessors.
    pub raw_schema: Arc<Value>,
    /// Primary compiled validator paired with [`Self::raw_schema`].
    pub validator: Arc<JSONSchema>,
    /// Compiled frontmatter-schema validator (FR-031), `None` when the
    /// archetype declares no `frontmatter_schema_ref`.
    pub frontmatter_schema: Option<Arc<Value>>,
    pub frontmatter_validator: Option<Arc<JSONSchema>>,
    /// Compiled `data_schema` validator over the *extracted record*
    /// (FR-031-AC-4) — distinct from the frontmatter validator.
    pub data_schema: Option<Arc<Value>>,
    pub data_validator: Option<Arc<JSONSchema>>,
    /// Parsed `body_extraction` DSL (FR-011), `None` when absent.
    /// Drives both `extract()` and `validate_document()` (FR-032).
    pub body_extraction: Option<ExtractionDsl>,
    /// Carry-over fields with no DSL representation (FR-031-AC-3).
    pub carry_over: ArchetypeCarryOver,
}

impl CompiledArchetype {
    /// `true` if this archetype can be validated — it has a frontmatter
    /// schema and/or a `body_extraction` contract. (An archetype with
    /// neither still trivially "validates", so this reports whether any
    /// substantive contract exists.)
    pub fn is_validatable(&self) -> bool {
        self.frontmatter_validator.is_some()
            || self.data_validator.is_some()
            || self.body_extraction.is_some()
    }

    /// Parsed body-extraction DSL, if any.
    pub fn body_extraction(&self) -> Option<&ExtractionDsl> {
        self.body_extraction.as_ref()
    }

    /// Compiled frontmatter-schema validator (FR-031-AC-4), if declared.
    pub fn frontmatter_validator(&self) -> Option<&JSONSchema> {
        self.frontmatter_validator.as_deref()
    }

    /// Compiled `data_schema` validator (FR-031-AC-4), if declared.
    pub fn data_validator(&self) -> Option<&JSONSchema> {
        self.data_validator.as_deref()
    }

    /// `defaults.id_pattern`, if declared (FR-031-AC-3).
    pub fn id_pattern(&self) -> Option<&str> {
        self.carry_over.id_pattern.as_deref()
    }

    /// `allowed_links` as verb → allowed target tokens, possibly empty
    /// (FR-040-AC-4; supersedes FR-031-AC-3's flat array per CR-001).
    pub fn allowed_links(&self) -> &crate::vocab::AllowedLinks {
        &self.carry_over.allowed_links
    }

    /// `roles` this object archetype opts into, possibly empty
    /// (FR-040-AC-5).
    pub fn roles(&self) -> &[String] {
        &self.carry_over.roles
    }

    /// `has_plugin` flag (FR-031-AC-3).
    pub fn has_plugin(&self) -> bool {
        self.carry_over.has_plugin
    }

    /// `grammar_ref`, if declared (FR-031-AC-3).
    pub fn grammar_ref(&self) -> Option<&str> {
        self.carry_over.grammar_ref.as_deref()
    }
}

impl std::fmt::Debug for CompiledArchetype {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompiledArchetype")
            .field("name", &self.name)
            .field("module", &self.module)
            .finish_non_exhaustive()
    }
}

/// Read + parse a JSON schema file. BOM-strip per FR-013 notes.
pub fn read_schema(path: &Path) -> Result<Value, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let text = String::from_utf8(bytes)
        .map_err(|e| format!("schema {} is not utf-8: {e}", path.display()))?;
    let stripped = text.strip_prefix('\u{FEFF}').unwrap_or(&text);
    serde_json::from_str::<Value>(stripped)
        .map_err(|e| format!("schema {} is not valid JSON: {e}", path.display()))
}

/// Compile a parsed JSON Schema document into a runtime validator.
pub fn compile_schema(schema: &Value) -> Result<JSONSchema, String> {
    JSONSchema::options()
        .compile(schema)
        .map_err(|e| format!("schema compile error: {e}"))
}

/// Convert a (module, archetype, path, reason) tuple into a
/// `ArchetypeLoadFailure` ready to be aggregated by the loader.
pub fn failure(
    module: &str,
    archetype: &str,
    path: PathBuf,
    reason: String,
) -> ArchetypeLoadFailure {
    ArchetypeLoadFailure {
        module: module.to_string(),
        archetype: archetype.to_string(),
        path,
        reason,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    fn tmppath(suffix: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "quire-rs-compile-test-{}-{suffix}",
            std::process::id()
        ));
        p
    }

    #[test]
    fn read_schema_strips_bom() {
        let path = tmppath("bom.json");
        let mut bytes: Vec<u8> = b"\xEF\xBB\xBF".to_vec();
        bytes.extend_from_slice(br#"{"type": "object"}"#);
        fs::write(&path, &bytes).unwrap();
        let v = read_schema(&path).unwrap();
        assert_eq!(v, json!({"type": "object"}));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn compile_schema_accepts_object_constraint() {
        let schema = json!({
            "type": "object",
            "required": ["id"],
            "properties": {"id": {"type": "string"}}
        });
        let validator = compile_schema(&schema).expect("compile");
        let good = json!({"id": "FR-001"});
        let bad = json!({});
        assert!(validator.is_valid(&good));
        assert!(!validator.is_valid(&bad));
    }

    #[test]
    fn compile_schema_rejects_malformed() {
        let schema = json!({"type": 99}); // type must be string or array
        assert!(compile_schema(&schema).is_err());
    }
}
