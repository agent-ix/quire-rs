---
id: SR-20261113045744
title: "Code review (with rust-review lane) — PLAT-1554 two-way relation coverage report (quire-rs PR 666)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs@8b45bcb; CHANGELOG.md, src/corpus/mod.rs, src/corpus/relation_report.rs, src/corpus/required_relations.rs, src/lib.rs, tests/fixtures/traceability/relation-report/manifest.yaml, tests/it/main.rs, tests/it/relation_report.rs"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-084", type: references}
---

## Summary

Ticket: PLAT-1554. PR head `8b45bcb` (one commit over origin/main), scoped to `git diff origin/main...HEAD` (8 files, +1584/-37). Code review with the Rust lane folded in (rust-review, rust-style loaded; the repository CLAUDE.md takes precedence). Oracle: FR-084 on main, with FR-058, FR-050 and FR-085.

Checked and clean:

- The engine holds no hazard, threat, mitigation or safety-classification name (FR-084-AC-17, FR-058-CON-2). A grep of `relation_report.rs` and `required_relations.rs` finds none. The kinds and verbs live only in the fixture manifests.
- The report `state` matches the FR-058 finding by construction. Both paths now call the extracted `obliged` (selection: `from`, then `where`, then `exclude`, then non-empty id) and `is_satisfied`. The report uses a per-member view and member root, as `validate_bundle` does. `edges_from`/`edges_to` and `doc` are slot-keyed and do not depend on the view, so passing the unscoped corpus to `is_satisfied` gives the same answer. The refactor of `check_relation` keeps its behaviour.
- Verification is read only from `CoverageMatrixRequirement`/`CoverageMatrixStatus` (FR-084-CON-1). The status match is exhaustive: `tagged`/`qualified-record` are backed, `untagged`/`tagged-by-ignored-test` are unbacked, `method-without-symbol` is not measured, and `retired` is skipped. The row aggregate follows the spec rule, and `not-tracked` documents are excluded.
- No manifest key is added (FR-084-CON-2).
- Serialization key order follows the struct field order, and it matches FR-084 Serialization for the relation, summary, row, document, resolved and dangling `linked` entries, `dangling` and `kind_mismatch`. `repo` and `selector` are omitted when absent, arrays are always present, and summary counts are emitted at zero. Ordering: rows by (repo, path, id); `linked` resolved by (repo, id, verb) with absent repo first, then dangling by (reference, verb); `dangling` by (reference, verb); `kind_mismatch` by (repo, id, verb). All of it comes from `BTreeMap`/`BTreeSet`, with no `HashMap`.
- `kind_mismatch` is reported for outgoing edges only, which matches FR-058 and the FR-085 outgoing-only ruling. This is accepted as the coder noted it.
- No `unsafe`, and no `unwrap` outside tests. `self.members[member]` indexing is guarded by the member-count check. The error enum follows the repository's `CorpusError` convention.
- No ceremony is introduced: no pins, digests, ledgers or compatibility shims.
- Focused tests run by the reviewer: `cargo test --test it relation_report` gave 15 passed, 0 failed (own target dir, deleted afterwards).

## Verdict

**Changes requested: one medium, three low.** The engine is correct, chain-neutral and shares selection and satisfaction with FR-058, as the spec requires. FND-001 is an error that the spec does not define. It refuses a corpus with no declared relations, where FR-084 says the report is produced for any set of declared relations. The test-evidence findings are in the gap analysis SR-20261113045745.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `relation_report` returns `RelationReportError::NoRelationsDeclared` when no loaded module declares a `required_relations` entry (or there is no traceability model). FR-084 says quire-rs "SHALL produce this report for any loaded corpus and any set of declared relations", and the only error it defines is naming an undeclared relation (AC-9). An empty set should yield `{"relations": []}`. The extra error makes the quire-cli consumer treat an ordinary module as a failure. No AC or test covers it. Fix: drop the variant and return an empty report, or get the spec amended first. | src/corpus/relation_report.rs:43-45, src/corpus/relation_report.rs:303-305 |
| FND-002 | low | `LinkedEntry::{Resolved,Dangling}` carry `resolution: &'static str` set to the literals `"resolved"`/`"dangling"`, and `Dangling` carries constant `verification`/`criteria` fields. This is stringly-typed state that the type system could encode, against the CLAUDE.md design taste. A caller building the public enum can write any string. Fix: use a unit enum `Resolution { Resolved, Dangling }` with `serde(rename_all = "lowercase")`, and drop the always-constant fields from the public variant, for example with a custom `Serialize` or private fields. | src/corpus/relation_report.rs:146-168, src/corpus/relation_report.rs:453, src/corpus/relation_report.rs:513 |
| FND-003 | low | New code uses the legacy textual form `/// Implements: FR-084`. `quire matrix --scope .` now warns `legacy-implements-form` on `relation_report` ("the module declares an attribute form, so write `#[implements(\"FR-084\")]`"). It also warns `trace-id-outside-marker` on the `FR-084` mentions in the docs of `obliged`, `is_satisfied`, `Verification` and `RelationReport`. Fix: use `#[implements("FR-084")]` on `relation_report`, and drop or rephrase the bare ids in docstrings, so the PR adds no new matrix warnings. | src/corpus/relation_report.rs:277, src/corpus/required_relations.rs:357, src/corpus/required_relations.rs:388 |
| FND-004 | low | An untyped counterpart gets `archetype: ""`, through `concept_type().unwrap_or("")` in `document()` and `kind_of`. That empty-string sentinel is serialized into `linked[].archetype` and `kind_mismatch[].archetype`. FR-084 does not define it, and the FR-058 finding renders the same case as `(untyped)`, so the report and the finding describe one edge differently. Fix: model the archetype as `Option<String>` (omitted when absent), or state the spelling in FR-084 and use one shared rendering. | src/corpus/relation_report.rs:408, src/corpus/relation_report.rs:424 |

## Dispositions

Round 1, at head `b2f9b44` (fix commit checked against review head `8b45bcb`). Focused tests run by the reviewer: `cargo test --test it relation_report` gave 17 passed, and `cargo test --lib relation` gave 4 passed. `quire matrix` reports no FR-084 warning.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `NoRelationsDeclared` is removed, so an empty declaration set yields `{"relations": []}`. FR-084 now says "including none", and the new AC-18 is tested by `a_module_declaring_no_relation_yields_an_empty_report`. |
| FND-002 | fixed | The new `LinkResolution { Resolved, Dangling }` enum replaces the string literals. `LinkedEntry` no longer carries constant fields, and its hand-written `Serialize` emits `resolution`, `verification` and `criteria` from the variant. The exact-JSON test pins both shapes. |
| FND-003 | still-open | The docstring mentions are gone. However, `#[cfg_attr(test, ix_trace_rs::implements("FR-084"))]` binds nothing: `quire trace --id FR-084` now reports `Claims (0)`, where the head under review had an `implements` claim through the line form. Because ix-trace-rs is a dev-dependency, `cfg_attr(test)` expands only in test builds, and the static matcher does not read the `cfg_attr` spelling. Ruling: this is a defect. It drops the code-to-requirement claim, which is worse than the legacy-form warning it removed. Fix: restore `/// Implements: FR-084`, the binding form the other 19 entry points use while ix-trace-rs is dev-only, and accept the legacy-form warning. |
| FND-004 | fixed | `archetype` is `Option<String>` on `RowDocument` and `KindMismatchEntry`, omitted when untyped. FR-084 Serialization states the omission, and the exact-JSON test covers an untyped `UNT-001` link. |
