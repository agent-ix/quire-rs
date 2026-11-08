---
id: SR-20261103118129
title: "Gap analysis — PLAT-57 inline expression extraction (quire-rs PR 600)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; spec/functional/FR-071-clause-and-operation-extraction.md, spec/functional/FR-075-model-feature-extraction.md, spec/tests.md, src/semantic/clauses.rs, src/semantic/model.rs, src/semantic/scan.rs, src/semantic/surface.rs, tests/fixtures/semantic/cases.json, tests/it/semantic_boundary.rs, tests/it/semantic_surface.rs, tests/python/test_semantic.py"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/FR-071", type: references}
  - {target: "ix://agent-ix/quire-rs/FR-075", type: references}
---

## Summary

Ticket: PLAT-57. PR head `964a9dafe75e0d1e82f67eb7c6a86e13b5300f31`. A planless, PR-scoped audit of FR-071-AC-10..14 and CON-3, FR-075-AC-28..30, and TC-2040..TC-2043. `quire matrix --scope .` reports all nine criteria `tagged`. The installed Quoin CLI has no `gap-analysis` command, so the reverse-gap, stub and semantic checks were done by hand. Plan completion: not assessed.

## Verdict

**CONDITIONAL** — every criterion maps to a tagged test whose `expect` block fails on wrong behaviour. The tests assert exact `diagnosticCodes`, explicit `values` (spans included) and `absentPaths`, plus record equality with `cases.expected.json` and schema validity.

- FR-071-AC-10 / FR-075-AC-30: `plat57-pre-expression` asserts the text, the span (6..23), `pre: []`, the frame `pre: []`, the advisory and lossy operations. Its control asserts `[a, b]`.
- AC-11: control, `Pre: a,` / `Post: a,,b`, `a, x > 1`, `Post: missing`, and `(a)` both declared and undeclared.
- AC-12: the empty and commas-only lines, each asserting `entry-errors: lines 28`.
- AC-13: byte-for-byte text plus the schema check in the helper.
- AC-14: an erroring entry and a duplicate (dropped) operation.
- FR-075-AC-28: the escaped-pipe span is 27..33 for a 5-byte text, so it is one byte longer, as required. The empty-cell case asserts both keys absent.
- FR-075-AC-29: dangling, declared, and the no-bundle-index `Emits` outcome compared with and without the expression.
- CON-3: static denylist; see the code-review SR, FND-002.
- The TC-2043 Python half was verified by building a wheel and running `tests/python/test_semantic.py` (4 passed).

Reverse gap: all new production code belongs to FR-071/FR-075. The changed source has no TODO, FIXME, `todo!`, `unimplemented!`, `unwrap()` or `expect(`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | `spec/tests.md` still marks FR-071 AC-10..14, FR-075 AC-28..30 and TC-2040..TC-2043 as 🚧, though this PR implements and tests them. Status flips are not content edits; #593 flipped its TC and summary rows while implementing. Flip them in this PR (no CR note needed for status-only changes). | spec/tests.md:111, spec/tests.md:114, spec/tests.md:1148-1151 |
