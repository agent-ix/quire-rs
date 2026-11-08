---
id: SR-16311
title: "PLAT-321 audit-table completeness gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; src/audit_table.rs; examples/spec_validate.rs; tests/it/audit_table.rs; spec/tests.md; PLAT-321; PR598"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/FR-050", type: references}
---

# PLAT-321 audit-table completeness gap-analysis

## Summary

Ticket: PLAT-321. Planless, change-scoped gap analysis of PR agent-ix/quire-rs#598
(head 6fe84930). Plan completion: not assessed. The reviewer checked all 96 added
audit rows mechanically in both directions. Every TC in an added row traces that
AC in its own Traces cell. Reverse direction: every TC that traces an added AC
appears in its row, except one (see the spec-review SR). The reviewer also checked
tags in source and `quire coverage --json` (quire 0.36.2, engine 0.50.2) for which
TC ids are minted.

## Examined Units

- **FR-050-AC-58** (examined), spec/functional/FR-050-declarative-coverage-computation.md:292. The audit-table completeness criterion.
- **TC-2006..TC-2012** (examined), spec/tests.md:1148-1154. The new test cases.
- **TC-970..TC-981** (examined). Re-traced from FR-063 to FR-074. Tags in src/plain_language.rs and tests/it/plain_language.rs already name FR-074 ACs, so the re-trace is correct.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | [coverage] The check works in one direction only. It finds declared criteria with no audit row, but it does not find a coverage-row id or an audit-row id that no document defines. Reproduced: an audit row `FR-001-AC-99` gives 0 gaps. A coverage row `AC-1..12` where AC-10 is undefined passes once someone authors a phantom `FR-xxx-AC-10` row. The CR-196 note says this class (FR-011 and FR-074 coverage rows declaring undefined ids, FR-063 audit rows mislabelled with FR-074 test cases) was found by hand in this change. Nothing stops it coming back. Fix: report a coverage-declared or audit-mapped id that no requirement document defines, with tombstones counted as defined. | src/audit_table.rs:220-245; spec/functional/FR-050-declarative-coverage-computation.md:1102 |
| FND-002 | medium | [trace] TC-2006, TC-2007, TC-2008 and TC-2009 are marked ✅, but no test carries their TC id. The backing tests carry AC ids only (src/loader/paths.rs:266, src/registry.rs:1689 and 1745, tools/quire-bench/tests/clean_checkout.rs:63-118, tests/it/coverage_rollup.rs:3427 and 3610, tests/it/corpus_cases.rs:2231). The repo convention is `#[trace("TC-n", "AC")]`. All seven new TCs, TC-2012 included, also sit in the `### PLAT-1181: guarded Rust implements attributes (planned)` table, whose preamble says its rows are not runtime or coverage evidence. `quire coverage` mints no target from that table: TC-2006 and TC-2012 are absent from `minted_targets`, while TC-976 is present. So their ✅ claims cannot be checked. Fix: move TC-2006..2012 into the main Test Case Summary table, and add the TC id to each backing test's trace tag. | spec/tests.md:1120-1154; src/loader/paths.rs:266; src/registry.rs:1689 |
| FND-003 | low | [coverage] Some AC-58 clauses have no test. No test exercises the `(RETIRED` tombstone. No integration test produces a named gap from a leading table-cell definition (only the bold path does). No test covers a stakeholder-style id, which would have caught FND-001 of the code-review SR. Fix: add these cases to TC-2012. | tests/it/audit_table.rs:35-104 |

## Verdict

Changes requested. Mechanically, the 96 added audit rows and the TC-970..981
re-trace are correct. TC-2010 (FR-050-AC-54) and TC-2011 (FR-072-AC-9) are
honestly 🚧: no test is tagged with either AC. TC-2006..2009 describe real tests
(env-root order, unknown-manifest-key refusal, require_clean, retirement), but
they claim ✅ without their own tag, and they sit in a table that coverage does
not read.

## Dispositions

Round 1, at head 8806d52.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `AuditFinding::Phantom` now reports a coverage-range id or an audit-row id that no document defines. Constructed `FR-001-AC-99` is now reported. The PR tree reports none. |
| FND-002 | fixed | The new test cases are renumbered TC-2050..2056 and sit in the main Test Case Summary table (spec/tests.md:1117-1123), above the PLAT-1181 planned table. Every backing test now carries its TC id: src/loader/paths.rs:266, src/registry.rs:1689 and 1745, tools/quire-bench/tests/clean_checkout.rs, tests/it/coverage_rollup.rs, tests/it/corpus_cases.rs:2231, tests/it/audit_table.rs. TC-2054 and TC-2055 stay 🚧. |
| FND-003 | fixed | tests/it/audit_table.rs now covers StR deletions, table-cell definitions, `(RETIRED`, `Removed:` and bold tombstones, a live `Withdrawn` sentence, phantom ids and unreadable cells, all tagged TC-2056. |
