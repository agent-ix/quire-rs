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

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-006 | low | [soundness] The new drop unit "one `exclude` glob of a relation (the relation loads without that glob)" makes the relation filter partly. Documents the author meant to exclude become obliged and report `trace:<check>` findings. FR-050-AC-25 sets the opposite rule for `source_exclude`: a pattern that does not compile "never partially filters". This is not silent, because the drop finding fails validation, but it adds false findings and breaks the "never partially filters" rule. Fix: make the relation the drop unit for an uncompilable `exclude` glob, and say so in AC-10 and AC-39. | spec/functional/FR-058-upward-trace-completeness.md:296; spec/functional/FR-050-declarative-coverage-computation.md:427 | wrong-requirement |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-007 | low | [ambiguous] For a duplicate relation name inside one module, the drop list and AC-10 now disagree. The drop list says "two entries with one `name` and different bodies drop both ... while identical entries merge to one, first-wins", with no cross-module qualifier, so it applies within a module too. AC-10 still says "a duplicate relation name" fails at module load and "the entry is dropped alone". Scenario: one module declares relation `R` twice with identical bodies. By the drop list, `R` merges with no failure and no drop finding. By AC-10, a failure is recorded and `R` is dropped. Fix: say AC-10's duplicate case is two different bodies (both dropped), or limit the identical-merge clause to declarations in different modules. | spec/functional/FR-058-upward-trace-completeness.md:294-295; spec/functional/FR-058-upward-trace-completeness.md:396 | wrong-requirement |
| FND-008 | low | [ambiguous] The drop list and AC-39 name a `field_rules` entry, but `field_rules` is not a manifest key. It is the Rust module (`src/traceability/field_rules.rs`, `validate_field_rules`) behind the keys `field_consistency`, `required_fields` and `vocabulary`. A test author for AC-39 has to guess which keys are meant. The "Any other kind" catch-all keeps the behaviour defined, so this is wording only. Fix: name the three manifest keys. | spec/functional/FR-058-upward-trace-completeness.md:296; spec/functional/FR-058-upward-trace-completeness.md:426; src/traceability.rs:443-455 | wrong-requirement |

## Dispositions

Round 1, at head 18d020c7 (fix commit checked against review head 50c28286).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | after: "The finding is a non-registrable bridged result in the sense of FR-057-CON-1: it is emitted without a pack, so no severity entry applies; it is an error under both `Okf` and `Strict`; and a manifest or CLI severity entry naming it is refused." New FR-058-AC-43 tests the refusal under both postures. |
| FND-002 | still-open | Narrowed, not closed. The list now covers acyclic_edges, exclude globs, trace_targets, document_references, obligations, archetype schema refs and an undeserializable section (AC-45). `TraceabilityModel::validate` also rejects five more kinds with no stated drop unit: model-level `exclude` and `source_exclude`, the `status` vocabulary, `design_relations` entries and `vocabulary_coverage` entries. Two cascades are also unstated: a `document_references` entry whose `status_column` depends on the `status` vocabulary, and a `design_relations` entry naming a dropped relation. Add these to the drop list and AC-39, or add a catch-all line with its drop unit. |
| FND-003 | fixed | after: "The load failure is structured: besides the module, path and reason it carries the declaration kind (for example `flow_rule`, `required_relation`, `carried_field`, `traceability_section`) and the item name. The engine tells a declaration drop from any other load failure by that kind, never by reading the reason text." New FR-058-AC-44 tests it. |
| FND-004 | fixed | after: FR-058-AC-38 "the other declares two `flow_rules` entries, `ok` (valid) and `bad` ...; `R`, the carried field and rule `ok` are present in the merged model." FR-096-AC-29 is amended the same way. |
| FND-005 | fixed | after: FR-058-AC-10 "A declaration that cannot be executed fails at module load naming the offending entry, and the entry is dropped alone (FR-058-AC-38) for each of: ..." `quire validate` now reports no warning on the file. |

Round 2, at head d818f6c7 (fix commit checked against round-1 head 18d020c7).

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | fixed | after: drop list adds "one `field_rules` entry; one `trace_tags` entry; one `vocabularies` entry; one `vocabulary_coverage` entry; one `design_relations` entry; the model-level `exclude` list and the model-level `source_exclude` list, each as one declaration ...; the `status` vocabulary, as one declaration"; cascades add the `status_column` and `design_relations` cases; and "Any other kind. A declaration kind `TraceabilityModel::validate` rejects that this list does not name is dropped as one entry of that kind, or as the whole declaration when the kind is a single value or list; it never fails the module or empties the merged model." AC-39 and AC-46 are extended. |
| FND-006 | fixed | after: "the same holds for a relation's own `exclude`, whose bad glob drops the relation that carries it"; AC-10 "an uncompilable `exclude` glob, which drops the whole relation carrying it". |

Round 3, at head 33e6e012 (fix commit checked against round-2 head d818f6c7).

| FND | outcome | reason |
| --- | --- | --- |
| FND-007 | fixed | after: drop list "two entries with one `name` in one module or across modules and different bodies drop both ... while identical entries merge to one, first-wins"; FR-058-AC-10 "two relations of one name in one module with different bodies, which drops both and records one failure naming the module (identical entries merge to one with no failure)". |
| FND-008 | fixed | after: drop list "one `field_consistency` entry; one `required_fields` entry; one `vocabulary` entry"; FR-058-AC-39 lists `field_consistency`, `required_fields`, `vocabulary`. |
