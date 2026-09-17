//! FR-069 semantic module contract at load (TC-1599..TC-1609, TC-1633,
//! TC-1645, TC-1646, TC-1848, TC-1849, TC-1864, TC-1866). Plan-003 Task-016.
//!
//! Every case starts from the quoin `module-ok` fixture (pinned under
//! `tests/fixtures/semantic/quoin/module-ok`), copied into a temp dir and
//! mutated in place; the fixture itself is never edited.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use ix_trace_rs::trace;
use quire_rs::semantic::{compile_module_schema, SemanticModule};
use quire_rs::{extract_filament_core, FilamentExtractionInput, Registry};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tempfile::TempDir;

type Mutate = Box<dyn Fn(&mut serde_yaml::Value, &Path)>;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/semantic/quoin/module-ok")
}

fn golden() -> Value {
    serde_json::from_slice(
        &fs::read(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/semantic/quoin/mapping/config-version.expected.json"),
        )
        .unwrap(),
    )
    .unwrap()
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn digest_of(path: &Path) -> String {
    format!("sha256:{:x}", Sha256::digest(fs::read(path).unwrap()))
}

/// A mutable copy of module-ok. `mutate` edits the manifest (as YAML value)
/// and may touch files under `root`.
fn module(
    tmp: &TempDir,
    name: &str,
    mutate: impl FnOnce(&mut serde_yaml::Value, &Path),
) -> PathBuf {
    let root = tmp.path().join(name);
    copy_dir(&fixture(), &root);
    let manifest_path = root.join("manifest.yaml");
    let mut manifest: serde_yaml::Value =
        serde_yaml::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    manifest["name"] = serde_yaml::Value::String(name.to_string());
    mutate(&mut manifest, &root);
    fs::write(&manifest_path, serde_yaml::to_string(&manifest).unwrap()).unwrap();
    root
}

fn semantic(m: &mut serde_yaml::Value) -> &mut serde_yaml::Mapping {
    m["semantic"].as_mapping_mut().unwrap()
}

fn entity_schema_ref(m: &mut serde_yaml::Value) -> &mut serde_yaml::Mapping {
    m["object_types"][0]["data_schema"]
        .as_mapping_mut()
        .unwrap()
}

/// Rewrite Entity.json (optionally editing it) and refresh the manifest digest.
fn edit_entity(m: &mut serde_yaml::Value, root: &Path, edit: impl FnOnce(&mut Value)) {
    let file = root.join("schemas/Entity.json");
    let mut schema: Value = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
    edit(&mut schema);
    fs::write(&file, serde_json::to_vec(&schema).unwrap()).unwrap();
    entity_schema_ref(m).insert("digest".into(), digest_of(&file).into());
}

/// Point the fixture at another package: `$id` and digest follow.
fn retarget(m: &mut serde_yaml::Value, root: &Path, package: &str) {
    edit_entity(m, root, |schema| {
        schema["$id"] = json!(format!(
            "https://schemas.agent-ix.org/{package}/0.1.0/Entity.json"
        ));
    });
    semantic(m).insert("package".into(), package.into());
}

fn load(root: &Path) -> Registry {
    Registry::load_module(root).unwrap()
}

fn reasons(registry: &Registry) -> Vec<String> {
    registry
        .failures()
        .iter()
        .map(|f| f.reason.clone())
        .collect()
}

fn codes(registry: &Registry) -> Vec<String> {
    reasons(registry)
        .iter()
        .map(|r| r.split(':').next().unwrap().to_string())
        .collect()
}

fn semantic_diagnostics(registry: &Registry) -> Vec<(String, String)> {
    registry
        .diagnostics()
        .iter()
        .filter_map(|d| match d {
            quire_rs::Diagnostic::Semantic { code, message, .. } => {
                Some((code.clone(), message.clone()))
            }
            _ => None,
        })
        .collect()
}

#[trace("TC-1599", "FR-069-AC-1")]
// a valid block and a digest-matching reference schema load; the resolved
// validator accepts the golden declaration set. (`validate_document` over an
// entity artifact joins this row in Task-018, when `fields[]` is extracted.)
#[test]
fn valid_block_and_reference_schema_load() {
    let tmp = tempfile::tempdir().unwrap();
    let root = module(&tmp, "spec-objects-fixture", |_, _| {});
    let registry = load(&root);
    assert!(registry.failures().is_empty(), "{:?}", reasons(&registry));
    let sem: &SemanticModule = registry
        .semantic_module("spec-objects-fixture")
        .expect("block");
    assert_eq!(sem.package, "agent-ix/spec-objects-fixture");
    assert_eq!(sem.semantic_core, "0.1.0");
    assert_eq!(sem.exports, vec!["entity".to_string()]);
    assert_eq!(sem.legacy_forms, "warning");
    let entity = registry.archetype("entity").unwrap();
    assert_eq!(
        entity.semantic_schema_digest.as_deref(),
        Some(digest_of(&root.join("schemas/Entity.json")).as_str())
    );
    let golden = golden();
    let validator = entity.data_validator().expect("resolved data schema");
    let record = json!({ "fields": golden["fields"], "clauses": golden["clauses"] });
    assert!(
        validator.is_valid(&record),
        "golden declaration set validates"
    );
    assert!(!validator.is_valid(&json!({ "fields": [{ "name": "x" }] })));
    // The enumeration keeps its inline schema and is not exported.
    assert!(registry
        .archetype("enumeration")
        .unwrap()
        .semantic_schema_digest
        .is_none());
}

#[trace("TC-1600", "FR-069-AC-2")]
// version refusals come first and alone.
#[test]
fn unsupported_versions_are_refused_first() {
    let tmp = tempfile::tempdir().unwrap();
    let root = module(&tmp, "v2", |m, _| {
        semantic(m).insert("contract_version".into(), "2.0.0".into());
        semantic(m).insert("bogus".into(), 1.into()); // would be unknown-key, never reached
    });
    let registry = load(&root);
    let codes = codes(&registry);
    assert!(!codes.is_empty());
    assert!(
        codes
            .iter()
            .all(|c| c == "semantic.unsupported-contract-version"),
        "{codes:?}"
    );
    assert!(registry.semantic_module("v2").is_none());
    assert!(
        registry.archetype("entity").is_none(),
        "no archetype loads as an empty model"
    );

    let root = module(&tmp, "core9", |m, _| {
        semantic(m).insert("semantic_core".into(), "0.9.0".into());
    });
    let registry = load(&root);
    let r = reasons(&registry);
    assert!(
        r.iter()
            .all(|x| x.starts_with("semantic.unsupported-semantic-core")),
        "{r:?}"
    );
    assert!(r[0].contains("0.9.0") && r[0].contains("0.1.0"), "{}", r[0]);
}

#[trace("TC-1601", "FR-069-AC-3")]
// block-shape refusals name the key or value.
#[test]
fn block_shape_refusals_name_the_offender() {
    let tmp = tempfile::tempdir().unwrap();
    let cases: Vec<(&str, Mutate, &str, &str)> = vec![
        (
            "unknown-key",
            Box::new(|m, _| {
                semantic(m).insert("extra".into(), 1.into());
            }),
            "semantic.unknown-key",
            "extra",
        ),
        (
            "export-undeclared",
            Box::new(|m, _| {
                semantic(m).insert(
                    "exports".into(),
                    serde_yaml::Value::Sequence(vec!["entity".into(), "ghost".into()]),
                );
            }),
            "semantic.export-undeclared",
            "ghost",
        ),
        (
            "bad-package",
            Box::new(|m, _| {
                semantic(m).insert("package".into(), "ix://agent-ix/x".into());
            }),
            "semantic.invalid-package",
            "ix://agent-ix/x",
        ),
        (
            "bad-target",
            Box::new(|m, _| {
                semantic(m).insert(
                    "targets".into(),
                    serde_yaml::Value::Sequence(vec!["go".into()]),
                );
            }),
            "semantic.unknown-target",
            "go",
        ),
        (
            "export-inline",
            Box::new(|m, _| {
                semantic(m).insert(
                    "exports".into(),
                    serde_yaml::Value::Sequence(vec!["entity".into(), "enumeration".into()]),
                );
            }),
            "semantic.export-without-schema",
            "enumeration",
        ),
    ];
    for (name, mutate, code, value) in cases {
        let root = module(&tmp, name, mutate);
        let registry = load(&root);
        let r = reasons(&registry);
        assert!(
            r.iter().any(|x| x.starts_with(code) && x.contains(value)),
            "{name}: {r:?}"
        );
        assert!(
            registry.archetype("entity").is_none(),
            "{name}: refused module loads no type"
        );
    }
}

#[trace("TC-1602", "FR-069-AC-4")]
// reference-form refusals name path and reason; the mixed form is ambiguous.
#[test]
fn reference_form_refusals() {
    let tmp = tempfile::tempdir().unwrap();
    let outside = tmp.path().join("outside.json");
    fs::write(&outside, "{}").unwrap();
    let cases: Vec<(&str, Mutate, &str)> = vec![
        (
            "mismatch",
            Box::new(|_, root| {
                let f = root.join("schemas/Entity.json");
                let mut b = fs::read(&f).unwrap();
                b.push(b'\n');
                fs::write(f, b).unwrap();
            }),
            "semantic.data-schema-digest-mismatch",
        ),
        (
            "missing",
            Box::new(|_, root| fs::remove_file(root.join("schemas/Entity.json")).unwrap()),
            "semantic.data-schema-missing",
        ),
        (
            "not-json",
            Box::new(|m, root| {
                let f = root.join("schemas/Entity.json");
                fs::write(&f, "{ nope").unwrap();
                entity_schema_ref(m).insert("digest".into(), digest_of(&f).into());
            }),
            "semantic.data-schema-not-json",
        ),
        (
            "no-schema-keyword",
            Box::new(|m, root| {
                edit_entity(m, root, |s| {
                    s.as_object_mut().unwrap().remove("$schema");
                })
            }),
            "semantic.data-schema-not-schema",
        ),
        (
            "wrong-id",
            Box::new(|m, root| {
                edit_entity(m, root, |s| {
                    s["$id"] =
                        json!("https://schemas.agent-ix.org/agent-ix/other/0.1.0/Entity.json");
                })
            }),
            "semantic.data-schema-id",
        ),
        (
            "dotdot",
            Box::new(|m, _| {
                entity_schema_ref(m).insert("schema".into(), "../outside.json".into());
            }),
            "semantic.data-schema-escape",
        ),
        (
            "symlink",
            Box::new({
                let outside = outside.clone();
                move |m, root| {
                    let f = root.join("schemas/Entity.json");
                    fs::remove_file(&f).unwrap();
                    std::os::unix::fs::symlink(&outside, &f).unwrap();
                    entity_schema_ref(m).insert("digest".into(), digest_of(&outside).into());
                }
            }),
            "semantic.data-schema-escape",
        ),
        (
            "mixed",
            Box::new(|m, _| {
                entity_schema_ref(m).insert("type".into(), "object".into());
            }),
            "semantic.data-schema-ambiguous",
        ),
    ];
    for (name, mutate, code) in cases {
        let root = module(&tmp, name, mutate);
        let registry = load(&root);
        let r = reasons(&registry);
        assert!(r.iter().any(|x| x.starts_with(code)), "{name}: {r:?}");
        if code != "semantic.data-schema-ambiguous" {
            assert!(
                r.iter()
                    .any(|x| x.contains("Entity.json") || x.contains("outside.json")),
                "{name} names the path: {r:?}"
            );
        }
        assert!(registry.archetype("entity").is_none(), "{name}");
    }
}

fn fields_items(schema: &mut Value) -> &mut Value {
    &mut schema["properties"]["fields"]["items"]
}

#[trace("TC-1603", "FR-069-AC-5")]
// `$ref` rules: version drift, unshipped, outside, cycle; self fragment ok.
#[test]
fn ref_rules() {
    let tmp = tempfile::tempdir().unwrap();
    let root = module(&tmp, "core-version", |m, root| {
        edit_entity(m, root, |s| {
            *fields_items(s) = json!({ "$ref": "https://schemas.agent-ix.org/semantic-core/0.2.0/FieldDecl.json" });
        })
    });
    let r = reasons(&load(&root));
    assert!(
        r.iter()
            .any(|x| x.starts_with("semantic.schema-ref-version") && x.contains("0.2.0")),
        "{r:?}"
    );

    let root = module(&tmp, "unshipped", |m, root| {
        edit_entity(m, root, |s| {
            *fields_items(s) = json!({ "$ref": "https://schemas.agent-ix.org/agent-ix/spec-objects-fixture/0.1.0/Missing.json" });
        })
    });
    let r = reasons(&load(&root));
    assert!(
        r.iter()
            .any(|x| x.starts_with("semantic.schema-ref-unshipped") && x.contains("Missing.json")),
        "{r:?}"
    );

    let root = module(&tmp, "outside", |m, root| {
        edit_entity(m, root, |s| {
            *fields_items(s) = json!({ "$ref": "https://example.org/x.json" });
        })
    });
    let r = reasons(&load(&root));
    assert!(
        r.iter()
            .any(|x| x.starts_with("semantic.schema-ref-unshipped") && x.contains("example.org")),
        "{r:?}"
    );

    let root = module(&tmp, "cycle", |m, root| {
        fs::write(
            root.join("schemas/Other.json"),
            serde_json::to_vec(&json!({
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "$id": "https://schemas.agent-ix.org/agent-ix/spec-objects-fixture/0.1.0/Other.json",
                "type": "object",
                "properties": { "back": { "$ref": "https://schemas.agent-ix.org/agent-ix/spec-objects-fixture/0.1.0/Entity.json" } }
            }))
            .unwrap(),
        )
        .unwrap();
        edit_entity(m, root, |s| {
            s["properties"]["other"] = json!({ "$ref": "https://schemas.agent-ix.org/agent-ix/spec-objects-fixture/0.1.0/Other.json" });
        });
    });
    let r = reasons(&load(&root));
    assert!(
        r.iter()
            .any(|x| x.starts_with("semantic.schema-ref-cycle") && x.contains("Other.json")),
        "{r:?}"
    );

    let root = module(&tmp, "self-fragment", |m, root| {
        edit_entity(m, root, |s| {
            s["$defs"] = json!({ "marker": { "type": "string" } });
            s["properties"]["marker"] = json!({ "$ref": "https://schemas.agent-ix.org/agent-ix/spec-objects-fixture/0.1.0/Entity.json#/$defs/marker" });
        })
    });
    let registry = load(&root);
    assert!(registry.failures().is_empty(), "{:?}", reasons(&registry));
    assert!(registry
        .archetype("entity")
        .unwrap()
        .data_validator()
        .unwrap()
        .is_valid(&json!({ "fields": [], "marker": "x" })));
}

#[trace("TC-1608", "FR-069-CON-1")]
// the resolver has no fetch path and reads only the module and the bundle.
#[test]
fn resolver_reads_no_network_and_nothing_outside_the_module() {
    let source = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/semantic/resolver.rs"),
    )
    .unwrap();
    assert!(
        !source.contains("reqwest") && !source.contains("ureq") && !source.contains("std::net")
    );
    assert_eq!(
        source.matches("std::fs::read").count(),
        1,
        "one filesystem read, inside read_module_file"
    );
    assert!(
        !source.contains("with_resolver"),
        "no schema-library resolver callback"
    );
    // The pure in-memory path (what the `wasm` feature runs): an `https://`
    // `$ref` outside both bundles is refused, never fetched.
    let schema = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://schemas.agent-ix.org/agent-ix/x/0.1.0/T.json",
        "properties": { "f": { "$ref": "https://example.org/remote.json" } }
    });
    let err = compile_module_schema(
        &schema,
        &|_| None,
        "0.1.0",
        "https://schemas.agent-ix.org/agent-ix/x/0.1.0/",
    )
    .err()
    .unwrap();
    assert_eq!(err.code, "semantic.schema-ref-unshipped");
    // Bundle-only references compile from memory.
    let ok = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://schemas.agent-ix.org/agent-ix/x/0.1.0/T.json",
        "properties": { "f": { "$ref": "https://schemas.agent-ix.org/semantic-core/0.1.0/FieldDecl.json" } }
    });
    let validator = compile_module_schema(
        &ok,
        &|_| None,
        "0.1.0",
        "https://schemas.agent-ix.org/agent-ix/x/0.1.0/",
    )
    .unwrap();
    assert!(validator.is_valid(&json!({ "f": { "name": "id", "type": { "target": "UUID" } } })));
    assert!(!validator.is_valid(&json!({ "f": { "name": "id" } })));
}

