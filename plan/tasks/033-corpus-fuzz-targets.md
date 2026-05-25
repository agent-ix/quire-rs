# Task 033: Corpus Fuzz Targets (NFR-011 extension)

Status: complete (code) — `fuzz/fuzz_targets/fuzz_load_repo.rs` + `fuzz_resolution.rs` added to `fuzz/Cargo.toml`; nightly `cargo check` passes against the API. Running needs `cargo-fuzz` on the scheduled lane (not installed locally).

## Scope

Extend the hardening suite (task 027) with `cargo-fuzz` targets for the new untrusted-input surfaces introduced by v0.3: the `load_repo` walk and reference resolution. These paths consume attacker-controllable on-disk trees, frontmatter, and `ix://` links and were not covered by the existing parse/extract/manifest fuzz targets.

## Subtasks
- [ ] **load_repo fuzz.** Target that feeds malformed file contents + adversarial frontmatter through the parse leg of `load_repo` (single-file harness; the walk itself is filesystem I/O, fuzz the per-file parse + id derivation). Assert no panic, bounded memory.
- [ ] **resolution fuzz.** Target that builds a small in-memory corpus from fuzzed documents (varied `relationships` arrays + `ix://` link bodies) and runs resolution. Assert no panic, O(edges) does not blow up, dedup stable.
- [ ] **CI wiring.** Add both targets to the existing weekly + workflow_dispatch + tag-push fuzz lane (not per-PR). Update the fuzz target inventory in NFR-011 / task 027 notes.
- [ ] **Regression seeds.** Any discovered crash → committed reproducer under `fuzz/corpus/` + a regression unit test (mirror NFR-011-AC-4 / TC-352 pattern).

## Owns
- NFR-011 extension (new fuzz targets only; the lane infrastructure already exists)

## Dependencies
- 028 (`load_repo`), 030 (resolution)

## Unblocks
- (hardening completeness for v0.3 — no downstream task)

## Deliverables
- `fuzz/fuzz_targets/load_repo.rs`, `fuzz/fuzz_targets/resolve_refs.rs`; CI lane update; inventory note

## Primary Tests
- Covered under the NFR-011 acceptance pattern (fuzz targets compile + run clean ≥60s; crashes → regression seed). No new TC numbers — extends the TC-350..352 family.

## Notes
- Parallel-ready once 028 + 030 land; independent agent. Runs on the existing scheduled lane, so it adds no PR latency.
- Don't fuzz the filesystem walk itself (rayon + `ignore` are upstream-tested); fuzz the bytes-in paths we own.
