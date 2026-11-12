---
id: SR-20261103118160
title: "Gap analysis — PLAT-1413 coverage_matrix requirement criticality (quire-rs PR 648)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs@c9711bd; spec/functional/FR-082-coverage-matrix-criticality.md, spec/functional/FR-050-declarative-coverage-computation.md (AC-47, AC-48, AC-51, AC-58), spec/functional/FR-053-obligation-record.md, spec/tests.md, src/coverage.rs, src/coverage/reconcile.rs, tests/it/coverage_matrix.rs"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-082", type: references}
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-050", type: references}
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-053", type: references}
---

## Summary

Ticket: PLAT-1413. PR head `c9711bd`. Plan completion: not assessed. Planless audit over the computed matrix (`quire matrix --format tsv`, quire 0.40.0 / engine 0.56.0) and `examples/spec_validate.rs` (the `make validate` check) run at head. Read semantically, the four new tests assert FR-082-AC-1..9 correctly against real output. But the computed matrix reports all nine FR-082 criteria `untagged`: each tag raises `criterion-tagged-outside-row` ("no matrix row cited by that test references it ... This tag does not provide backing"). The repo declares matrix rows, so a criterion tag binds only through a TC row (PLAT-1392). `spec_validate` fails with 9 FR-050-AC-58 audit-table errors, one per FR-082-AC-n "declared by functional/FR-082-coverage-matrix-criticality.md but absent from the AC → TC Coverage Audit table". This is pre-existing on main since the spec PR #645, which added FR-082 without matrix rows, and this PR does not close it. This PR is the one that mints the tests, so this PR is where the rows belong.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-082-AC-1..9 have no Test Case rows and no AC → TC Coverage Audit rows in spec/tests.md, and the new tests carry no TC id. As a result, the computed matrix shows all nine criteria `untagged`: the PR's tests back nothing. `make validate` (part of `make ci`) fails FR-050-AC-58 with 9 errors. Fix: mint TC ids (the next free ids after TC-2227) with Test Cases rows naming FR-082-AC-n. Add an audit row per FR-082-AC-1..9, and add the TC id to each `#[trace(...)]`, as in the FR-081 tests (`#[trace("TC-2167", "FR-081-AC-1", ...)]`). Then re-run `quire matrix` (FR-082 tagged) and `spec_validate` (0 audit errors). | spec/tests.md:1410, tests/it/coverage_matrix.rs:1813-1976 |
| FND-002 | low | Criticality is exercised only on the legacy `obligation::derive` path. The FR-078 explicit-evidence path (`compute_with_evidence`, where obligations come from the SelectedInventory) builds the same `requirements[]` entries, but no test asserts that `criticality` is carried there. FR-082-AC-1 does not limit itself to one path. Add one evidence-mode assertion. | src/coverage/reconcile.rs:56-67, src/coverage/reconcile.rs:923-937 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-082-AC-1 | examined | Test asserts `low`/`medium`/`high`; untagged in matrix (FND-001) |
| FR-082-AC-2 | examined | Key absence asserted; untagged (FND-001) |
| FR-082-AC-3 | examined | Literal `unassigned` present; untagged (FND-001) |
| FR-082-AC-4 | examined | Mixed corpus asserted; untagged (FND-001) |
| FR-082-AC-5 | examined | Four relationship verbs, no inheritance; untagged (FND-001) |
| FR-082-AC-6 | examined | `High` and `3` carried verbatim; untagged (FND-001) |
| FR-082-AC-7 | examined | Both levels independent; untagged (FND-001) |
| FR-082-AC-8 | examined | Files byte-identical; untagged (FND-001) |
| FR-082-AC-9 | examined | Two runs byte-identical JSON; untagged (FND-001) |
| FR-082-CON-1 | examined | No enumeration held; method-without-symbol |
| FR-050-AC-58 | examined | Violated (FND-001) |
| FR-050-AC-48 | context_only | Entry shape extended additively |
| FR-050-AC-51 | context_only | Determinism carried by AC-9 test |
| FR-053-AC-7 | context_only | Obligation criticality untouched by the diff |

## Verdict

FAIL: not mergeable until FND-001 is fixed. The implementation is complete against FR-082 with no stubs or coverage inflation, and no production code lacks an owning requirement. But the criteria are unbacked in the computed matrix, and the FR-050-AC-58 audit check is red.
