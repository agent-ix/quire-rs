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
