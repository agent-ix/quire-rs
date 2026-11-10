---
id: SR-20266
title: gap analysis of FR-069 import_types spec change
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-rs; spec/functional/FR-069-semantic-module-contract-at-load.md; spec/tests.md
review_set: subset
date: '2026-11-10'
---

# gap analysis of FR-069 import_types spec change

## Summary

Ticket: PLAT-212 (agent-ix/quire-rs PR #637, head e2b6bc42). The change is spec-only. Plan completion: not assessed. AC-16..20 each have one planned TC (TC-2214..TC-2218), marked 🚧, and the FR-069 coverage row is marked 🚧 to match. No test in the repository yet claims these criteria, and the spec claims no implementation, so there is no stub or coverage inflation. The coverage-row omission of AC-15 is recorded in SR-20263.

## Scope

- `FR-069-AC-16` (examined) — spec/functional/FR-069-semantic-module-contract-at-load.md
- `FR-069-AC-17` (examined) — spec/functional/FR-069-semantic-module-contract-at-load.md
- `FR-069-AC-18` (examined) — spec/functional/FR-069-semantic-module-contract-at-load.md
- `FR-069-AC-19` (examined) — spec/functional/FR-069-semantic-module-contract-at-load.md
- `FR-069-AC-20` (examined) — spec/functional/FR-069-semantic-module-contract-at-load.md

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

Clean. Each new criterion has a planned test, none claims to be implemented, and the statuses are honest.
