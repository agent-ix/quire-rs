---
id: SR-20261113085039
title: "Spec review (base) of the FR-007 and FR-010 fence amendments (quire-rs PR 673)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs#673 @713efca9; spec/functional/FR-007-fenced-block-heading-walk.md; spec/functional/FR-010-query-api.md; spec/functional/FR-011-body-extraction-dsl.md (CR-004, context)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-010
    type: references
  - target: ix://agent-ix/quire-rs/FR-007
    type: references
---

# Spec review (base) of the FR-007 and FR-010 fence amendments (quire-rs PR 673)

## Summary

Ticket: PLAT-1682. A base checklist pass over the spec diff: FR-007 rule 6 and
AC-5, and FR-010 CR-008 and AC-5..8. FR-011 CR-004 was read as context. IDs are
well formed and contiguous, `quire validate` on both files exits 0, and each new
AC is a concrete test. The defects are in what the prose claims and what it
leaves undefined.

## Verdict

**CONDITIONAL**: medium and low findings only. Clean: the new AC ids follow on
from the existing ones; each AC states inputs and expected outputs; the
consumer link (quire-cli FR-042) is named; no hash, pin or tracking record was
introduced. AssuranceProfile AP-201 was considered. It applies only indirectly
(section boundaries), so the review was run directly and not as an enforced
ix-flow run.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | CR-008 and FR-007 rule 6 say "Fences follow the CommonMark rules", but they measure indentation as an absolute count ("indented at most three spaces"). CommonMark measures from the containing block. The spec never says that container blocks (list items, block quotes) are out of scope, though FR-042 already treats nested blocks as unsupported, so the claim is wider than the behaviour, and list-nested blocks drop silently. State "top-level blocks only" or specify indentation relative to the container. | spec/functional/FR-010-query-api.md:51-67; spec/functional/FR-007-fenced-block-heading-walk.md:24 |
| FND-002 | low | "Info word" is not defined. CommonMark's first word runs to whitespace; the code takes the leading `\w*` run, so `c++` gives `c`, `objective-c` gives `objective`, and `{.mermaid}` gives the empty string. Two implementers would read CR-008 differently. | spec/functional/FR-010-query-api.md:60; src/query.rs:72 |
| FND-003 | low | CR-008 says it "extends FR-011 CR-004", but FR-011 CR-004 was not amended or back-linked. It still describes the scanner behind `code_block` as closing on any matching-character fence line, so FR-011, the locator's owning requirement, now understates the rule. | spec/functional/FR-011-body-extraction-dsl.md:62-81 |
