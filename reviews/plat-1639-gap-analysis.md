---
id: SR-2026112016223475846355048
title: "Gap analysis — quire-rs self-test resources"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; src/lib.rs, src/selftest.rs, selftest/cases/fr-032-placeholder-detection/**, spec/functional/FR-032-validate-document.md; PLAT-1639 and quire-cli FR-047 context"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-032
    type: references
---

## Summary

Ticket: PLAT-1639. Audited the changed case-resource surface against the local FR-032-AC-3 matrix row and the cross-repository case ownership contract. The new case exercises the stated placeholder behavior, but its `required-tools` file is not empty as the shared runner contract requires.

## Verdict

**CONDITIONAL** — the one malformed case resource can make the initial quire-rs case appear unavailable or invalid to the consuming CLI.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The new case's `required-tools` file is a single newline byte. The consuming runner's contract treats each non-empty line as a bare executable and uses a truly empty file to mean no tools; this resource can instead produce an empty-name requirement and prevent the case from running. | selftest/cases/fr-032-placeholder-detection/required-tools:1 |

## Coverage

- Plan completion: not assessed.
- Computed matrix: `quire matrix --scope . --format json` exited 0; FR-032-AC-3 had two existing tagged binders in `src/validate_document.rs` and `tests/it/markdown_validation.rs`.
- Reverse-gap inspection was limited to the changed API and case: the exported case table serves the downstream quire-cli FR-047 contract; the case behavior is owned by local FR-032-AC-3. No source stub or copied case asset was found.
- The matrix command emitted many pre-existing diagnostics outside the changed case. No full repository audit or semantic review was performed; semantic review is opt-in.
