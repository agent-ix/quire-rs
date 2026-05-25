---
id: FR-024
title: "Parallel Repository Walk + Parse (load_repo)"
artifact_type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-005"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-005"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-025"
    type: "requires"
    cardinality: "1:1"
---

## Behavior

`quire-rs` SHALL provide `load_repo`, a function that walks a directory tree, parses every markdown file it finds into a `QuireDocument` (FR-005), and returns the collection together with per-file diagnostics. This is the Rust home for the work currently done sequentially in `filament_parser/loader.py`.

### Public API

```rust
pub struct LoadedDocument {
    pub path: PathBuf,
    pub id: String,            // human artifact id from frontmatter `id` (resolution key)
    pub uuid: Option<Uuid>,    // durable catalog id from frontmatter `uuid` (UUID7); None if absent
    pub doc: QuireDocument,
}

pub struct RepoLoad {
    pub documents: Vec<LoadedDocument>,
    pub diagnostics: Vec<Diagnostic>,   // per-file parse failures, skipped paths
}

impl RepoLoad {
    pub fn load_repo(root: &Path) -> RepoLoad;
    pub fn load_repo_with(root: &Path, opts: WalkOptions) -> RepoLoad;
}
```

### Walk semantics

- The walk SHALL be **ignore-file aware**: `.gitignore` and `.ignore` entries under the root are honored by default (via the `ignore` crate), so vendored/build directories are skipped. `WalkOptions` MAY disable this.
- Only files matching the markdown extension set (`.md` by default; configurable via `WalkOptions`) are parsed. Non-markdown files are skipped silently.
- A default **skip set** of `{README.md, tests.md}` is excluded (matching `filament_parser/loader.py`'s `_DEFAULT_SKIP`), overridable via `WalkOptions`. These are documentation/test-matrix files, not artifacts.
- Hidden files/directories (dotfiles) are skipped by default; configurable.
- Symlink loops SHALL be broken via a visited-canonical-path set (same guarantee as FR-013); a cycle emits a warning diagnostic and the branch is skipped.

### Parallelism

- File parsing SHALL run on a **rayon** parallel iterator over the discovered file list, so the CPU-bound parse scales with available cores (NFR-015).
- The directory walk itself MAY be sequential (it is I/O-bound and cheap relative to parsing); the parse fan-out is where parallelism applies.
- `load_repo` SHALL be deterministic in its **output ordering**: `documents` is sorted by path so the result is reproducible regardless of thread scheduling (NFR-006).
- **No shared mutable state.** The parallel parse SHALL be *data-parallel*: each task parses one file into an owned result with no mutation of shared state; results are collected (e.g. `par_iter().map(...).collect()`), not pushed into a shared mutable buffer. This is the invariant that keeps the engine free of hand-written synchronization (no `Mutex`/`RwLock`/atomics in first-party code) and is what makes the loom/shuttle skip valid — verified by NFR-017 (loom permutation) + a `Send + Sync`-bound audit, not merely assumed.

### Failure model

- A file that fails to parse SHALL NOT abort the call. The failure is recorded as a `Diagnostic` (path + reason) and the remaining files load. `RepoLoad.documents` contains the successes; `RepoLoad.diagnostics` contains the failures and skips (US-011-AC-2).
- A `root` that does not exist or is not a directory returns an empty `RepoLoad` with a single warning diagnostic — not a panic, not an `Err`.

### Identity (read, never derived) — CR-002

`quire-rs` reads identity from frontmatter; it does NOT derive ids from path or content, and it does NOT write ids back into files during a load. Going forward, quire authors a durable `uuid` into every document's frontmatter (see new-doc creation, FR-001 / §8), so the loader simply reads it.

- `LoadedDocument.id` — the **human artifact id** from frontmatter `id` (e.g. `"FR-023"`). This is the intra-spec resolution key (FR-026/027): `ix://` link targets and `relationships` reference it. If frontmatter has no `id`, the document is keyed by its `uuid` string and a `Diagnostic::UntypedArtifact` is emitted.
- `LoadedDocument.uuid` — the **durable catalog id** from frontmatter `uuid`, a UUID7 (time-ordered, move/rename-stable). Carried for downstream/cross-spec use (the service layer's global identity). A document lacking a `uuid` produces a non-fatal `Diagnostic::MissingUuid` (it is NOT synthesized at load time and the file is NOT mutated).

> **CR-002 note:** This replaces the original FR-024 recipe ("content-derived SHA-256 → UUID5, matching `loader.py` `_content_hash`/`_synthetic_uuid`"), which was factually wrong — the Python loader used a *path-based* `uuid5(custom-ns, "id://"|"path://")`, and path/content-derived ids break on move/edit. The durable-`uuid`-in-frontmatter model (UUID7, authored by quire) was adopted instead. See spec.md §2.2 (ID-generation scope relaxed) and the project decision record.

## Acceptance

- **FR-024-AC-1**: `load_repo` over a fixture tree with N markdown files returns N `LoadedDocument`s, each with a parsed `QuireDocument` matching a direct `parse_document` of that file.
- **FR-024-AC-2**: A fixture tree containing one malformed file returns the N-1 good documents plus exactly one diagnostic naming the bad file; the call does not error or panic (US-011-AC-2).
- **FR-024-AC-3**: A `.gitignore` entry excluding a subdirectory causes files under it to be skipped by default; with `WalkOptions` disabling ignore-files, they are parsed.
- **FR-024-AC-4**: `documents` is sorted by path; two runs over the same tree produce byte-identical ordering and content (NFR-006).
- **FR-024-AC-5**: A symlink loop inside the tree completes with a warning diagnostic and no infinite walk (parity with FR-013-AC-7).
- **FR-024-AC-6**: A document's `LoadedDocument.id` equals its frontmatter `id` (empty string when absent), and `LoadedDocument.uuid` equals its frontmatter `uuid` parsed as a `Uuid` (`None` when absent or unparseable); neither is derived from path or content, and no file is written during load. A document missing `uuid` emits a non-fatal `Diagnostic::MissingUuid { path }`. (Corpus-level id/type keying diagnostics — duplicate id, untyped — are FR-025/FR-027 concerns, not the walk's.)
- **FR-024-AC-7**: A `root` that points to a regular file or a nonexistent path returns an empty `RepoLoad` with one warning diagnostic (no error, no panic).
- **FR-024-AC-8**: A criterion bench measures `load_repo` over a 1,000-document corpus on 1 and 8 threads and records the speedup (feeds NFR-015).
- **FR-024-AC-9**: A static audit (`rg` for `Mutex`/`RwLock`/`Atomic*` in first-party `src/`) confirms the parallel parse uses no hand-written shared-mutable synchronization; the implementation collects owned results rather than mutating a shared buffer (the invariant underpinning NFR-017 and the loom/shuttle skip).
