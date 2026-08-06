---
id: US-017
title: "Agent Verifies Requirement Coverage Deterministically"
type: US
relationships:
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-006"
    type: "traces_to"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-049"
    type: "exercises"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-050"
    type: "exercises"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-051"
    type: "exercises"
---
# US-017: Agent Verifies Requirement Coverage Deterministically

## Story

**As an** LLM agent closing out an implementation plan
**I want** a deterministic, machine-readable rollup of how the spec's
acceptance criteria, the Test Matrix, and the test code actually connect
**So that** I can verify the matrix's coverage claims without hand-grepping the
source tree, and trust that two runs over the same tree agree.

The story is about mechanical reconciliation only; judging whether a test is a
*good* test remains an agent-review concern.

## Context

Today the `gap-analysis` workflow answers "is the Test Matrix real?" by grepping
the test tree for tracking tags (`TC-xxx`, `FR-xxx-AC-x`, `Trace:` lines) and
reconciling the hits against matrix rows by hand. The grep step is slow,
non-deterministic across agents (each one improvises patterns), and its results
are not reusable by other consumers (semantic review, the knowledge graph).
The corpus engine ([US-012](./US-012-agent-audits-whole-spec.md)) already loads
the whole spec; what is missing is a deterministic view of the code side and a
generic reconciliation over the two.

## Acceptance Examples (Illustrative)

These examples clarify the agent's expectations. They are illustrative only —
not test cases and not verification criteria.

### US-017-EX-1: Matrix overclaim is surfaced

- **Given** a matrix row marked complete whose test case has no tagged test in the tree
- **When** the agent runs the coverage rollup
- **Then** the row is reported as a status lie, with the requirement and test-case ids

### US-017-EX-2: Untracked test is surfaced

- **Given** a test function carrying a trace tag that appears in no matrix row
- **When** the agent runs the coverage rollup
- **Then** the test is reported as untracked, with its file and symbol name

## Dependencies (Contextual)

Upstream: the whole-spec corpus ([StR-006](../stakeholder/StR-006-whole-spec-corpus.md))
and the module-declared traceability vocabulary. Downstream: the `gap-analysis`
review workflow consumes the rollup instead of grepping; the knowledge graph
ingests the extracted symbol relations. These are potential relationships, not
formal traceability.

## Priority and Risk (Informative)

Business value is high: coverage verification is a release gate for every
spec-driven repo, and today it silently degrades with agent grep quality. Risk
if unmet: matrices keep overclaiming and gates pass on unbacked claims. This
information is for planning only.
