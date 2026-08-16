---
id: FR-024
title: "Parallel Repository Walk + Parse (load_repo)"
type: FR
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

## Description

`quire-rs` SHALL provide `load_repo`, a function that walks a directory tree, parses every markdown file it finds into a `QuireDocument` ([FR-005](./FR-005-parse-document-api.md)), and returns the collection together with per-file diagnostics. This is the Rust home for the work currently done sequentially in `filament_parser/loader.py`.

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
- Corpus membership SHALL be **type-driven, not filename-driven**: a markdown file carrying a frontmatter block is a candidate document whatever it is named, and a markdown file with **no frontmatter block is not a document** and is dropped silently, with no diagnostic. There is no skip set and no `WalkOptions::skip_names` (CR-044).
- The walk SHALL be bounded by the **document root** the caller supplies: it never ascends above that root and never reads outside it. The document root is the directory that holds authored documents — by ecosystem convention `<repo>/spec` — and is **not the repository root**; consumers derive it from their scope rather than passing the scope through ([FR-050](./FR-050-declarative-coverage-computation.md) states the two-root derivation). A caller whose document root is missing surfaces that as a named condition; falling back to walking a wider tree is how a repository-wide crawl survives unnoticed (CR-045).
- Frontmatter present but naming an unregistered `type` is still a corpus document. Which types are acceptable is a validation question ([FR-025](./FR-025-spec-corpus-model.md), bundle postures), not the walk's.
- Hidden files/directories (dotfiles) are skipped by default; configurable.
- Symlink loops SHALL be broken via a visited-canonical-path set (same guarantee as [FR-013](./FR-013-archetype-loader.md)); a cycle emits a warning diagnostic and the branch is skipped.

### Parallelism

- File parsing SHALL run on a **rayon** parallel iterator over the discovered file list, so the CPU-bound parse scales with available cores ([NFR-015](../non-functional/NFR-015-repo-walk-throughput.md)).
- The directory walk itself MAY be sequential (it is I/O-bound and cheap relative to parsing); the parse fan-out is where parallelism applies.
- `load_repo` SHALL be deterministic in its **output ordering**: `documents` is sorted by path so the result is reproducible regardless of thread scheduling ([NFR-006](../non-functional/NFR-006-determinism.md)).
- **No shared mutable state.** The parallel parse SHALL be *data-parallel*: each task parses one file into an owned result with no mutation of shared state; results are collected (e.g. `par_iter().map(...).collect()`), not pushed into a shared mutable buffer. This is the invariant that keeps the engine free of hand-written synchronization (no `Mutex`/`RwLock`/atomics in first-party code) and is what makes the loom/shuttle skip valid — verified by [NFR-017](../non-functional/NFR-017-concurrency-permutation.md) (loom permutation) + a `Send + Sync`-bound audit, not merely assumed.

### Failure model

- A file that fails to parse SHALL NOT abort the call. The failure is recorded as a `Diagnostic` (path + reason) and the remaining files load. `RepoLoad.documents` contains the successes; `RepoLoad.diagnostics` contains the failures and skips ([US-011-AC-2](../usecase/US-011-python-parses-repo-via-bindings.md)).
- A `root` that does not exist or is not a directory returns an empty `RepoLoad` with a single warning diagnostic — not a panic, not an `Err`.

### Identity (read, never derived) — CR-002

`quire-rs` reads identity from frontmatter; it does NOT derive ids from path or content, and it does NOT write ids back into files during a load. Going forward, quire authors a durable `uuid` into every document's frontmatter (see new-doc creation, [FR-001](./FR-001-render-dispatch.md) / §8), so the loader simply reads it.

