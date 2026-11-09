---
id: SR-16421
title: "EARS review of CR-215 only marker forms bind (quire-rs PR 623)"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-rs@9901e1f3; spec/functional/FR-050-declarative-coverage-computation.md; spec/functional/FR-051-source-symbol-extraction.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-051
    type: references
---

# EARS review of CR-215 (quire-rs PR 623)

## Summary

Ticket: PLAT-847. Checked requirement grammar for the new and edited statements: FR-051 narrative, CON-3, CON-4, AC-11, AC-22, AC-49, AC-50, and FR-050-AC-44. `quire validate` (0.36.2) raises `ears:non-singular` on FR-051 line 188 (CON-3), which origin/main did not raise.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-051-CON-3 carries two `SHALL`s ("SHALL carry no field ..." and "SHALL NOT retain a deprecated compatibility path"). `quire validate` flags `ears:non-singular`. Split it, or fold the second clause into the first | spec/functional/FR-051-source-symbol-extraction.md:188 |
| FND-002 | low | FR-050-AC-44 is compound. It is a statement of absence about the FR-051 census plus a second clause about a different channel ("An authored tag no marker read is signalled by the unbound candidate's `unmatched_example` and by the FR-051-AC-49 finding") that this AC does not own and TC-2164 does not assert | spec/functional/FR-050-declarative-coverage-computation.md:361 |

## Verdict

These are minor grammar defects. AC-49 and AC-50 are long but each states one observable trigger and response.
