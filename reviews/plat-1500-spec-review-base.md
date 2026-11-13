---
id: SR-20261113091248
title: "Spec review (base) — PLAT-1500 FR-049 AC-26/28 reversal, AC-29/30, CR-419 (quire-rs PR 671)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs@0191395; spec/functional/FR-049-verification-reference-integrity.md"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-049", type: references}
---

## Summary

Ticket: PLAT-1500. Spec review of the FR-049 edit: the rewritten "The rule is not this requirement's bundle check alone" paragraph, AC-26 and AC-28 reversed, new AC-29 and AC-30, and the CR-419 note. `quire validate` passes on the file. This review ran the base review only. The EARS, integrity and dependency sub-analyses were not run separately: the edit adds no requirement statement and no relationship edge, and its structural consistency is covered here.

## Verdict

**PASS with findings: one medium, two low.** The reversal is consistent. No other FR-049 text still says "first table" for minting, obligations or the rollup, and the CR-419 note states the change and its cause. On main, CR-419 appears only in historical PLAT-1526 review rows (a note since deleted). The PLAT-1418 engine worktree has no CR-419 at present. That is a collision risk to watch, not a defect, and it is flagged only.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The per-section rule names only `id-column-matches-nothing` and the status-column finding. Every reader of a declared section now reads every table, including the FR-053 obligation inventory. Its `MissingColumn` and `MissingTable` causes have no rule saying whether they are per section or per table, so two implementers would differ. The code chose per table (code-review FND-001). Fix: state that every per-section column cause, the inventory's included, follows the same first-table and no-table-carries-it rule, or state the exception. | spec/functional/FR-049-verification-reference-integrity.md:140-151 |
| FND-002 | low | The body says the findings "describe the section" and fire "only when no table under that heading carries the column". AC-30 says "per section". When two sections share heading text, "that heading" and "the section" mean different things, and the code follows the heading text (code-review FND-002). Fix: say "under that section". | spec/functional/FR-049-verification-reference-integrity.md:148-151 |
| FND-003 | low | The legend sentence cites "(FR-049-AC-29, AC-30)". AC-29 is about obligation derivation, not the per-section findings, so the citation should be AC-26 and AC-30. | spec/functional/FR-049-verification-reference-integrity.md:151 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-049 (body, "The rule is not this requirement's bundle check alone") | examined | FND-001, FND-002 |
| FR-049-AC-26 | examined | Clean; testable, reversal consistent |
| FR-049-AC-28 | examined | Clean; includes fenced control |
| FR-049-AC-29 | examined | Clean |
| FR-049-AC-30 | examined | Clean; three testable clauses |
| CR-419 note | examined | Clean; numbering free on main (collision watch with PLAT-1418 flagged only) |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1 (fe3f6bb): the body names the FR-053 selected inventory and FR-061 combinatorial sources, and makes the inventory's `MissingColumn` cause per section. AC-30 includes the inventory and an inventory over a trailing legend being Complete. |
| FND-002 | fixed | Round 1: body reads "one matched section is the unit; two sections sharing a heading text are two" and "no table of that section"; AC-30 says the same. |
| FND-003 | fixed | Round 1: the citation now reads "(FR-049-AC-26, AC-30)". |
| FND-004 | fixed | Round 3 (59032fa): the body and AC-32 limit the no-rows rule to a table "in a section where another table carries it". Where no table carries the column, the rows stay and both paths report it (skipped row and `obligation-row-states-nothing`; `MissingColumn`), consistent with AC-30. |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | medium | FR-049-AC-32 and the round-2 body sentence say any table lacking a required column "contributes no rows ... no `obligation-row-states-nothing` diagnostic", with no per-section qualifier. Read literally, a section whose only table misspells the statement column must report nothing on the coverage path, while AC-30 requires the inventory to raise `MissingColumn` for the same section. That specifies a silent zero and contradicts the per-section rule. Fix: limit AC-32 to a table whose section has another table carrying the column, and state the coverage-path signal when no table does. | spec/functional/FR-049-verification-reference-integrity.md:150, 208 |
