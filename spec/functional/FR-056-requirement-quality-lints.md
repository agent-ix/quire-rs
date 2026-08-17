---
id: FR-056
title: "Requirement-Quality Lints"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-042"
    type: "extends"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-048"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-017"
    type: "implements"
---
# FR-056: Requirement-Quality Lints

## Description

`quire-rs` SHALL check every normative statement against the mechanically
decidable ISO 29148 **quality characteristics**, as a distinct grammar on the
[FR-042](./FR-042-requirement-grammar-check.md) framework.

The grammar stack judges whether a statement matches an EARS *pattern*
(FR-042) and whether an acceptance criterion has the right *shape*
([FR-047](./FR-047-acceptance-criteria-grammar.md)). Neither asks whether the
statement is well-formed **as a requirement** — unambiguous, verifiable, and
clear about who acts. A statement can be a flawless ubiquitous EARS pattern and
still say "the system shall be appropriately robust".

### A separate grammar id, not more `ears:` checks

These register as `quality:*` rather than under `ears`. They are a different
judgement — 29148 quality rather than EARS conformance — and
[FR-048](./FR-048-per-check-grammar-severity.md) keys severity on
`<grammar>:<check>`, so a deployment can silence or promote the quality pack
without touching pattern conformance. Bundling them under `ears` would have
made those two decisions one lever.

### The checks

| Check | Fires when | Why it is mechanical |
|---|---|---|
| `quality:ambiguous-term` | The statement uses a term from the merged ambiguity lexicon (`adequate`, `as appropriate`, `robust`, `minimize`, `and/or`, `etc`, …) | Closed denylist, exact word match with a short inflection tail |
| `quality:agentless-passive` | `shall be <past participle>` with no `by <agent>` following | A syntactic pattern, not a judgement about style |
| `quality:mixed-modal` | Two different modals (`shall`/`should`/`may`/`must`) in one statement | Presence of two tokens |

**Non-singular already ships.** agent-ix/quire-rs#83 listed a statement-level
non-singular check as a fifth item. `ears:non-singular` already fires on
`shall_count > 1` over exactly these sections, and has since FR-042. Adding a
`quality:` twin would have produced two findings for one defect. Recorded here
rather than silently dropped.

### Why this is not the CR-014 failure mode

CR-014 retired `no-observable-outcome` because its **membership test** was
unreliable: an open verb set, required to earn a label, at ~13% sampled
precision. These three do not guess. `ambiguous-term` matches a closed list;
`agentless-passive` matches a syntactic shape; `mixed-modal` counts tokens. The
detection is exact by construction, so the fit question is not "is the detector
right" but "is the reported rate liveable".

> **[RAN] Ecosystem fit check, before shipping — the CR-014 discipline.**
> `cargo run --example fr056_fit_check` over `~/dev`: **239 repositories, 3,335
> FR/NFR/StR documents**, worktree copies deduped.
>
> | | Findings | |
> |---|---|---|
> | `quality:agentless-passive` | 678 | the dominant one |
> | `quality:ambiguous-term` | 145 | |
> | `quality:mixed-modal` | 130 | |
> | **Documents with ≥1 finding** | **674 / 3,335 = 20.2%** | |
>
> One fifth of the corpus, not one half, and every finding advisory and
> independently silenceable. Dogfooded: this repository's own spec reports
> **17 / 76 = 22.4%**, so the ecosystem number is not something the engine's
> own authors are exempt from.
>
> The check is kept as a **compiled example** rather than a one-off script, so
> it cannot rot the way an unrun baseline does (the CR-057 lesson): re-running
> it is `cargo run --example fr056_fit_check`, optionally scoped to one repo.

### Vocabulary is module data

The engine ships a deliberately small built-in ambiguity lexicon and merges a
module's `ambiguity_terms:` registry **over** it, first-wins — the
`vacuous_predicates` arrangement (ADR 0009). The built-ins are never replaced,
so a module extends the lexicon rather than swapping it out, and a deployment
adds its own house words without a code change.

## Inputs

