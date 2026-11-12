---
id: SR-20275
title: "Gap analysis — PLAT-1472 kind-typed cell references (quire-rs PR 650)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; FR-049-AC-10..18, FR-050 document_references kind fields, FR-057-AC-4, FR-057-AC-11; tests/it/trace_reference_kinds.rs; src/traceability.rs; src/corpus/trace_refs.rs"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-049", type: references}
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-057", type: references}
---

## Summary

Ticket: PLAT-1472. PR head: `3e7083b`. Planless gap analysis; plan completion was not assessed. `quire matrix --format tsv` (quire 0.40.0, engine 0.56.0) shows FR-049-AC-10..18 and FR-057-AC-11 `tagged` to tests in `tests/it/trace_reference_kinds.rs`, and FR-057-AC-4 tagged to `tests/it/bundle_validate.rs:486`. The `spec/tests.md` rows TC-2240..2247 are flipped to ✅. `examples/spec_validate.rs` is green (204 documents, 0 failed).

## Verdict

All in-scope ACs are bound, and every test asserts its AC's main clauses. There are two gaps between spec and code: FR-049-AC-15 (exact duplicate kind key) and FR-049-AC-18 (per-table cardinality, zero-row table). Each is untested and failed a reviewer probe. AC-16 is clean: grep finds no kind name in `src/`, and the `kinds` fixture uses neutral names (`Class`, `north`, `south`). AC-17 is clean: TC-2245 asserts interleaved mismatch and dangling findings in id order across repeated runs. FR-050's "coverage computation ignores them" holds: no coverage code reads the new fields.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-049-AC-15's "duplicate kind key" is not implemented for an exact duplicate. A YAML map with `north` twice loads Ok, because yaml_serde keeps one entry in the BTreeMap. TC-2244 asserts only the case-folded duplicate, so the AC is tagged but its literal clause has no test. | spec/functional/FR-049-verification-reference-integrity.md:135, tests/it/trace_reference_kinds.rs:215-218 |
| FND-002 | medium | FR-049-AC-18 ("exactly one ... finding" per table with `column` and no `kind_column`) is satisfied only for a single table that has rows. A zero-row table yields none, and two such tables in one document yield one. TC-2247 covers only one table with rows. | spec/functional/FR-049-verification-reference-integrity.md:133, tests/it/trace_reference_kinds.rs:329-376 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-049-AC-10 | examined | Tagged TC-2240; clean |
| FR-049-AC-11 | examined | Tagged TC-2240; clean |
| FR-049-AC-12 | examined | Tagged TC-2241; clean |
| FR-049-AC-13 | examined | Tagged TC-2242; clean (empty, unknown, missing header, case fold) |
| FR-049-AC-14 | examined | Tagged TC-2243; clean |
| FR-049-AC-15 | examined | FND-001 |
| FR-049-AC-16 | examined | Tagged TC-2245; clean |
| FR-049-AC-17 | examined | Tagged TC-2245; clean |
| FR-049-AC-18 | examined | FND-002 |
| FR-050 document_references kind fields | examined | Clean; serialization unchanged when absent |
| FR-057-AC-4 | context_only | Tagged bundle_validate.rs:486; unaffected |
| FR-057-AC-11 | examined | Tagged TC-2246; clean |
