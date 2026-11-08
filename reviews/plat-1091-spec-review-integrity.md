---
id: SR-16312
title: "spec-review (integrity) of four malformed spec table rows (quire-rs PR 599)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-rs@fcef99e0; spec/functional/FR-026-intra-spec-reference-resolution.md; spec/functional/FR-070-typed-properties-extraction.md; spec/functional/FR-077-trace-search-index.md; spec/reviews/SR-070-semantic-extraction-integrity.md; spec/reviews/SR-068-semantic-extraction-base.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-026
    type: references
  - target: ix://agent-ix/quire-rs/FR-070
    type: references
  - target: ix://agent-ix/quire-rs/FR-077
    type: references
---

# Spec review (integrity) of four malformed spec table rows (quire-rs PR 599)

## Summary

Ticket: PLAT-1091. Gate: `spec_validate` at `origin/main` with both
providers at `origin/main` reports 4 failed documents, all of them row-width
errors: FR-026-AC-14 has 5 cells against 3, FR-070-AC-6 has 4 against 3,
FR-077-CON-3 has 3 against 4, and SR-070 FND-327 has 5 against 4. At the PR
head it reports 176 documents and 0 failed. The row-width assert came from
`ffb7af0` (#590, PLAT-217). A main CI dispatch at `eb2a041` on 2026-11-03
validated the same 176 documents with 0 failures, so #590 turned main's
validate red and this PR turns it green again.

Examined, meaning preserved:

- FR-026-AC-14 was added in `14d515d` (CR-071). The text still describes
  mutating `&&` to `||`. Only the pipes inside the code span are escaped.
- FR-070-AC-6 and SR-070 FND-327 were added in `17b80e4`. The pattern
  `/a,b|c/` and `parent | ConfigVersion` keep their pipes. The table parser
  turns `\|` back into `|` in cell text (`src/query.rs` CR-007 tests), so
  the extracted AC text and the "pattern text carried verbatim" clause do not
  change.
- FR-077-CON-3 was added in `7efe616` (#477) without its `Type` cell. The fix
  adds `Design`, which matches CON-1 and CON-2. `Type` is free text in this
  repo (26 FR constraints use `Design`). Verification `Test (TC-1894)` is
  unchanged.

The reviewer also scanned every Markdown table under `spec/`, `plan/`,
`docs/`, `quality/`, `requirements/` and `reviews/` with the engine's own
splitting rule (an unescaped `|` is a delimiter, inside code spans too). The
same scan at `origin/main` finds exactly the four fixed rows plus the two
findings below, so it agrees with the gate.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | [other] This is the same defect class, and the gate still misses it. The SR-068 `## Dispositions` row FND-286 contains `` `info\|warning\|error` `` with both pipes unescaped (shown escaped here), which gives 4 cells against a 2-column header. The row-width assert covers only schema-declared tables, and `## Dispositions` is not one, so `spec_validate` passes. A GFM render drops the extra cells, so the disposition reads "severities stay `info" and loses `warning`, `error` and its `(FR-072 Outputs, AC-9)` reference. Fix: put a backslash before both pipes. | spec/reviews/SR-068-semantic-extraction-base.md:46 | correct-requirement-no-evidence |
| FND-002 | low | [other] The same class appears outside the validated `spec/` root. Two top-level review records quote the stray fragment `` `\| FR-051-AC-17 \| ✅ \|` `` with the pipes unescaped inside a code span (shown escaped here): 7 cells against 4 in the gap-analysis file, 9 against 4 in the retro file. Their rendered `Summary` is cut at the fragment and the `Refs` column is lost. Fix: escape the pipes inside those code spans. The text stays the same; only the escaping changes. | reviews/2026-08-21-gap-analysis-v0.41.0-batch.md:42; reviews/2026-08-21-retro-code-review-prs-202-209.md:58 | correct-requirement-no-evidence |

## Verdict

The four fixes are pure escape or missing-cell repairs, and none of them
changes what an AC or finding means. Fix FND-001, which is the same defect in
the same `spec/` tree, and FND-002 in the same PR.
