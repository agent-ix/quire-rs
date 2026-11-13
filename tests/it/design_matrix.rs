//! FR-092 — the design column on the coverage matrix.
//!
//! Every kind, verb, relation and id below is authored in a fixture module
//! (`tests/fixtures/traceability/design-join`) or an ad hoc manifest and is
//! unrelated to requirements or design: a `Glyph` is the obliged document, a
//! `Lantern` the thing that lights it, and `kindles` / `kindled_by` the verb
//! pair. The engine knows none of them.

use std::fs;
use std::path::{Path, PathBuf};

use ix_trace_rs::trace;
use quire_rs::corpus::{
    design_join, design_matrix, CorpusMember, DesignMatrixEntry, RelationReportError,
    RelationReportMember, Spec,
};
use quire_rs::coverage::{compute, CoverageMatrixStatus, CoverageReport};
use quire_rs::symbols::{extract_tree, trace as sym_trace};
use quire_rs::Registry;
use serde_json::{json, Value};
use tempfile::TempDir;

fn module(name: &str) -> Registry {
    Registry::load_module(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/traceability")
            .join(name),
    )
    .expect("fixture module loads")
}

fn write(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    fs::write(path, body).expect("write");
}

/// A document of kind `ty` with extra frontmatter lines, `relationships` and
/// one `section` table minting `ids`.
fn doc(
    id: &str,
    ty: &str,
    extra: &str,
    rels: &[(&str, &str)],
    section: &str,
    ids: &[&str],
) -> String {
    let mut text = format!("---\nid: {id}\ntype: {ty}\ntitle: {id}\n{extra}");
    if !rels.is_empty() {
        text.push_str("relationships:\n");
        for (verb, target) in rels {
            text.push_str(&format!("  - target: \"{target}\"\n    type: \"{verb}\"\n"));
        }
    }
    text.push_str(&format!("---\n\n# {id}\n"));
    if !ids.is_empty() {
        text.push_str(&format!(
            "\n## {section}\n\n| ID | Criteria |\n|----|----------|\n"
        ));
        for minted in ids {
            text.push_str(&format!("| {minted} | The thing holds. |\n"));
        }
    }
    text
}

fn glyph(id: &str, rels: &[(&str, &str)], criteria: &[&str]) -> String {
    doc(id, "Glyph", "", rels, "Acceptance Criteria", criteria)
}

fn lantern(id: &str, rels: &[(&str, &str)]) -> String {
    doc(id, "Lantern", "", rels, "Test Cases", &[])
}

fn repo(docs: &[String]) -> TempDir {
    let dir = TempDir::new().expect("tempdir");
    write(dir.path(), "src/lib.rs", "//! empty\n");
    for text in docs {
        let id = text
            .lines()
            .find_map(|l| l.strip_prefix("id: "))
            .expect("id line");
        write(dir.path(), &format!("spec/{id}.md"), text);
    }
    dir
}

fn coverage_of(dir: &TempDir, registry: &Registry) -> CoverageReport {
    let spec = Spec::from_path(&dir.path().join("spec"));
    let model = registry.traceability().cloned().unwrap_or_default();
    let graph = sym_trace::bind(&extract_tree(&dir.path().join("src")), &model);
    compute(&spec, registry, &graph, dir.path()).expect("coverage")
}

fn matrix_of(dir: &TempDir, registry: &Registry) -> Vec<DesignMatrixEntry> {
    let report = coverage_of(dir, registry);
    design_matrix(
        &Spec::from_path(&dir.path().join("spec")),
        registry,
        &[RelationReportMember {
            root: dir.path(),
            matrix: &report.coverage_matrix,
        }],
    )
    .expect("design matrix")
}

fn json_of(entries: &[DesignMatrixEntry]) -> Value {
    serde_json::to_value(entries).expect("json")
}

fn entry<'v>(matrix: &'v Value, document: &str) -> &'v Value {
    matrix
        .as_array()
        .expect("array")
        .iter()
        .find(|e| e["document"] == document)
        .unwrap_or_else(|| panic!("no entry {document}"))
}

/// A module on disk with the given manifest text.
fn temp_module(manifest: &str) -> (TempDir, Registry) {
    let dir = TempDir::new().expect("tempdir");
    write(dir.path(), "manifest.yaml", manifest);
    let registry = Registry::load_module(dir.path()).expect("module loads");
    (dir, registry)
}

