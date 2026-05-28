//! Intra-spec reference resolution (FR-026).
//!
//! Harvests edge stubs from two already-parsed sources — frontmatter
//! `relationships` arrays and `ix://` body links — unifies + dedups
//! them, and resolves each target against the corpus id index. A
//! target present in the loaded set is [`Resolution::Resolved`];
//! anything else (including a target that lives only in a *different*
//! spec) is [`Resolution::Dangling`]. Resolution is O(edges) — one hash
//! lookup per stub — and never reaches outside the loaded set
//! (StR-006-AC-4).

use std::collections::{BTreeMap, BTreeSet, HashMap};

use regex::Regex;

use crate::corpus::spec::artifact_key;
use crate::corpus::walk::LoadedDocument;
use crate::corpus::ArtifactId;
use crate::diagnostic::Diagnostic;

/// A resolved or dangling reference between two artifacts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    /// Source artifact id (the document the reference was harvested from).
    pub source: ArtifactId,
    /// Target artifact id (last `/`-segment of an `ix://` URI, or a bare id).
    pub target: ArtifactId,
    /// Edge type — the `relationships` entry's `type`, or `references`
    /// for an `ix://` body link.
    pub edge_type: String,
    /// Whether the target is present in the loaded set.
    pub resolution: Resolution,
}

/// Whether an [`Edge`]'s target was found in the corpus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resolution {
    Resolved,
    Dangling,
}

/// Output of [`resolve`]: the edge set + forward/reverse indices +
/// dangling diagnostics.
pub(crate) struct ResolveOutput {
    pub edges: Vec<Edge>,
    /// source id → edge slots (includes dangling).
    pub outgoing: BTreeMap<ArtifactId, Vec<usize>>,
    /// target id → edge slots (resolved only — dangling has no target doc).
    pub incoming: BTreeMap<ArtifactId, Vec<usize>>,
    pub diagnostics: Vec<Diagnostic>,
}

/// Resolve every reference in `documents` against `by_id`. Deterministic
/// (stubs are collected into a sorted set before edges are built).
pub(crate) fn resolve(
    documents: &[LoadedDocument],
    by_id: &HashMap<ArtifactId, usize>,
) -> ResolveOutput {
    let ix_link = ix_link_regex();

    // Harvest into a sorted, deduplicated stub set: identical
    // (source, target, type) triples from frontmatter + body collapse
    // to one (FR-026-AC-8); same-pair-different-type stay distinct.
    let mut stubs: BTreeSet<(ArtifactId, ArtifactId, String)> = BTreeSet::new();
    for doc in documents {
        let source = artifact_key(doc);
        for (target_raw, edge_type) in harvest_frontmatter(doc) {
            stubs.insert((
                source.clone(),
                extract_target_id(&target_raw).to_string(),
                edge_type,
            ));
        }
        for target_raw in harvest_body_links(doc, &ix_link) {
            stubs.insert((
                source.clone(),
                extract_target_id(&target_raw).to_string(),
                "references".to_string(),
            ));
        }
    }

    let mut edges = Vec::with_capacity(stubs.len());
    let mut outgoing: BTreeMap<ArtifactId, Vec<usize>> = BTreeMap::new();
    let mut incoming: BTreeMap<ArtifactId, Vec<usize>> = BTreeMap::new();
    let mut diagnostics = Vec::new();

    for (source, target, edge_type) in stubs {
        let resolution = if by_id.contains_key(&target) {
            Resolution::Resolved
        } else {
            Resolution::Dangling
        };
        let idx = edges.len();
        outgoing.entry(source.clone()).or_default().push(idx);
        match resolution {
            Resolution::Resolved => {
                incoming.entry(target.clone()).or_default().push(idx);
            }
            Resolution::Dangling => diagnostics.push(Diagnostic::DanglingReference {
                source: source.clone(),
                target: target.clone(),
                edge_type: edge_type.clone(),
            }),
        }
        edges.push(Edge {
            source,
            target,
            edge_type,
            resolution,
        });
    }

    ResolveOutput {
        edges,
        outgoing,
        incoming,
        diagnostics,
    }
}

