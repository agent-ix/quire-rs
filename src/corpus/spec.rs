//! `Spec` — a bounded, in-memory, immutable spec corpus (FR-025).
//!
//! Built from a [`RepoLoad`], indexed by
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
use std::ops::Range;
use std::path::Path;
use std::sync::Arc;

use crate::corpus::resolve::{ix_authority, resolve, Edge, Index};
use crate::corpus::walk::{load_repo, LoadedDocument, RepoLoad};
use crate::corpus::ArtifactId;
use crate::diagnostic::Diagnostic;

/// One repository supplied to a multi-repository corpus (FR-085): a loaded
/// repository plus the `<org>/<repo>` identity the caller declares for it.
/// Identity is never derived from content or the filesystem.
#[derive(Debug, Clone)]
pub struct CorpusMember {
    /// Declared `<org>/<repo>` identity. Optional only in a one-member corpus.
    pub identity: Option<String>,
    /// The member's loaded documents.
    pub load: RepoLoad,
}

/// Why a corpus could not be built or validated (FR-085).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CorpusError {
    /// A corpus is built from one or more members.
    #[error("a corpus is built from one or more members; none were supplied")]
    NoMembers,
    /// A corpus of several members fixes each identity at construction.
    #[error("the corpus has {members} members; each declares its identity at construction")]
    IdentityFixed {
        /// Members in the corpus.
        members: usize,
    },
    /// A corpus of two or more members requires an identity on every member.
    #[error(
        "corpus member {member} declares no repository identity; a corpus of several members \
         requires one per member"
    )]
    MissingIdentity {
        /// Zero-based position of the member in the supplied order.
        member: usize,
    },
    /// Two members declared one identity (compared with ASCII case folded).
    #[error("corpus members declare the same repository identity '{identity}'")]
    DuplicateIdentity {
        /// The identity as the later member spelled it.
        identity: String,
    },
    /// `validate_corpus` takes exactly one root pair per member.
    #[error("corpus has {members} member(s) but {roots} root pair(s) were supplied")]
    RootCount {
        /// Members in the corpus.
        members: usize,
        /// Root pairs supplied.
        roots: usize,
    },
}

/// Compiled, immutable spec corpus. `Send + Sync`, cheap to clone
/// (`Arc<Inner>`).
#[derive(Clone)]
pub struct Spec {
    pub(crate) inner: Arc<SpecInner>,
    /// When set, this value is a view of one member of a multi-member corpus:
    /// document-iterating queries and by-id lookups see only that member while
    /// edges still resolve across the whole corpus (FR-085).
    pub(crate) scope: Option<usize>,
}

/// One member's place in the corpus.
#[derive(Clone)]
pub(crate) struct Member {
    /// Declared identity, stored as given.
    pub(crate) identity: Option<Arc<str>>,
    /// The member's document slots (members are stored contiguously).
    pub(crate) slots: Range<usize>,
    /// The member's edges, which `resolve` emits grouped by source member.
    pub(crate) edges: Range<usize>,
    /// The member's slice of the load diagnostics.
    pub(crate) load_diagnostics: Range<usize>,
}

#[derive(Clone)]
pub(crate) struct SpecInner {
    /// Loaded documents: members in supplied order, each path-sorted (stable
    /// order from `load_repo`).
    pub(crate) documents: Vec<LoadedDocument>,
    pub(crate) members: Vec<Member>,
    /// Document slot → member.
    pub(crate) member_of: Vec<usize>,
    /// Per member: artifact key → document slot. First-wins on duplicates. A
    /// point lookup, so a `HashMap` is fine (iteration order never observed).
    pub(crate) ids: Vec<HashMap<ArtifactId, usize>>,
    /// Artifact key → the slot in each member holding it (member order).
    pub(crate) holders: HashMap<ArtifactId, Vec<usize>>,
    /// ASCII-folded member identity → member.
    pub(crate) identities: BTreeMap<String, usize>,
    /// Frontmatter type → document slots. `BTreeMap` keeps `by_type`
    /// iteration deterministic (NFR-006).
    pub(crate) by_type: BTreeMap<String, Vec<usize>>,
    /// Resolved + dangling reference edges (FR-026), grouped by source member.
    pub(crate) edges: Vec<Edge>,
    /// source slot → edge slots (includes dangling).
    pub(crate) outgoing: BTreeMap<usize, Vec<usize>>,
    /// target slot → edge slots (resolved only).
    pub(crate) incoming: BTreeMap<usize, Vec<usize>>,
    /// Load + indexing + resolution diagnostics: the members' load
    /// diagnostics first (member order), then indexing, then resolution.
    pub(crate) diagnostics: Vec<Diagnostic>,
    /// Where the resolution diagnostics begin, so resolution can re-run.
    resolved_from: usize,
}

