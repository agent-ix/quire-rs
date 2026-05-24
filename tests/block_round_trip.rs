//! End-to-end: parse a real-looking markdown artifact with stable
//! block IDs, apply a block patch via the public API, and assert
//! that only the patched block's bytes changed.
//!
//! Exercises FR-019 (stable block IDs), FR-021 (block edit API), and
//! FR-022 (writeback) together — the core v0.2 contract.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use quire_rs::{apply_block_patch, parse_document, replace_block, Registry};
use serde_json::json;

fn tmpdir(suffix: &str) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut p = std::env::temp_dir();
    p.push(format!(
        "quire-rs-block-rt-{}-{}-{suffix}",
        std::process::id(),
        n
    ));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).expect("mkdir");
    p
}

fn write_behavior_module(parent: &Path) -> PathBuf {
    let module = parent.join("behavior-module");
    fs::create_dir_all(module.join("schemas")).unwrap();
    fs::create_dir_all(module.join("templates")).unwrap();
    fs::write(
        module.join("manifest.yaml"),
        "name: bm\nartifact_types:\n- name: behavior\n  template_ref: templates/behavior.md.j2\n  frontmatter_schema_ref: schemas/behavior.schema.json\n",
    )
    .unwrap();
    fs::write(
        module.join("schemas/behavior.schema.json"),
        r#"{
            "type": "object",
            "required": ["block_id", "summary"],
            "properties": {
                "block_id": {"type": "string"},
                "summary":  {"type": "string", "minLength": 1}
            }
        }"#,
    )
    .unwrap();
    // Template emits `## Behavior {#<block_id>}\n<summary>\n` exactly,
    // so re-rendered bytes preserve the block_id verbatim.
    fs::write(
        module.join("templates/behavior.md.j2"),
        "## Behavior {% raw %}{#{% endraw %}{{ block_id }}{% raw %}}{% endraw %}\n{{ summary }}\n",
    )
    .unwrap();
    module
}

#[test]
fn block_patch_changes_only_target_block_bytes() {
    let root = tmpdir("only-target");
    let _module = write_behavior_module(&root);
    let registry = Registry::load_from(&[&root]).expect("load");

    let md = "\
---
id: FR-007
title: Sample
---
## Purpose
something
## Behavior {#blk-behavior-1}
old summary
## Acceptance {#blk-accept-1}
- AC-1: first
- AC-2: second
";
    let doc = parse_document(md);
    let current = json!({"block_id": "blk-behavior-1", "summary": "old summary"});
    let patch = json!({"summary": "patched summary"});

    let out = apply_block_patch(
        &registry,
        &doc,
        "blk-behavior-1",
        "behavior",
        &current,
        &patch,
    )
    .expect("apply_block_patch");

    // Frontmatter byte-identical.
    assert!(out.starts_with("---\nid: FR-007\ntitle: Sample\n---\n"));
    // Untouched sections byte-identical.
    assert!(out.contains("## Purpose\nsomething\n"));
    assert!(out.contains("## Acceptance {#blk-accept-1}\n- AC-1: first\n- AC-2: second\n"));
    // Target block carries the new content + preserves its block_id.
    assert!(out.contains("## Behavior {#blk-behavior-1}\npatched summary\n"));
    // Old content gone.
    assert!(!out.contains("old summary"));
}

#[test]
fn replace_block_renders_fresh_data_into_existing_block_bytes() {
    let root = tmpdir("replace");
    let _module = write_behavior_module(&root);
    let registry = Registry::load_from(&[&root]).expect("load");

    let md = "## Behavior {#blk-b}\nbefore\n## Other {#blk-o}\nother\n";
    let doc = parse_document(md);
    let new_data = json!({"block_id": "blk-b", "summary": "completely replaced"});

    let out = replace_block(&registry, &doc, "blk-b", "behavior", &new_data).expect("replace");
    assert!(out.contains("## Behavior {#blk-b}\ncompletely replaced\n"));
    assert!(out.contains("## Other {#blk-o}\nother\n"));
    assert!(!out.contains("before"));
}

#[test]
fn round_trip_is_idempotent_when_patch_is_noop() {
    let root = tmpdir("idempotent");
    let _module = write_behavior_module(&root);
    let registry = Registry::load_from(&[&root]).expect("load");

    let md = "## Behavior {#blk-b}\nstable\n## Tail {#blk-t}\ntail\n";
    let doc = parse_document(md);
    let current = json!({"block_id": "blk-b", "summary": "stable"});
    let empty_patch = json!({});

    let out = apply_block_patch(&registry, &doc, "blk-b", "behavior", &current, &empty_patch)
        .expect("apply");

    // Patched block re-renders identically (modulo template's trailing newline policy).
    assert!(out.contains("## Behavior {#blk-b}\nstable\n"));
    assert!(out.contains("## Tail {#blk-t}\ntail\n"));
}

#[test]
fn block_id_survives_parse_reparse_round_trip() {
    let root = tmpdir("reparse");
    let _module = write_behavior_module(&root);
    let registry = Registry::load_from(&[&root]).expect("load");

    let md = "## Behavior {#blk-b}\nv1\n## Other {#blk-o}\notherbody\n";
    let doc = parse_document(md);
    let current = json!({"block_id": "blk-b", "summary": "v1"});
    let patch = json!({"summary": "v2"});

    let out =
        apply_block_patch(&registry, &doc, "blk-b", "behavior", &current, &patch).expect("apply");
    // Reparse the result — block_ids must still be discoverable.
    let doc2 = parse_document(&out);
    let ids: Vec<Option<&str>> = doc2
        .sections
        .iter()
        .map(|s| s.block_id.as_deref())
        .collect();
    assert_eq!(ids, vec![Some("blk-b"), Some("blk-o")]);
}
