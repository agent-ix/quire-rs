---
id: StR-007
title: "Assurance decisions use source-grounded, interpretable data"
type: StR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-006"
    type: "depends_on"
    cardinality: "1:1"
---
# StR-007: Assurance decisions use source-grounded, interpretable data

## Stakeholder Need

Assurance owners require that Quire shall supply every claim and relationship
with the exact source revision and declared contract and module versions needed
to resolve and interpret it, so an audit can compare revisions without
reconstructing Quire's semantics from markdown.

## Rationale

Coverage and property payloads expose useful slices, while the bounded corpus
retains relationships that no stable machine contract exports. Consumers such
as Quoin consequently re-read frontmatter, recreate relationship direction,
and infer whether absent data means missing, inapplicable, or unread. Those
copies can disagree with the engine while still producing a plausible assurance
case. Source locations, statement digests, explicit states, and version premises
make that disagreement detectable at the boundary.

## Validation Criteria

| ID | Criteria | Validation |
|----|----------|------------|
| StR-007-VC-1 | Every exported artifact, obligation, evidence symbol, and relationship resolves to a source location at the export's pinned repository revision. | Demonstration |
| StR-007-VC-2 | A consumer refuses an export whose contract version, module version, or module-schema digest it does not recognize. | Test |
| StR-007-VC-3 | A consumer can distinguish a required relationship that is missing from one that is not applicable or could not be evaluated. | Test |
| StR-007-VC-4 | A compatibility fixture detects an unintended change to an identity, relationship kind, source locator, or state token. | Test |

## Stakeholders

The primary stakeholders are assurance owners and reviewers who decide whether
evidence supports a release or control claim. Quoin and other assurance tools
are affected consumers; module authors own the vocabulary those tools must
interpret.

## Context and Assumptions

The source repository and module set are available at immutable revisions. The
consumer validates the export before using it. Quire remains the parser and
bounded-corpus authority; the assurance consumer remains responsible for
freshness and sufficiency verdicts.

## Dependencies

**Upstream**: the bounded corpus need in
[StR-006](./StR-006-whole-spec-corpus.md). **Downstream**: the assurance-export
workflow in [US-018](../usecase/US-018-consume-assurance-export.md).

## Priority and Risk (Informative)

Priority is P1. If unmet, assurance tooling can silently audit a graph or source
revision different from the one Quire read.
