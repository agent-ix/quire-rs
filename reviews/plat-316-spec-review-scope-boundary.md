---
id: SR-316
title: "Scope boundary review of FR-050 trace target sources"
type: SpecReview
analysis: scope-boundary
scope: "FR-050-AC-58; declarative trace-target source contract"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-050"
    type: reviews
---

## Summary

FR-050-AC-58 assigns source selection, section scoping, and extraction behavior
to Quire, with table-row behavior preserved by default. The owner has allocated
declaration ownership: Quoin owns ADR/StR/US and spec-artifacts-process owns
marker declarations (owner decision recorded for PLAT-332). The remaining
boundary gap is named cross-repository evidence that a producer declaration
accepts the new sources and quire-rs extracts them.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The owner decision resolves declaration ownership: Quoin owns ADR/StR/US and spec-artifacts-process owns marker declarations. FR-050-AC-58 assigns `from` interpretation and invalid-pattern rejection to Quire at module load, but names no cross-repository contract evidence proving that a producer declaration accepts `list_item` and `heading` and quire-rs extracts their ids. Add a named contract test covering declaration loading and extraction across that boundary. | Owner decision recorded for PLAT-332; FR-050 Description; FR-050-AC-58 |
