---
id: SR-20261115183665
title: "Code review (Rust lane) of PLAT-1441 runtime PR #690, independent pass"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; src/validate_document.rs, src/corpus/relation_report.rs, src/loader/mod.rs, src/semantic/frontmatter_owned.rs, src/semantic/record.rs, src/semantic/surface.rs, src/semantic/context.rs, src/traceability.rs, tests/it/frontmatter_owned_keys.rs, tests/it/record_carry.rs, tests/fixtures/traceability/record-keys/ (PR #690, head 63152cd57f246a777eaca5871fa53b0f8908a6b0)"
review_set: all
---

## Summary

Ticket: PLAT-1441. PR #690 at head `63152cd`, three commits, 15 files, +1271/-46.
Production Rust plus integration tests. The `rust-review` lane is in this file.
The PR implements FR-069 frontmatter-owned record keys (AC-38..46), the FR-081
section rule for those keys (AC-6, AC-18..21), FR-058 `record_keys` (AC-36/37)
and FR-084 carried records (AC-20..29).

This is the second reviewer pass at this head. The first pass (run
`plat-1441-pr690-r1`, SR-20261115183663) was re-checked against the code, not
copied. Its FND-001 is restated below with a corrected analysis. Its FND-004 is
not carried over: `RecordSchema::new` (`src/semantic/record.rs:290`) also reads
only root `properties`, so the load check agrees with the engine's own idea of a
record key.

Evidence: the code was read, and a focused `cargo llvm-cov` run covered the PR's
own integration tests (see gap analysis SR-20261115183666). `quoin` and `quire`
were blocked by the permission layer. No aggregate gate was run, because the
lead owns `make ci`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `selected_record` extracts under the `object:` archetype first, then falls back to `type`. The two branches disagree with different parts of the spec: the FR-058-AC-36 load check and the FR-084 `archetype` field use `type`, while registry validation computes the FR-069 record only for `object:`. The PR picks an order on its own, without a CR. | src/validate_document.rs:545 |
| FND-002 | medium | Every `RowDocument` built for a relation with `record_keys` re-parses the document and re-runs full semantic extraction and schema validation. Nothing is memoized across rows, linked entries or relations, and `FrontmatterOwned::project` parses the document again inside extraction. | src/corpus/relation_report.rs:562 |
| FND-003 | low | `Owned::resolve` drops a frontmatter-owned value whose source line cannot be found (`find_map(l.line)?`, `frontmatter_key_line(..)?`). `key_line` returns `None` for a key produced by a YAML merge key, so a stated value becomes absent, which FR-069 forbids. | src/semantic/frontmatter_owned.rs:105 |
| FND-004 | low | `selected_record` returns `None` whenever any semantic kind is unavailable. FR-084's omission list does not include that condition, so an unreadable section unrelated to the selected keys silences them. | src/validate_document.rs:557 |
| FND-005 | low | The new `undeclared_record_key` and its doc comment were inserted between `carried_field_conflict`'s doc comment and that function. The FR-090 doc now belongs to the wrong function, and `carried_field_conflict` has none. | src/loader/mod.rs:1778 |

### FND-001 detail

`selected_record` (`src/validate_document.rs:545-551`) walks `["object", "type"]`
and takes the first archetype that has a data validator and a semantic module.

- The load check (`undeclared_record_key`, `src/loader/mod.rs:1787`) validates each
  key against the `from`/`to` archetypes. Those are matched on `type`
  (`concept_type`, `src/query.rs:28`). FR-084 Serialization takes `archetype` from
  `type`.
- `validate_in_registry_core` (`src/validate_document.rs:740`) runs
  `semantic_findings`, which composes the FR-069 record, only for the `object:`
  archetype. A `type`-only document never gets a composed record or a
  `semantic.record-invalid` finding from validation.

So with object-first resolution, a key accepted at load because the `type`
endpoint declares it is read from the `object` archetype's schema instead. The
record can then be dropped (a closed object schema without the key), or a key can
be carried from a schema the load check never consulted. With the `type`
fallback, the report computes and enforces a record that document validation
never computes. `record_carry.rs` relies entirely on this fallback, since every
document is `type: Classification`, and CLS-003 is "invalid" only in the report.
FR-084 says "omit when record validation of the document fails (any
`semantic.record-invalid` …)", which reads as the validation findings. No test
covers the `object:` branch.

The spec does not settle which archetype supplies "the document's composed
semantic record". Raise a CR on FR-084 (and FR-058-AC-36) that names it, then
make the load check, the report and validation use the same archetype. Do not
change the order silently in either direction.

### FND-002 detail

For each relation with `record_keys`, `Context::document`
(`src/corpus/relation_report.rs:560-569`) calls `selected_record` once for the
obliged document and once per linked entry. Each call:

- runs `parse_document`;
- builds a `SemanticContext` that clones the module and every bundle export list;
- runs `extract_semantic`, whose `extract_record` and `compose_frontmatter_record`
  each call `FrontmatterOwned::project`, and each of those runs `parse_document`;
- validates against `data_schema`.

The full record is the same for every relation and row a slot appears in. The
fixture's four `record_keys` relations already extract each classification up to
four times. NFR-001/002 make latency normative. Compute the full record once per
slot per report (a lazy `BTreeMap<usize, Option<Map>>` in `report_of`), then
filter it per relation.

### FND-003 detail

`resolve` (`src/semantic/frontmatter_owned.rs:103-121`) needs a line before it
returns a value. `frontmatter_lines::key_line` documents that it returns `None`
for "a key produced by a YAML merge key". In that case the parsed frontmatter
holds `P`, but `K` is not projected. `compose_frontmatter_record` has already
skipped `P` as a mapped root, so the value disappears without a diagnostic, and
under `record-sections` `frontmatter_valued` misses it, which turns the FR-081
warning into an error. Fall back to `FRONTMATTER_LINE` rather than dropping the
value.

### FND-004 detail

`semantic_findings` skips record validation when a kind is unavailable, so the
record was never judged invalid. FR-084 lists three omission conditions, and this
is not one of them. Either name the condition in FR-084 or restrict the check to
the selected keys' kinds.

## Verdict

Changes requested (medium: FND-001, FND-002).

Examined with no finding:

- the `record_keys` parse (`src/traceability.rs:620-644`), where `deserialize_present`
  keeps `null` as a refusal; empty, scalar, blank and repeated input are refused and
  name the relation;
- the FR-058-AC-36 load check, including `to: []` checking only `from`, a key
  declared by one endpoint passing, and failures attributed to each declaring
  module;
- `FrontmatterOwned::from_dsl`, `mapped_roots` (`K == P` is not self-duplicated,
  and `P != K` with a literal `K` duplicates, per AC-45), `leaf_is_null`, and
  `regex`/`multiple` through `eval_located`;
- the FR-081 branch in `extract_record` (`src/semantic/record.rs:518-543`): an
  error without a value, a warning with one, and no `recordSources` or availability
  entry;
- `finding_line`, whose dotted paths from `schema_validation_detail` split correctly
  to the owned key;
- `RowDocument.record` placed after `criteria` with `skip_serializing_if`, and the
  BTreeMap/sorted key order;
- no `unwrap` or panic outside tests, and no `HashMap` added.

Rust lane: idiomatic overall (`Option`/`Result`, `let-else`, `is_some_and`).
`SemanticContext` dropping `Eq` is a public API change, which is acceptable
pre-release. The PR touches no `src/python` or `src/grammar` code, so
`make ci-python` is not triggered.
