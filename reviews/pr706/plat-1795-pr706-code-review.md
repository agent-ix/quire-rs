---
id: SR-2026111921333400000005
title: "Code review of PLAT-1795 assurance JSON refusal tests"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs; PR #706; tests/it/assurance_export.rs"
review_set: base
relationships:
  - target: ix://agent-ix/quire-rs/spec/functional/FR-067
    type: reviews
---

## Summary

Reviewer: Claude Opus 5.5. Initial review at
fbfc2887a74a09605848e98c7fee878975215fcf; final disposition at
78003b2a195a06ae04f59f7d8780f3747f6a2f5a.

The initial review found that duplicate-key refusal was tested only at the root
and that the deep-input case accepted any InvalidJson reason.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Duplicate-key refusal was tested only at the root, leaving nested maps and array elements uncovered. | tests/it/assurance_export.rs |
| FND-002 | low | The deep-input test accepted any InvalidJson reason instead of requiring the recursion-limit reason. | tests/it/assurance_export.rs |

## Disposition history

At e3979a11049a0eaaaccde3b3ad3cd50366b23ea7, the nested-object duplicate
case and recursion-limit assertion fixed the initial findings. Review added
FND-003: the spec said “more than 128” although serde_json refuses at the
128th nested container. The disposition kept FND-001 open because an object
inside an array was not covered, fixed FND-002, and added FND-003.

The final change covers duplicate keys in an object inside an array, requires
the recursion-limit reason, and tests exactly 128 total containers (127 arrays
inside the root object). FR-067-AC-10 and CR-423 specify refusal at 128 or
more nested containers.

| Finding | Final result |
| --- | --- |
| FND-001 | Fixed: duplicates in nested objects and objects inside arrays are refused. |
| FND-002 | Fixed: deep input returns InvalidJson with “recursion limit exceeded,” without panicking. |
| FND-003 | Fixed: the test reaches the 128-container boundary and the spec uses the same bound. |

An intermediate check at eefc534187582a32e442f9c5d78ecf6c1cf89b5d confirmed
the array-element and reason assertions but counted the root object as an
additional container. No separate round-two artifact was persisted. The
boundary was corrected and rechecked at the final head.

## Final disposition record

<!-- reviewer-dispositions repo=agent-ix/quire-rs visibility=public quoin=0.36.8 module=spec-artifacts-process@v0.36.0 id=SR-2026111921333400000005 round=3 model=claude-opus-5-5 run=plat706-disposition-final -->

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | - |
| FND-002 | fixed | - |
| FND-003 | fixed | - |

Round 3 disposition for PR quire-rs#706 at
78003b2a195a06ae04f59f7d8780f3747f6a2f5a. Focused assurance-reader tests:
2 passed, 956 filtered. Wrapped formatting and git diff --check passed.
No new blockers.

Disposition data:

    pr: quire-rs#706
    head: 78003b2a195a06ae04f59f7d8780f3747f6a2f5a
    dispositions:
      - fnd: FND-001
        outcome: fixed
        after: array-element duplicate object is refused with the repeated key named
      - fnd: FND-002
        outcome: fixed
        after: InvalidJson reason contains “recursion limit exceeded”
      - fnd: FND-003
        outcome: fixed
        after: FR-067-AC-10 states 128 or more nested arrays or objects are refused
