---
id: SR-20261115183663
title: "Code review of PLAT-1441 implementation PR #690"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; src/validate_document.rs, src/corpus/relation_report.rs, src/loader/mod.rs, src/semantic/frontmatter_owned.rs, src/semantic/record.rs, src/semantic/surface.rs, src/semantic/context.rs, src/traceability.rs, tests/it/frontmatter_owned_keys.rs, tests/it/record_carry.rs, tests/fixtures/traceability/record-keys/ (PR #690, head 63152cd)"
review_set: all
---

## Summary

Ticket: PLAT-1441. Implementation PR #690 at frozen head `63152cd`: production Rust plus
tests, +1271/-46 over 15 files. This file also holds the `rust-review` lane, as the
code-review dispatch rule requires. The work was mapped against the merged FR-069
(frontmatter-owned record keys, AC-38..46), FR-081 (AC-6, AC-18..21), FR-058 (AC-36/37) and
FR-084 (AC-20..29) from #688.

Evidence: `cargo test --test it -- frontmatter_owned_keys record_carry required_relations`
gave 49 passed and 0 failed. `cargo clippy --all-targets -- -D warnings` was clean. The full
gates (`make ci`, `make ci-python`) were not run, because they run once on the final head.

The coder's `object`/`type` archetype assumption was checked against `concept_type()`
(`src/query.rs:28`, `src/corpus/walk.rs:79`), which reads `type` only. Required-relation
endpoints match on `concept_type()` (`src/corpus/required_relations.rs:300`, and
`accepted_kinds` in `relation_report.rs`). The FR-058-AC-36 load check validates `record_keys`
against the `from`/`to` archetypes, which are `type` names. The FR-084 Document shape takes
`archetype` from `type`. The spec therefore resolves the question to `type`. The coder's
object-first order is not supported (FND-001), so no owner decision is needed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `selected_record` resolves the record archetype from `object` before `type`, but FR-058-AC-36 validates `record_keys` against the `type` endpoint archetypes and FR-084 names the document's archetype by `type`. A key can be load-checked against one schema and carried from another, and a record can be carried where AC-24 requires omission. | src/validate_document.rs:545 |
| FND-002 | medium | Each `Context::document` call re-parses the document and re-runs full semantic extraction plus `data_schema` validation, and rebuilds the context by cloning the module and bundle imports. The same document is re-extracted once per row and per linked entry in every relation with `record_keys`, with no memoization. | src/corpus/relation_report.rs:560 |
| FND-003 | low | `selected_record` drops the whole record when any semantic kind is unavailable (`any_kind_unavailable()`). FR-084's omission list does not name that condition, so an unreadable unrelated section silences the selected keys. | src/validate_document.rs:557 |
| FND-004 | low | `undeclared_record_key` reads only the root `properties` of the archetype's `data_schema`, so a key the resolved schema declares through a root `allOf`/`$ref` is refused at load. | src/loader/mod.rs:1797 |

### FND-001 detail

`selected_record` (`src/validate_document.rs:545-551`) iterates `["object", "type"]` and
takes the first archetype that has a data validator and a semantic module. Failure cases:

- Load passes, record dropped. A document `type: Classification` has a `data_schema` that
  declares `cats`, so the AC-36 check passes for a relation `to: [Classification]`. The same
  document also has `object: Thing`, a semantic object type with a closed schema and no
  `cats`. The document states `cats: [a]`. The record is extracted against `Thing` and is
  then invalid or has no `cats`, so `record` is omitted. The endpoint archetype declares the
  key and the document states it.
- Record carried where AC-24 requires omission. The `type` archetype has no `data_schema`
  but `object:` does, and load passed because another endpoint declares the key. The record
  is carried anyway.

The object-first branch is also untested: every `record_carry` document is `type`-only.

Fix: resolve the archetype by `concept_type()` alone, so the report agrees with endpoint
matching, the AC-36 check and the `archetype` field. If object-typed endpoints are wanted
for real data-classification artifacts, that is a spec change (a CR on FR-058/FR-084). An
implementation branch must not silently pick object-first.

### FND-002 detail

`selected_record` calls `parse_document` and `extract_semantic`, then validates. Inside
that path, `FrontmatterOwned::project` calls `parse_document` again from both
`extract_record` and `compose_frontmatter_record`. The corpus already holds the parsed
document, and the result per `(slot, archetype)` is the same for every relation.

Fix: compute `selected_record` once per slot per report, for example with a
`BTreeMap<usize, Option<Map>>` built lazily in `report_of`, and reuse the corpus's parsed
frontmatter. The record is the same for every relation, so the per-relation key filter can
run on the cached full record.

### FND-003 detail

`semantic_findings` returns without a `semantic.record-invalid` finding when a kind is
unavailable, so record validation has not "failed" in FR-084's sense. `selected_record`
still returns `None`. Two examples:

- A record-sections reference key under the registry surface, which has no bundle index:
  FR-081-AC-16 gives it `no-bundle-index`.
- A row error in an unrelated `## Relationships` table.

In both cases `cats` disappears from the report. Either name the condition in FR-084 or
limit the check to the selected keys.

## Verdict

Changes requested (medium: FND-001, FND-002). Examined with no finding:

- the `record_keys` shape parse (`traceability.rs` `parse_record_keys`, `deserialize_present`
  keeping `null` as a refusal);
- the frontmatter-owned projection: `mapped_roots` skips `P`, so `K == P` is not
  self-duplicated, and `P != K` with the literal `K` duplicates as AC-45 requires;
- the explicit-`null` leaf, `regex`/`multiple` through `eval_located`, and the
  whole-value-mustache filter;
- the FR-081 section branch: an error with no value, a warning with a value, and no
  `recordSources`/availability entry;
- `finding_line` keying on the dotted path, so a failure is located at `P`;
- `RowDocument.record` serde placement after `criteria` and `skip_serializing_if`.

Rust lane: idiomatic overall. It returns `Option`/`Result` and never panics outside tests;
the loader collects `BTreeMap`/`BTreeSet`. `SemanticContext` losing `Eq` is a pub API change,
which is acceptable pre-release.