/// A manifest with Glyph criteria, `kindles`/`kindled_by`, and the given relations.
fn manifest(relations: &str, design_relations: &str) -> String {
    format!(
        "name: ad-hoc
version: 0.1.0
description: ad hoc
artifact_types:
- name: Glyph
  defaults:
    id_pattern: GLY-{{next:03d}}
- name: Lantern
  defaults:
    id_pattern: LAN-{{next:03d}}
- name: Prism
  defaults:
    id_pattern: PRI-{{next:03d}}
edge_types:
  kindles: {{ description: x, category: traceability, inverse: kindled_by }}
  echoes: {{ description: y, category: traceability }}
traceability:
  trace_targets:
  - name: glyph-check
    archetype: Glyph
    section: Acceptance Criteria
    id_column: ID
  - name: prism-check
    archetype: Prism
    section: Acceptance Criteria
    id_column: ID
  obligations:
  - name: glyph-check
    target: glyph-check
    statement_column: Criteria
  - name: prism-check
    target: prism-check
    statement_column: Criteria
  required_relations:
{relations}{design_relations}"
    )
}

const LIT: &str = "  - name: glyph-has-lantern
    from: Glyph
    edges: [kindles]
    to: [Lantern]
    direction: incoming
    check: glyph-dark
";

#[trace("FR-092-AC-1")]
#[trace("FR-092-AC-2")]
#[trace("FR-092-AC-7")]
#[test]
fn a_realized_requirement_is_covered_and_an_unrealized_one_is_a_gap() {
    let dir = repo(&[
        glyph("GLY-001", &[], &["chk-a"]),
        glyph("GLY-002", &[], &["chk-b"]),
        lantern("LAN-001", &[("kindles", "GLY-001")]),
    ]);
    let (_m, registry) = temp_module(&manifest(LIT, "  design_relations: [glyph-has-lantern]\n"));
    let matrix = json_of(&matrix_of(&dir, &registry));
    assert_eq!(
        entry(&matrix, "spec/GLY-001.md")["design"],
        json!([{
            "relation": "glyph-has-lantern",
            "check": "glyph-dark",
            "state": "covered",
            "elements": [{
                "id": "LAN-001",
                "kind": "Lantern",
                "path": "spec/LAN-001.md",
                "verb": "kindles",
                "realized_criteria": [],
            }],
            "dangling": [],
            "kind_mismatch": [],
        }])
    );
    assert_eq!(
        entry(&matrix, "spec/GLY-002.md")["design"],
        json!([{
            "relation": "glyph-has-lantern",
            "check": "glyph-dark",
            "state": "uncovered",
            "elements": [],
            "dangling": [],
            "kind_mismatch": [],
        }])
    );
    // Both entries carry their id, and criteria stay as the matrix has them.
    assert_eq!(entry(&matrix, "spec/GLY-001.md")["id"], "GLY-001");
    assert_eq!(
        entry(&matrix, "spec/GLY-001.md")["criteria"][0]["id"],
        "chk-a"
    );
}

#[trace("FR-092-AC-2")]
#[test]
fn an_untyped_element_has_no_kind_and_a_member_without_identity_no_repo() {
    let dir = repo(&[
        glyph("GLY-001", &[], &["chk-a"]),
        "---\nid: LAN-009\ntitle: bare\nrelationships:\n  - target: \"GLY-001\"\n    type: \"kindles\"\n---\n\n# bare\n"
            .to_string(),
    ]);
    let (_m, registry) = temp_module(&manifest(
        "  - name: glyph-has-any
    from: Glyph
    edges: [kindles]
    to: []
    direction: incoming
    check: glyph-dark
",
        "  design_relations: [glyph-has-any]\n",
    ));
    let matrix = json_of(&matrix_of(&dir, &registry));
    let element = &entry(&matrix, "spec/GLY-001.md")["design"][0]["elements"][0];
    assert_eq!(element["id"], "LAN-009");
    assert!(element.get("kind").is_none(), "{element}");
    assert!(element.get("repo").is_none(), "{element}");
    assert!(entry(&matrix, "spec/GLY-001.md").get("repo").is_none());
}

