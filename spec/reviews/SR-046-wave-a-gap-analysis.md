---
id: SR-046
title: "gap-analysis of ADR-0011 Phase 2 Wave A (CR-067..CR-072)"
type: SpecReview
analysis: gap-analysis
scope: "spec/functional/FR-026, FR-022, FR-057; spec/tests.md; spec/assets/adr/0010; src/corpus/, src/grammar/, src/writeback.rs"
review_set: subset
---

## Summary

Post-implementation gate over the five Wave A tickets — quire-rs#89 (CR-067), #161 (FR-057 /
CR-068), #84 (CR-069), #86 (CR-070) and the quoin#48 pilot half (CR-071), plus the review fixes
in CR-072. All five are complete. The matrix reconciliation found **22 acceptance criteria that
were unbacked because their `Verification` cells named no test** — fixed here, taking the corpus
from 436 unbacked rows to 414. Two pre-existing gaps recorded and not fixed.

## Verdict

**CONDITIONAL** — no incomplete task, no status lie, no `high` finding. Two `low` findings are
pre-existing and honestly declared; one `medium` was found and fixed during this analysis.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | 22 acceptance criteria across FR-026 and FR-057 carried a bare `Test` verification cell naming no test, so the rollup counted them unbacked — fixed | spec/functional/FR-026-intra-spec-reference-resolution.md:114, spec/functional/FR-057-corpus-check-severity.md:95 |
| FND-002 | low | FR-026-AC-7 has no implemented test; TC-492 is declared `🚧` and is honestly reported as unbacked | spec/tests.md:341 |
| FND-003 | low | 15 test symbols carry no trace binding, two of which are *named* for a TC id the extractor does not bind — all pre-existing, none from Wave A | src/symbols/trace.rs, tests/markdown_validation.rs |
| FND-004 | low | The new `make mutants-fr` / `make mutants-scope` targets and their example have no owning requirement | Makefile:127, examples/mutants_scope.rs |

## Detail

### 1. Plan completion

Wave A is tracked as GitHub issues under epic agent-ix/quire-rs#81 rather than a `plan/` bundle,
so there is no `Task` frontmatter to assert on. Completion was checked against the tickets:

| Ticket | Deliverable | State |
| --- | --- | --- |
| #89 | `ix://` URI grammar (CR-067) | done — FR-026-AC-12/13, CON-1, TC-880..882 |
| #161 | corpus severity registry (CR-068) | done — FR-057 with AC-1..10, CON-1..2, TC-883..889 |
| #84 | metamorphic properties (CR-069) | done — FR-022-AC-6/7, TC-890..896, TC-024 relabelled |
| #86 | ADR-0010 decisions (CR-070) | done — status *Partially decided*, Q1/Q4 decided, Q2 spiked to #164, Q3 deferred |
| quoin#48 | mutation pilot half (CR-071) | done — FR-026-AC-14, TC-897, `make mutants-fr` |

`plan/Plan-001-ac-grammar-coverage/` is a **different, earlier** plan and was not in scope. Its
`plan.md` references a `TC-750` gap that is stale — TC-750 is implemented at
`src/symbols/trace.rs:870`, recorded in CR-069.

### 2. Matrix verification

`quire coverage --scope . --json`, before and after this analysis:

| | before | after |
| --- | --- | --- |
| backed / total | 461 / 1094 | 461 / 1094 |
| unbacked rows | 436 | **414** |
| **status lies** | **0** | **0** |
| untracked symbols | 15 | 15 |
| no-symbol rows | 1 | 1 |

**Zero status lies is the load-bearing number**: no matrix row claims `✅` while nothing backs
it. That is the defect this program exists to prevent, and the wave did not introduce one.

**FND-001.** Every acceptance criterion added by CR-067, CR-068 and CR-071 was written with a bare
`| Test |` verification cell. In this repo an AC is backed when a symbol carries the AC id
*or* when its `Verification` cell names a TC that resolves — so a bare `Test` is unbacked by
construction, no matter how good the test is. All the tests existed and bound correctly
(`FR-057` resolves to TC-883..889 through the matrix), but the ACs themselves did not reconcile.

CR-069 got this right — FR-022-AC-6/7 were written `Test (TC-896)` and reconciled from the start —
which is what made the inconsistency visible. The 11 pre-existing FR-026 criteria had the same
bare cells and were annotated in the same pass, since the mapping was already authoritative in
`spec/tests.md`.

**FND-002.** FR-026-AC-7 asks for a proptest showing resolution time linear in edge count and
classification identical across thread counts. TC-492 exists as a matrix row marked `🚧`, and no
test carries the id. This is a real gap in FR-026's coverage — but it is **declared**, not hidden,
which is why it reports as an unbacked row rather than a status lie. Pre-existing; not created or
worsened by Wave A.

**FND-003.** Fifteen test symbols bind no trace id. Two of them —
`tc806_legacy_comma_list_binds_every_id` and `tc798_comment_stripping_is_string_aware` — are
*named* after a TC id, which means the name-based legacy form is not binding them. All fifteen
predate Wave A; every test added by CR-067..CR-072 binds (verified independently through the
CR-071 scope tool).

### 3. Underspecified code

**FND-004.** CR-071 added `make mutants-fr`, `make mutants-scope` and
`examples/mutants_scope.rs` with no owning requirement. Recorded rather than fixed: the repo
already ships `make fuzz`, `make loom` and `make sanitize` on the same footing — hardening and
measurement tooling that is not itself a product behaviour — so minting an FR for this one would
be inconsistent with the four beside it. If the mutation score ever becomes an obligation
(quoin#48's plumbing half), the requirement arrives with it.

Everything else added by the wave has an owning requirement: `BundleFinding.pack`/`.severity`,
`route`, `bridged`, `posture_tier` and the `pack` module are FR-057; the URI grammar and the
`OnceLock` compile-once change are FR-026; the writeback separator and empty-section fixes are
FR-022-AC-6/7; the fixpoint loops are FR-042 and FR-049. No stub, no `todo!()`, no placeholder
return was found in the changed surface.

### 4. Semantic review

**Skipped** as the optional step. Two substitutes ran during implementation and are stronger than
a read-through would have been: CR-069 stated the normalizers and writeback as metamorphic
relations and **four real defects fell out**, and CR-071 ran `cargo-mutants` over FR-026's traced
file, where the surviving mutants named a criterion asserted in prose and nowhere else
(FR-026-AC-14). Both are intent↔test↔code checks executed rather than judged.

## Coverage

- Plan completion: 5 of 5 tickets complete.
- Matrix: 461 / 1094 trace ids backed; **0 status lies**; 414 unbacked rows, down 22 from this
  analysis.
- Wave A's own criteria: **15 of 15 backed** after FND-001 (FR-026-AC-12/13/14 + CON-1,
  FR-057-AC-1..10 + CON-1, FR-022-AC-6/7).
- Reverse gap: 1 finding, accepted with reasons.
- Semantic review: skipped; see §4.
