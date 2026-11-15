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

## New findings (disposition pass 1)

Pass 1 re-read FR-058, FR-069, FR-081 and FR-084 at head `5731e42`.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | low | The `record_keys` load check validates keys against "`from`, and each archetype in `to`". FR-058 defines `to: []` as "any document", so for such a relation only `from`'s schema is checked. A key that only counterpart documents declare is then refused, and FR-058 does not say whether that is intended. Also, FR-058-AC-36 refuses a `record_keys` that is "not a list", but no body statement says so. | spec/functional/FR-058-upward-trace-completeness.md:136-140 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | FR-069 now states the change on purpose. For K equal to P, "the projection is unchanged; only the FR-081 section rule changes". Precedence is stated: an FR-081 section under a frontmatter-owned key "is not a source for `semantic.duplicate-record-key`". AC-38 is amended with that exception, and FR-081-AC-19 asserts no duplicate-key error. The corpus measurement belongs to implementation, not to this spec. |
| FND-002 | fixed | Each case is now a rule tied to FR-011 and FR-033: `multiple` (AC-43), `regex` and `assert` (AC-43), a multi-segment `path` (AC-42), a present `null` (AC-44), and P and K both authored, which gives `semantic.duplicate-record-key` with two frontmatter sources (AC-45). AC-43 matches FR-011-AC-15 (group 1 projection, non-match drops) and FR-011-AC-21 (`multiple` wraps every resolved value). |
| FND-003 | fixed | FR-084 omits `record` when record validation fails (`semantic.record-invalid` or `semantic.duplicate-record-key`), and orders keys lexicographically (AC-24, AC-25). |
| FND-004 | still-open | New this pass; the fix round has not run yet. |

- FND-001 after_excerpt (`spec/functional/FR-069-semantic-module-contract-at-load.md:300-303`): `A FR-081 section under a frontmatter-owned key is not a source for semantic.duplicate-record-key. FR-081 governs it, and only when record-sections is enabled: the frontmatter value wins and the section is a frontmatter-owned diagnostic.`
- FND-002 after_excerpt (`spec/functional/FR-069-semantic-module-contract-at-load.md:287-299`): `An explicit null at the path is present, not absent: the engine SHALL project K as null and data_schema validates it ...` / `If one document authors both the mapped key P (with P different from K) and the literal key K in frontmatter, then the engine SHALL treat them as two frontmatter sources of K and emit semantic.duplicate-record-key by the rule below; the record carries no K.`
- FND-003 after_excerpt (`spec/functional/FR-084-two-way-relation-coverage-report.md:92-96`): `The engine SHALL order the keys of record lexicographically by key name ...` / `The engine SHALL omit record ... when record validation of the document fails (any semantic.record-invalid or semantic.duplicate-record-key error).`

Pass-1 verdict: approved for this method. FND-004 is low and does not block.
