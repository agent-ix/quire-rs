---
id: SR-193
title: "dependency review of FR-078 requires Quoin FR-116 (quire-rs#571)"
type: SpecReview
analysis: dependency
scope: "agent-ix/quire-rs; spec/functional/FR-078-qualified-evidence-binding.md (diff origin/main...f7844ce)"
review_set: subset
---

## Summary

Ticket: IR-1006. Examined the new `requires ix://agent-ix/quoin/FR-116` edge and the
Dependencies section. Direction is correct (FR-078 consumes, FR-116 lists FR-078 downstream,
no reverse pin, no reciprocal SHA cycle; the removed SHA baseline was pin ceremony). Two gaps:
a transitive owner dependency is missing and the enablement build has no named owner.

## Verdict

**PASS with findings** - edge direction and enablement/feature separation are sound; FND-001
should be fixed before the bridge build starts.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | FR-116 requires FR-093 bounded reliance as a mandatory input and positive premise (FR-116 L36, L72, L150-156), but FR-078's owner-dependency list (L367-372) names only FR-030, FR-032, FR-115; the bridge must pass the FR-093 decision and its trigger is not traced. | spec/functional/FR-078-qualified-evidence-binding.md:367-372 | missing-requirement |
| FND-002 | low | The first blocked-work row (L347) says "Implement and expose the reviewed Quoin FR-116 owner decision" with no owning ticket; the prior text named PLAT-1162 and FR-116 L32 says PLAT-1162 does not supply it. The qualifier runtime and bridge build is enablement with no named owner in the spec. | spec/functional/FR-078-qualified-evidence-binding.md:347 | missing-requirement |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `requires ix://agent-ix/quoin/FR-093` added (L16-17) and FR-093 dependency bullet L449-453. |
| FND-002 | fixed | Blocked-work row L423 names owning build IR-1006 (PLAT-1160 is Done). Quoin FR-116 L258 still attributes the bridge to PLAT-1160; that is a Quoin-side text, outside this PR. |
