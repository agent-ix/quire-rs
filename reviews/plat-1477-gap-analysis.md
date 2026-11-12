---
id: SR-20261112163907
title: "Gap analysis of PLAT-1477 FR-058 where selector (quire-rs PR 654)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs@253ccad; spec/functional/FR-058-upward-trace-completeness.md (FR-058-AC-7, AC-12..22, CON-3); src/corpus/required_relations.rs; src/traceability.rs; tests/it/required_relations_where.rs; tests/it/required_relations.rs; spec/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-058
    type: references
---

# Gap analysis of PLAT-1477 FR-058 where selector (quire-rs PR 654)

## Summary

Ticket: PLAT-1477. Scope is `git diff origin/main...253ccad`, checked against the full FR-058 on origin/main. Plan completion was not assessed.

**Matrix.** `quoin matrix --repo .` (quoin 0.30.1) marks every criterion from FR-058-AC-12 to AC-22 as `tagged`. FR-058-AC-6, AC-7 and AC-9 are `untagged`. That predates this PR, but AC-7 matters here (FND-001).

**Trace bindings.** Each of TC-2248..2257 asserts the AC it is tagged with:

- TC-2248: AC-12 and AC-15.
- TC-2249: AC-13.
- TC-2250: AC-14.
- TC-2251: AC-16.
- TC-2252: AC-17.
- TC-2253: AC-18.
- TC-2254: AC-19.
- TC-2255: AC-20.
- TC-2256: AC-21.
- TC-2257: AC-22.

CON-3 is verified by Inspection: no field or value name appears in the `src/` diff.

**Mutation sweep.** Each mutant was applied to the review worktree, followed by `cargo test --test it required_relations` and a restore. 13 of 14 were killed:

- A. An absent field is selected. Killed by TC-2249.
- B. A list containing a listed value is selected. Killed by TC-2249.
- C1. `where` also filters the target of an outgoing edge. Killed by TC-2248.
- C2. `where` also filters the source of an incoming edge. Killed by TC-2248.
- D. `exclude` is ignored when `where` selects. Killed by TC-2251.
- E1. A whitespace-only `values` entry is accepted. Killed by TC-2254.
- E2. A whitespace-only `field` is accepted. Killed by TC-2254.
- F. Duplicate detection does not fold. Killed by TC-2254.
- G. A `values` entry is not folded in the AC-22 check. Killed by TC-2257.
- **H. Document values are not folded in the AC-22 held set. Survived (FND-002).**
- I. The dead-field trigger is removed. Killed by TC-2257.
- J. The dead-value trigger is removed. Killed by TC-2257.
- K. Old Strict routing through `posture_tier` is restored. Killed by TC-2253 and TC-2257.
- L. The AC-20 suffix is dropped. Killed by TC-2255.

The AC-22 case where a differently-cased entry matches the enum and is not reported (`values: [Security, " safety "]`) is live through the enum route alone for `" safety "`, because no document holds safety. Mutation G kills it.

**Status rows.** The `spec/tests.md` changes are accurate: TC-2248..2257 are marked ✅, and the FR-058 row reads "AC-1..22 ✅ Implemented". `spec_validate` reports 204 documents and 0 failed. The FR-050-AC-58 audit-table check passes.

**Scope note.** The ticket also asks for `where` declarations in the spec-objects-security and spec-objects-safety manifests. Those are other repositories and are not in this PR, so merging this PR does not complete the ticket.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The PR changes FR-058-AC-7 behaviour: required-relation, cycle and dead-declaration findings are now warnings under `Strict`. AC-7's own test does not cover the change. TC-903 runs only `Okf` and is tagged with no AC (`#[trace("TC-903")]`), and the matrix shows AC-7 (and AC-6) as untagged. The Strict regression is caught only indirectly, by TC-2253 and TC-2257, which use `where` relations and are tagged with AC-18 and AC-22. A relation without `where`, a cycle and an undeclared-kind finding are never validated under `Strict`. Fix: in TC-903, loop over both postures and assert `is_valid()`, include a cycle and a dead-kind declaration, and tag it with `FR-058-AC-6` and `FR-058-AC-7`. | tests/it/required_relations.rs:223-269, spec/functional/FR-058-upward-trace-completeness.md (FR-058-AC-7) |
| FND-002 | low | Part of FR-058-AC-22 is untested: an entry "held" by a document is compared after trimming and ASCII case folding. If the held set is built from raw document values (mutation H), every test still passes, because every held-route case in TC-2257 uses the same case. Add a document holding `" Secrity "` against `values: [secrity]` and assert no dead-vocabulary finding. | tests/it/required_relations_where.rs:454-456, src/corpus/required_relations.rs:198-207 |
| FND-003 | low | FR-058-AC-15 says "`to` is constrained exactly as without `where`", but no test with `where` uses an edge to or from a kind outside `to`. TC-2248 only uses edges that satisfy `to: [Threat]`. Add a selected NFR whose only `mitigates` edge targets a non-Threat document and assert it is reported. | tests/it/required_relations_where.rs:111-156 |

## Verdict

AC-12..22 are all implemented and tagged, and 13 of 14 mutants are killed. Fix FND-001 (medium) in this PR. FND-002 and FND-003 are small test additions.

## Dispositions

Round 1, checked at 7a35156. The full sweep was re-run, adding two mutants. 15 of 16 were killed. The Strict-routing mutant K is now killed by TC-903 as well as TC-2253 and TC-2257.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | TC-903 is tagged `FR-058-AC-6` and `FR-058-AC-7` and runs under both `Okf` and `Strict`. Under both postures, a relation without `where`, a `cyclic-derives_from` cycle and an `undeclared-relation-vocabulary` dead kind all fire as warnings, and the report is valid. Mapping `trace:orphan-fr`, `trace:cyclic-derives_from` or `trace:undeclared-relation-vocabulary` to `error` promotes that finding. Mapping `orphan-fr` to `off` leaves its sibling reporting. Mutation K (old Strict routing) now fails TC-903. |
| FND-002 | still-open | The new TC-2257 case pairs `values: [security, " SECRITY "]` with a document holding the lowercase `secrity`. Only the `values` entry needs folding there, and mutation G already covers that. A held set built from raw document values still contains `secrity`, so mutation H (no fold on the held side) survives again at 7a35156. Fix: the document must hold a value that differs from the folded form, for example `quality_attribute: " Secrity "`, against `values: [secrity]`. |
| FND-003 | fixed | TC-2248 adds a selected NFR whose only `mitigates` edge targets an NFR (N-011, reported), one whose edge targets `T-404`, which is absent from the bundle (N-012, reported), and an incoming `mitigated_by` edge from an NFR that does not satisfy the incoming relation (N-011 reported under `untargeted-security-nfr`). |
