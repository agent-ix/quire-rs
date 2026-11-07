---
id: SR-192
title: "base review of FR-078 qualified evidence alignment with Quoin FR-116 (quire-rs#571)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs; spec/functional/FR-078-qualified-evidence-binding.md (diff origin/main...f7844ce)"
review_set: subset
---

## Summary

Ticket: IR-1006. PR quire-rs#571 at f7844ce, diff `origin/main...HEAD`, one file. Compared against
Quoin `spec/functional/FR-116-owner-qualified-quire-evidence.md` at quoin origin/main 5af0ff2e.
The PR correctly removes the frozen SHA/version baseline (pin antipattern) and points the
qualification premises at FR-116 without narrowing any AC. It does not yet make the bridge
input contract, cause mapping and verification partitions agree with FR-116, so the qualifier
runtime, bridge and Quire fixtures would still each have to guess. Not ready to build as written.

## Verdict

**FAIL (changes requested)** - one high and four medium consistency/completeness defects. What
is right: no AC dropped or downgraded; AC-7 widened rather than narrowed; the measured-baseline
SHA/version text was deleted; no copied schema or vendored contract; waiver policy remains
blocked consistently with FR-116; Analysis-stays-Analysis and both-Clippy-lanes rule agree
with FR-116-AC-3 (IR FR-038-AC-195).

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | `qualify` input list (L40-54) omits FR-116's required host-admitted inputs: producer definition, FR-093 bounded reliance, actor authority, performed-act observation, performed source revision, tool/configuration context, execution association and current observation time; L46 says the caller assembles inputs from the Quoin store, which holds none of these (FR-116 L50-54, L80). | spec/functional/FR-078-qualified-evidence-binding.md:40-54,152-157 | wrong-requirement |
| FND-002 | medium | Strict table maps "Stale or behind-HEAD run" to `stale` (L232) while FR-116 L112/L183-186 define performed-revision != HEAD as `wrong-revision`; FR-078 L233 also says `wrong-revision` for wrong source revision, so the same input has two codes. | spec/functional/FR-078-qualified-evidence-binding.md:232-233 | wrong-requirement |
| FND-003 | medium | `qualification-rejected` row (L236) and AC-7/TC-1969 partitions (L303) omit FR-116 premises: bounded use (FR-093), execution/CI association, manual-reason, and missing-authority=`unevaluated` vs evaluated-excluded=`qualification-rejected` (FR-116-AC-6, AC-9). AC-7 now claims every FR-116 premise but its required partitions do not cover them. | spec/functional/FR-078-qualified-evidence-binding.md:236,277,303 | correct-requirement-no-evidence |
| FND-004 | medium | Unchanged Dependencies prose still says the audited-input handoff "must be confirmed with that owner before admission implementation" and producer guarantees "require an owner-reviewed contract" (L373-383), contradicting the new L109-112 statement that FR-116 specifies those premises. | spec/functional/FR-078-qualified-evidence-binding.md:373-383 | wrong-requirement |
| FND-005 | medium | Outputs (L161-164) and `record_binders` (L174-176) retain only "method, actor and qualification provenance"; FR-116 L93-97 positive decisions carry producer-definition id, bounded-decision and authority references, performed/current revisions, tool/configuration context, execution association and freshness basis. Which of these `record_binders` and trace expose to IR-499 is unspecified. | spec/functional/FR-078-qualified-evidence-binding.md:161-164,174-176 | missing-requirement |
| FND-006 | low | L109 calls it "The reviewed Quoin FR-116" while FR-116 L28 self-describes as a "draft owner contract, with implementation planned". | spec/functional/FR-078-qualified-evidence-binding.md:109 | wrong-requirement |
| FND-007 | low | spec/tests.md TC-1963..1966, TC-1969, TC-1981 status still reads "Blocked - Quoin owner qualification/CI carrier"; the FR now names the blocker as FR-116 implementation. Trace table not updated with the FR (spec-matrix rule). | spec/tests.md:906-912,924 | wrong-requirement |

## Assurance Context

AP-201 (spec/assurance/AP-201-detection-minting.md) is active and covers trace binding and
coverage interpretation, so it applies. It still uses the legacy `review_selection` key, which
this review does not interpret; analyses were chosen on merits: base, dependency, scope-boundary
(SR-192..194). `quire validate --scope . spec/functional/FR-078-qualified-evidence-binding.md`
exits 0 (module warnings only).

## Examined Units

FR-078 statement and Inputs/Host/Owner-qualification/Outputs/Strict sections; FR-078-AC-1..AC-20;
FR-078-CON-1..CON-4; Verification Plan partitions; Dependencies table and prose; spec/tests.md
rows TC-1963..TC-1982. Context only: Quoin FR-116 full text.
