---
id: SR-16431
title: "EARS conformance review of FR-051-AC-47/48 (quire-rs PR 622)"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-rs; spec/functional/FR-051-source-symbol-extraction.md (PR 622 diff only, head 21408e0)"
review_set: subset
---

## Summary

Tickets: PLAT-334, PLAT-335. The PR adds no requirement statement and edits none. Its additions are two acceptance criteria, FR-051-AC-47 and FR-051-AC-48, and an explanatory note. The FR-051 `SHALL` statements are unchanged. The deleted FR-079 and FR-080 statements now live in ix-trace-py and ix-trace-ts FR-001, worded the same, and are out of this repository's scope.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

Examined: FR-051-AC-47, FR-051-AC-48 and the PLAT-334 / PLAT-335 note. Both ACs are declarative and observable, and each names its observation (`quire coverage --json` pairs; JUnit attributes, `quoin evidence record`, binding). They follow the FR-051-AC-32/33 shape. The note states no obligation, so it needs no EARS form. Clean.
