---
id: SR-20261102118109
title: "PLAT-1149 code-review review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; PLAT-1149; five-file diff and adjacent identity/evidence paths"
review_set: subset
---

## Summary

Candidate: 204d70ed841d6a0872ca239dd36e4d4639c51a1c; base main: 9adea425. Ticket: PLAT-1149; PR: quire-rs#515. Review-only; frozen code was not edited.

Main-red log reproduces the missing five RT criterion gaps; candidate focused logs show 53 rollup, 17 matrix, and 2 baseline tests passing. Candidate formatter and clippy logs pass. These were inspected existing logs, not rerun by this reviewer. No aggregate Cargo gate was run in this reviewer as directed. `corpus_cases_hold` is red with 63 forward-expectation differences; corpus ref is unchanged and QA-corpus #31 is an explicit unresolved dependency. No full-bank or merge-ready claim.

## Verdict

**PASS** — No defect found in the dispatched changed behavior; merge gates remain independently outstanding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Assurance Context

AP-201 (`spec/assurance/AP-201-detection-minting.md`) applies: wrong evidence interpretation has material impact across coverage consumers. Reviewed the existing selected-inventory path, exact native retained row lookup and retirement/status handling. AP-201 still contains legacy `review_selection`; it has no current `review_policy.operations` and is not interpreted as an enforced workflow instruction. No active exception was supplied or inferred. The focused candidate tests share engine implementation and do not establish independent producer qualification. Whole `make ci`, candidate-wide matrix, and a compatible green independent corpus are unavailable in this review; QA-corpus #31 remains open/unowned in this dispatch. No measurement target or gate is waived.

## Coverage

The changed production branch uses the existing module-derived obligation ID set and checks only that obligation's own graph relation. All other rows retain own-or-referenced backing. The retained-evidence OR still calls exact document/ID/line lookup; selected native inventory is consumed without a second census. No module/source-kind/FR-prefix predicate, binder shim, public API, wire schema or CI change was introduced. Retirement still exits before gap/status-lie classification; pending remains non-complete.

Twelve declarative cases execute real native extraction/binding/reconciliation: Rust/Python/TypeScript adverse/control pairs, aggregate-reference-only adverse/control, non-ISO Rule identity adverse/control, and parked FR-275-AC-23 adverse/control. Their trivial source assertions are extractor input, explicitly not qualification of copied RT behavior. FR-275-AC-23's authored method is Test. Exact unbacked sets check absence as well as presence. The baseline change is justified: its round-trip fixture source binds TC-001 only, with no FR-001-AC-1 claim.

## Examined Units

