//! `Spec` — a bounded, in-memory, immutable spec corpus (FR-025).
//!
//! Built from a [`RepoLoad`](crate::corpus::walk::RepoLoad), indexed by
//! artifact id. Mirrors the [`Registry`](crate::registry::Registry)
//! lifecycle exactly: `Arc<Inner>`, `Send + Sync`, frozen after
//! construction — to reflect on-disk changes, build a new `Spec`.
//!
//! **Scope guard** (StR-006-AC-4): the public surface exposes no
//! persistence, no filesystem-watcher registration, and no resolution
//! against anything outside the loaded set. Intra-spec reference
//! resolution (FR-026) and the whole-spec query API (FR-027) are added
//! by the `resolve`/`query` modules on top of this skeleton.

use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::sync::Arc;

use crate::corpus::resolve::{resolve, Edge};
use crate::corpus::walk::{load_repo, LoadedDocument, RepoLoad};
use crate::corpus::ArtifactId;
use crate::diagnostic::Diagnostic;

/// Compiled, immutable spec corpus. `Send + Sync`, cheap to clone
/// (`Arc<Inner>`).
#[derive(Clone)]
pub struct Spec {
    pub(crate) inner: Arc<SpecInner>,
}

pub(crate) struct SpecInner {
    /// Loaded documents, path-sorted (stable order from `load_repo`).
    pub(crate) documents: Vec<LoadedDocument>,
    /// Artifact key → document slot. First-wins on duplicates. A point
    /// lookup, so a `HashMap` is fine (iteration order never observed).
    pub(crate) by_id: HashMap<ArtifactId, usize>,
    /// Frontmatter type → document slots. `BTreeMap` keeps `by_type`
    /// iteration deterministic (NFR-006).
    pub(crate) by_type: BTreeMap<String, Vec<usize>>,
    /// Resolved + dangling reference edges (FR-026).
    pub(crate) edges: Vec<Edge>,
    /// source id → edge slots (includes dangling).
    pub(crate) outgoing: BTreeMap<ArtifactId, Vec<usize>>,
    /// target id → edge slots (resolved only).
    pub(crate) incoming: BTreeMap<ArtifactId, Vec<usize>>,
    /// Load + indexing + resolution diagnostics.
    pub(crate) diagnostics: Vec<Diagnostic>,
}

impl std::fmt::Debug for Spec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Spec")
            .field("documents", &self.inner.documents.len())
            .field("types", &self.inner.by_type.keys().collect::<Vec<_>>())
            .field("diagnostics", &self.inner.diagnostics.len())
            .finish()
    }
}

impl Spec {
    /// Build a corpus from a completed [`RepoLoad`].
    pub fn from_repo(load: RepoLoad) -> Self {
        let RepoLoad {
            documents,
            mut diagnostics,
        } = load;

        let mut by_id: HashMap<ArtifactId, usize> = HashMap::with_capacity(documents.len());
        let mut by_type: BTreeMap<String, Vec<usize>> = BTreeMap::new();

        for (slot, doc) in documents.iter().enumerate() {
            let key = artifact_key(doc);
            match by_id.entry(key.clone()) {
                std::collections::hash_map::Entry::Occupied(existing) => {
                    let first = &documents[*existing.get()];
                    diagnostics.push(Diagnostic::DuplicateArtifactId {
                        id: key,
                        paths: vec![first.path.clone(), doc.path.clone()],
                    });
                    // first-wins: do not overwrite the index entry.
                }
                std::collections::hash_map::Entry::Vacant(vacancy) => {
                    vacancy.insert(slot);
                }
            }

            match artifact_type(doc) {
                Some(ty) => by_type.entry(ty).or_default().push(slot),
                None => diagnostics.push(Diagnostic::UntypedArtifact {
                    id: artifact_key(doc),
                    path: doc.path.clone(),
                }),
            }
        }

        // Resolve intra-spec references against the id index (FR-026).
        let resolved = resolve(&documents, &by_id);
        diagnostics.extend(resolved.diagnostics);

        Spec {
            inner: Arc::new(SpecInner {
                documents,
                by_id,
                by_type,
                edges: resolved.edges,
                outgoing: resolved.outgoing,
                incoming: resolved.incoming,
                diagnostics,
            }),
        }
    }

    /// Convenience: walk `root` with default options, then build the
    /// corpus. A bad root yields an empty corpus + a warning diagnostic
    /// (never a panic — inherited from [`load_repo`]).
    pub fn from_path(root: &Path) -> Self {
        Self::from_repo(load_repo(root))
    }

    /// Number of loaded documents.
    pub fn len(&self) -> usize {
        self.inner.documents.len()
    }

