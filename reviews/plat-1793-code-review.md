---
id: SR-320
title: "code, Rust and gap review of the PLAT-1793 source walk implementation (PR #710)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; src/symbols/mod.rs, src/symbol_table.rs, src/symbols/trace.rs, src/symbols/shell_trace.rs, src/coverage.rs, src/coverage/reconcile.rs, schemas/output/coverage-v1.schema.json, tests/it/source_walk.rs, tests/fixtures/corpus_cases/plat-1793-*.json @ d1a2a45295c29008594e0f29b2a84e66037ecbe4"
review_set: subset
---

## Summary

Ticket: PLAT-1793. This is one review of `git diff origin/main...HEAD` (14 files, +1216/-59) at head
d1a2a45. It covers code review, a Rust review lane and gap analysis against FR-050-AC-69 and
FR-051-AC-54..57 as merged in #708 (9e335a4).

The implementation matches the merged Source walk rules. Rule order is template prune, then
hidden, then repository ignore, then `source_exclude`, then the symlink report. It uses
`standard_filters(false)` with `ignore`, `git_ignore` and `require_git(false)`, and never follows
links. `ignored_source_files` comes from the difference between two walks, is omitted when it
is 0, and is admitted by the schema. Symbol-table attribution is now keyed per declaration by
`(id, line, column)`.

Focused tests: `cargo test --test it -- source_walk tc856… tc859…` gave 10 passed and 0 failed.
`quire matrix` binds all five new criteria to tests.

Probes (a scratch crate against this head):

- A nested `.git` does not change rule-3 results. The suspected defect is refuted.
- A differently named symlink to the document root is silently pruned (FND-001).

The public `ImplementsRelation` change adds two fields. It has an in-crate consumer
(`symbol_table::build`). quire-cli origin/main only reads its fields (`implements_record`,
`src/commands/trace.rs:584`) and quoin builds no literal of it, so there is no cross-repo break.

## Scope examined

- FR-051 Source walk rules 1-5: examined
- FR-051-AC-54, AC-55, AC-56, AC-57: examined
- FR-050-AC-69: examined
- `ImplementsRelation` public API and downstream consumers (quire-cli, quoin): examined
- CR-423 note, FR-050-AC-24, FR-051-AC-31: context_only

## Findings

| ID      | Severity | Summary | Refs |
| ------- | -------- | ------- | ---- |
| FND-001 | low      | A symlink to the document root under another name (`docs -> spec`) is pruned silently instead of reported; the prune canonicalizes the link's target | src/symbols/mod.rs:444 |
| FND-002 | medium   | AC-54's target-presence clause is unasserted: no paired run compares a tree with a symlink target present and absent, or before and after deleting an in-root target | tests/it/source_walk.rs:73 |
| FND-003 | medium   | The AC-57 discovery-order test does not control discovery order: creation order does not fix readdir order, so removing `files.sort()` is not shown to fail it | tests/it/source_walk.rs:202 |
| FND-004 | low      | AC-55 "identically with and without a `.git` directory" is asserted only without `.git`; no paired with-`.git` run of the same ignore case | tests/it/source_walk.rs:73 |
| FND-005 | low      | FR-050-AC-69 "counted in at most one" has no case where one file is both gitignored and matched by `source_exclude` | tests/it/source_walk.rs:446 |
| FND-006 | low      | The `extract_tree` doc comment still says "gitignore-aware like the corpus loader"; the walk now deliberately differs from the corpus walk (CR-423 note) | src/symbols/mod.rs:467 |

## Verdict

CHANGES. There are no high findings, and the code is correct on every path I traced. Two
medium findings are test-evidence gaps. The ticket requires mutation sensitivity for critical
checks, and AC-54's target-independence clause and AC-57's discovery-order clause have none.

Fix scope:

- Add paired present/absent symlink-target cases and a deleted-in-root-target case (FND-002).
- Make the order test actually permute discovery, or show that removing the sort fails it
  (FND-003).
- Report or prune `docs -> spec` according to link metadata, with a case for it (FND-001).
- Add a with-`.git` twin of the ignore case (FND-004) and a gitignored-plus-excluded case
  (FND-005).
- Correct the doc comment (FND-006).

No API or cross-repo change is required.
