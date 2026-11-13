---
id: SR-20261113203418
title: "Spec review (integrity) - PLAT-1689 FR-040 / FR-057 amendments (quire-rs PR 678)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-rs@1e5ba09; spec/functional/FR-040-object-edge-vocabulary.md, spec/functional/FR-057-corpus-check-severity.md"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-040", type: references}
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-057", type: references}
---

# Spec review (integrity) - PLAT-1689 FR-040 / FR-057 amendments (quire-rs PR 678)

## Summary

Ticket: PLAT-1689. Scope: the FR-040 and FR-057 hunks of `git diff origin/main...HEAD`. Checked
them against the rest of FR-040 and against FR-041 and US-015, which restate Tier-2 behaviour.
Sub-analysis: integrity (consistency, completeness, atomicity). EARS and the other sub-analyses
were not run: no new requirement statement was added, only ACs and prose.

## Verdict

**FAIL: 1 high, 2 medium, 1 low.** Did FR-040 skip targets without `object:` by design? Yes.
The old Tier-2 prose skipped "when the target document declares no `object:` (no object type to
constrain)", and FR-040-AC-9 asserted that skip. The ticket's second AC explicitly asks to change
it, so the change is in scope. But AC-9 was left asserting the old rule. AC-12..14 are atomic and
testable. Using the document's `type:` as its archetype matches how FR-040 already resolves the
source side (`T` = artifact archetype). The FR-057 table row and AC-4 amendment agree with AC-13.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-040-AC-9 still says a Tier-2 mismatch "yields a warning" and that "Target checking is skipped for `"*"` lists, targets with no `object:`, and dangling (cross-repo) targets". That contradicts AC-12 (no-`object:` targets are judged by archetype name), AC-13 (an error under Strict) and the amended Tier-2 prose. | spec/functional/FR-040-object-edge-vocabulary.md:226 |
| FND-002 | medium | FR-041's Tier-2 prose and US-015-AC-3 were not amended. FR-041 still says a mismatch yields the `DisallowedEdgeTarget` **warning**, that checking is "skipped for ... targets with no `object:`", and that "Inverse handling never errors". US-015-AC-3 describes the target only by "object type — or any role", without the archetype name. | spec/functional/FR-041-authorable-inverse-edges.md:84-100, spec/usecase/US-015-object-edge-vocabulary.md:48-51 |
| FND-003 | medium | The amended Tier-2 rule does not say which edges it judges or what happens to an unresolved `object:`. The corpus edge set includes resolver-synthesized `references` edges from body `ix://` links, relative links and re-harvested frontmatter URIs. Those are now kind-checked and fail under Strict (code-review FND-001, reproduced on a spec-objects-architecture valid fixture). A target whose `object:` is unknown is now judged on `type:` alone (code-review FND-003). Two implementers could read the rule differently on both points. | spec/functional/FR-040-object-edge-vocabulary.md:175-193 |
| FND-004 | low | Wording and layout. The section heading "Validation tiers (advisory)" is now inaccurate for Tier 2. AC-12..14 sit between AC-10 and AC-11 in the table. "its `type:` as the manifest declares it" reads as a manifest field; it is the document's frontmatter `type:` value naming a manifest archetype. | spec/functional/FR-040-object-edge-vocabulary.md:157, spec/functional/FR-040-object-edge-vocabulary.md:161, spec/functional/FR-040-object-edge-vocabulary.md:227-231 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-040 "Resolution and target matching" amendment | examined | FND-004 |
| FR-040 Tier-2 prose amendment | examined | FND-003 |
| FR-040 tier summary paragraph | examined | Consistent with AC-13 |
| FR-040-AC-9 | examined | FND-001 |
| FR-040-AC-10 (amended) | examined | Consistent |
| FR-040-AC-12 | examined | Atomic, testable |
| FR-040-AC-13 | examined | Atomic, testable |
| FR-040-AC-14 | examined | Atomic, testable |
| FR-057 pack table row, FR-057-AC-4 | examined | Consistent with AC-13 |
| FR-041 Tier-2 prose, US-015-AC-3 | context_only | FND-002 |
