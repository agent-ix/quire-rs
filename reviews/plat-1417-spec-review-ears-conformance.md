---
id: SR-20261113045748
title: "ears-conformance review of FR-086 design-element join and FR-058 CR-416 (quire-rs PR 668)"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-rs@8f9a5b9; spec/functional/FR-086-design-element-join.md; spec/functional/FR-058-upward-trace-completeness.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-086
    type: references
  - target: ix://agent-ix/quire-rs/FR-058
    type: references
---

# EARS conformance review of FR-086 and FR-058 CR-416 (quire-rs PR 668)

## Summary

Ticket: PLAT-1417. Checked the new normative statements for EARS shape: the CR-416 SHALL statement, the FR-086 load-failure and vocabulary SHALL statements, and the `design_relations` MAY. These are well-formed ubiquitous or conditional statements. FR-086 has no principal statement of what the system SHALL produce.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-086 never says what `quire-rs` SHALL produce. The join's core behaviour is descriptive ("`design_join` takes ... and returns ..."), and the only SHALLs cover load failure and vocabulary reporting. FR-084 opens with "`quire-rs` SHALL produce this report". Fix: one ubiquitous statement, for example "`quire-rs` SHALL produce the design-element join for any loaded corpus ...". | spec/functional/FR-086-design-element-join.md (The join) |
| FND-002 | low | The CR-416 paragraph puts normative rules outside its SHALL sentence: "An edge authored both ways between one pair counts once" and "only relation matching normalizes" are stated as facts. Fix: fold them into the SHALL statement or a second one. | spec/functional/FR-058-upward-trace-completeness.md (Authored inverse verbs) |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | low | CR-418 changes `dangling-reference` behaviour, but the rule exists only in a `> CR-418 note` blockquote and FR-038-AC-12. The FR-038 body still defines `dangling-reference` without the exception, so the normative statement is missing. Fix: one SHALL sentence in the FR-038 body where `dangling-reference` is defined. | spec/functional/FR-038-okf-bundle-validation.md |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1: FR-086 now opens with "`quire-rs` SHALL produce the design-element join for any loaded corpus and any set of declared design relations, including none ...". |
| FND-002 | fixed | Round 1: CR-416 now states its rules as SHALL statements ("For each verb `V` ... a relation SHALL accept", "The relation SHALL count an edge authored both ways ... once, and its FR-084 `linked` entry SHALL carry the listed verb"). |
| FND-003 | fixed | Round 2 (0a89ebd): the FR-038 body now states, where `dangling-reference` is defined, that a reference to an id minted from exactly one document by a trace target named by an `obligations:` source is not a dangling reference and "the check SHALL NOT report it". |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | low | The CR-417 SHALL sentence is split by a stray blank line (FR-058 line 161). One paragraph ends "...(the `target:` field of FR-053)," and the next begins "SHALL be read as an edge to `D`", so the rendered normative statement is two fragments and the second has no subject. Fix: delete the blank line. | spec/functional/FR-058-upward-trace-completeness.md:160-162 |