- `LoadedDocument.id` — the **human artifact id** from frontmatter `id` (e.g. [FR-023](./FR-023-python-binding-surface.md)). This is the intra-spec resolution key ([FR-026](./FR-026-intra-spec-reference-resolution.md)/027): `ix://` link targets and `relationships` reference it. If frontmatter has no `id`, the document is keyed by its `uuid` string and a `Diagnostic::UntypedArtifact` is emitted.
- `LoadedDocument.uuid` — the **durable catalog id** from frontmatter `uuid`, a UUID7 (time-ordered, move/rename-stable). Carried for downstream/cross-spec use (the service layer's global identity). A document lacking a `uuid` produces a non-fatal `Diagnostic::MissingUuid` (it is NOT synthesized at load time and the file is NOT mutated).

> **CR-002 note:** This replaces the original FR-024 recipe ("content-derived SHA-256 → UUID5, matching `loader.py` `_content_hash`/`_synthetic_uuid`"), which was factually wrong — the Python loader used a *path-based* `uuid5(custom-ns, "id://"|"path://")`, and path/content-derived ids break on move/edit. The durable-`uuid`-in-frontmatter model (UUID7, authored by quire) was adopted instead. See spec.md §2.2 (ID-generation scope relaxed) and the project decision record.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-024-AC-1 | `load_repo` over a fixture tree with N markdown files returns N `LoadedDocument`s, each with a parsed `QuireDocument` matching a direct `parse_document` of that file. | Test |
| FR-024-AC-2 | A fixture tree containing one malformed file returns the N-1 good documents plus exactly one diagnostic naming the bad file; the call does not error or panic ([US-011-AC-2](../usecase/US-011-python-parses-repo-via-bindings.md)). | Test |
| FR-024-AC-3 | A `.gitignore` entry excluding a subdirectory causes files under it to be skipped by default; with `WalkOptions` disabling ignore-files, they are parsed. | Test |
| FR-024-AC-4 | `documents` is sorted by path; two runs over the same tree produce byte-identical ordering and content ([NFR-006](../non-functional/NFR-006-determinism.md)). | Test |
| FR-024-AC-5 | A symlink loop inside the tree completes with a warning diagnostic and no infinite walk (parity with [FR-013-AC-7](./FR-013-archetype-loader.md)). | Test |
| FR-024-AC-6 | A document's `LoadedDocument.id` equals its frontmatter `id` (empty string when absent), and `LoadedDocument.uuid` equals its frontmatter `uuid` parsed as a `Uuid` (`None` when absent or unparseable); neither is derived from path or content, and no file is written during load. Both are read from the header tier (`parse_header`, [FR-005](./FR-005-parse-document-api.md), CR-046) — the guarantee is unchanged, its source is narrowed to the one frontmatter extraction that also decides membership. A document missing `uuid` emits a non-fatal `Diagnostic::MissingUuid { path }`. (Corpus-level id/type keying diagnostics — duplicate id, untyped — are [FR-025](./FR-025-spec-corpus-model.md)/[FR-027](./FR-027-whole-spec-query-api.md) concerns, not the walk's.) | Test |
| FR-024-AC-7 | A `root` that points to a regular file or a nonexistent path returns an empty `RepoLoad` with one warning diagnostic (no error, no panic). | Test |
| FR-024-AC-8 | A criterion bench measures `load_repo` over a 1,000-document corpus on 1 and 8 threads and records the speedup (feeds [NFR-015](../non-functional/NFR-015-repo-walk-throughput.md)). | Test |
| FR-024-AC-9 | The **parallel parse fan-out** uses no shared-mutable synchronization: it is a `par_iter().map().collect()` of owned results, with diagnostics gathered after the parallel region (the invariant underpinning [NFR-017](../non-functional/NFR-017-concurrency-permutation.md) and the loom/shuttle skip). Interior mutability elsewhere in `src/corpus` appears **only** as a named, justified exemption in `scripts/audits/check_no_shared_mutable.sh`, whose pattern now also catches `OnceLock`/`OnceCell` so exemptions are visible, not silent (CR-047). | Inspection |
| FR-024-AC-10 | A tree containing a typed `tests.md`, an untyped `tests.md` in a sibling directory (frontmatter, no `type` key), a `notes.md` declaring an unregistered type, and frontmatter-less `README.md` and `CHANGELOG.md` files loads exactly the first three; the two frontmatter-less files are absent from `documents` **and produce no diagnostic**. No filename participates in the decision. | Test (TC-807) |
| FR-024-AC-11 | `glossary_terms_from_path` applies the same membership rule as the walk: a frontmatter-less file carrying a `## Terms` or `## Ubiquitous Language` heading contributes no project term, while the same content in a document does. | Test (TC-808) |

