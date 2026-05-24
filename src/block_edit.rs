//! Block edit API (FR-021): patch / replace one block's data, then
//! writeback into canonical markdown.
//!
//! Per INPUT.md the LLM-facing edit flow is:
//!
//! 1. caller gets the block-type schema (`Registry::schema_for`)
//! 2. caller emits a JSON merge-patch (or a full replacement) against
//!    that schema
//! 3. [`apply_block_patch`] / [`replace_block`] merges or replaces,
//!    re-validates against the block-type schema, re-renders the
//!    block via the block-type template, and splices the new bytes
//!    back into the canonical markdown via
//!    [`crate::writeback::update_block`]
//!
//! Returns the full updated markdown string. Frontmatter + untouched
//! blocks stay byte-identical.
//!
//! v0.2 scope: block_type maps 1:1 to archetype. The caller supplies
//! `(block_id, block_type, current_data)` because v0.2 has no
//! manifest-level mechanism for declaring per-block-type metadata
//! inside a heading attribute (that would be a v0.3 spec extension).

use serde_json::Value;

use crate::ast::QuireDocument;
use crate::error::QuireError;
use crate::merge::deep_merge;
use crate::registry::Registry;
use crate::render::render_block;
use crate::validate::validate_block;
use crate::writeback::update_block;

/// Merge `patch` onto `current_data`, validate against the block-type
/// schema, re-render via the block-type template, and writeback the
/// new bytes into `doc` under `block_id`. Returns the full updated
/// markdown.
///
/// Errors:
/// - `UnknownArchetype` when `block_type` is not registered.
/// - `SchemaViolation` when the merged data fails validation.
/// - `MissingField` when `block_id` is not present in `doc`.
/// - `TemplateError` when the block-type template fails to render.
pub fn apply_block_patch(
    registry: &Registry,
    doc: &QuireDocument,
    block_id: &str,
    block_type: &str,
    current_data: &Value,
    patch: &Value,
) -> Result<String, QuireError> {
    let merged = deep_merge(current_data, patch);
    render_and_splice(registry, doc, block_id, block_type, &merged)
}

/// Full-replace variant: validate `new_data` against the block-type
/// schema, render via its template, splice into `doc` under
/// `block_id`. Returns the full updated markdown.
pub fn replace_block(
    registry: &Registry,
    doc: &QuireDocument,
    block_id: &str,
    block_type: &str,
    new_data: &Value,
) -> Result<String, QuireError> {
    render_and_splice(registry, doc, block_id, block_type, new_data)
}

