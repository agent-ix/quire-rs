---
id: SR-322
title: "Integrity analysis of PLAT-1831 outputCatalog producers (quire-rs PR 713)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-rs; spec/functional/FR-073-generic-clause-sets.md (PR 713 diff only, head 5a7ac5dde4020113d16ba5237151e72bb15d9a1c); context src/clauses.rs, schemas/output/clause-diff-v1.schema.json"
review_set: subset
date: 2026-11-23
---

## Summary

Ticket: PLAT-1831. Checked the new `producers` rules and FR-073-AC-7..AC-10 for completeness and consistency with the rest of FR-073 and with src/clauses.rs. The clause-set diff (FR-073-AC-4, `diff_clause_sets`) compares clauses only and never the output catalog, so a `producers` change between versions is invisible. Two smaller gaps concern which validation path AC-9 binds and padded kind strings.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Clause-set diff ignores `outputCatalog`. `diff_clause_sets` keys on clause ids and `ChangedClause` compares `Clause` values, whose `expectedOutputs` hold catalog names only. A version that changes an entry's `producers` (or `kind`/`description`) yields an empty diff, although quoin FR-211 discharge outcomes change. With `producers` the catalog becomes machine-checked, so FR-073 must specify catalog changes in the diff: add an AC that diff reports added, removed and changed `outputCatalog` entries (by name, before/after), and extend `clause-diff-v1` (top-level `additionalProperties: false`) with that member. | spec/functional/FR-073-generic-clause-sets.md:71; src/clauses.rs:418-474; schemas/output/clause-diff-v1.schema.json:13 |
| FND-002 | low | "Loading and `validate` reject" is ambiguous (`ClauseSet::validate` or the `quire validate` CLI), and AC-9 only binds loading, so a `ClauseSet` built in code with `producers: Some(vec![])` is untested. "Names the entry" does not say whether the name is the `outputCatalog` key or the kind. Fix: AC-9 says `ClauseSet::validate` (and therefore loading) rejects these cases and the error names the `outputCatalog` key. | spec/functional/FR-073-generic-clause-sets.md:50; spec/functional/FR-073-generic-clause-sets.md:76 |
| FND-003 | low | Only empty or whitespace-only kinds are rejected, so `" run"` loads, while quoin FR-211 matches retained kinds byte for byte; such an entry never matches and the clause shows `kind-not-expected` at discharge with no load-time error. Fix: reject kinds with leading or trailing whitespace too (add to AC-9), or state that kinds are stored and compared verbatim. | spec/functional/FR-073-generic-clause-sets.md:49-51 |

## Verdict

Request changes for FND-001. The `producers` shape rules themselves are complete and consistent with FR-073's domain-neutral stance and with quoin FR-211; AC-7, AC-8 and AC-10 are consistent and implementable as written.
