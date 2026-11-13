---
id: SR-20261113085040
title: "Spec integrity review of the FR-007 and FR-010 fence amendments (quire-rs PR 673)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-rs#673 @713efca9; spec/functional/FR-007-fenced-block-heading-walk.md; spec/functional/FR-010-query-api.md; spec/stakeholder/StR-003-parse-parity-with-quire.md (context)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-007
    type: references
---

# Spec integrity review of the FR-007 and FR-010 fence amendments (quire-rs PR 673)

## Summary

Ticket: PLAT-1682. An integrity pass (consistency and atomicity) over the amended
FR-007 and FR-010. FR-010 CR-008 is internally consistent and agrees with its
ACs. FR-007 now holds rules that contradict each other, and its parity framing
is out of date.

## Verdict

**CONDITIONAL**: one medium and one low finding. FR-010 is clean: CR-008's
statements map one to one onto AC-5..8, with no contradiction against CR-004's
matching-character rule, which CR-008 refines.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-007 contradicts itself. Rule 1 says "A line whose `trim_start()` begins with three backticks toggles `in_fence`", and rule 3 says the same of tildes. Rule 6 says only a run indented at most three spaces opens, and a fence closes only on a same-character run at least as long as the opener with whitespace after it. A four-space `` ``` `` or a `` ``` text `` line inside a fence obeys rule 1 and breaks rule 6. Rules 1 and 3 must be rewritten or removed, not added to. | spec/functional/FR-007-fenced-block-heading-walk.md:18-24 |
| FND-002 | low | FR-007 still says the walk follows "the TS/Py reference algorithm" (StR-003 parity). Rule 6 deliberately diverges from TS/Py (`trimStart().startsWith('```')` toggling), and no CR note records the change or the divergence, unlike FR-010 CR-008 and FR-011 CR-004. Repo CLAUDE.md: a spec change requires a CR note. | spec/functional/FR-007-fenced-block-heading-walk.md:16 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | FR-007 rules 1 to 6 are rewritten as a single model, and the trim_start toggle rules are gone. |
| FND-002 | fixed | The FR-007 description no longer claims the TS/Py algorithm, and the CR-009 note records the deliberate divergence. |
