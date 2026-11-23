//! FR-067/FR-068 assurance export contract (TC-1084..TC-1098).

use std::collections::BTreeMap;
use std::fs;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;

use ix_trace_rs::trace;
use jsonschema::{Draft, Validator};
use quire_rs::assurance::{
    AssuranceFreshness, AssuranceLocator, AssuranceModulePremise, AssuranceObligation,
    AssuranceRelation, AssuranceResolution, RelationAvailability, ASSURANCE_V1_SCHEMA,
};
use quire_rs::corpus::walk::load_repo;
use quire_rs::diagnostic::Diagnostic;
use quire_rs::symbols::trace::{ImplementsRelation, SymbolGraph, VerifiesRelation};
use quire_rs::symbols::{extract_tree, SymbolExtraction, SymbolKind};
use quire_rs::{
    build_assurance_export, read_assurance_export, AcceptedAssurancePremises, AssuranceError,
    AssuranceExport, AssuranceInput, Registry, Spec,
};
use serde_json::Value;
use tempfile::TempDir;

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
                form: "rust-trace".to_string(),
                line: evidence.line,
                column: evidence.column,
                kind: evidence.kind,
                ignored: evidence.ignored,
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
        })
    }
}

fn schema() -> Validator {
    let value: Value = serde_json::from_str(ASSURANCE_V1_SCHEMA).expect("schema JSON");
    jsonschema::options()
        .with_draft(Draft::Draft202012)
        .build(&value)
        .expect("schema compiles")
}

fn golden() -> Value {
    serde_json::from_str(include_str!("../fixtures/assurance/v1.json")).expect("golden JSON")
}

fn obligation_export(reverse_creation_order: bool, unrelated_body: &str) -> AssuranceExport {
    let manifest = br#"
name: obligation-export-fixture
version: 1.0.0
artifact_types:
- name: FR
- name: NFR
- name: StR
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
    parameters:
      threshold: Threshold
    criticality_column: Priority
    criticality_field: criticality
  - name: nfr-metric
    archetype: NFR
    section: Measurement and Evaluation
    id_format: "{document}-M-{row}"
    statement_column: Metric
    method_column: Method
    parameters:
      target: Target
      threshold: Threshold
"#;
    let registry = Registry::from_inline_parts(manifest, &BTreeMap::new()).expect("registry");
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    let documents = [
        (
            "spec/FR-001.md",
            "---\nid: FR-001\ntype: FR\ncriticality: elevated\n---\n\n## Acceptance Criteria\n\n| ID | Criteria | Verification | Threshold | Priority |\n|---|---|---|---|---|\n| FR-001-AC-1 | Repeat this exact acceptance statement. | Test (TC-1901) | 5 ms | high |\n| FR-001-AC-2 | Repeat this exact acceptance statement. | Analysis (TC-1902) | 10 ms |  |\n",
        ),
        (
            "spec/FR-002.md",
            "---\nid: FR-002\ntype: FR\ncriticality: severe\n---\n\n## Acceptance Criteria\n\n| ID | Criteria | Verification | Threshold | Priority |\n|---|---|---|---|---|\n| FR-002-AC-1 | Repeat this exact acceptance statement. | Demonstration (TC-1903) | 1 s |  |\n",
        ),
        (
            "spec/NFR-003.md",
            "---\nid: NFR-003\ntype: NFR\n---\n\n## Measurement and Evaluation\n\n| Metric | Method | Target | Threshold |\n|---|---|---|---|\n| Maximum export delay | Analysis | under 5 ms | 5 ms |\n| Maximum retained bytes | Test | under 1 KiB | 1 KiB |\n",
        ),
        (
            "spec/FR-099.md",
            unrelated_body,
        ),
    ];
    if reverse_creation_order {
        for (path, contents) in documents.iter().rev() {
            write(root, path, contents);
        }
    } else {
        for (path, contents) in documents {
            write(root, path, contents);
        }
    }
    let spec = Spec::from_repo(load_repo(&root.join("spec")));
    let symbols = extract_tree(root);
    build_assurance_export(AssuranceInput {
        spec: &spec,
        registry: &registry,
        corpus_root: root.join("spec").as_path(),
        symbols: &symbols,
        symbol_graph: &SymbolGraph::default(),
    })
    .expect("assurance export")
}

