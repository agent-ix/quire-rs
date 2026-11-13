//! Whole-spec query API (FR-027).
//!
//! Read-only views over a constructed [`Spec`] and its resolved edge
//! set (FR-026). Nothing here reads the filesystem or re-parses. All
//! results are returned sorted by id so they are reproducible
//! (NFR-006). Reachability beyond one hop is composed by the caller —
//! there is no traversal/query DSL (StR-006).

use crate::corpus::resolve::{Edge, Resolution};
use crate::corpus::spec::{artifact_key, artifact_type, Spec};
use crate::corpus::walk::LoadedDocument;

impl Spec {
    /// Every document whose frontmatter `type` equals
    /// `ty`, sorted by id. Untyped documents are never returned (they
    /// were diagnosed as `UntypedArtifact` at construction, FR-027-AC-9).
    pub fn by_type(&self, ty: &str) -> Vec<&LoadedDocument> {
        self.by_type_slots(ty)
            .into_iter()
            .map(|slot| self.doc(slot))
            .collect()
    }

    /// Every document of type `ty` in the whole corpus, whatever the view.
    pub(crate) fn corpus_by_type(&self, ty: &str) -> Vec<&LoadedDocument> {
        self.inner
            .by_type
            .get(ty)
            .map(|slots| slots.iter().map(|&slot| self.doc(slot)).collect())
            .unwrap_or_default()
    }

    /// Slots of every document of type `ty` in the whole corpus, whatever the
    /// view, in slot order.
    pub(crate) fn corpus_type_slots(&self, ty: &str) -> Vec<usize> {
        self.inner.by_type.get(ty).cloned().unwrap_or_default()
    }

    /// Whether `slot` is in the member this view sees.
    pub(crate) fn in_view(&self, slot: usize) -> bool {
        self.scope
            .is_none_or(|member| self.member_of(slot) == member)
    }

    /// Slots of the documents of type `ty` this view sees, sorted by id.
    pub(crate) fn by_type_slots(&self, ty: &str) -> Vec<usize> {
        let mut slots: Vec<usize> = self
            .inner
            .by_type
            .get(ty)
            .map(|slots| {
                slots
                    .iter()
                    .copied()
                    .filter(|&slot| self.scope.is_none_or(|m| self.member_of(slot) == m))
                    .collect()
            })
            .unwrap_or_default();
        slots.sort_by_key(|&slot| (artifact_key(self.doc(slot)), slot));
        slots
    }

    /// Resolved edges whose target is `id` (reverse-edge lookup),
    /// sorted by source id. Dangling edges have no resolved target and
    /// never appear here.
    pub fn referencing(&self, id: &str) -> Vec<&Edge> {
        let mut edges: Vec<&Edge> = self
            .slot_of(id)
            .map(|slot| self.edges_to(slot))
            .unwrap_or_default();
        edges.sort_by(|a, b| {
            a.source
                .cmp(&b.source)
                .then(a.source_repo.cmp(&b.source_repo))
                .then(a.edge_type.cmp(&b.edge_type))
        });
        edges
    }

    /// All edges whose source is `id` (resolved *and* dangling), sorted
    /// by target id.
    pub fn outgoing(&self, id: &str) -> Vec<&Edge> {
        let mut edges: Vec<&Edge> = self
            .slot_of(id)
            .map(|slot| self.edges_from(slot))
            .unwrap_or_default();
        edges.sort_by(|a, b| {
            a.target
                .cmp(&b.target)
                .then(a.edge_type.cmp(&b.edge_type))
                .then(a.target_repo.cmp(&b.target_repo))
        });
        edges
    }

    /// Edges whose source is the document at `slot`.
    pub(crate) fn edges_from(&self, slot: usize) -> Vec<&Edge> {
        self.inner
            .outgoing
            .get(&slot)
            .map(|edges| edges.iter().map(|&i| &self.inner.edges[i]).collect())
            .unwrap_or_default()
    }

    /// Resolved edges whose target is the document at `slot`.
    pub(crate) fn edges_to(&self, slot: usize) -> Vec<&Edge> {
        self.inner
            .incoming
            .get(&slot)
            .map(|edges| edges.iter().map(|&i| &self.inner.edges[i]).collect())
            .unwrap_or_default()
    }

    /// Every edge of the whole corpus, whatever the view: a relation reads the
    /// corpus-wide edge set (FR-085).
    pub(crate) fn corpus_edges(&self) -> &[Edge] {
        &self.inner.edges
    }

    /// Every dangling edge in the corpus (target absent from the loaded
    /// set), sorted by `(source, target)`.
    pub fn dangling(&self) -> Vec<&Edge> {
        let mut edges: Vec<&Edge> = self
            .edges()
            .iter()
            .filter(|e| e.resolution == Resolution::Dangling)
            .collect();
        edges.sort_by(|a, b| {
            a.source
                .cmp(&b.source)
                .then(a.target.cmp(&b.target))
                .then(a.source_repo.cmp(&b.source_repo))
        });
        edges
    }

