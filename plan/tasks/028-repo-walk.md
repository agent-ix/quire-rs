# Task 028: Parallel Repository Walk + Parse (load_repo)

Status: complete — `src/corpus/walk.rs` (9 unit tests), `benches/load_repo.rs`; deps pinned; `make ci`-equivalent green (201 lib tests). Identity is read-from-frontmatter per CR-002 (no derivation), default-skips README.md/tests.md.

## Scope

Add `load_repo` — a parallel, ignore-aware directory walk that parses every `.md` into a `QuireDocument` and returns `RepoLoad { documents, diagnostics }`. This is the Rust home for `filament_parser/loader.py`'s sequential walk, and the foundation the corpus (029) is built on.

## Subtasks
- [ ] **Walk.** `ignore::WalkBuilder` over `root`; honor `.gitignore`/`.ignore` by default, skip dotfiles, markdown-extension filter — all overridable via `WalkOptions`. Reuse the FR-013 symlink-loop visited-set guard.
- [ ] **Parallel parse.** rayon parallel iterator over discovered files calling existing `parse_document`. Walk stays sequential (I/O-bound); parse is the fan-out.
- [ ] **Deterministic output.** Sort `documents` by path so results are reproducible regardless of thread scheduling (NFR-006).
- [ ] **Id derivation.** Prefer frontmatter `id`; else content-derived synthetic (SHA-256 → UUID5), matching `filament_parser/loader.py` `_content_hash`/`_synthetic_uuid` so downstream ids stay stable.
- [ ] **Failure model.** Per-file parse failure → `Diagnostic`, non-fatal. Bad/nonexistent `root` → empty `RepoLoad` + warning (no panic/Err).
- [ ] **Deps + pinning.** Add `ignore`, `rayon`, `sha2`, `uuid` to `Cargo.toml` with tilde/equals pins (NFR-009); run `make deny` and add any new license to the allowlist with rationale (NFR-004).
- [ ] **Throughput bench.** `benches/load_repo.rs`: 1k-doc synthetic corpus at 1 + 8 threads; assert NFR-015 medians + parallel efficiency ≥ 0.6.

## Owns
- FR-024, NFR-015 (+ NFR-009 extension for the four new deps)

## Dependencies
- FR-005 `parse_document` (complete)

## Unblocks
- 029 (Spec corpus), 033 (fuzz)

## Deliverables
- `RepoLoad`, `LoadedDocument`, `WalkOptions` types + `load_repo`/`load_repo_with`
- `benches/load_repo.rs` + baseline
- Pinned deps + deny.toml update

## Primary Tests
- TC-470 (N files→N docs), TC-471 (malformed→diagnostic), TC-472 (gitignore), TC-473 (path-sorted determinism), TC-474 (symlink loop), TC-475 (id derivation), TC-476 (bad root), TC-455 (throughput bench → NFR-015)

## Notes
- Mirror the FR-013 loader's path-handling guarantees (tilde, dedup, symlink loop) where they apply.
- Keep `WalkOptions` minimal — extension set + toggle ignore-files + toggle dotfiles. Don't over-build.
