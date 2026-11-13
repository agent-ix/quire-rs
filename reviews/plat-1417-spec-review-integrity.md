---
id: SR-20261113045747
title: "integrity review of FR-086 design-element join and FR-058 CR-416 (quire-rs PR 668)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-rs@8f9a5b9; spec/functional/FR-086-design-element-join.md; spec/functional/FR-058-upward-trace-completeness.md; context FR-026, FR-041, FR-084, FR-085, src/corpus/resolve.rs, src/corpus/required_relations.rs, src/corpus/relation_report.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-086
    type: references
  - target: ix://agent-ix/quire-rs/FR-058
    type: references
---

# Integrity review of FR-086 and FR-058 CR-416 (quire-rs PR 668)

## Summary

Ticket: PLAT-1417. Checked the diff against FR-026, FR-041, FR-084, FR-085 and the code on main (`src/corpus/resolve.rs`, `required_relations.rs`, `relation_report.rs`). The fragment doubt is confirmed. `extract_target_id` keeps the `#fragment` in the target id, so `FR-001#FR-001-AC-1` (bare or `ix://`) is Dangling. This was reproduced with quire 0.44.0 (engine 0.59.1): `dangling reference 'FR-001#FR-001-AC-1'`. Relative `.md` links drop the fragment before the reference is recorded. Today's live `required_relations` (safety and security `mitigates`, incoming, `to: []`) list no verb that has an inverse, so CR-416 changes no current output. The changes below concern how CR-416 is defined and how FR-086 fits FR-084 and FR-085.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | FR-086 says "an edge to a criterion resolves, by FR-026, to the requirement document". FR-026 has no rule that strips a fragment, and main does not strip one: `extract_target_id` returns `FR-001#FR-001-AC-1`, the lookup misses and the edge is Dangling (reproduced). The relative-link harvest keeps only the id, so the fragment is lost there. FR-086-AC-8 cannot pass without an FR-026 amendment. That amendment changes resolution for every fragment-bearing reference in the ecosystem (for example `ix://agent-ix/ecaz/spire-leaf-v2#segment_tuple`, Dangling today) and must keep the fragment in `Edge.references`. Fix: add an FR-026 CR with its own AC and state that blast radius. | spec/functional/FR-086-design-element-join.md (Criteria, AC-8); src/corpus/resolve.rs:490-494, 375-382 | missing-requirement |
| FND-002 | high | FR-086 adds `"criteria": ["<id>", ...]` to a resolved `linked` entry. Under FR-084 Serialization, that entry already carries `criteria` as the counts object `{"backed","unbacked","not_measured"}` (RowDocument flattened into the entry). Two different values would share one JSON key. FR-086-CON-2 ("no field beyond FR-084's except the `criteria` list") makes the clash binding. Fix: use a distinct key (for example `realized_criteria`) and amend CON-2 and AC-8. | spec/functional/FR-086-design-element-join.md (Criteria, AC-8, CON-2); spec/functional/FR-084-two-way-relation-coverage-report.md (Serialization); src/corpus/relation_report.rs:112-127, 180-190 | wrong-requirement |
| FND-003 | high | FR-086-AC-7 lists an edge from a wrong-kind element in `kind_mismatch`. FR-086-AC-1 implies the element authors the realizing edge, so the relation on the requirement is `incoming`. FR-085 says that under an `incoming` relation a wrong-kind source only leaves the relation unsatisfied and yields no kind mismatch. relation_report.rs does the same: its incoming branch skips such sources silently. FR-086 also claims everything FR-084 decides is unchanged. Fix: either restrict AC-7 to an outgoing declaration (requirement-authored inverse) or amend FR-084 and FR-085 to report wrong-kind incoming sources. | spec/functional/FR-086-design-element-join.md (AC-7); spec/functional/FR-085-cross-repository-edge-resolution.md (Kind of a resolved target); src/corpus/relation_report.rs:525-541 | wrong-requirement |
| FND-004 | medium | CR-416 defines the counterpart of `V` as "`V`'s declared inverse, or the forward verb whose declared inverse is `V`". It ignores FR-041's precedence rules: a label that is also a forward key is governed by its own entry, and a duplicate inverse is first-wins with `DuplicateInverseEdge`. Real vocabularies hit both: in one module `contains` and `aggregates` both declare `part_of`, and iso declares `satisfied_by` with inverse `satisfies`. "The forward verb" is singular, but there can be two. Fix: define the counterpart through `Registry::inverse_index` (FR-041-AC-1) so the precedence carries over. | spec/functional/FR-058-upward-trace-completeness.md (Authored inverse verbs) | wrong-requirement |
| FND-005 | medium | "A verb with no declared inverse is matched exactly as before", and the AC-24 clause "a relation listing only verbs with no declared inverse is byte-identical", both contradict the counterpart rule. A listed verb that is itself an inverse label (for example `satisfies`) declares no inverse, yet it gains the counterpart `satisfied_by`. That is a behaviour change the byte-identity clause says cannot happen. Fix: say "a verb with no counterpart" in both places. | spec/functional/FR-058-upward-trace-completeness.md (Authored inverse verbs, AC-24) | wrong-requirement |
| FND-006 | medium | FR-058-AC-24 leaves three questions open. (1) Under which `direction` an inverse-authored wrong-kind edge yields `trace:target-kind-mismatch` (FR-085 restricts it to outgoing/either). (2) Which document the finding names as source and whose `repo` it carries: authored `b` (FR-041 reports authored) or normalized `a` (FR-085-AC-19). (3) Whether an inverse-authored edge toward an absent document can satisfy an `incoming` relation with `to: []`. This is a case main never has, since `edges_to` holds resolved edges only. Fix: answer each and pin it in the AC. | spec/functional/FR-058-upward-trace-completeness.md (AC-24); spec/functional/FR-085-cross-repository-edge-resolution.md | missing-requirement |
| FND-007 | low | FR-086 extends FR-084, whose FR-084-CON-2 says the report SHALL NOT introduce a manifest key, and adds `traceability.design_relations`. The key belongs to FR-086, not to FR-084's report, but the text never says so. Fix: one sentence in FR-086 scoping FR-084-CON-2. | spec/functional/FR-086-design-element-join.md (The declaration); spec/functional/FR-084-two-way-relation-coverage-report.md (CON-2) | missing-requirement |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-008 | medium | CR-417, CR-418 and FR-086 key on a "declared acceptance-criterion trace target", but `trace_targets` entries carry no kind. They are module-named (`acceptance-criterion`, `test-case`, `requirement-document`, ...; `TraceTarget` has only name, archetype, section, source, required, evidence). The engine can identify such a target only by its name, which FR-058-CON-2 and FR-086-AC-13 forbid, or else by taking every trace target. Taking every target would make an edge to a `TC-…` id read as an edge to the TestMatrix document. Fix: define the minting set in engine-neutral terms, for example "a trace target named by an `obligations:` source (FR-053 `target:`)", which spec-artifacts-process already declares for both criterion targets. Pin it in FR-058-AC-27/28 and FR-038-AC-12. | spec/functional/FR-058-upward-trace-completeness.md (Edges to a criterion, AC-27, AC-28); spec/functional/FR-038-okf-bundle-validation.md (AC-12); spec/functional/FR-086-design-element-join.md (Criteria, AC-15); src/traceability.rs:1010-1040 | wrong-requirement |
| FND-009 | medium | New rule: "When a relation lists both labels of a pair, every edge is read under its own label and nothing is normalized." It makes `edges` non-monotonic. Take a relation on `b` listing only `V` with `direction: incoming`: it is satisfied by `(b, W, a)` (normalized to `(a, V, b)`). Add `W` to `edges` and the same document fails, because the outgoing `W` edge is ignored under `incoming`. That contradicts FR-058-AC-2 ("any one of the declared edges satisfies") and this section's own purpose, that either end may author the edge. Fix: always normalize the counterpart and count the pair once, or have load reject a relation listing both labels. | spec/functional/FR-058-upward-trace-completeness.md (Authored inverse verbs, AC-25) | wrong-requirement |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1: `#fragment` addressing was dropped (CR-417 note records why) and FR-026 is unchanged. Criterion edges are now read for relation matching only by FR-058 CR-417; new findings on that scope are FND-008 and FND-009. |
| FND-002 | fixed | Round 1: the key is renamed `realized_criteria`, with an explicit note that `criteria` is FR-084's counts object; CON-2 and AC-8 updated. |
| FND-003 | fixed | Round 1: FR-086-AC-7 now states that under an `incoming` relation a wrong-kind source yields no `kind_mismatch` (FR-085-AC-25), and only `outgoing`/`either` list one. |
| FND-004 | fixed | Round 1: the counterpart is now defined by the merged registry's inverse index (FR-041-AC-1), with forward-key-wins and first-wins carried over; FR-058-AC-24 pins both. |
| FND-005 | fixed | Round 1: AC-24's byte-identity clause now covers "verbs the inverse index relates to nothing", and a verb that is an inverse label gains its forward verb. |
| FND-006 | fixed | Round 1: the bullets under Authored inverse verbs and FR-058-AC-26 now fix the direction rule, the source and `repo` named (the normalized source), and the `to: []` dangling case. |
| FND-007 | fixed | Round 1: FR-086 now scopes FR-084-CON-2 to the report and makes `design_relations` the join's own key. |
| FND-008 | fixed | Round 2 (0a89ebd): a criterion target is now "a trace target named by an `obligations:` source" (FR-053 `target:`), with no engine name. This is applied throughout FR-058 CR-417, FR-038 (body, CR-418, AC-12) and FR-086 (Criteria, AC-15). AC-28 pins that a test-case target is never read this way. |
| FND-009 | fixed | Round 2: the counterpart is now always normalized, even when both labels are listed ("adding a verb to `edges` never removes an edge a relation accepted"). FR-058-AC-25 asserts that listing `V` and listing `V`+`W` give the same result. |