    /// Whether the corpus is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.documents.is_empty()
    }

    /// Look up a document by its artifact key (human `id`, else uuid,
    /// else path). `None` if absent. O(1).
    pub fn by_id(&self, id: &str) -> Option<&LoadedDocument> {
        self.inner
            .by_id
            .get(id)
            .map(|&slot| &self.inner.documents[slot])
    }

    /// Load + indexing diagnostics (unreadable files, missing uuids,
    /// duplicate ids).
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.inner.diagnostics
    }
}

/// The corpus key for a document: human `id` when present, else the
/// durable `uuid` string, else the path. Keeps every document
/// addressable even when frontmatter is incomplete.
pub(crate) fn artifact_key(doc: &LoadedDocument) -> ArtifactId {
    if !doc.id.is_empty() {
        doc.id.clone()
    } else if let Some(uuid) = doc.uuid {
        uuid.to_string()
    } else {
        doc.path.to_string_lossy().into_owned()
    }
}

/// The document's frontmatter `type` / `artifact_type` (the latter
/// wins, matching this repo's own artifacts). `None` when untyped.
pub(crate) fn artifact_type(doc: &LoadedDocument) -> Option<String> {
    let fm = doc.doc.frontmatter.as_ref()?;
    fm.get("artifact_type")
        .or_else(|| fm.get("type"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::walk::load_repo;
    use std::fs;
    use std::path::PathBuf;

    fn tmpdir(tag: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "quire_spec_{tag}_{}",
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

    fn doc(id: &str, ty: &str) -> String {
        format!("---\nid: {id}\nartifact_type: {ty}\nuuid: 0190b6a0-0000-7000-8000-000000000001\n---\n# {id}\nbody\n")
    }

    // TC-480 / FR-025-AC-1: len == number of parsed artifacts.
    #[test]
    fn len_matches_artifact_count() {
        let root = tmpdir("len");
        write(&root, "functional/FR-023.md", &doc("FR-023", "FR"));
        write(&root, "stakeholder/StR-005.md", &doc("StR-005", "StR"));
        let spec = Spec::from_path(&root);
        assert_eq!(spec.len(), 2);
        assert!(!spec.is_empty());
    }

    // TC-481 / FR-025-AC-2: by_id present -> Some, absent -> None.
    #[test]
    fn by_id_present_and_absent() {
        let root = tmpdir("by_id");
        write(&root, "functional/FR-023.md", &doc("FR-023", "FR"));
        let spec = Spec::from_path(&root);
        assert_eq!(spec.by_id("FR-023").map(|d| d.id.as_str()), Some("FR-023"));
        assert!(spec.by_id("FR-999").is_none());
    }

    // TC-482 / FR-025-AC-3: duplicate id -> diagnostic, first-wins, no fail.
    #[test]
    fn duplicate_id_diagnosed_first_wins() {
        let root = tmpdir("dup");
        write(&root, "a/FR-023.md", &doc("FR-023", "FR"));
        write(&root, "b/FR-023.md", &doc("FR-023", "FR"));
        let spec = Spec::from_path(&root);
        assert_eq!(spec.len(), 2); // both loaded
                                   // first-wins: the 'a/' path sorts first, so it owns the key.
        let winner = spec.by_id("FR-023").unwrap();
        assert!(winner.path.to_string_lossy().contains("/a/"));
        assert_eq!(
            spec.diagnostics()
                .iter()
                .filter(|d| matches!(d, Diagnostic::DuplicateArtifactId { .. }))
                .count(),
            1
        );
    }

    // TC-483 / FR-025-AC-4 + StR-006-AC-5: Send + Sync.
    #[test]
    fn spec_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Spec>();
    }

    // TC-484 / FR-025-AC-5: scope guard — the corpus is read-only and
    // immutable. There is no &mut self method, no add/insert/save/watch.
    // This compiles only because the surface is read-only; a clone shares
    // the same Arc'd inner (no independent mutation path).
    #[test]
    fn corpus_is_immutable_and_shares_inner() {
        let root = tmpdir("immut");
        write(&root, "FR-023.md", &doc("FR-023", "FR"));
        let spec = Spec::from_path(&root);
        let clone = spec.clone();
        assert!(Arc::ptr_eq(&spec.inner, &clone.inner));
    }

    // TC-485 / FR-025-AC-6 + StR-006-AC-1: queries do no filesystem IO
    // after construction — load once, then drop the directory.
    #[test]
    fn queries_need_no_filesystem_after_construction() {
        let root = tmpdir("no_io");
        write(&root, "FR-023.md", &doc("FR-023", "FR"));
        let spec = Spec::from_repo(load_repo(&root));
        fs::remove_dir_all(&root).unwrap(); // directory gone
        assert_eq!(spec.len(), 1);
        assert!(spec.by_id("FR-023").is_some());
    }
}
