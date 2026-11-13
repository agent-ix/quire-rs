---
id: SR-20261113104840
title: "Code review — PLAT-1587 design_matrix engine op (quire-rs PR 674)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; src/corpus/design_matrix.rs, src/corpus/mod.rs, src/coverage.rs, src/lib.rs, CHANGELOG.md, tests/it/design_matrix.rs, tests/it/main.rs"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-092", type: references}
---

## Summary

Ticket: PLAT-1587. PR head: `e0d4c6f`. Code review with the rust-review lane, scoped to `git diff origin/main...HEAD`, checked against the merged FR-092 text. Focused checks run: `cargo test --test it design_matrix` (15 passed), `cargo clippy --all-targets -D warnings` (clean), `cargo fmt --check` (clean). The aggregate gate was not run.

## Verdict

**CONDITIONAL: no high or medium code defect, two low findings.** The engine side is correct against FR-092:

- **Key order.** `DesignMatrixEntry` declares `document, repo, id, criticality, carried, criteria, design`, with every optional key `skip_serializing_if`. This is the FR-092 order.
- **`carried`.** It is always `None` and typed `Option<serde_json::Map<String, Value>>`, which matches FR-090's "object (any string-keyed values)". Skipped when `None`, it adds no key, so when FR-090 lands the entry shape stays right. This is the minimal sound placeholder (see FND-001 for how to make the hand-off fail loudly).
- **Criteria-less entries.** Obliged documents with no matrix entry get `criteria: []`, `id` from the FR-086 row, and `criticality` read from the document through `document_criticality`, which is now `pub(crate)`.
- **Dedupe and ordering.** `design_elements` is deduped by `(repo, id)` in block then element order. Entries are sorted by `(repo, document)` with `None` first; the sort is stable and keys are unique per member. Dangling `linked` entries are dropped from `elements`.
- **Errors.** An unknown `design_relations` entry propagates `RelationReportError::UnknownDesignRelation` from the single `design_join` call, and no matrix is returned.
- **No mutation and no domain names.** `coverage` and `coverage_matrix` are untouched; the only edit to `coverage.rs` is the visibility of `document_criticality`. The engine file holds no archetype, verb or verdict name: its only code string literal is `"Option::is_none"`, 9 times.
- **Determinism.** Only `BTreeMap`/`BTreeSet` are used, with no `HashMap` and no unwrap.
- **CHANGELOG.** The entry is accurate.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | `entry_of` builds the entry from named fields of `CoverageMatrixRequirement` and hard-codes `carried: None`. When PLAT-1578 adds `carried` to `CoverageMatrixRequirement`, this still compiles and silently drops the value. FR-092-AC-5 byte identity then breaks for any module that declares `carried_fields`. Criteria-less entries will also need `carried` read from the document, as `criticality` is read. Destructure exhaustively (`let CoverageMatrixRequirement { document, criticality, criteria } = requirement;`) so that the FR-090 field turns into a compile error here. | src/corpus/design_matrix.rs:205, 214-238 |
| FND-002 | low | Two maps share one key (`ids` and `blocks`). Each row clones `repo` and `path` for the key, plus `id`, even though the `RowDocument` is owned and is then discarded by `from_row`. The orphan loop also clones `key` again. A single `BTreeMap<key, (String, Vec<DesignBlock>)>` filled by moving `row.document` fields out of the owned row removes the twin lookups and the clones. | src/corpus/design_matrix.rs:165-193 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | baf9888: `entry_of` now destructures `CoverageMatrixRequirement { document, criticality, criteria }` exhaustively, so the FR-090 `carried` field is a compile error there until handled. The criteria-less path has a comment marking where `carried` is read from the document. |
| FND-002 | fixed | baf9888: a single `obliged: BTreeMap<(repo, path), (id, Vec<DesignBlock>)>` replaces the twin maps. The matrix loop does one `remove`, and the orphan loop destructures the key with no clone. Each row still clones three short strings because `from_row` consumes the row; that is negligible. |
