---
id: SR-061
title: "Dependency review of source-grounded assurance export"
type: SpecReview
analysis: dependency
scope: "StR-007, FR-067, FR-068, IT-001"
review_set: all
---

## Summary

The dependency graph is acyclic. Existing corpus, obligation, schema, and symbol surfaces enable the new v1 envelope; the projection then enables the cross-repository Quoin contract.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No open dependency issues | - |

## Classification

| Requirement | Class | Rationale |
| --- | --- | --- |
| StR-007 | Feature | States the reviewer-visible source-grounding need |
| FR-067 | Enablement | Defines the stable envelope, schema, premise inventory, and reader boundary |
| FR-068 | Feature | Projects the authoritative records used by assurance consumers |

## Dependency Graph

```mermaid
graph TD
  FR025[FR-025 corpus] --> FR067[FR-067 envelope]
  FR051[FR-051 symbols] --> FR067
  FR053[FR-053 obligations] --> FR067
  FR055[FR-055 output contract] --> FR067
  FR067 --> FR068[FR-068 projection]
  FR026[FR-026 relations] --> FR068
  FR051 --> FR068
  FR053 --> FR068
  FR068 --> IT001[IT-001 Quoin contract]
```

Topological order: existing prerequisites, FR-067, FR-068, then IT-001. No cycles were detected.
