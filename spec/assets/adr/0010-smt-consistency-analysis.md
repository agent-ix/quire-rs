---
id: ADR-0010
title: "SMT-backed cross-requirement consistency analysis"
type: ADR
---

# ADR 0010: SMT-backed cross-requirement consistency analysis

**Status**: **Partially decided** — Q1 and Q4 decided, Q2 spiked, Q3 deferred
**Date**: 2026-08-04, amended 2026-08-18 (CR-070, agent-ix/quire-rs#86)
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

## Decision (2026-08-18)

Two of the four questions are decided, one is handed to a spike, one is
deferred behind that spike. The ADR is **partially decided**, not accepted —
saying "accepted" would imply the design is settled enough to build against,
and Q2 is load-bearing enough that it is not.

### Q1 — solver integration mode: **(b), SMT-LIB2 over stdio**

`quire-analyze` talks to a solver over SMT-LIB2 on stdio (`rsmt2` or
`easy-smt`), not through the `z3` crate's native bindings.

Three properties decide it, in order:

1. **Runtime-only dependency.** Option (a) makes `libz3` a *link-time*
   dependency, so every build of the workspace — including CI lanes that never
   run the analysis — either links a system library or pays a slow static
   build. A link-time dependency on a crate the core must never depend on is
   precisely the coupling the placement decision exists to prevent.
2. **Graceful absence.** With stdio, "no solver installed" is a runtime
   condition the tool reports and degrades on. With native bindings it is a
   build failure. An advisory analysis that cannot be installed without a
   solver is not advisory.
3. **Solver-agnostic.** SMT-LIB2 is the interchange format; z3 and cvc5 are
   both reachable, and disagreement between them is a signal about the
   encoding rather than a rewrite.

Option (c), a pure-Rust SAT tier, is **not chosen and not foreclosed**: it is
an optimization for whatever propositional subset turns out to dominate, and
that subset is unknown until Q2 reports. Revisit it with data, not before.

The stdio round-trip's cost is the accepted risk. If it proves limiting —
per-query process overhead dominating on a large corpus — (a) is the
documented fallback, and the SMT-LIB2 encoding is portable to it.

### Q4 — scope: **decided upstream, recorded here**

ADR 0011 §Consequences settles it: solving in `quire-analyze`, LLM
formalization orchestration in a quoin skill, **no LLM call anywhere inside
quire**. This ADR's own leaning was already that; it is now binding, and the
grammar/severity precedent (FR-042/FR-048) it appealed to is the general rule
rather than an analogy.

### Q2 — IR shape: **spiked, not decided here**

Tracked as agent-ix/quire-rs#164. Deciding the IR from first principles in an
ADR would be guessing: the questions it raises — a typed guard/obligation
algebra versus direct SMT-LIB, how cross-FR shared variables are identified,
how an encoding is versioned across spec edits — are answerable by encoding a
real subset of a real corpus and seeing what breaks, and not otherwise.

One input the spike should take rather than rediscover: the **obligation
record's content hash** (FR-053) is the natural versioning key. An encoding is
derived from a statement; the statement's hash already changes exactly when the
statement's words change, which is the event that should invalidate an
encoding.

### Q3 — formalization non-determinism: **deferred behind Q2**

N-sample-and-cluster is the candidate mitigation and remains so. Choosing N,
and judging whether the cost is acceptable, is **empirical and needs a solver
and an IR to exist first** — equivalence clustering is solver-checked, so there
is nothing to measure until Q2 lands and a solver is wired. Deciding N now
would be inventing a number.

The one part worth restating as a commitment: **"the samples disagree" is a
finding, not an error.** A statement whose formalizations do not agree is
too ambiguous to formalize, and that is a requirement-quality result the
consuming workflow should surface — not a retry loop.

## Consequences

**Implementation is P3, and deliberately so.** Nothing in Phase 2 builds
`quire-analyze`. Standing it up is not one crate: it is a new workspace member,
a new CI matrix lane, a new `deny.toml` surface for the solver-adjacent
dependency tree, and a standing obligation that `quire-rs` core, the wasm build
and the Python wheel **never** depend on it. That cost is worth paying once the
IR is known and not before, and it is stated here so a future reader does not
mistake "partially decided" for "ready to build".

**Formalization coverage is a first-class output.** Whatever `quire-analyze`
reports, it SHALL report **what fraction of the corpus's requirements are in
the encoding**, alongside every verdict. "No conflicts found" over 40% of a
spec is a weak claim, and one that reads as a strong one unless the denominator
travels with it. This is the same defect FR-050-AC-14 fixed for the coverage
rollup — a `0/0` that rendered as `100%` — and the analysis must not
reintroduce it in a new surface.

**Unchanged from the original proposal**: the coverage/symbol work
(FR-050/FR-051) keeps trace ids and statement excerpts machine-addressable,
since every candidate design consumes per-statement trace identities. That has
since shipped, so this consequence is satisfied rather than pending.

**Gate cleared**: the requirement-quality lint pack (agent-ix/quire-rs#83,
FR-056) raises the strict/EARS-conformant fraction of the corpus, which is the
formalizable fraction. It landed in P1, so the sequencing note this decision
was waiting on no longer blocks the Q2 spike.
