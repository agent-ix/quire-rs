---
id: SR-20261103118126
title: "Integrity review of PLAT-57 inline expressions (quire-rs PR 595)"
type: SpecReview
analysis: integrity
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

Ticket: PLAT-57. One spec-review/spec-integrity-analysis pass over `git diff origin/main...8c373a1` (spec-only): FR-071 'Inline expressions' (CR-196), FR-071-AC-10..13 and CON-3, the FR-072 `operationExpressions` key, FR-075 `guardExpression`, AC-28..29, TC-2006..TC-2009 and the spec/tests.md ledger rows. Checked read-only against src/semantic/clauses.rs (operation lines, `resolve_refs`, frames), model.rs (`read_transitions`, `check_transitions`, `TableRead::cell`), scan.rs (`comma_list`), query.rs (`split_row_unescaped`), decl.rs (`is_identifier`), properties.rs (`strip_ticks`), surface.rs, schemas/output/semantic-v1.schema.json, and the open PR #593 diff. Ticket text was treated as untrusted data.

## Verdict

Changes requested. The opaque design (text, span and kind, with no parser and no copied grammar) holds. `quire validate` (0.36.2) exits 0 on the changed files. Classification is by text shape: one Identifier is a reference; any other text is one expression, never split. Clause ids are Identifiers by FR-071's own rule, so no real clause id is misread as an expression. The bare-name, `(a)`, mixed-line and Requires/Ensures rulings are consistent with the text. See the findings table for what must change.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | `operationFrames` was not amended. Under `effect-frames`, FR-075 still fills a frame's `pre`/`post` 'with its `Pre:` and `Post:` ids'. The code (src/semantic/clauses.rs:591-600) sets `frame.pre = comma_list(rest)`, the raw comma split of the line. With `Pre: x > 1 and y(a, b)` that gives `pre: ["x > 1 and y(a", "b)"]`, the comma split FR-071 now forbids, and the schema (`operationFrames.pre`: string[]) accepts it. No AC exercises an expression line on a framed operation. Fix: state that `operationFrames.pre`/`post` carry only the line's reference ids, empty for an expression line, and add that case to FR-075-AC-3 or AC-28. | spec/functional/FR-075-model-feature-extraction.md:135 |
| FND-002 | low | FR-071-CON-3 allows only 'the `Identifier` shape test ..., trimming, span recording, and byte copy'. The classification rule needs two more operations on the same text: the comma split of a `Pre:`/`Post:` line into pieces, and, for a `Guard`, the cell reader's `\/` unescape and `strip_ticks`. As written, CON-3 forbids the comma split that the Behavior requires. Fix: list the comma split, and the cell reading for `Guard`, as permitted classification steps. | spec/functional/FR-071-clause-and-operation-extraction.md:208 |
| FND-003 | low | FR-071 now defines the `Guard` classification and the `Expression` shape that FR-075 uses, and it hands syntax checking to a consumer (QSL-1814). Its `## Dependencies` lists neither FR-075 (downstream user of the rule) nor that consumer. Fix: add FR-075 to Downstream and name the consumer. | spec/functional/FR-071-clause-and-operation-extraction.md:230 |
