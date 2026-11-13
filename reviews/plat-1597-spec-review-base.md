---
id: SR-20261113104844
title: "Spec review (base) - PLAT-1597 FR-053/FR-082/FR-090 criticality_field amendments (quire-rs PR 676)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs@2930079; spec/functional/FR-053-obligation-record.md, spec/functional/FR-082-coverage-matrix-criticality.md, spec/functional/FR-090-declared-carried-fields.md"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-053", type: references}
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-082", type: references}
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-090", type: references}
---

# Spec review (base) - PLAT-1597 FR-053/FR-082/FR-090 criticality_field amendments (quire-rs PR 676)

## Summary

Ticket: PLAT-1597. PR head `2930079`. Base spec-review of the three amended FRs. `quire validate` on
the three files reports no failures. One `ac:unclassifiable` warning at FR-053 line 391 predates this
PR. Sub-analyses: EARS was not run because no FR statement changed, only description prose and ACs.
Integrity and consistency checks are folded into this base pass.

## Verdict

**FAIL: 1 high, 1 medium, 2 low.** The FR-053 item-5 rule is clear and implementable. The
precedence is pinned, absence is never defaulted, and the key is module data. The FR-082-AC-7
rescoping does not weaken an existing guarantee, because the key is opt-in. A source declaring no
`criticality_field` keeps exactly the old AC-7 behaviour, which existing tests still assert.
However, FR-082's unamended description contradicts the new rule.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-082's "Relation to FR-053 criticality" section, which this PR leaves unamended, contradicts the new FR-053 rule. It says FR-053's criticality is "read from a module-declared column", and that the engine "SHALL keep both as read and SHALL NOT let either supply, override or fill in the other". Under FR-053 item 5, a source declaring `criticality_field: criticality` (the stated spec-artifacts-iso follow-up) makes the document's frontmatter `criticality` fill in the obligation's criticality wherever the column cell is empty. Only AC-7 was rescoped, so the SHALL NOT still forbids this. Fix: amend that FR-082 paragraph to scope the prohibition to the engine's own behaviour (no implicit fill), and name FR-053's module-declared `criticality_field` as the one sanctioned source. | spec/functional/FR-082-coverage-matrix-criticality.md:33-38, spec/functional/FR-053-obligation-record.md:183-197 |
| FND-002 | medium | FR-053-AC-26 bundles four independent claims: no inference across relationships; unchanged output for undeclared sources; "the engine holds no frontmatter key name"; and coverage JSON carries the value and validates against the schema. The third claim is a static code property that no `Test` can fail. A failing test cannot say which clause broke. Fix: split it into separate ACs, and verify the no-key-name clause by Inspection, or drop it as already stated in item 5. | spec/functional/FR-053-obligation-record.md:414 |
| FND-003 | low | FR-053's description of the nested classification obligation still says "a criterion's method and criticality are all readable from the criterion's own row". This is no longer true once `criticality_field` supplies criticality from the document frontmatter. | spec/functional/FR-053-obligation-record.md:110-112 |
| FND-004 | low | FR-090 still says "A consumer that needs the fact for an obligation joins the obligation to its requirement entry, as it does for FR-082 criticality". With `criticality_field`, criticality can now ride on the obligation directly, so the "as it does" example is stale. The edited sentence above it is correct. | spec/functional/FR-090-declared-carried-fields.md:28-29 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-053 item 5 (criticality rule) | examined | Clear; precedence pinned |
| FR-053 CR note | examined | Clean |
| FR-053 traceability example (`criticality_field: criticality`) | examined | Clean |
| FR-053-AC-24 | examined | Testable; clean |
| FR-053-AC-25 | examined | Testable; clean |
| FR-053-AC-26 | examined | FND-002 |
| FR-053 nested-obligation prose | examined | FND-003 |
| FR-082-AC-7 (rescoped) | examined | Does not weaken: opt-in only |
| FR-082 "Relation to FR-053 criticality" | examined | FND-001 |
| FR-090 description | examined | FND-004 |
