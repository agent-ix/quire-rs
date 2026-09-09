---
id: SR-081
title: "Integrity review of the YAML engine decision"
type: SpecReview
analysis: integrity
scope: "ADR-0012, NFR-009, TC-1820..TC-1831"
review_set: all
---

## Summary

> **Superseded by SR-087.** This review evaluates the withdrawn 0.10.2
> candidate and remains only as decision history.

The exact package alias, locked graph, corpus equality, and immutable provenance
form the right integrity chain. Two independent NFR obligations are collapsed
into one TC, weakening attribution when only one gate runs or fails.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-441 | medium | TC-1824 jointly covers NFR-009-AC-9 (`sca-sbom`) and AC-10 (`static-quality`). A single status cannot prove both the license/advisory graph and the unsafe/static dependency audit ran, nor identify which evidence is absent. | NFR-009-AC-9..AC-10; spec/tests.md:852,1812-1813 | missing-requirement |

## Integrity chain

Manifest alias → lock/tree resolution → exact corpus manifest/digests → two
engine outputs → cross-language and typed-consumer results → MSRV/security/static
gates. Each link needs a distinct retained result identity.

## Repeat-review disposition

| ID | Disposition |
| --- | --- |
| FND-441 | Fixed: TC-1824 now covers refreshed advisory/license evidence only; independent TC-1829 covers NFR-009-AC-10 unsafe/static evidence. |
