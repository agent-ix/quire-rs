---
id: FR-048
title: "Per-Check Grammar Severity Configuration"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-042"
    type: "extends"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-014"
    type: "implements"
---
# FR-048: Per-Check Grammar Severity Configuration

## Description

[FR-042](./FR-042-requirement-grammar-check.md) requires that grammar-finding
severity "be sourced from configuration"; the shipped engine hardcodes
`Warning` (`DEFAULT_SEVERITY` in `src/grammar/ears.rs`) and the only escalation
lever is `quire validate --strict`, which promotes **all** warnings globally.
This FR specifies the configuration mechanism as a **per-check severity map**,
so one check (e.g. every `ac` check) can be promoted to `error` while the rest
of the bundle stays advisory. It is authored as a new FR rather than an
amendment because FR-042's normative statement is unchanged — this FR supplies
the mechanism FR-042 left to configuration.

A Filament module MAY declare a `grammar_severity` registry in its
`manifest.yaml`: a map from `<grammar>:<check>` keys (e.g. `ac:vacuous-outcome`)
to a level in `off` | `warning` | `error`. The `off` level suppresses the
check entirely: the framework SHALL record no finding for an `off`-mapped
check — not in `warnings`, not in `errors`, and not in the `--summary`
histogram. This is the conservative rollout lever for high-volume advisory
checks (the [FR-047](./FR-047-acceptance-criteria-grammar.md) AC checks are
the expected first users). The loader SHALL merge the per-module `grammar_severity`
registries across all loaded modules first-wins, mirroring the `lexicon` merge
([FR-043](./FR-043-module-concrete-lexicon.md)). If two modules redeclare the
same key with different values, then the loader SHALL emit one non-fatal
`DuplicateGrammarSeverity` diagnostic. The `Registry` SHALL expose the merged
map through a `grammar_severity()` accessor.

When a grammar emits a finding, the framework SHALL set the finding's severity
from the merged map keyed by the finding's `grammar` and `check`, defaulting to
`warning` when the key is absent. Severity routing into `ValidationResult`
(warning → `warnings`, error → `errors` + `is_valid` false) is unchanged from
FR-042-AC-7; an `off`-mapped finding is dropped before routing.

`quire validate` SHALL accept a repeatable `--severity <grammar>:<check>=<level>`
option over the same `off` | `warning` | `error` vocabulary; a CLI-supplied
entry SHALL take precedence over the manifest map for
that key. The existing `--strict` flag SHALL keep its global semantics
(escalate the exit code on any remaining warning) unchanged.

When a grammar runs without a registry (the type-only `validate_document`
path), the framework SHALL apply the all-default map (every check `warning`),
matching the empty-lexicon degradation documented in FR-043.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-048-AC-1 | A manifest declaring `grammar_severity: {"ac:unclassifiable": error}` loads, and `Registry::grammar_severity()` returns the merged map containing that entry. | Test (TC-716) |
| FR-048-AC-2 | Two modules redeclaring the same key with different levels merge first-wins and emit one `DuplicateGrammarSeverity` diagnostic; identical redeclaration emits none. | Test (TC-717) |
| FR-048-AC-3 | With `ac:unclassifiable` mapped to `error`, an unclassifiable criteria cell produces an entry in `ValidationResult.errors` and sets `is_valid` false, while an `ears` finding with no map entry stays in `warnings`. | Test (TC-718) |
| FR-048-AC-4 | A finding whose `<grammar>:<check>` key is absent from the merged map defaults to `warning`. | Test (TC-719) |
| FR-048-AC-5 | `quire validate --severity ears:vague-response=error` fails a document whose only finding is a vague response, and the same CLI entry overrides a conflicting manifest entry for that key. | Test (TC-720) |
| FR-048-AC-6 | `--strict` semantics are unchanged: with no severity map, `--strict` still exits 1 on any warning, and without `--strict` the exit code stays 0 for warning-only documents. | Test (TC-721) |
| FR-048-AC-7 | The type-only `validate_document` path applies the all-default map: every grammar finding surfaces as a warning regardless of any module's manifest. | Test (TC-722) |
| FR-048-AC-8 | A malformed `grammar_severity` entry (unknown level, non-string key) fails module load like any other manifest shape error. | Test (TC-723) |
| FR-048-AC-9 | A check mapped `off` (manifest or `--severity ac:vague-response=off`) records no finding in `warnings`, `errors`, or the `--summary` histogram, while sibling checks of the same grammar still report. | Test (TC-752) |
| FR-048-AC-10 | A malformed `--severity` entry (unknown level, missing `=`, or an unparseable `<grammar>:<check>` key) is rejected with a usage diagnostic and a non-zero exit before validation runs. | Test (TC-755) |

## Dependencies

- **Upstream**: [FR-042](./FR-042-requirement-grammar-check.md) (severity routing), [FR-043](./FR-043-module-concrete-lexicon.md) (the first-wins registry merge pattern), [FR-014](./FR-014-module-activation.md) (module loading)
- **Downstream**: [FR-047](./FR-047-acceptance-criteria-grammar.md) AC checks are the first intended promotion target; `spec-artifacts-iso` ships the ISO severity defaults
