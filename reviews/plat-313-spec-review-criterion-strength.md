---
id: SR-16342
title: "Criterion strength — PLAT-313 FR-050-AC-61 (quire-rs PR 609)"
type: SpecReview
analysis: criterion-strength
scope: "agent-ix/quire-rs; spec/functional/FR-050-declarative-coverage-computation.md, spec/tests.md"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-050", type: references}
---

## Summary

Ticket: PLAT-313. PR head: `3630737`. The Jev client is not installed on this machine, so the calibrated Jev judgment was skipped. This is a manual judgment of whether AC-61 can fail.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

**PASS, clean (manual).** AC-61 can fail. These implementations would each fail it:

- a list form that mints only the first entry's column (TC-2083..2085);
- a one-name target that also reads `ID`/`Test ID` (TC-2086, the adverse control);
- a header read twice (TC-2086);
- a diagnostic that fires when one of two entries misses (TC-2087);
- an empty list accepted at load, or a scalar serialized back as a one-element list (TC-2087).

Adverse-case coverage for the requirement is present: control and refusal vectors, not only the happy path. TC-2084 and TC-2085 repeat TC-2083 over other source languages. Table reading does not depend on source language, so they add little discriminating power, but they are not vacuous.

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-050-AC-61 | examined | Can fail; adverse cases present |
| TC-2083..TC-2087 | examined | Each discriminates |
