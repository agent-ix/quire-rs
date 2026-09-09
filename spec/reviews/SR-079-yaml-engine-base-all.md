---
id: SR-079
title: "Owner-selected base review of the YAML engine decision (#414)"
type: SpecReview
analysis: base
scope: "ADR-0012, NFR-009, TC-330..TC-332, TC-1820..TC-1831"
review_set: all
---

## Summary

> **Superseded by SR-087.** This review evaluates the withdrawn 0.10.2
> candidate and remains only as decision history.

The amended candidate is structurally and grammatically valid, identifies the
complete YAML surface, and maps NFR-009-AC-1..AC-10. It remains conditional: the
mandatory repository-wide validation fails pre-existing assurance/assets, and
the retained differential provenance is a decision gate without an AC/TC.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-438 | high | `quire validate --scope . "spec/**/*.md" --summary` fails 11 documents: two untyped assets, invalid AP-201, and MP-201..MP-208 missing required body sections. The target ADR/NFR are 2/2 clean, but the full-scope `/spec-review` gate cannot be reported validated. | AP-201; MP-201..MP-208; spec/assets/external-blockers.md; spec/assets/render-parity-notes.md | wrong-requirement |
| FND-439 | high | ADR-0012 requires retained source/corpus/module revisions, input digests, comparator/producer versions, raw output, counts, exclusions, and differences, but NFR-009-AC-6 and TC-1821 assert only equality. An implementation can pass TC-1821 while retaining none of the evidence needed to reproduce the decision. | ADR-0012:138-153; NFR-009-AC-6; TC-1821 | missing-requirement |

## Coverage

The initial ten NFR criteria mapped to TC rows. Option, failure, boundary, state,
and edge-case coverage still need the additions identified by this all-analysis
set; mapping alone is not evidence.

## Repeat-review dispositions

| ID | Disposition |
| --- | --- |
| FND-438 | Open external gate: repository-wide validation still fails the 11 Engineering Assurance/assets documents. The target ADR/NFR and all eight selected reviews validate individually. |
| FND-439 | Fixed: NFR-009-AC-12 and TC-1827 now require and test the complete reproducibility envelope. |

The amended NFR has 15 criteria, all mapped through TC-330..TC-332 and
TC-1820..TC-1831, including the required canonical `ix-trace-rs` marker gate.
