---
id: SR-16424
title: "failure-domain review of CR-215 only marker forms bind (quire-rs PR 623)"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-rs@9901e1f3; spec/functional/FR-050-declarative-coverage-computation.md; spec/functional/FR-051-source-symbol-extraction.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-051
    type: references
---

# Failure-domain review of CR-215 (quire-rs PR 623)

## Summary

Ticket: PLAT-847. Looked for unstated failure modes and identity confusion in the new advisory (FR-051-AC-49/50), its channels and the rollout window. The major issues are recorded in SR-16420: wrong replacement text on non-binding kinds (FND-003), the release-order load failure (FND-002), and the range rule for implements textual forms (FND-004). Two smaller gaps are recorded here.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The same token is reported on two channels without saying so. An id-shaped token in an evidence symbol's comment is both an FR-050-AC-39 `unmatched_tags` record and an AC-49 `trace-id-outside-marker` diagnostic. AC-28 states explicitly that its finding replaces both generic channels, while AC-49 does not say whether it coexists with or replaces `unmatched_tags`, so two implementations could differ | spec/functional/FR-051-source-symbol-extraction.md:241 |
| FND-002 | low | Findings can share an identity. AC-49 locates a finding by file and line only, so the same id written twice on one line (`FR-1 ... see FR-1`) yields two findings with identical content. Either name a column or state that one id per line is one occurrence | spec/functional/FR-051-source-symbol-extraction.md:241 |

## Verdict

These are minor. With SR-16420's high findings fixed, the advisory's failure surface is bounded.
