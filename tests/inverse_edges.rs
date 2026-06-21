//! FR-041 — Authorable Inverse Edge Verbs (TC-652..656).
//!
//! A declared `inverse:` label is an authorable verb (a derived view of its
//! forward edge): Tier-1 recognizes it, Tier-2 normalizes it to the forward
//! orientation for the target check. Warn-tier + deterministic.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use quire_rs::{
    validate_bundle_at, validate_document_in_registry, BundlePosture, Diagnostic, Registry,
    ValidationReason,
};

fn schemas() -> BTreeMap<String, String> {
    let mut s = BTreeMap::new();
    s.insert(
        "schemas/thing.schema.json".to_string(),
        r#"{"type":"object","required":["id","title"],"properties":{"id":{"type":"string"},"title":{"type":"string"}}}"#
            .to_string(),
    );
    s
}

fn registry(manifest: &[u8]) -> Registry {
    let r = Registry::from_inline_parts(manifest, &schemas()).expect("inline registry");
    assert!(r.failures().is_empty(), "load failures: {:?}", r.failures());
    r
}

// TC-652 (FR-041-AC-1): the merged registry exposes an inverse index mapping
// each declared `inverse:` label to its forward verb; a registry with no
// declared inverses exposes an empty index.
#[test]
fn tc652_inverse_index_exposed() {
    let reg = registry(
        br#"
name: inv
artifact_types:
- name: thing
  frontmatter_schema_ref: schemas/thing.schema.json
edge_types:
  publishes: { description: x, category: dataflow, inverse: consumed_by }
  supersedes: { description: x, category: traceability, inverse: superseded_by }
"#,
    );
    let idx = reg.inverse_index();
    assert_eq!(
        idx.get("consumed_by").map(String::as_str),
        Some("publishes")
    );
    assert_eq!(
        idx.get("superseded_by").map(String::as_str),
        Some("supersedes")
    );
    assert_eq!(idx.len(), 2, "{idx:?}");

    // No declared inverses → empty index.
    let bare = registry(
        br#"
name: inv0
artifact_types:
- name: thing
  frontmatter_schema_ref: schemas/thing.schema.json
edge_types:
  references: { description: x, category: traceability }
"#,
    );
    assert!(bare.inverse_index().is_empty());
}

// TC-653 (FR-041-AC-2): a frontmatter edge whose `type` is a declared inverse
// label is type-allowed during validate (no DisallowedEdgeType), even though
// it is absent from resolve_allowed_links; a verb that is neither a resolved
// key nor a known inverse still yields exactly one DisallowedEdgeType.
#[test]
fn tc653_tier1_recognizes_inverse_label() {
    let reg = registry(
        br#"
name: inv1
artifact_types:
- name: thing
  frontmatter_schema_ref: schemas/thing.schema.json
  allowed_links: [implements]
edge_types:
  implements: { description: x, category: dependency }
  publishes: { description: x, category: dataflow, inverse: consumed_by }
"#,
    );
    let thing = reg.archetype("thing").expect("thing");
    // `implements` ∈ allowed_links; `consumed_by` is the inverse of
    // `publishes` (authorable, recognized); `teleports` is unknown.
    let doc = "---\nid: T-1\ntitle: A\n\
               relationships:\n\
               - target: ix://o/r/X-1\n  type: implements\n\
               - target: ix://o/r/Y-1\n  type: consumed_by\n\
               - target: ix://o/r/Z-1\n  type: teleports\n---\n\
               # A\n";
    let r = validate_document_in_registry(&reg, thing, doc);
    let edge: Vec<_> = r
        .warnings
        .iter()
        .filter(|w| w.reason == ValidationReason::DisallowedEdgeType)
        .collect();
    assert_eq!(
        edge.len(),
        1,
        "only the unknown verb warns: {:?}",
        r.warnings
    );
    assert!(edge[0].message.contains("teleports"), "{}", edge[0].message);
    assert!(r.is_valid, "edges are advisory");
}

// TC-654 (FR-041-AC-3): a label that is both a forward edge_types key and an
// inverse of another verb resolves to the forward registration (not the
// inverse); two forward verbs declaring the same inverse label are first-wins
// and emit a non-fatal DuplicateInverseEdge.
#[test]
fn tc654_precedence_and_collision() {
    // Precedence: `references` is declared as `governs`'s inverse but is also
    // its own forward edge_type → it must NOT appear in the inverse index.
    let prec = registry(
        br#"
name: inv2
artifact_types:
- name: thing
  frontmatter_schema_ref: schemas/thing.schema.json
edge_types:
  references: { description: x, category: traceability }
  governs: { description: x, category: governance, inverse: references }
"#,
    );
    assert!(
        !prec.inverse_index().contains_key("references"),
        "forward registration governs: {:?}",
        prec.inverse_index()
    );

    // Collision: both `emits` and `publishes` declare inverse `consumed_by`.
    // First-wins lexicographically (`emits` < `publishes`) + diagnostic.
    let coll = registry(
        br#"
name: inv3
artifact_types:
- name: thing
  frontmatter_schema_ref: schemas/thing.schema.json
edge_types:
  emits: { description: x, category: dataflow, inverse: consumed_by }
  publishes: { description: x, category: dataflow, inverse: consumed_by }
"#,
    );
    assert_eq!(
        coll.inverse_index().get("consumed_by").map(String::as_str),
        Some("emits"),
        "first-wins"
    );
    let dup = coll.diagnostics().iter().any(
        |d| matches!(d, Diagnostic::DuplicateInverseEdge { name, .. } if name == "consumed_by"),
    );
    assert!(
        dup,
        "DuplicateInverseEdge emitted: {:?}",
        coll.diagnostics()
    );
}

