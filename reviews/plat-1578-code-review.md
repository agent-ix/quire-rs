---
id: SR-20261113091249
title: "Code review — PLAT-1578 carried_fields, field_consistency, required_fields and vocabulary (quire-rs PR 675)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; git diff origin/main...6d569a7: src/corpus/field_rules.rs, src/traceability/field_rules.rs, src/traceability.rs, src/corpus/relation_edges.rs, src/corpus/validate.rs, src/coverage.rs, src/coverage/reconcile.rs, src/loader/mod.rs, schemas/output/coverage-v1.schema.json, CHANGELOG.md, tests/it/field_rules.rs, tests/it/output_contract.rs, tests/fixtures/traceability/field-rules/**"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/FR-090", type: references}
  - {target: "ix://agent-ix/quire-rs/FR-091", type: references}
---

## Summary

Ticket: PLAT-1578. PR head: `6d569a7` (origin/main is an ancestor). Code review with the rust-review lane folded in, scoped to `git diff origin/main...HEAD`. Focused run: `cargo test --test it field_rules` on the PR head, 27 passed. A local probe (uncommitted, in a scratch worktree, reverted) built a two-member corpus to check cross-member reporting.

## Verdict

**PASS with fixes.** One medium (FND-001) and three lows. The design is sound and matches the merged FR-090/FR-091 text.

- **No names in the engine (FR-090-CON-1, FR-091-CON-1).** The added non-test source holds no archetype, field, verb, rank or value. Every name comes from the manifest, and the rank is the position in a frontmatter-schema `enum` read through FR-059's `enum_at`. The only literal tokens are the engine's own check names (`undeclared-field-value`, `undeclared-relation-vocabulary`, `carried-vocabulary-mismatch`).
- **Never assigns or writes (FR-090-CON-2, FR-091-CON-2).** `carried_values` reads only the document's own frontmatter. No path writes a file, defaults a value, or uses one document's rank to fill another. `carried_vocabulary` is the schema enum and nothing else.
- **Schema.** `coverage-v1.schema.json` adds exactly `carried` (object, `minProperties: 1`, any values) on `CoverageMatrixRequirement` and `carried_vocabulary` (object of string arrays) on the top level. `tc856` and `tc859` in `tests/it/output_contract.rs` are updated. FR-090-AC-11's test checks that stray keys are still rejected at both levels.
- **CHANGELOG.** A `Breaking, PLAT-1578` entry names the new public fields on `CoverageMatrixRequirement`, `CoverageReport` and `TraceabilityModel`.
- **Edge reading.** The `via` and `edges` hops reuse `RelationEdges`, so CR-416 inverse reading, both-ways dedup and FR-085 cross-member resolution come for free. Several routes to one target collapse to one pair (`BTreeMap<target, BTreeSet<via>>`).
- **Merge.** Merge contradictions reset the traceability model, as the existing obligation-target conflict already does. `from` lists are united without regard to load order. The vocabulary is read with archetypes in name order, so it does not depend on load order.
- **Ceremony.** None found.

### Rulings on the coder's flagged ambiguities (against merged FR-090/FR-091)

1. **Archetype existence checked against the merged registry only.** Conforms. FR-090 says "checked when the corpus is merged, against the merged registry of the loaded modules", and FR-090-AC-8 says "an archetype that no loaded module declares". FR-091 says the same. This is narrower than FR-058's `check_declaration_is_live`, which also accepts a kind some document uses. The spec text, not that precedent, governs here.
2. **Missing `direction`, `via.direction` or `rule` is a load failure.** Conforms. "A direction other than incoming or outgoing" and "a rule other than not_below" include absence. A missing `order_from` also fails load. That goes slightly beyond the listed structural defects, because FR-091 sends rank-existence questions to the merge. It is harmless, since without the key there is no rank to check, so it is not raised as a finding.
3. **'Stated' = present and not null, blank, empty array or empty object; carried values only non-null.** Conforms to each requirement's own text. FR-091 required_fields says "absent, null or empty", and FR-090 says "non-null". The two texts differ, so `tier: ""` is carried as `""` and also reported as unassigned. That is a spec-level inconsistency, not a code deviation.
4. **undeclared-field-value per side of a joined pair, once per document, entry name omitted.** Conforms in a single member: "on either side … once per document … naming the document, the field and the value" does not ask for the entry name. Across members it breaks "once per document" (FND-001).
5. **Model-level `exclude` applied to the `from` documents of all three rules, `vocabulary` included.** Not a deviation. FR-091 is silent. CR-060's model-level exclude marks non-traceable data, and `required_relations::obliged` applies it the same way.
6. **Non-document findings located at the reference root and repeated per member view.** Conforms to precedent: `required_relations` reports its `undeclared-relation-vocabulary` the same way. Not a deviation.
7. **A carried_fields merge conflict resets the whole merged traceability model.** Conforms. FR-090 says "fails the merge naming both modules". The loader already fails a merge this way for the obligation-target conflict (`TraceabilityModel::default()`), and the test checks that both modules fail, naming each other and the key.
8. **`repo:id` naming for cross-member targets.** Conforms to the existing convention in `required_relations.rs` (`format!("{repo}:{id}")`).

None of the eight deviates from the merged spec text. FND-001 is a separate defect inside (4).

### Interplay with quire-rs#674 (design_matrix)

