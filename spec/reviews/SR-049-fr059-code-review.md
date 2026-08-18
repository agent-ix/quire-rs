---
id: SR-049
title: "code-review of FR-059 declared-vocabulary coverage (CR-076)"
type: SpecReview
analysis: code-review
scope: "src/corpus/vocabulary_coverage.rs, src/traceability.rs, src/corpus/spec.rs, tests/vocabulary_coverage.rs"
review_set: subset
---

## Summary

Reviewed FR-059, which landed after the Wave B review (SR-047) and so was not covered by it.
Two findings, neither high. The design defect the fit-check exposed — one fact reported twelve
times — was found and fixed **before** the code was committed, so it is recorded here as history
rather than as an open finding.

## Verdict

**CONDITIONAL** — no high-severity finding; two low-severity observations, one of which is a
deliberate design choice now written down rather than left implicit.

## Findings

| ID      | Severity | Summary                                                                                     | Refs                                    |
| ------- | -------- | ------------------------------------------------------------------------------------------- | --------------------------------------- |
| FND-001 | low      | `enum_at` resolves the shallowest match when a schema declares the same field in two branches; deterministic but arbitrary, and undocumented | src/corpus/vocabulary_coverage.rs:96     |
| FND-002 | low      | The check uses a third P2 manifest key against the program's stated budget of two            | src/traceability.rs:103                  |

## Detail

### FND-001 — deterministic, but the rule is implicit

`enum_at` walks the frontmatter schema breadth-first looking for
`properties.<field>.enum`, rather than a fixed path, because a real schema wraps its properties
in `allOf` / `$defs` / `oneOf` branches and a flat-path reader would report "no such vocabulary"
for a schema that plainly declares one.

**Determinism was verified rather than assumed**: `serde_json`'s `Map` is a sorted-key `BTreeMap`
in this build (probed directly — `{"z","a","m"}` iterates `a, m, z`), so the walk order is stable
across runs and NFR-006 holds.

What is *not* written down is the resolution rule when a schema declares the same field in two
branches with different enums: shallowest wins, then alphabetically by key. That is a real
answer and a defensible one, but a reader has to derive it from the traversal.

### FND-002 — the third manifest key

`traceability.vocabulary_coverage` is the third new manifest key of Phase 2, against a stated
budget of two (FR-058 spent both on `required_relations` and `acyclic_edges`).

`ColumnVocabularies` — the existing `traceability.vocabularies` key — is specifically the
test-type column contract that a matrix and its coverage rollup must agree on. Folding a
frontmatter-field projection into it would make one key mean two unrelated things, which is worse
than the extra key. Flagged in PR #175 for a decision rather than absorbed silently.

### The defect the fit-check found, fixed pre-commit

The first implementation reported every declared value as unowned when the bundle contained no
document of the projected archetype. Over 243 `~/dev` bundles, **90 carry no NFR at all**, so that
turned one fact into **1080 of the sweep's 2792 findings**.

Fixed to a single finding naming how many values are unowned; the sweep fell to **1802**. Recorded
because the reasoning matters more than the change: this is *not* the forbidden "widen a rule
because it lowers the count". No bundle that was reported stopped being reported, the number is
still in the message, and one specific true statement replaced twelve vaguer ones.

## Coverage

`make ci` passes: `fmt-check`, `clippy -D warnings`, the full test suite including the 8 new
`tests/vocabulary_coverage.rs` cases, `cargo deny`, and all seven static audits.

`TC-916` and `TC-918` exist because of this review's own concern — a check that silently reports
nothing is indistinguishable from success — and both were written before the code shipped.
