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
