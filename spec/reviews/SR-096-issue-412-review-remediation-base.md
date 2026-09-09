---
id: SR-096
title: "Base review of issue 412 review remediation"
type: SpecReview
analysis: base
scope: "NFR-023; TC-1810; TC-1811; PR #415 SR-092 findings"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-rs/NFR-023"
    type: reviews
---

## Summary

This QUOIN base review applies SR-092 to the performance requirement before
the production implementation is changed. The optimized selection result is
already correct; its traversal evidence and no-candidate cost were not.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-108 | high | Closed at the specification boundary: TC-1810 must count the actual regex traversal entry point, so restoring a traversal inside the binding loop increments the counter and fails. | NFR-023-AC-1; TC-1810; SR-092 FND-001 | correct-requirement-no-evidence |
| FND-109 | high | Closed at the specification boundary: TC-1811 must reject every raw direct-candidate capture traversal outside the measured wrapper and prove the assertion traversal precedes the assignment join, independent of the regex variable name. | NFR-023 Verification; TC-1811; SR-092 FND-002 | correct-requirement-no-evidence |
| FND-110 | medium | Closed at the specification boundary: a span without a binding performs no assertion traversal, and a span without any selected direct/helper candidate builds no line index. | NFR-023 Measurement; NFR-023-AC-1; NFR-023-AC-2; SR-092 FND-003 | missing-requirement |

## Disposition

The requirement and planned gates now cover the independent review findings.
TC-1810 and TC-1811 remain pending until their tests first reproduce the
defect and the production path is then remediated. A fresh rust-review and
independent review are required before merge.