The new field is `pub carried: Option<serde_json::Map<String, serde_json::Value>>` on `CoverageMatrixRequirement`. That is exactly the type `DesignMatrixEntry.carried` declares in #674, so `entry_of` can take `requirement.carried.clone()`. #674's design-only entries (no matrix entry) have no requirement to copy from. For those, #674 will need a per-document accessor. Today `coverage::carried_values` is private and walks the whole spec. `git merge-tree` against `plat-1587-engine` conflicts only in `CHANGELOG.md` (adjacent entries) and `src/corpus/mod.rs` (adjacent `pub mod` lines). `src/coverage.rs` merges cleanly.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `trace:undeclared-field-value` is reported once per member view, not once per document, and carries the viewing member's `repo`. `validate_corpus` runs `validate_field_rules` per member view, and `Run.found` is deduplicated only within one view. A target in member B that is reached from member A is reported in A's view, labelled `repo: o/a` with B's path. If B's own documents also reach it, it is reported again under `o/b`. Probe: the two-member corpus a{GLY-001 dim, BRG-001 rests_on ix://o/b/spec/PRI-001}, b{PRI-001 blazing, GLY-002 dim, BRG-002 → PRI-001} gives two identical findings for PRI-001, with repo o/a and repo o/b. FR-091 says "report once per document". Fix: report a target-side outside-enum value only in the target's own member view (or only when `member_of(target) == view member`), and add a two-member test. | src/corpus/field_rules.rs:413-425, 442-449; src/corpus/validate.rs:304-317 |
| FND-002 | low | The `where` on `required_fields` narrows as for `required_relations`, but a dead `where` (a field no `from` schema declares and no document carries, or values outside its enum) is not reported. `check_required_is_live` checks only `from` and `field`. `required_relations` has `check_selector_is_live` (FR-058-AC-22). A misspelt `where` field silently obliges nothing. Fix: run the same selector-liveness check for `required_fields`. | src/corpus/field_rules.rs:344-348 |
| FND-003 | low | In `consistency`, the `compared` flag is redundant. The `let (Standing::Rank(low), Standing::Rank(high), true) = … else { continue }` already refuses any `Outside` side, so `compared` never changes the outcome. A mutation removing `compared = false` is equivalent. Fix: drop the flag and match on the two standings alone. | src/corpus/field_rules.rs:441-454 |
| FND-004 | low | `hop_relation` makes a `RequiredRelation` with a blank `from`, a blank `check` and empty `to`/`exclude`, only to call `RelationEdges::new`, which reads only `edges`. A placeholder value of a domain type stands in for a narrower parameter. Fix: let `RelationEdges::new` (or a sibling constructor) take the verb slice. | src/corpus/relation_edges.rs:378-389 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| src/corpus/field_rules.rs | examined | FND-001, FND-002, FND-003 |
| src/traceability/field_rules.rs (Raw* shapes, validate_field_rules) | examined | Clean; structural checks match FR-090 and FR-091 Ownership |
| src/traceability.rs (model fields, WhereSelector::parse section) | examined | Clean |
| src/corpus/relation_edges.rs (FieldHops, hop_relation) | examined | FND-004 |
| src/coverage.rs carried_values, compute | examined | Clean; reads only the document's own frontmatter |
| src/coverage/reconcile.rs coverage_matrix | examined | Clean |
| src/loader/mod.rs merge_traceability, carried_field_conflict | examined | Clean; reset follows the obligation-target precedent |
| schemas/output/coverage-v1.schema.json | examined | Clean; adds exactly carried and carried_vocabulary |
| CHANGELOG.md | examined | Clean; breaking entry present |
| src/corpus/required_relations.rs obliged, check_selector_is_live | context_only | Precedent for exclude, where and repo:id |
| quire-rs#674 design_matrix diff | context_only | Type matches |

## Verification

`CARGO_PROFILE_DEV_DEBUG=0 locked-build.sh <scratch>/target cargo test --test it field_rules` on `6d569a7`: 27 passed. The cross-member probe was a local, uncommitted test in a scratch worktree, reverted afterwards.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | low | Inserting `carried_for` split a doc comment. The old `carried_values` comment ("Each document's declared carried values … keyed by the scope-relative path …") now sits on top of `carried_for`'s own comment, so `carried_for` documents a path-keyed map it does not return, and `carried_values` has no doc comment. Fix: move the first four lines back above `fn carried_values`. | src/coverage.rs:669-676 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | fb79f77: consistency reads `from` documents corpus-wide (`corpus_documents_of`) and reports an out-of-enum value only when that end is `in_view`, and a comparison only when the `from` end is. The new two-member test `an_out_of_enum_target_in_another_member_is_reported_once_by_its_own_member` asserts one finding with repo o/b and o/b's path. Mutants that drop either gate, or revert to view-only `from` documents, are killed. |
| FND-002 | fixed | fb79f77: `dead_selector` runs from `check_required_is_live` (dead field, or values outside the enum held by no document, trimmed and ASCII folded as FR-058-AC-22). Test `a_where_that_selects_nothing_reports_under_undeclared_relation_vocabulary`; removing the call and removing the fold are both killed. Ruling: acceptable, not a spec deviation. FR-091 says the `where` narrows "exactly as for required_relations" and that field existence reports under `trace:undeclared-relation-vocabulary`. No AC owns it yet (see SR-20261113091250 FND-005). |
| FND-003 | fixed | fb79f77: `compared` removed; the let-else now gates on `from_here`. |
| FND-004 | fixed | fb79f77: `hop_relation` removed; the hops use `RelationEdges::over_verbs(spec, registry, criteria, verbs)`, and `RelationEdges::new` delegates to it. |
