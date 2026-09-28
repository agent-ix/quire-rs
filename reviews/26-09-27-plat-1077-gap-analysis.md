---
id: SR-121
title: "Gap analysis — CR-187 coverage_matrix / ignored / range (quire-rs#495)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs@271dba170a53f7b0b2e333b20a1845fdc1313d0f; FR-050-AC-5, FR-050-AC-20, FR-050-AC-47..51, FR-051-AC-3, FR-051-AC-18, FR-051-AC-21, FR-051-AC-23, FR-051-AC-27, FR-051-AC-28; spec/tests.md TC-1930..TC-1944; src/coverage/reconcile.rs, src/symbols/*.rs, src/symbol_table.rs"
review_set: subset
relationships:
  - type: references
    target: ix://agent-ix/quire-rs/TM-001
---

## Summary

Ticket: PLAT-1077. PR: quire-rs#495 at `271dba1`. The audit is planless and
scoped to the PR's CR-187/CR-188 surface. It covers the chain from spec AC to
Test Matrix row to tagged test to code.

Matrix verification: `quire coverage --scope . --json` (quire 0.33.0, engine
0.47.1) reports every row TC-1930..TC-1944 as backed, with no status lie and
no untracked symbol in scope. The only unbacked FR-051 row is FR-051-AC-26
(TC-1882), which is pre-existing and outside this PR. TC-1939 and TC-1943 bind
through the `tcNNNN_` name form. The rest bind through `#[trace]` markers.

Intent check (mechanical, from reading each test against its AC; the full
semantic review was not run): the rows are backed, but four AC clauses have
**no test at all**, and the code breaks each of them when probed:

- FR-051-AC-27, aliased unittest forms
- FR-051-AC-27, `skipif` excluded at module level
- FR-051-AC-28, `implements` channel
- FR-051-AC-28, `mentions` channel

Reverse gap: no unowned production behaviour was found. Every new code path
cites CR-187 and an owning AC.

## Verdict

**CONDITIONAL.** All matrix rows are backed by real tags, with no stubs and no
unowned code. But the ✅ on TC-1936, TC-1937 and TC-1941 overstates what their
tests prove (AC clauses without a test, below). There is no `high` finding in
this lane. The `high` is in SR-120.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-051-AC-28's `implements` clause ("mints no `verifies`/`implements` relation") has no test behind it, and the code violates it. An `implements` marker range still binds its literal string (SR-120 FND-005). TC-1936 and TC-1937 exercise only `verifies`. | spec/tests.md:891, src/symbols/trace.rs:1007 |
| FND-002 | medium | FR-051-AC-28's `mentions` clause ("contributes to neither ... `unmatched_tags`, `mentions`") is untested, and the code violates it (SR-120 FND-006). TC-1936 asserts `unmatched_tags` only. | spec/tests.md:891, src/symbols/trace.rs:741 |
| FND-003 | medium | FR-051-AC-27's aliased-import clause and its module-level `skipif` exclusion have no test. TC-1941 covers `skipif` only as a decorator and never as `pytestmark = pytest.mark.skipif(...)`, and it has no aliased form. Both are broken (SR-120 FND-002, FND-003). The Rust `proptest!` form of "any function AC-3 classifies as a test" is also untested and broken (SR-120 FND-004). | spec/tests.md:896, src/symbols/python.rs:435 |
| FND-004 | medium | FR-051-AC-27's inheritance clause is tested only with unique suite titles. With duplicate titles, which AC-21 permits and TC-1944 uses as its premise, the inherited flag is computed from the wrong container (SR-120 FND-001). | spec/tests.md:895, src/symbols/mod.rs:520 |
| FND-005 | low | TC-1932's row claims `statement` (CR-188) and a full ordering tuple, but the test asserts neither. Only the FR-050-AC-20 baseline golden catches a blanked `statement`. TC-1936's "reported once" and its line and symbol clauses are likewise unasserted. | tests/coverage_matrix.rs:127, tests/coverage_matrix.rs:323 |

## Coverage

- Rows in scope: TC-1930..TC-1944 (15), all ✅, all backed by a tagged test (15/15).
- ACs in scope: FR-050-AC-5, AC-20, AC-47, AC-48, AC-49, AC-50, AC-51; FR-051-AC-3, AC-18, AC-21, AC-23, AC-27, AC-28. Each has at least one tagged test. Clause-level gaps are listed above.
- Stubs or coverage inflation: none found in the diff.
- Reverse gap (code with no owning requirement): none.
- Semantic review: skipped (reviewer lane; clause-level intent checked by hand, as above).
- Plan completion: not assessed

## New findings (disposition pass 1)

Reviewed at `agent-ix/quire-rs@12490853bcae9bdaef017eb4e4f6f6b80c3086ba`.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | medium | FR-051-AC-28's "whether read by a canonical marker or a legacy textual form" clause is untested for a legacy range on a production (non-binding) symbol, and the code breaks it: no range finding is raised and the endpoints land in `mentions` (SR-120 FND-014). TC-1936 and TC-1937 cover only evidence symbols. | spec/tests.md:891, src/symbols/trace.rs:560 |

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed b69a5c9 | `tc1936_a_range_in_an_implements_marker_binds_nothing` pins AC-28's implements clause, and the TC-1936 row was widened (d7c8f95). |
| FND-002 | fixed b69a5c9 | `tc1936_range_endpoints_do_not_leak_into_mentions` pins the mentions clause for marker ranges. |
| FND-003 | fixed 1673a5d | Added `tc1941_aliased_unittest_skip_forms_are_ignored`, `tc1941_module_level_skipif_is_a_control` and `tc1939_ignore_on_a_proptest_declared_test_marks_it_ignored`. |
| FND-004 | fixed 1673a5d | `tc1940_inherited_ignored_follows_the_declaration_site_not_the_title` covers duplicate titles in both orders. |
| FND-005 | fixed b69a5c9 | TC-1932 asserts `statement` and ordering. TC-1936 asserts once-per-occurrence, line and symbol. |
| FND-006 | fixed ca6f2ba | `tc1937_a_legacy_range_on_a_production_function_is_reported` covers Rust and Python with a control, and pins the range, the non-binding tag and the absence of mentions. |
| FND-007 | fixed cc49805 | `tc1937_a_legacy_range_on_a_contained_test_is_the_tests` pins the only-record clause for Rust `mod tests`, a Python class and a TS `describe`. The corpus cases now assert `tag-on-non-binding-symbol` is absent (qa-corpus 876a170). |

Matrix check at 1249085: `quire coverage` (the engine under test) still backs TC-1930..TC-1944. The gate runs green. Plan completion: not assessed.

## New findings (disposition pass 2)

Reviewed at `agent-ix/quire-rs@74460aeb173ae5f4c5b5c925f0c6a9ae467bada8`.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-007 | medium | FR-051-AC-28 says a range occurrence's "range finding is the only record of it". For a legacy range on a test, the occurrence now also yields a `tag-on-non-binding-symbol` finding against the enclosing container (SR-120 FND-018). No test checks that the evidence-placed legacy range reports nothing else, and the corpus case does not assert that the reason is absent. | spec/functional/FR-051-source-symbol-extraction.md:172, src/symbols/trace.rs:581 |


Plan completion: not assessed.

Round 3 at 165a1d3: no open gap findings. Plan completion: not assessed.
