---
id: SR-2026112316341703
title: "integrity review of the per-declaration load drop (quire-rs PR 712)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-rs@50c28286; spec/functional/FR-058-upward-trace-completeness.md; spec/functional/FR-096-flow-rules.md; spec/functional/FR-090-declared-carried-fields.md; spec/functional/FR-057-corpus-check-severity.md; spec/functional/FR-050-declarative-coverage-computation.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-058
    type: references
  - target: ix://agent-ix/quire-rs/FR-090
    type: references
---

# Integrity review of the per-declaration load drop (quire-rs PR 712)

## Summary

Ticket: PLAT-1804. Checks the change for consistency with FR-050, FR-057,
FR-090 and FR-096, and for coherence of the conflicting-declaration rule
("each conflicting declaration is dropped and each failure names the other
declaring module"). Oracles: the cross-module checks in `src/loader/mod.rs`
1415-1574 and `report_cross_module_duplicates` in
`src/loader/flow_rules.rs`.

Examined, clean: FR-096's load-time intro and FR-096-AC-29 agree with FR-058;
the qualifier-overlap and carried-field conflicts match today's
"fail every declaring module" behaviour, narrowed to the entries; FR-050's
"rejected at module load" clauses (AC-25, AC-60, AC-63) still read correctly
under a per-entry drop. The FR-057 tunability conflict is recorded in the base
review (FND-001 there).

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | [ambiguous] The conflict rule names two conflicts (carried-field keys, overlapping qualifier prefixes), but the loader has three more cross-module conflicts, and the spec does not say whether each one drops all parties or stays first-wins. (a) The same flow-rule or remediation name in two modules (`report_cross_module_duplicates`, which fails today). (b) The retirement check runs on the merged model (`validate_retirement`, `mod.rs:1420-1436`); a `retired` value in one module that contradicts another module's vocabulary is a conflict between modules, yet the drop list calls it "one retirement-vocabulary entry", and which module's entry goes is unstated. (c) A duplicate `required_relations` name across modules (AC-10's "duplicate relation name"). Also, "names the other declaring module" is singular, so it is undefined for three or more declaring modules. Fix: list every cross-module conflict with its outcome (all dropped), and write "names every other declaring module". | spec/functional/FR-058-upward-trace-completeness.md:295-298; src/loader/flow_rules.rs:29; src/loader/mod.rs:1420-1436 | wrong-requirement |
| FND-002 | low | [other] FR-090-AC-10 still reads "the same `name` with different `key` fails the merge naming both modules". That is compatible with FR-058-AC-40, but it does not say that only the two entries are dropped and the rest loads, and it gives no cross-reference, so a reader of FR-090 alone still infers the whole-model failure the code has today. Fix: amend FR-090-AC-10 to say both entries are dropped (FR-058-AC-40). | spec/functional/FR-090-declared-carried-fields.md:104; spec/functional/FR-058-upward-trace-completeness.md:378 | wrong-requirement |

## Verdict

Mergeable on this lens after FND-001 is resolved; FND-002 is a one-line
cross-reference.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-003 | medium | [soundness] The new cross-module conflict "a `required_relations` name declared by more than one module" drops every declaration of that name. That contradicts the merge rule in force: `merge_traceability` merges relations "first-wins by name ... like every other named entry" (`src/loader/mod.rs:1730-1742`), and FR-050 states named entries merge first-wins. It also differs from `carried_fields`, where an identical name and key merge (FR-090-AC-10). Scenario: a base module and an extension module both declare relation `ac-has-test` with identical bodies. Today this loads, and the first wins. Under this text both are dropped, the relation's checks vanish, and every repo selecting both modules fails validation. Fix: make it a conflict only when the declarations differ, with identical ones merging to one, as FR-090 does; or state on purpose that relation names are module-unique and drop the first-wins rule from FR-050 and the code comment. | spec/functional/FR-058-upward-trace-completeness.md:318-323; src/loader/mod.rs:1730-1742; spec/functional/FR-050-declarative-coverage-computation.md:1018-1020 | wrong-requirement |


## Dispositions

Round 1, at head 18d020c7 (fix commit checked against review head 50c28286).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | after: "The conflicts are: a `carried_fields` name declared with different keys; `trace_id_qualifiers` prefixes that are not unique or overlap; a `flow_rules` or `remediations` name declared by more than one module; a `required_relations` name declared by more than one module; a retirement-vocabulary value that contradicts another module's vocabulary." The text now names "every other declaring module", and FR-058-AC-40 covers three modules. (The required_relations conflict raises the new FND-003.) |
| FND-002 | fixed | after: FR-090-AC-10 "the same `name` with different `key` drops both entries, each failure naming every other declaring module, and every other declaration loads (FR-058-AC-40)." |

Round 2, at head d818f6c7 (fix commit checked against round-1 head 18d020c7).

| FND | outcome | reason |
| --- | --- | --- |
| FND-003 | fixed | after: "a `required_relations` name declared by more than one module with different bodies (identical declarations merge to one, first-wins, as `carried_fields` do)"; FR-058-AC-40 "Two modules declaring a required relation with one name and an identical body merge to one relation and record no failure." |