fn render_and_splice(
    registry: &Registry,
    doc: &QuireDocument,
    block_id: &str,
    block_type: &str,
    data: &Value,
) -> Result<String, QuireError> {
    let bt = registry
        .block_type(block_type)
        .ok_or_else(|| QuireError::UnknownArchetype {
            name: block_type.to_string(),
        })?;
    validate_block(bt, data)?;
    let rendered = render_block(registry, block_type, data)?;
    update_block(doc, block_id, &rendered.markdown)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_document;
    use serde_json::json;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    fn tmpdir(suffix: &str) -> PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let mut p = std::env::temp_dir();
        p.push(format!(
            "quire-rs-block-edit-test-{}-{}-{suffix}",
            std::process::id(),
            n
        ));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).expect("mkdir");
        p
    }

    /// `parent` is the search root the loader will walk; the module
    /// itself lives in `parent/callout-module/`.
    fn write_callout_module(parent: &Path) -> PathBuf {
        let module = parent.join("callout-module");
        fs::create_dir_all(module.join("schemas")).unwrap();
        fs::create_dir_all(module.join("templates")).unwrap();
        fs::write(
            module.join("manifest.yaml"),
            "name: m\nartifact_types:\n- name: callout\n  template_ref: templates/callout.md.j2\n  frontmatter_schema_ref: schemas/callout.schema.json\n",
        )
        .unwrap();
        fs::write(
            module.join("schemas/callout.schema.json"),
            r#"{
                "type": "object",
                "required": ["kind", "text"],
                "properties": {
                    "kind": {"type": "string", "enum": ["note", "warning"]},
                    "text": {"type": "string", "minLength": 1}
                }
            }"#,
        )
        .unwrap();
        fs::write(
            module.join("templates/callout.md.j2"),
            "## {{ kind | capitalize }} {% raw %}{#blk-7af2}{% endraw %}\n{{ text }}\n",
        )
        .unwrap();
        module
    }

    #[test]
    fn apply_block_patch_merges_renders_and_splices() {
        let root = tmpdir("merge");
        let _module = write_callout_module(&root);
        let registry = Registry::load_from(&[&root]).expect("load");

        let md = "## Note {#blk-7af2}\nold body\n## Other {#blk-9c14}\nother body\n";
        let doc = parse_document(md);
        let current = json!({"kind": "note", "text": "old body"});
        let patch = json!({"text": "new body"});

        let out = apply_block_patch(&registry, &doc, "blk-7af2", "callout", &current, &patch)
            .expect("apply");

        assert!(out.contains("## Note {#blk-7af2}\nnew body\n"));
        assert!(out.contains("## Other {#blk-9c14}\nother body\n"));
    }

    #[test]
    fn replace_block_full_replaces_and_renders() {
        let root = tmpdir("replace");
        let _module = write_callout_module(&root);
        let registry = Registry::load_from(&[&root]).expect("load");

        let md = "## Note {#blk-7af2}\nold\n## Other {#blk-9c14}\nb\n";
        let doc = parse_document(md);
        let new_data = json!({"kind": "warning", "text": "danger"});

        let out =
            replace_block(&registry, &doc, "blk-7af2", "callout", &new_data).expect("replace");

        assert!(out.contains("## Warning {#blk-7af2}\ndanger\n"));
        assert!(out.contains("## Other {#blk-9c14}\nb\n"));
    }

    #[test]
    fn apply_block_patch_invalid_returns_schema_violation() {
        let root = tmpdir("invalid");
        let _module = write_callout_module(&root);
        let registry = Registry::load_from(&[&root]).expect("load");

        let md = "## Note {#blk-7af2}\nold\n";
        let doc = parse_document(md);
        let current = json!({"kind": "note", "text": "old"});
        let patch = json!({"kind": "shouty"}); // not in enum

        let err = apply_block_patch(&registry, &doc, "blk-7af2", "callout", &current, &patch)
            .expect_err("must fail");
        assert!(matches!(err, QuireError::SchemaViolation { .. }));
    }

    #[test]
    fn apply_block_patch_unknown_block_type_returns_unknown_archetype() {
        let root = tmpdir("unk-type");
        let _module = write_callout_module(&root);
        let registry = Registry::load_from(&[&root]).expect("load");
        let doc = parse_document("## Note {#blk-7af2}\nold\n");
        let err = apply_block_patch(&registry, &doc, "blk-7af2", "nope", &json!({}), &json!({}))
            .expect_err("must fail");
        assert!(matches!(err, QuireError::UnknownArchetype { .. }));
    }

    #[test]
    fn apply_block_patch_unknown_block_id_returns_missing_field() {
        let root = tmpdir("unk-id");
        let _module = write_callout_module(&root);
        let registry = Registry::load_from(&[&root]).expect("load");
        let doc = parse_document("## Note {#blk-7af2}\nold\n");
        let current = json!({"kind": "note", "text": "old"});
        let patch = json!({"text": "new"});
        let err = apply_block_patch(&registry, &doc, "blk-MISSING", "callout", &current, &patch)
            .expect_err("must fail");
        assert!(matches!(err, QuireError::MissingField { .. }));
    }

    // LLM-flow integration: the rendered bytes from apply_block_patch
    // must equal what running the template against the patched data
    // directly produces.
    #[test]
    fn llm_flow_rendered_bytes_match_direct_template() {
        let root = tmpdir("llm");
        let _module = write_callout_module(&root);
        let registry = Registry::load_from(&[&root]).expect("load");

        let md = "## Note {#blk-7af2}\nold\n## Trailing {#blk-tail}\ntail\n";
        let doc = parse_document(md);
        let current = json!({"kind": "note", "text": "old"});
        let patch = json!({"kind": "warning", "text": "wake up"});

        let out = apply_block_patch(&registry, &doc, "blk-7af2", "callout", &current, &patch)
            .expect("apply");

        // Render the merged data directly via the registry.
        let merged = deep_merge(&current, &patch);
        let direct = render_block(&registry, "callout", &merged).expect("render");
        // The bytes spliced in must be exactly the direct render.
        assert!(out.starts_with(&direct.markdown));
        // And the trailing block survives byte-identical.
        assert!(out.contains("## Trailing {#blk-tail}\ntail\n"));
    }
}
