---
id: SR-20261113091250
title: "Gap analysis — PLAT-1578 FR-090 and FR-091 engine (quire-rs PR 675)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; spec/functional/FR-090-declared-carried-fields.md (AC-1..13, CON-1..2), spec/functional/FR-091-ordered-field-consistency.md (AC-1..18, CON-1..2), tests/it/field_rules.rs, src/corpus/field_rules.rs, src/coverage.rs, src/loader/mod.rs"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/FR-090", type: references}
  - {target: "ix://agent-ix/quire-rs/FR-091", type: references}
---

## Summary

Ticket: PLAT-1578. PR head: `6d569a7`. Plan completion: not assessed. All 13 FR-090 ACs and all 18 FR-091 ACs carry a `#[trace]` tag on a test in `tests/it/field_rules.rs` (27 tests, all pass). Each tagged test was read against its criterion. A mutation sample of 14 source mutants was run with the focused suite: 8 killed, 5 survived, 1 equivalent.

| Mutant | Result |
| --- | --- |
| `low >= high` → `low > high` | killed (FR-091-AC-2) |
| drop `found.dedup()` | killed (FR-091-AC-7) |
| carry null values | killed (FR-090-AC-2) |
| skip enum-mismatch detection | killed (FR-090-AC-13) |
| drop `to` kind filter | killed (FR-091-AC-13) |
| skip carried-field key conflict | killed (FR-090-AC-10) |
| case-fold vocabulary match | killed (FR-091-AC-17) |
| drop `found.sort()` | **survived** (FND-001) |
| drop `via_ids.sort()` | **survived** (FND-002) |
| drop required_fields `exclude` | **survived** (FND-003) |
| ignore required_fields `where` | **survived** (FND-003) |
| drop `via` archetype filter | **survived** (FND-003) |
| drop model-level `exclude` | **survived** (FND-003) |
| drop from-side `compared = false` | equivalent (SR-20261113091249 FND-003) |

## Verdict

**PASS with fixes.** Coverage is complete by tag. 26 of the 31 criteria are backed by a test that fails on the broken state. Two tagged assertions are vacuous (FND-001, FND-002). Untested narrowing branches and partial assertions are low.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-091-AC-14's test passes with `run.found.sort()` deleted. Writing the files to disk in reverse order does not change the walk order, so "supplied in reverse order" is never exercised. The check-order assertion also holds by accident, because the rules push in consistency → required → vocabulary order and `tier-below-prism` < `unassigned-tier` < `undeclared-tier-value`. Fix: supply documents in an order that differs from the sorted order, for example via `Spec::from_members` with a reversed member list, or ids whose path order differs from id order. Name the checks so push order differs from lexical order, and assert the via and target tie-breaks. | tests/it/field_rules.rs:679-733 |
| FND-002 | medium | FR-091-AC-4's "names both via documents in lexical id order" passes with `via_ids.sort()` deleted. The via set is a `BTreeSet<usize>` in slot order, and slots follow path order, which equals id order in the fixture (`spec/BRG-000.md` < `spec/BRG-001.md`). The comment "authored before the first by id" does not change load order. Fix: give the via documents ids whose order differs from their file paths, or use a cross-member `repo:id` via, so slot order and lexical id order disagree. | tests/it/field_rules.rs:514-536 |
| FND-003 | low | FR-091 states several narrowing branches that no test exercises: required_fields `exclude` and `where` ("narrow the obliged documents exactly as for required_relations"), the via hop's archetype filter ("an edge whose far end is outside the hop's archetypes joins nothing", where FR-091-AC-13 covers only the target kind), and the model-level `exclude` applied to all three rules. Each survives removal. Fix: one focused test per branch. | src/corpus/field_rules.rs:249, 392-394, 495-511 |
| FND-004 | low | Three tests assert less than their criterion. FR-090-AC-12 says "with and without the keys", but only the run with carried fields is compared. FR-091-AC-6 says "output identical to the output before this requirement", but only finding absence is checked. FR-091-AC-18's order-independence is asserted with both modules loaded, where a vocabulary exists, not in the AC's no-`carried_vocabulary` state. Fix: compare a run without the declarations, and run the AC-18 order check on a two-module set that yields no vocabulary. | tests/it/field_rules.rs:430-469, 834-865, 927-958 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-090-AC-1 | examined | Tagged test asserts carried equals the value and no new top-level key; correct |
| FR-090-AC-2 | examined | Absent and null keys give no carried; correct, kills the carry-null mutant |
| FR-090-AC-3 | examined | Related document infers nothing (traces_to, refines); correct |
| FR-090-AC-4 | examined | Criticality and carried independent; obligations byte-identical; correct |
| FR-090-AC-5 | examined | No carried or carried_vocabulary without declaration; correct |
| FR-090-AC-6 | examined | Out-of-enum value carried as written; correct |
| FR-090-AC-7 | examined | Structural failures name the entry; correct |
| FR-090-AC-8 | examined | Merge-time liveness; correct |
| FR-090-AC-9 | examined | Vocabulary in schema order; correct |
| FR-090-AC-10 | examined | Union and conflict naming both modules; correct, kills the conflict mutant |
| FR-090-AC-11 | examined | Schema admits new keys and rejects strays; correct |
| FR-090-AC-12 | examined | FND-004 |
| FR-090-AC-13 | examined | Mismatch both orders; correct, kills the mismatch mutant |
| FR-091-AC-1..3, 5, 7..13, 15..17 | examined | Tagged tests assert the criterion; correct |
| FR-091-AC-4 | examined | FND-002 |
| FR-091-AC-6 | examined | FND-004 |
| FR-091-AC-14 | examined | FND-001 |
| FR-091-AC-18 | examined | FND-004 |
| FR-090-CON-1, FR-091-CON-1 | examined | Inspection: no names in non-test source; the AC-15 neutral fixture passes |
| FR-090-CON-2, FR-091-CON-2 | examined | Inspection: no assign or write path |
