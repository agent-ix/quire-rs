//! `harvest_edges` (FR-015, Task 017).
//!
//! Reads three sources in deterministic order and folds them into one
//! `EdgeHarvest`:
//!
//! 1. **Structured `relationships:` block** in frontmatter (typed entries).
//! 2. **Sugar fields** at the top level of frontmatter
//!    (`depends_on`, `parent`, `parent_process`, `template_for`,
//!    `archetype_for`, `replaced_by`), declared in fixed order so the
//!    output is stable across runs.
//! 3. **`emit_edges` from the body-extraction DSL** (`ExtractionResult.edges`).
//!
//! Each bare target is normalized via the supplied
//! [`RelationshipResolver`]. Unresolvable targets keep the bare value
//! and emit `Diagnostic::UnresolvableEdgeTarget`.
//!
//! Output is deduped by `(source, type, target)`; the first occurrence
//! wins (per FR-015) and subsequent drops emit
//! `Diagnostic::DuplicateEdgeDropped`.

use std::collections::HashSet;

use indexmap::IndexMap;
use serde_json::Value;

use crate::ast::QuireDocument;
use crate::diagnostic::Diagnostic;
use crate::edges::resolver::RelationshipResolver;
use crate::extract::{ExtractionResult, HarvestedEdge};

/// Sugar fields, in the canonical order the harvester walks them.
pub const SUGAR_FIELDS: &[(&str, &str)] = &[
    // (frontmatter key, edge type)
    ("depends_on", "depends_on"),
    ("parent", "parent"),
    ("parent_process", "parent"),
    ("template_for", "template_for"),
    ("archetype_for", "archetype_for"),
    ("replaced_by", "replaced_by"),
];

/// Result of [`harvest_edges`].
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeHarvest {
    /// Edges in canonical order: structured → sugar → DSL.
    pub edges: Vec<HarvestedEdge>,
    pub diagnostics: Vec<Diagnostic>,
}

