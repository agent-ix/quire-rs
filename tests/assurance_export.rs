//! FR-067/FR-068 assurance export contract (TC-1084..TC-1098).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use ix_trace_rs::trace;
use jsonschema::{Draft, JSONSchema};
use quire_rs::assurance::{
    AssuranceFreshness, AssuranceModulePremise, AssuranceRelation, AssuranceResolution,
    RelationAvailability, ASSURANCE_V1_SCHEMA,
};
use quire_rs::corpus::walk::load_repo;
use quire_rs::diagnostic::Diagnostic;
use quire_rs::symbols::trace::{
    ImplementsRelation, SymbolGraph, TraceProvenance, VerifiesRelation,
};
use quire_rs::symbols::{extract_tree, SymbolExtraction, SymbolKind};
use quire_rs::{
    build_assurance_export, read_assurance_export, AcceptedAssurancePremises, AssuranceError,
    AssuranceExport, AssuranceInput, AssuranceSource, Registry, Spec,
};
use serde_json::Value;
use tempfile::TempDir;

const REVISION: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn write(root: &Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create fixture parent");
    }
    fs::write(path, contents).expect("write fixture");
}

fn registry() -> Registry {
    let manifest = br#"
name: assurance-fixture
version: 1.2.3
artifact_types:
- name: FR
- name: StR
- name: NFR
traceability:
  trace_targets:
  - name: acceptance-criterion
    archetype: FR
    section: Acceptance Criteria
    id_column: ID
  obligations:
  - name: acceptance-criterion
    target: acceptance-criterion
    statement_column: Criteria
    method_column: Verification
  required_relations:
  - name: fr-has-need
    from: FR
    edges: [satisfies]
    to: [StR]
    direction: outgoing
    check: orphan-fr
  - name: nfr-has-owner
    from: NFR
    edges: [owned_by]
    direction: outgoing
    check: orphan-nfr
"#;
    Registry::from_inline_parts(manifest, &BTreeMap::new()).expect("fixture registry")
}

struct Fixture {
    _tmp: TempDir,
    registry: Registry,
    spec: Spec,
    symbols: SymbolExtraction,
    graph: SymbolGraph,
}

impl Fixture {
    fn new() -> Self {
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path();
        write(
            root,
            "spec/FR-001.md",
            "---\nid: FR-001\ntype: FR\ntitle: Linked\nrelationships:\n  - target: ix://agent-ix/fixture/StR-001\n    type: satisfies\n---\n\n## Acceptance Criteria\n\n| ID | Criteria | Verification |\n|---|---|---|\n| FR-001-AC-1 | The export contains one linked requirement. | Test (TC-1092) |\n",
        );
        write(
            root,
            "spec/FR-002.md",
            "---\nid: FR-002\ntype: FR\ntitle: Unlinked\n---\n\n## Description\n\nThe engine shall expose this artifact.\n",
        );
        write(
            root,
            "spec/StR-001.md",
            "---\nid: StR-001\ntype: StR\ntitle: Need\n---\n\n## Stakeholder Need\n\nReviewers require grounded data.\n",
        );
        write(
            root,
            "src/lib.rs",
            "/// Implements: FR-001\npub fn production() {}\n\n#[test]\nfn tc1095_evidence() { assert_eq!(2 + 2, 4); }\n",
        );

        let mut load = load_repo(&root.join("spec"));
        load.diagnostics.push(Diagnostic::DocumentUnreadable {
            path: root.join("spec/unread.md"),
            reason: "permission denied".to_string(),
        });
        let spec = Spec::from_repo(load);
        let symbols = extract_tree(root);
        let production = symbols
            .symbols
            .iter()
            .find(|symbol| symbol.qualified_name.ends_with("production"))
            .expect("production symbol");
        let evidence = symbols
            .symbols
            .iter()
            .find(|symbol| symbol.kind == SymbolKind::TestFunction)
            .expect("test symbol");
        let graph = SymbolGraph {
            verifies: vec![VerifiesRelation {
                symbol_id: evidence.id.clone(),
                symbol: evidence.qualified_name.clone(),
                path: evidence.path.clone(),
                trace_id: "TC-1095".to_string(),
                provenance: TraceProvenance::Canonical,
                form: "rust-trace".to_string(),
                line: evidence.line,
            }],
            implements: vec![ImplementsRelation {
                symbol_id: production.id.clone(),
                symbol: production.qualified_name.clone(),
                path: production.path.clone(),
                trace_id: "FR-001".to_string(),
                form: "rust-implements".to_string(),
            }],
            ..SymbolGraph::default()
        };
        Self {
            _tmp: tmp,
            registry: registry(),
            spec,
            symbols,
            graph,
        }
    }

