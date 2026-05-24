//! Render dispatch — the engine's runtime entry point (FR-001).
//!
//! `render(archetype, data)` validates `data` against the archetype's
//! pre-compiled JSON Schema, then renders the validated data through
//! the archetype's pre-parsed MiniJinja template via the shared
//! strict-undefined env from the [`Registry`](crate::Registry).
//!
//! `render_by_name(registry, name, data)` is the registry-aware
//! convenience: resolves `name` then calls `render`.

pub mod env;

use serde_json::Value;

use crate::error::QuireError;
use crate::loader::compile::CompiledArchetype;
use crate::registry::Registry;
use crate::validate::validate;

/// Validate + render an archetype against `data`. The validator and
/// template are pre-compiled at load time so this path does no disk
/// reads (FR-013-AC-5) and only the schema-check + template-render
/// work happens per call.
pub fn render(
    registry: &Registry,
    archetype: &CompiledArchetype,
    data: &Value,
) -> Result<String, QuireError> {
    #[cfg(feature = "tracing")]
    let _span = tracing::debug_span!("quire_rs::render", archetype = %archetype.name).entered();
    validate(archetype, data)?;

    let template_name =
        archetype
            .template_name
            .as_ref()
            .ok_or_else(|| QuireError::TemplateError {
                archetype: archetype.name.clone(),
                template_path: archetype.template_path.clone().unwrap_or_default(),
                message: "archetype has no template (object_type — use validate-only)".to_string(),
            })?;

    let template =
        registry
            .env()
            .get_template(template_name)
            .map_err(|e| QuireError::TemplateError {
                archetype: archetype.name.clone(),
                template_path: archetype.template_path.clone().unwrap_or_default(),
                message: format!("template not registered: {e}"),
            })?;

    template
        .render(data)
        .map_err(|e| QuireError::TemplateError {
            archetype: archetype.name.clone(),
            template_path: archetype.template_path.clone().unwrap_or_default(),
            message: e.to_string(),
        })
}

/// Resolve `name` against `registry`, then [`render`].
pub fn render_by_name(registry: &Registry, name: &str, data: &Value) -> Result<String, QuireError> {
    let archetype = registry
        .archetype(name)
        .ok_or_else(|| QuireError::UnknownArchetype {
            name: name.to_string(),
        })?;
    render(registry, archetype, data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use std::thread;

    fn tmpdir(suffix: &str) -> PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let mut p = std::env::temp_dir();
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        p.push(format!(
            "quire-rs-render-test-{}-{}-{suffix}",
            std::process::id(),
            n
        ));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).expect("mkdir");
        p
    }

    fn write_render_module(root: &Path) {
        fs::create_dir_all(root.join("schemas")).unwrap();
        fs::create_dir_all(root.join("templates")).unwrap();
        fs::write(
            root.join("manifest.yaml"),
            "name: m\nartifact_types:\n- name: fr\n  template_ref: templates/fr.md.j2\n  frontmatter_schema_ref: schemas/fr.schema.json\n",
        )
        .unwrap();
        fs::write(
            root.join("schemas/fr.schema.json"),
            r#"{
                "type": "object",
                "required": ["id", "title"],
                "properties": {
                    "id":    {"type": "string", "pattern": "^FR-"},
                    "title": {"type": "string", "minLength": 1}
                }
            }"#,
        )
        .unwrap();
        fs::write(
            root.join("templates/fr.md.j2"),
            "# {{ id }}\n\n{{ title }}\n",
        )
        .unwrap();
    }

    fn build_registry() -> (PathBuf, Registry) {
        let parent = tmpdir("render");
        write_render_module(&parent.join("m"));
        let r = Registry::load_from(&[&parent]).expect("ok");
        (parent, r)
    }

    #[test]
    fn render_by_name_happy_path() {
        let (_p, r) = build_registry();
        let out = render_by_name(&r, "fr", &json!({"id": "FR-001", "title": "Hi"})).unwrap();
        assert!(out.contains("FR-001"));
        assert!(out.contains("Hi"));
    }

    // FR-001-AC-2
    #[test]
    fn render_by_name_unknown_returns_unknown_archetype() {
        let (_p, r) = build_registry();
        let err = render_by_name(&r, "nope", &json!({})).expect_err("unknown");
        assert!(matches!(err, QuireError::UnknownArchetype { .. }));
    }

    // FR-001-AC-3
    #[test]
    fn render_missing_required_returns_schema_violation() {
        let (_p, r) = build_registry();
        let err = render_by_name(&r, "fr", &json!({"id": "FR-001"})).expect_err("violation");
        let s = err.to_string();
        assert!(matches!(err, QuireError::SchemaViolation { .. }), "{s}");
        assert!(s.contains("title") || s.contains("required"), "{s}");
    }

    // FR-001-AC-4: 64-thread concurrent render produces byte-identical output.
    #[test]
    fn render_is_thread_safe_under_concurrency() {
        let (_p, r) = build_registry();
        let r = Arc::new(r);
        let data = Arc::new(json!({"id": "FR-001", "title": "Concurrent"}));
        let baseline = render_by_name(&r, "fr", &data).unwrap();
        let handles: Vec<_> = (0..64)
            .map(|_| {
                let r = Arc::clone(&r);
                let data = Arc::clone(&data);
                let baseline = baseline.clone();
                thread::spawn(move || {
                    let got = render_by_name(&r, "fr", &data).unwrap();
                    assert_eq!(got, baseline, "non-deterministic render");
                })
            })
            .collect();
        for h in handles {
            h.join().expect("thread");
        }
    }

    // FR-001-AC-5 / TC-005: adding an archetype is a data-only change.
    // (Re-load registry against a directory with one extra archetype;
    // no source change needed.)
    #[test]
    fn adding_new_archetype_requires_no_source_change() {
        let parent = tmpdir("add");
        let m = parent.join("m");
        fs::create_dir_all(m.join("schemas")).unwrap();
        fs::create_dir_all(m.join("templates")).unwrap();
        fs::write(
            m.join("manifest.yaml"),
            r#"
name: m
artifact_types:
- name: fr
  template_ref: templates/fr.md.j2
  frontmatter_schema_ref: schemas/fr.schema.json
- name: nfr
  template_ref: templates/nfr.md.j2
  frontmatter_schema_ref: schemas/nfr.schema.json
"#,
        )
        .unwrap();
        fs::write(
            m.join("schemas/fr.schema.json"),
            r#"{"type":"object","required":["id"],"properties":{"id":{"type":"string"}}}"#,
        )
        .unwrap();
        fs::write(
            m.join("schemas/nfr.schema.json"),
            r#"{"type":"object","required":["id","priority"],"properties":{"id":{"type":"string"},"priority":{"enum":["P0","P1","P2"]}}}"#,
        )
        .unwrap();
        fs::write(m.join("templates/fr.md.j2"), "{{ id }}\n").unwrap();
        fs::write(m.join("templates/nfr.md.j2"), "{{ id }} ({{ priority }})\n").unwrap();
        let r = Registry::load_from(&[&parent]).expect("ok");
        let out = render_by_name(&r, "nfr", &json!({"id": "NFR-1", "priority": "P0"})).unwrap();
        assert!(out.contains("NFR-1"));
        assert!(out.contains("P0"));
    }
}