> **CR-044 note (2026-08-15):** The original walk semantics above declared a
> default skip set of `{README.md, tests.md}`, "matching
> `filament_parser/loader.py`'s `_DEFAULT_SKIP`". Both the rule and its
> justification are withdrawn.
>
> **The constant meant something else upstream.** In `loader.py` it was a
> **graph-ingestion** filter. `filament-parser-lib` commit `1d17b6f` states it
> outright: the listed files "validate via quire as their own archetypes but
> are **not graph nodes**." quire-rs commit `8dc32a5` copied the list into
> `load_repo` — a *validation* loader — and *"not a graph node"* silently became
> *"not a document."*
>
> Two independent confirmations it was a slip, not a decision. The Python list
> carried four names and the Rust one took two, because `index.md`/`log.md` had
> already been promoted to real archetypes — the same correction, already made
> twice. And the premise expired: `tests.md` was a frontmatter-less checklist
> when this was written, whereas `spec-artifacts-process` now ships a
> `TestMatrix` archetype with a frontmatter schema, an `id_pattern` and a
> `body_extraction` contract, and this repo's own `spec/tests.md` opens
> `id: TM-001 / type: TestMatrix`.
>
> **The cost was structural, not cosmetic.** The engine could not load the
> canonical instance of a document type its own module registers. Downstream,
> `spec-artifacts-process` was forced into path binding — three near-identical
> `document:` targets, one per filename the ecosystem happens to invent — and
> `tests/spec_dogfood.rs` hand-rolled a `read_dir` recursion precisely so
> TC-794 could reach `spec/tests.md`. That was the type-driven rule, already
> implemented, in a test, as a workaround for the engine not implementing it.
>
> **The replacement rule is frontmatter presence**, which is what actually
> retires `README.md` and generalizes to every stray `.md` — a CHANGELOG, an
> AGENTS file, a design note — without the engine knowing any of their names.
> It has to be stated explicitly because `validate_document` *errors* on a
> document missing `type`, which is the reason `README.md` needed a name-based
> skip in the first place.
>
> **Measured blast radius. [RAN]** `scripts/classify_matrices.py` over `~/dev`,
> worktrees and `-task<N>` copies deduped: of 184 matrices at a path the
> ecosystem binds, **0 have no frontmatter block**, 170 are typed `TestMatrix`
> and stay, and 14 are mis-typed — 10 declaring `type: index`, which is those
> documents saying they are not matrices. Six of the 14 mint rows today and
> need a one-line frontmatter fix. Against that, **20 real matrices in 9 repos
> become visible for the first time** (12 of them minting rows), in filename
> conventions the enumeration never covered — `spec/test-matrix.md`,
> `spec/test_matrix.md`, `spec/traceability_matrix.md`, `spec/*/matrix/tests.md`.
>
> **Ecosystem-wide, ~100 of 184 matrices fail the current contract once
> visible.** That is the deliverable, not a cost. No suppression is added to
> keep validation green.

> **CR-045 note (2026-08-15):** The walk-bounding clause under *Walk semantics*
> is new. Nothing anywhere declared a document root — `quire coverage`, `fix`,
> and OKF `validate` all handed the repository root to `Spec::from_path`, so
> "every document" meant every `.md` in the repository: `README.md`,
> `AGENTS.md`, `CHANGELOG.md`, `plan/`, `reviews/` and `docs/` were read and
> fully parsed as candidate spec documents. That misrouted traversal produced
> 9,172 `required 'type' is missing` errors across 223 repos, which CR-044
> then silenced at the membership layer — the right membership rule, but also
> the evidence of this bug being discarded. The bound is stated here because
> the walk is where it must hold; the two-root derivation consumers use to
> honor it lives in [FR-050](./FR-050-declarative-coverage-computation.md)
> (agent-ix/quire-rs#91, umbrella #90).

> **CR-046 note (2026-08-15):** AC-6's identity read is narrowed to the
> header tier: `walk::parse_one` calls `parse_header`
> ([FR-005](./FR-005-parse-document-api.md)), which decides membership and
> identity in **one** frontmatter extraction — retiring the walk's own
> `read_identity` and the duplicate `is_document` extraction CR-044 had
> introduced after the full parse. The guarantee (read, never derived; no
> file written) is unchanged (agent-ix/quire-rs#92, umbrella #90).

> **CR-047 note (2026-08-15):** AC-9 is **narrowed to the walk and stated
> that way** — from "no `Mutex`/`RwLock`/`Atomic*` anywhere in first-party
> `src/`" to "the parallel parse fan-out uses no shared-mutable
> synchronization" (agent-ix/quire-rs#93, umbrella #90). The occasion is the
> FR-025 lazy body tier: a per-document once-init cell behind `Arc<SpecInner>`
> is a different mechanism with a different failure surface than the walk —
> the rayon fan-out builds every document with an empty cell and parses no
> body, so the data-parallel-collect invariant this AC protects is untouched.
> The audit is **not loosened until it goes quiet**: its pattern is *widened*
> to also match `OnceLock`/`OnceCell` (which the old pattern missed entirely —
> `declared_tables.rs` had carried compile-once `OnceLock` regexes invisibly),
> and every hit must either fail the build or appear in a **named exemption
> list** (`file|match-substring|why`) stating what is exempt and why. The
> concurrency risk the blanket ban stood in for moves to explicit coverage:
> [NFR-017](../non-functional/NFR-017-concurrency-permutation.md)-AC-4 (loom
> first-touch permutation, TC-815) and the
> [NFR-018](../non-functional/NFR-018-ffi-sanitizer-lanes.md) TSAN lane
> (TC-816).

## Dependencies

- **Upstream**: [StR-005](../stakeholder/StR-005-native-python-bindings.md), [FR-005](./FR-005-parse-document-api.md), [FR-025](./FR-025-spec-corpus-model.md)
- **Downstream**: [FR-025](./FR-025-spec-corpus-model.md), [FR-026](./FR-026-intra-spec-reference-resolution.md), [FR-027](./FR-027-whole-spec-query-api.md)
