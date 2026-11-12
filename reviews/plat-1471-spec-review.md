---
id: SR-20261103118159
title: "Base spec-review of FR-049 kind-typed references and FR-057-AC-11 (quire-rs PR 649)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs@a2e2b4c0; spec/functional/FR-049-verification-reference-integrity.md, spec/functional/FR-057-corpus-check-severity.md, spec/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-049
    type: references
  - target: ix://agent-ix/quire-rs/FR-057
    type: references
---

# Base spec-review of FR-049 kind-typed references and FR-057-AC-11 (quire-rs PR 649)

## Summary

Ticket: PLAT-1471. Spec-only diff `origin/main...a2e2b4c0`: FR-049 section "Kind-typed references" and FR-049-AC-10..17; FR-057 pack table, unconfigured-tier table, new-check default rule, FR-057-AC-11; spec/tests.md coverage rows, TC-2228..TC-2234, audit rows. Examined: FR-049 kind section, FR-049-AC-10..17, FR-057 tables and line 74 rule, FR-057-AC-4 and AC-11, FR-050 Document references bullet (context). Consumer context only: spec-artifacts-process FR-015-AC-6 (branch plat-1415-anomaly) needs a kind mismatch to "fail validation"; the posture-routed Strict error meets it.

`quire validate --scope .` on the three files exits 0. `quire validate --okf --scope .` output is byte-identical on origin/main and the branch (exit 0). Author points judged: (1) the FR-057 exception is sound in substance (the check is a narrowed dangling-trace-reference), but see FND-002 on wording and FND-001 on the AC it now contradicts; (2) see the failure-domain SR; (3) confirmed: FR-050 does not mention the new fields (FND-003).

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | FR-057-AC-4 says every check in the pack table keeps the tier it had before FR-057, "posture-routed for the four posture-routed checks". The pack table now lists `reference-kind-mismatch`, which had no pre-FR tier, and the unconfigured-tier table lists five posture-routed checks. AC-4 (TC-884) now contradicts the tables. Fix: scope AC-4 to the checks that existed before FR-057 (four posture-routed, three fixed) and leave the new check to AC-11, or count five and cite AC-11. | spec/functional/FR-057-corpus-check-severity.md:99,50,68 | wrong-requirement |
| FND-002 | low | Line 74 now reads "SHALL declare `warning` unless the table above enumerates it". That lets any future check escape the warning default by adding a table row, and the sentence then still claims to "confine the exception to the enumerated list". The parenthetical also tangles the sentence. Fix: keep the rule as written and name the one exception: "...SHALL declare `warning`. The one exception is `trace:reference-kind-mismatch`, a kind-narrowed `dangling-trace-reference` that shares its routing (FR-049-AC-12)." | spec/functional/FR-057-corpus-check-severity.md:74-75 | wrong-requirement |
| FND-003 | low | FR-050's Document references bullet lists the optional declaration fields (`expand_ranges`, `strip_annotations`; `status_column` in FR-050-AC-45) but not `kind_column`/`kind_targets`. The author notes the cross-reference edit did not apply. FR-049 does not say whether the kind fields change coverage (`target_ids`, backing) or that a declaration that omits them serializes identically, which is the AC-45 precedent. `DocumentReference` is `deny_unknown_fields`, so the loader contract is FR-050's. Fix: add one sentence to the FR-050 bullet: the two optional fields, defined in FR-049, apply to FR-049 validation only, leave coverage unchanged, and serialize identically when omitted. | spec/functional/FR-050-declarative-coverage-computation.md:42-50; spec/functional/FR-049-verification-reference-integrity.md:94-101 | missing-requirement |

## Verdict

Not mergeable yet: FND-001 (medium) is a direct contradiction between FR-057-AC-4 and the tables this PR edits. FND-002 and FND-003 are one-sentence fixes. The kind-typed design itself is clear: union first, kind subset second, no double finding, all-or-nothing load validation, and nothing hardcoded in the engine.
