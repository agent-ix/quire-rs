---
id: SR-20261115183658
title: "Gap analysis of PLAT-1441 spec PR #688"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; spec/functional/FR-069-semantic-module-contract-at-load.md, spec/functional/FR-081-record-sections-extraction.md, spec/functional/FR-084-two-way-relation-coverage-report.md (PR #688)"
review_set: all
---

## Summary

Ticket: PLAT-1441. Planless; plan completion was not assessed. The PR is spec-only, so the
nine new ACs (FR-069-AC-39..42, FR-081-AC-18..19, FR-084-AC-20..22) have no tagged tests or
code. That is expected at the spec gate and is not a finding. `quire matrix` could not run
here (permission layer). The gap measured is requirement coverage across the PLAT-1441 chain:
does each upstream obligation routed to quire-rs have a quire-rs criterion? This is
measured against spec-objects-security FR-023, spec-objects-architecture FR-013, quoin
FR-121 (PR #927) and quire-cli FR-030 (PR #204).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | spec-objects-architecture FR-013-AC-5 ("contains no `level` value") is routed to quire-rs FR-084, but no FR-084 criterion asserts that any record key is absent from the report; and quire-cli FR-030 (PR #204) passes the record through unfiltered, so no layer in the chain owns the obligation. | spec/functional/FR-084-two-way-relation-coverage-report.md:212 |

## Verdict

Changes requested (medium). Also examined, with no gap found: FR-023-AC-1..4 and AC-9/10 map
onto FR-069-AC-39..41 and FR-081-AC-18. FR-023-AC-2's "names the value `medical`" maps onto
FR-069-AC-40's "naming `K` and the value `x`". FR-013's reverse per-classification view maps
onto FR-084-AC-21. A minor consistency note for the FR-030 author: FR-030 says the record is
on "the linked target", while FR-084 also puts it on the obliged `document`.

## Dispositions

Pass 1, head `5731e42`. No new findings from this method. The spec-objects-security FR-023
obligations now also have frontmatter-locator ACs (FR-069-AC-42..45). The FR-013 obligations
(carry the category list, carry no `level`, a reverse per-classification view) map onto
FR-084-AC-20, AC-22, AC-23 and AC-28.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | FR-084-AC-23 explicitly backs spec-objects-architecture FR-013-AC-5. With `record_keys: [regulatedCategories]` over a record stating `level: restricted`, the report contains no `level` key at any depth. AC-22 asserts the generic case, in which an unselected key appears nowhere in the report. quire-rs now owns the obligation, so quire-cli FR-030 passing `record` through unchanged is correct. |

- FND-001 after_excerpt (`spec/functional/FR-084-two-way-relation-coverage-report.md:229`): `FR-084-AC-23 | Backing agent-ix/spec-objects-architecture FR-013: with a relation declaring record_keys: [regulatedCategories] over a data-classification record that states level: restricted and regulatedCategories: [health], the report holds the regulatedCategories list and contains no level key at any depth. A second classification stating only level has no record in its entry.`

Pass-1 verdict: approved for this method.
