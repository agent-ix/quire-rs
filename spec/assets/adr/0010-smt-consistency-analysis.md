---
id: ADR-0010
title: "SMT-backed cross-requirement consistency analysis"
type: ADR
---

# ADR 0010: SMT-backed cross-requirement consistency analysis

**Status**: Proposed — open questions only, no decision
**Date**: 2026-08-04
**Decision authority**: kreneskyp

## Context

The grammar checks (FR-042, FR-047) judge individual statements in isolation.
Nothing in the toolchain checks requirements **against each other**: two FRs can
impose contradictory obligations, an `If`-branch can leave its complement
unhandled, and a promoted AC can be unsatisfiable under another FR's
constraint — all invisible to per-statement analysis. Kiro's "analyze
requirements" feature is the commercial analog: it reports cross-requirement
conflicts, completeness gaps, and contradictions.

The candidate architecture is a two-stage pipeline:

1. **LLM formalization** — an agent translates each FR/AC normative statement
   into a constraint IR (typed variables, guards, obligations). The LLM does
   translation only; it renders no verdicts.
2. **Deterministic solving** — a solver (SMT, e.g. Z3) checks the IR corpus for
   pairwise conflicts (unsatisfiable conjunctions), coverage gaps (guard
   space not exhausted), and redundancy (implied obligations), and reports each
   with the trace ids of the implicated statements.

This split keeps the judgment deterministic and reproducible: the only
non-deterministic step is formalization, whose output is inspectable data.

## Placement (decided in principle)

The analysis lives in a **separate workspace crate** (working name
`quire-analyze`) that the `quire-rs` core, the wasm build, and the Python-wheel
build never depend on. Any solver dependency is confined to that crate; the
core's dependency hygiene, wasm target, and build cost are unaffected. Rust is
the implementation language — no Python sidecar.

## Open questions

1. **Solver integration mode.** Within `quire-analyze`, three options:
   (a) the `z3` crate's native bindings — the most mature API, but a link-time
   dependency (system `libz3` or a slow static build);
   (b) SMT-LIB2 over stdio via `rsmt2`/`easy-smt` — no link-time dependency,
   solver-agnostic (z3 or cvc5), the solver becomes a runtime-only requirement
   with graceful "solver not installed" degradation;
   (c) a pure-Rust SAT tier (`varisat`/`batsat`) for the propositional subset,
   with no external dependency at all.
   Leaning: (b) as the default, with (a) documented as the alternative if the
   stdio round-trip proves limiting.
2. **IR shape.** What is the constraint IR — a typed guard/obligation algebra
   per statement, or full SMT-LIB emitted directly? How are cross-FR shared
   variables identified (the lexicon? explicit declarations?), and how is the
   IR versioned so formalizations survive spec edits?
3. **Formalization non-determinism.** Two formalization runs can produce
   different IR for the same statement. Candidate mitigation: sample N
   formalizations and cluster by logical equivalence (solver-checked), keeping
   the majority class and flagging statements whose samples disagree as
   too-ambiguous-to-formalize (itself a useful finding). Is the cost
   acceptable, and what N?
4. **Scope.** Is this an advisory analysis *skill* (a quoin workflow that
   orchestrates formalization + `quire-analyze` solving and writes a
   SpecReview), or an engine feature with a stable CLI surface? The
   grammar/severity precedent (FR-042/FR-048) suggests engine features stay
   deterministic-only, which argues for: solving in `quire-analyze`,
   orchestration in a skill, and no LLM call anywhere inside quire.

## Decision

None. This ADR records the problem framing and open questions for review;
implementation is not planned until the questions above are resolved.

## Consequences

Deferred until a decision is taken. The immediate consequence of recording the
proposal: the coverage/symbol work (FR-050/FR-051) should keep trace ids and
statement excerpts machine-addressable, since every candidate design consumes
per-statement trace identities.