    fn export(&self) -> Result<AssuranceExport, AssuranceError> {
        build_assurance_export(AssuranceInput {
            spec: &self.spec,
            registry: &self.registry,
            corpus_root: self._tmp.path().join("spec").as_path(),
            symbols: &self.symbols,
            symbol_graph: &self.graph,
            source: AssuranceSource {
                repository: "agent-ix/fixture".to_string(),
                revision: REVISION.to_string(),
            },
        })
    }
}

fn schema() -> JSONSchema {
    let value: Value = serde_json::from_str(ASSURANCE_V1_SCHEMA).expect("schema JSON");
    JSONSchema::options()
        .with_draft(Draft::Draft202012)
        .compile(&value)
        .expect("schema compiles")
}

fn golden() -> Value {
    serde_json::from_str(include_str!("fixtures/assurance/v1.json")).expect("golden JSON")
}

#[trace("TC-1084", "FR-067-AC-1")]
#[test]
fn tc1084_assurance_schema_compiles_and_accepts_the_complete_fixture() {
    let value: Value = serde_json::from_str(ASSURANCE_V1_SCHEMA).expect("schema JSON");
    assert_eq!(
        value["$schema"],
        "https://json-schema.org/draft/2020-12/schema"
    );
    assert!(value["$id"]
        .as_str()
        .expect("schema id")
        .ends_with("assurance-v1.schema.json"));
    assert!(schema().validate(&golden()).is_ok());
}

#[trace("TC-1085", "FR-067-AC-2")]
#[test]
fn tc1085_construction_refuses_incomplete_premises_atomically() {
    let fixture = Fixture::new();
    let mut source = AssuranceSource {
        repository: String::new(),
        revision: REVISION.to_string(),
    };
    let build = |source: AssuranceSource, registry: &Registry| {
        build_assurance_export(AssuranceInput {
            spec: &fixture.spec,
            registry,
            corpus_root: fixture._tmp.path().join("spec").as_path(),
            symbols: &fixture.symbols,
            symbol_graph: &fixture.graph,
            source,
        })
    };
    assert!(matches!(
        build(source.clone(), &fixture.registry),
        Err(AssuranceError::EmptyRepository)
    ));
    source.repository = "agent-ix/fixture".to_string();
    source.revision = "main".to_string();
    assert!(matches!(
        build(source.clone(), &fixture.registry),
        Err(AssuranceError::InvalidRevision { .. })
    ));

    let unversioned =
        Registry::from_inline_parts(b"name: unversioned\nartifact_types: []\n", &BTreeMap::new())
            .expect("unversioned registry");
    source.revision = REVISION.to_string();
    assert!(matches!(
        build(source.clone(), &unversioned),
        Err(AssuranceError::MissingModuleVersion { .. })
    ));

    let unnamed =
        Registry::from_inline_parts(b"version: 1.0.0\nartifact_types: []\n", &BTreeMap::new())
            .expect("unnamed tolerant registry");
    assert!(matches!(
        build(source.clone(), &unnamed),
        Err(AssuranceError::MissingModuleName { .. })
    ));

    let missing = Registry::from_inline_parts(
        b"name: broken\nversion: 1.0.0\nartifact_types:\n- name: FR\n  frontmatter_schema_ref: missing.json\n",
        &BTreeMap::new(),
    )
    .expect("tolerant registry");
    assert!(matches!(
        build(source, &missing),
        Err(AssuranceError::ArchetypeLoadFailure { .. })
    ));

    let outside = build_assurance_export(AssuranceInput {
        spec: &fixture.spec,
        registry: &fixture.registry,
        corpus_root: fixture._tmp.path().join("different-root").as_path(),
        symbols: &fixture.symbols,
        symbol_graph: &fixture.graph,
        source: AssuranceSource {
            repository: "agent-ix/fixture".to_string(),
            revision: REVISION.to_string(),
        },
    });
    assert!(matches!(
        outside,
        Err(AssuranceError::PathOutsideRoot { .. })
    ));
}

