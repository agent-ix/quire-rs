---
id: SR-20261103118162
title: "Dependency analysis of FR-049 kind-typed references (quire-rs PR 649)"
type: SpecReview
analysis: dependency
scope: "agent-ix/quire-rs@a2e2b4c0; spec/functional/FR-049-verification-reference-integrity.md (relationships, Dependencies), spec/functional/FR-057-corpus-check-severity.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-049
    type: references
---

# Dependency analysis of FR-049 kind-typed references (quire-rs PR 649)

## Summary

Ticket: PLAT-1471. Examined: the FR-049 frontmatter edges (`extends` FR-038, `requires` FR-050, `implements` US-017), the Dependencies section, and the FR-057 link. The consumer, spec-artifacts-process FR-015-AC-6 (open PR spec-artifacts-process#140), is downstream. This PR adds no edge to it, which is correct: the consumer depends on the engine, not the reverse. There is no ordering constraint on this merge, because this is a spec-only addition with open TCs.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | low | FR-049 now states normatively that `reference-kind-mismatch` is "registrable as `trace:reference-kind-mismatch` (FR-057)", and FR-049-AC-12 depends on FR-057's registry. FR-049's `relationships:` and its Dependencies "Upstream" list still name only FR-050, FR-038 and FR-025. Fix: add FR-057 to the Upstream list, and add a frontmatter edge if the repo's convention records corpus-severity coupling as an edge. | spec/functional/FR-049-verification-reference-integrity.md:5-13,90-92,137 | missing-requirement |

## Verdict

One low fix. No blocking ordering.

## Dispositions

Round 1 at d5d98b3d.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | FR-049 frontmatter gains `requires ix://agent-ix/quire-rs/spec/functional/FR-057`, and the Dependencies Upstream list now leads with FR-057 (the severity registry AC-12 relies on). |