impl SpecInner {
    /// Resolve every reference under the members' current identities. Run at
    /// construction and again when a one-member corpus declares one.
    fn resolve_edges(&mut self) {
        let declared: Vec<Option<Arc<str>>> =
            self.members.iter().map(|m| m.identity.clone()).collect();
        let resolved = resolve(&Index {
            documents: &self.documents,
            member_of: &self.member_of,
            member_count: self.members.len(),
            ids: &self.ids,
            identities: &self.identities,
            identity_mode: declared.iter().any(Option::is_some),
            declared: &declared,
        });
        self.diagnostics.truncate(self.resolved_from);
        self.diagnostics.extend(resolved.diagnostics);
        for (member, edges) in self.members.iter_mut().zip(resolved.edge_spans) {
            member.edges = edges;
        }
        self.edges = resolved.edges;
        self.outgoing = resolved.outgoing;
        self.incoming = resolved.incoming;
    }
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

/// ASCII-case-folded identity, the comparison form (FR-038-AC-10).
pub(crate) fn fold_identity(identity: &str) -> String {
    identity.to_ascii_lowercase()
}

impl Spec {
    /// Build a one-member corpus from a completed [`RepoLoad`]. It declares no
    /// identity, so `ix://` targets resolve lexically by their trailing id
    /// (FR-026); [`Spec::with_bundle_package`] declares one.
    pub fn from_repo(load: RepoLoad) -> Self {
        Self::build(vec![CorpusMember {
            identity: None,
            load,
        }])
        .expect("a one-member corpus has no identity to reject")
    }

    /// Build a corpus from one or more repositories (FR-085). Every member of
    /// a corpus of two or more must declare a distinct `<org>/<repo>` identity;
    /// identities compare with ASCII case folded.
    pub fn from_members(members: Vec<CorpusMember>) -> Result<Self, CorpusError> {
        Self::build(members)
    }

