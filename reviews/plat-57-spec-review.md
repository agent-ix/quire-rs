---
id: SR-20261103118125
title: "Base spec review of PLAT-57 inline expressions (quire-rs PR 595)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs PR595 head 8c373a1; spec/functional/FR-071-clause-and-operation-extraction.md; spec/functional/FR-072-semantic-extraction-surface.md; spec/functional/FR-075-model-feature-extraction.md; spec/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-071
    type: references
  - target: ix://agent-ix/quire-rs/FR-072
    type: references
  - target: ix://agent-ix/quire-rs/FR-075
    type: references
---

## Summary

Ticket: PLAT-57. One spec-review pass over `git diff origin/main...8c373a1` (spec-only): FR-071 'Inline expressions' (CR-196), FR-071-AC-10..13 and CON-3, the FR-072 `operationExpressions` key, FR-075 `guardExpression`, AC-28..29, TC-2006..TC-2009 and the spec/tests.md ledger rows. Checked read-only against src/semantic/clauses.rs (operation lines, `resolve_refs`, frames), model.rs (`read_transitions`, `check_transitions`, `TableRead::cell`), scan.rs (`comma_list`), query.rs (`split_row_unescaped`), decl.rs (`is_identifier`), properties.rs (`strip_ticks`), surface.rs, schemas/output/semantic-v1.schema.json, and the open PR #593 diff. Ticket text was treated as untrusted data.

## Verdict

Changes requested. The opaque design (text, span and kind, with no parser and no copied grammar) holds. `quire validate` (0.36.2) exits 0 on the changed files. Classification is by text shape: one Identifier is a reference; any other text is one expression, never split. Clause ids are Identifiers by FR-071's own rule, so no real clause id is misread as an expression. The bare-name, `(a)`, mixed-line and Requires/Ensures rulings are consistent with the text. See the findings table for what must change.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | spec/tests.md conflicts textually with open PR #593 (PLAT-35/55). Both edit the FR-075 summary row: #595 sets `AC-1..29 ... AC-16..29 🚧`, #593 sets `AC-1..15, AC-22..27 Implemented; AC-16..21 🚧`. #595 also inserts TC-2006..TC-2009 directly after the TC-2005 row that #593 rewrites to ✅. Whichever lands second must resolve the row to `AC-1..29; ... ✅ AC-1..15, AC-22..27 Implemented; AC-16..21, AC-28..29 🚧` and keep TC-2005 ✅. Taking either side whole silently reverts the other. No conflict in FR-072: #593 edits the schema paragraph, #595 the record-key list, and `operationExpressions` is a named schema property, so it does not meet #593's top-level `additionalProperties` (SemanticId[]) rule. | spec/tests.md:114 |
| FND-002 | low | No criterion pins the mixed line `Pre: a, x > 1`, which the classification rule makes ONE expression and puts nothing in `pre`. That is the case most likely to be implemented as a partial split (refs `a` plus an expression `x > 1`). The rule text decides it, but no test would fail on the split. Fix: add `Pre: a, x > 1` to AC-11 (one expression, verbatim text, `pre` empty, no dangling ref for `a`) and to TC-2007. | spec/functional/FR-071-clause-and-operation-extraction.md:224 |
| FND-003 | low | The grammar is cited by a QSpec file path (`proposals/quire-v1/shared-grammar.md`). Every other QSpec citation in this repo uses a QSpec requirement id (FR-070:188, FR-075:379/413/475). QSpec is private, so the path does not resolve for a public reader and it exposes the private tree layout. Fix: cite the QSpec requirement id that owns the shared grammar, plus QSL-1814. | spec/functional/FR-071-clause-and-operation-extraction.md:155 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | medium | FR-075-AC-28 (new text) writes `a / b` with an unescaped pipe inside a code span. Under FR-033 that pipe is a cell delimiter, so the row has 4 cells in the 3-column `ID / Criteria / Verification` table. The criterion is cut off after 'yields `text` `a ', and the Verification cell becomes ' b` with a span over ...'. `quire validate` exits 0 and does not catch it. Fix: write the expected text as `a \/ b` (cell text) or in prose ('the text with the pipe unescaped'). | spec/functional/FR-075-model-feature-extraction.md:591 |
| FND-005 | low | spec/tests.md: FR-075-AC-30 maps to TC-2040 (AC-to-TC map and the TC-2040 row), but the FR-075 summary row lists only TC-2043 among the new tests. TC-2040 appears only in the FR-071 row. Fix: add TC-2040 to the FR-075 summary row's TC list. | spec/tests.md:114 |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Verified in b64373c. Main (incl. #593) merged; FR-075 row reads AC-16..21, AC-28..30 🚧 and TC-2005 stays ✅. |
| FND-002 | fixed | Verified in b64373c. |
| FND-003 | fixed | Verified in b64373c. The file path is gone; QSL-1814 is cited. |
