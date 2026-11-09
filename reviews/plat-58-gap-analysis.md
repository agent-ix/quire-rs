---
id: SR-20261103118134
title: "Gap analysis — PLAT-58 entry-error reasons by locus (quire-rs PR 619)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; spec/functional/FR-076-relationships-extraction.md (FR-076-AC-22), spec/functional/FR-071-clause-and-operation-extraction.md, spec/tests.md (TC-2150), tests/it/semantic_relations.rs, tests/it/semantic_clauses.rs, crates/quire-fixtures/fixtures/mapping/"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-076", type: references}
---

## Summary

Ticket: PLAT-58. PR head: `03e7c99`. Plan completion: not assessed. The PR adds FR-076-AC-22 and TC-2150. `spec/tests.md` carries the audit row `FR-076-AC-22 | TC-2150` and the FR-076 summary row. The test `entry_error_reasons_are_stated_by_locus` carries `#[trace("TC-2150", "FR-076-AC-22")]`.

The engine reason string `entry-errors: lines N` is unchanged, and so are `tests/fixtures/semantic/cases.json` and `cases.expected.json`. Both are consistent with the spec as written. FR-076-AC-22 and the new FR-076 bullet scope the rule to mapping fixture cases, `relationships-cases.json` and `operations-cases.json`. `cases.json` is the FR-072 suite, and `cases.expected.json` is engine output, which must keep the engine string. The FR-076 bullet that defines the engine reason (line 269) and FR-076-AC-14 still name the string form. That is correct, because they describe engine output rather than fixture expectations.

## Verdict

**Two medium gaps.** The relationships half of AC-22 is implemented and has discriminating tests. Two parts are not done. The clauses half is vacuous: no fixture case exercises it. The fixture README, which documents each case field, does not define `reasonLoci`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `operations-cases.json` has no case with clauses or operations `unavailable` through entry errors; its only `availability` is `available`. So the new `reasonLoci` branch in the clauses harness is never reached. The "operations-cases.json" leg of FR-076-AC-22 and TC-2150 passes vacuously, and the new FR-071 sentence ("A fixture case states those loci as `reasonLoci`") describes no existing case. The ticket's original case was `fence-quire-invalid`, which records reason `fence`; its fixture was deleted with the 0.1.0 clause-language branch. An `operations-cases.json` case with `reasonLoci: ["fence"]` would make the branch and the claim real | tests/it/semantic_clauses.rs:244-264, crates/quire-fixtures/fixtures/mapping/operations-cases.json, spec/functional/FR-071-clause-and-operation-extraction.md:123 |
| FND-002 | medium | The fixture README's field table for `relationships-cases.json` defines `availability` but not `reasonLoci`, and nothing defines it for `operations-cases.json` either. Its meaning, ascending order, one entry per distinct line, and exclusion with `reason` are stated only in FR-076, not where the fixture fields are documented | crates/quire-fixtures/fixtures/mapping/README.md:37 |
| FND-003 | low | TC-2150 also states "a reason whose lines differ from the resolved loci fails". That leg is implemented by `reason_lines_differing_from_resolved_loci_fail`, which has no `#[trace]` tag, so the matrix credits FR-076-AC-22 only to the positive test | tests/it/semantic_relations.rs:943-950 |