#[trace("TC-1609", "FR-069-CON-4")]
// the recorded digest is over shipped bytes; the schema is not normalized.
#[test]
fn digest_is_over_shipped_bytes() {
    let tmp = tempfile::tempdir().unwrap();
    // Pretty-printed with odd whitespace: bytes change, meaning does not.
    let root = module(&tmp, "whitespace", |m, root| {
        let f = root.join("schemas/Entity.json");
        let v: Value = serde_json::from_slice(&fs::read(&f).unwrap()).unwrap();
        fs::write(
            &f,
            format!("\n\n{}\n\n", serde_json::to_string_pretty(&v).unwrap()),
        )
        .unwrap();
        entity_schema_ref(m).insert("digest".into(), digest_of(&f).into());
    });
    let registry = load(&root);
    assert!(registry.failures().is_empty(), "{:?}", reasons(&registry));
    let entity = registry.archetype("entity").unwrap();
    assert_eq!(
        entity.semantic_schema_digest.as_deref(),
        Some(digest_of(&root.join("schemas/Entity.json")).as_str())
    );
    let on_disk: Value =
        serde_json::from_slice(&fs::read(root.join("schemas/Entity.json")).unwrap()).unwrap();
    assert_eq!(entity.data_schema.as_deref(), Some(&on_disk));
}

