---
id: SR-20261117110652
title: "Gap analysis — PLAT-1769 flow-rule schema refs (PR #700)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs@e0024eca; FR-096-AC-17, FR-096-AC-26..28; src/loader/flow_rules.rs, src/semantic/resolver.rs, src/corpus/flow_rules.rs, tests/it/flow_rules.rs"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-rs/FR-096", type: references }
---

## Summary

Ticket: PLAT-1769. This targeted audit checks FR-096-AC-17 and AC-26–28 across spec, computed matrix, tagged tests, and implementation. `quire matrix --scope . --format json` reports all four criteria tagged. The computed binders are present, but the AC-17 binder for referenced enums does not assert the flow report vocabulary; the report still derives vocabularies from inline-only enum lookup. AC-26's array-item test only asserts a valid value loads, and AC-27 lacks a remote-reference case and does not assert all required diagnostic context together. Quoin's `gap-analysis` command is unavailable (`quoin gap-analysis --help` returns command not found); the required matrix read and focused acceptance-to-test and code checks were performed manually. Optional intent↔test↔code semantic review was skipped. No plan was supplied.

## Verdict

**CONDITIONAL** — the targeted criteria are tagged, but the report behavior and test oracles do not fully close the amended contract.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-096-AC-17 report vocabulary does not follow local schema refs | src/corpus/flow_rules.rs:332 |
| FND-002 | low | Array-item and fail-closed reference cases lack complete contract assertions | tests/it/flow_rules.rs:186 |

## Review Scope

| Criterion | Matrix status and binders | Check result |
| --- | --- | --- |
| FR-096-AC-17 | tagged: `flow_rule_any_of_resolves_a_local_schema_enum`; existing `report_keeps_rule_and_vocabulary_order_and_uses_full_rule_shape` | The new ref test asserts load acceptance and document resolution, not `FlowReport.vocabularies`; the existing report test uses inline enums. Referenced vocabulary is omitted by the report path. |
| FR-096-AC-26 | tagged: scalar ref and array-item ref tests | Scalar test covers valid and invalid values. Array-item test only verifies that `[covered]` loads; it would also pass if the ref-derived enum were ignored. |
| FR-096-AC-27 | tagged: `flow_rule_enum_refs_fail_closed_when_missing_or_cyclic` | Missing case checks schema path/ref name but not rule and field; cycle case checks rule/field/cycle but not schema location. Remote ref refusal/no-fetch is not tested. Resolver code refuses refs outside the local bundle. |
| FR-096-AC-28 | tagged: `loader_rejects_unknown_registry_references_values_and_check_collisions` | Inline scalar success/failure is exercised; there is no flow-report assertion tying inline enum order and values to this criterion, nor an inline array-item invalid-value assertion. The unchanged inline report path remains covered by the existing report-order test. |

## Coverage

- Plan completion: not assessed.
- Matrix was computed with `quire matrix --scope . --format json`; AC-17, AC-26, AC-27, and AC-28 are all tagged.
- Reverse-gap check on changed behavior found the flow-rule loader and resolver owned by FR-096; no new stub was observed.
- Focused test logs supplied for `flow_rule_` (3 passed) and `flow_rules::` (17 passed). No tests were run during this review.
- Quoin gap-analysis command was unavailable; manual targeted analysis was performed. Optional semantic review was skipped.

## Findings Detail

### FND-001 — FR-096-AC-17 report vocabulary is still inline-only

The new loader resolves referenced documents for enum validation, but report construction at `src/corpus/flow_rules.rs:332` still calls `vocabulary_coverage::enum_at(schema, field)`, which only inspects inline enums. Add resolved-reference vocabulary extraction to the report path and assert the referenced enum values and their schema order in the flow report.

### FND-002 — Acceptance-test oracles are partial

The array-item test at `tests/it/flow_rules.rs:186` has only an in-enum `any_of` value, so it does not prove the referenced enum constrains values. The missing/cyclic test does not assert schema location, rule, and field together for each case, and has no remote-reference refusal case. Extend the cases so a bad array-item value fails and each missing/cyclic/remote diagnostic is checked for the contract's context and fail-closed behavior.
