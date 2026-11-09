//! FR-069 semantic module contract at load (TC-1599..TC-1608, TC-1633,
//! TC-1645, TC-1646, TC-1848, TC-1849, TC-1864, TC-1866). Plan-003 Task-016.
//!
//! Every case starts from the `module-ok` fixture in the `quire-fixtures`
//! crate (PLAT-901), copied into a temp dir and mutated in place; the
//! fixture itself is never edited.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use ix_trace_rs::trace;
use quire_rs::semantic::{compile_module_schema, embedded, SemanticModule};
use quire_rs::{extract_filament_core, FilamentExtractionInput, Registry};
use serde_json::{json, Value};
use tempfile::TempDir;

type Mutate = Box<dyn Fn(&mut serde_yaml::Value, &Path)>;

fn fixture() -> PathBuf {
    quire_fixtures::module_ok_dir()
}

fn golden() -> Value {
    quire_fixtures::mapping_json("config-version.expected.json")
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

/// Rewrite Entity.json, editing it.
fn edit_entity(root: &Path, edit: impl FnOnce(&mut Value)) {
    let file = root.join("schemas/Entity.json");
    let mut schema: Value = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
    edit(&mut schema);
    fs::write(&file, serde_json::to_vec(&schema).unwrap()).unwrap();
}

/// Point the fixture at another package.
fn retarget(m: &mut serde_yaml::Value, package: &str) {
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
// a valid block and a reference schema load; the resolved
// validator accepts the golden declaration set. (`validate_document` over an
// entity artifact joins this row in Task-018, when `fields[]` is extracted.)
#[test]
fn valid_block_and_reference_schema_load() {
    let tmp = tempfile::tempdir().unwrap();
    for (module_name, semantic_core) in [
        ("spec-objects-fixture-032", "0.3.2"),
        ("spec-objects-fixture-033", "0.3.3"),
        ("spec-objects-fixture-future", "99.0.0"),
    ] {
        let root = module(&tmp, module_name, |m, _| {
            semantic(m).insert("semantic_core".into(), semantic_core.into());
        });
        let registry = load(&root);
        assert!(
            registry.failures().is_empty(),
            "{semantic_core}: {:?}",
            reasons(&registry)
        );
        let sem: &SemanticModule = registry.semantic_module(module_name).expect("block");
        assert_eq!(sem.package, "agent-ix/spec-objects-fixture");
        assert_eq!(sem.exports, vec!["entity".to_string()]);
        assert_eq!(sem.legacy_forms, "warning");
        let entity = registry.archetype("entity").unwrap();
        let golden = golden();
        let validator = entity.data_validator().expect("resolved data schema");
        let record = json!({ "fields": golden["fields"], "clauses": golden["clauses"] });
        assert!(
            validator.is_valid(&record),
            "{semantic_core}: golden declaration set validates"
        );
        assert!(!validator.is_valid(&json!({ "fields": [{ "name": "x" }] })));
    }
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
}

/// Move the fixture's `entity` into `artifact_types` and add a second
/// archetype `entity2` under `archetypes`.
fn spread_across_sections(m: &mut serde_yaml::Value) {
    let entity = m["object_types"][0].clone();
    let mut second = entity.clone();
    second["name"] = "entity2".into();
    m["artifact_types"] = serde_yaml::Value::Sequence(vec![entity]);
    m["archetypes"] = serde_yaml::Value::Sequence(vec![second]);
    m["object_types"] = serde_yaml::Value::Sequence(Vec::new());
}

#[trace("TC-1600", "FR-069-AC-2")]
#[test]
fn refused_block_fails_every_archetype_in_every_section() {
    let tmp = tempfile::tempdir().unwrap();
    let root = module(&tmp, "spread", |m, _| {
        spread_across_sections(m);
        semantic(m).insert("contract_version".into(), "2.0.0".into());
    });
    let registry = load(&root);
    assert_eq!(codes(&registry).len(), 2, "{:?}", reasons(&registry));
    assert!(codes(&registry)
        .iter()
        .all(|c| c == "semantic.unsupported-contract-version"));
    assert!(registry.archetype("entity").is_none());
    assert!(registry.archetype("entity2").is_none());
}

#[trace("FR-069-AC-6")]
#[test]
fn inline_schema_warning_names_the_declaring_section() {
    let tmp = tempfile::tempdir().unwrap();
    let root = module(&tmp, "inline-section", |m, _| {
        let mut inline = m["object_types"][0].clone();
        inline["data_schema"] = serde_yaml::from_str("type: object").unwrap();
        m["artifact_types"] = serde_yaml::Value::Sequence(vec![inline]);
        m["object_types"] = serde_yaml::Value::Sequence(Vec::new());
    });
    let registry = load(&root);
    let paths: Vec<&str> = registry
        .diagnostics()
        .iter()
        .filter_map(|d| match d {
            quire_rs::Diagnostic::Semantic { code, path, .. }
                if code == "semantic.inline-data-schema" =>
            {
                Some(path.as_str())
            }
            _ => None,
        })
        .collect();
    assert_eq!(paths, ["artifact_types[entity].data_schema"]);
}

#[trace("FR-069-AC-3")]
#[test]
fn exported_artifact_type_is_a_declared_module_type() {
    let tmp = tempfile::tempdir().unwrap();
    let root = module(&tmp, "spec-artifact-fixture", |m, _| {
        m["artifact_types"] = m["object_types"].clone();
        m["object_types"] = serde_yaml::Value::Sequence(Vec::new());
    });
    let registry = load(&root);
    assert!(registry.failures().is_empty(), "{:?}", reasons(&registry));
    assert!(registry.archetype("entity").is_some());
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
            "missing",
            Box::new(|_, root| fs::remove_file(root.join("schemas/Entity.json")).unwrap()),
            "semantic.data-schema-missing",
        ),
        (
            "not-json",
            Box::new(|_, root| {
                fs::write(root.join("schemas/Entity.json"), "{ nope").unwrap();
            }),
            "semantic.data-schema-not-json",
        ),
        (
            "no-schema-keyword",
            Box::new(|_, root| {
                edit_entity(root, |s| {
                    s.as_object_mut().unwrap().remove("$schema");
                })
            }),
            "semantic.data-schema-not-schema",
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
                move |_, root| {
                    let f = root.join("schemas/Entity.json");
                    fs::remove_file(&f).unwrap();
                    std::os::unix::fs::symlink(&outside, &f).unwrap();
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
// `$ref` rules: any-version core ref resolves, unshipped, outside, cycle; self fragment ok.
#[test]
fn ref_rules() {
    let tmp = tempfile::tempdir().unwrap();
    let root = module(&tmp, "core-version", |_, root| {
        edit_entity(root, |s| {
            *fields_items(s) = json!({ "$ref": "https://schemas.agent-ix.org/semantic-core/0.2.0/FieldDecl.json" });
        })
    });
    // The version segment of a semantic-core `$ref` is not consulted: it
    // resolves to the embedded bundle's file of that name.
    let r = reasons(&load(&root));
    assert!(r.is_empty(), "{r:?}");

    let root = module(&tmp, "unshipped", |_, root| {
        edit_entity(root, |s| {
            *fields_items(s) = json!({ "$ref": "https://schemas.agent-ix.org/agent-ix/spec-objects-fixture/0.1.0/Missing.json" });
        })
    });
    let r = reasons(&load(&root));
    assert!(
        r.iter()
            .any(|x| x.starts_with("semantic.schema-ref-unshipped") && x.contains("Missing.json")),
        "{r:?}"
    );

    let root = module(&tmp, "outside", |_, root| {
        edit_entity(root, |s| {
            *fields_items(s) = json!({ "$ref": "https://example.org/x.json" });
        })
    });
    let r = reasons(&load(&root));
    assert!(
        r.iter()
            .any(|x| x.starts_with("semantic.schema-ref-unshipped") && x.contains("example.org")),
        "{r:?}"
    );

    let root = module(&tmp, "cycle", |_, root| {
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
        edit_entity(root, |s| {
            s["properties"]["other"] = json!({ "$ref": "https://schemas.agent-ix.org/agent-ix/spec-objects-fixture/0.1.0/Other.json" });
        });
    });
    let r = reasons(&load(&root));
    assert!(
        r.iter()
            .any(|x| x.starts_with("semantic.schema-ref-cycle") && x.contains("Other.json")),
        "{r:?}"
    );

    let root = module(&tmp, "self-fragment", |_, root| {
        edit_entity(root, |s| {
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
        "https://schemas.agent-ix.org/agent-ix/x/0.1.0/",
    )
    .err()
    .unwrap();
    assert_eq!(err.code, "semantic.schema-ref-unshipped");
    // Bundle-only references compile from memory.
    let ok = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://schemas.agent-ix.org/agent-ix/x/0.1.0/T.json",
        "properties": { "f": { "$ref": format!("https://schemas.agent-ix.org/semantic-core/{}/FieldDecl.json", embedded::embedded_semantic_core_version()) } }
    });
    let validator = compile_module_schema(
        &ok,
        &|_| None,
        "https://schemas.agent-ix.org/agent-ix/x/0.1.0/",
    )
    .unwrap();
    assert!(validator.is_valid(&json!({ "f": { "name": "id", "type": { "target": { "kind": "KernelScalar", "value": "UUID" } } } })));
    assert!(!validator.is_valid(&json!({ "f": { "name": "id" } })));
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
    let reference = json!({ "schema": "schemas/Entity.json" });
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
    let context = json!({ "contractVersion": "1.0.0", "package": "agent-ix/spec-objects-fixture", "exports": ["entity"], "imports": {} });
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
    input.markdown = quire_fixtures::mapping_fixture("config-version.table.md");
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
    let code = "semantic.unsupported-contract-version";
    let context = json!({ "contractVersion": "2.0.0", "package": "agent-ix/x" });
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

#[trace("TC-2190", "FR-069-AC-15")]
// two packages exporting one type: the later sorted root is refused; disjoint
// exports load both.
#[test]
fn duplicate_export_refuses_the_later_module() {
    let tmp = tempfile::tempdir().unwrap();
    module(&tmp, "a-first", |m, _| retarget(m, "agent-ix/a-first"));
    module(&tmp, "b-second", |m, _| retarget(m, "agent-ix/b-second"));
    let registry = Registry::load_from(&[tmp.path()]).unwrap();
    let r = reasons(&registry);
    assert!(
        r.iter().any(|x| x.starts_with("semantic.duplicate-export")
            && x.contains("entity")
            && x.contains("a-first")),
        "{r:?}"
    );
    assert!(registry.semantic_module("a-first").is_some());
    assert!(registry.semantic_module("b-second").is_none());
    assert!(registry.archetype_in_module("b-second", "entity").is_none());

    let tmp = tempfile::tempdir().unwrap();
    module(&tmp, "a-first", |m, _| retarget(m, "agent-ix/a-first"));
    module(&tmp, "b-second", |m, _| {
        retarget(m, "agent-ix/b-second");
        semantic(m).insert("exports".into(), serde_yaml::Value::Sequence(vec![]));
    });
    let registry = Registry::load_from(&[tmp.path()]).unwrap();
    assert!(registry.semantic_module("a-first").is_some());
    assert!(registry.semantic_module("b-second").is_some());
    assert!(reasons(&registry).is_empty(), "{:?}", reasons(&registry));
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
    module(&tmp, "needy", |m, _| {
        retarget(m, "agent-ix/needy");
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
    module(&tmp, "x-cycle", |m, _| {
        retarget(m, "agent-ix/x");
        semantic(m).insert(
            "imports".into(),
            serde_yaml::from_str("{ 'agent-ix/y': '0.1.0' }").unwrap(),
        );
    });
    module(&tmp, "y-cycle", |m, _| {
        retarget(m, "agent-ix/y");
        semantic(m).insert("exports".into(), serde_yaml::Value::Sequence(vec![]));
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
    quire_fixtures::mapping_fixture("config-version.table.md")
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

    let context = |mappings: &[&str]| json!({ "contractVersion": "1.0.0", "package": "agent-ix/spec-objects-fixture", "exports": ["entity"], "imports": {}, "mappings": mappings });
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

    let record = extract_semantic(&doc, &gated, &required);
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
        "module": { "contractVersion": "1.0.0", "package": "agent-ix/spec-objects-fixture", "exports": ["entity"] },
        "path": "spec/FR-006.md",
        "sourceIdentity": "ix://agent-ix/fixture/spec",
        "bodyExtraction": dsl,
    }))
    .unwrap();
    assert_eq!(record, adapter, "the typed path and the adapter agree");

    // The control: no `body_extraction`, so the table is refused.
    let refused = extract_semantic(&doc, &bare, &required);
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

/// `RequiredSections::from_extraction` marks exactly the headings a required
/// locator sits under, wherever the locator is declared: `match`,
/// `per_match`, an `emit_edges` target, or a fallback-chain member. Each DSL
/// is authored JSON with the module key names, so a serde rename fails here.
#[trace("TC-1870", "FR-075-AC-11")]
#[test]
fn required_sections_read_every_locator_of_the_typed_dsl() {
    use quire_rs::extract::dsl::ExtractionDsl;
    use quire_rs::semantic::RequiredSections;

    let sections = |properties, invariants, operations| RequiredSections {
        properties,
        invariants,
        operations,
    };
    let cases = [
        (
            "match: defaulted and explicit `required`, a fallback chain",
            json!({ "yield_pattern": { "match": {
                "fields": { "from": "table_row", "under_section": "Properties" },
                "rules": [
                    { "from": "list_item", "after_heading": "Invariants", "required": false },
                    { "from": "section_body", "after_heading": "Invariants" }
                ],
                "ops": { "from": "code_block", "under_section": "Operations", "required": false },
                "id": { "from": "frontmatter_field", "path": ["id"] }
            } } }),
            sections(true, true, false),
        ),
        (
            "per_match: a required Operations locator",
            json!({ "yield_pattern": {
                "iterate_over": { "section_path": ["Operations"], "kind": "heading" },
                "per_match": {
                    "body": { "from": "code_block", "under_section": "Operations", "required": true },
                    "fields": { "from": "table_row", "under_section": "Properties", "required": false }
                }
            } }),
            sections(false, false, true),
        ),
        (
            "emit_edges: a locator target, beside a static one",
            json!({
                "yield_pattern": { "match": {
                    "id": { "from": "frontmatter_field", "path": ["id"] }
                } },
                "emit_edges": [
                    { "type": "references", "target": { "from": "table_row", "under_section": "Properties" } },
                    { "type": "references", "target": [
                        { "from": "heading", "level": 1 },
                        { "from": "list_item", "under_section": "Invariants" }
                    ] },
                    { "type": "references", "target": "FR-001" }
                ]
            }),
            sections(true, true, false),
        ),
    ];
    for (case, authored, expected) in cases {
        let typed: ExtractionDsl = serde_json::from_value(authored)
            .unwrap_or_else(|e| panic!("{case}: the authored DSL deserializes: {e}"));
        assert_eq!(
            RequiredSections::from_extraction(&typed),
            expected,
            "{case}"
        );
    }
}

/// The golden table with its prose `## Relationships` replaced by a table of
/// `rows`.
fn related_document(rows: &str) -> String {
    quire_fixtures::mapping_fixture("config-version.table.md").replace(
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
    let context = json!({ "contractVersion": "1.0.0", "package": "agent-ix/spec-objects-fixture", "exports": ["entity"], "imports": {}, "mappings": ["relationships"] });
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

#[trace("TC-2152", "FR-076-AC-23")]
// The own-package check folds ASCII case on both sides and lowers the target
// in the declared package's spelling: a frontmatter relationship written in
// that spelling is a duplicate of the row. A foreign package without an
// import is refused as `target-not-id`.
#[test]
fn relationship_own_package_check_folds_ascii_case() {
    let tmp = TempDir::new().unwrap();
    let registry = related_registry(&tmp);
    let entity = registry.archetype("entity").unwrap();
    let check = |package: &str, frontmatter: &str, row_target: &str| {
        let document =
            related_document(&format!("| overlay | references | {row_target} | 1..1 |\n"))
                .replacen(
                    "type: FR\n",
                    &format!(
                "type: FR\nrelationships:\n  - target: \"{frontmatter}\"\n    type: references\n"
            ),
                    1,
                );
        relationship_findings(quire_rs::validate_document_in_bundle(
            &registry, entity, &document, package, None,
        ))
        .0
    };

    // (declared package, frontmatter identity in that spelling, row cell)
    for (package, frontmatter, cell) in [
        ("acme/widgets", "ix://acme/widgets/FR-005", "FR-005"),
        (
            "acme/widgets",
            "ix://acme/widgets/FR-005",
            "ix://Acme/Widgets/FR-005",
        ),
        (
            "Acme/Widgets",
            "ix://Acme/Widgets/FR-005",
            "ix://acme/widgets/FR-005",
        ),
        // A frontmatter identity in a different case than the declared
        // package is normalised to the declared spelling (FR-076-AC-23).
        ("acme/widgets", "ix://Acme/Widgets/FR-005", "FR-005"),
        (
            "acme/widgets",
            "ix://Acme/Widgets/FR-005",
            "ix://acme/widgets/FR-005",
        ),
        ("Acme/Widgets", "ix://acme/widgets/FR-005", "FR-005"),
    ] {
        let errors = check(package, frontmatter, cell);
        assert_eq!(errors.len(), 1, "{package} / {cell}: {errors:?}");
        assert!(
            errors[0].starts_with("semantic.duplicate-model-entry"),
            "{package} / {cell}: {errors:?}"
        );
    }

    // A foreign package without an import is not an own-package target.
    let errors = check(
        "acme/widgets",
        "ix://acme/widgets/FR-006",
        "ix://other/repo/FR-005",
    );
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].starts_with("semantic.invalid-model-cell")
            && errors[0].contains("is not a bundle artifact id"),
        "{errors:?}"
    );
}

#[trace("TC-1868", "FR-076-AC-15")]
// validate_bundle qualifies relationships under the repository identity the
// Spec declares (PLAT-48); without one the table is unavailable with
// `no-bundle-package`.
#[test]
fn validate_bundle_passes_the_declared_bundle_package() {
    use quire_rs::corpus::{validate_bundle, BundlePosture, Spec};

    let tmp = TempDir::new().unwrap();
    let registry = related_registry(&tmp);
    let root = tmp.path().join("bundle");
    fs::create_dir_all(root.join("spec")).unwrap();
    fs::write(
        root.join("spec/FR-006.md"),
        related_document("| overlay | holds | FR-005 | 1..1 |\n")
            .replace("type: FR\n", "type: entity\n"),
    )
    .unwrap();
    let messages = |spec: &Spec| -> Vec<String> {
        let report = validate_bundle(spec, &registry, BundlePosture::Strict, &root, &root);
        report
            .errors
            .iter()
            .chain(report.warnings.iter())
            .map(|f| f.message.clone())
            .collect()
    };

    // Control: no declared identity -> one `no-bundle-package` advisory.
    let spec = Spec::from_path(&root);
    assert_eq!(spec.bundle_package(), None);
    let without = messages(&spec);
    assert!(
        without
            .iter()
            .any(|m| m.contains("relationships-no-bundle-package")),
        "{without:?}"
    );

    // Declared identity: the row is checked (`holds` is not a registered verb).
    let spec = spec.with_bundle_package(BUNDLE_PACKAGE);
    assert_eq!(spec.bundle_package(), Some(BUNDLE_PACKAGE));
    let with = messages(&spec);
    assert!(
        !with.iter().any(|m| m.contains("no-bundle-package")),
        "{with:?}"
    );
    assert!(with.iter().any(|m| m.contains("holds")), "{with:?}");
}

/// The fixture module plus a `process` type whose `data_schema` declares the
/// typed relation key `emits` when `typed` is set, whose `allowed_links`
/// name `emits`, and whose `body_extraction` keys a `table_row` locator
/// `emits` when `locator` is set; the module names `relationships` when
/// `relationships` is set.
fn process_module(
    tmp: &TempDir,
    name: &str,
    relationships: bool,
    typed: bool,
    locator: bool,
) -> PathBuf {
    module(tmp, name, |m, root| {
        let core = format!(
            "https://schemas.agent-ix.org/semantic-core/{}/",
            embedded::embedded_semantic_core_version()
        );
        let mut properties = json!({
            "fields": { "type": "array", "items": { "$ref": format!("{core}FieldDecl.json") } }
        });
        if typed {
            properties["emits"] =
                json!({ "type": "array", "items": { "$ref": format!("{core}SemanticId.json") } });
        }
        let schema = json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": "https://schemas.agent-ix.org/agent-ix/spec-objects-fixture/0.1.0/Process.json",
            "type": "object",
            "properties": properties,
        });
        fs::write(
            root.join("schemas/Process.json"),
            serde_json::to_vec(&schema).unwrap(),
        )
        .unwrap();
        if relationships {
            semantic(m).insert("mappings".into(), vec!["relationships"].into());
        }
        let mut matches = serde_yaml::Mapping::new();
        matches.insert(
            "id".into(),
            serde_yaml::from_str("{ from: frontmatter_field, path: [id], required: true }")
                .unwrap(),
        );
        if locator {
            matches.insert(
                "emits".into(),
                serde_yaml::from_str(
                    "{ from: table_row, under_section: Emits, assert: { columns: [Event] } }",
                )
                .unwrap(),
            );
        }
        let mut process = serde_yaml::Mapping::new();
        process.insert("name".into(), "process".into());
        process.insert(
            "data_schema".into(),
            serde_yaml::from_str("{ schema: schemas/Process.json }").unwrap(),
        );
        process.insert(
            "allowed_links".into(),
            serde_yaml::from_str("{ emits: [event] }").unwrap(),
        );
        let mut pattern = serde_yaml::Mapping::new();
        pattern.insert("match".into(), serde_yaml::Value::Mapping(matches));
        let mut body = serde_yaml::Mapping::new();
        body.insert("yield_pattern".into(), serde_yaml::Value::Mapping(pattern));
        process.insert("body_extraction".into(), serde_yaml::Value::Mapping(body));
        m["object_types"]
            .as_sequence_mut()
            .unwrap()
            .push(serde_yaml::Value::Mapping(process));
    })
}

#[trace("TC-2005", "FR-069-AC-13")]
// A `table_row` locator keyed on a typed relation key under the
// `relationships` mapping refuses every archetype of the module.
#[test]
fn locator_on_a_typed_relation_key_is_duplicate_authority() {
    let tmp = TempDir::new().unwrap();
    let registry = load(&process_module(&tmp, "dup", true, true, true));
    let failures = registry.failures();
    let archetypes: Vec<&str> = failures.iter().map(|f| f.archetype.as_str()).collect();
    assert_eq!(
        archetypes,
        ["entity", "enumeration", "process"],
        "{failures:?}"
    );
    for f in failures {
        assert!(
            f.reason.starts_with("semantic.duplicate-authority"),
            "{}",
            f.reason
        );
        assert!(
            f.reason.starts_with(
                "semantic.duplicate-authority: object_types[process].body_extraction: "
            ),
            "{}",
            f.reason
        );
        for needle in ["dup", "process", "emits"] {
            assert!(f.reason.contains(needle), "{needle}: {}", f.reason);
        }
    }
    assert!(registry.archetype("process").is_none());
    assert!(registry.archetype("entity").is_none());

    // The same locator under `yield_pattern.per_match` is refused too.
    let root = process_module(&tmp, "dup-per-match", true, true, false);
    let manifest_path = root.join("manifest.yaml");
    let mut manifest: serde_yaml::Value =
        serde_yaml::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    manifest["object_types"][2]["body_extraction"]["yield_pattern"] = serde_yaml::from_str(
        "{ iterate_over: { section_path: [Steps], kind: heading }, per_match: { emits: { from: table_row, under_section: Emits, assert: { columns: [Event] } } } }",
    )
    .unwrap();
    fs::write(&manifest_path, serde_yaml::to_string(&manifest).unwrap()).unwrap();
    let registry = load(&root);
    assert_eq!(registry.failures().len(), 3, "{:?}", reasons(&registry));
    assert!(codes(&registry)
        .iter()
        .all(|c| c == "semantic.duplicate-authority"));

    for (name, relationships, typed, locator) in [
        ("no-locator", true, true, false),
        ("untyped", true, false, true),
        ("no-mapping", false, true, true),
    ] {
        let registry = load(&process_module(&tmp, name, relationships, typed, locator));
        assert!(
            registry.failures().is_empty(),
            "{name}: {:?}",
            reasons(&registry)
        );
        assert!(registry.archetype("process").is_some(), "{name}");
    }
}

#[trace("TC-2081", "FR-069-AC-14")]
// PLAT-1299: a semantic refusal's reason names its manifest location.
#[test]
fn refusals_carry_their_manifest_path() {
    let tmp = tempfile::tempdir().unwrap();
    let cases: Vec<(&str, Mutate, &str, &str)> = vec![
        (
            "unknown-key",
            Box::new(|m, _| {
                semantic(m).insert("extra".into(), 1.into());
            }),
            "semantic.unknown-key",
            "semantic.unknown-key: semantic: ",
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
            "semantic.export-undeclared: semantic.exports.ghost: ",
        ),
        (
            "ambiguous",
            Box::new(|m, _| {
                entity_schema_ref(m).insert("type".into(), "object".into());
            }),
            "semantic.data-schema-ambiguous",
            "semantic.data-schema-ambiguous: object_types[entity].data_schema: ",
        ),
        (
            "no-block",
            Box::new(|m, _| {
                m.as_mapping_mut().unwrap().remove("semantic");
            }),
            "semantic.data-schema-reference-without-block",
            "semantic.data-schema-reference-without-block: object_types[entity].data_schema: ",
        ),
        (
            "missing-file",
            Box::new(|_, root| fs::remove_file(root.join("schemas/Entity.json")).unwrap()),
            "semantic.data-schema-missing",
            "semantic.data-schema-missing: object_types[entity].data_schema.schema: ",
        ),
    ];
    for (name, mutate, code, prefix) in cases {
        let root = module(&tmp, name, mutate);
        let r = reasons(&load(&root));
        assert!(
            r.iter().any(|x| x.starts_with(prefix)),
            "{name} ({code}) carries its path: {r:?}"
        );
        let path = prefix.trim_start_matches(code).trim_matches([':', ' ']);
        let tail = &r.iter().find(|x| x.starts_with(prefix)).unwrap()[prefix.len()..];
        assert!(
            !tail.contains(path) || path == "semantic",
            "{name}: path appears once: {r:?}"
        );
    }
    assert_eq!(
        quire_rs::semantic::SemanticFailure::error("semantic.x", "", "msg").reason(),
        "semantic.x: msg"
    );
}

#[trace("TC-2081", "FR-069-AC-14")]
// The duplicate-authority path names the section the type is declared in.
#[test]
fn duplicate_authority_path_names_its_section() {
    let tmp = tempfile::tempdir().unwrap();
    let root = process_module(&tmp, "dup-artifact-types", true, true, true);
    let manifest_path = root.join("manifest.yaml");
    let mut manifest: serde_yaml::Value =
        serde_yaml::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    manifest["artifact_types"] = manifest["object_types"].clone();
    manifest["object_types"] = serde_yaml::Value::Sequence(Vec::new());
    fs::write(&manifest_path, serde_yaml::to_string(&manifest).unwrap()).unwrap();
    let r = reasons(&load(&root));
    assert!(
        r.iter().all(|x| x.starts_with(
            "semantic.duplicate-authority: artifact_types[process].body_extraction: "
        )),
        "{r:?}"
    );
    assert!(!r.is_empty());
}
