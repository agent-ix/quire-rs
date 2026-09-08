---
id: SR-085
title: "Scope and boundary review of the YAML engine decision"
type: SpecReview
analysis: scope-boundary
scope: "ADR-0012, NFR-009"
review_set: all
---

## Summary

The decision correctly keeps parser semantics, TypeScript/Python references,
and all Rust typed consumers in scope while excluding parser redesign and MSRV
increase. It lacks a mapped call-site census proving that the enumerated classes
remain complete at the implementation revision.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-447 | medium | NFR-009 measures omitted production YAML consumer classes at zero, but no acceptance criterion or TC performs a repository-wide `serde_yaml` call-site/import census and classifies production versus test-loader use. A new or overlooked consumer can escape TC-1822 while the named suites pass. | ADR-0012:35-39,122-126; NFR-009:79-81; TC-1822 | missing-requirement |

## Boundary allocation

Quire-rs owns the selected package, backend, Rust call sites, and retained
evidence. TypeScript/Python supply reference behavior; upstream maintenance is
an assumption monitored by the ADR, not an upstream guarantee.

## Repeat-review disposition

| ID | Disposition |
| --- | --- |
| FND-447 | Fixed: ADR-0012 evidence item 7 and NFR-009-AC-14/TC-1830 require a repository-wide call-site census and exact equality with the exercised production classes. |
