---
id: SR-2026112316341702
title: "failure-domain review of the per-declaration load drop (quire-rs PR 712)"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-rs@50c28286; spec/functional/FR-058-upward-trace-completeness.md; spec/functional/FR-096-flow-rules.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-058
    type: references
---

# Failure-domain review of the per-declaration load drop (quire-rs PR 712)

## Summary

Ticket: PLAT-1804. Looks for failure modes the new drop rule leaves unstated:
cascades between dependent declarations, surfaces other than `validate_bundle`
that read a now-partial model, and multi-member and ordering behaviour of
the new finding. Oracles: `src/loader/mod.rs` 1415-1574,
`src/loader/flow_rules.rs` 20-96, `src/corpus/validate.rs` 242-330,
FR-050 (coverage, "no traceability model" exit).

Examined, clean: the remediation-to-rule cascade the section states; the
obligations-source drop for an unreadable bound target, which matches
`mod.rs:1452-1491` with the failure naming the first-wins target's module.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | [soundness] The guarantee covers only bundle validation, but coverage reads the same model. Before this change, a failing declaration emptied the model, so `quire coverage` hit FR-050's "no traceability model declared" distinct exit: loud. After it, the model is partial. Scenario: an `obligations` source whose bound target cannot be read is dropped; `quire coverage` / `quire matrix` computes over the remaining sources and reports their criteria complete, with nothing marking that a source is missing. Nothing in the spec makes coverage fail or flag the drop. The engine has no such check today (the only non-test reader of `registry.failures()` is `assurance.rs:456`); quire-cli may check, which this review could not confirm. Fix: state that the coverage report (FR-050) carries the drops, or refuses, when the registry recorded one, and add an AC on the AC-38-style selection for coverage. | spec/functional/FR-058-upward-trace-completeness.md:300-306; spec/functional/FR-050-declarative-coverage-computation.md:280-284; src/assurance.rs:456 | missing-requirement |
| FND-002 | medium | [ambiguous] Cascades are stated for one edge only (a rule naming a dropped remediation). Unstated: (a) a remediation whose `sets` value is valid for one naming rule's `subject` and invalid for another's. Today `validate_flow_declarations` fails the remediation's module for that pair (`flow_rules.rs:74-91`); the spec does not say whether the remediation, the rule, or both are dropped. (b) An `obligations` source bound to a `trace_targets` entry that is itself dropped. (c) A flow rule whose condition reads a dropped `carried_fields` entry or reserves a dropped relation's `trace:<check>`. Fix: add one general rule (a declaration that names a dropped declaration is dropped, naming it) and decide case (a) explicitly. | spec/functional/FR-058-upward-trace-completeness.md:290-291; src/loader/flow_rules.rs:48-93 | missing-requirement |
| FND-003 | low | [ambiguous] Where the finding lives and how it is ordered is unstated. A drop has no source document, but `BundleFinding` carries a `path`, and FR-058-AC-9 orders findings by document then id. `validate_corpus` calls `validate_view` once per member, so emitting the finding "in the view validation" yields one copy per member for an N-member corpus. Fix: state that the finding's path is the declaring module's `manifest.yaml`, that it is emitted once per run (not per member), and that drops are ordered by module then item (NFR-006). | spec/functional/FR-058-upward-trace-completeness.md:301-303; src/corpus/validate.rs:292-315; src/corpus/validate.rs:84-110 | missing-requirement |

## Verdict

Not mergeable on this lens until FND-001 and FND-002 are resolved: each leaves a
silent or two-readings outcome on a real path. FND-003 is a determinism detail.

## Dispositions

Round 1, at head 18d020c7 (fix commit checked against review head 50c28286).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | after: "`quire coverage` and every other consumer that computes over the merged model SHALL NOT compute silently over a model with dropped declarations: when any declaration was dropped, `quire coverage` exits with the distinct non-zero exit of FR-050 and names each dropped declaration on stderr, and no report is written." New FR-058-AC-47 tests it. |
| FND-002 | fixed | after: "Cascades. A declaration that depends on a dropped one is itself unexecutable and is dropped, each with its own failure naming the declaration it depended on", with the remediation, carried-field, trace-target and `sets` cases listed; FR-058-AC-46 tests three of the four. The two cascades still missing (status vocabulary, design_relations) are tracked in base FND-002. |
| FND-003 | fixed | after: "The path is the module's `manifest.yaml`. Findings are ordered by module load order, then kind, then item. In `validate_corpus` the findings are raised once for the corpus, not once per member." FR-058-AC-41 tests the path and the single raise. |
