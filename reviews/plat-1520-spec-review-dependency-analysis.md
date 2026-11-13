---
id: SR-20261113045741
title: "PLAT-1520 quire-rs PR664 spec-review dependency analysis"
type: SpecReview
analysis: dependency
scope: "agent-ix/quire-rs PR664 (git diff origin/main...HEAD); spec/functional/FR-084-two-way-relation-coverage-report.md; spec/functional/FR-085-cross-repository-edge-resolution.md; spec/functional/FR-058-upward-trace-completeness.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-084
    type: references
  - target: ix://agent-ix/quire-rs/FR-085
    type: references
---

## Summary

Ticket: PLAT-1520. Relationship-edge pass over the new `relationships:` entries. FR-084 extends FR-058 and requires FR-050 and FR-085. FR-085 extends FR-026 and FR-025. Both implement StR-006. All targets exist on origin/main or in this PR. The FR-084 → FR-085 → FR-084 Upstream/Downstream sections agree. The cross-repo consumer quire-cli FR-030 (quire-cli#180) references FR-084/FR-085 consistently. That PR was not reviewed here.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-085 `implements` StR-006, but StR-006 puts resolution across specs out of scope. Its "Out of scope (stateful engine, → service layer)" list includes "resolving a reference that points into a *different* spec", and its in-scope list says "no cross-corpus state". FR-025 repeats this: "Inter-spec and stateful concerns belong to the service layer (StR-006)". FR-085 resolves edges into sibling repositories inside the engine, and neither StR-006 nor FR-025 is amended. Fix: amend StR-006's scope (and the FR-025 sentence) to allow a caller-supplied multi-member corpus resolved without I/O, keeping fetch/persistence out of scope; or trace FR-085 to a stakeholder need that allows it. | spec/functional/FR-085-cross-repository-edge-resolution.md:11 |
| FND-002 | low | FR-058 gains a note pointing at FR-085, but its Dependencies section still lists only `spec-objects-security#5` downstream. FR-084 (which `extends` FR-058) and FR-085 (which changes what FR-058 counts as dangling) are not named there. Fix: add both to FR-058's Downstream line. | spec/functional/FR-058-upward-trace-completeness.md:224 |

## Verdict

Changes requested on FND-001 (high): the new FR's parent stakeholder requirement explicitly excludes what the FR does. FND-002 is a low bookkeeping gap in the Dependencies prose.
