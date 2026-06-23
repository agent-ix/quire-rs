---
id: FR-042
title: "Requirement-Grammar Check (EARS)"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-032"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-010"
    type: "requires"
    cardinality: "1:1"
---

## Description

`quire-rs` SHALL provide a **grammar-check framework** that evaluates the
natural-language requirement statements inside requirement-bearing artifacts
against a registered **grammar**, and SHALL ship **EARS** (Easy Approach to
Requirements Syntax) as its first grammar. Grammar checking is a posture
distinct from declarative lint ([FR-036](./FR-036-declarative-lint-rules.md)) and
structural validation ([FR-032](./FR-032-validate-document.md)): a grammar
classifies and checks the *prose of individual normative statements*, whereas
lint checks declarative content patterns and validation checks document
structure.

The framework SHALL register each grammar against an `(archetype, section)`
binding, so a grammar runs only on the sections it governs. When a
requirement-bearing document is validated, the framework SHALL segment each
governed section into **normative statements** — a sentence, list item, or
table cell bearing a modal verb (`shall`/`shall not`) — and SHALL skip text
inside fenced code blocks, blockquotes, and reference lines.

The framework SHALL route every grammar finding into the validation result by
**severity**: when a finding's severity is `warning`, the framework SHALL record
it in `ValidationResult.warnings` without setting `is_valid` false; when the
severity is `error`, the framework SHALL record it in `ValidationResult.errors`
and SHALL fail validation. Severity SHALL be sourced from configuration so a
deployment can promote a grammar from advisory to enforcing without a code
change.

### EARS grammar

The EARS grammar SHALL classify each normative statement into exactly one
pattern — `ubiquitous`, `event` (`When …`), `state` (`While …`), `unwanted`
(`If … then …`), `optional` (`Where …`), or `complex` (a combination) — or SHALL
mark it `unclassifiable` when no pattern matches.

For each statement, the EARS grammar SHALL emit a finding when the statement
violates a clause rule:

1. **non-singular** — the statement contains more than one `shall`.
2. **missing-subject** — the statement names no system/actor subject.
3. **vague-response** — the response verb is vague (`support`, `handle`,
   `manage`, `process`, `deal with`, `provide`, `enable`, `be able to`). The
   check is **object-aware**: a verb is flagged only when its object is abstract
   or absent — a concrete object surface (`provide an endpoint`, `process push
   events`) or a mechanism/quantitative qualifier (`handle X by Y`, `process
   within 16 ms`) states a verifiable response and is not flagged. `be able to`
   is the one verb-intrinsic case (capability-not-behavior) and is always flagged.
4. **non-canonical-trigger** — the statement leads with a non-EARS trigger
   (`On …`, `Upon`, `After`, `Before`, `Once`, `During`) instead of `When`/
   `While`.

The EARS grammar SHALL apply per-archetype dialects: an enumerated
`The <system> SHALL:` stem followed by a numbered response list SHALL count as a
single statement (not non-singular); a `StR` statement SHALL accept a stakeholder
or product subject (not only "the system"); and an `NFR` statement absent any
trigger SHALL NOT be reported as a defect. The EARS grammar SHALL NOT flag
passive voice.

### Non-goals (v1)

- The framework SHALL NOT judge whether the author chose the *semantically*
  correct keyword (e.g. `When` used for a continuous `While` state); that
  judgment is left to the agent review lens.
- No grammar other than EARS ships in v1; `GWT` (acceptance criteria) and the
  `US` story grammar register onto the same framework later.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-042-AC-1 | Each of the six EARS patterns (`ubiquitous`, `event`, `state`, `unwanted`, `optional`, `complex`) is classified from a representative statement, and a statement matching none is reported `unclassifiable`. | Test |
| FR-042-AC-2 | A statement with two `shall` clauses yields exactly one `non-singular` finding; an enumerated `The X SHALL:` stem followed by a numbered list yields none. | Test |
| FR-042-AC-3 | A statement with no system subject yields a `missing-subject` finding; a `StR` statement with a stakeholder subject (`The operator …`) does not. | Test |
| FR-042-AC-4 | A statement using a vague response verb (`shall support`) yields a `vague-response` finding; a passive-voice statement (`shall be included`) yields none. | Test |
| FR-042-AC-5 | A statement leading with `On startup, … shall …` yields a `non-canonical-trigger` finding; an `NFR` statement with no trigger yields none. | Test |
| FR-042-AC-6 | A grammar runs only against its bound `(archetype, section)` pairs: an EARS rule bound to FR `Description` produces no findings against an FR `Dependencies` section or an `IT` document. | Test |
| FR-042-AC-7 | A `warning`-severity finding is recorded in `ValidationResult.warnings` and leaves `is_valid` true; promoting the same finding to `error` records it in `errors` and sets `is_valid` false. | Test |
| FR-042-AC-8 | Each finding carries the offending statement excerpt, a 1-based line number, the matched pattern, and a severity. | Test |
| FR-042-AC-9 | Statements inside fenced code blocks, blockquotes, and reference lines are not segmented as normative statements and yield no findings. | Test |
| FR-042-AC-10 | The framework entry point is exposed through the Python (PyO3) binding and returns the same findings as the in-process Rust call for a fixture document. | Test |

## Dependencies

- **Upstream**: [FR-032](./FR-032-validate-document.md) (requires — findings merge into `ValidationResult`), [FR-010](./FR-010-query-api.md) (requires — section/table/list extraction)
- **Downstream**: the authoring (`/specify`), coverage (`/spec-matrix`), and review (`/spec-review`) workflows consume grammar findings; the `US-014` author-validates-markdown story is extended by this gate.