- FR `Description` / `Behavior` / `Constraints`, NFR `Statement`, StR
  `Stakeholder Need` — the same sections `ears` binds, because a statement worth
  pattern-checking is worth quality-checking.
- The merged `ambiguity_terms` registry, layered over the built-ins.

## Outputs

- `GrammarFinding` values with `grammar: "quality"`, one per violated check,
  each addressable by the FR-048 severity registry as `quality:<check>`.

## Behavior

Each bound statement SHALL be checked against all three checks independently, so
a statement violating two reports two findings. Every finding SHALL carry
`GrammarSeverity::Warning` on arrival.

Detection SHALL read the **CR-017 masked** copy of the statement, so a term
inside an inline code span is a mention rather than a use: `the manifest key
` `optimize` ` is rejected` is not an ambiguous requirement, and reading the raw
text would call it one.

The ambiguity lexicon SHALL match longest-term-first, so a statement containing
`as appropriate` is reported as that rather than as the `appropriate` inside it —
the report has to name what the author actually wrote.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-056-CON-1 | Every check SHALL ship advisory (`Warning`). Promotion to `error` is a corpus sweep plus explicit sign-off, never a default — the FR-047-CON-1 pattern. | Process | Test |
| FR-056-CON-2 | The engine's built-in ambiguity lexicon SHALL be closed, with a module's declared terms layered over it rather than replacing it. An open set whose membership drives a finding is the shape CR-014 retired. | Architecture | Test |
| FR-056-CON-3 | No check SHALL depend on a judgement the engine cannot make exactly. Each fires on a closed list, a syntactic pattern, or a token count — never on an inference about meaning. | Architecture | Inspection |
| FR-056-CON-4 | The pack SHALL NOT change any `ears` or `ac` finding. It adds a grammar; it does not reinterpret the two that exist. | Architecture | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-056-AC-1 | A statement using a built-in ambiguity term reports `quality:ambiguous-term` naming the term, and the same statement with the term removed reports nothing. | Test (TC-861) |
| FR-056-AC-2 | A statement containing `as appropriate` is reported as `as appropriate`, not as `appropriate` — the longest matching term names the finding. | Test (TC-862) |
| FR-056-AC-3 | A module declaring `ambiguity_terms:` has its terms merged over the built-ins: the declared term fires, and every built-in still fires. | Test (TC-863) |
| FR-056-AC-4 | `shall be validated` reports `quality:agentless-passive`, and `shall be validated by the parser` reports nothing — the check is about missing allocation, not the passive voice. | Test (TC-864) |
| FR-056-AC-5 | A statement mixing `shall` and `should` reports `quality:mixed-modal` naming both, while a statement using one modal reports nothing. | Test (TC-865) |
| FR-056-AC-6 | An ambiguity term inside an inline code span fires nothing, and the same statement with the term unquoted fires (CR-017 parity). | Test (TC-866) |
| FR-056-AC-7 | Every finding the pack emits carries `Warning` severity, and each is independently addressable by an FR-048 `quality:<check>` key — set to `off`, the check contributes nothing (CON-1). | Test (TC-867) |
| FR-056-AC-8 | A corpus checked with the quality pack reachable yields the same `ears` and `ac` findings, in the same order and with the same fields, as the same corpus checked before the pack existed (CON-4). | Test (TC-868) |
| FR-056-AC-9 | A statement violating two checks reports two findings, one per check, rather than the first one only. | Test (TC-869) |

## Dependencies

- **Upstream**: [FR-042](./FR-042-requirement-grammar-check.md) (the grammar framework, the bound sections and the statement collection), [FR-047](./FR-047-acceptance-criteria-grammar.md) (the CR-017 mask), [FR-048](./FR-048-per-check-grammar-severity.md) (per-check severity, which is what makes a separate grammar id worth having), [FR-014](./FR-014-module-activation.md) (manifest loading for the registry)
- **Downstream**: `spec-artifacts-iso` may declare its own `ambiguity_terms`; quoin's spec-review lenses read the findings (agent-ix/quoin#89)