/// Harvest *all* edge stubs (frontmatter + body `ix://` links) for a
/// single document, dedup'd. Returns `(target_id, edge_type)` pairs
/// where `target_id` is already reduced via [`extract_target_id`].
/// This is the public single-doc shape used by the Python binding
/// (`quire.harvest_edges`) — corpus-level resolution still goes
/// through [`resolve`].
pub fn harvest_edges(doc: &LoadedDocument) -> Vec<(String, String)> {
    let ix_link = ix_link_regex();
    let mut out: BTreeSet<(String, String)> = BTreeSet::new();
    for (target_raw, edge_type) in harvest_frontmatter(doc) {
        out.insert((extract_target_id(&target_raw).to_string(), edge_type));
    }
    for target_raw in harvest_body_links(doc, &ix_link) {
        out.insert((
            extract_target_id(&target_raw).to_string(),
            "references".to_string(),
        ));
    }
    out.into_iter().collect()
}

/// `(target, edge_type)` pairs from the document's frontmatter
/// `relationships` array. Entries missing a `target` are skipped;
/// entries missing a `type` default to `references`.
fn harvest_frontmatter(doc: &LoadedDocument) -> Vec<(String, String)> {
    let Some(fm) = doc.doc.frontmatter.as_ref() else {
        return Vec::new();
    };
    let Some(rels) = fm.get("relationships").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    rels.iter()
        .filter_map(|entry| {
            let target = entry.get("target").and_then(|v| v.as_str())?;
            let edge_type = entry
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("references");
            Some((target.to_string(), edge_type.to_string()))
        })
        .collect()
}

/// Raw `ix://` URIs found in the document body.
fn harvest_body_links(doc: &LoadedDocument, re: &Regex) -> Vec<String> {
    re.find_iter(&doc.doc.raw)
        .map(|m| m.as_str().to_string())
        .collect()
}

/// The artifact id a reference target points at: the last `/`-segment
/// of an `ix://` URI, or a bare id unchanged (FR-026-AC-6).
pub(crate) fn extract_target_id(target: &str) -> &str {
    let rest = target.strip_prefix("ix://").unwrap_or(target);
    rest.rsplit('/').next().unwrap_or(rest)
}

