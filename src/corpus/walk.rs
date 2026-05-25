//! Parallel repository walk + parse — `load_repo` (FR-024, NFR-015).
//!
//! Walks a directory tree (ignore-file aware), parses every markdown
//! file into a [`QuireDocument`] on a rayon pool, and returns the
//! collection plus per-file diagnostics. Per-file failures are
//! non-fatal.
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
//! scheduling (NFR-006).

use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use rayon::prelude::*;
use uuid::Uuid;

use crate::ast::QuireDocument;
use crate::diagnostic::Diagnostic;
use crate::parser::parse_document;

/// Files excluded by default — documentation / test-matrix files that
/// are not artifacts. Matches `filament_parser/loader.py::_DEFAULT_SKIP`.
const DEFAULT_SKIP: &[&str] = &["README.md", "tests.md"];

/// One parsed document plus its read identity.
#[derive(Debug, Clone, PartialEq)]
pub struct LoadedDocument {
    /// Path on disk (absolute or root-relative as discovered).
    pub path: PathBuf,
    /// Human artifact id from frontmatter `id` (e.g. `"FR-023"`); the
    /// intra-spec resolution key. Empty string when absent.
    pub id: String,
    /// Durable catalog id from frontmatter `uuid` (a UUID7, authored by
    /// quire). `None` when absent or unparseable.
    pub uuid: Option<Uuid>,
    /// The parsed document.
    pub doc: QuireDocument,
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

/// Walk configuration. `Default` matches the Python loader's behavior:
/// markdown only, gitignore honored, dotfiles skipped, and the
/// `{README.md, tests.md}` skip set applied.
#[derive(Debug, Clone)]
pub struct WalkOptions {
    /// File extensions (without the dot) to parse. Default `["md"]`.
    pub extensions: Vec<String>,
    /// Honor `.gitignore`/`.ignore` files. Default `true`.
    pub respect_ignore_files: bool,
    /// Walk hidden files/directories (dotfiles). Default `false`.
    pub include_hidden: bool,
    /// File names to skip. Default `{README.md, tests.md}`.
    pub skip_names: Vec<String>,
}

impl Default for WalkOptions {
    fn default() -> Self {
        Self {
            extensions: vec!["md".to_string()],
            respect_ignore_files: true,
            include_hidden: false,
            skip_names: DEFAULT_SKIP.iter().map(|s| s.to_string()).collect(),
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
                documents.push(doc);
                diagnostics.append(&mut diags);
            }
            Outcome::Failed { diag } => diagnostics.push(diag),
        }
    }

    RepoLoad {
        documents,
        diagnostics,
    }
}

/// Per-file parse result — owned, no shared state.
enum Outcome {
    Loaded {
        doc: LoadedDocument,
        diags: Vec<Diagnostic>,
    },
    Failed {
        diag: Diagnostic,
    },
}

impl Outcome {
    fn path(&self) -> &Path {
        match self {
            Outcome::Loaded { doc, .. } => &doc.path,
            Outcome::Failed { diag } => match diag {
                Diagnostic::DocumentUnreadable { path, .. } => path,
                _ => Path::new(""),
            },
        }
    }
}

fn discover_files(root: &Path, opts: &WalkOptions) -> Vec<PathBuf> {
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
        if is_skipped(path, &opts.skip_names) {
            continue;
        }
        files.push(path.to_path_buf());
    }
    files
}

fn has_allowed_extension(path: &Path, extensions: &[String]) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => extensions.iter().any(|allowed| allowed == ext),
        None => false,
    }
}

fn is_skipped(path: &Path, skip_names: &[String]) -> bool {
    match path.file_name().and_then(|n| n.to_str()) {
        Some(name) => skip_names.iter().any(|s| s == name),
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

    let doc = parse_document(&text);
    let (id, uuid) = read_identity(&doc);

    let mut diags = Vec::new();
    if uuid.is_none() {
        diags.push(Diagnostic::MissingUuid {
            path: path.to_path_buf(),
        });
    }

    Outcome::Loaded {
        doc: LoadedDocument {
            path: path.to_path_buf(),
            id,
            uuid,
            doc,
        },
        diags,
    }
}

/// Read `(id, uuid)` from frontmatter — never derived (CR-002).
fn read_identity(doc: &QuireDocument) -> (String, Option<Uuid>) {
    let fm = match doc.frontmatter.as_ref() {
        Some(fm) => fm,
        None => return (String::new(), None),
    };
    let id = fm
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let uuid = fm
        .get("uuid")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    (id, uuid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

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

    const FR_023: &str = "---\nid: FR-023\nartifact_type: FR\nuuid: 0190b6a0-0000-7000-8000-000000000023\n---\n# Behavior\nbody\n";
    const STR_005: &str = "---\nid: StR-005\nartifact_type: StR\nuuid: 0190b6a0-0000-7000-8000-000000000005\n---\n# Need\nbody\n";

    // TC-470: N files -> N docs, each matching a direct parse_document.
    #[test]
    fn loads_one_document_per_markdown_file() {
        let root = tmpdir("n_docs");
        write(&root, "functional/FR-023.md", FR_023);
        write(&root, "stakeholder/StR-005.md", STR_005);

        let load = load_repo(&root);

        assert_eq!(load.documents.len(), 2);
        let fr = load.documents.iter().find(|d| d.id == "FR-023").unwrap();
        assert_eq!(fr.doc, parse_document(FR_023));
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

    // FR-024 walk semantics: default skip set excludes README.md / tests.md.
    #[test]
    fn default_skip_set_excludes_readme_and_tests() {
        let root = tmpdir("skip");
        write(&root, "functional/FR-023.md", FR_023);
        write(&root, "README.md", "# readme\n");
        write(&root, "tests.md", "# matrix\n");

        let load = load_repo(&root);
        assert_eq!(load.documents.len(), 1);
        assert_eq!(load.documents[0].id, "FR-023");
    }

    // TC-475 / FR-024-AC-6: id + uuid read from frontmatter; missing uuid -> diagnostic.
    #[test]
    fn identity_is_read_and_missing_uuid_diagnosed() {
        let root = tmpdir("identity");
        write(
            &root,
            "functional/FR-099.md",
            "---\nid: FR-099\nartifact_type: FR\n---\n# No uuid\n",
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
