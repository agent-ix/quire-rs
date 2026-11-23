---
id: SR-321
title: "Base spec review of PLAT-1831 outputCatalog producers (quire-rs PR 713)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs; spec/functional/FR-073-generic-clause-sets.md (PR 713 diff only, head 5a7ac5dde4020113d16ba5237151e72bb15d9a1c)"
review_set: subset
date: 2026-11-23
---

## Summary

Ticket: PLAT-1831. Base checklist over the new `producers` rules in FR-073 and FR-073-AC-7..AC-10. Checked AC id format and sequence, `quire validate --scope . spec/functional/FR-073-generic-clause-sets.md` (exit 0), the hash/digest/pin antipattern rule, compatibility-layer wording, and the consumer contract in quoin FR-211 on quoin origin/main. AssuranceProfile AP-201 (`review_policy.mode: require`, `operations: [spec-review]`) governs coverage collection, trace binding, row minting and published schemas; clause-set catalog shape is outside its decision boundary, so it does not apply and no enforced run is required.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

Clean. AC-7..AC-10 continue the AC-1..AC-6 sequence and are all Test-verified. The spec matches quoin FR-211 exactly: the member is named `producers`, it is optional, when present it is a non-empty list, and the kind set is open (quire checks shape only). Absent `producers` stays absent (AC-8), which is what FR-211 needs to report `no-producer`. No hash, digest, pin, count or version assertion is added. No compatibility layer: the text explicitly says there is no default list and no other field stands in for `producers`. The ACs are implementable against src/clauses.rs: `ExpectedOutput` (line 82) gains `Option<Vec<String>>` with `skip_serializing_if`, and the existing catalog loop in `ClauseSet::validate` (line 257) gains the shape checks; `load_clause_set` already calls `validate`, and no `deny_unknown_fields` or clause-set input schema needs to change.