    fn build(members: Vec<CorpusMember>) -> Result<Self, CorpusError> {
        if members.is_empty() {
            return Err(CorpusError::NoMembers);
        }
        let several = members.len() > 1;
        let mut identities: BTreeMap<String, usize> = BTreeMap::new();
        let mut declared: Vec<Option<Arc<str>>> = Vec::with_capacity(members.len());
        for (index, member) in members.iter().enumerate() {
            let identity = member.identity.as_deref().filter(|i| !i.is_empty());
            match identity {
                None if several => return Err(CorpusError::MissingIdentity { member: index }),
                None => declared.push(None),
                Some(identity) => {
                    if identities.insert(fold_identity(identity), index).is_some() {
                        return Err(CorpusError::DuplicateIdentity {
                            identity: identity.to_string(),
                        });
                    }
                    declared.push(Some(Arc::from(identity)));
                }
            }
        }

        let mut documents: Vec<LoadedDocument> = Vec::new();
        let mut member_of: Vec<usize> = Vec::new();
        let mut spans: Vec<(Range<usize>, Range<usize>)> = Vec::with_capacity(members.len());
        let mut diagnostics: Vec<Diagnostic> = Vec::new();
        for (index, member) in members.into_iter().enumerate() {
            let start = documents.len();
            let diag_start = diagnostics.len();
            member_of.extend(std::iter::repeat_n(index, member.load.documents.len()));
            documents.extend(member.load.documents);
            diagnostics.extend(member.load.diagnostics);
            spans.push((start..documents.len(), diag_start..diagnostics.len()));
        }

        let mut ids: Vec<HashMap<ArtifactId, usize>> = vec![HashMap::new(); spans.len()];
        let mut holders: HashMap<ArtifactId, Vec<usize>> = HashMap::with_capacity(documents.len());
        let mut by_type: BTreeMap<String, Vec<usize>> = BTreeMap::new();

        for (slot, doc) in documents.iter().enumerate() {
            let key = artifact_key(doc);
            match ids[member_of[slot]].entry(key.clone()) {
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
                    holders.entry(key).or_default().push(slot);
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

        let members = spans
            .into_iter()
            .zip(declared)
            .map(|((slots, load_diagnostics), identity)| Member {
                identity,
                slots,
                edges: 0..0,
                load_diagnostics,
            })
            .collect();
        let mut inner = SpecInner {
            documents,
            members,
            member_of,
            ids,
            holders,
            identities,
            by_type,
            edges: Vec::new(),
            outgoing: BTreeMap::new(),
            incoming: BTreeMap::new(),
            resolved_from: diagnostics.len(),
            diagnostics,
        };
        inner.resolve_edges();
        Ok(Spec {
            inner: Arc::new(inner),
            scope: None,
        })
    }

    /// Declare the repository identity `<org>/<repo>` of a one-member corpus.
    /// Resolution re-runs under it: an `ix://<org>/<repo>/...` target resolves
    /// within the member only when it names this identity, and one naming a
    /// repository no member declares stays dangling (FR-085); `validate_bundle`
    /// also qualifies `## Relationships` targets under it (FR-076); without it
    /// a relationships table is reported `unavailable` with reason
    /// `no-bundle-package`. An empty string clears the identity. A corpus of
    /// several members fixes each identity at [`Spec::from_members`], so this
    /// is [`CorpusError::IdentityFixed`] there.
    pub fn with_bundle_package(self, package: &str) -> Result<Self, CorpusError> {
        let members = self.inner.members.len();
        if members != 1 {
            return Err(CorpusError::IdentityFixed { members });
        }
        // Reuse the documents and indices when this is the only handle.
        let mut inner = Arc::try_unwrap(self.inner).unwrap_or_else(|shared| (*shared).clone());
        let identity = (!package.is_empty()).then(|| Arc::<str>::from(package));
        inner.identities.clear();
        if identity.is_some() {
            inner.identities.insert(fold_identity(package), 0);
        }
        inner.members[0].identity = identity;
        inner.resolve_edges();
        Ok(Spec {
            inner: Arc::new(inner),
            scope: None,
        })
    }

    /// The declared repository identity of a one-member corpus, if any.
    pub fn bundle_package(&self) -> Option<&str> {
        match self.inner.members.as_slice() {
            [only] => only.identity.as_deref(),
            _ => None,
        }
    }

    /// Convenience: walk `root` with default options, then build the
    /// corpus. A bad root yields an empty corpus + a warning diagnostic
    /// (never a panic — inherited from [`load_repo`]).
    pub fn from_path(root: &Path) -> Self {
        Self::from_repo(load_repo(root))
    }

    /// Number of members in the corpus.
    pub fn member_count(&self) -> usize {
        self.inner.members.len()
    }

    /// The identity member `member` declared, stored as given.
    pub fn member_identity(&self, member: usize) -> Option<&str> {
        self.inner.members.get(member)?.identity.as_deref()
    }

    /// A view of one member: queries that walk documents or look up by id see
    /// that member only, while its edges still resolve across the corpus.
    pub(crate) fn member_view(&self, member: usize) -> Spec {
        Spec {
            inner: Arc::clone(&self.inner),
            scope: Some(member),
        }
    }

    /// The slots of the documents member `member` holds.
    pub(crate) fn member_slots(&self, member: usize) -> Range<usize> {
        self.inner.members[member].slots.clone()
    }

    /// The member an authored reference `raw`, harvested from a document of
    /// `source_member`, is looked up in (FR-085): the member declaring the
    /// `ix://<org>/<repo>/` authority when members declare identities, the
    /// source's own member otherwise. `None` when the authority names no
    /// loaded member. Mirrors the lookup `resolve` makes.
    pub(crate) fn reference_member(&self, source_member: usize, raw: &str) -> Option<usize> {
        let identity_mode = self.inner.members.iter().any(|m| m.identity.is_some());
        if !identity_mode {
            return Some(0);
        }
        match ix_authority(raw) {
            Some(authority) => self
                .inner
                .identities
                .get(&fold_identity(&authority))
                .copied(),
            None => Some(source_member),
        }
    }

    /// The view's member, or the identity of the lone member of an unscoped
    /// corpus, which is what labels findings.
    pub(crate) fn view_identity(&self) -> Option<&str> {
        match self.scope {
            Some(member) => self.member_identity(member),
            None => self.bundle_package(),
        }
    }

    /// Number of loaded documents.
    pub fn len(&self) -> usize {
        self.documents().len()
    }

    /// Whether the corpus is empty.
    pub fn is_empty(&self) -> bool {
        self.documents().is_empty()
    }

    /// Every loaded document in stable path order (member order across a
    /// multi-member corpus; one member for a member view).
    ///
    /// This is a read-only view over the corpus header tier. Consumers such as
    /// the assurance exporter can project authoritative identities and exact
    /// source bytes without re-walking or re-parsing the repository.
    pub fn documents(&self) -> &[LoadedDocument] {
        match self.scope {
            Some(member) => &self.inner.documents[self.inner.members[member].slots.clone()],
            None => &self.inner.documents,
        }
    }

    /// Every loaded document, in the stable path order `load_repo` produced.
    ///
    /// Crate-internal: the public query surface is `by_id`/`by_type`, and a
    /// check that wants "any document in the bundle" (FR-059's justified
    /// absence, which is recorded on the spec document rather than on a
    /// requirement) needs the whole set rather than one archetype's.
    pub(crate) fn all(&self) -> &[LoadedDocument] {
        self.documents()
    }

    /// The slot `id` names: in this view's member, or, over a whole corpus, the
    /// one member holding it. `None` when absent or held by several members.
    pub(crate) fn slot_of(&self, id: &str) -> Option<usize> {
        match self.scope {
            Some(member) => self.inner.ids[member].get(id).copied(),
            None => match self.inner.holders.get(id)?.as_slice() {
                [only] => Some(*only),
                _ => None,
            },
        }
    }

    /// The document at a slot.
    pub(crate) fn doc(&self, slot: usize) -> &LoadedDocument {
        &self.inner.documents[slot]
    }

    /// The member a slot belongs to.
    pub(crate) fn member_of(&self, slot: usize) -> usize {
        self.inner.member_of[slot]
    }

    /// Whether any member holds a document of `ty`, whatever the view.
    pub(crate) fn corpus_has_type(&self, ty: &str) -> bool {
        self.inner.by_type.contains_key(ty)
    }

    /// Look up a document by its artifact key (human `id`, else uuid,
    /// else path). O(1). Over a corpus of several members it answers only when
    /// exactly one member holds the id (FR-027).
    pub fn by_id(&self, id: &str) -> Option<&LoadedDocument> {
        self.slot_of(id).map(|slot| self.doc(slot))
    }

    /// Look up `id` in the member declaring `identity` (compared with ASCII
    /// case folded). `None` when no member declares it or it does not hold the
    /// id. O(1).
    pub fn by_qualified_id(&self, identity: &str, id: &str) -> Option<&LoadedDocument> {
        let member = *self.inner.identities.get(&fold_identity(identity))?;
        self.inner.ids[member].get(id).map(|&slot| self.doc(slot))
    }

    /// The diagnostics the loads of the members this view sees carried
    /// (unreadable files, files without frontmatter), in member order.
    pub(crate) fn load_diagnostics(&self) -> &[Diagnostic] {
        let inner = &self.inner;
        match self.scope {
            Some(member) => &inner.diagnostics[inner.members[member].load_diagnostics.clone()],
            None => {
                &inner.diagnostics[..inner.members.last().map_or(0, |m| m.load_diagnostics.end)]
            }
        }
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

/// The document's concept type — the same `type` read as the canonical
/// discriminator ([`crate::query::concept_type`]), answered from the
/// header tier without a body parse (CR-047). `None` when untyped.
pub(crate) fn artifact_type(doc: &LoadedDocument) -> Option<String> {
    doc.concept_type().map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::walk::load_repo;
    use ix_trace_rs::trace;
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
        format!("---\nid: {id}\ntype: {ty}\nuuid: 0190b6a0-0000-7000-8000-000000000001\n---\n# {id}\nbody\n")
    }

    // len == number of parsed artifacts.
    #[trace("FR-025-AC-1")]
    #[test]
    fn len_matches_artifact_count() {
        let root = tmpdir("len");
        write(&root, "functional/FR-023.md", &doc("FR-023", "FR"));
        write(&root, "stakeholder/StR-005.md", &doc("StR-005", "StR"));
        let spec = Spec::from_path(&root);
        assert_eq!(spec.len(), 2);
        assert!(!spec.is_empty());
    }

    // by_id present -> Some, absent -> None.
    #[trace("FR-025-AC-2")]
    #[test]
    fn by_id_present_and_absent() {
        let root = tmpdir("by_id");
        write(&root, "functional/FR-023.md", &doc("FR-023", "FR"));
        let spec = Spec::from_path(&root);
        assert_eq!(spec.by_id("FR-023").map(|d| d.id.as_str()), Some("FR-023"));
        assert!(spec.by_id("FR-999").is_none());
    }

    // duplicate id -> diagnostic, first-wins, no fail.
    #[trace("FR-025-AC-3")]
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

    // Send + Sync (StR-006-AC-5).
    #[trace("FR-025-AC-4")]
    #[test]
    fn spec_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Spec>();
    }

    // scope guard — the corpus is read-only and
    // immutable. There is no &mut self method, no add/insert/save/watch.
    // This compiles only because the surface is read-only; a clone shares
    // the same Arc'd inner (no independent mutation path).
    #[trace("FR-025-AC-5")]
    #[test]
    fn corpus_is_immutable_and_shares_inner() {
        let root = tmpdir("immut");
        write(&root, "FR-023.md", &doc("FR-023", "FR"));
        let spec = Spec::from_path(&root);
        let clone = spec.clone();
        assert!(Arc::ptr_eq(&spec.inner, &clone.inner));
    }

    // FR-025-AC-6 + StR-006-AC-1: queries do no filesystem IO
    // after construction — load once, then drop the directory.
    #[trace("FR-025-AC-6", "StR-006-AC-1", "US-012-AC-5")]
    #[test]
    fn queries_need_no_filesystem_after_construction() {
        let root = tmpdir("no_io");
        write(&root, "FR-023.md", &doc("FR-023", "FR"));
        let spec = Spec::from_repo(load_repo(&root));
        fs::remove_dir_all(&root).unwrap(); // directory gone
        assert_eq!(spec.len(), 1);
        assert!(spec.by_id("FR-023").is_some());
        // CR-047: the lazy body parse needs no filesystem either — the
        // verbatim text was captured at load, so first-touch
        // materialisation succeeds with the directory gone.
        let body = spec.by_id("FR-023").unwrap().body();
        assert_eq!(body.sections.len(), 1);
        assert_eq!(body.sections[0].heading, "FR-023");
    }

    // CR-047: the header-tier query surface — len,
    // by_id, by_type, diagnostics, and the FR-026/FR-027 edge queries —
    // completes with ZERO body parses; touching one body then parses
    // exactly that one document and no other.
    #[trace("FR-025-AC-7")]
    #[test]
    fn tc817_queries_parse_zero_bodies() {
        let root = tmpdir("lazy");
        write(
            &root,
            "f/FR-023.md",
            "---\nid: FR-023\ntype: FR\nrelationships:\n  - target: \"StR-005\"\n    type: implements\n---\n# FR-023\nsee also a dangling [link](ix://o/r/StR-404)\n",
        );
        write(&root, "f/FR-024.md", &doc("FR-024", "FR"));
        write(&root, "s/StR-005.md", &doc("StR-005", "StR"));
        let spec = Spec::from_path(&root);

        // Construction (walk + indexing + FR-026 resolution) plus every
        // header-tier query below runs off frontmatter + verbatim text.
        assert_eq!(spec.len(), 3);
        assert!(!spec.is_empty());
        assert!(spec.by_id("FR-023").is_some());
        assert_eq!(spec.by_type("FR").len(), 2);
        let _ = spec.diagnostics();
        assert!(!spec.edges().is_empty());
        assert!(!spec.outgoing("FR-023").is_empty());
        assert!(!spec.referencing("StR-005").is_empty());
        assert_eq!(spec.dangling().len(), 1);
        assert_eq!(spec.orphans("FR", "implements", Some("StR")).len(), 1);

        for d in &spec.inner.documents {
            assert!(
                !d.body_is_parsed(),
                "{}: body parsed during header-tier queries",
                d.id
            );
        }

        // First touch materialises exactly the touched document.
        let touched = spec.by_id("FR-023").unwrap();
        assert!(!touched.body().sections.is_empty());
        for d in &spec.inner.documents {
            assert_eq!(d.body_is_parsed(), d.id == "FR-023");
        }
    }
}