#[trace("TC-1086", "FR-067-AC-3")]
#[trace("TC-1088", "FR-067-AC-5")]
#[test]
fn tc1086_tc1088_premises_and_bytes_are_deterministic() {
    let fixture = Fixture::new();
    let first = fixture.export().expect("first export");
    let second = fixture.export().expect("second export");
    assert_eq!(
        first.to_json_bytes().unwrap(),
        second.to_json_bytes().unwrap()
    );
    assert_eq!(first.modules.len(), 1);
    assert_eq!(first.modules[0].name, "assurance-fixture");
    assert!(first.modules[0]
        .schemas
        .windows(2)
        .all(|pair| pair[0] < pair[1]));

    let mut changed = first.clone();
    changed.source.revision = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string();
    let mut original_value = serde_json::to_value(&first).unwrap();
    let mut changed_value = serde_json::to_value(&changed).unwrap();
    original_value["source"]["revision"] = Value::Null;
    changed_value["source"]["revision"] = Value::Null;
    assert_eq!(original_value, changed_value);

    let manifest = b"name: digest-fixture\nversion: 1.0.0\nartifact_types:\n- name: FR\n  frontmatter_schema_ref: fr.json\n";
    let registry_with = |schema: &str| {
        let mut schemas = BTreeMap::new();
        schemas.insert("fr.json".to_string(), schema.to_string());
        Registry::from_inline_parts(manifest, &schemas).expect("digest registry")
    };
    let digest_of = |registry: &Registry| {
        build_assurance_export(AssuranceInput {
            spec: &fixture.spec,
            registry,
            corpus_root: fixture._tmp.path().join("spec").as_path(),
            symbols: &fixture.symbols,
            symbol_graph: &fixture.graph,
            source: AssuranceSource {
                repository: "agent-ix/fixture".to_string(),
                revision: REVISION.to_string(),
            },
        })
        .expect("digest export")
        .modules[0]
            .schemas[0]
            .schema_digest
            .clone()
    };
    let ordered = registry_with(
        r#"{"type":"object","properties":{"a":{"type":"string"},"b":{"type":"number"}}}"#,
    );
    let reordered = registry_with(
        r#"{ "properties": { "b": { "type": "number" }, "a": { "type": "string" } }, "type": "object" }"#,
    );
    let changed = registry_with(
        r#"{"type":"object","properties":{"a":{"type":"boolean"},"b":{"type":"number"}}}"#,
    );
    assert_eq!(digest_of(&ordered), digest_of(&reordered));
    assert_ne!(digest_of(&ordered), digest_of(&changed));
}

