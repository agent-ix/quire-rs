---
id: SR-20261103118131
title: "Gap analysis — PLAT-1299 semantic refusals carry their manifest location (quire-rs PR 606)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; spec/functional/FR-069-semantic-module-contract-at-load.md (FR-069-AC-14), spec/tests.md (TC-2081), src/semantic/contract.rs, tests/it/semantic_contract.rs"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-069", type: references}
---

## Summary

Ticket: PLAT-1299. Frozen PR head: `baa7cb0`. This is a planless gap analysis scoped to the PR diff. Plan completion was not assessed. `quire trace --id FR-069-AC-14` resolves to one structural claim, `tests/it/semantic_contract.rs:1390 refusals_carry_their_manifest_path` (TC-2081). The test passes. It asserts the exact `<code>: <path>: ` prefix for all five loci the AC names. Every new production line traces to FR-069-AC-14 or to the FR-069 Outputs clause.

## Verdict

**PASS** with one low gap.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The empty-path fallback (`<code>: <message>`) that FR-069 Outputs promises ("A refusal with no manifest locus omits the path segment") is not asserted by any test. The existing `schema-ref-*` tests only check `starts_with(code)`, so they pass on either branch. A regression to `<code>: : <message>` would go unnoticed. One direct unit assertion on `SemanticFailure::error(code, "", msg).reason()` would back it. | src/semantic/contract.rs:58-63 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-069-AC-14 | examined | Backed by TC-2081; all five named loci asserted |
| TC-2081 → FR-069-AC-14 binding | examined | Correct |
| FR-069 Outputs no-locus clause | examined | Unasserted (FND-001) |
| Production code with no owning requirement | examined | None |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | low | The "path appears once" check slices `r[0]`, not the reason that matched `prefix`. When a case yields more than one reason, it checks the wrong string, and `&r[0][prefix.len()..]` can mis-slice. Bind the matched reason from `find` instead. | tests/it/semantic_contract.rs:1449-1454 |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | 10a2506 asserts `SemanticFailure::error("semantic.x", "", "msg").reason() == "semantic.x: msg"` |
