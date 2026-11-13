---
id: SR-20261113104841
title: "Gap analysis — PLAT-1587 design_matrix engine op (quire-rs PR 674)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; spec/functional/FR-092-matrix-design-column.md, tests/it/design_matrix.rs, src/corpus/design_matrix.rs"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-092", type: references}
---

## Summary

Ticket: PLAT-1587. PR head: `e0d4c6f`. Plan completion: not assessed (planless). This analysis checks the computed matrix (`quire matrix --scope .`, quire 0.44.0 / engine 0.59.1) for FR-092, and compares each tagged test's assertions with its criterion's clauses. Semantic review was not run as a separate fan-out; the test-to-criterion comparison below was done inline.

## Verdict

**CONDITIONAL: no untagged criterion, three medium findings.**

- **Tagging.** FR-092-AC-1..12, AC-14..18 and CON-2 are `tagged`. AC-13 and CON-1 are Inspection and show `method-without-symbol`, as Inspection criteria do elsewhere in this repo (for example FR-086-AC-13).
- **CON-1, by inspection.** It holds: `design_join` is called once, and a block has exactly the six FR-086 fields, with `archetype` renamed `kind`.
- **AC-13, by inspection.** It holds for the code. The test tagged to it is a source grep; FND-002 judges that test.
- **Clauses asserted directly:**
  - AC-1, AC-2, AC-3/18, AC-4 (`from`, `where` and `exclude` all exercised), AC-5/16/CON-2 (string equality of serialized output, and the faulty-relation `coverage` control).
  - AC-6, AC-8/15, AC-9, AC-10, AC-11, AC-17.
  - The member-order half of AC-14, and its no-write half.
- **AC-12** is met by the whole suite using invented kinds, verbs and ids.
- **No stubs and no inflation.** All new code is owned by FR-092.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-092-AC-7 requires each block to equal the FR-086 row for the same inputs. No test compares a block with `design_join` output, and `kind_mismatch` is never non-empty in any test: it appears only as `[]` in AC-1. A target-kind-mismatch case compared against the matching `design_join` row's `state`, `dangling` and `kind_mismatch` (and the projected `linked`) would assert the clause. | tests/it/design_matrix.rs:177-223, 460-474 |
| FND-002 | medium | The FR-092-AC-13 test is a denylist grep that passes on the broken state it exists to catch. It looks for `Part"`, `Connection"`, `ADR"`, `Requirement"`, but the installed modules spell these archetypes `part`, `connection`, `allocation`, `FR` and `NFR`. It also omits real verbs such as `allocates` and `allocated_to`, so `== "part"` or `== "FR"` in the engine would pass. A grep is a valid assertion for this criterion only if it can fail. The file's code holds a single string literal, `"Option::is_none"`; asserting that every non-comment string literal is in that set catches any name. Otherwise drop the test and leave AC-13 as Inspection, as FR-086-AC-13 is. | tests/it/design_matrix.rs:642-667 |
| FND-003 | medium | The document-order half of FR-092-AC-14 is vacuous. `load_repo` sorts by path, so writing `GLY-002` before `GLY-001` builds the same corpus, and no test reorders `load.documents`. This is the same defect PR 672 fixed for FR-086-AC-14, by reversing the loaded documents before `Spec::from_members`. | tests/it/design_matrix.rs:669-701 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-092-AC-1 .. AC-6, AC-8 .. AC-12, AC-15 .. AC-18 | examined | Tagged and asserted |
| FR-092-AC-7 | examined | FND-001 |
| FR-092-AC-13 | examined | Inspection clean; test FND-002 |
| FR-092-AC-14 | examined | FND-003 |
| FR-092-CON-1 | examined | Inspection clean |
| FR-092-CON-2 | examined | Tagged and asserted |
