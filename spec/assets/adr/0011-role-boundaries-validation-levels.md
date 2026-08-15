---
id: ADR-0011
title: "Role boundaries: validation levels and capability roles"
type: ADR
---

# ADR 0011: Role boundaries — validation levels and capability roles

**Status**: Accepted
**Date**: 2026-08-15
**Decision authority**: kreneskyp

## Context

The verification program keeps adding capabilities — property extraction
(`quire properties --json`), spec-correctness (quoin), coverage binding
(FR-050/FR-051), the AC grammar (FR-042/FR-047), mutation scoring (queued,
agent-ix/quoin#48), SMT consistency analysis (ADR 0010), requirement-quality
lints, and traceability/suspect-link checks (planned). Each one re-raises the
same placement question: does this belong in the quire engine, in quoin, or in
the consumer repo? Is it a test, or something that defines what must be
tested?

The mission frames the answer: quoin/quire guide agents and people to build
better software together — by guiding them to write better specs and adhere
to higher testing standards. Guidance tooling states and audits obligations;
it does not take over the consumer's execution.

Three prior decisions already point one way:

- FR-042/FR-048: engine checks are deterministic-only; no LLM inside quire.
- ADR 0010: solver dependencies live in a separate optional crate
  (`quire-analyze`); the core, wasm, and Python-wheel builds never depend on
  it; orchestration involving an LLM is a quoin skill.
- FR-050: declarative coverage *computation* is a quire feature; the *verdict
  policy* stays in quoin.

This ADR generalizes those precedents into one model so future capabilities
stop being case-by-case debates.

## Decision

### 1. Three validation levels

Every verification capability targets exactly one subject:

| Level | Subject under validation | Who executes | quire role | quoin role |
|---|---|---|---|---|
| **L0** | quire/quoin themselves | their own repos' CI | is the target | is the target |
| **L1** | the spec corpus | quire/quoin binaries | Validator (lints, grammar, graph checks, analysis encodings) | runs spec-level analyses (solving, graph queries) and keeps the evidence store |
| **L2** | a consumer implementation | **the consumer repo's CI, always** | Advisor (emits verification obligations) | Generator (skeletons/seeds) + Auditor (binds evidence, ratchets, flags vacuous/stale) |

### 2. Four capability roles

- **Validator** — a deterministic fact about the spec alone ("this
  requirement is non-singular", "this guard space has a gap"). At L1 the spec
  *is* the artifact under test, so validators are tests — of the spec.
- **Advisor** — emits **verification obligations**: data stating what must be
  verified and by what method. Not a test. `quire properties --json` is the
  existing instance.
- **Generator** — turns an obligation into a test skeleton or seed in a
  target language. Output lands in the consumer repo and is owned by the
  consumer after generation.
- **Auditor** — binds evidence (test runs, coverage, mutation scores) back to
  obligations; applies verdict policy, ratchets, freshness and vacuity
  checks.

### 3. Placement rule

Placement follows the capability's input domain:

- Needs only the spec corpus, deterministic → **quire-rs core**.
- Needs the spec corpus plus a heavy dependency (solver, embeddings) →
  **separate optional crate/extra** (`quire-analyze` pattern, ADR 0010);
  core, wasm, and wheel builds never depend on it.
- Needs the consumer's code, test results, or repo state → **quoin**.
- Needs an LLM → **quoin skill** (orchestration); never inside the engine.

### 4. Invariants

1. **quoin executes nothing at L2.** It defines what must run, optionally
   generates scaffolding, and audits the evidence that comes back. Consumer
   suites run in consumer CI.
2. **The obligation record is the quire↔quoin contract**: requirement id +
   statement content hash + verification method + parameters + criticality.
   Every capability either produces obligations (quire) or discharges/audits
   them (quoin + consumer).
3. **Heavy dependencies are optional even when recommended.** Solver and
   similar dependencies confine to L1 analysis components with graceful
   "not installed" degradation (ADR 0010's leaning).
4. **Generated artifacts are consumer-owned.** Regeneration is a proposal
   (diff), never a silent overwrite.

## Consequences

Concrete placements this settles:

- **Mutation testing** — L2: consumer CI runs it; quoin advises where a
  mutation-score threshold is obligatory (criticality on the obligation) and
  audits the reported score. Mutation-testing quire-rs itself is L0.
- **Fuzzing** — cargo-fuzz on the quire-rs parser (NFR-011) is L0
  self-testing. A possible future "generate a fuzz harness from a consumer's
  input grammar" is an L2 Generator feature; the two are distinct.
- **SMT consistency analysis** — L1: encoding export is engine-adjacent,
  solving is `quire-analyze`, LLM formalization orchestration is a quoin
  skill (unchanged from ADR 0010).
- **Requirement-quality lints** (ambiguity lexicon, non-singular,
  unverifiable predicates, agentless passive, modal discipline) — L1
  Validators in quire-rs core, on the FR-042 grammar framework.
- **Suspect links / evidence freshness / vacuous evidence** — L1 Auditor
  checks in quoin over its evidence store; the content-hash inputs come from
  quire's obligation records.

Existing features already conform (FR-050 verdict split, ADR 0010 placement);
no migration is required. New capability proposals must state their level and
role before placement is debated.