```yaml
scope:
- id: FR-050-AC-3
  path: spec/functional/FR-050-declarative-coverage-computation.md
  role: examined
  excerpt: A live reference row whose trace target has no backing `verifies` relation
    appears in the report's unbacked rows with the row id and its target id. When
    its row ID names a module-derived obligation, source backing requires a `verifies`
    relation for that obligation's own ID; a referenced TC or sibling obligation cannot
    supply it. Other reference rows retain the own-ID-or-referenced-ID rule. Explicit
    retained evidence continues to require FR-078's accepted exact row binding.
- id: FR-050-AC-8
  path: spec/functional/FR-050-declarative-coverage-computation.md
  role: examined
  excerpt: A fixture module with a non-ISO vocabulary (different archetype, id pattern,
    and status values) obtains a correct rollup from its own declaration, with no
    engine change.
- id: source_backed
  path: src/coverage/reconcile.rs
  role: examined
  excerpt: Some(id) if obligation_ids.contains(id) => backed.contains(id),
- id: obligation_reference_rows_require_their_own_evidence
  path: tests/coverage_rollup.rs
  role: examined
  excerpt: 'assert_eq!(actual, expected, "{name}: sibling criteria cannot borrow TC-025
    backing; aggregate TC row retains referenced evidence");'
- id: shared-tc-partial-owner-rust
  path: tests/fixtures/corpus_cases/obligation-reference-backing.json
  role: examined
  excerpt: '{"unbacked": ["FR-008-AC-3", "FR-008-AC-4", "FR-008-AC-10", "FR-008-AC-11",
    "FR-008-AC-12"], "obligations": 6}'
- id: shared-tc-partial-owner-rust-control
  path: tests/fixtures/corpus_cases/obligation-reference-backing.json
  role: examined
  excerpt: '{"unbacked": [], "obligations": 6}'
- id: shared-tc-partial-owner-python
  path: tests/fixtures/corpus_cases/obligation-reference-backing.json
  role: examined
  excerpt: '{"unbacked": ["FR-008-AC-3", "FR-008-AC-4", "FR-008-AC-10", "FR-008-AC-11",
    "FR-008-AC-12"], "obligations": 6}'
- id: shared-tc-partial-owner-python-control
  path: tests/fixtures/corpus_cases/obligation-reference-backing.json
  role: examined
  excerpt: '{"unbacked": [], "obligations": 6}'
- id: shared-tc-partial-owner-typescript
  path: tests/fixtures/corpus_cases/obligation-reference-backing.json
  role: examined
  excerpt: '{"unbacked": ["FR-008-AC-3", "FR-008-AC-4", "FR-008-AC-10", "FR-008-AC-11",
    "FR-008-AC-12"], "obligations": 6}'
- id: shared-tc-partial-owner-typescript-control
  path: tests/fixtures/corpus_cases/obligation-reference-backing.json
  role: examined
  excerpt: '{"unbacked": [], "obligations": 6}'
- id: shared-tc-partial-owner-rust-aggregate-reference-only
  path: tests/fixtures/corpus_cases/obligation-reference-backing.json
  role: examined
  excerpt: '{"unbacked": ["FR-008-AC-3", "FR-008-AC-4", "FR-008-AC-10", "FR-008-AC-11",
    "FR-008-AC-12"], "obligations": 6}'
- id: shared-tc-partial-owner-rust-control-aggregate-reference-only
  path: tests/fixtures/corpus_cases/obligation-reference-backing.json
  role: examined
  excerpt: '{"unbacked": [], "obligations": 6}'
- id: shared-tc-partial-owner-rust-module-vocabulary
  path: tests/fixtures/corpus_cases/obligation-reference-backing.json
  role: examined
  excerpt: '{"unbacked": ["Rule-008-AC-3", "Rule-008-AC-4", "Rule-008-AC-10", "Rule-008-AC-11",
    "Rule-008-AC-12"], "obligations": 6}'
- id: shared-tc-partial-owner-rust-control-module-vocabulary
  path: tests/fixtures/corpus_cases/obligation-reference-backing.json
  role: examined
  excerpt: '{"unbacked": [], "obligations": 6}'
- id: parked-rt-handoff-retains-own-gap
  path: tests/fixtures/corpus_cases/obligation-reference-backing.json
  role: examined
  excerpt: '{"obligations": 1, "unbacked": ["FR-275-AC-23"]}'
- id: parked-rt-handoff-retains-own-gap-control
  path: tests/fixtures/corpus_cases/obligation-reference-backing.json
  role: examined
  excerpt: '{"obligations": 1, "unbacked": []}'
- id: baseline:FR-001-AC-1
  path: tests/fixtures/coverage_baseline/expected.json
  role: examined
  excerpt: '"row_id": "FR-001-AC-1", "target_ids": ["FR-001-AC-1", "TC-001"]'
- id: FR-008-AC-3
  path: tests/fixtures/corpus_cases/obligation-reference-backing.json
  role: context_only
  excerpt: '`construct_collection` charges `collection.element` before running each
    deferred element and stops at the first non-completing element with no later element
    run; a completed element outside its declared type returns `CheckedInvariant {
    cause: CollectionElementNotAdmitted }`; `form_collection` refuses a type-mismatched
    occurrence before any charge; formation charges membership comparisons, `collection.bound`
    and `collection.result-retain` in that order, and a set or bag is stored in ascending
    canonical-key order.'
- id: FR-008-AC-4
  path: tests/fixtures/corpus_cases/obligation-reference-backing.json
  role: context_only
  excerpt: The type-owned canonical key totally orders every keyed type, ranks `Absent
    < Null < Present`, fixes set and bag canonical and visiting order, and is evaluated
    iteratively at a depth that would overflow a recursive host-stack walk.
- id: FR-008-AC-7
  path: tests/fixtures/corpus_cases/obligation-reference-backing.json
  role: context_only
  excerpt: The extended `Undefined` and `Refusal` vocabularies, `BoundViolation` and
    `Refusal::cause()` are closed, typed and distinct from every FR-006 variant; both
    `CardinalityOutOfBound` directions are reachable and report their `code()` and
    `cause()`; `CheckedInvariant` is unreachable from any admitted vector in the shared
    corpus.
- id: FR-008-AC-10
  path: tests/fixtures/corpus_cases/obligation-reference-backing.json
  role: context_only
  excerpt: When formation coalesces occurrences, the runtime charges one `collection.member-walk`
    and one `collection.member-test` per membership comparison for a set, bag or ordered
    set, and charges neither point for a sequence.
- id: FR-008-AC-11
  path: tests/fixtures/corpus_cases/obligation-reference-backing.json
  role: context_only
  excerpt: When a collection is retained, the runtime stores a sequence in supplied
    occurrence order, an ordered set as the first occurrence of each distinct member
    in first-occurrence order, a set as one occurrence of each distinct member in
    ascending canonical-key order, and a bag as every occurrence in ascending canonical-key
    order.
- id: FR-008-AC-12
  path: tests/fixtures/corpus_cases/obligation-reference-backing.json
  role: context_only
  excerpt: When a retained collection is visited by equality planning or by a further
    formation, the runtime visits its elements in the order it stored them, for every
    one of the four kinds.
- id: FR-078-AC-9
  path: spec/tests.md
  role: context_only
  excerpt: TC-1971
- id: FR-078-AC-13
  path: spec/tests.md
  role: context_only
  excerpt: TC-1975
- id: FR-078-AC-14
  path: spec/tests.md
  role: context_only
  excerpt: TC-1976
- id: FR-078-AC-15
  path: spec/tests.md
  role: context_only
  excerpt: TC-1977
```