fn ix_link_regex() -> Regex {
    // Match an ix:// URI up to whitespace or a closing delimiter so it
    // works for `[t](ix://…)`, `<ix://…>`, and bare occurrences.
    Regex::new(r#"ix://[^\s)\]>"']+"#).expect("static ix-link regex is valid")
}

#[cfg(test)]
mod tests {
    use super::*;

    // TC-491 / FR-026-AC-6: target-id extraction is a pure function.
    #[test]
    fn extract_target_id_handles_uri_and_bare() {
        assert_eq!(
            extract_target_id("ix://agent-ix/quire-rs/spec/functional/FR-021"),
            "FR-021"
        );
        assert_eq!(extract_target_id("FR-021"), "FR-021");
        assert_eq!(extract_target_id("ix://x/StR-001"), "StR-001");
    }

    #[test]
    fn ix_regex_extracts_from_markdown_link_and_autolink() {
        let re = ix_link_regex();
        let body = "see [the FR](ix://o/r/spec/functional/FR-021) and <ix://o/r/StR-001>.";
        let hits: Vec<_> = re.find_iter(body).map(|m| m.as_str()).collect();
        assert_eq!(
            hits,
            vec!["ix://o/r/spec/functional/FR-021", "ix://o/r/StR-001"]
        );
    }

    use crate::parser::parse_document;
    use std::path::PathBuf;

    fn loaded(id: &str, frontmatter_extra: &str, body: &str) -> LoadedDocument {
        let text = format!("---\nid: {id}\n{frontmatter_extra}---\n{body}");
        LoadedDocument {
            path: PathBuf::from(format!("{id}.md")),
            id: id.to_string(),
            uuid: None,
            doc: parse_document(&text),
        }
    }

    fn index(docs: &[LoadedDocument]) -> HashMap<ArtifactId, usize> {
        docs.iter()
            .enumerate()
            .map(|(i, d)| (artifact_key(d), i))
            .collect()
    }

    // TC-486 / FR-026-AC-1: frontmatter relationship to a present id -> Resolved.
    #[test]
    fn frontmatter_relationship_resolves() {
        let docs = vec![
            loaded(
                "FR-023",
                "relationships:\n  - target: \"ix://o/r/spec/stakeholder/StR-005\"\n    type: implements\n",
                "# body\n",
            ),
            loaded("StR-005", "", "# need\n"),
        ];
        let out = resolve(&docs, &index(&docs));
        let edge = out.edges.iter().find(|e| e.source == "FR-023").unwrap();
        assert_eq!(edge.target, "StR-005");
        assert_eq!(edge.edge_type, "implements");
        assert_eq!(edge.resolution, Resolution::Resolved);
    }

    // TC-487 / FR-026-AC-2: ix:// body link to a present id -> Resolved, type "references".
    #[test]
    fn body_link_resolves() {
        let docs = vec![
            loaded(
                "FR-023",
                "",
                "see [it](ix://o/r/spec/stakeholder/StR-005)\n",
            ),
            loaded("StR-005", "", "# need\n"),
        ];
        let out = resolve(&docs, &index(&docs));
        let edge = out.edges.iter().find(|e| e.target == "StR-005").unwrap();
        assert_eq!(edge.edge_type, "references");
        assert_eq!(edge.resolution, Resolution::Resolved);
    }

    // TC-488 / FR-026-AC-3: absent target -> Dangling + diagnostic; no failure.
    #[test]
    fn absent_target_is_dangling() {
        let docs = vec![loaded(
            "FR-023",
            "relationships:\n  - target: \"StR-999\"\n    type: implements\n",
            "# body\n",
        )];
        let out = resolve(&docs, &index(&docs));
        let edge = &out.edges[0];
        assert_eq!(edge.target, "StR-999");
        assert_eq!(edge.resolution, Resolution::Dangling);
        assert_eq!(
            out.diagnostics
                .iter()
                .filter(|d| matches!(d, Diagnostic::DanglingReference { .. }))
                .count(),
            1
        );
    }

    // TC-489 / FR-026-AC-4: cross-spec target (absent from loaded set) -> Dangling.
    #[test]
    fn cross_spec_target_is_dangling_not_resolved() {
        // Target lives only in a *different* spec — not in this index.
        let docs = vec![loaded(
            "FR-023",
            "relationships:\n  - target: \"ix://other-org/other-repo/spec/stakeholder/StR-001\"\n    type: requires\n",
            "# body\n",
        )];
        let out = resolve(&docs, &index(&docs));
        assert_eq!(out.edges[0].target, "StR-001");
        assert_eq!(out.edges[0].resolution, Resolution::Dangling);
    }

    // TC-490 / FR-026-AC-5: a Resolved edge is in both outgoing(src) and incoming(tgt).
    #[test]
    fn resolved_edge_is_bidirectional() {
        let docs = vec![
            loaded(
                "FR-023",
                "relationships:\n  - target: \"StR-005\"\n    type: implements\n",
                "# body\n",
            ),
            loaded("StR-005", "", "# need\n"),
        ];
        let out = resolve(&docs, &index(&docs));
        let from = out.outgoing.get("FR-023").unwrap();
        let to = out.incoming.get("StR-005").unwrap();
        assert_eq!(from.len(), 1);
        assert_eq!(to.len(), 1);
        assert_eq!(from[0], to[0]); // same edge slot
    }

    // TC-501 / FR-026-AC-8: identical (src,target,type) from both sources -> one edge;
    // same pair, different type -> two.
    #[test]
    fn dedup_collapses_identical_triples() {
        // frontmatter `references` StR-005 AND a body ix:// link to StR-005
        // (also type `references`) => one edge.
        let docs = vec![
            loaded(
                "FR-023",
                "relationships:\n  - target: \"StR-005\"\n    type: references\n",
                "see [it](ix://o/r/spec/stakeholder/StR-005)\n",
            ),
            loaded("StR-005", "", "# need\n"),
        ];
        let out = resolve(&docs, &index(&docs));
        let to_str005: Vec<_> = out
            .edges
            .iter()
            .filter(|e| e.source == "FR-023" && e.target == "StR-005")
            .collect();
        assert_eq!(to_str005.len(), 1, "identical triples should dedup");

        // Different type from each source => two distinct edges.
        let docs2 = vec![
            loaded(
                "FR-024",
                "relationships:\n  - target: \"StR-005\"\n    type: implements\n",
                "see [it](ix://o/r/spec/stakeholder/StR-005)\n",
            ),
            loaded("StR-005", "", "# need\n"),
        ];
        let out2 = resolve(&docs2, &index(&docs2));
        let types: BTreeSet<_> = out2
            .edges
            .iter()
            .filter(|e| e.source == "FR-024" && e.target == "StR-005")
            .map(|e| e.edge_type.clone())
            .collect();
        assert_eq!(
            types,
            BTreeSet::from(["implements".to_string(), "references".to_string()])
        );
    }
}
