---
id: SR-20261103118161
title: "Spec-integrity analysis of FR-049-AC-10..17, FR-057-AC-11 and TC-2228..TC-2234 (quire-rs PR 649)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-rs@a2e2b4c0; spec/functional/FR-049-verification-reference-integrity.md, spec/functional/FR-057-corpus-check-severity.md, spec/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-049
    type: references
  - target: ix://agent-ix/quire-rs/FR-057
    type: references
---

# Spec-integrity analysis of FR-049-AC-10..17, FR-057-AC-11 and TC-2228..TC-2234 (quire-rs PR 649)

## Summary

Ticket: PLAT-1471.

- **Mapping.** AC-10 and AC-11 map to TC-2228, AC-12 to TC-2229, AC-13 to TC-2230, AC-14 to TC-2231, AC-15 to TC-2232, AC-16 and AC-17 to TC-2233, and FR-057-AC-11 to TC-2234. Every audit row agrees with its TC's `Traces To` cell.
- **FR-050-AC-58 audit check.** Checked by hand, because `make validate` is a cargo example and this was a spec-only review. All nine new ACs are defined by a leading table cell and have an audit row. The coverage ranges `AC-1..17` (FR-049) and `AC-1..11` (FR-057) declare them.
- **Ids.** TC-2228..TC-2234 are each defined once. TC-2227 was the previous maximum.
- **Planned table.** The new rows are in the main `## Test Case Summary` table (lines 1187-1193), before the `(planned)` sections, so they mint. `quire coverage --json` lists TC-2228..TC-2234 as `traces-to` rows binding their ACs. By contrast, TC-2227 (in a planned section) dangles in `quire validate --okf`.
- **Validation.** `quire validate --scope .` on the three files exits 0. The `--okf` bundle output is identical to origin/main.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | low | FR-049-AC-10..17 and FR-057-AC-11 use a bare `Test` in the Verification cell. FR-049-AC-1..9 and FR-057-AC-1..10 cite `Test (TC-nnn)`, and the same gap was fixed in PLAT-1386 (SR-20270 FND-002). A bare cell gives FR-049's own check nothing to verify from the AC side. The TCs mint, so citing them will not dangle. Fix: `Test (TC-2228)` for AC-10 and AC-11, `Test (TC-2229)`, `Test (TC-2230)`, `Test (TC-2231)`, `Test (TC-2232)`, `Test (TC-2233)` for AC-16 and AC-17, and `Test (TC-2234)` for FR-057-AC-11. | spec/functional/FR-049-verification-reference-integrity.md:115-122; spec/functional/FR-057-corpus-check-severity.md:106 | wrong-requirement |
| FND-002 | low | FR-049-AC-17 requires mismatch findings "interleaved with `dangling-trace-reference` findings, in the same order", but neither the AC nor the prose defines the order key. Today the engine sorts dangling findings alone by (path, id, message) and routes them as one batch (`src/corpus/trace_refs.rs`). "Interleaved" implies a combined key that two implementers would choose differently. Fix: state the key (for example path, id, reason, message), or drop "interleaved" and keep the repeat-run determinism. | spec/functional/FR-049-verification-reference-integrity.md:122 | wrong-requirement |

## Verdict

Structure is complete in both directions, and traces bind. Two low wording fixes.
