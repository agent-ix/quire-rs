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

## New findings (disposition pass 1)

Checked at 5435ed19fe0d90c084a5131e7aa0a39b7af2a257. Line numbers refer to that head.

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-008 | medium | The positive-fixture rule (L82-89) requires CI observations "admitted from the CI system's own record of the run" and forbids harness-authored observations, but does not say whether the IR-1006 harness reads that record live (network, contrary to hermetic tests) or from a retained captured CI record, nor what admission mechanism makes a captured record "admitted" rather than "transcribed" (L66-73). Two fixture builders would diverge; a captured file is indistinguishable from a transcription under the stated rule. | spec/functional/FR-078-qualified-evidence-binding.md:66-73,82-89 | missing-requirement |
| FND-009 | low | Normative text still narrates PR history: "not ... a capability delivered by apparatus-intake PR #689" (L144-145, a Quoin PR cited without repo, reads as quire-rs#689), "not APIs already delivered by this spec PR" / "this PR does not claim" (L121-124), "publication of a draft API or apparatus-intake repair" (L416-417). These belong in the Change record or the PR body; in requirement text they rot and carry no testable meaning. | spec/functional/FR-078-qualified-evidence-binding.md:121-124,144-145,416-417 | wrong-requirement |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Input table L52-64 lists all eleven inputs with source and admitting party; store vs host-admitted split at L45-47. |
| FND-002 | fixed | L307 stale now owner validity/supersession only; L308 performed revision != HEAD is wrong-revision. Matches FR-116. |
| FND-003 | fixed | L311/L312 split unevaluated vs qualification-rejected incl. FR-093, CI/manual; AC-7 (L353) and TC-1969 partitions (L379) cover each premise. |
| FND-004 | fixed | Dependencies L454-464 rewritten; handoff is the input table and bridge invocation; no unconfirmed-owner gate remains. |
| FND-005 | fixed | Claim field table L219-231; record_binders, trace and strict decision expose the same set to IR-499 (L233-237, L249-250). |
| FND-006 | fixed | L158-159 now "a draft owner contract with implementation planned". |
| FND-007 | fixed | spec/tests.md TC-1963..1966, 1969, 1981 now cite the FR-116 qualifier and bridge build; TC-1969 title widened. |
| FND-008 | fixed | L82-97 specify the admission mechanism: a live authenticated provider read by run/attempt/job identity; a stored/cached/copied record is transcribed; an unreachable provider is unevaluated; a manual attestation is read from the named location. Fixture roles L102-111; partitions L402-403. |
| FND-009 | fixed | The PR-history narration is removed at L143-146, L166-169 and L440-441; the Change record is kept. |
| FND-010 | fixed | At d51d47f, L115-122 and the TC-1964/1965 row (L416) say the positive CI partition runs only with a live read; without provider access it is reported not executed, the TC does not pass and AC-2/AC-3 are unverified. The unreachable and stored-copy cases are separate refusal partitions that verify only their refusal outcome. |
| FND-011 | fixed | At d51d47f, the producer-definition row (L57) and the term paragraph (L229-234) list only FR-116 L71 members. CI provider identity and attestation retention location are host-configured inputs outside the producer definition (L66-71) and are recorded in admission references (L93-95, L100-104). Actor authority is checked against the owner-authorized scope (L59). |

## New findings (disposition pass 2)

Checked at 940d147f7470a8e0e6c1b90335425162d0f1a50e. Line numbers refer to that head.

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-010 | medium | Environment-gated positive CI fixtures "report `unevaluated` and never pass" where the provider is unreachable (L106-109), and the TC-1964/TC-1965 partition row lists "provider unreachable (`unevaluated`)" as an expected outcome (L403). The spec does not say that a run without provider access leaves the positive "qualifies" partition not executed and the TC not passing. A harness that asserts the unreachable branch's `unevaluated` result goes green on every CI-less machine, so AC-2/AC-3 read as verified without the positive path ever running. | spec/functional/FR-078-qualified-evidence-binding.md:106-109,403 | correct-requirement-no-evidence |
| FND-011 | medium | The producer-definition input (L57) now requires two members: "the CI provider whose authenticated API admits its CI observations" and "where its manual attestations are retained". FR-116's producer definition (FR-116 L71) defines neither. L47-50 says each row is a member "that Quoin FR-116 specifies", so FR-078 is extending the Quoin owner contract from quire-rs. An FR-116 implementer will not supply these members, and FR-078's admission rule (L82-97) then leaves every CI and manual premise `unevaluated`. Fix by amending FR-116, or by moving provider and retention location to the host. | spec/functional/FR-078-qualified-evidence-binding.md:47-50,57,82-97 | wrong-requirement |