#[trace("TC-1087", "FR-067-AC-4", "StR-007-VC-2")]
#[test]
fn tc1087_reader_rejects_unsupported_premises_before_returning_records() {
    let export: AssuranceExport = serde_json::from_value(golden()).expect("typed golden");
    let accepted = AcceptedAssurancePremises::from_export(&export);
    assert_eq!(
        read_assurance_export(include_bytes!("fixtures/assurance/v1.json"), &accepted).unwrap(),
        export
    );

    let mut value = golden();
    value["format_version"] = 2.into();
    assert!(matches!(
        read_assurance_export(&serde_json::to_vec(&value).unwrap(), &accepted),
        Err(AssuranceError::UnsupportedFormatVersion { version: 2 })
    ));

    let mut rejected = accepted.clone();
    rejected.modules[0].version = "9.9.9".to_string();
    assert!(matches!(
        read_assurance_export(include_bytes!("fixtures/assurance/v1.json"), &rejected),
        Err(AssuranceError::UnacceptedModuleVersion { .. })
    ));
    rejected = accepted.clone();
    rejected.modules[0].schemas[0].schema_digest = "0".repeat(64);
    assert!(matches!(
        read_assurance_export(include_bytes!("fixtures/assurance/v1.json"), &rejected),
        Err(AssuranceError::UnacceptedSchemaDigest { .. })
    ));
    rejected = accepted.clone();
    rejected.modules.clear();
    assert!(matches!(
        read_assurance_export(include_bytes!("fixtures/assurance/v1.json"), &rejected),
        Err(AssuranceError::UnacceptedModule { .. })
    ));

    let mut duplicate_modules = golden();
    let duplicate = duplicate_modules["modules"][0].clone();
    duplicate_modules["modules"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    assert!(matches!(
        read_assurance_export(&serde_json::to_vec(&duplicate_modules).unwrap(), &accepted),
        Err(AssuranceError::DuplicateModulePremise { .. })
    ));

    let mut duplicate_schemas = golden();
    let mut duplicate = duplicate_schemas["modules"][0]["schemas"][0].clone();
    duplicate["schema_digest"] = Value::String("f".repeat(64));
    duplicate_schemas["modules"][0]["schemas"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    assert!(matches!(
        read_assurance_export(&serde_json::to_vec(&duplicate_schemas).unwrap(), &accepted),
        Err(AssuranceError::DuplicateSchemaPremise { .. })
    ));
}

#[trace("TC-1089", "FR-067-AC-6", "StR-007-VC-4")]
#[test]
fn tc1089_v1_fixture_pins_fields_variants_and_state_tokens() {
    let export: AssuranceExport = serde_json::from_value(golden()).expect("typed golden");
    assert_eq!(export.relations.len(), 3);
    assert_eq!(export.relation_observations.len(), 4);
    let mut removed = golden();
    removed.as_object_mut().unwrap().remove("artifacts");
    let compiled = schema();
    let errors: Vec<_> = compiled
        .validate(&removed)
        .expect_err("required field mutation must fail")
        .collect();
    assert!(!errors.is_empty());
    let mut renamed = golden();
    renamed["relations"][0]["resolution"] = "present".into();
    assert!(schema().validate(&renamed).is_err());
}

#[trace("TC-1091", "FR-068-AC-1", "StR-007-VC-1")]
#[trace("TC-1092", "FR-068-AC-2")]
#[trace("TC-1093", "FR-068-AC-3")]
#[test]
fn tc1091_tc1093_records_preserve_authoritative_identifiers_and_locators() {
    let fixture = Fixture::new();
    let export = fixture.export().expect("export");
    assert_eq!(export.artifacts.len(), 3);
    assert!(export
        .artifacts
        .iter()
        .all(|artifact| artifact.locator.line == 1 && artifact.locator.digest.len() == 64));
    assert_eq!(export.obligations.len(), 1);
    assert_eq!(export.obligations[0].id, "FR-001-AC-1");
    assert_eq!(
        export.obligations[0].statement_hash,
        quire_rs::statement_hash(&export.obligations[0].statement)
    );
    assert_eq!(export.symbols.len(), fixture.symbols.symbols.len());
    for projected in &export.symbols {
        let source = fixture
            .symbols
            .symbols
            .iter()
            .find(|symbol| symbol.id == projected.id)
            .expect("symbol identity preserved");
        assert_eq!(projected.qualified_name, source.qualified_name);
        assert_eq!(projected.locator.line, source.line);
    }
}

#[trace("TC-1094", "FR-068-AC-4")]
#[trace("TC-1095", "FR-068-AC-5")]
#[trace("TC-1097", "FR-068-AC-7")]
#[test]
fn tc1094_tc1097_relation_variants_preserve_graph_meaning() {
    let fixture = Fixture::new();
    let export = fixture.export().expect("export");
    let corpus: Vec<_> = export
        .relations
        .iter()
        .filter(|relation| matches!(relation, AssuranceRelation::Corpus { .. }))
        .collect();
    assert_eq!(corpus.len(), fixture.spec.edges().len());
    assert!(
        export.relation_kinds.iter().any(|kind| {
            kind.kind == "owned_by"
                && kind
                    .sources
                    .contains(&quire_rs::assurance::RelationKindSource::RequiredRelation)
        }),
        "a declared kind with zero observed edges remains explicitly available"
    );
    assert!(corpus.iter().any(|relation| matches!(
        relation,
        AssuranceRelation::Corpus {
            edge_type,
            resolution: AssuranceResolution::Resolved,
            ..
        } if edge_type == "satisfies"
    )));
    assert!(export.relations.iter().any(|relation| matches!(
        relation,
        AssuranceRelation::Verifies {
            freshness: AssuranceFreshness::Unknown,
            provenance,
            ..
        } if provenance == "canonical"
    )));
    assert!(export.relations.iter().any(|relation| matches!(
        relation,
        AssuranceRelation::Implements {
            freshness: AssuranceFreshness::NotApplicable,
            ..
        }
    )));
    assert!(!export.relations.iter().any(|relation| match relation {
        AssuranceRelation::Corpus { freshness, .. }
        | AssuranceRelation::Verifies { freshness, .. }
        | AssuranceRelation::Implements { freshness, .. } => matches!(
            freshness,
            AssuranceFreshness::Current | AssuranceFreshness::Suspect
        ),
    }));
}

#[trace("TC-1096", "FR-068-AC-6", "StR-007-VC-3")]
#[test]
fn tc1096_relation_observations_keep_all_four_states_distinct() {
    let fixture = Fixture::new();
    let observations = fixture.export().expect("export").relation_observations;
    let states: BTreeMap<_, _> = observations
        .iter()
        .map(|item| {
            (
                (item.declaration.as_str(), item.subject.as_deref()),
                item.availability,
            )
        })
        .collect();
    assert_eq!(
        states.get(&("fr-has-need", Some("FR-001"))),
        Some(&RelationAvailability::Available)
    );
    assert_eq!(
        states.get(&("fr-has-need", Some("FR-002"))),
        Some(&RelationAvailability::Missing)
    );
    assert_eq!(
        states.get(&("nfr-has-owner", None)),
        Some(&RelationAvailability::NotApplicable)
    );
    assert!(observations.iter().any(|item| {
        item.availability == RelationAvailability::Unknown
            && item.reason.as_deref() == Some("permission denied")
    }));
}

#[trace("TC-1098", "FR-068-AC-8")]
#[test]
fn tc1098_repeated_projection_is_byte_identical() {
    let fixture = Fixture::new();
    assert_eq!(
        fixture.export().unwrap().to_json_bytes().unwrap(),
        fixture.export().unwrap().to_json_bytes().unwrap()
    );
}

#[test]
fn accepted_module_premises_remain_sorted_contract_values() {
    let export: AssuranceExport = serde_json::from_value(golden()).expect("typed golden");
    let modules: Vec<&AssuranceModulePremise> = export.modules.iter().collect();
    assert_eq!(modules[0].name, "fixture-module");
}
