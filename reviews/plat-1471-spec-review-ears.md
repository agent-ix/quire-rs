---
id: SR-20261103118163
title: "EARS analysis of FR-049 kind-typed reference statements (quire-rs PR 649)"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-rs@a2e2b4c0; spec/functional/FR-049-verification-reference-integrity.md (Kind-typed references), spec/functional/FR-057-corpus-check-severity.md:74"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-049
    type: references
---

# EARS analysis of FR-049 kind-typed reference statements (quire-rs PR 649)

## Summary

Ticket: PLAT-1471. Examined these statements: the optional-feature clause (`MAY carry kind_column, kind_targets`), the event-driven resolution clause (`the engine SHALL resolve ... first against the union ... then against the kind subset`), the mismatch response (`SHALL yield one reference-kind-mismatch`), the posture clause, the load-time rejection clause (`Module load SHALL reject`), and the FR-057 new-check rule. Each SHALL clause has one actor and one response.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | low | The matching and fallback rules are written in the indicative with no modal: "Kind matching trims the cell and compares ASCII case-insensitively", "is checked against the union only", "extracts no id and yields no reference". The clauses around them all use SHALL. A reader cannot tell normative from descriptive. Fix: "Kind matching SHALL trim ... and compare ASCII case-insensitively"; "... SHALL be checked against the union only". | spec/functional/FR-049-verification-reference-integrity.md:80-87 | wrong-requirement |

## Verdict

Conformant apart from one low modal fix. Confidence is low: other prose in this repo also mixes SHALL with the indicative.

## Dispositions

Round 1 at d5d98b3d.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | The rules now read "Kind matching SHALL trim the cell and compare ASCII case-insensitively", "SHALL be checked against the union only", and "SHALL likewise be checked against the union only, and the engine SHALL report ...". |