#[trace("TC-1604", "FR-069-AC-6")]
// inline data_schema under a block warns; without a block it is silent.
#[test]
fn inline_schema_warns_only_under_a_block() {
    let tmp = tempfile::tempdir().unwrap();
    let root = module(&tmp, "inline-warn", |_, _| {});
    let registry = load(&root);
    let diags = semantic_diagnostics(&registry);
    assert!(
        diags
            .iter()
            .any(|(c, m)| c == "semantic.inline-data-schema" && m.contains("enumeration")),
        "{diags:?}"
    );

    let root = module(&tmp, "no-block", |m, _| {
        m.as_mapping_mut().unwrap().remove("semantic");
        m["object_types"][0]["data_schema"] = serde_yaml::from_str("type: object").unwrap();
    });
    let registry = load(&root);
    assert!(registry.failures().is_empty(), "{:?}", reasons(&registry));
    assert!(semantic_diagnostics(&registry).is_empty());
    assert!(registry.semantic_module("no-block").is_none());
    // The reference form without a block is refused, never compiled as a schema.
    let root = module(&tmp, "ref-no-block", |m, _| {
        m.as_mapping_mut().unwrap().remove("semantic");
    });
    let r = reasons(&load(&root));
    assert!(
        r.iter()
            .any(|x| x.starts_with("semantic.data-schema-reference-without-block")),
        "{r:?}"
    );
}

