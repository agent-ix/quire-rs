---
id: SR-16498
title: "integrity review of FR-062 Python and TypeScript structural implements syntax positions (quire-rs PR 628)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-rs@8db9e04; spec/functional/FR-062-implements-relation.md; spec/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-062
    type: references
---

# Integrity review of FR-062 Python and TypeScript syntax positions (quire-rs PR 628)

## Summary

Ticket: PLAT-866. Checked completeness, consistency and atomicity of the new section and rows. Each new AC is mapped to one TC in both the TC table and the audit table; TC rows carry kind, priority and status. AC-18 mirrors AC-12 (load, round-trip, refusals naming the form); AC-25 mirrors the Rust partial-match rule; AC-27 mirrors AC-17. FR-062-AC-16 stays consistent with AC-26's "a module that still declares a textual form binds as before". The cross-requirement contradiction with FR-051-AC-22 is recorded in SR-16496 FND-002 and not repeated here.

The two new finding reasons are specified like the Rust one: FR-057 severity `warning`, named fields (form, file, 1-based line, verbatim candidate; the partial one also the qualified symbol), no relation, no fallback, and a nonmatching candidate yields nothing. `implements-marker-unattached` has no Rust counterpart because a Rust attribute always attaches. Both are reasons the QA corpus and the coverage reason registry (`src/coverage.rs` reason list, `FindingGuidance`, `tests/fixtures/coverage_baseline/expected.json`) will need entries for when the code lands; noted, not a finding against this spec PR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Naming of the two new reasons. The partial-match condition now has two reasons, `partial-implements-attribute-match` (Rust) and `partial-implements-syntax-match` (Python/TypeScript), and the section does not say the Rust reason stays as it is; `implements-marker-unattached` uses "marker", which in this spec and FR-051 names the `trace_tags.markers` (`verifies`) list, not an `implements` form. Fix: say the Rust reason is unchanged and why the new one differs, and spell the unattached reason with `implements`/`syntax` wording (for example `implements-wrapper-unattached`). | spec/functional/FR-062-implements-relation.md (A wrapper that attaches to nothing; Partial matches) |

## Verdict

Approve with one low wording finding.

## Dispositions

Checked at c7eadc1 (via 592e7ad, d8a3c7a). Line numbers refer to that head.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The unattached reason is now `implements-syntax-unattached` (L217-227, AC-24, TC-2206), so it no longer uses "marker". `partial-implements-syntax-match` is scoped to the new section, and the Rust reason in the CR-191 section is untouched. |
