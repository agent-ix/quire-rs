---
id: SR-20261103118136
title: "Criterion strength — PLAT-1357 FR-038-AC-9/AC-10 (quire-rs PR 620)"
type: SpecReview
analysis: criterion-strength
scope: "agent-ix/quire-rs PR620 head 65437a4; spec/functional/FR-038-okf-bundle-validation.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-038
    type: references
---

## Summary

Ticket: PLAT-1357. No Jev client is installed on this machine, so the calibrated Jev judgment was skipped. This is a manual judgment of whether each AC can fail.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Every AC-10 vector declares the lowercase package `acme/widgets`. An implementation that lowercases only the link authority (for example inside `ix_authority`) and still compares byte-for-byte passes every clause, yet skips `ix://acme/widgets/X` as foreign for a library caller that declares `Acme/Widgets`. The "stores the package as given" clause pins the getter, not the comparison. Fix: add a mixed-case declared-package vector, e.g. package `Acme/Widgets` with absent `ix://acme/WIDGETS/X` still `dangling-reference`. | spec/functional/FR-038-okf-bundle-validation.md:106 |

## Verdict

Changes requested. AC-9 can fail: an implementation that skips with no package declared, skips own-authority or two-segment links, or reports foreign links fails it, and it carries adverse cases. AC-10 can fail against the current byte-exact code (the `ix://Acme/widgets/X` vector), but it does not tell a symmetric fold from a one-sided one (FND-001).

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-038-AC-9 | examined | Can fail; adverse cases present |
| FR-038-AC-10 | examined | FND-001 |