// ── Tier-2 corpus fixtures (TC-655) ──

fn tmpdir(tag: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!(
        "quire_inv_{tag}_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&p).unwrap();
    p
}

fn write(dir: &Path, rel: &str, body: &str) {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, body).unwrap();
}

/// A `thing` doc with an `object:` and an authored `consumed_by` edge to `A-001`.
fn consumer_doc(id: &str, object: &str) -> String {
    format!(
        "---\nid: {id}\ntitle: {id}\ntype: thing\nobject: {object}\n\
         relationships:\n- target: ix://x/y/A-001\n  type: consumed_by\n---\n# {id}\n"
    )
}

fn tier2_registry() -> Registry {
    // `producer.publishes -> [consumer]`; inverse is `consumed_by`.
    registry(
        br#"
name: inv-corpus
artifact_types:
- name: thing
  frontmatter_schema_ref: schemas/thing.schema.json
object_types:
- name: producer
  allowed_links:
    publishes: [consumer]
- name: consumer
edge_types:
  publishes: { description: x, category: dataflow, inverse: consumed_by }
"#,
    )
}

// TC-655 (FR-041-AC-4): a corpus inverse edge `(B, consumed_by, A)` is
// normalized to `(A, publishes, B)` before target_satisfies; a forward-valid
// target passes, a forward-direction mismatch yields one DisallowedEdgeTarget
// reported with the authored inverse source/target/edge_type.
#[test]
fn tc655_tier2_inverse_normalization() {
    // PASS case: A=producer, B=consumer → A publishes B is valid.
    let root = tmpdir("ok");
    write(
        &root,
        "A-001.md",
        "---\nid: A-001\ntitle: A\ntype: thing\nobject: producer\n---\n# A\n",
    );
    write(&root, "B-001.md", &consumer_doc("B-001", "consumer"));
    let report = validate_bundle_at(&root, &tier2_registry(), BundlePosture::Okf);
    assert!(
        !report
            .warnings
            .iter()
            .any(|w| w.reason == "disallowed-edge-target"),
        "forward-valid inverse edge passes: {:?}",
        report.warnings
    );

    // FAIL case: B=producer → A publishes B requires `consumer`, producer
    // does not satisfy → one disallowed-edge-target naming the authored edge.
    let root = tmpdir("bad");
    write(
        &root,
        "A-001.md",
        "---\nid: A-001\ntitle: A\ntype: thing\nobject: producer\n---\n# A\n",
    );
    write(&root, "B-001.md", &consumer_doc("B-001", "producer"));
    let report = validate_bundle_at(&root, &tier2_registry(), BundlePosture::Okf);
    let targets: Vec<_> = report
        .warnings
        .iter()
        .filter(|w| w.reason == "disallowed-edge-target")
        .collect();
    assert_eq!(
        targets.len(),
        1,
        "one target violation: {:?}",
        report.warnings
    );
    let m = &targets[0].message;
    assert!(
        m.contains("consumed_by") && m.contains("B-001") && m.contains("A-001"),
        "reported with authored inverse edge: {m}"
    );
    // Warn-tier: no hard errors from the edge check.
    assert!(
        !report
            .errors
            .iter()
            .any(|e| e.reason == "disallowed-edge-target"),
        "edge findings are warnings, never errors"
    );
}

// TC-656 (FR-041-AC-5): inverse recognition/normalization are warnings only
// and deterministic — the inverse index is identical across repeated loads.
#[test]
fn tc656_warn_only_and_deterministic() {
    let manifest: &[u8] = br#"
name: inv-det
artifact_types:
- name: thing
  frontmatter_schema_ref: schemas/thing.schema.json
edge_types:
  publishes: { description: x, category: dataflow, inverse: consumed_by }
  emits: { description: x, category: dataflow, inverse: emitted_by }
  supersedes: { description: x, category: traceability, inverse: superseded_by }
"#;
    let a = registry(manifest);
    let b = registry(manifest);
    assert_eq!(
        a.inverse_index(),
        b.inverse_index(),
        "inverse index is deterministic across loads"
    );

    // A document full of inverse edges never fails validation (warn-tier).
    let reg = registry(
        br#"
name: inv-warn
artifact_types:
- name: thing
  frontmatter_schema_ref: schemas/thing.schema.json
  allowed_links: [references]
edge_types:
  references: { description: x, category: traceability }
  publishes: { description: x, category: dataflow, inverse: consumed_by }
"#,
    );
    let thing = reg.archetype("thing").expect("thing");
    let doc = "---\nid: T-2\ntitle: A\n\
               relationships:\n- target: ix://o/r/X-1\n  type: consumed_by\n---\n# A\n";
    let r = validate_document_in_registry(&reg, thing, doc);
    assert!(r.is_valid, "inverse edges are advisory: {:?}", r.errors);
}
