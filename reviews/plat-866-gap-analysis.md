---
id: SR-20261103118146
title: "Gap analysis of structural Python and TypeScript implements syntax against FR-062 and FR-051-AC-51 (quire-rs PR 630)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; git diff origin/main...0d94218; spec/functional/FR-062-implements-relation.md AC-18..27 and CON-5, spec/functional/FR-051-source-symbol-extraction.md AC-22 and AC-51, spec/tests.md TC-2200..TC-2210"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/FR-062", type: references}
  - {target: "ix://agent-ix/quire-rs/FR-051", type: references}
---

## Summary

Ticket: PLAT-866. PR head `0d94218`. Plan completion: not assessed (planless). This is a manual AC-to-test-to-code check of FR-062-AC-18..27, FR-062-CON-5, FR-051-AC-22 (the structural widening) and FR-051-AC-51 against `tests/it/implements_structural.rs` (14 tests) and the changed source. Each TC-2200..TC-2210 is tagged on at least one test with its own AC through `#[trace("TC-…", "FR-…")]`. All tags were read and all are correct (bindings below).

The 14 tests pass at the head, run in a scratch worktree with `locked-build.sh`, debug=0.

Bindings checked, all correct:
- TC-2200 → AC-18 (1 test)
- TC-2201 → AC-19 (2)
- TC-2202 → AC-20 (1)
- TC-2203 → AC-21 (2)
- TC-2204 → AC-22 (1)
- TC-2205 → AC-23 (1)
- TC-2206 → AC-24 (1)
- TC-2207 → AC-25 (1)
- TC-2208 → AC-26 and FR-051-AC-22 (1)
- TC-2209 → AC-27 (2)
- TC-2210 → FR-051-AC-51 (2)

## Verdict

**Pass with findings: one medium, two low.**

Every AC has a test that can fail on its main clause, and every new production path has an owning requirement. No stub, no hollow assertion, and no pin, digest or count kept for its own sake. `assert_eq!(before_report.totals.backed, 1)` is a functional control.

Fully covered: AC-18 (all seven refusal cases name the form), AC-19, AC-20, AC-21 (including the abstract-signature per-file rejection), AC-22, AC-23, AC-24 (all five unattached shapes plus the attached and decorator controls) and FR-051-AC-51 (`memoize(fn)`, `callee(1)(memoize(fn))` and an unwrapped function expression mint nothing).

CON-5 inspection: decorator and wrapper candidates come only from tree-sitter nodes in `python.rs` and `typescript.rs`. The binder applies the module pattern to node text. There is no comment/string lexer, no import resolution and no execution.

The code-review SR records one untested defect (FND-001 there, a comment inside a nested chain). That is a code fix, so it is not repeated here.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-2209 skips three oracles that spec/tests.md requires. **Identity** must compare "Python and TypeScript declarations … every symbol id, path, kind and declaration position". The test compares `(id, path, kind)` for one Python file, with no TypeScript and no line or column. **Locations** must "place a decorator and a wrapper after unrelated source and a trailing comment" and assert each candidate's line. Only the wrapper's line is asserted. **Determinism** must compare ordered diagnostics "including partial-match and unattached findings". The repeated input produces neither kind, so their ordering is never compared. Each subcase must fail independently, so these AC-27 clauses are unbacked. | tests/it/implements_structural.rs:645-723, spec/tests.md:1288-1292 |
| FND-002 | low | AC-26 names `@implements("FR-007-AC-2")` "for a Python function, method or class". The test has only a Python module-level function. The method and class renderings (positions taken from the method's and class's own `decorated_definition`) are never asserted. | tests/it/implements_structural.rs:586-629 |
| FND-003 | low | AC-25 requires the partial finding and no relation from the partial candidate "even where another candidate matches fully". No structural test gives one symbol both a partially matched and a fully matched candidate. Example: `@implementing("A")` plus `@x.implementing("B")` under one form. | tests/it/implements_structural.rs:529-576 |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | fd0834d: `typescript_identity_decorator_location_and_finding_order_are_preserved` (TC-2209) covers the three missing oracles. Identity compares TypeScript `(id, path, kind, line, end_line)` with and without the structural forms. Locations asserts the decorator lines (class 3, method 4, Python def 4) and the wrapper line (6), each after unrelated source and a trailing comment. Determinism repeats an input that yields 2 partial-match findings and 1 unattached finding, and compares the graph and diagnostics. Python declaration position is still not compared, but a Python decorator always occupies its own line, so that clause cannot hold literally for Python. |
| FND-002 | fixed | fd0834d: `the_advisory_names_the_decorator_for_a_python_method_and_class` asserts `@implements("FR-007-AC-2")` for a Python class (line 1) and a method (line 6). |
| FND-003 | fixed | fd0834d: `a_partial_candidate_creates_no_relation_even_where_another_candidate_matches_fully` gives one Python function a fully matching candidate and a partially matching one. It asserts that only `("f", "FR-775")` binds and that exactly one partial finding names `@t.implements("FR-776")`. |