/// Harvest every edge from `doc` (plus the optional `extraction`
/// produced by `extract::extract`). Targets resolved via `resolver`.
///
/// `source_ref` is included in the dedup key — typically the
/// document's own ix:// URI — and surfaced in diagnostics.
pub fn harvest_edges<R: RelationshipResolver>(
    doc: &QuireDocument,
    source_ref: &str,
    extraction: Option<&ExtractionResult>,
    resolver: &R,
) -> EdgeHarvest {
    #[cfg(feature = "tracing")]
    let _span = tracing::debug_span!("quire_rs::harvest_edges", source = %source_ref).entered();
    let mut edges: Vec<HarvestedEdge> = Vec::new();
    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    // 1. Structured relationships: block.
    if let Some(map) = &doc.frontmatter {
        if let Some(Value::Array(arr)) = map.get("relationships") {
            for entry in arr {
                if let Some(obj) = entry.as_object() {
                    let r#type = obj
                        .get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let target_raw = obj
                        .get("target")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    if r#type.is_empty() || target_raw.is_empty() {
                        continue;
                    }
                    let target =
                        resolve_or_warn(resolver, &target_raw, source_ref, &mut diagnostics);
                    let metadata = obj
                        .iter()
                        .filter(|(k, _)| k.as_str() != "type" && k.as_str() != "target")
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect::<IndexMap<String, Value>>();
                    edges.push(HarvestedEdge {
                        r#type,
                        target,
                        metadata,
                    });
                }
            }
        }

        // 2. Sugar fields, in canonical order.
        for (key, edge_type) in SUGAR_FIELDS {
            if let Some(value) = map.get(*key) {
                match value {
                    Value::Array(arr) => {
                        for item in arr {
                            if let Some(t) = item.as_str() {
                                let target =
                                    resolve_or_warn(resolver, t, source_ref, &mut diagnostics);
                                edges.push(HarvestedEdge {
                                    r#type: (*edge_type).to_string(),
                                    target,
                                    metadata: IndexMap::new(),
                                });
                            }
                        }
                    }
                    Value::String(s) => {
                        let target = resolve_or_warn(resolver, s, source_ref, &mut diagnostics);
                        edges.push(HarvestedEdge {
                            r#type: (*edge_type).to_string(),
                            target,
                            metadata: IndexMap::new(),
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    // 3. DSL emit_edges union.
    if let Some(ex) = extraction {
        for e in &ex.edges {
            let target = resolve_or_warn(resolver, &e.target, source_ref, &mut diagnostics);
            edges.push(HarvestedEdge {
                r#type: e.r#type.clone(),
                target,
                metadata: e.metadata.clone(),
            });
        }
        // Bubble up the diagnostics the extraction collected.
        diagnostics.extend(ex.diagnostics.iter().cloned());
    }

    // Dedup by (source_ref, type, target). Stable order: keep first.
    let mut seen: HashSet<(String, String)> = HashSet::new();
    let mut deduped: Vec<HarvestedEdge> = Vec::with_capacity(edges.len());
    for e in edges {
        let key = (e.r#type.clone(), e.target.clone());
        if seen.contains(&key) {
            diagnostics.push(Diagnostic::DuplicateEdgeDropped {
                source: source_ref.to_string(),
                edge_type: e.r#type.clone(),
                target: e.target.clone(),
            });
            continue;
        }
        seen.insert(key);
        deduped.push(e);
    }

    EdgeHarvest {
        edges: deduped,
        diagnostics,
    }
}

fn resolve_or_warn<R: RelationshipResolver>(
    resolver: &R,
    bare: &str,
    source: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> String {
    // Sniff the caller's own ix:// URI for org/repo hints — when the
    // doc's source_ref is itself ix://org/repo/<id>, we pass those
    // to the resolver per FR-018-AC-2.
    let (org_hint, repo_hint) = parse_ix_hints(source);
    match resolver.resolve(org_hint.as_deref(), repo_hint.as_deref(), bare) {
        Ok(uri) => uri,
        Err(_) => {
            diagnostics.push(Diagnostic::UnresolvableEdgeTarget {
                source: source.to_string(),
                target: bare.to_string(),
            });
            bare.to_string()
        }
    }
}

/// Extract `(org, repo)` from a `source_ref` shaped like
/// `ix://<org>/<repo>/<rest>`. Returns `(None, None)` when the hint
/// is unavailable.
fn parse_ix_hints(source: &str) -> (Option<String>, Option<String>) {
    let rest = match source.strip_prefix("ix://") {
        Some(r) => r,
        None => return (None, None),
    };
    let mut parts = rest.splitn(3, '/');
    let org = parts.next().filter(|s| !s.is_empty()).map(str::to_string);
    let repo = parts.next().filter(|s| !s.is_empty()).map(str::to_string);
    (org, repo)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edges::resolver::{IdentityResolver, MockResolver};
    use crate::parser::parse_document;

    fn doc() -> QuireDocument {
        parse_document(
            "---\n\
             id: FR-001\n\
             depends_on:\n- FR-002\n- FR-003\n\
             parent: SPEC-A\n\
             relationships:\n\
             - type: implements\n  target: StR-001\n  cardinality: '1:1'\n\
             ---\nbody\n",
        )
    }

    #[test]
    fn structured_block_then_sugar_then_dsl_in_canonical_order() {
        let d = doc();
        let h = harvest_edges(&d, "ix://test/FR-001", None, &IdentityResolver);
        // Expected order: structured (implements StR-001), then sugar
        // (depends_on FR-002, FR-003, parent SPEC-A).
        let types: Vec<&str> = h.edges.iter().map(|e| e.r#type.as_str()).collect();
        let targets: Vec<&str> = h.edges.iter().map(|e| e.target.as_str()).collect();
        assert_eq!(
            types,
            vec!["implements", "depends_on", "depends_on", "parent"]
        );
        assert_eq!(targets, vec!["StR-001", "FR-002", "FR-003", "SPEC-A"]);
    }

    #[test]
    fn resolver_normalizes_bare_to_ix_uri() {
        let d = doc();
        let resolver = MockResolver::new()
            .with("StR-001", "ix://agent-ix/iso/StR-001")
            .with("FR-002", "ix://agent-ix/iso/FR-002")
            .with("FR-003", "ix://agent-ix/iso/FR-003")
            .with("SPEC-A", "ix://agent-ix/iso/SPEC-A");
        let h = harvest_edges(&d, "ix://test/FR-001", None, &resolver);
        assert!(h.edges.iter().all(|e| e.target.starts_with("ix://")));
    }

    #[test]
    fn unresolvable_target_keeps_bare_and_emits_diagnostic() {
        let d = doc();
        let resolver = MockResolver::new();
        let h = harvest_edges(&d, "ix://test/FR-001", None, &resolver);
        assert!(h
            .diagnostics
            .iter()
            .any(|d| matches!(d, Diagnostic::UnresolvableEdgeTarget { .. })));
        assert!(h.edges.iter().any(|e| e.target == "StR-001"));
    }

    #[test]
    fn duplicate_edges_are_deduped_with_diagnostic_naming_source_type_target() {
        let d = parse_document(
            "---\n\
             id: FR-001\n\
             depends_on:\n- FR-002\n- FR-002\n\
             ---\nbody\n",
        );
        let h = harvest_edges(&d, "ix://test/FR-001", None, &IdentityResolver);
        assert_eq!(h.edges.len(), 1);
        let dup = h
            .diagnostics
            .iter()
            .find_map(|x| match x {
                Diagnostic::DuplicateEdgeDropped {
                    source,
                    edge_type,
                    target,
                } => Some((source.as_str(), edge_type.as_str(), target.as_str())),
                _ => None,
            })
            .expect("DuplicateEdgeDropped");
        assert_eq!(dup, ("ix://test/FR-001", "depends_on", "FR-002"));
    }

    /// FR-015-AC-7: `harvest_edges` is deterministic across threads.
    /// Run the same harvest from 64 threads and assert every result
    /// matches the baseline byte-for-byte (Vec equality includes
    /// edge ordering — catches HashSet-iteration-order leakage).
    #[test]
    fn harvest_edges_deterministic_under_64_threads() {
        use std::sync::Arc;
        use std::thread;
        let doc = parse_document(
            "---\n\
             id: FR-001\n\
             depends_on:\n- FR-002\n- FR-003\n- FR-004\n\
             parent: SPEC-A\n\
             relationships:\n\
             - type: implements\n  target: StR-001\n\
             - type: requires\n  target: NFR-001\n\
             ---\nbody\n",
        );
        let arc_doc = Arc::new(doc);
        let baseline = harvest_edges(&arc_doc, "ix://test/FR-001", None, &IdentityResolver);
        let handles: Vec<_> = (0..64)
            .map(|_| {
                let d = Arc::clone(&arc_doc);
                let baseline = baseline.clone();
                thread::spawn(move || {
                    let h = harvest_edges(&d, "ix://test/FR-001", None, &IdentityResolver);
                    assert_eq!(h, baseline, "non-deterministic edge harvest");
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
    }

    #[test]
    fn metadata_from_relationships_block_passes_through() {
        let d = doc();
        let h = harvest_edges(&d, "ix://test/FR-001", None, &IdentityResolver);
        let impl_edge = h.edges.iter().find(|e| e.r#type == "implements").unwrap();
        assert_eq!(
            impl_edge.metadata.get("cardinality"),
            Some(&serde_json::json!("1:1"))
        );
    }
}