fn snapshot_input(object_types: Vec<Value>) -> FilamentExtractionInput {
    serde_json::from_value(json!({
        "projectId": "p", "documentId": "d", "artifactId": "a", "relPath": "spec/FR-006.md",
        "repoName": "config-service", "org": "agent-ix",
        "markdown": "---\nid: FR-006\ntitle: ConfigVersion\nobject: entity\n---\n# FR-006\n",
        "objectTypes": object_types
    }))
    .unwrap()
}

fn entity_snapshot(data_schema: Value, semantic: Option<Value>) -> Value {
    let mut v = json!({ "name": "entity", "dataSchema": data_schema, "allowedLinks": {}, "bodyExtraction": null, "hasPlugin": false, "moduleId": "m" });
    if let Some(s) = semantic {
        v["semantic"] = s;
    }
    v
}

#[trace("TC-1605", "FR-069-AC-7")]
// a Filament snapshot with the reference form is refused; inline + context extracts.
#[test]
fn filament_snapshot_reference_form_is_refused() {
    let reference = json!({ "schema": "schemas/Entity.json", "digest": "sha256:00" });
    let result = extract_filament_core(snapshot_input(vec![entity_snapshot(reference, None)]));
    assert!(result.diagnostics.iter().any(|d| d.code == "semantic.data-schema-unresolved-reference" && d.severity == "error"), "{:?}", result.diagnostics);
    // The document-level artifact node (FR-045 fallback) may remain; no node
    // of the refused object type is produced.
    assert!(
        result.nodes.iter().all(|n| n.object_type != "entity"),
        "{:?}",
        result.nodes
    );
    assert!(!result
        .diagnostics
        .iter()
        .any(|d| d.code == "unknown_object_type"));

    let inline: Value =
        serde_json::from_slice(&fs::read(fixture().join("schemas/Entity.json")).unwrap()).unwrap();
    let context = json!({ "contractVersion": "1.0.0", "semanticCore": "0.1.0", "package": "agent-ix/spec-objects-fixture", "exports": ["entity"], "imports": {} });
    let result = extract_filament_core(snapshot_input(vec![entity_snapshot(
        json!({ "type": "object" }),
        Some(context.clone()),
    )]));
    assert!(
        result.nodes.iter().any(|n| n.object_type == "entity"),
        "{:?}",
        result.diagnostics
    );
    // A bundle-referencing schema compiles through the vendored bundle and
    // validates the declaration record: the golden table passes, a bare
    // document fails it with `semantic.record-invalid`.
    let mut input = snapshot_input(vec![entity_snapshot(inline.clone(), Some(context.clone()))]);
    input.markdown = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/semantic/quoin/mapping/config-version.table.md"),
    )
    .unwrap();
    let result = extract_filament_core(input);
    assert!(
        !result.diagnostics.iter().any(|d| d.severity == "error"),
        "{:?}",
        result.diagnostics
    );
    let result =
        extract_filament_core(snapshot_input(vec![entity_snapshot(inline, Some(context))]));
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.code == "semantic.record-invalid"),
        "{:?}",
        result.diagnostics
    );
}