#[trace("FR-092-AC-3")]
#[trace("FR-092-AC-18")]
#[test]
fn an_obliged_requirement_without_criteria_gets_an_entry_in_document_order() {
    let dir = repo(&[
        glyph("GLY-001", &[], &["chk-a"]),
        glyph("GLY-002", &[], &[]),
        glyph("GLY-003", &[], &["chk-c"]),
    ]);
    let (_m, registry) = temp_module(&manifest(LIT, "  design_relations: [glyph-has-lantern]\n"));
    let matrix = json_of(&matrix_of(&dir, &registry));
    let documents: Vec<&str> = matrix
        .as_array()
        .expect("array")
        .iter()
        .map(|e| e["document"].as_str().expect("document"))
        .collect();
    assert_eq!(
        documents,
        ["spec/GLY-001.md", "spec/GLY-002.md", "spec/GLY-003.md"]
    );
    let bare = entry(&matrix, "spec/GLY-002.md");
    assert_eq!(bare["criteria"], json!([]));
    assert_eq!(bare["id"], "GLY-002");
    assert_eq!(bare["design"][0]["state"], "uncovered");
    assert_eq!(bare["design"][0]["elements"], json!([]));

    // No `obligations:` source at all: the array is still there.
    let no_source = "name: ad-hoc
version: 0.1.0
description: ad hoc
artifact_types:
- name: Glyph
  defaults:
    id_pattern: GLY-{next:03d}
- name: Lantern
  defaults:
    id_pattern: LAN-{next:03d}
edge_types:
  kindles: { description: x, category: traceability, inverse: kindled_by }
traceability:
  required_relations:
  - name: glyph-has-lantern
    from: Glyph
    edges: [kindles]
    to: [Lantern]
    direction: incoming
    check: glyph-dark
  design_relations: [glyph-has-lantern]
";
    let (_m2, bare_registry) = temp_module(no_source);
    let matrix = json_of(&matrix_of(&dir, &bare_registry));
    assert_eq!(matrix.as_array().expect("array").len(), 3);
    assert_eq!(entry(&matrix, "spec/GLY-001.md")["criteria"], json!([]));
}

