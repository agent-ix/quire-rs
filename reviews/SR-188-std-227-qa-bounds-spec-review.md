---
id: SR-188
title: "STD-227 scoped review of FR-065 and AP-201"
type: SpecReview
analysis: base
scope: "Changed FR-065-AC-19/30-48 verification references and AP-201 Tool Reliance and Independence text"
review_set: subset
---

## Summary

Reviewed the changed requirement citations and assurance statement against the QA Rust reader and Quire test harness. Requirement IDs, obligations, and acceptance conditions are unchanged; the change updates implementation and evidence names after the QA port.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AP-201 initially described a Python reader that the QA Rust port removes, overstating independence of the new direct bounds calls. | spec/assurance/AP-201-detection-minting.md:72 |
| FND-002 | medium | The AC-19 citation to QA bounds tests overstated the executed evidence for undeclared-state refusal. | spec/functional/FR-065-controlled-corpus-contract.md:624 |

## Dispositions

- FND-001: fixed. AP-201 now identifies two Rust readers and explicitly limits the independence claim for direct QA bounds calls.
- FND-002: fixed. TC-1021 now derives the bounds report, checks the declared-cell sum, and mutates the declaration to require an undeclared-state refusal.

## Verdict

The changed specification has no unresolved introduced base-review finding. All changed FR-065 criteria have test bindings in the computed matrix. The pinned QA reader remains the authority for its own bounds behavior; Quire's test harness is a separate reader for grading parity.

## Validation

Scoped `quire validate --scope . spec/functional/FR-065-controlled-corpus-contract.md` passed. Whole-spec validation fails on pre-existing provider/schema conflicts, including AP-201 frontmatter fields and nine other documents; this branch changes only AP-201 body prose. The computed matrix has 241 repository-wide untagged criteria outside this diff.
