---
id: ADR-0009
title: "Concrete grammar vocabulary is module data, not engine"
type: ADR
---

# ADR 0009: Concrete grammar vocabulary is module data, not engine

**Status**: Accepted
**Date**: 2026-06-23
**Decision authority**: kreneskyp

## Context

The EARS `vague-response` check ([FR-042](../../functional/FR-042-requirement-grammar-check.md)) is
**object-aware**: a weak verb (`provide`/`support`/`process`/…) is flagged only when its object is
abstract or absent; a concrete object, a mechanism/numeric qualifier, or a backticked code
identifier suppresses it. This keeps high recall (flag-unless-concrete) while removing the worst
noise — a full-corpus pass went from 791 to 173 vague-response findings.

The "concrete object" test, however, leaned on a **hardcoded ~60-term software-noun list** baked
into the engine (`re_concrete_noun()` in `src/grammar/ears.rs`). A corpus study of the residual
soft tail showed this is a dead end: the leaks are ~40 distinct domain nouns (`pagination`,
`cursor`, `replica`, `backend`, …), almost all appearing exactly once. The concrete-noun space is
**unbounded and domain-specific** — every domain has its own — so no fixed engine list can close it,
and a software-noun list hardcoded into a domain-agnostic engine is a layering violation.

Two structural alternatives were weighed:

- **Flip the default** to *suppress-unless-vague* (flag only on an abstract-term match). This makes
  the maintained lexicon small and bounded, but trades the failure mode from visible, self-
  correcting false positives to **silent false negatives** — a tool that fails to catch vagueness
  without telling anyone. Since the check will eventually enforce, a silent miss defeats the gate.
- **Grow the engine noun list.** Whack-a-mole against an unbounded, domain-specific set; bakes a
  software assumption into a generic engine.

## Decision

**Concrete vocabulary is data supplied by modules, not code in the engine.**

- The engine keeps only the *bounded, generic* signals: the vague-quality lexicon, the
  mechanism/numeric qualifiers, and the backtick hatch. `re_concrete_noun()` is removed.
- Modules declare a `lexicon` registry in `manifest.yaml`, merged first-wins across all loaded
  modules exactly like `edge_types`/`roles` ([FR-040](../../functional/FR-040-object-edge-vocabulary.md)).
  The grammar consumes the merged set ([FR-043](../../functional/FR-043-module-concrete-lexicon.md)).
- The baseline software vocabulary is **distributed across the domain `spec-objects-*` modules** —
  each domain owns its terms, co-located with the objects that define them, and a software project
  gets them by activating the modules it already uses.
- A project-scoped **Ubiquitous-Language** artifact layers on top later (possibly several, per
  framework — DDD via `spec-objects-business`, EA via `spec-objects-enterprise`).

We deliberately **do not flip** the default: flag-unless-concrete is kept, so recall stays high and
the only failure mode is a visible, self-correcting false positive — which an author silences by
making the object concrete (backtick it, name it, or define it in the project glossary), each of
which improves the spec. The backtick hatch already lets authors reference code objects outside any
fixed lexicon.

## Consequences

- **Positive**: the noun list becomes each domain's bounded vocabulary, not an ever-growing engine
  list; the engine stays domain-agnostic; defining a term to silence a finding is itself good
  requirements hygiene (ISO/IEC/IEEE 29148 wants a project glossary for specially-used terms, and
  does not require redefining common industry terms — so a shipped baseline is standards-aligned).
- **Negative / accepted**: the grammar's type-only path (`validate_document`, no registry) cannot
  see the merged lexicon, so under it bare domain nouns are not suppressed (mechanism/bound/backtick
  still are). The registry-backed path — the CLI and Python surfaces — is the standard one and
  applies the full lexicon. Revisit only if the type-only path becomes a real consumer.

## Alternatives rejected

- **Flip to suppress-unless-vague** — rejected: silent false negatives defeat an enforcing gate.
- **Grow the hardcoded engine noun list** — rejected: unbounded, domain-specific, and a layering
  violation in a generic engine.
