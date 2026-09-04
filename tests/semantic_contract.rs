//! FR-069 semantic module contract at load (TC-1599..TC-1609, TC-1633,
//! TC-1645, TC-1646). Plan-003 Task-016.
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
    // A bundle-referencing schema compiles through the vendored bundle.
    let result =
        extract_filament_core(snapshot_input(vec![entity_snapshot(inline, Some(context))]));
    assert!(
        !result.diagnostics.iter().any(|d| d.severity == "error"),
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