#[trace("TC-1633", "FR-072-AC-4")]
// unsupported snapshot versions are refused before any node is produced.
#[test]
fn filament_snapshot_unsupported_versions_are_refused() {
    for (context, code) in [
        (
            json!({ "contractVersion": "2.0.0", "semanticCore": "0.1.0", "package": "agent-ix/x" }),
            "semantic.unsupported-contract-version",
        ),
        (
            json!({ "contractVersion": "1.0.0", "semanticCore": "0.9.0", "package": "agent-ix/x" }),
            "semantic.unsupported-semantic-core",
        ),
    ] {
        let result = extract_filament_core(snapshot_input(vec![entity_snapshot(
            json!({ "type": "object" }),
            Some(context),
        )]));
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.code == code && d.severity == "error"),
            "{code}: {:?}",
            result.diagnostics
        );
        assert!(
            result.nodes.iter().all(|n| n.object_type != "entity"),
            "{code}: {:?}",
            result.nodes
        );
    }
}

#[trace("TC-1645", "FR-069-AC-10")]
// cross-module: duplicate package, unresolved import, import cycle.
#[test]
fn cross_module_checks() {
    // `load_from` takes search roots holding module directories.
    let tmp = tempfile::tempdir().unwrap();
    module(&tmp, "a-first", |_, _| {});
    module(&tmp, "b-second", |_, _| {});
    let registry = Registry::load_from(&[tmp.path()]).unwrap();
    let r = reasons(&registry);
    assert!(
        r.iter()
            .any(|x| x.starts_with("semantic.duplicate-package") && x.contains("a-first")),
        "{r:?}"
    );
    assert!(registry.semantic_module("a-first").is_some());
    assert!(
        registry.semantic_module("b-second").is_none(),
        "later sorted root refused"
    );
    assert!(registry.archetype_in_module("b-second", "entity").is_none());

    let tmp = tempfile::tempdir().unwrap();
    module(&tmp, "needy", |m, root| {
        retarget(m, root, "agent-ix/needy");
        semantic(m).insert(
            "imports".into(),
            serde_yaml::from_str("{ 'agent-ix/other': '0.2.0' }").unwrap(),
        );
    });
    let registry = Registry::load_from(&[tmp.path()]).unwrap();
    assert!(registry.failures().is_empty(), "{:?}", reasons(&registry));
    let diags = semantic_diagnostics(&registry);
    assert!(
        diags.iter().any(|(c, m)| c == "semantic.import-unresolved"
            && m.contains("agent-ix/other@0.2.0")
            && m.contains("none")),
        "{diags:?}"
    );

    let tmp = tempfile::tempdir().unwrap();
    module(&tmp, "x-cycle", |m, root| {
        retarget(m, root, "agent-ix/x");
        semantic(m).insert(
            "imports".into(),
            serde_yaml::from_str("{ 'agent-ix/y': '0.1.0' }").unwrap(),
        );
    });
    module(&tmp, "y-cycle", |m, root| {
        retarget(m, root, "agent-ix/y");
        semantic(m).insert(
            "imports".into(),
            serde_yaml::from_str("{ 'agent-ix/x': '0.1.0' }").unwrap(),
        );
    });
    let registry = Registry::load_from(&[tmp.path()]).unwrap();
    let r = reasons(&registry);
    assert!(
        r.iter().any(|x| x.starts_with("semantic.import-cycle")
            && x.contains("agent-ix/x")
            && x.contains("agent-ix/y")),
        "{r:?}"
    );
    assert!(
        registry.semantic_module("x-cycle").is_none()
            && registry.semantic_module("y-cycle").is_none()
    );
}

#[trace("TC-1646", "FR-069-AC-11")]
// inline parts serve the reference form from the `schemas` map.
#[test]
fn inline_parts_resolve_the_reference_form() {
    let manifest = fs::read(fixture().join("manifest.yaml")).unwrap();
    let entity = fs::read_to_string(fixture().join("schemas/Entity.json")).unwrap();
    let mut schemas = BTreeMap::new();
    schemas.insert("schemas/Entity.json".to_string(), entity.clone());
    let registry = Registry::from_inline_parts(&manifest, &schemas).unwrap();
    assert!(registry.failures().is_empty(), "{:?}", reasons(&registry));
    assert!(registry
        .archetype("entity")
        .unwrap()
        .semantic_schema_digest
        .is_some());

    // Missing from the map: refused as missing, never read from disk.
    let registry = Registry::from_inline_parts(&manifest, &BTreeMap::new()).unwrap();
    assert!(
        codes(&registry).contains(&"semantic.data-schema-missing".to_string()),
        "{:?}",
        reasons(&registry)
    );

    // A `..` key is an escape.
    let text = String::from_utf8(manifest.clone())
        .unwrap()
        .replace("schemas/Entity.json", "../Entity.json");
    let mut escaped = BTreeMap::new();
    escaped.insert("../Entity.json".to_string(), entity);
    let registry = Registry::from_inline_parts(text.as_bytes(), &escaped).unwrap();
    assert!(
        codes(&registry).contains(&"semantic.data-schema-escape".to_string()),
        "{:?}",
        reasons(&registry)
    );
}

