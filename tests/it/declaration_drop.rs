//! A declaration that cannot load drops itself, never the model (FR-058-AC-38 onward).

use std::fs;
use std::path::PathBuf;

use ix_trace_rs::trace;
use quire_rs::Registry;
use tempfile::TempDir;

const FIRST: &str = "name: decl-one
version: 0.1.0
description: Declares a relation and a carried field
archetypes:
- name: alpha
  frontmatter_schema_ref: schemas/alpha.json
edge_types:
  satisfies: {description: satisfies, category: traceability}
traceability:
  required_relations:
  - name: R
    from: alpha
    direction: outgoing
    edges: [satisfies]
    check: r-check
  carried_fields:
  - name: tier
    from: [alpha]
    key: tier
";

const SECOND: &str = "name: decl-two
version: 0.1.0
description: Declares one valid and one unloadable flow rule
archetypes:
- name: beta
  frontmatter_schema_ref: schemas/beta.json
traceability:
  flow_rules:
  - name: ok
    subject: [beta]
    match: [{path: [], field: id, present: true}]
    check: ok-check
    rationale: Valid.
  - name: bad
    subject: [nowhere]
    match: [{path: [], field: id, present: true}]
    check: bad-check
    rationale: Names an archetype nothing declares.
";

fn write_modules(manifests: &[(&str, &str)]) -> (TempDir, Vec<PathBuf>) {
    let parent = tempfile::tempdir().expect("modules");
    let mut roots = Vec::new();
    for (name, manifest) in manifests {
        let root = parent.path().join(name);
        fs::create_dir_all(&root).expect("module dir");
        fs::write(root.join("manifest.yaml"), manifest).expect("manifest");
        fs::create_dir_all(root.join("schemas")).expect("schemas dir");
        for archetype in ["alpha", "beta"] {
            fs::write(
                root.join("schemas").join(format!("{archetype}.json")),
                r#"{"type":"object","properties":{"id":{"type":"string"}}}"#,
            )
            .expect("schema");
        }
        roots.push(root);
    }
    (parent, roots)
}

fn load(manifests: &[(&str, &str)]) -> (TempDir, Registry) {
    let (parent, roots) = write_modules(manifests);
    let paths: Vec<&std::path::Path> = roots.iter().map(PathBuf::as_path).collect();
    let registry = Registry::load_module_set(&paths).expect("modules load");
    (parent, registry)
}

#[test]
#[trace("FR-058-AC-38", "FR-096-AC-29")]
fn an_unloadable_flow_rule_drops_only_itself() {
    let (_dir, registry) = load(&[("decl-one", FIRST), ("decl-two", SECOND)]);

    let failures = registry.failures();
    assert_eq!(failures.len(), 1, "{failures:?}");
    assert_eq!(failures[0].module, "decl-two");
    assert!(failures[0].reason.contains("bad"), "{}", failures[0].reason);

    let model = registry
        .traceability()
        .expect("the merged model survives one dropped declaration");
    assert_eq!(model.required_relations.len(), 1);
    assert_eq!(model.required_relations[0].name, "R");
    assert_eq!(model.carried_fields.len(), 1);
    assert_eq!(model.flow_rules.len(), 1);
    assert_eq!(model.flow_rules[0].name, "ok");
}
