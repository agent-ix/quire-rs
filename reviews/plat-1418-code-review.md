---
id: SR-20261113045750
title: "Code review — PLAT-1418 design-element join engine (quire-rs PR 672)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; src/corpus/relation_edges.rs, src/corpus/relation_report.rs, src/corpus/required_relations.rs, src/corpus/validate.rs, src/corpus/spec.rs, src/corpus/resolve.rs, src/traceability.rs, src/loader/mod.rs, src/lib.rs, src/corpus/mod.rs, CHANGELOG.md, tests/it/design_join.rs, tests/fixtures/traceability/design-join*/manifest.yaml"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-086", type: references}
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-058", type: references}
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-038", type: references}
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-084", type: references}
---

## Summary

Ticket: PLAT-1418. PR head: `30ac10c`. This is a code review with the rust-review lane folded in, scoped to `git diff origin/main...HEAD`. The PR adds the `design_relations` manifest field, `design_join`, relation matching that is aware of inverse labels (FR-058 CR-416), reading an edge to a criterion id as an edge to the document that minted it (CR-417), and dropping `dangling-reference` for such edges (FR-038 CR-418).

## Verdict

**FAIL: one high finding (FND-001) and three low findings.**

- **What the PR gets right.** The engine names no archetype, verb or inverse label (FR-086-AC-13, checked by reading every added `src/` line). The counterpart comes only from `registry.inverse_index()`. Criterion ids come only from trace targets that an `obligations:` source names.
- **FR-084 output is unchanged.** `realized_criteria` is `None` outside the join and skipped when serialized. The existing `relation_report` tests pass unchanged except for one `..` pattern update. For relations whose verbs have no counterpart, readings are exactly the authored edges, so FR-058-AC-24 byte identity holds by construction. Mismatch findings now name the target member's identity instead of `edge.target_repo`, which gives the same result for forward edges. Findings are sorted afterwards.
- **`trace:undeclared-relation-vocabulary` for a design entry is specified.** FR-086 §The declaration requires it. The PR implements it with the error and the finding both naming the entry.
- **Rulings on the coder's flagged ambiguities:**
  - (1) AC-15 is satisfiable as implemented. FR-058 CR-417 does not let the engine know which requirement an unread edge from a design element was meant for, so that edge cannot appear in any requirement row's `dangling` list. It stays in the FR-026 dangling set and draws `dangling-reference`, and the test asserts both. A requirement that authors the inverse toward the id gets it listed in its row's `dangling`, which the test also asserts.
  - (2) The literal reading of CR-416 / FR-058-AC-25 governs, and the code diverges from it. See FND-001.
  - (3) Naming the inverse-authored normalized target by its document id is the only coherent reading of "the target as authored". That end was never authored as a reference. Accepted.
  - (4) Not a defect. `Spec::build` rejects a multi-member corpus without identities (`MissingIdentity`). The single `reference_root` that `validate_bundle` passes only affects exclude-glob matching, which matches the single-root contract `validate_required_relations` already has.
  - (5) The breaking change to `LinkedEntry::Resolved` is acceptable before 1.0. The local quire-cli checkout does not match on `LinkedEntry`. The change still needs the repo's Breaking changelog entry (FND-003).
- **CR numbering.** The PR mints no CR. It cites CR-416/417/418 from the merged #668. quire-rs#671 uses CR-419. There is no collision.
- **Checks run.** `cargo test --test it` filtered to design_join, relation_report, required_relation and cross_repo: 82 passed. Filtered to okf, bundle, dangling, inverse and target_kind: 46 passed. `cargo clippy --all-targets -D warnings` is clean and `cargo fmt --check` is clean. The aggregate gate was not run, as the brief asked.
- **Ceremony.** None found.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | A relation listing both labels of a pair (`[V, W]`) reads every pair in both orientations. `forms` gives `W` its own non-reversed reading plus a reversed one. So `incoming [kindles, kindled_by]` also covers a `from` document that is the source of `kindles`, for example a Glyph authoring `kindles LAN-x`, which `incoming [kindles]` reports uncovered. Under `outgoing` it also covers `kindles` targets. This contradicts FR-058-AC-25 ("accept the same edges and report the same documents") and CR-416 ("count an edge between one pair once however it is authored or listed"). The `glyph-both` test has no edge authored in the opposite orientation, so it cannot fail. Fix: when both labels of a pair are listed, read the pair in one orientation only (as the forward verb), and add a case where a Glyph authors `kindles` toward a Lantern under `glyph-both`. | src/corpus/relation_edges.rs:224-236, tests/it/design_join.rs:344-364 |
| FND-002 | low | Repeated work. `check_relation` and `Context::row` call `edges.matches` and then `is_satisfied`, which calls `matches` again, and each call clones the index vectors. `validate_corpus` rebuilds `RelationEdges` over the whole edge set once per relation per member view. Satisfaction could be derived from the one `matches` result. | src/corpus/required_relations.rs:266-311, src/corpus/relation_report.rs:570-654, src/corpus/relation_edges.rs:313-344 |
| FND-003 | low | The CHANGELOG lists the change under Added only. The repo convention (`**Breaking, PLAT-nnnn:**`) is not followed for the new public `realized_criteria` field on `LinkedEntry::Resolved`, which breaks exhaustive patterns and constructors. It is also not followed for the FR-084 behaviour change, where inverse-authored edges now satisfy relations and `incoming` rows now list `dangling`. | CHANGELOG.md:9-21, src/corpus/relation_report.rs:163-170 |
| FND-004 | low | Two `bool` flags encode a two-way choice that the repo's design guidance says to express as an enum: `reversed` in `forms` (`Vec<(&str, bool)>`) and `realized` on `Context`. A named orientation enum would make the `match (&end, reversed)` arms self-describing. | src/corpus/relation_edges.rs:227-272, src/corpus/relation_report.rs:474 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-086 (statement, §The declaration, §The join) | examined | Implemented; join ordering, error and empty case correct |
| FR-086-AC-13 | examined | Clean by inspection |
| FR-086-CON-2 | examined | Clean; only `realized_criteria` added |
| FR-058 §Authored inverse verbs (CR-416) | examined | FND-001 |
| FR-058 §Edges to a criterion (CR-417) | examined | Clean; owner uniqueness, document-id precedence, member choice mirror `resolve` |
| FR-038 CR-418 / FR-038-AC-12 | examined | Clean |
| FR-084 serialization | examined | Byte-identical outside the join |
| src/corpus/relation_edges.rs | examined | FND-001, FND-002, FND-004 |
| src/corpus/relation_report.rs | examined | FND-004 |
| src/corpus/required_relations.rs | examined | FND-002 |
| src/corpus/validate.rs, spec.rs, resolve.rs, traceability.rs, loader/mod.rs | examined | Clean |
| CHANGELOG.md | examined | FND-003 |
