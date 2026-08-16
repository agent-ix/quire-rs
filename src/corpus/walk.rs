//! Parallel repository walk + parse — `load_repo` (FR-024, NFR-015).
//!
//! Walks a directory tree (ignore-file aware), parses every markdown
//! file's **header tier** on a rayon pool, and returns the collection
//! plus per-file diagnostics. Per-file failures are non-fatal. Bodies
//! are NOT parsed here (CR-047): each [`LoadedDocument`] carries the
//! verbatim text and materialises its [`QuireDocument`] on first
//! [`body()`](LoadedDocument::body) access, exactly once.
//!
//! **Identity is read, never derived** (CR-002): the human `id` and
//! durable `uuid` come straight from frontmatter; nothing is hashed,
//! and no file is written.
//!
//! **No shared mutable state** (FR-024-AC-9): the walk is sequential
//! (I/O-bound), then parsing is a data-parallel `par_iter().map(…)
//! .collect()` of *owned* results — no `Mutex`/`RwLock`/`Atomic`.
//! Diagnostics are gathered after the parallel region. Output is
//! sorted by path so results are reproducible regardless of thread
//! scheduling (NFR-006). The per-document lazy body cell is not walk
//! state — see `body_cache.rs` for why it does not touch this
//! invariant.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use ignore::WalkBuilder;
use rayon::prelude::*;
use uuid::Uuid;

use crate::ast::QuireDocument;
use crate::corpus::body_cache::LazyBody;
use crate::diagnostic::Diagnostic;
use crate::parser::Header;

/// One loaded document: its read identity plus the two parse tiers —
/// the eager header tier (verbatim text + frontmatter, FR-005/CR-046)
/// and the lazy body tier ([`QuireDocument`], parsed on first
/// [`body()`](LoadedDocument::body) access, CR-047).
#[derive(Clone)]
pub struct LoadedDocument {
    /// Path on disk (absolute or root-relative as discovered).
    pub path: PathBuf,
    /// Human artifact id from frontmatter `id` (e.g. `"FR-023"`); the
    /// intra-spec resolution key. Empty string when absent.
    pub id: String,
    /// Durable catalog id from frontmatter `uuid` (a UUID7, authored by
    /// quire). `None` when absent or unparseable.
    pub uuid: Option<Uuid>,
    /// The full verbatim file text (header tier; what [`Self::raw`] serves).
    text: Arc<str>,
    /// The parsed header for walk-loaded documents. `None` only for
    /// [`Self::from_parsed`] documents, whose body cell is seeded at
    /// construction so the header is never needed to parse.
    header: Option<Header>,
    /// The lazy body tier (see `body_cache.rs`).
    body: LazyBody,
}

impl LoadedDocument {
    /// The full verbatim document text, as read from disk.
    pub fn raw(&self) -> &str {
        &self.text
    }

    /// The parsed frontmatter mapping (header tier — no body parse).
    /// Every walk-loaded corpus document has one; `None` can occur only
    /// for a [`Self::from_parsed`] document whose text carried no
    /// frontmatter block.
    pub fn frontmatter(&self) -> Option<&serde_json::Map<String, serde_json::Value>> {
        match &self.header {
            Some(h) => Some(&h.frontmatter),
            None => self.seeded_body().frontmatter.as_ref(),
        }
    }

    /// The document's concept type — frontmatter `type` as a string,
    /// exactly the [`crate::query::concept_type`] read, answered from
    /// the header tier without a body parse.
    pub fn concept_type(&self) -> Option<&str> {
        self.frontmatter()?.get("type").and_then(|v| v.as_str())
    }

    /// The parsed body tier. First access parses via the header
    /// (exactly once, no filesystem read — the text was captured at
    /// load); every later access returns the same cached value.
    pub fn body(&self) -> &QuireDocument {
        match &self.header {
            Some(header) => self.body.get_or_parse(&self.text, header),
            None => self.seeded_body(),
        }
    }

