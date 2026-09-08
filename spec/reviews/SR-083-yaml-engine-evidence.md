---
id: SR-083
title: "Evidence review of the YAML engine decision"
type: SpecReview
analysis: evidence
scope: "ADR-0012, NFR-009, TC-330..TC-332, TC-1820..TC-1831"
review_set: all
---

## Summary

The human review independently confirmed the MSRV, current manifest mismatch,
and advisory status. The exploratory 575-document zero-difference run remains
properly labelled non-reproducible; deterministic advisor evidence is unavailable
in this sandbox and must not be fabricated.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-444 | medium | `quoin advise --repo . --json` cannot spawn its nested `quire` process in this sandbox (`EPERM`) and reports a misleading version-premise failure. Direct `quire --version` is 0.31.0 and target validation succeeds. Record advisor output unavailable; do not treat this as an old-CLI or #414 defect. | Quoin 0.23.1 `src/quire/exec.ts`; review run 2026-09-08 | correct-requirement-no-evidence |
| FND-445 | medium | The exploratory differential deliberately lacks the exact TypeScript revision, complete input manifest/digests, comparator, module identities, and raw payload. It supports option selection but cannot discharge implementation or acceptance. | ADR-0012:76-106 | correct-requirement-no-evidence |

## Required evidence

Implementation must produce the retained differential envelope (FND-439),
unchanged language/typed-consumer results, lock/tree resolution, Rust 1.75 build,
and distinct license/advisory and unsafe/static results.

## Repeat-review dispositions

| ID | Disposition |
| --- | --- |
| FND-444 | Open environment limitation, explicitly retained; direct Quire version/validation evidence is separate. |
| FND-445 | Resolved as a decision-evidence boundary: the spike remains non-dispositive, and AC-12/TC-1827 requires the reproducible replacement result before implementation acceptance. |