#[trace("FR-067-AC-1")]
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

#[trace("FR-072-AC-9")]
// the assurance-v1 bytes for the fixed fixture equal the checked-in export.
#[test]
fn assurance_export_matches_the_checked_in_bytes() {
    let actual = Fixture::new().export().unwrap().to_json_bytes().unwrap();
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/assurance/export.json");
    if std::env::var_os("QUIRE_UPDATE_ASSURANCE_GOLDEN").is_some() {
        fs::write(&path, &actual).unwrap();
    }
    let expected = fs::read(&path).expect("read checked-in assurance export");
    assert_eq!(
        String::from_utf8_lossy(&actual),
        String::from_utf8_lossy(&expected),
        "regenerate with QUIRE_UPDATE_ASSURANCE_GOLDEN=1 if the change is intended"
    );
}

#[trace("FR-067-AC-2")]
#[test]
fn tc1085_construction_refuses_incomplete_premises_atomically() {
    let fixture = Fixture::new();
    let build = |registry: &Registry| {
        build_assurance_export(AssuranceInput {
            spec: &fixture.spec,
            registry,
            corpus_root: fixture._tmp.path().join("spec").as_path(),
            symbols: &fixture.symbols,
            symbol_graph: &fixture.graph,
        })
    };
    let unversioned =
        Registry::from_inline_parts(b"name: unversioned\nartifact_types: []\n", &BTreeMap::new())
            .expect("unversioned registry");
    assert!(matches!(
        build(&unversioned),
        Err(AssuranceError::MissingModuleVersion { .. })
    ));

    let unnamed =
        Registry::from_inline_parts(b"version: 1.0.0\nartifact_types: []\n", &BTreeMap::new())
            .expect("unnamed tolerant registry");
    assert!(matches!(
        build(&unnamed),
        Err(AssuranceError::MissingModuleName { .. })
    ));

    let missing = Registry::from_inline_parts(
        b"name: broken\nversion: 1.0.0\nartifact_types:\n- name: FR\n  frontmatter_schema_ref: missing.json\n",
        &BTreeMap::new(),
    )
    .expect("tolerant registry");
    assert!(matches!(
        build(&missing),
        Err(AssuranceError::ArchetypeLoadFailure { .. })
    ));

    let outside = build_assurance_export(AssuranceInput {
        spec: &fixture.spec,
        registry: &fixture.registry,
        corpus_root: fixture._tmp.path().join("different-root").as_path(),
        symbols: &fixture.symbols,
        symbol_graph: &fixture.graph,
    });
    assert!(matches!(
        outside,
        Err(AssuranceError::PathOutsideRoot { .. })
    ));
}

#[trace("FR-067-AC-8")]
#[test]
fn tc2063_export_refuses_an_unavailable_language() {
    let mut fixture = Fixture::new();
    fixture.export().expect("capable build exports");
    fixture.symbols.unavailable_languages = vec![quire_rs::traceability::SourceLanguage::Rust];
    let err = fixture.export().expect_err("must refuse");
    assert!(matches!(err, AssuranceError::ExtractionUnavailable(_)));
    assert!(err.to_string().contains("rust"));
}

#[trace("FR-067-AC-3")]
#[trace("FR-067-AC-5")]
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
}

