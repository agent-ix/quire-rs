---
id: SR-194
title: "scope-boundary review of FR-078 / Quoin FR-116 qualifier boundary (quire-rs#571)"
type: SpecReview
analysis: scope-boundary
scope: "agent-ix/quire-rs; spec/functional/FR-078-qualified-evidence-binding.md (diff origin/main...f7844ce)"
review_set: subset
---

## Summary

Ticket: IR-1006. Examined the allocation between Quire core (owner-neutral backing,
reconciliation), the Quoin `quoin-quire` bridge, the FR-116 qualifier and the acceptance host.
Quire core stays Quoin-free and the bridge invokes the audit itself, which is right. The new
owner-qualification paragraph reopens the forgery boundary the audit rule closed, and leaves
host admission unallocated.

## Verdict

**FAIL (changes requested)** - FND-001 is a soundness hole for the qualified channel.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | L152 says `qualify` SHALL "consume Quoin FR-116's typed premises and evaluated decision". That lets a caller hand in a precomputed positive decision, the same forgery L48-51 forbids for `AuditReport`/`healthy`; FR-116-AC-15/AC-18 require the bridge to use the owner result over admitted context, not a self-asserted one. It does not say the bridge invokes the FR-116 qualifier itself in the same Quoin workspace. | spec/functional/FR-078-qualified-evidence-binding.md:152-157 | wrong-requirement |
| FND-002 | medium | FR-116 L80-87 and CON-1 allocate authentication/admission of CI, manual and actor-authority observations to "the host". FR-078 names only a "caller" that reads the store (L46) and an "executable acceptance host" for exit codes (L69-72); it does not say which party admits observations for the IR-1006 fixtures or the IR-499 consumer, or how admitted vs transcribed context reaches `qualify`. | spec/functional/FR-078-qualified-evidence-binding.md:44-47,69-72 | missing-requirement |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | L202-211: qualify runs the FR-116 qualifier itself in the same workspace/feature; caller-supplied decisions, verdicts, cause lists and public constructors are forbidden. |
| FND-002 | fixed | L75-94 define the host as the qualify caller, allocate admission to it, and name the IR-1006 harness and the IR-499 RT strict-coverage gate as hosts. |
