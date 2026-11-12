---
id: SR-20261103118155
title: "spec-review (base) of FR-038-AC-11 (quire-rs PR 646)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs@07890bc3; spec/functional/FR-038-okf-bundle-validation.md (FR-038-AC-11)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-038
    type: references
---

# Spec review (base) of FR-038-AC-11 (quire-rs PR 646)

## Summary

Ticket: PLAT-1407. Base checklist review of the single added row FR-038-AC-11 (a `dangling-reference` finding carries the source document path in `BundleFinding.path`; its message names the edge type and missing target id). Read for consistency: FR-038 Description and AC-1..AC-10, FR-026 (edge stub sources, target id extraction), FR-025-AC-3, FR-057 (corpus-check severity), `spec/tests.md` audit table, and the current emitter `src/corpus/validate.rs:313-340` (which today passes `PathBuf::from(&edge.source)`, the source id, and already names target and edge type). Companions read only: quire-cli#167 (FR-014-AC-11) and spec-objects-safety#30 (FR-007-AC-3) are consistent with the AC's intent. Sub-analyses run: base and failure-domain (separate SpecReview). Assurance profile AP-201 (`review_policy.mode: require`, `operations: [spec-review]`) scopes "coverage collection, trace binding, row minting, diagnostics, and published schemas"; bundle-validation finding paths are a borderline fit under "diagnostics". This is a direct review; no enforced `ix-flow` review run was started because its final transition needs human acknowledgement.

`quire validate --scope . "spec/**/*.md"` (quire 0.40.0, engine 0.56.0) exits 3 with 10 failing documents, all under `spec/assets/` and `spec/assurance/` and untouched by this PR; FR-038 validates clean on its own. `make validate` (the repo's FR-050-AC-58 audit-table gate, a cargo example) was not run because this is a spec-only review; FND-001 is deduced from its code (`examples/spec_validate.rs:328-341`).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-038-AC-11 has no TC row and no audit-table row in spec/tests.md (FR-038 audit rows stop at AC-10 -> TC-2151); the FR-050-AC-58 audit-table check in `make validate` fails on a defined AC with no row, and CLAUDE.md requires 100% AC->TC coverage. AC-9/AC-10 cite their TC in the Verification cell; AC-11 says only `Test` | spec/functional/FR-038-okf-bundle-validation.md:107 |
| FND-002 | medium | Fixture says the finding's "path is `a.md`", but every other per-document BundleFinding carries the loaded `doc.path` (root-joined; existing tests assert `path.ends_with("draft.md")`); two implementers would differ on relative vs loaded path. State it as the source document's loaded path (as the other per-document findings carry), ending in `a.md` | spec/functional/FR-038-okf-bundle-validation.md:107 |
| FND-003 | medium | "yields one finding" is ambiguous: under Strict the same fixture also needs a known `type`, archetype conformance, a complete index with `okf_version`, and a `mitigates` verb allowed for the source type, or other findings appear. Say "exactly one `dangling-reference` finding" | spec/functional/FR-038-okf-bundle-validation.md:107 |
| FND-004 | medium | "the missing target id" is undefined for a dangling relative-path link: FR-026 says an unmatched relative destination "yields no `target_id`" (the resolver carries the raw destination string). Say the message names the target as harvested: the id, or the raw relative destination | spec/functional/FR-038-okf-bundle-validation.md:107 |
| FND-005 | low | The parenthetical "(error under `Strict`, warning under `Okf`)" restates the unmapped tier only; FR-057 lets `refs:dangling-reference` be promoted, demoted or suppressed. The path/message obligation should hold at whatever tier the finding is routed | spec/functional/FR-038-okf-bundle-validation.md:107 |

## Verdict

FAIL until FND-001 is fixed (add the TC row and the audit-table row, and cite the TC in the Verification cell). The AC's intent is right and matches the companion chain (quire-cli FR-014-AC-11, spec-objects-safety FR-007-AC-3); FR-038-AC-9/AC-10 and FR-026 do not contradict it. The message half is already met by the current emitter; only the path half is new behaviour.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | 6d279b6 adds TC-2227 (FR-038-AC-11, status 🚧 test not yet written), the audit row `FR-038-AC-11 -> TC-2227`, and cites `Test (TC-2227)` in the Verification cell |
| FND-002 | fixed | 6d279b6: "the source document's loaded path (the root-joined `doc.path` ...)"; fixture "whose path ends in `a.md`" |
| FND-003 | fixed | 6d279b6: "in an otherwise conformant bundle, yields exactly one `dangling-reference` finding" |
| FND-004 | fixed | 6d279b6: message names "the missing target as harvested", with the raw destination string for a dangling relative-path link |
| FND-005 | fixed | 6d279b6: default posture tier "unless FR-057 promotes, demotes or suppresses `refs:dangling-reference`, in which case the FR-057 tier applies and the path and message are unchanged" |
