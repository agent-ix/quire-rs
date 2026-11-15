---
id: SR-20261115183661
title: "Failure-domain review of PLAT-1441 FR-069, FR-081, FR-084 amendments"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-rs; spec/functional/FR-069-semantic-module-contract-at-load.md, spec/functional/FR-081-record-sections-extraction.md, spec/functional/FR-084-two-way-relation-coverage-report.md (PR #688)"
review_set: all
---

## Summary

Ticket: PLAT-1441. Looks for unstated inputs, collisions and blast radius in the new
frontmatter-owned projection and the carried record. Three medium gaps.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Ownership is implicit for every existing `frontmatter_field` locator whose key is a `data_schema` property (e.g. security `severity`, `algorithm`, `likelihood`, `purl`), changing their section-plus-frontmatter result from FR-069-AC-38 `semantic.duplicate-record-key` error to a frontmatter-owned warning; AC-41 covers only archetypes without the locator, and no precedence between AC-38 and FR-081 is stated. | spec/functional/FR-069-semantic-module-contract-at-load.md:256-279,462 |
| FND-002 | medium | FR-069 projection leaves unstated: a locator with `multiple: true`, with `regex`/`assert`, or a multi-segment `path`; a present `P: null`; and a document authoring both `P` and literal `K` in frontmatter (two frontmatter sources for one key). | spec/functional/FR-069-semantic-module-contract-at-load.md:262-271 |
| FND-003 | medium | FR-084 "exactly as extracted" does not say whether a record that failed `data_schema` validation (e.g. `[medical]`) or lost keys to `semantic.duplicate-record-key` is carried, nor the key order inside `record`, which quire-cli FR-030 parity and AC-22 byte-identity depend on. | spec/functional/FR-084-two-way-relation-coverage-report.md:81-87 |

### FND-001 detail

FR-069's definition makes any `yield_pattern.match` key `K` with `from: frontmatter_field`
and `K` a schema property frontmatter-owned, without an opt-in. That is consistent with quoin
FR-121, but it changes behavior for existing modules. The amendment should state the change
deliberately: one AC with an existing K==P locator plus a same-named section, asserting the new
warning, and one sentence saying the FR-081 frontmatter-owned rule takes precedence over AC-38
for section sources. No compatibility layer is implied. Measure the corpus count of such
locators and section collisions before implementation.

### FND-002 detail

The text "the locator is declared without `multiple: true`" is a note, not a rule. Either refuse
`multiple: true` on a frontmatter-owned locator at load, or define its projection. Say whether
`regex`/`assert` filter the projected value. Restrict `path` to one segment, or define nested
lookup. Treat a present null as a value that fails type, not as absence. For P and K both
authored, the likely answer is `semantic.duplicate-record-key` with two frontmatter sources,
but it must be stated.

### FND-003 detail

State that `record` is the composed record after duplicate-key omission, carried regardless of
validation verdict (or omitted when invalid). Fix its key order (for example lexicographic,
as `BTreeMap` gives, or document-source order), so two implementations produce identical bytes.

## Verdict

Changes requested (medium). Also examined, with no defect found: the absence path (no K, no
default), the refusal of empty, duplicate and scalar values, location at the line of `P`, and
the reverse view's dangling/uncovered handling, which reuses existing FR-084 rules.