/// The golden table with an `abstract` flag and a `## Values` table.
fn featured_document() -> String {
    fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/semantic/quoin/mapping/config-version.table.md"),
    )
    .unwrap()
    .replace("type: FR\n", "type: FR\nabstract: true\n")
        + "\n## Values\n\n| Value | Description |\n|---|---|\n| draft | |\n"
}

fn values_locator() -> Value {
    json!({ "from": "table_row", "under_section": "Values", "assert": { "columns": ["Value", "Description"] } })
}

#[trace("TC-1848", "FR-069-AC-12")]
// `semantic.mappings` is recorded on the loaded module.
#[test]
fn mappings_are_recorded() {
    let tmp = TempDir::new().unwrap();
    let root = module(&tmp, "mapped", |m, _| {
        semantic(m).insert("mappings".into(), vec!["abstract-types", "presence"].into());
    });
    let registry = load(&root);
    let module = registry.semantic_module("mapped").unwrap();
    assert_eq!(module.mappings, ["abstract-types", "presence"]);
    let registry = load(&fixture());
    assert!(registry
        .semantic_module("spec-objects-fixture")
        .unwrap()
        .mappings
        .is_empty());
}

#[trace("TC-1849", "FR-075-AC-9")]
// validate_document and Filament extraction read a declared feature and
// refuse an undeclared one, from the manifest alone.
#[test]
fn surfaces_gate_model_features_on_the_manifest() {
    let doc = featured_document();
    let tmp = TempDir::new().unwrap();
    let refusals = |registry: &Registry| {
        let entity = registry.archetype("entity").unwrap();
        let result = quire_rs::validate_document_in_registry(registry, entity, &doc);
        result
            .errors
            .into_iter()
            .filter(|e| e.message.starts_with("semantic.feature-not-extractable"))
            .map(|e| e.message)
            .collect::<Vec<_>>()
    };
    let declared = module(&tmp, "declared", |m, _| {
        semantic(m).insert("mappings".into(), vec!["abstract-types"].into());
        let locator = serde_yaml::to_value(values_locator()).unwrap();
        m["object_types"][0]["body_extraction"]["yield_pattern"]["match"]
            .as_mapping_mut()
            .unwrap()
            .insert("values".into(), locator);
    });
    assert_eq!(refusals(&load(&declared)), Vec::<String>::new());
    let bare = refusals(&load(&fixture()));
    assert_eq!(bare.len(), 2, "{bare:?}");
    assert!(bare.iter().all(|m| m.contains("<document>")), "{bare:?}");
    assert!(
        bare.iter().any(|m| m.contains("abstract-types")),
        "{bare:?}"
    );
    assert!(bare.iter().any(|m| m.contains("values")), "{bare:?}");

    let context = |mappings: &[&str]| json!({ "contractVersion": "1.0.0", "semanticCore": "0.1.0", "package": "agent-ix/spec-objects-fixture", "exports": ["entity"], "imports": {}, "mappings": mappings });
    let run = |mappings: &[&str], body_extraction: Value| {
        let mut object_type = entity_snapshot(json!({ "type": "object" }), Some(context(mappings)));
        object_type["bodyExtraction"] = body_extraction;
        let mut input = snapshot_input(vec![object_type]);
        input.markdown = doc.clone();
        let result = extract_filament_core(input);
        let node = result
            .nodes
            .iter()
            .find(|n| n.object_type == "entity")
            .cloned();
        (result, node)
    };
    let dsl = json!({ "yield_pattern": { "match": { "values": values_locator() } } });
    let (result, node) = run(&["abstract-types"], dsl.clone());
    assert!(
        !result
            .diagnostics
            .iter()
            .any(|d| d.code == "semantic.feature-not-extractable"),
        "{:?}",
        result.diagnostics
    );
    let data: Value = serde_json::from_str(&node.unwrap().data_json).unwrap();
    let model = &data["semantic"]["model"];
    assert_eq!(model["abstract"]["value"], true, "{data:#}");
    assert_eq!(model["values"][0]["value"], "draft");

    let (result, node) = run(&[], dsl);
    let refused: Vec<&str> = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "semantic.feature-not-extractable")
        .map(|d| d.message.as_str())
        .collect();
    assert_eq!(refused.len(), 1, "{refused:?}");
    assert!(refused[0].contains("abstract-types") && refused[0].contains("spec/FR-006.md"));
    let data: Value = serde_json::from_str(&node.unwrap().data_json).unwrap();
    assert!(data["semantic"].get("model").is_none(), "{data:#}");
    assert_eq!(
        data["semantic"]["availability"]["model"]["state"],
        "unavailable"
    );
}

