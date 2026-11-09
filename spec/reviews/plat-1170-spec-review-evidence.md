---
id: SR-16329
title: "Evidence review of PLAT-1170 FR-050-AC-61 (quire-rs PR 610)"
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-rs; spec/functional/FR-050-declarative-coverage-computation.md, spec/tests.md (PR 610 diff only)"
review_set: subset
---

## Summary

Ticket: PLAT-1170. This review checked that FR-050-AC-61's verification method, `Test (TC-2083)` (Integration, P0), fits a module-loaded, engine-observable diagnostic, and that TC-2083 exercises each clause of the AC. The method fits. One load-bearing clause has no case that could fail.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-2083 has no case for the AC's declaration-driven clause ("never from form names in the engine"), such as the same fixture with both forms renamed. An implementation that hard-codes `rust-implements` and `Implements:` passes every case listed. TC-2083 also covers only a single id, so it never exercises the template being rendered "with the ids that occurrence binds" for a comma list. | spec/tests.md:1130, spec/functional/FR-050-declarative-coverage-computation.md:307 |

## Verdict

Examined: FR-050-AC-61 → TC-2083. The method is right. The listed negative controls (canonical attribute, module without the form, non-Rust source) and the unchanged-counts check can each fail. Add a renamed-form control and a multi-id occurrence to TC-2083.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | TC-2090 adds a renamed-form control ("the same fixture with every form renamed yields the same findings"), a two-id rendering case, a template-less control and a no-`RewriteSuggestion` check. |