    /// Build a `LoadedDocument` from an already-parsed [`QuireDocument`]
    /// (test fixtures, fuzz targets, the PyO3 `harvest_edges` view). The
    /// body cell is seeded, so no re-parse ever happens and no header is
    /// needed; `frontmatter()` reports the parsed document's own map.
    pub fn from_parsed(path: PathBuf, id: String, uuid: Option<Uuid>, doc: QuireDocument) -> Self {
        LoadedDocument {
            path,
            id,
            uuid,
            text: doc.raw.clone().into(),
            header: None,
            body: LazyBody::seeded(doc),
        }
    }

    /// Whether the body tier has been materialised. Test observability
    /// only (TC-816/TC-817) — not part of the corpus contract.
    #[doc(hidden)]
    pub fn body_is_parsed(&self) -> bool {
        self.body.is_parsed()
    }

    /// The body cell of a headerless (`from_parsed`) document, which is
    /// seeded by construction.
    fn seeded_body(&self) -> &QuireDocument {
        self.body
            .get()
            .expect("a headerless LoadedDocument is always seeded (from_parsed)")
    }
}

/// Manual: equality is over the document's *content* — path, identity,
/// verbatim text, frontmatter — never over body-cache state, so two loads
/// of the same tree compare equal whether or not a body was touched
/// (TC-473 path-sorted determinism, NFR-006).
impl PartialEq for LoadedDocument {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
            && self.id == other.id
            && self.uuid == other.uuid
            && *self.text == *other.text
            && self.frontmatter() == other.frontmatter()
    }
}

/// Manual for the same reason as `PartialEq`: cache state is not part of
/// the document's observable identity.
impl std::fmt::Debug for LoadedDocument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadedDocument")
            .field("path", &self.path)
            .field("id", &self.id)
            .field("uuid", &self.uuid)
            .field("raw", &self.text)
            .finish()
    }
}

/// Result of a [`load_repo`] call: the successes plus non-fatal
/// diagnostics (skipped/unreadable files, missing uuids).
#[derive(Debug, Clone, PartialEq)]
pub struct RepoLoad {
    /// Parsed documents, sorted by path (deterministic).
    pub documents: Vec<LoadedDocument>,
    /// Non-fatal diagnostics gathered during the load.
    pub diagnostics: Vec<Diagnostic>,
}

/// Walk configuration: markdown only, gitignore honored, dotfiles
/// skipped.
///
/// **Membership is type-driven, not filename-driven** (CR-044). There is
/// no skip list: a markdown file with a frontmatter block is a candidate
/// document whatever it is called, and one without a frontmatter block is
/// not a document at all. The previous `skip_names` default of
/// `{README.md, tests.md}` was ported from
/// `filament_parser/loader.py::_DEFAULT_SKIP`, where it meant *"not a
/// graph node"*; in this validation loader it silently became *"not a
/// document"*, which made the engine unable to load the canonical
/// instance of a document type its own module registers.
#[derive(Debug, Clone)]
pub struct WalkOptions {
    /// File extensions (without the dot) to parse. Default `["md"]`.
    pub extensions: Vec<String>,
    /// Honor `.gitignore`/`.ignore` files. Default `true`.
    pub respect_ignore_files: bool,
    /// Walk hidden files/directories (dotfiles). Default `false`.
    pub include_hidden: bool,
}

impl Default for WalkOptions {
    fn default() -> Self {
        Self {
            extensions: vec!["md".to_string()],
            respect_ignore_files: true,
            include_hidden: false,
        }
    }
}

/// Walk `root`, parse every markdown file, return the collection +
/// diagnostics. Uses [`WalkOptions::default`].
pub fn load_repo(root: &Path) -> RepoLoad {
    load_repo_with(root, &WalkOptions::default())
}

