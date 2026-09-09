---
id: SR-086
title: "EARS conformance review of the YAML engine decision"
type: SpecReview
analysis: ears-conformance
scope: "NFR-009"
review_set: all
---

## Summary

> **Superseded by SR-087.** This review evaluates the withdrawn 0.10.2
> candidate and remains only as decision history.

Quire 0.31.0 reports ADR-0012 and NFR-009 2/2 grammar-clean with zero target
findings. Human review finds one compound criterion that joins independent
language and typed-consumer outcomes under a single result.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-448 | medium | NFR-009-AC-7 combines the TypeScript/Python frontmatter parity suite with six Rust typed-YAML consumer classes. These have different owners, inputs, and failure loci; one criterion/TC can report “pass” without proving each independent obligation. Split the reference-language and typed-consumer outcomes. | NFR-009-AC-7; TC-1822 | missing-requirement |

## Pattern result

The target statement and remaining criteria have named subjects, bounded
conditions, and observable outcomes. The split should preserve their present
closed consumer list and not weaken it to “affected suites.”

## Repeat-review disposition

| ID | Disposition |
| --- | --- |
| FND-448 | Fixed: AC-7/TC-1822 now cover only TypeScript/Python frontmatter parity; new AC-11/TC-1826 separately cover the five Rust typed-consumer suites. |
