---
id: SR-048
title: "gap-analysis of ADR-0011 Phase 2 Wave B (FR-058, CR-073..CR-075)"
type: SpecReview
analysis: gap-analysis
scope: "src/corpus/required_relations.rs, src/traceability.rs, src/loader/mod.rs, tests/required_relations.rs, spec/tests.md"
review_set: subset
---

## Summary

Verified Wave B's engine half — FR-058 (CR-073), the load-time rejection of unexecutable
declarations (CR-074) and the dead-vocabulary check (CR-075) — against the Test Matrix and the
source tree. **Every one of the twelve Wave B test cases is backed by a real test carrying the
matching tag, and the repository has zero status lies.** Three findings, none high, all pre-existing
or scope-tracking rather than defects in the change.

Measured with **this branch's engine**, not the installed `quire` CLI: that is 0.22.0 against this
crate's 0.30.1, so it predates the reconciliation being audited. The harness is checked in as
`examples/wave_b_gap.rs` so the numbers can be re-derived rather than trusted.

## Verdict

**CONDITIONAL** — no unbacked Wave B test case, no status lie, and no high-severity finding. The
conditions are FND-001 (Wave B's declaration half is blocked downstream and the wave is therefore
not closeable yet) and FND-002 (15 pre-existing untracked symbols).

## Findings

| ID      | Severity | Summary                                                                                          | Refs                                     |
| ------- | -------- | ------------------------------------------------------------------------------------------------ | ---------------------------------------- |
| FND-001 | medium   | Wave B cannot close: the declaration half needs a 3-repo release chain before any module can declare a relation | agent-ix/spec-objects-security#5          |
| FND-002 | medium   | 15 tagged tests carry an FR/NFR id rather than a TC id, so they match no matrix row — all pre-existing | src/grammar/mod.rs, tests/markdown_validation.rs |
| FND-003 | low      | No plan bundle exists for Wave B; completion is tracked in the GitHub epic, so step 1 has nothing to assert against | plan/                                     |

## Detail

### FND-001 — the wave is verified but not closeable

The engine half is complete and tested. The **declaration** half is not startable: a module cannot
declare `traceability.required_relations` until both gates that reject the key ship — quire-rs's own
`deny_unknown_fields` (this branch) and `spec-artifacts-iso`'s `additionalProperties: false`
(agent-ix/spec-artifacts-iso#21). Engine → contract → module, three hops in that order.

This is scope tracking, not a defect: nothing in this branch can shorten the chain. Recorded so the
wave is not read as closed because its code is green.

### FND-002 — untracked symbols, all pre-existing

15 tests carry a tracking tag that names an **FR or NFR** rather than a test case, so the engine
finds a tagged test with no matrix row to reconcile it against:

| Tag form | Example | Count |
|---|---|---|
| FR/NFR id | `FR-048`, `FR-036`, `NFR-016`, `StR-005` | 9 |
| AC id | `FR-025-AC-9`, `FR-003-AC-6` | 5 |
| Fixture id | `TC-999` in `tests/fixtures/coverage_baseline/` | 1 |

**None is from Wave B**, and every one predates this branch. The fixture case (`TC-999`) is correct
by construction — it is a fixture asserting that a scope covers nothing declared. The other 14 are
real drift worth a separate ticket rather than an in-wave fix.

### What was checked and found clean

- **All twelve Wave B test cases backed.** TC-898 … TC-909, each resolving to a test function whose
  name carries the tag.
- **Zero status lies repo-wide.** No matrix row claims ✅ while no tagged test backs it. The 414
  unbacked rows are all correctly marked 🚧 — the matrix is honest about what is not implemented,
  which is the property that matters more than the ratio.
- **No underspecified Wave B code.** Every function added traces to an acceptance criterion:
  `validate_required_relations` → AC-1..9, `check_declaration_is_live` → AC-11, `check_relation` →
  AC-1..4, `target_is_accepted` → AC-4, `check_acyclic` and `shortest_cycle_through` → AC-5, the new
  `TraceabilityModel::validate` rules → AC-10, and the `merge_traceability` arm → CON-1.

## Coverage

Reconciliation over this repository, computed by `quire_rs::coverage::compute` on this branch:

| Measure | Value |
|---|---|
| Backed rows / total reference rows | 473 / 1117 |
| Unbacked rows | 414 |
| Rows exempt by declared method (CR-041) | 1 |
| Untracked symbols | 15 |
| **Status lies** | **0** |

"Reference rows" counts matrix cells referencing a trace id, which is not the same population as
matrix TC rows (573) — the two are reported separately rather than blended, because a single test
case row can be referenced from several places.

The optional semantic review (intent ↔ test ↔ code) was **skipped**, as requested.
