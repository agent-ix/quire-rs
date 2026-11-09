---
id: SR-20261103118146
title: "Gap analysis — PLAT-277 F4 presentation-only archetypes rows (quire-rs PR 631)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; spec/functional/FR-013-archetype-loader.md, spec/tests.md, src/loader/manifest.rs, src/loader/mod.rs"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-013", type: references}
---

## Summary

Ticket: PLAT-277 (finding F4). Frozen PR head: `91cdf225b0ceaf29646f4f59a88e9ecad2043ea4`.

`quoin matrix` (quoin 0.29.1) with the fresh origin/main process and iso modules reports no coverage_matrix criteria, because the module declares no `obligations:` source. A manual criterion-to-test check was therefore done with `quire trace --id`:

- FR-013-AC-20 has one structural claim: `tc2211_same_module_presentation_row_is_merged_not_a_duplicate` (src/loader/mod.rs:2821).
- FR-013-AC-21 has one structural claim: `tc2212_presentation_only_row_is_not_a_type` (src/loader/mod.rs:2846).

Both tests were run against the origin/main loader, with only `src/loader/manifest.rs` reverted, and both failed. They pass on the PR. So the tests detect the defect and are not vacuous.

The grammar_ref keep path is covered indirectly: TC-981 (tests/it/plain_language.rs) loads an `archetypes:` row with only `grammar_ref` and expects it to resolve. No stub or unowned code was added.

## Verdict

**PASS with gaps.** The new criteria are backed by tests that fail without the fix. Two coverage gaps remain. The regression this PR introduces on the retired-field path (code-review SR FND-001) passed every test, because no test checks the FR-031-AC-5 refusal for an `archetypes:` row. The `object_types:` half of FR-013-AC-20 is claimed but not exercised.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | No test checks FR-031-AC-5 (retired-field refusal) for an `archetypes:` row. The existing retired-field tests use `artifact_types:` only. That is why silently dropping a `required_sections`/`template_ref` presentation row passed the whole suite. | src/registry.rs:1206-1216, src/registry.rs:1246-1256 |
| FND-002 | low | FR-013-AC-20 claims the `object_types:` case ("or `object_types:`") and implies the same-module case where both rows carry a validation part, but TC-2211 exercises only `artifact_types:` with a presentation row. | src/loader/mod.rs:2817-2840 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-013-AC-20 to TC-2211 binding | examined | Correct; fails without the fix; FND-002 for the untested branch |
| FR-013-AC-21 to TC-2212 binding | examined | Correct; fails without the fix |
| FR-031-AC-5 coverage for the `archetypes:` section | examined | FND-001 |
| grammar_ref keep path (TC-981) | examined | Covered |
| spec/tests.md TC-2211/TC-2212 rows and mapping rows | examined | Consistent with the tags |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1 (25a0495): TC-2213 checks that a `required_sections` `archetypes:` row gives an ArchetypeLoadFailure and that the allowed_links/roles rows stay registered. It fails on the round-0 predicate and passes at 25a0495. |
| FND-002 | fixed | Round 1 (25a0495): TC-2211 now covers the `object_types:` case (no duplicate) and the same-module row with its own schema (DuplicateArchetype naming the module twice). |
