# Task 029: Spec Corpus Model

Status: complete — `src/corpus/spec.rs` (6 unit tests). `Arc<SpecInner>` mirrors Registry; by_id (HashMap point-lookup) + by_type (BTreeMap, deterministic); DuplicateArtifactId diagnostic first-wins; Send+Sync + immutability + no-IO guards green.

## Scope

Add the `Spec` corpus — a bounded, in-memory, immutable set of loaded documents indexed by stable artifact id, constructed from a `RepoLoad`. Substrate for resolution (030) and queries (031). A data structure, not a stateful engine.

## Subtasks
- [ ] **Construction.** `Spec::from_repo(RepoLoad)` + `Spec::from_path(&Path)` convenience (calls `load_repo` then `from_repo`).
- [ ] **Id index.** `HashMap<ArtifactId, usize>` (id → slot) built at construction. Id treated as opaque stable string (no grammar imposed).
- [ ] **Duplicate ids.** `Diagnostic::DuplicateArtifactId`; first occurrence wins for lookup; duplicate queryable. Construction does not fail.
- [ ] **Lifecycle.** Immutable after construction; `Send + Sync`; `Arc<Inner>` cheap clone. No add/remove, no watcher, no reload — rebuild to refresh.
- [ ] **Scope guard.** Public surface exposes NO persistence, NO filesystem-watcher registration, NO external (cross-spec) resolution. Enforce via an API-surface test enumerating allowed methods.
- [ ] **Carry diagnostics.** Surface load + (later) resolution diagnostics via `Spec::diagnostics()`.

## Owns
- FR-025

## Dependencies
- 028 (`RepoLoad`, `LoadedDocument`)

## Unblocks
- 030 (resolution)

## Deliverables
- `Spec` type + `from_repo`/`from_path`/`len`/`diagnostics`
- API-surface guard test

## Primary Tests
- TC-480 (len), TC-481 (id index present/absent), TC-482 (dup id), TC-483 (Send+Sync compile assert), TC-484 (scope-guard surface), TC-485 (no-IO queries audit)

## Notes
- Mirror the `Registry` lifecycle contract (immutable, `Arc`, `Send+Sync`) — same pattern, different payload.
- The scope-guard test (TC-484) is the structural enforcement of StR-006-AC-4; keep the allowed-method list explicit so it fails loudly if someone adds persistence later.