#[trace("FR-092-AC-4")]
#[test]
fn a_requirement_outside_from_where_or_exclude_has_no_design_key() {
    let dir = TempDir::new().expect("tempdir");
    write(dir.path(), "src/lib.rs", "//! empty\n");
    let docs = [
        doc(
            "GLY-001",
            "Glyph",
            "tier: high\n",
            &[],
            "Acceptance Criteria",
            &["chk-a"],
        ),
        doc(
            "GLY-002",
            "Glyph",
            "tier: low\n",
            &[],
            "Acceptance Criteria",
            &["chk-b"],
        ),
        doc(
            "GLY-003",
            "Glyph",
            "tier: high\n",
            &[],
            "Acceptance Criteria",
            &["chk-c"],
        ),
        // Outside `from`: a Prism that states criteria through its own source.
        doc(
            "PRI-001",
            "Prism",
            "tier: high\n",
            &[],
            "Acceptance Criteria",
            &["chk-d"],
        ),
    ];
    for text in &docs {
        let id = text
            .lines()
            .find_map(|l| l.strip_prefix("id: "))
            .expect("id line");
        let sub = if id == "GLY-003" { "skip" } else { "spec" };
        write(dir.path(), &format!("{sub}/{id}.md"), text);
    }
    // GLY-003 lives under `skip/`, which is excluded; the others under `spec/`.
    let relation = "  - name: glyph-has-lantern
    from: Glyph
    where: { field: tier, values: [high] }
    edges: [kindles]
    to: [Lantern]
    direction: incoming
    check: glyph-dark
    exclude: [\"skip/**\"]
";
    let (_m, registry) = temp_module(&manifest(
        relation,
        "  design_relations: [glyph-has-lantern]\n",
    ));
    let spec = Spec::from_path(dir.path());
    let model = registry.traceability().cloned().unwrap_or_default();
    let graph = sym_trace::bind(&extract_tree(&dir.path().join("src")), &model);
    let report = compute(&spec, &registry, &graph, dir.path()).expect("coverage");
    let entries = design_matrix(
        &spec,
        &registry,
        &[RelationReportMember {
            root: dir.path(),
            matrix: &report.coverage_matrix,
        }],
    )
    .expect("matrix");
    let matrix = json_of(&entries);
    assert_eq!(matrix.as_array().expect("array").len(), 4);
    assert!(entry(&matrix, "spec/GLY-001.md").get("design").is_some());
    for out in ["spec/GLY-002.md", "spec/PRI-001.md", "skip/GLY-003.md"] {
        let e = entry(&matrix, out);
        assert!(e.get("design").is_none(), "{out}: {e}");
        assert!(e.get("id").is_none(), "{out}: {e}");
        assert!(e["criteria"][0].get("design_elements").is_none(), "{out}");
    }
}

#[trace("FR-092-AC-5")]
#[trace("FR-092-AC-16")]
#[trace("FR-092-CON-2")]
#[test]
fn without_design_relations_the_output_is_the_coverage_matrix() {
    let dir = repo(&[
        glyph("GLY-001", &[], &["chk-a", "chk-b"]),
        glyph("GLY-002", &[], &["chk-c"]),
        lantern("LAN-001", &[("kindles", "GLY-001")]),
    ]);
    let (_m, plain) = temp_module(&manifest(LIT, ""));
    let report = coverage_of(&dir, &plain);
    let entries = matrix_of(&dir, &plain);
    assert_eq!(
        serde_json::to_string(&entries).expect("json"),
        serde_json::to_string(&report.coverage_matrix).expect("json")
    );

    // Declaring design relations leaves the coverage report byte-identical.
    // The module directory is the same, so its path in diagnostics is too.
    let reload = |design: &str| {
        write(_m.path(), "manifest.yaml", &manifest(LIT, design));
        Registry::load_module(_m.path()).expect("module loads")
    };
    let designed = reload("  design_relations: [glyph-has-lantern]\n");
    assert_eq!(report.to_json(), coverage_of(&dir, &designed).to_json());
    // Even a faulty entry never makes `coverage` fail.
    let faulty = reload("  design_relations: [ghost]\n");
    assert_eq!(report.to_json(), coverage_of(&dir, &faulty).to_json());
}

#[trace("FR-092-AC-6")]
#[trace("FR-092-AC-12")]
#[test]
fn two_relations_give_two_blocks_in_design_relations_order() {
    let dir = repo(&[
        glyph("GLY-001", &[("kindled_by", "LAN-002")], &["chk-a"]),
        lantern("LAN-001", &[("kindles", "GLY-001")]),
        lantern("LAN-002", &[]),
    ]);
    let registry = module("design-join");
    let matrix = json_of(&matrix_of(&dir, &registry));
    let blocks = entry(&matrix, "spec/GLY-001.md")["design"]
        .as_array()
        .expect("design")
        .clone();
    // Declared as [prism-has-lantern, glyph-has-lantern, glyph-lit-by]; the
    // Prism relation obliges no Glyph.
    let named: Vec<(&str, &str, &str)> = blocks
        .iter()
        .map(|b| {
            (
                b["relation"].as_str().expect("relation"),
                b["check"].as_str().expect("check"),
                b["state"].as_str().expect("state"),
            )
        })
        .collect();
    assert_eq!(
        named,
        [
            ("glyph-has-lantern", "glyph-dark", "covered"),
            ("glyph-lit-by", "glyph-unlit", "covered"),
        ]
    );
}

#[trace("FR-092-AC-7")]
#[test]
fn a_dangling_edge_is_listed_and_leaves_the_block_uncovered() {
    let dir = repo(&[glyph(
        "GLY-001",
        &[("kindled_by", "ix://o/b/spec/LAN-404")],
        &["chk-a"],
    )]);
    let (_m, registry) = temp_module(&manifest(LIT, "  design_relations: [glyph-has-lantern]\n"));
    let matrix = json_of(&matrix_of(&dir, &registry));
    let block = &entry(&matrix, "spec/GLY-001.md")["design"][0];
    assert_eq!(block["state"], "uncovered");
    assert_eq!(block["elements"], json!([]));
    assert_eq!(block["dangling"][0]["reference"], "ix://o/b/spec/LAN-404");
}

#[trace("FR-092-AC-17")]
#[test]
fn an_open_relation_with_only_a_dangling_edge_is_covered_with_no_element() {
    let dir = repo(&[glyph(
        "GLY-001",
        &[("echoes", "ix://o/b/spec/LAN-404")],
        &["chk-a"],
    )]);
    let (_m, registry) = temp_module(&manifest(
        "  - name: glyph-echoed
    from: Glyph
    edges: [echoes]
    to: []
    direction: outgoing
    check: glyph-silent
",
        "  design_relations: [glyph-echoed]\n",
    ));
    let matrix = json_of(&matrix_of(&dir, &registry));
    let block = &entry(&matrix, "spec/GLY-001.md")["design"][0];
    assert_eq!(block["state"], "covered");
    assert_eq!(block["elements"], json!([]));
    assert_eq!(block["dangling"][0]["reference"], "ix://o/b/spec/LAN-404");
    assert_eq!(
        entry(&matrix, "spec/GLY-001.md")["criteria"][0]["design_elements"],
        json!([])
    );
}

fn identified(dirs: &[(&TempDir, &str)]) -> Spec {
    Spec::from_members(
        dirs.iter()
            .map(|(dir, identity)| CorpusMember {
                identity: Some((*identity).to_string()),
                load: quire_rs::corpus::walk::load_repo(&dir.path().join("spec")),
            })
            .collect(),
    )
    .expect("corpus")
}

/// The design matrix of a multi-member corpus, each member's matrix computed
/// over its own documents.
fn multi(dirs: &[(&TempDir, &str)], registry: &Registry) -> Vec<DesignMatrixEntry> {
    let reports: Vec<CoverageReport> = dirs
        .iter()
        .map(|(dir, _)| coverage_of(dir, registry))
        .collect();
    let members: Vec<RelationReportMember<'_>> = dirs
        .iter()
        .zip(&reports)
        .map(|((dir, _), report)| RelationReportMember {
            root: dir.path(),
            matrix: &report.coverage_matrix,
        })
        .collect();
    design_matrix(&identified(dirs), registry, &members).expect("design matrix")
}

#[trace("FR-092-AC-8")]
#[trace("FR-092-AC-15")]
#[test]
fn an_element_in_another_member_names_its_repo_and_equal_paths_stay_apart() {
    let a = repo(&[glyph("GLY-001", &[], &["chk-a"])]);
    let b = repo(&[
        glyph("GLY-001", &[], &["chk-b"]),
        lantern("LAN-001", &[("kindles", "ix://o/a/spec/GLY-001")]),
    ]);
    let (_m, registry) = temp_module(&manifest(LIT, "  design_relations: [glyph-has-lantern]\n"));
    // Fed in reverse member order, the entries are still ordered by repo.
    let matrix = json_of(&multi(&[(&b, "o/b"), (&a, "o/a")], &registry));
    let entries = matrix.as_array().expect("array");
    let keys: Vec<(&str, &str)> = entries
        .iter()
        .map(|e| {
            (
                e["repo"].as_str().expect("repo"),
                e["document"].as_str().expect("document"),
            )
        })
        .collect();
    assert_eq!(
        keys,
        [("o/a", "spec/GLY-001.md"), ("o/b", "spec/GLY-001.md"),]
    );
    let covered = &entries[0]["design"][0];
    assert_eq!(covered["state"], "covered");
    assert_eq!(covered["elements"][0]["id"], "LAN-001");
    assert_eq!(covered["elements"][0]["repo"], "o/b");
    assert_eq!(entries[1]["design"][0]["state"], "uncovered");
    assert_eq!(entries[1]["criteria"][0]["id"], "chk-b");
}

#[trace("FR-092-AC-9")]
#[test]
fn each_criterion_lists_the_elements_that_realize_it_once() {
    let dir = repo(&[
        glyph("GLY-001", &[("kindled_by", "LAN-001")], &["chk-a", "chk-b"]),
        lantern("LAN-001", &[("kindles", "GLY-001")]),
        lantern("LAN-002", &[("kindles", "chk-a")]),
    ]);
    let registry = module("design-join");
    let matrix = json_of(&matrix_of(&dir, &registry));
    let criteria = &entry(&matrix, "spec/GLY-001.md")["criteria"];
    // LAN-001 realizes the whole requirement and sits in two blocks; LAN-002
    // names chk-a only.
    assert_eq!(
        criteria[0]["design_elements"],
        json!([{"id": "LAN-001"}, {"id": "LAN-002"}])
    );
    assert_eq!(criteria[1]["design_elements"], json!([{"id": "LAN-001"}]));
}

#[trace("FR-092-AC-10")]
#[test]
fn an_undeclared_design_relation_fails_the_matrix_but_not_coverage() {
    let dir = repo(&[glyph("GLY-001", &[], &["chk-a"])]);
    let (_m, registry) = temp_module(&manifest(LIT, "  design_relations: [ghost]\n"));
    let report = coverage_of(&dir, &registry);
    let err = design_matrix(
        &Spec::from_path(&dir.path().join("spec")),
        &registry,
        &[RelationReportMember {
            root: dir.path(),
            matrix: &report.coverage_matrix,
        }],
    )
    .expect_err("no matrix");
    assert_eq!(
        err,
        RelationReportError::UnknownDesignRelation {
            name: "ghost".into()
        }
    );
    assert!(err.to_string().contains("ghost"));
}

#[trace("FR-092-AC-11")]
#[test]
fn a_retired_criterion_keeps_its_place_under_the_design_block() {
    let dir = repo(&[
        glyph("GLY-001", &[], &["chk-a", "chk-b"]),
        lantern("LAN-001", &[("kindles", "chk-b")]),
    ]);
    let (_m, registry) = temp_module(&manifest(LIT, "  design_relations: [glyph-has-lantern]\n"));
    let mut report = coverage_of(&dir, &registry);
    report.coverage_matrix[0].criteria[1].status = CoverageMatrixStatus::Retired;
    let entries = design_matrix(
        &Spec::from_path(&dir.path().join("spec")),
        &registry,
        &[RelationReportMember {
            root: dir.path(),
            matrix: &report.coverage_matrix,
        }],
    )
    .expect("matrix");
    let matrix = json_of(&entries);
    let found = entry(&matrix, "spec/GLY-001.md");
    assert_eq!(found["criteria"][1]["status"], "retired");
    assert_eq!(
        found["criteria"][1]["design_elements"],
        json!([{"id": "LAN-001"}])
    );
    assert_eq!(found["design"][0]["state"], "covered");
}

#[trace("FR-092-AC-13")]
#[test]
fn the_engine_source_holds_no_string_literal_but_the_serde_skip_predicate() {
    // Every kind, verb and relation name reaches the engine as data; the only
    // string literal in code is the serde attribute predicate.
    let source = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/corpus/design_matrix.rs"),
    )
    .expect("source");
    for line in source.lines().filter(|l| !l.trim_start().starts_with("//")) {
        for (n, literal) in line.split('"').enumerate().filter(|(n, _)| n % 2 == 1) {
            assert!(
                literal == "Option::is_none",
                "string literal {literal:?} (segment {n}) in: {line}"
            );
        }
    }
}

