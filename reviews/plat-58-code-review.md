---
id: SR-20261103118133
title: "Code review (Rust lane) — PLAT-58 entry-error reasons by locus (quire-rs PR 619)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; tests/it/semantic_relations.rs, tests/it/semantic_clauses.rs, crates/quire-fixtures/fixtures/mapping/relationships-cases.json, spec/functional/FR-076-relationships-extraction.md, spec/functional/FR-071-clause-and-operation-extraction.md, spec/tests.md"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-076", type: references}
---

## Summary

Ticket: PLAT-58. PR head: `03e7c99` (one commit over its merge base). Code review with the Rust lane (rust-review, rust-style), scoped to `git diff origin/main...HEAD`.

Tests the reviewer ran at head through `locked-build.sh` with `CARGO_PROFILE_DEV_DEBUG=0`: `cargo test --test it semantic_relations` passed 21 of 21, and `cargo test --test it semantic_clauses` passed 9 of 9. `quire validate` on FR-071 and FR-076 exited 0.

Mutation check, run in a scratch worktree and not on the branch: one case's `reasonLoci` was changed from `row` to `second-row`, and `two-error-lines` lost `second-row`. Three tests failed. `verb-unknown` failed with "no diagnostic at locus second-row", `two-error-lines` with lines [20, 21] against [20], and TC-2150 failed because it could not find the two-row case. A wrong locus does fail.

No files were copied or vendored. The fixture was edited in place in the `quire-fixtures` crate. No line substitution is left in either harness. TC-2150 and FR-076-AC-22 do not collide with `main` or with open PRs 611, 614, 615, 616 and 617.

## Verdict

**Changes requested (two low, plus the gap-analysis mediums in SR-20261103118134).** The relationships harness is correct. Each locus resolves through the case's diagnostic with the same `locus`. The test asserts that diagnostic's `line` against the engine and checks its shape against the artifact with `locus_holds`. The resolved lines are then compared in order with the lines the engine's reason lists. The negative test is real: it fails if the comparison is skipped. The clauses harness compiles and is correct as written, but no fixture case reaches it (see the gap analysis).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | A comment in `request()` still says the fixture "is not edited in place (PLAT-901: it is quoin's own fixture, moved not owned-for-editing)". This PR edits it in place, and FR-076 now calls it derived and owned by quire-rs. The comment justifies the `operations` backfill with a premise that is no longer true | tests/it/semantic_relations.rs:66-71 |
| FND-002 | low | `assert_reason_loci` in the relationships harness refuses a `reason` next to `reasonLoci`. The clauses harness has no such check, so a clause case carrying both passes and its stale `reason` is never read | tests/it/semantic_clauses.rs:245-264 |
