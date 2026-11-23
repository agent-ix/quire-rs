---
id: SR-323
title: "EARS review of PLAT-1831 outputCatalog producers (quire-rs PR 713)"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-rs; spec/functional/FR-073-generic-clause-sets.md (PR 713 diff only, head 5a7ac5dde4020113d16ba5237151e72bb15d9a1c)"
review_set: subset
date: 2026-11-23
---

## Summary

Ticket: PLAT-1831. Checked the new `producers` paragraph and rules (lines 41-62) and FR-073-AC-7..AC-10 for EARS form, modality and atomicity. The ACs are single-trigger event or state clauses a test can fail. The new normative rules carry no modal, and two sentences state consumer semantics quire does not implement.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The new rules ("`producers` is optional", "Loading and `validate` reject ...", "Quire validates the shape of `producers` only", "serializes as ... and round-trips unchanged") are normative but use no SHALL; FR-073's only modal is the opening "Quire shall load". Fix: phrase them as "Quire SHALL accept an optional `producers` ...", "Quire SHALL reject ...", "Quire SHALL NOT check kinds against a vocabulary". | spec/functional/FR-073-generic-clause-sets.md:41-62 |
| FND-002 | low | "a record whose kind is in the list satisfies the output" and "`producers` is the only producer binding the catalog carries: it names record kinds, not commands or tools" describe consumer (quoin FR-211) semantics that quire never evaluates, so no quire test can fail them. Fix: mark them informative and cite quoin FR-211 as the consumer, or drop them. | spec/functional/FR-073-generic-clause-sets.md:43-44; spec/functional/FR-073-generic-clause-sets.md:57-58 |

## Verdict

Wording only; no behaviour changes. AC-7..AC-10 are atomic enough: AC-9 lists three rejection inputs of one load-time check, and AC-7 covers one round-trip property in two encodings.