#[trace("FR-092-AC-14")]
#[test]
fn the_matrix_is_byte_identical_across_runs_and_changes_no_document() {
    let a = repo(&[
        glyph(
            "GLY-002",
            &[("kindled_by", "ix://o/b/spec/LAN-001")],
            &["chk-z"],
        ),
        glyph("GLY-001", &[], &["chk-a"]),
    ]);
    let b = repo(&[
        lantern("LAN-001", &[("kindles", "ix://o/a/spec/GLY-001")]),
        lantern("LAN-002", &[("kindles", "ix://o/a/spec/GLY-001")]),
    ]);
    let (_m, registry) = temp_module(&manifest(LIT, "  design_relations: [glyph-has-lantern]\n"));
    let snapshot = |dir: &TempDir| {
        let mut out: Vec<(PathBuf, Vec<u8>)> = fs::read_dir(dir.path().join("spec"))
            .expect("dir")
            .map(|e| {
                let path = e.expect("entry").path();
                let bytes = fs::read(&path).expect("read");
                (path, bytes)
            })
            .collect();
        out.sort();
        out
    };
    let before = (snapshot(&a), snapshot(&b));
    let render =
        |dirs: &[(&TempDir, &str)]| serde_json::to_string(&multi(dirs, &registry)).expect("json");
    // `load_repo` sorts by path, so document order is varied after the load.
    let reversed = |dirs: &[(&TempDir, &str)]| {
        let corpus = Spec::from_members(
            dirs.iter()
                .map(|(dir, identity)| {
                    let mut load = quire_rs::corpus::walk::load_repo(&dir.path().join("spec"));
                    load.documents.reverse();
                    CorpusMember {
                        identity: Some((*identity).to_string()),
                        load,
                    }
                })
                .collect(),
        )
        .expect("corpus");
        let reports: Vec<CoverageReport> = dirs
            .iter()
            .map(|(d, _)| coverage_of(d, &registry))
            .collect();
        let members: Vec<RelationReportMember<'_>> = dirs
            .iter()
            .zip(&reports)
            .map(|((dir, _), report)| RelationReportMember {
                root: dir.path(),
                matrix: &report.coverage_matrix,
            })
            .collect();
        serde_json::to_string(&design_matrix(&corpus, &registry, &members).expect("matrix"))
            .expect("json")
    };
    let first = render(&[(&a, "o/a"), (&b, "o/b")]);
    assert_eq!(first, render(&[(&a, "o/a"), (&b, "o/b")]));
    assert_eq!(first, render(&[(&b, "o/b"), (&a, "o/a")]));
    assert_eq!(first, reversed(&[(&a, "o/a"), (&b, "o/b")]));
    assert_eq!(first, reversed(&[(&b, "o/b"), (&a, "o/a")]));
    assert_eq!((snapshot(&a), snapshot(&b)), before);
}

