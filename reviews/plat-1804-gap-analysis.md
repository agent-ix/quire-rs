---
id: SR-2026112316341705
title: "gap analysis of the per-declaration load drop (quire-rs PR 715)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; PR #715 at b098f417; spec/functional/FR-058-upward-trace-completeness.md (AC-10, AC-38..47), spec/functional/FR-096-flow-rules.md (AC-29), spec/functional/FR-090-declared-carried-fields.md (AC-10); tests/it/declaration_drop.rs, tests/it/field_rules.rs, tests/it/flow_rules.rs, tests/it/required_relations.rs, src/loader/mod.rs tests"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-058
    type: references
  - target: ix://agent-ix/quire-rs/FR-096
    type: references
  - target: ix://agent-ix/quire-rs/FR-090
    type: references
---

# Gap analysis of the per-declaration load drop (quire-rs PR 715)

## Summary

Ticket: PLAT-1804. Manual acceptance-criterion-to-test check, scoped to the ACs the
ticket names; the full planless `quoin:gap-analysis` matrix and reverse-gap sweep over
the repository was not run, so this is not a full gap audit. Plan completion: not
assessed. For each AC the tagged test was read and its oracle judged against the AC
text; focused runs of every listed test pass (see the code-review SR of this PR).

## Examined scope

- FR-058-AC-10: each listed unit drops alone, `tests/it/declaration_drop.rs` and
  `tests/it/required_relations.rs` (tc907: different bodies drop with one failure,
  identical merge). Covered. (examined)
- FR-058-AC-38 / FR-096-AC-29: `an_unloadable_flow_rule_drops_only_itself`, red at
  origin/main. Covered. (examined)
- FR-058-AC-39: prune-level test per kind. Archetype unit not covered (FND-002). (examined)
- FR-058-AC-40 / FR-090-AC-10: three-module relation test names both others; carried
  field test in `field_rules.rs`. Other kinds two-module only (FND-003). (examined)
- FR-058-AC-41: finding count, path, pack-less, both postures, `is_valid`, `R` findings,
  corpus once. Covered. (examined)
- FR-058-AC-42: the AC-38 test fails under a reset mutation (traceability() is None). Covered. (examined)
- FR-058-AC-43: CLI parser refusal and the other-entry case covered; manifest oracle weak (FND-004). (examined)
- FR-058-AC-44: oracle cannot fail under reason-text detection (FND-001). (examined)
- FR-058-AC-45: section drop, archetypes load, model empty, finding raised. Covered. (examined)
- FR-058-AC-46: all five cascades exercised; design-relation naming weak (FND-004). (examined)
- FR-058-AC-47: library refusal covered; the CLI exit, stderr and no-report half is
  in quire-cli and untested here (FND-005). (examined)

## Verdict

Most ACs have real oracles. Gaps: AC-44's test cannot detect the mutation it guards,
AC-39 lacks the archetype unit, AC-40 three-module coverage exists for relations only,
and AC-47's CLI half is unverified. All are test additions inside this PR except FND-005,
which needs the quire-cli bump.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-058-AC-44 requires a failure whose reason text matches a drop, but carries no kind, to raise no finding. The test uses a schema "not json" failure whose reason matches nothing a drop says, and the kinded half counts failures whose reasons also look like drops. An implementation detecting drops by `reason.starts_with("traceability:")` or "merged selected traceability model" passes. Needs a non-kinded failure whose reason imitates a drop (for example a schema path containing that text) | tests/it/declaration_drop.rs:520-560 |
| FND-002 | medium | FR-058-AC-39 lists "an archetype with an unresolvable frontmatter_schema_ref" as a drop unit. No test loads a module with one such archetype next to a valid one and asserts the valid one loads, the bad one is absent and one `archetype` failure names it; only the FR-096-AC-27 tests touch the path, with a flow rule in the way. The AC-39 test also runs `prune` only, so "no module fails to load" is not asserted per kind | tests/it/declaration_drop.rs:139-330 |
| FND-003 | low | FR-058-AC-40 "with three declaring modules, two names" is tested for `required_relations` only. Qualifier, flow-rule, remediation and retirement conflicts have two-module tests that check kind and item but not that each failure names the other module. The retirement defect in the code-review SR (duplicate failures) would have been caught by a three-module case | tests/it/declaration_drop.rs:345-395 |
| FND-004 | low | Weak oracles. FR-058-AC-43: the manifest half asserts only `load_module(..).is_err()`, without the reason or a control manifest that loads with another key. FR-058-AC-46: `reasons("gone").contains("gone")` finds the required-relation failure first and is true whatever the design-relation failure says, so "naming the declaration depended on" is unasserted for design relations | tests/it/declaration_drop.rs:480-500 |
| FND-005 | low | FR-058-AC-47 is about `quire coverage`: distinct exit, stderr naming each drop, no report. Only the library `CoverageError::DeclarationsDropped` is tested. In quire-cli `compute_report` maps any `CoverageError` through `invalid_error!` (the same exit as ModelUndeclared), so a dependency bump gives the exit, but `require_traceability_model` runs first and, when every declaration is dropped (AC-45), prints "no module in scope declares a traceability model" instead of naming the drop | src/coverage.rs:943-956 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The AC-44 test's kind-less failure now has a reason containing "merged selected traceability model: flow_rules entry 'bad' dropped" (through the schema path) and asserts the imitation is present and raises no finding |
| FND-002 | fixed | New test drops an archetype with an unresolvable `$ref` next to a valid one (valid loads, broken absent, one `archetype` failure naming Missing.json); `one_drop` now also loads each AC-39 case as a module and asserts it loads with the drop on record |
| FND-003 | fixed | `three_way` covers carried fields, flow rules, remediations, qualifiers and retirement with three modules, one failure per declaration naming the two others |
| FND-004 | fixed | AC-43 asserts the refusal reason and adds a control key that loads; AC-46 asserts the design-relation failure says it depends on dropped required_relations entry 'gone' |
| FND-005 | deferred | The CLI half of AC-47 lives in quire-cli: the CHANGELOG upgrade notes say `require_traceability_model` must check `registry.dropped_declarations()` first (coverage and trace), and follow-up ticket PLAT-1854 owns it with the quoin `coverage_error` arm |
