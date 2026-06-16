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
        let mut docs: Vec<&LoadedDocument> = self
            .inner
            .by_type
            .get(ty)
            .map(|slots| slots.iter().map(|&s| &self.inner.documents[s]).collect())
            .unwrap_or_default();
        docs.sort_by_key(|d| artifact_key(d));
        docs
    }

    /// Resolved edges whose target is `id` (reverse-edge lookup),
    /// sorted by source id. Dangling edges have no resolved target and
    /// never appear here.
    pub fn referencing(&self, id: &str) -> Vec<&Edge> {
        let mut edges: Vec<&Edge> = self
            .inner
            .incoming
            .get(id)
            .map(|slots| slots.iter().map(|&i| &self.inner.edges[i]).collect())
            .unwrap_or_default();
        edges.sort_by(|a, b| a.source.cmp(&b.source).then(a.edge_type.cmp(&b.edge_type)));
        edges
    }

    /// All edges whose source is `id` (resolved *and* dangling), sorted
    /// by target id.
    pub fn outgoing(&self, id: &str) -> Vec<&Edge> {
        let mut edges: Vec<&Edge> = self
            .inner
            .outgoing
            .get(id)
            .map(|slots| slots.iter().map(|&i| &self.inner.edges[i]).collect())
            .unwrap_or_default();
        edges.sort_by(|a, b| a.target.cmp(&b.target).then(a.edge_type.cmp(&b.edge_type)));
        edges
    }

    /// Every dangling edge in the corpus (target absent from the loaded
    /// set), sorted by `(source, target)`.
    pub fn dangling(&self) -> Vec<&Edge> {
        let mut edges: Vec<&Edge> = self
            .inner
            .edges
            .iter()
            .filter(|e| e.resolution == Resolution::Dangling)
            .collect();
        edges.sort_by(|a, b| a.source.cmp(&b.source).then(a.target.cmp(&b.target)));
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
        self.by_type(of_type)
            .into_iter()
            .filter(|doc| {
                let src = artifact_key(doc);
                let has_edge = self
                    .inner
                    .outgoing
                    .get(&src)
                    .map(|slots| {
                        slots.iter().any(|&i| {
                            let e = &self.inner.edges[i];
                            e.resolution == Resolution::Resolved
                                && e.edge_type == missing_edge_type
                                && toward_type
                                    .map_or(true, |tt| self.target_has_type(&e.target, tt))
                        })
                    })
                    .unwrap_or(false);
                !has_edge
            })
            .collect()
    }

    /// All resolved + dangling edges, in stored (deterministic) order.
    pub fn edges(&self) -> &[Edge] {
        &self.inner.edges
    }

    fn target_has_type(&self, target: &str, ty: &str) -> bool {
        self.by_id(target)
            .and_then(artifact_type)
            .is_some_and(|t| t == ty)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::Diagnostic;
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

    // TC-493 / FR-027-AC-1, US-012-AC-1.
    #[test]
    fn by_type_returns_exactly_that_type_sorted() {
        let root = tmpdir("by_type");
        let spec = build(&root);
        let frs: Vec<_> = spec.by_type("FR").iter().map(|d| d.id.clone()).collect();
        assert_eq!(frs, vec!["FR-001", "FR-002"]);
        let strs: Vec<_> = spec.by_type("StR").iter().map(|d| d.id.clone()).collect();
        assert_eq!(strs, vec!["StR-001"]);
    }

    // TC-494 / FR-027-AC-2, US-012-AC-3.
    #[test]
    fn referencing_is_reverse_lookup() {
        let root = tmpdir("ref");
        let spec = build(&root);
        let refs = spec.referencing("StR-001");
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].source, "FR-001");
        assert!(spec.referencing("FR-002").is_empty());
    }

    // TC-495 / FR-027-AC-3, US-012-AC-2: orphan FRs lacking implements->StR.
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

    // TC-500 / FR-027-AC-9: untyped doc -> not in by_type, in by_id, diagnostic.
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

    // TC-497 / FR-027-AC-5: outgoing includes dangling; dangling()/referencing agree.
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

    // TC-498 / FR-027-AC-6 + NFR-006: query results deterministic across runs.
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