/// A Rust caller gates model tables through the public context builder and
/// gets the record the JSON adapter returns for the same request.
#[trace("TC-1869", "FR-075-AC-10")]
#[test]
fn rust_callers_gate_model_tables_through_the_public_context() {
    use quire_rs::extract::dsl::ExtractionDsl;
    use quire_rs::semantic::python_entry::extract_semantic_json;
    use quire_rs::semantic::{extract_semantic, BundleIndex, RequiredSections, SemanticContext};

    let doc = featured_document().replace("abstract: true\n", "");
    let dsl = json!({ "yield_pattern": { "match": { "values": values_locator() } } });
    let typed: ExtractionDsl = serde_json::from_value(dsl.clone()).unwrap();
    let module = SemanticModule {
        contract_version: "1.0.0".into(),
        semantic_core: "0.1.0".into(),
        package: "agent-ix/spec-objects-fixture".into(),
        exports: vec!["entity".into()],
        imports: BTreeMap::new(),
        targets: Vec::new(),
        compatibility_posture: "additive".into(),
        legacy_forms: "warning".into(),
        mappings: Vec::new(),
    };
    let bare = SemanticContext::new(module, "spec/FR-006.md", BundleIndex::default())
        .with_source_identity("ix://agent-ix/fixture/spec");
    let gated = bare.clone().with_body_extraction(&typed);
    let required = RequiredSections::default();

    let record = extract_semantic(&doc, &gated, None, &required);
    let model = record
        .model
        .as_ref()
        .expect("the declared Values table extracts");
    let values: Vec<&str> = model
        .values
        .iter()
        .flatten()
        .map(|v| v.value.as_str())
        .collect();
    assert_eq!(values, ["draft"]);
    assert!(
        !record
            .diagnostics
            .iter()
            .any(|d| d.code == "semantic.feature-not-extractable"),
        "{:?}",
        record.diagnostics
    );

    let adapter = extract_semantic_json(&json!({
        "markdown": doc,
        "module": { "contractVersion": "1.0.0", "semanticCore": "0.1.0", "package": "agent-ix/spec-objects-fixture", "exports": ["entity"] },
        "path": "spec/FR-006.md",
        "sourceIdentity": "ix://agent-ix/fixture/spec",
        "bodyExtraction": dsl,
    }))
    .unwrap();
    assert_eq!(record, adapter, "the typed path and the adapter agree");

    // The control: no `body_extraction`, so the table is refused.
    let refused = extract_semantic(&doc, &bare, None, &required);
    assert!(refused.model.is_none());
    assert!(
        refused
            .diagnostics
            .iter()
            .any(|d| d.code == "semantic.feature-not-extractable"),
        "{:?}",
        refused.diagnostics
    );
}

/// `RequiredSections` read from a typed `body_extraction` equals the one read
/// from the same DSL as authored JSON and as the typed DSL serialized back to
/// JSON, and names exactly the headings a required locator sits under.
#[trace("TC-1870", "FR-075-AC-11")]
#[test]
fn required_sections_agree_across_the_typed_and_json_dsl() {
    use quire_rs::extract::dsl::ExtractionDsl;
    use quire_rs::semantic::RequiredSections;

    let authored = json!({
        "yield_pattern": { "match": {
            "fields": { "from": "table_row", "under_section": "Properties" },
            "rules": [
                { "from": "list_item", "after_heading": "Invariants", "required": false },
                { "from": "section_body", "after_heading": "Invariants" }
            ],
            "ops": { "from": "code_block", "under_section": "Operations", "required": false },
            "id": { "from": "frontmatter_field", "path": ["id"] }
        } }
    });
    let typed: ExtractionDsl = serde_json::from_value(authored.clone()).unwrap();
    let expected = RequiredSections {
        properties: true,
        invariants: true,
        operations: false,
    };
    assert_eq!(RequiredSections::from_extraction(&typed), expected);
    assert_eq!(RequiredSections::from_dsl(&authored), expected);
    assert_eq!(
        RequiredSections::from_dsl(&serde_json::to_value(&typed).unwrap()),
        expected
    );
}

/// The golden table with its prose `## Relationships` replaced by a table of
/// `rows`.
fn related_document(rows: &str) -> String {
    fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/semantic/quoin/mapping/config-version.table.md"),
    )
    .unwrap()
    .replace(
        "- `overlay`: belongs_to → ConfigOverlay (FR-005)\n",
        &format!("| Name | Verb | Target | Multiplicity |\n|---|---|---|---|\n{rows}"),
    )
}

/// The fixture module with the `relationships` token, `references` and
/// `contains` edge types, and an entity whose role satisfies `references`.
fn related_registry(tmp: &TempDir) -> Registry {
    let root = module(tmp, "related", |m, _| {
        semantic(m).insert("mappings".into(), vec!["relationships"].into());
        m["edge_types"] = serde_yaml::from_str(
            "references: { description: r, category: traceability }\n\
             contains: { description: c, category: structural, inverse: part_of }\n",
        )
        .unwrap();
        m["roles"] = serde_yaml::from_str("domain-object: { description: d }\n").unwrap();
        let entity = m["object_types"][0].as_mapping_mut().unwrap();
        entity.insert(
            "roles".into(),
            serde_yaml::from_str("[domain-object]").unwrap(),
        );
        entity.insert(
            "allowed_links".into(),
            serde_yaml::from_str("{ references: [domain-object], contains: [enumeration] }")
                .unwrap(),
        );
    });
    load(&root)
}

/// The bundle the related documents belong to.
const BUNDLE_PACKAGE: &str = "agent-ix/config-service";

/// The `relationships` errors and warnings of `result`.
fn relationship_findings(result: quire_rs::ValidationResult) -> (Vec<String>, Vec<String>) {
    let messages = |m: Vec<String>| {
        m.into_iter()
            .filter(|m| m.contains("relationships"))
            .collect::<Vec<_>>()
    };
    (
        messages(result.errors.into_iter().map(|e| e.message).collect()),
        messages(result.warnings.into_iter().map(|w| w.message).collect()),
    )
}

