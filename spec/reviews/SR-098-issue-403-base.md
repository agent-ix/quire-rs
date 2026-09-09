---
id: SR-098
title: "Base review of issue 403 TypeScript regex-literal specification"
type: SpecReview
analysis: base
scope: "FR-051-AC-25..AC-28, TC-1836..TC-1839, CR-161"
review_set: base
relationships:
  - target: "ix://agent-ix/quire-rs/FR-051"
    type: reviews
---

## Summary

The owner-selected base review examined the #403 specification slice before
implementation. The four criteria separate lexer behavior, refusal behavior,
controlled-corpus mutation evidence, and the retained external-file
demonstration; the slash classifier now states its bounded token contexts and
explicit automatic-semicolon-insertion limit.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-453 | high | AP-201 is applicable to a detector change and declares `review_selection.mode: require` with evidence and scope-boundary analyses, but the installed AssuranceProfile schema rejects `review_selection`, three other top-level fields, the impact-assessment shape, and four required body sections. The profile therefore cannot validly select or complete its required review set. This base review is provisional evidence only; it cannot represent profile satisfaction or owner acceptance. The owner-contract repair is tracked by Engineering Assurance #59 / TASK-024. | AP-201; FR-051-AC-25..AC-28 | wrong-requirement |

## Automated checks

- CR-161, TC-1836..TC-1839, and SR-098 are unique across the reviewed local
  branches; all new identifiers use the repository's declared formats.
- FR-051-AC-25..AC-28 each map to exactly one new matrix row, and the summary
  row exposes all four pending cases rather than presenting them as backed.
- The changed FR, Test Matrix, log, and this review pass Quire structural and
  relationship validation. AP-201 fails separately with the exact schema
  incompatibility recorded in FND-453.

## Six-rule coverage

- **Coverage:** AC-25 maps to TC-1836, AC-26 to TC-1837, AC-27 to TC-1838,
  and AC-28 to TC-1839. The pre-existing AC-23/AC-24 omissions and TC-1042
  summary omission are also reconciled without changing their behavior.
- **Option/permutation:** delimiter cases include escaped opening, closing, and
  paired braces, `[^}]`, escaped slash, comment-shaped content, flags, and a
  regex-shaped template fragment. Division and `/=` are independent controls.
- **Constraint boundary:** expression-ending and expression-start token sets,
  character-class entry/exit, escape consumption, regex closure, flags, and
  statement-start semicolon behavior are explicit. ASI-dependent ambiguity is
  deliberately outside the syntax-level classifier.
- **Error path:** unterminated regex, unmatched closing brace, and unmatched
  opening brace each require a construct-specific 1-based line diagnostic,
  zero symbols for that file, and continued extraction of a readable sibling.
- **State transition:** the one lexer pass owns comment, quoted/template,
  regex, character-class, escape, and code-depth transitions; regex contents
  cannot independently change the later balance state.
- **Edge case:** the pending qa-corpus failure/control pair binds the regression
  and a one-token control, while TC-1839 separately binds the exact unchanged
  external blob and digest. Mutation requirements cover loss of regex masking
  and misclassification of division.

## Review disposition

The base content review is complete and the specification slice is ready for
independent review. FND-453 remains a high governance dependency, so this
artifact does not authorize implementation, claim the AP-201 review set is
complete, or cross the ix-flow human acceptance gate.