/// Walk `root` with explicit options. A `root` that is missing or not
/// a directory yields an empty [`RepoLoad`] plus one warning — never a
/// panic or `Err` (FR-024-AC-7).
pub fn load_repo_with(root: &Path, opts: &WalkOptions) -> RepoLoad {
    if !root.exists() {
        return RepoLoad {
            documents: Vec::new(),
            diagnostics: vec![Diagnostic::SearchPathMissing {
                path: root.to_path_buf(),
            }],
        };
    }
    if !root.is_dir() {
        return RepoLoad {
            documents: Vec::new(),
            diagnostics: vec![Diagnostic::SearchPathNotADirectory {
                path: root.to_path_buf(),
            }],
        };
    }

    // Phase 1 — sequential walk (I/O-bound), collect matching paths.
    let files = discover_files(root, opts);

    // Phase 2 — data-parallel parse. Each task produces an owned
    // `Outcome`; nothing is shared mutably across threads.
    let mut outcomes: Vec<Outcome> = files.par_iter().map(|p| parse_one(p)).collect();

    // Deterministic order, independent of thread scheduling.
    outcomes.sort_by(|a, b| a.path().cmp(b.path()));

    // Drain owned results into documents + diagnostics, single-threaded.
    let mut documents = Vec::new();
    let mut diagnostics = Vec::new();
    for outcome in outcomes {
        match outcome {
            Outcome::Loaded { doc, mut diags } => {
                documents.push(*doc);
                diagnostics.append(&mut diags);
            }
            Outcome::Failed { diag } => diagnostics.push(diag),
            Outcome::NotADocument { diag, .. } => diagnostics.push(diag),
        }
    }

    RepoLoad {
        documents,
        diagnostics,
    }
}

/// Per-file parse result — owned, no shared state. The document is boxed
/// so the common `NotADocument`/`Failed` outcomes don't pay the large
/// variant's size (clippy::large_enum_variant).
enum Outcome {
    Loaded {
        doc: Box<LoadedDocument>,
        diags: Vec<Diagnostic>,
    },
    Failed {
        diag: Diagnostic,
    },
    /// Read fine, but carries no frontmatter block — not a document
    /// (CR-044), surfaced as a non-fatal warning naming the path (CR-048).
    /// Contributes no document; the path is kept so ordering stays
    /// deterministic.
    NotADocument {
        path: PathBuf,
        diag: Diagnostic,
    },
}

impl Outcome {
    fn path(&self) -> &Path {
        match self {
            Outcome::Loaded { doc, .. } => &doc.path,
            Outcome::NotADocument { path, .. } => path,
            Outcome::Failed { diag } => match diag {
                Diagnostic::DocumentUnreadable { path, .. } => path,
                _ => Path::new(""),
            },
        }
    }
}

pub(crate) fn discover_files(root: &Path, opts: &WalkOptions) -> Vec<PathBuf> {
    let mut builder = WalkBuilder::new(root);
    builder
        .hidden(!opts.include_hidden)
        .git_ignore(opts.respect_ignore_files)
        .git_exclude(opts.respect_ignore_files)
        .ignore(opts.respect_ignore_files)
        .parents(opts.respect_ignore_files)
        // Honor .gitignore even when `root` is not itself a git repo
        // (the default `require_git(true)` would otherwise ignore it).
        .require_git(false)
        .follow_links(false); // never follow symlinks → no loop risk (FR-024-AC-5)

    let mut files = Vec::new();
    for entry in builder.build().flatten() {
        let path = entry.path();
        if !entry.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }
        if !has_allowed_extension(path, &opts.extensions) {
            continue;
        }
        files.push(path.to_path_buf());
    }
    files
}

/// The corpus-membership rule, in one place (CR-044): a markdown file is a
/// document iff it carries a frontmatter block.
///
/// Shared with [`glossary_terms_from_path`](crate::glossary_terms_from_path),
/// which scans raw text rather than building a `Spec` but must agree on what
/// counts as a document — otherwise a `README.md` could define a repository's
/// ubiquitous language while not being part of its corpus.
pub(crate) fn is_document(text: &str) -> bool {
    // The borrowing extraction: this decides membership for every file the
    // glossary harvester sees, and the owning variant copied the whole body
    // to answer a yes/no question (CR-046 leftover, removed CR-055).
    crate::parser::frontmatter::extract_frontmatter_ref(text)
        .frontmatter
        .is_some()
}

