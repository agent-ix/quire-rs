//! Schema + template compilation step (FR-013 load-time work).
//!
//! For each archetype in a module manifest the loader resolves the
//! referenced JSON Schema file and (for `artifact_types`) the
//! MiniJinja template file, parses both, and caches the parsed forms
//! in a [`CompiledArchetype`]. Per-render and per-extract paths never
//! re-read disk after this.
//!
//! Cross-file `$ref` is rejected at compile time (FR-002-AC-7) by
//! using the `jsonschema` crate without a network resolver — the
//! default file resolver only walks the document supplied at compile.
//!
//! `{% include %}` in templates is rejected at parse time (FR-004) by
//! configuring the MiniJinja env with includes disabled before
//! `add_template` runs.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use jsonschema::JSONSchema;
use minijinja::Environment;
use serde_json::Value;

use crate::error::ArchetypeLoadFailure;

/// A schema + (optional) template that the loader has fully parsed and
/// cached. Cloning is `Arc`-cheap so consumers can share without
/// re-parsing.
pub struct CompiledArchetype {
    /// Registered archetype name.
    pub name: String,
    /// Module the archetype belongs to.
    pub module: String,
    /// Verbatim raw schema document — `schema_for` returns it byte-exact
    /// per FR-003.
    pub raw_schema: Arc<Value>,
    /// Compiled JSON-Schema validator.
    pub validator: Arc<JSONSchema>,
    /// On-disk template path. `None` for object_types (data-only).
    pub template_path: Option<PathBuf>,
    /// Registered template name in the shared MiniJinja env. `None`
    /// for object_types.
    pub template_name: Option<String>,
}

impl CompiledArchetype {
    /// `true` if this archetype has a renderable template (i.e. it's
    /// an `artifact_type`, not an `object_type`).
    pub fn is_renderable(&self) -> bool {
        self.template_name.is_some()
    }
}

impl std::fmt::Debug for CompiledArchetype {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompiledArchetype")
            .field("name", &self.name)
            .field("module", &self.module)
            .field("template_name", &self.template_name)
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

/// Add `template_source` to `env` under `template_name`, returning a
/// neutral error string on parse failure.
///
/// `env` must already be configured with the FR-004 strict settings
/// (caller's responsibility — see `render::env::build_env`).
pub fn register_template(
    env: &mut Environment<'static>,
    template_name: String,
    template_source: String,
) -> Result<(), String> {
    env.add_template_owned(template_name, template_source)
        .map_err(|e| format!("template parse error: {e}"))
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

    #[test]
    fn register_template_parses_minijinja_source() {
        let mut env = Environment::new();
        env.set_undefined_behavior(minijinja::UndefinedBehavior::Strict);
        register_template(
            &mut env,
            "fr".to_string(),
            "id: {{ id }}\ntitle: {{ title }}\n".to_string(),
        )
        .expect("register");
        let rendered = env
            .get_template("fr")
            .unwrap()
            .render(minijinja::context!(id => "FR-001", title => "Hi"))
            .unwrap();
        assert!(rendered.contains("FR-001"));
        assert!(rendered.contains("Hi"));
    }

    #[test]
    fn register_template_surfaces_parse_error() {
        let mut env = Environment::new();
        let err = register_template(&mut env, "broken".to_string(), "{% if".to_string());
        assert!(err.is_err());
    }
}
