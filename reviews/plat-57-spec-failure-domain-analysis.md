---
id: SR-20261103118127
title: "Failure-domain review of PLAT-57 inline expressions (quire-rs PR 595)"
type: SpecReview
analysis: failure-domain
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

Ticket: PLAT-57. One spec-review/spec-failure-domain-analysis pass over `git diff origin/main...8c373a1` (spec-only): FR-071 'Inline expressions' (CR-196), FR-071-AC-10..13 and CON-3, the FR-072 `operationExpressions` key, FR-075 `guardExpression`, AC-28..29, TC-2006..TC-2009 and the spec/tests.md ledger rows. Checked read-only against src/semantic/clauses.rs (operation lines, `resolve_refs`, frames), model.rs (`read_transitions`, `check_transitions`, `TableRead::cell`), scan.rs (`comma_list`), query.rs (`split_row_unescaped`), decl.rs (`is_identifier`), properties.rs (`strip_ticks`), surface.rs, schemas/output/semantic-v1.schema.json, and the open PR #593 diff. Ticket text was treated as untrusted data.

## Verdict

Changes requested. The opaque design (text, span and kind, with no parser and no copied grammar) holds. `quire validate` (0.36.2) exits 0 on the changed files. Classification is by text shape: one Identifier is a reference; any other text is one expression, never split. Clause ids are Identifiers by FR-071's own rule, so no real clause id is misread as an expression. The bare-name, `(a)`, mixed-line and Requires/Ensures rulings are consistent with the text. See the findings table for what must change.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | An empty comma piece changes a valid reference list into an expression. Today `comma_list` drops empty pieces (src/semantic/scan.rs:142-144), so `Pre: a,` and `Pre: a,,b` resolve as references. Under 'pieces are each a semantic-core `Identifier`', an empty piece is not an Identifier, so the whole line becomes ONE expression `a,`. Clause `a` silently leaves `pre`, and the only signal is the unchecked advisory. This contradicts 'as before' and the CR note's 'no existing criterion changes'. Fix: state that empty pieces are dropped before the shape test (today's behaviour), or that they are an error, and add the case to AC-11. | spec/functional/FR-071-clause-and-operation-extraction.md:131 |
| FND-002 | medium | `text` 'the trimmed text byte-for-byte' and a `sourceSpan` from 'first byte' to 'one past its last byte' are undefined for a `Guard` cell. The cell reader consumes `\/` into `/` (src/query.rs:243) and `strip_ticks` removes one outer backtick pair (src/semantic/properties.rs:72; model.rs:1238). So for a cell written `a \/\/ b` the text is `a // b`, which is not the source bytes, and the span cannot both cover the source and match the text length. A cell `` `x` == `y` `` is read as `` x` == `y ``, which corrupts the expression. A `Pre:` line is not a table cell, so its text keeps backslashes and backticks, which is inconsistent with `Guard`. The FR-033/PLAT-217 escaped-pipe convention is not stated here. Fix: say `Guard` text is the cell text as the FR-033 cell reader yields it (`\/` read as `/`). Say the span covers the cell's source bytes, escapes included. Either drop `strip_ticks` for `Guard` or define it as the whole-cell code span only. Add a `\/` case to FR-075-AC-28. | spec/functional/FR-071-clause-and-operation-extraction.md:143 |
| FND-003 | medium | The spec does not say what happens to `operationExpressions` when `operations` is `unavailable` (`entry-errors`), or when an operation is dropped (duplicate heading, an error on its own lines). FR-075 states that a refused `Guard` expression is omitted with the whole `model`, but FR-071 gives no matching rule. A reasonable implementer could emit expressions that point at operations that are absent or have no `OperationDecl`. Fix: state that `operationExpressions` is absent whenever `operations` is absent or unavailable, and covers only emitted operations. Add the absence to AC-12. | spec/functional/FR-071-clause-and-operation-extraction.md:163 |
| FND-004 | low | The rule 'keeps a mistyped clause id an error' holds only for spelling typos that stay an Identifier. Clause-id lines that today fail as `semantic.dangling-clause-ref` become advisory-only expressions: `` Pre: `notArchived` `` (backticked id, ticks are not stripped on `Pre:`), `Pre: a b` (missing comma), `Pre: a; b`, `Pre: notArchived.`, and a `Guard` cell `a, b`. The ruling accepts this, since the text is preserved and the advisory plus a downstream syntax check expose it. The spec should still say so instead of claiming every typo stays an error. Fix: narrow the sentence to 'a mistyped id that is still an Identifier', and note that other shapes are expressions. | spec/functional/FR-071-clause-and-operation-extraction.md:136 |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Verified in b64373c. |
| FND-002 | fixed | Verified in b64373c. Residual by design: one outer backtick pair is still stripped, so a whole-cell `x` == `y` loses its outer ticks. That is now stated behaviour, not an ambiguity. |
| FND-003 | fixed | Verified in b64373c. |
| FND-004 | fixed | Verified in b64373c. |