    /// Every document of `of_type` that has **no** resolved outgoing
    /// edge of `missing_edge_type` (optionally constrained to a target
    /// of `toward_type`) — the traceability-gap query. Sorted by id.
    ///
    /// `orphans("FR", "implements", Some("StR"))` → every FR with no
    /// resolved `implements` edge to a StR.
    pub fn orphans(
        &self,
        of_type: &str,
        missing_edge_type: &str,
        toward_type: Option<&str>,
    ) -> Vec<&LoadedDocument> {
        self.by_type_slots(of_type)
            .into_iter()
            .filter(|&slot| {
                !self.edges_from(slot).iter().any(|e| {
                    e.resolution == Resolution::Resolved
                        && e.edge_type == missing_edge_type
                        && toward_type.is_none_or(|tt| {
                            e.target_slot
                                .and_then(|t| artifact_type(self.doc(t)))
                                .is_some_and(|t| t == tt)
                        })
                })
            })
            .map(|slot| self.doc(slot))
            .collect()
    }

    /// All resolved + dangling edges, in stored (deterministic) order. A member
    /// view yields the edges whose source is in that member.
    pub fn edges(&self) -> &[Edge] {
        match self.scope {
            Some(member) => &self.inner.edges[self.inner.members[member].edges.clone()],
            None => &self.inner.edges,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::Diagnostic;
    use ix_trace_rs::trace;
    use std::fs;
    use std::path::{Path, PathBuf};

    fn tmpdir(tag: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "quire_query_{tag}_{}",
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

    /// FR with an `implements` edge to a StR.
    fn fr(id: &str, implements: Option<&str>) -> String {
        match implements {
            Some(t) => format!(
                "---\nid: {id}\ntype: FR\nrelationships:\n  - target: \"{t}\"\n    type: implements\n---\n# {id}\n"
            ),
            None => format!("---\nid: {id}\ntype: FR\n---\n# {id}\n"),
        }
    }

    fn build(root: &Path) -> Spec {
        write(root, "f/FR-001.md", &fr("FR-001", Some("StR-001")));
        write(root, "f/FR-002.md", &fr("FR-002", None)); // orphan: no StR
        write(
            root,
            "s/StR-001.md",
            "---\nid: StR-001\ntype: StR\n---\n# need\n",
        );
        Spec::from_path(root)
    }

    // Covers (FR-027-AC-1, US-012-AC-1).
    #[trace("FR-027-AC-1", "US-012-AC-1")]
    #[test]
    fn by_type_returns_exactly_that_type_sorted() {
        let root = tmpdir("by_type");
        let spec = build(&root);
        let frs: Vec<_> = spec.by_type("FR").iter().map(|d| d.id.clone()).collect();
        assert_eq!(frs, vec!["FR-001", "FR-002"]);
        let strs: Vec<_> = spec.by_type("StR").iter().map(|d| d.id.clone()).collect();
        assert_eq!(strs, vec!["StR-001"]);
    }

    // Covers (FR-027-AC-2, US-012-AC-3).
    #[trace("FR-027-AC-2", "US-012-AC-3")]
    #[test]
    fn referencing_is_reverse_lookup() {
        let root = tmpdir("ref");
        let spec = build(&root);
        let refs = spec.referencing("StR-001");
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].source, "FR-001");
        assert!(spec.referencing("FR-002").is_empty());
    }

    // Covers (FR-027-AC-3, US-012-AC-2): orphan FRs lacking implements->StR.
    #[trace("FR-027-AC-3", "US-012-AC-2")]
    #[test]
    fn orphans_finds_untraced_frs() {
        let root = tmpdir("orphans");
        let spec = build(&root);
        let orphans: Vec<_> = spec
            .orphans("FR", "implements", Some("StR"))
            .iter()
            .map(|d| d.id.clone())
            .collect();
        assert_eq!(orphans, vec!["FR-002"]); // FR-001 has the edge; FR-002 doesn't
    }

    // untyped doc -> not in by_type, in by_id, diagnostic.
    #[trace("FR-027-AC-9")]
    #[test]
    fn untyped_document_is_excluded_and_diagnosed() {
        let root = tmpdir("untyped");
        write(root.as_path(), "x.md", "---\nid: X-1\n---\n# no type\n");
        let spec = Spec::from_path(&root);
        assert!(spec.by_type("FR").is_empty());
        assert!(spec.by_id("X-1").is_some());
        assert!(spec
            .diagnostics()
            .iter()
            .any(|d| matches!(d, Diagnostic::UntypedArtifact { .. })));
    }

    // outgoing includes dangling; dangling()/referencing agree.
    #[trace("FR-027-AC-5")]
    #[test]
    fn dangling_appears_in_outgoing_not_referencing() {
        let root = tmpdir("dangling");
        write(root.as_path(), "FR-009.md", &fr("FR-009", Some("StR-404")));
        let spec = Spec::from_path(&root);
        let out = spec.outgoing("FR-009");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].resolution, Resolution::Dangling);
        assert_eq!(spec.dangling().len(), 1);
        assert!(spec.referencing("StR-404").is_empty());
    }

    // query results deterministic across runs (NFR-006).
    #[trace("FR-027-AC-6")]
    #[test]
    fn query_results_are_deterministic() {
        let root = tmpdir("determ");
        let spec1 = build(&root);
        let spec2 = build(&root);
        let a: Vec<_> = spec1.by_type("FR").iter().map(|d| d.id.clone()).collect();
        let b: Vec<_> = spec2.by_type("FR").iter().map(|d| d.id.clone()).collect();
        assert_eq!(a, b);
    }
}
