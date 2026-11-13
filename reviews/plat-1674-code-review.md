---
id: SR-20261113073338
title: "Code review — PLAT-1674 when_any edge-condition selector (quire-rs PR 667)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs@18c3bb9; src/traceability.rs (RequiredRelation.when_any, EdgeCondition), src/corpus/required_relations.rs (in_scope_by_edge, check_declaration_is_live), tests/it/required_relations_when_any.rs, tests/it/required_relations.rs; consumer read: src/assurance.rs, src/loader/mod.rs"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-058", type: references}
---

## Summary

Ticket: PLAT-1674. PR head `18c3bb9`, diff `origin/main...HEAD` (2 commits). Code review with the rust-review lane folded in (rust-style loaded; quire-rs CLAUDE.md/AGENTS.md take priority). The change adds an optional `when_any` list of `{verb, direction, other}` conditions to a required relation, parsed and validated in `RawRequiredRelation -> RequiredRelation`, applied in `check_relation` after `where` and before `exclude`, and liveness-checked for `other` kinds under `trace:undeclared-relation-vocabulary`.

## Verdict

**FAIL (one high, one medium, one low).**

- Bundle validation matches FR-058's new section and safety FR-302: in scope when any condition holds over a resolved edge, dangling and wrong-kind edges hold nothing, intersection with `where`/`exclude` in the stated order, load-time rejection naming the relation and `when_any`, `undeclared-relation-vocabulary` for a dead `other` kind.
- The engine holds no verb, direction or kind (FR-058-CON-4), confirmed by inspection.
- `deny_unknown_fields` on `RawRequiredRelation` is satisfied by the new `when_any` field. `when_any: null` is kept as `Some(Null)` and rejected. The loader merge clones the whole relation, so CON-1 holds.
- The high finding is the second consumer of required relations, the FR-068 assurance projection. It ignores `when_any` (and `where`, a pre-existing gap), so it exports `missing` for documents bundle validation says are out of scope.
- Checks run: `cargo test --test it required_relations` gave 29 passed, 0 failed. `cargo fmt --check` is clean. Clippy and the full gate were not run (the lead runs them once).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The assurance projection builds required-relation subjects from `from`, model `exclude` and relation `exclude` only. It never applies `where` (pre-existing) or the new `when_any`. For a `when_any` relation, every out-of-scope `from` document is exported as a `missing` observation, and a relation with no in-scope subject emits per-document `missing` rows instead of one `not_applicable` row. That contradicts FR-068 Behavior: missing is for an applicable subject, a declaration with no applicable subject is `not_applicable`, and exclusions apply exactly as they do in bundle validation. Fix in this PR: extract one `applicable_subjects` selection (from, where, when_any, exclude) shared by `check_relation` and `project_relation_observations`, which also closes the `where` gap, and add an FR-068 test with a `when_any` relation. | src/assurance.rs:631-640, src/corpus/required_relations.rs:262-281 |
| FND-002 | medium | `EdgeCondition` has public fields and reuses the three-variant `RelationDirection`, so code can build an `Either` condition, a blank verb or an empty `other` that the parser would reject. `TraceabilityModel::validate` does not check `when_any`. `in_scope_by_edge` then matches `_ =>` and silently treats `Either` as outgoing. rust-style: match closed enums exhaustively and do not map a variant to a valid but wrong value. `WhereSelector` keeps its fields private for the same reason. Fix: a two-variant direction type (or private fields with accessors) and an exhaustive match. | src/traceability.rs:579-588, src/corpus/required_relations.rs:411-420 |
| FND-003 | low | The dead-`other` message joins kinds without deduplicating, so a misspelt kind named in two conditions is listed twice (`'Hazz', 'Hazz'`). Collect into a `BTreeSet`. | src/corpus/required_relations.rs:151-156 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| src/traceability.rs EdgeCondition::parse_list | examined | Rejects every AC-26 shape; the error names the relation and `when_any`; FND-002 |
| src/traceability.rs RawRequiredRelation/TryFrom | examined | Correct; null is kept and rejected |
| src/corpus/required_relations.rs check_relation selection order | examined | from, then where, then when_any, then exclude, as FR-058 states |
| src/corpus/required_relations.rs in_scope_by_edge | examined | Resolved-only, kind-checked; FND-002 wildcard |
| src/corpus/required_relations.rs check_declaration_is_live | examined | Dead `other` reported; FND-003 |
| src/assurance.rs project_relation_observations | examined | FND-001 |
| src/loader/mod.rs merge | context_only | Clones the whole relation; CON-1 holds |
| tests/it/required_relations_when_any.rs | examined | See gap-analysis SR for AC binding defects |
