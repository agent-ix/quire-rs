---
id: SR-2026112316341701
title: "spec-review of the per-declaration load drop (quire-rs PR 712)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs@50c28286; spec/functional/FR-058-upward-trace-completeness.md; spec/functional/FR-096-flow-rules.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-058
    type: references
  - target: ix://agent-ix/quire-rs/FR-096
    type: references
---

# Spec review (base) of the per-declaration load drop (quire-rs PR 712)

## Summary

Ticket: PLAT-1804. Base checklist pass over `git diff origin/main...50c28286`
(2 files, +37/-4): the new FR-058 section "A declaration that cannot load drops
itself, never the model", FR-058-AC-38..42, the AC-10/AC-19 rewording, the
FR-096 load-time intro and FR-096-AC-29. Oracles read at the head:
`src/loader/mod.rs` (the six model reset sites, 1436-1573),
`src/loader/flow_rules.rs::validate_flow_declarations`,
`src/loader/manifest.rs::parse_manifest`, `src/corpus/validate.rs`
(`validate_bundle`, `validate_corpus`, `BundleReport::route`/`bridged`),
`src/error.rs::ArchetypeLoadFailure`.

Gates run by the reviewer: `quire validate --scope .` on both changed files
reports no error; two `ac:non-canonical-shape` warnings (FR-058 lines 347-348).

Examined, clean: AC-38 fails on today's code (the flow-rule reset at
`mod.rs:1565-1574` empties the model, so `R` is absent) and is a good red test;
AC-42's mutation is well-formed and kills exactly the reset behaviour;
AC-40 matches the cross-module carried-field case at `mod.rs:1495-1521`;
AC-41's non-zero result maps onto `BundleReport::is_valid()`; FR-096's intro
and AC-29 agree with FR-058.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | [soundness] The section says `trace:declared-model-dropped` "is an error by default and is tunable like any other `trace:<check>` key (FR-057)". The lead's ruling is that it must be non-tunable, because tuning it re-creates the silent wrong result the ticket exists to remove. Scenario: module B ships a flow rule naming an undeclared archetype and `grammar_severity: {"trace:declared-model-dropped": off}`; `grammar_severity` merges first-wins across modules, so B (or any selected module) silences its own drop for every repo, and `validate` exits 0 with B's rule silently gone. Fix: state that the finding is not registrable, the same way FR-057-CON-1 treats bridged document-level results (in code: emitted with `pack: None` through `BundleReport::bridged`, so `route()` never reads a severity entry for it), that it is an error under both `Okf` and `Strict`, and that a manifest or CLI severity entry naming the key is refused at load. Add an AC: mapping the key to `off` or `warning` is refused (or has no effect) and the AC-38 selection still fails validation under both postures. | spec/functional/FR-058-upward-trace-completeness.md:300-306; spec/functional/FR-058-upward-trace-completeness.md:379; spec/functional/FR-057-corpus-check-severity.md:113; src/corpus/validate.rs:178-226 | wrong-requirement |
| FND-002 | medium | [ambiguous] The drop list does not cover every declaration that can fail today, and AC-10 now says every one of its cases "is dropped alone (FR-058-AC-38)". AC-10's cases include a blank verb in `acyclic_edges`, a duplicate relation name, and an uncompilable `exclude` glob; none of these is a kind in the drop list, and "duplicate relation name" has no stated drop unit (the first, the second, or both). Other unlisted kinds that fail at load today: `trace_targets`, `document_references`, model-level `exclude`/`source_exclude`, and an archetype `frontmatter_schema_ref_error`, which `validate_flow_declarations` reports and which today resets the whole model (`mod.rs:1565-1574`). These are all checked inside `parse_manifest` → `TraceabilityModel::validate()?`, whose `Err` drops the whole module, archetypes included. Two implementers would differ on whether an unlisted kind drops one entry, the module's traceability, or the whole module, and on whether it raises `trace:declared-model-dropped`. Fix: make the drop list exhaustive over the `traceability:` keys, or add one catch-all line (any other unexecutable traceability entry drops itself; a `traceability:` section that cannot be deserialized at all drops that module's traceability contribution, not its archetypes), state that both duplicates are dropped, and say that every such drop raises the finding. | spec/functional/FR-058-upward-trace-completeness.md:287-293; spec/functional/FR-058-upward-trace-completeness.md:347; src/loader/manifest.rs:608-615; src/loader/flow_rules.rs:29-46; src/loader/mod.rs:1565-1574 | wrong-requirement |
| FND-003 | medium | [untestable-ac] "Each drop records one load failure naming the module, the declaration kind, the dropped item and the reason", and the finding fires "for each load failure that dropped a declaration". `ArchetypeLoadFailure` has only `module`, `archetype`, `path`, `reason`, and the registry's `failures()` also holds archetype and template failures that drop no declaration. With no structured kind/item, the engine can tell a declaration drop from an archetype failure only by parsing `reason` text, and AC-41's "naming the dropped rule" has no field to assert on. Fix: state that a declaration drop is recorded with its kind and item as fields (a new failure variant or fields on the failure), and that the finding is built from those fields; or state that every recorded load failure raises the finding, which removes the distinction. | spec/functional/FR-058-upward-trace-completeness.md:300-306; spec/functional/FR-058-upward-trace-completeness.md:379; src/error.rs:129-134; src/registry.rs:495 | missing-requirement |
| FND-004 | low | [untestable-ac] AC-38 says "the other module's remaining rules load", but its selection gives the second module only the one unexecutable flow rule, so there are no remaining rules and the clause cannot fail. FR-096-AC-29 has the same "still evaluates the module's other rules" clause. Fix: give the second module a second, valid flow rule in AC-38 and assert it is in the merged model. | spec/functional/FR-058-upward-trace-completeness.md:376; spec/functional/FR-096-flow-rules.md:341 | wrong-requirement |
| FND-005 | low | [other] `quire validate` warns `ac:non-canonical-shape` on AC-10 (line 347): it is obligation-shaped ("SHALL fail at module load"). The PR rewrote this row. Fix: phrase it as an assertion ("A declaration that cannot be executed fails at module load, naming the offending entry, and is dropped alone ..."). | spec/functional/FR-058-upward-trace-completeness.md:347 | wrong-requirement |

## Verdict

Not mergeable. FND-001 contradicts the lead's ruling: as written, the finding
can be tuned off, which brings back the silent result. FND-002 and FND-003
leave the drop unit and the finding's trigger open to two readings.
FND-004 and FND-005 are small wording fixes.
