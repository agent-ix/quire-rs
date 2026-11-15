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

## New findings (disposition pass 1)

Pass 1 re-read the AC tables of FR-058, FR-069, FR-081 and FR-084 at head `5731e42`.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | low | FR-084-AC-29 requires "AC-20..AC-28 pass with a fixture using none of" `regulatedCategories`, `health`, `personal`, `financial`. AC-23 is in that range, and its fixture must use `regulatedCategories` and `health`, so the two ACs cannot both hold as written. Use the range AC-20..AC-22, AC-24..AC-28. | spec/functional/FR-084-two-way-relation-coverage-report.md:235 |
| FND-004 | low | FR-069-AC-38 excepts "a FR-081 section under a frontmatter-owned key ... FR-081-AC-18". AC-18 is the section-only case, which has one source and is never a duplicate. The two-source exemption it means is FR-081-AC-19. | spec/functional/FR-069-semantic-module-contract-at-load.md:483 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The undefined "regulated-data vocabulary" is replaced by the named list `regulatedCategories`, `health`, `personal`, `financial`, in separate Inspection ACs (FR-069-AC-46, FR-084-AC-29). The FR-081 fixture-naming AC is gone. |
| FND-002 | fixed | The compound AC is split. Determinism is FR-084-AC-25 (with key order), placement after `criteria` is AC-27, and the source and fixture check is AC-29. |
| FND-003 | still-open | New this pass; the fix round has not run yet. |
| FND-004 | still-open | New this pass; the fix round has not run yet. |

- FND-001 after_excerpt (`spec/functional/FR-069-semantic-module-contract-at-load.md:491`): `FR-069-AC-46 | The engine source names none of regulatedCategories, health, personal or financial as a key, value or special case, and AC-39..AC-45 pass with a fixture using none of them. | Inspection`
- FND-002 after_excerpt (`spec/functional/FR-084-two-way-relation-coverage-report.md:231,233`): `FR-084-AC-25 | With record_keys: [zeta, alpha] and a document stating zeta before alpha, record serializes alpha then zeta; two runs of the report are byte-identical.` / `FR-084-AC-27 | In every document entry that carries record, the record key follows criteria.`

Pass-1 verdict: approved for this method. FND-003 and FND-004 are low and do not block, but they
are one-token fixes and should go in the same fix round as the blocking findings of the base and
integrity reviews.
