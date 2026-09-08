---
id: SR-082
title: "Dependency review of the YAML engine decision"
type: SpecReview
analysis: dependency
scope: "ADR-0012, NFR-009"
review_set: all
---

## Summary

The body correctly orders decision acceptance before implementation. The typed
graph does not encode that prerequisite, and full validation depends on the
Engineering Assurance contract repair being handled under #59.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-442 | medium | NFR-009 frontmatter labels ADR-0012 as `references`, while its Dependencies section says the ADR is upstream and no migration may begin from Proposed. A scheduler sees a reference rather than a prerequisite; the edge must be `requires` or an equivalent ordering relation. | NFR-009:5-14,97-103 | missing-requirement |
| FND-443 | high | The full-scope validation dependency is externally blocked by the installed Engineering Assurance 0.2 contract rejecting AP-201/MP-201..208. #414 must not edit those artifacts around their owner; #59 must supply an accepted versioned contract/migration before this review can validate repository-wide. | FND-438; engineering-assurance PR #22 / #59 | correct-requirement-no-evidence |

## Order

Engineering Assurance contract repair → clean full-scope validation → accept
ADR-0012/NFR-009 → implement dependency change → run differential and all gates.

## Repeat-review dispositions

| ID | Disposition |
| --- | --- |
| FND-442 | Fixed: NFR-009 now carries the allowed typed `depends_on` edge to ADR-0012; the body still forbids implementation from Proposed. |
| FND-443 | Open and owned by Engineering Assurance #59. #414 does not alter AP-201/MP-201..208 to evade their versioned contract owner. |
