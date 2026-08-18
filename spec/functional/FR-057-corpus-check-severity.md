---
id: FR-057
title: "Per-Check Corpus Severity Configuration"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-048"
    type: "extends"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-038"
    type: "extends"
    cardinality: "1:1"
---
# FR-057: Per-Check Corpus Severity Configuration

## Description

[FR-048](./FR-048-per-check-grammar-severity.md) gives every **grammar** check an operator-facing
severity knob. Corpus-level checks — the packs [FR-038](./FR-038-okf-bundle-validation.md),
[FR-026](./FR-026-intra-spec-reference-resolution.md), [FR-040](./FR-040-object-edge-vocabulary.md)
and [FR-049](./FR-049-verification-reference-integrity.md) run over a whole bundle — have none.
`BundleFinding` carries `path`, `message` and a stable `reason` and no severity at all, so a
corpus pack has exactly **two** settings, chosen by *posture* rather than by the operator: a hard
error under `Strict`, a warning under `Okf`. There is no `off`, no per-check override, and no
promotion path.

This FR supplies the same mechanism FR-048 supplies for grammars, over the same registry, the
same key shape and the same CLI option.

### The registry is the one already shipped

Corpus checks SHALL be keyed `<pack>:<check>` into the **existing** `grammar_severity` registry.
No new manifest key is introduced and no schema change is required: the shipped
`module-manifest.schema.json` fixture declares `grammar_severity` with
`additionalProperties: {enum: [off, warning, error]}` and
`propertyNames: {pattern: "^[a-z0-9-]+:[a-z0-9-]+$"}`, so it is **open-valued on both halves** and
already accepts `refs:dangling-reference` against `spec-artifacts-iso` as released. The
engine→iso→module→quoin chain does not fire for this FR.

`quire validate --severity <pack>:<check>=<level>` therefore requires **no surface change**
either: the CLI already merges its entries over the module map and installs the result on the
`Registry` it hands to `validate_bundle`, which is where corpus checks will read it.

### The packs

| Pack | Checks | Owning FR |
|---|---|---|
| `bundle` | `no-frontmatter`, `malformed-frontmatter`, `index-incomplete`, `index-okf-version` | FR-038 |
| `refs` | `dangling-reference` | FR-026 |
| `edges` | `disallowed-edge-target` | FR-040 |
| `trace` | `dangling-trace-reference`, `archetype-matches-nothing` | FR-049 |

The **`reason` token is unchanged** by this FR. It is the machine surface consumers already read
(`quire validate` prints `… [dangling-reference]`); the pack is a prefix applied when forming the
registry key, not a rename.

### Default when the key is absent

FR-048-AC-4 makes an unconfigured grammar check a `warning`. Applying that blanket rule here would
**silently downgrade** every corpus check that hard-errors under `Strict` today — turning a
failing build green is not a severity mechanism's job. Each corpus check therefore declares the
tier it uses when unconfigured, and for every check that exists at the time of this FR that
declared tier **is its current behaviour**:

| Check | Unconfigured tier |
|---|---|
| `refs:dangling-reference` | posture-routed — error under `Strict`, warning under `Okf` |
| `trace:dangling-trace-reference` | posture-routed |
| `bundle:index-incomplete`, `bundle:index-okf-version` | posture-routed |
| `bundle:no-frontmatter`, `bundle:malformed-frontmatter` | warning in both postures (CR-048) |
| `edges:disallowed-edge-target` | warning in both postures (FR-040-AC-10) |
| `trace:archetype-matches-nothing` | warning in both postures (CR-054) |

A corpus check introduced **after** this FR SHALL declare `warning`, which restores FR-048-AC-4's
rule for everything new and confines the exception to the enumerated list above.

A configured key wins over the declared tier in both directions and in both postures: `error`
promotes an `Okf` warning, `warning` demotes a `Strict` error, and `off` drops the finding
entirely — recorded in neither `errors` nor `warnings` — matching FR-048-AC-9.

### What the registry MUST NOT reach

`validate_bundle` also bridges **document-level** validation results into the same report:
base-concept violations, archetype schema errors, `unknown-type`, and the missing-`type` error.
Those are not corpus check packs and SHALL NOT be registrable. A module that could map
`unknown-type: off` would be switching off schema validation, which is a different decision from
tuning a check's severity and must not wear the same lever.

Order is unchanged: severity is resolved as each finding is routed, so the input order every pack
already guarantees is preserved (NFR-006).

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-057-AC-1 | Validating a bundle on disk under `Okf` with a registry mapping `refs:dangling-reference` to `error` puts the dangling-reference finding in `errors` and makes `is_valid()` false, where the same bundle with no mapping reports it as a warning. | Test (TC-883) |
| FR-057-AC-2 | The same key mapped `warning` under `Strict` demotes a finding that is a hard error by default, and `is_valid()` becomes true — the promotion lever works in both directions. | Test (TC-883) |
| FR-057-AC-3 | A check mapped `off` records no finding in `errors` and none in `warnings`, in either posture. | Test (TC-883) |
| FR-057-AC-4 | With no entry for its key, every check in the pack table keeps the exact tier it had before this FR — posture-routed for the four posture-routed checks, warning-in-both-postures for the three that are fixed — verified per check rather than in aggregate. | Test (TC-884) |
| FR-057-AC-5 | Overrides layered through `Registry::with_grammar_severity` (the path `quire validate --severity` takes, FR-048-AC-5) apply to corpus checks, and a CLI-shaped entry wins over a conflicting module-declared entry for the same key. | Test (TC-885) |
| FR-057-AC-6 | Mapping one check `off` leaves its siblings reporting: with `refs:dangling-reference=off`, a bundle that also has a dangling **trace** reference still reports `trace:dangling-trace-reference`. | Test (TC-889) |
| FR-057-AC-7 | Every `BundleFinding` carries the severity that was applied, so a surface renders the configured level rather than inferring it from which vector the finding landed in. | Test (TC-886) |
| FR-057-AC-8 | The `reason` token of every check listed in the pack table is byte-identical to its pre-FR value, so a consumer matching on the machine surface is unaffected. | Test (TC-886) |
| FR-057-AC-9 | Every corpus finding routed through the pack surface carries a well-formed `<pack>:<check>` key accepted by `is_severity_key`, so a pack cannot ship unregistrable — asserted over the packs the engine emits, not a hardcoded list. | Test (TC-886) |
| FR-057-AC-10 | Findings appear in the same order with and without a severity map, and repeated runs over one bundle are identical (NFR-006). | Test (TC-887) |

## Constraints

| ID | Constraint | Verification |
|----|-----------|--------------|
| FR-057-CON-1 | Document-level validation results bridged into `BundleReport` — base-concept violations, archetype schema errors, `unknown-type`, missing `type` — SHALL NOT be registrable. Mapping them would let a module switch off schema validation under a severity key. | Test (TC-888) |
| FR-057-CON-2 | This FR introduces **no new manifest key** and no `module-manifest.schema.json` change. It reuses `grammar_severity`, which is open-valued on both halves as released in `spec-artifacts-iso` v0.12.0. | Inspection |

## Dependencies

- **Upstream**: [FR-048](./FR-048-per-check-grammar-severity.md) (the registry, the key shape, `--severity` layering), [FR-038](./FR-038-okf-bundle-validation.md) (the postures and `BundleReport`), [FR-014](./FR-014-module-activation.md) (module loading)
- **Downstream**: the P2 corpus checks — upward-trace completeness (agent-ix/quire-rs#85) and declared-vocabulary coverage (agent-ix/quire-rs#162) — ship advisory and need this knob to be tunable per repository
