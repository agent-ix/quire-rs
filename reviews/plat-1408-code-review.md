---
id: SR-20261112034125
title: "Code review — PLAT-1408 dangling-reference finding located at source document path (quire-rs PR 647)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; src/corpus/validate.rs (validate_bundle reference-resolution loop), tests/it/bundle_validate.rs (dangling_edge_is_located_at_source_document_path), CHANGELOG.md, spec/tests.md; context: src/corpus/spec.rs (artifact_key, Spec::from_repo, by_id), src/corpus/resolve.rs (edge harvest/dedup)"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-038", type: references}
---

## Summary

Ticket: PLAT-1408. PR head: `96d1ddf`. Code review with the rust-review lane folded in (rust-style loaded; repo AGENTS.md/CLAUDE.md idiom rules applied). The change builds a `BTreeMap<artifact_key, &Path>` over `spec.inner.documents` with first-occurrence-wins (`entry().or_insert`) and uses it to set `BundleFinding.path` of every `dangling-reference` finding to the source document's loaded path instead of `PathBuf::from(edge.source)`.

## Verdict

**PASS with low findings.** The behaviour is correct: every edge source is `artifact_key(doc)` of a loaded document (resolve.rs:114), documents are in `load_repo` path order, and `or_insert` keeps the first occurrence, matching FR-025-AC-3's index. Tiering through `report.route` changes only `severity` and the target list, so FR-057 overrides leave path and message untouched by construction. The new test fails on the old code (path `A-1` does not end in `a.md`) and would fail under a last-wins map (`z.md`). `cargo fmt --check` clean; `cargo clippy --lib --tests -D warnings` clean; `cargo test --test it bundle_validate` 19 passed, 0 failed. No ceremony introduced.

Out-of-diff observation (not a finding of this PR): `disallowed-edge-target` (src/corpus/validate.rs:472) still reports `PathBuf::from(&edge.source)`, the same id-as-path shape AC-11 fixes for dangling references; it is governed by FR-040/FR-041, not AC-11.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | A second first-wins id→document index is rebuilt (one String allocation per document) although `Spec::by_id` already answers it with the same FR-025-AC-3 semantics; two sources of truth for "which document owns this key" | src/corpus/validate.rs:316-319 |
| FND-002 | low | The `map_or_else` fallback to `PathBuf::from(&edge.source)` is unreachable (every edge source is a loaded document's key) and, if the invariant ever broke, would silently reproduce the exact id-as-path defect AC-11 forbids instead of surfacing it | src/corpus/validate.rs:339-341 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| src/corpus/validate.rs (validate_bundle refs loop) | examined | Correct; FND-001, FND-002 |
| tests/it/bundle_validate.rs (TC-2227 test) | examined | Assertions can fail; distinguishes old code and last-wins |
| CHANGELOG.md | examined | Accurate |
| spec/tests.md (TC-2227 row, FR-038 rollup) | examined | Covered by gap-analysis review |
| src/corpus/spec.rs (artifact_key, from_repo, by_id) | context_only | First-wins index confirmed |
| src/corpus/resolve.rs (edge harvest, dedup by source/target/type) | context_only | Edge source = artifact_key(doc) |