#[trace("TC-1864", "FR-076-AC-13")]
// validate_document checks relationship rows against the loaded registry's
// edge_types, roles, and allowed_links; Filament and the Python entry, which
// carry no vocabulary, report `no-relation-vocabulary`.
#[test]
fn surfaces_supply_the_relation_vocabulary() {
    let tmp = TempDir::new().unwrap();
    let registry = related_registry(&tmp);
    let entity = registry.archetype("entity").unwrap();
    let findings = |doc: &str| {
        relationship_findings(quire_rs::validate_document_in_bundle(
            &registry,
            entity,
            doc,
            BUNDLE_PACKAGE,
            None,
        ))
    };

    // The own id satisfies `references` through the entity's role, so the
    // row lowers with no finding; a bare id this surface cannot check lowers
    // with the `no-bundle-index` advisory.
    let (errors, warnings) = findings(&related_document(
        "| predecessor | references | FR-006 | 0..1 |\n| overlay | references | FR-005 | 1..1 |\n",
    ));
    assert_eq!(errors, Vec::<String>::new());
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(
        warnings[0].starts_with("semantic.unresolved-target") && warnings[0].contains("FR-005"),
        "{warnings:?}"
    );

    // The registry's vocabulary refuses: an unregistered verb, an inverse
    // label, and an own-id target outside `contains`'s allowed_links.
    let (errors, _) = findings(&related_document(
        "| a | holds | FR-006 | 1 |\n| b | part_of | FR-006 | 1 |\n| c | contains | FR-006 | 1 |\n",
    ));
    assert_eq!(errors.len(), 3, "{errors:?}");
    for (message, verb) in errors.iter().zip(["holds", "part_of", "contains"]) {
        assert!(message.contains(verb), "{message}");
    }
    assert!(errors[1].contains("`contains`"), "{errors:?}");

    // Filament: the snapshot carries no edge_types or roles.
    let doc = related_document("| overlay | references | FR-005 | 1..1 |\n");
    let context = json!({ "contractVersion": "1.0.0", "semanticCore": "0.1.0", "package": "agent-ix/spec-objects-fixture", "exports": ["entity"], "imports": {}, "mappings": ["relationships"] });
    let mut input = snapshot_input(vec![entity_snapshot(
        json!({ "type": "object" }),
        Some(context.clone()),
    )]);
    input.markdown = doc.clone();
    let result = extract_filament_core(input);
    let advisory: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "semantic.relationships-no-vocabulary")
        .collect();
    assert_eq!(advisory.len(), 1, "{:?}", result.diagnostics);
    let node = result
        .nodes
        .iter()
        .find(|n| n.object_type == "entity")
        .unwrap();
    let data: Value = serde_json::from_str(&node.data_json).unwrap();
    assert_eq!(
        data["semantic"]["availability"]["relations"],
        json!({ "state": "unavailable", "reason": "no-relation-vocabulary", "lossy": false })
    );
    assert!(data["semantic"].get("relations").is_none(), "{data:#}");

    // Python entry without `relationVocabulary`.
    let record = quire_rs::semantic::python_entry::extract_semantic_json(&json!({
        "markdown": doc,
        "module": context,
        "path": "spec/FR-006.md",
    }))
    .unwrap();
    let record = serde_json::to_value(record).unwrap();
    assert_eq!(
        record["availability"]["relations"]["reason"],
        "no-relation-vocabulary"
    );
    assert!(record.get("relations").is_none());
}

#[trace("TC-1866", "FR-076-AC-15")]
// validate_document qualifies targets under the caller's bundle package,
// never the module's; with no bundle package the feature is unavailable
// with `no-bundle-package`.
#[test]
fn validate_document_qualifies_under_the_bundle_package() {
    let tmp = TempDir::new().unwrap();
    let registry = related_registry(&tmp);
    let entity = registry.archetype("entity").unwrap();
    let in_bundle = |doc: &str| {
        relationship_findings(quire_rs::validate_document_in_bundle(
            &registry,
            entity,
            doc,
            BUNDLE_PACKAGE,
            None,
        ))
    };
    let with_frontmatter = |target: &str, row_target: &str| {
        related_document(&format!("| overlay | references | {row_target} | 1..1 |\n")).replacen(
            "type: FR\n",
            &format!("type: FR\nrelationships:\n  - target: \"{target}\"\n    type: references\n"),
            1,
        )
    };

    // An own-package identity lowers with the no-bundle-index advisory.
    let (errors, warnings) = in_bundle(&related_document(
        "| overlay | references | ix://agent-ix/config-service/FR-005 | 1..1 |\n",
    ));
    assert_eq!(errors, Vec::<String>::new());
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(
        warnings[0].starts_with("semantic.unresolved-target"),
        "{warnings:?}"
    );

    // A bare id qualifies under the bundle package: the frontmatter
    // duplicate is caught across the bare and `ix://` forms, both ways.
    for (frontmatter, row) in [
        (
            "ix://agent-ix/config-service/spec/functional/FR-005",
            "FR-005",
        ),
        ("FR-005", "ix://agent-ix/config-service/FR-005"),
    ] {
        let (errors, _) = in_bundle(&with_frontmatter(frontmatter, row));
        assert_eq!(errors.len(), 1, "{frontmatter} / {row}: {errors:?}");
        assert!(
            errors[0].starts_with("semantic.duplicate-model-entry"),
            "{errors:?}"
        );
    }
    // Under the module's package the bare row would not match the
    // frontmatter identity; the bundle package is what makes it match.
    let (errors, _) = in_bundle(&with_frontmatter(
        "ix://agent-ix/spec-objects-fixture/FR-005",
        "FR-005",
    ));
    assert_eq!(errors, Vec::<String>::new());

    // No bundle package: one heading advisory, no row checked.
    let result = quire_rs::validate_document_in_registry(
        &registry,
        entity,
        &related_document("| overlay | holds | FR-005 | 1..1 |\n"),
    );
    let (errors, warnings) = relationship_findings(result);
    assert_eq!(errors, Vec::<String>::new());
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(
        warnings[0].starts_with("semantic.relationships-no-bundle-package"),
        "{warnings:?}"
    );
}