#[trace("FR-067-AC-4", "StR-007-VC-2")]
#[test]
fn tc1087_reader_rejects_unsupported_premises_before_returning_records() {
    let export: AssuranceExport = serde_json::from_value(golden()).expect("typed golden");
    let accepted = AcceptedAssurancePremises::from_export(&export);
    assert_eq!(
        read_assurance_export(include_bytes!("../fixtures/assurance/v1.json"), &accepted).unwrap(),
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
        read_assurance_export(include_bytes!("../fixtures/assurance/v1.json"), &rejected),
        Err(AssuranceError::UnacceptedModuleVersion { .. })
    ));
    rejected = accepted.clone();
    rejected.modules.clear();
    assert!(matches!(
        read_assurance_export(include_bytes!("../fixtures/assurance/v1.json"), &rejected),
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
    let duplicate = duplicate_schemas["modules"][0]["schemas"][0].clone();
    duplicate_schemas["modules"][0]["schemas"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    assert!(matches!(
        read_assurance_export(&serde_json::to_vec(&duplicate_schemas).unwrap(), &accepted),
        Err(AssuranceError::DuplicateSchemaPremise { .. })
    ));
}

#[trace("FR-067-AC-9")]
#[test]
fn tc2248_assurance_reader_refuses_duplicate_json_object_keys() {
    let export: AssuranceExport =
        serde_json::from_slice(include_bytes!("../fixtures/assurance/v1.json"))
            .expect("typed golden export");
    let accepted = AcceptedAssurancePremises::from_export(&export);
    let golden = include_bytes!("../fixtures/assurance/v1.json");
    let mut duplicate = br#"{"format":"incorrect", "#.to_vec();
    duplicate.extend_from_slice(&golden[1..]);

    let error = read_assurance_export(&duplicate, &accepted)
        .expect_err("duplicate object keys must be refused");
    assert!(
        matches!(
            error,
            AssuranceError::InvalidJson { ref reason }
                if reason.contains("duplicate JSON object key \"format\"")
        ),
        "unexpected duplicate-key refusal: {error}"
    );

    let object_end = golden
        .iter()
        .rposition(|byte| *byte == b'}')
        .expect("golden object terminator");
    let mut nested_duplicate = golden[..object_end].to_vec();
    nested_duplicate.extend_from_slice(br#","unknown_nested":{"member":1,"member":2}}"#);

    let error = read_assurance_export(&nested_duplicate, &accepted)
        .expect_err("duplicate keys in nested objects must be refused");
    assert!(
        matches!(
            error,
            AssuranceError::InvalidJson { ref reason }
                if reason.contains("duplicate JSON object key \"member\"")
        ),
        "unexpected nested duplicate-key refusal: {error}"
    );

    let mut array_nested_duplicate = golden[..object_end].to_vec();
    array_nested_duplicate.extend_from_slice(br#","unknown_array":[{"member":1,"member":2}]}"#);

    let error = read_assurance_export(&array_nested_duplicate, &accepted)
        .expect_err("duplicate keys in objects inside arrays must be refused");
    assert!(
        matches!(
            error,
            AssuranceError::InvalidJson { ref reason }
                if reason.contains("duplicate JSON object key \"member\"")
        ),
        "unexpected duplicate-key refusal in array: {error}"
    );
}

#[trace("FR-067-AC-10")]
#[test]
fn tc2249_assurance_reader_returns_error_for_deep_nesting_without_panicking() {
    let export: AssuranceExport =
        serde_json::from_slice(include_bytes!("../fixtures/assurance/v1.json"))
            .expect("typed golden export");
    let accepted = AcceptedAssurancePremises::from_export(&export);
    let golden = include_bytes!("../fixtures/assurance/v1.json");
    let object_end = golden
        .iter()
        .rposition(|byte| *byte == b'}')
        .expect("golden object terminator");
    let mut deeply_nested = golden[..object_end].to_vec();
    deeply_nested.extend_from_slice(b",\"unknown_nested\":");
    let depth = 128;
    deeply_nested.extend(std::iter::repeat(b'[').take(depth));
    deeply_nested.push(b'0');
    deeply_nested.extend(std::iter::repeat(b']').take(depth));
    deeply_nested.push(b'}');

    let result = catch_unwind(AssertUnwindSafe(|| {
        read_assurance_export(&deeply_nested, &accepted)
    }))
    .expect("deep nesting must return an error instead of panicking");
    assert!(
        matches!(
            result,
            Err(AssuranceError::InvalidJson { ref reason })
                if reason.contains("recursion limit exceeded")
        ),
        "deep nesting must report the JSON recursion limit, got {result:?}"
    );
}

#[trace("FR-068-AC-1", "StR-007-VC-1")]
#[trace("FR-068-AC-2")]
#[trace("FR-068-AC-3")]
#[test]
fn tc1091_tc1093_records_preserve_authoritative_identifiers_and_locators() {
    let fixture = Fixture::new();
    let export = fixture.export().expect("export");
    assert_eq!(export.artifacts.len(), 3);
    assert!(export
        .artifacts
        .iter()
        .all(|artifact| artifact.locator.line == 1));
    assert_eq!(export.obligations.len(), 1);
    assert_eq!(export.obligations[0].id, "FR-001-AC-1");
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

#[trace("FR-068-AC-2")]
#[trace("FR-068-AC-8")]
#[test]
fn obligations_preserve_every_field_across_rendered_and_configured_identities() {
    let unrelated = |body: &str| {
        format!("---\nid: FR-099\ntype: FR\ntitle: Unrelated\n---\n\n## Description\n\n{body}\n")
    };
    let export = obligation_export(false, &unrelated("Initial unrelated text."));
    let expected = vec![
        AssuranceObligation {
            source: "acceptance-criterion".to_string(),
            id: "FR-001-AC-1".to_string(),
            document: "FR-001.md".to_string(),
            statement: "Repeat this exact acceptance statement.".to_string(),
            method: Some("Test".to_string()),
            parameters: BTreeMap::from([("threshold".to_string(), "5 ms".to_string())]),
            criticality: Some("high".to_string()),
            target_ids: vec!["TC-1901".to_string()],
            locator: AssuranceLocator {
                path: "FR-001.md".to_string(),
                line: 11,
            },
        },
        AssuranceObligation {
            source: "acceptance-criterion".to_string(),
            id: "FR-001-AC-2".to_string(),
            document: "FR-001.md".to_string(),
            statement: "Repeat this exact acceptance statement.".to_string(),
            method: Some("Analysis".to_string()),
            parameters: BTreeMap::from([("threshold".to_string(), "10 ms".to_string())]),
            criticality: Some("elevated".to_string()),
            target_ids: vec!["TC-1902".to_string()],
            locator: AssuranceLocator {
                path: "FR-001.md".to_string(),
                line: 11,
            },
        },
        AssuranceObligation {
            source: "acceptance-criterion".to_string(),
            id: "FR-002-AC-1".to_string(),
            document: "FR-002.md".to_string(),
            statement: "Repeat this exact acceptance statement.".to_string(),
            method: Some("Demonstration".to_string()),
            parameters: BTreeMap::from([("threshold".to_string(), "1 s".to_string())]),
            criticality: Some("severe".to_string()),
            target_ids: vec!["TC-1903".to_string()],
            locator: AssuranceLocator {
                path: "FR-002.md".to_string(),
                line: 11,
            },
        },
        AssuranceObligation {
            source: "nfr-metric".to_string(),
            id: "NFR-003-M-1".to_string(),
            document: "NFR-003.md".to_string(),
            statement: "Maximum export delay".to_string(),
            method: Some("Analysis".to_string()),
            parameters: BTreeMap::from([
                ("target".to_string(), "under 5 ms".to_string()),
                ("threshold".to_string(), "5 ms".to_string()),
            ]),
            criticality: None,
            target_ids: Vec::new(),
            locator: AssuranceLocator {
                path: "NFR-003.md".to_string(),
                line: 10,
            },
        },
        AssuranceObligation {
            source: "nfr-metric".to_string(),
            id: "NFR-003-M-2".to_string(),
            document: "NFR-003.md".to_string(),
            statement: "Maximum retained bytes".to_string(),
            method: Some("Test".to_string()),
            parameters: BTreeMap::from([
                ("target".to_string(), "under 1 KiB".to_string()),
                ("threshold".to_string(), "1 KiB".to_string()),
            ]),
            criticality: None,
            target_ids: Vec::new(),
            locator: AssuranceLocator {
                path: "NFR-003.md".to_string(),
                line: 11,
            },
        },
    ];
    assert_eq!(export.obligations, expected);

    let bytes = export.to_json_bytes().expect("serialize export");
    let accepted = AcceptedAssurancePremises::from_export(&export);
    assert_eq!(
        read_assurance_export(&bytes, &accepted).expect("read export"),
        export,
        "FR-068-AC-8: reading preserves the complete projected record set"
    );
    assert_eq!(
        bytes,
        obligation_export(true, &unrelated("Initial unrelated text."))
            .to_json_bytes()
            .expect("serialize reversed input order"),
        "identical inputs serialize independently of document creation order"
    );
    assert_eq!(
        export,
        obligation_export(false, &unrelated("Edited unrelated prose.")),
        "editing an unrelated document leaves every exported record unchanged"
    );
}

#[trace("FR-068-AC-4")]
#[trace("FR-068-AC-5")]
#[trace("FR-068-AC-7")]
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
    assert!(
        export.relations.iter().any(|relation| matches!(
            relation,
            AssuranceRelation::Verifies {
                freshness: AssuranceFreshness::Unknown,
                form,
                ..
            } if form == "rust-trace"
        )),
        "FR-068-AC-5: a test binding names the marker form that bound it"
    );
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

#[trace("FR-068-AC-6", "StR-007-VC-3")]
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

#[trace("FR-068-AC-8")]
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

#[trace("FR-068-AC-6")]
#[test]
fn tc_assurance_rows_follow_the_scope_of_where_and_when_any() {
    let manifest = br#"
name: scope-fixture
version: 1.0.0
artifact_types:
- name: Ctl
- name: Haz
- name: Req
traceability:
  required_relations:
  - name: ctl-has-req
    from: Ctl
    when_any:
    - { verb: protects, direction: outgoing, other: [Haz] }
    edges: [satisfies]
    to: [Req]
    direction: incoming
    check: ctl-without-req
  - name: nothing-in-scope
    from: Ctl
    when_any:
    - { verb: protects, direction: incoming, other: [Req] }
    edges: [satisfies]
    direction: incoming
    check: nothing-in-scope
"#;
    let registry = Registry::from_inline_parts(manifest, &BTreeMap::new()).expect("registry");
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    let doc = |id: &str, kind: &str, edge: &str| {
        format!("---\nid: {id}\ntype: {kind}\ntitle: T\n{edge}---\n\n## Description\n\nText.\n")
    };
    let protects = "relationships:\n  - target: ix://agent-ix/fixture/H-1\n    type: protects\n";
    write(root, "spec/H-1.md", &doc("H-1", "Haz", ""));
    write(root, "spec/C-1.md", &doc("C-1", "Ctl", protects));
    write(root, "spec/C-2.md", &doc("C-2", "Ctl", ""));
    let spec = Spec::from_repo(load_repo(&root.join("spec")));
    let symbols = extract_tree(root);
    let export = build_assurance_export(AssuranceInput {
        spec: &spec,
        registry: &registry,
        corpus_root: root.join("spec").as_path(),
        symbols: &symbols,
        symbol_graph: &SymbolGraph::default(),
    })
    .expect("export");
    let rows: Vec<_> = export
        .relation_observations
        .iter()
        .map(|o| (o.declaration.as_str(), o.subject.as_deref(), o.availability))
        .collect();
    assert_eq!(
        rows,
        vec![
            ("ctl-has-req", Some("C-1"), RelationAvailability::Missing),
            (
                "nothing-in-scope",
                None,
                RelationAvailability::NotApplicable
            ),
        ]
    );
}