fn has_allowed_extension(path: &Path, extensions: &[String]) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => extensions.iter().any(|allowed| allowed == ext),
        None => false,
    }
}

fn parse_one(path: &Path) -> Outcome {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            return Outcome::Failed {
                diag: Diagnostic::DocumentUnreadable {
                    path: path.to_path_buf(),
                    reason: e.to_string(),
                },
            }
        }
    };

    // The header tier decides membership AND identity in one frontmatter
    // extraction (CR-046) — no body work is done for a non-document.
    //
    // A markdown file with no frontmatter block is not a document (CR-044).
    // This is the rule that retires the `README.md` filename skip, and it
    // generalizes to every stray `.md` in a repository — CHANGELOG, AGENTS,
    // a design note — without the engine knowing any of their names.
    //
    // Dropped with a **warning** (CR-048, inverting CR-044's silence): the
    // silent drop was justified only by tolerating a walk pointed at a
    // repository root, where README/CHANGELOG are legitimately present and
    // legitimately not documents. With the walk bounded to the document
    // root (CR-045), what remains here is a markdown file someone put in
    // the spec directory that carries no front block — almost certainly an
    // authoring mistake, and silence made it a real error nobody ever saw:
    // absent from the corpus, from index completeness, and from coverage
    // denominators, with nothing saying so. Non-fatal; the file still
    // contributes nothing. Never re-suppressed by filename — the CR-044
    // rule holds. Frontmatter present but naming an unregistered type keeps
    // today's behavior — error under `Strict`, warning under `Okf`.
    // `Malformed` (a complete fence block that is not a YAML mapping) is the
    // sharper finding; absent/unterminated blocks read as a misplaced or draft
    // file (FR-006 status classification). The status comes from the SAME
    // extraction that decided membership — recovering it used to cost a
    // second, copying `extract_frontmatter` over the whole text (CR-055).
    let (header, status) = crate::parser::parse_header_status(&text);
    let Some(header) = header else {
        let malformed = status == crate::parser::FrontmatterStatus::Malformed;
        return Outcome::NotADocument {
            path: path.to_path_buf(),
            diag: Diagnostic::DocumentWithoutFrontmatter {
                path: path.to_path_buf(),
                malformed,
            },
        };
    };

    // Identity comes from the header — read, never derived (CR-002).
    // NO body parse happens in the walk (CR-047): the body tier is
    // materialised lazily on first `body()` access, so the rayon region
    // stays a data-parallel collect of owned header-tier results.
    let mut diags = Vec::new();
    if header.uuid.is_none() {
        diags.push(Diagnostic::MissingUuid {
            path: path.to_path_buf(),
        });
    }

    Outcome::Loaded {
        doc: Box::new(LoadedDocument {
            path: path.to_path_buf(),
            id: header.id.clone(),
            uuid: header.uuid,
            text: text.into(),
            header: Some(header),
            body: LazyBody::empty(),
        }),
        diags,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    use crate::parser::parse_document;

    fn tmpdir(tag: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "quire_load_repo_{tag}_{}",
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

    const FR_023: &str = "---\nid: FR-023\ntype: FR\nuuid: 0190b6a0-0000-7000-8000-000000000023\n---\n# Behavior\nbody\n";
    const STR_005: &str = "---\nid: StR-005\ntype: StR\nuuid: 0190b6a0-0000-7000-8000-000000000005\n---\n# Need\nbody\n";
    const TM_001: &str = "---\nid: TM-001\ntype: TestMatrix\nuuid: 0190b6a0-0000-7000-8000-000000000001\n---\n# Test Matrix\n\n## Test Case Summary\n";

    // TC-470: N files -> N docs, each matching a direct parse_document.
    #[test]
    fn loads_one_document_per_markdown_file() {
        let root = tmpdir("n_docs");
        write(&root, "functional/FR-023.md", FR_023);
        write(&root, "stakeholder/StR-005.md", STR_005);

        let load = load_repo(&root);

        assert_eq!(load.documents.len(), 2);
        let fr = load.documents.iter().find(|d| d.id == "FR-023").unwrap();
        assert_eq!(fr.body(), &parse_document(FR_023));
        assert_eq!(
            fr.uuid,
            Some(Uuid::parse_str("0190b6a0-0000-7000-8000-000000000023").unwrap())
        );
    }

    // TC-476 / FR-024-AC-7: bad root -> empty + one warning, no panic.
    #[test]
    fn missing_root_yields_empty_with_warning() {
        let load = load_repo(Path::new("/no/such/dir/quire-test"));
        assert!(load.documents.is_empty());
        assert!(matches!(
            load.diagnostics.as_slice(),
            [Diagnostic::SearchPathMissing { .. }]
        ));
    }

    #[test]
    fn file_root_yields_empty_with_warning() {
        let root = tmpdir("file_root");
        write(&root, "FR-001.md", FR_023);
        let load = load_repo(&root.join("FR-001.md"));
        assert!(load.documents.is_empty());
        assert!(matches!(
            load.diagnostics.as_slice(),
            [Diagnostic::SearchPathNotADirectory { .. }]
        ));
    }

    // TC-471 / FR-024-AC-2: one malformed (non-UTF8) file -> N-1 docs + 1 diagnostic.
    #[test]
    fn malformed_file_is_diagnosed_not_fatal() {
        let root = tmpdir("malformed");
        write(&root, "functional/FR-023.md", FR_023);
        fs::write(root.join("functional/bad.md"), [0xff, 0xfe, 0x00, 0x9f]).unwrap();

        let load = load_repo(&root);

        assert_eq!(load.documents.len(), 1);
        assert_eq!(load.documents[0].id, "FR-023");
        assert_eq!(
            load.diagnostics
                .iter()
                .filter(|d| matches!(d, Diagnostic::DocumentUnreadable { .. }))
                .count(),
            1
        );
    }

    // TC-473 / FR-024-AC-4 + NFR-006: path-sorted, byte-identical across runs.
    #[test]
    fn output_is_path_sorted_and_deterministic() {
        let root = tmpdir("sorted");
        write(&root, "z/StR-005.md", STR_005);
        write(&root, "a/FR-023.md", FR_023);

        let first = load_repo(&root);
        let second = load_repo(&root);

        assert_eq!(first, second);
        let paths: Vec<_> = first.documents.iter().map(|d| d.path.clone()).collect();
        let mut sorted = paths.clone();
        sorted.sort();
        assert_eq!(paths, sorted);
    }

    // TC-472 / FR-024-AC-3: .gitignore honored by default; overridable.
    #[test]
    fn gitignore_honored_by_default_and_overridable() {
        let root = tmpdir("gitignore");
        write(&root, ".gitignore", "vendor/\n");
        write(&root, "functional/FR-023.md", FR_023);
        write(&root, "vendor/StR-005.md", STR_005);

        let default = load_repo(&root);
        assert_eq!(default.documents.len(), 1);
        assert_eq!(default.documents[0].id, "FR-023");

        let opts = WalkOptions {
            respect_ignore_files: false,
            ..WalkOptions::default()
        };
        let all = load_repo_with(&root, &opts);
        assert_eq!(all.documents.len(), 2);
    }

    // FR-024 walk semantics (CR-044): membership is decided by the presence of
    // a frontmatter block, never by filename. `tests.md` is the case that
    // motivated the change — it is the canonical filename for `TestMatrix`,
    // a fully registered archetype, and the old skip list made the engine
    // unable to load the canonical instance of a type its own module declares.
    // TC-807, FR-024-AC-10 (CR-048: the CR-044 "produce no diagnostic"
    // assertion is inverted — a frontmatter-less file INSIDE the walked
    // root warns; files outside the root are simply never visited, #91).
    #[test]
    fn tc807_membership_is_type_driven_and_frontmatterless_warns() {
        let scope = tmpdir("membership");
        let root = scope.join("spec");
        write(&root, "tests.md", TM_001);
        // Untyped: frontmatter present, no `type:` key. Still a document —
        // only a *missing frontmatter block* excludes a file, and which types
        // are acceptable is validation's question, not the walk's. In a
        // sibling directory because one directory cannot hold two `tests.md`.
        write(&root, "module/tests.md", "---\nid: TM-002\n---\n# matrix\n");
        write(
            &root,
            "notes.md",
            "---\nid: N-1\ntype: Nonsense\n---\n# note\n",
        );
        // Repo-root strays live OUTSIDE the document root (CR-045): the walk
        // never visits them, so they produce nothing at all.
        write(&scope, "README.md", "# readme\n\nno frontmatter here.\n");
        write(&scope, "CHANGELOG.md", "# changelog\n");
        // A frontmatter-less file INSIDE the document root: almost certainly
        // an authoring mistake — a draft that never got its front block.
        write(&root, "draft.md", "# a draft\n\nno front block yet.\n");
        // A malformed block (complete fences, not a YAML mapping) is the
        // sharper flavor of the same finding.
        write(&root, "broken.md", "---\n- a\n- b\n---\n# broken\n");

        let load = load_repo(&root);

        // Exactly three documents, and no filename decided any of it: a typed
        // `tests.md` is in, an untyped `tests.md` is in, and an unregistered
        // type is in (triaged downstream by validation, not here). The
        // frontmatter-less files contribute nothing to the corpus.
        let ids: Vec<&str> = load.documents.iter().map(|d| d.id.as_str()).collect();
        assert_eq!(ids, vec!["TM-002", "N-1", "TM-001"]);

        // Each in-root frontmatter-less file warns EXACTLY once, naming its
        // path and carrying the right flavor (CR-048).
        let fm_warnings: Vec<(&PathBuf, bool)> = load
            .diagnostics
            .iter()
            .filter_map(|d| match d {
                Diagnostic::DocumentWithoutFrontmatter { path, malformed } => {
                    Some((path, *malformed))
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            fm_warnings.len(),
            2,
            "expected one warning per in-root frontmatter-less file: {fm_warnings:?}"
        );
        assert!(fm_warnings
            .iter()
            .any(|(p, malformed)| p.ends_with("draft.md") && !malformed));
        assert!(fm_warnings
            .iter()
            .any(|(p, malformed)| p.ends_with("broken.md") && *malformed));

        // Files outside the document root are never visited: no diagnostic
        // of any kind mentions them (#91 makes them invisible, not CR-044).
        for name in ["README.md", "CHANGELOG.md"] {
            assert!(
                !load
                    .diagnostics
                    .iter()
                    .any(|d| format!("{d:?}").contains(name)),
                "{name} is outside the document root and should never be visited"
            );
        }
    }

    // TC-475 / FR-024-AC-6: id + uuid read from frontmatter; missing uuid -> diagnostic.
    #[test]
    fn identity_is_read_and_missing_uuid_diagnosed() {
        let root = tmpdir("identity");
        write(
            &root,
            "functional/FR-099.md",
            "---\nid: FR-099\ntype: FR\n---\n# No uuid\n",
        );

        let load = load_repo(&root);

        assert_eq!(load.documents.len(), 1);
        let d = &load.documents[0];
        assert_eq!(d.id, "FR-099");
        assert_eq!(d.uuid, None);
        assert!(load
            .diagnostics
            .iter()
            .any(|x| matches!(x, Diagnostic::MissingUuid { .. })));
    }

    // FR-024-AC-9 guard: result types are owned + Send (data-parallel safe).
    #[test]
    fn repo_load_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<RepoLoad>();
        assert_send_sync::<LoadedDocument>();
    }
}