#[test]
fn the_entry_keys_serialize_in_the_fixed_order() {
    let dir = repo(&[
        doc(
            "GLY-001",
            "Glyph",
            "criticality: high\n",
            &[],
            "Acceptance Criteria",
            &["chk-a"],
        ),
        lantern("LAN-001", &[]),
    ]);
    let (_m, registry) = temp_module(&manifest(LIT, "  design_relations: [glyph-has-lantern]\n"));
    let entries = matrix_of(&dir, &registry);
    let text = serde_json::to_string(&entries[0]).expect("json");
    let at = |key: &str| {
        text.find(&format!("\"{key}\":"))
            .unwrap_or_else(|| panic!("{key}"))
    };
    assert!(
        at("document") < at("id")
            && at("id") < at("criticality")
            && at("criticality") < at("criteria"),
        "{text}"
    );
    assert!(
        text.rfind("\"design\":").expect("design") > at("criteria"),
        "{text}"
    );
    assert!(text.contains("\"criticality\":\"high\""), "{text}");
}

#[trace("FR-092-AC-7")]
#[test]
fn each_block_equals_the_join_row_and_a_wrong_kind_target_is_a_mismatch() {
    let dir = repo(&[
        glyph("GLY-001", &[("kindled_by", "BUL-001")], &["chk-a"]),
        glyph("GLY-002", &[("kindled_by", "LAN-001")], &["chk-b"]),
        lantern("LAN-001", &[("kindles", "chk-a")]),
        doc("BUL-001", "Bulb", "", &[], "Notes", &[]),
    ]);
    let registry = module("design-join");
    let spec = Spec::from_path(&dir.path().join("spec"));
    let report = coverage_of(&dir, &registry);
    let members = [RelationReportMember {
        root: dir.path(),
        matrix: &report.coverage_matrix,
    }];
    let join =
        serde_json::to_value(design_join(&spec, &registry, &members).expect("join")).expect("json");
    let matrix = json_of(&design_matrix(&spec, &registry, &members).expect("matrix"));

    let mut mismatches = 0;
    let mut blocks = 0;
    for section in join["relations"].as_array().expect("sections") {
        for row in section["rows"].as_array().expect("rows") {
            let path = row["document"]["path"].as_str().expect("path");
            let block = entry(&matrix, path)["design"]
                .as_array()
                .expect("design")
                .iter()
                .find(|b| b["relation"] == section["name"])
                .unwrap_or_else(|| panic!("no block for {path}"));
            blocks += 1;
            for key in ["check", "state", "dangling", "kind_mismatch"] {
                assert_eq!(block[key], row[key], "{key} of {path}");
            }
            let projected: Vec<Value> = row["linked"]
                .as_array()
                .expect("linked")
                .iter()
                .filter(|l| l["resolution"] == "resolved")
                .map(|l| {
                    let mut out = serde_json::Map::new();
                    for (to, from) in [
                        ("id", "id"),
                        ("kind", "archetype"),
                        ("path", "path"),
                        ("repo", "repo"),
                        ("verb", "verb"),
                        ("realized_criteria", "realized_criteria"),
                    ] {
                        if let Some(v) = l.get(from) {
                            out.insert(to.into(), v.clone());
                        }
                    }
                    Value::Object(out)
                })
                .collect();
            assert_eq!(
                block["elements"],
                Value::Array(projected),
                "elements of {path}"
            );
            mismatches += block["kind_mismatch"].as_array().expect("list").len();
        }
    }
    assert!(blocks >= 2, "{blocks}");
    assert_eq!(mismatches, 1, "the Bulb target is a kind mismatch");
}
