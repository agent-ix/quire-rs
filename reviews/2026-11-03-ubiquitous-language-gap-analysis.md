---
id: SR-20261103118116
title: "Gap analysis — QSpec Ubiquitous Language engine fix"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; spec/; src/semantic/context.rs; src/semantic/model.rs; tests/semantic_model.rs"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-rs/FR-075"
    type: references
---

## Summary

Planless, change-focused review of the section-body/model-table fix. `quire matrix` confirms the affected FR-075-AC-5 criterion is tagged by the regression test, but the repository-wide matrix still contains untagged criteria; the installed Quoin CLI does not provide a `gap-analysis` command, so the full reverse-gap and stub inventory could not be run.

## Verdict

**FAIL** — the computed repository matrix contains 271 untagged criteria; this is a repository-wide finding, while the changed behavior itself has tagged test coverage.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The computed matrix reports 271 untagged criteria among 1,020 total; FR-075-AC-5 is tagged. | `quire matrix --scope <repo> --format tsv` |
| FND-002 | medium | The installed Quoin CLI reports `gap-analysis` as an unknown command, preventing the required full reverse-gap and stub audit; only changed files were inspected for those conditions. | `quoin gap-analysis --help` |

## Coverage

- Plan completion: not assessed.
- Matrix: `quire matrix --scope /private/tmp/quire-rs-ubiquitous --format tsv`; 642 tagged, 271 untagged, 107 method-without-symbol (1,020 criteria total). `FR-075-AC-5` is tagged.
- Code/test alignment: the new test declares the same `section_body` locator and model table used by the production module; it asserts both successful table extraction and refusal without the body declaration.
- Reverse-gap/stub inspection: changed files only. No stubs or unowned behavior were found in the changed code; repository-wide inventory unavailable because the installed Quoin CLI lacks the skill command.
