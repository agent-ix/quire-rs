---
id: SR-20261115183657
title: "Code review of PLAT-1441 spec PR #688"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; spec/functional/FR-069-semantic-module-contract-at-load.md, spec/functional/FR-081-record-sections-extraction.md, spec/functional/FR-084-two-way-relation-coverage-report.md (PR #688)"
review_set: all
---

## Summary

Ticket: PLAT-1441. The diff is spec-only: three FRs, +82/-3, with no Rust, so `rust-review`
does not apply. The code-review lens covers whether each new AC is a testable claim an
implementation can fail. The design was checked against the current locator code
(`src/extract/locator.rs` `FrontmatterField`). A `multiple: false` locator returns an
authored array whole (`frontmatter_field_array_value_returned_whole`), so the "no
`multiple: true`" premise holds.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-069-AC-42, FR-081-AC-19 and FR-084-AC-22 test "unrelated to any regulated-data vocabulary" and "the engine source names no such key", but no vocabulary is defined, so the check cannot fail deterministically; name the word list (or reuse the FR-058-CON-2 / FR-084-AC-17 mechanism) and mark the source claim Inspection. | spec/functional/FR-069-semantic-module-contract-at-load.md:463; spec/functional/FR-081-record-sections-extraction.md:284; spec/functional/FR-084-two-way-relation-coverage-report.md:214 |
| FND-002 | low | FR-084-AC-22 is compound: byte-identity, key placement and fixture naming in one criterion. | spec/functional/FR-084-two-way-relation-coverage-report.md:214 |

## Verdict

Changes requested (medium on FND-001). The remaining new ACs (FR-069-AC-39..41,
FR-081-AC-18 with its control, FR-084-AC-20/21) are concrete and falsifiable. They include
the absence assertions this repo's fixture rule asks for (AC-41 key absent, not `[]`). Not
validated: `quire validate` was denied by the permission layer in this session.
